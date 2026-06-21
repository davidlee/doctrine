# SL-131 design: MCP memory retrieval and find tools for agent harnesses

## 1. Writer abstraction

### Problem

Four memory `run_*` functions write to `io::stdout()` directly. The MCP server
also owns `io::stdout()` for JSON-RPC responses — concurrent use causes
interleaving.

### Solution

Add `writer: &mut impl Write` as the first parameter to each function, replacing
`write!(io::stdout(), ...)` with `write!(writer, ...)`:

| Function | File | Signature change |
|---|---|---|
| `run_find` | `retrieve.rs` | `(writer: &mut impl Write, path, paths, globs, …)` |
| `run_retrieve` | `retrieve.rs` | `(writer: &mut impl Write, path, paths, globs, …)` |
| `expand_graph` | `retrieve.rs` | `(writer: &mut impl Write, …)` — module-private, called only by `run_retrieve` |
| `run_show` | `memory.rs` | `(writer: &mut impl Write, path, reference, format)` |
| `run_list` | `memory.rs` | `(writer: &mut impl Write, path, type_f, args)` |

Callers adapt:

| Caller | Pattern |
|---|---|
| CLI (`main.rs`) | `run_find(&mut io::stdout(), …)` — byte-identical |
| MCP (`tools.rs`) | `let mut buf = Vec::new(); run_find(&mut buf, …); let s = String::from_utf8(buf)?;` |

`use std::io::Write` already present in both files. Pipeline logic changes:
limit validation moved into `run_find`/`run_retrieve` so both CLI and MCP paths
are gated identically (see §2 validation).

### D4: Why `impl Write` not `dyn Write`

Static dispatch — only 2 call-side types (stdout and `Vec<u8>`), so monomorphization
is bounded. No vtable overhead per call. Follows existing `write!` macro idiom.

## 2. MCP dispatch unification

### Problem

`call_tool` currently returns `anyhow::Result<ReviewOutput>` — tightly coupled
to the review domain. Memory tools don't produce `ReviewOutput`.

### Solution

Change `call_tool` return type to `anyhow::Result<String>`. The 10 existing
review arms gain a `.map(|out| serde_json::to_string(&out)?)` wrapper around
their `Ok(ReviewOutput::…)`. The 4 new memory arms return the captured writer
buffer as a `String`.

```rust
fn call_tool(…, root: &Path) -> anyhow::Result<String> {
    let (name, arguments) = extract(params)?;
    match name {
        "review_new" => {
            let args: NewArgs = serde_json::from_value(arguments)?;
            let out = review::run_new(root, &args)?;
            Ok(serde_json::to_string(&out)?)    // ← added wrapper
        }
        // … 9 more review arms, same pattern …
        "memory_find" => {
            let fields = ExtractFields::from_value(arguments, &[]);
            let mut buf = Vec::new();
            retrieve::run_find(&mut buf, …)?;
            Ok(String::from_utf8(buf)?)
        }
        "memory_retrieve" => { … }
        "memory_show" => { … }
        "memory_list" => { … }
        _ => anyhow::bail!("Tool not found: {name}"),
    }
}
```

### Validation: CLI gate lives in main.rs, MCP bypasses it

CLI currently validates limits, pages, and `--min-trust` in `main.rs` before
calling the `run_*` functions. MCP calls them directly — bypassing all gates.

**Known gaps:**

| Guard | CLI (main.rs) | MCP (no change) | Impact |
|---|---|---|---|
| `--limit 0` | `bail!("--limit must be >= 1")` | `limit=Some(0)` → `usize::MAX` | runaway results |
| `--limit > MAX` | `.min(RETRIEVE_LIMIT_MAX)` → capped at 20 | passes through verbatim | overhead at large N |
| `--min-trust` bad value | clap `value_parser` → parse error | passes through → default floor | silently wrong |

**Fix: guard `limit=0` in both functions; cap in `run_retrieve`; MCP handler caps `memory_find`.**

Rule: `limit=0` rejected everywhere; no silent unbounded path for retrieve.

```rust
// In run_find:
let limit = match limit {
    Some(0) => anyhow::bail!("--limit must be >= 1"),
    None => None,  // unbounded (CLI default); MCP handler caps separately
    Some(n) => Some(n),  // no cap in run_find — CLI --limit 9999 passes through
};

// In run_retrieve (limit is already a resolved usize):
match limit {
    0 => anyhow::bail!("--limit must be >= 1"),
    n => n.min(RETRIEVE_LIMIT_MAX),  // retrieve always caps at 20
}
```

`run_find` stays unbounded via CLI (`None` passes through as `usize::MAX` in
the pagination `take()`) and explicit CLI `--limit 9999` passes through
unchanged — preserving existing CLI behaviour. The MCP handler applies its
own default cap BEFORE calling the structured helper (`find_for_mcp`,
see §3) — §3 memory_find schema. `run_retrieve` caps at 20 in both CLI
and MCP paths (the retrieve contract always enforces a ceiling).

**MCP `memory_find` with no selectors** — when query/path_scope/glob/command/
tag/type/status are ALL absent AND no explicit `limit` is given in the MCP
handler, default to `limit=20` before calling `run_find`. This prevents
accidental full-corpus dumps while still allowing bounded discovery. If the
caller explicitly supplies `limit: 0`, it's rejected as `--limit must be >= 1`.

For `--min-trust`, the MCP handler calls `retrieve::parse_min_trust` explicitly
and returns `-32602` on failure — no need to move into `run_retrieve` since
it's a pure parser with no side effects.

### handle_tools_call: double-encoding trapdoor

With `call_tool` returning `String` (already serialized JSON), the Ok arm in
`handle_tools_call` must use the string directly — not re-serialize:

```rust
Ok(out) => {
    let tool_result = McpToolResult::text(out);  // String, NOT serde_json::to_string
    let result_val = serde_json::to_value(&tool_result)?;
    JsonRpcResponse::success(id, result_val)
}
```

The old code did `serde_json::to_string(&out)` where `out` was `ReviewOutput`.
If that were applied to an already-serialized `String`, it would produce a
JSON-quoted/escaped string rather than the intended JSON object.

### Error mapping

Memory tools are read-only and stateless. Only three failure modes:
- bad arguments → `-32602` (Invalid params)
- not found / IO error → `-32603` (Internal error)

No structured error enum needed. The existing `map_review_error` fn already
handles generic prefixes before the `ReviewError` downcast:
1. `"Tool not found: …"` → `-32601`
2. `"invalid arguments: …"` → `-32602`
3. `ReviewError` downcast → structured codes (`-32000`, `-32602`)
4. Fallthrough → `-32603` Internal

Memory errors hit this same flow: arg validation errors start with "invalid
arguments" (from `serde_json::from_value` or `anyhow::bail!`) and get `-32602`.
Read/IO/not-found errors fall through to `-32603` with the error message as
the detail. No changes needed to the mapper.

## 2a. Single-memory retrieve with holdback (`retrieve_reference`)

`memory_retrieve(reference: "<uid>")` needs a dedicated path: resolve one memory,
apply the full retrieve contract (lifecycle suppression, draft handling, trust
holdback, staleness, security-framed render), and output a framed block — without
reimplementing pieces scattered across `memory.rs` and `retrieve.rs`.

### Function: `retrieve::retrieve_reference`

```rust
// In src/retrieve.rs — new pub(crate) function
pub(crate) fn retrieve_reference(
    writer: &mut impl Write,
    root: &Path,
    reference: &str,
    include_draft: bool,
    min_trust: Option<&str>,
) -> Result<()> {
    // 1. Collect all memories
    let all = crate::memory::collect_all(root)?;

    // 2. Resolve the reference (uid / key / uid-prefix)
    let mref = crate::memory::MemoryRef::parse(reference)?;
    let memory = crate::memory::resolve_memory_from_all(&all, &mref)?;

    // 3. Freeze snapshot (same as load_query — one clock + git capture)
    let snap = freeze(root);

    // 4. Full retrieve-admission gate: partition, lifecycle, draft,
    //    thread expiry, trust holdback — same contract as scope-based path
    let (ok, reason) = check_retrievable(
        memory, &snap.part, include_draft, min_trust, &snap.today);
    if !ok {
        anyhow::bail!("memory {reference}: {}", reason.unwrap_or_default());
    }

    // 5. Staleness (same computation as the scope-based path)
    let facts = git_facts(root, memory, &snap);
    let st = staleness(memory, facts, &snap.today);

    // 6. Read body
    let body = crate::memory::read_body(root, &memory.uid);

    // 7. Security-framed render (same render_show as run_retrieve's per-block loop)
    let nonce = uuid::Uuid::new_v4().simple().to_string();
    let rendered = crate::memory::render_show(
        memory,
        &body,
        &nonce,
        Some(st.label()),
        &[],  // no wikilinks on single-reference retrieve
    );
    write!(writer, "{rendered}")?;
    Ok(())
}
```

**Visibility bumps needed (cumulative with §2b):**
- `memory::resolve_memory_from_all` → `pub(crate)` (currently private)
- `staleness` → `pub(crate)` (currently private)
- `git_facts` → `pub(crate)` (currently private; called by `retrieve_reference`)

Already `pub(crate)`: `collect_all`, `MemoryRef`, `MemoryRef::parse`, `read_body`,
`render_show`, `freeze`, `base_filter`, `thread_expiry`, `check_retrievable`.

## 2b. Retrievability helper (`check_retrievable`)

Three surfaces need the same retrieve-admission check — `retrieve_reference`
(pre-render gate), `memory_show` (`consumable`/`notes` flags), and the normal
`run_retrieve` pipeline (partition + thread + holdback in `query()` then
`held_back`). Factor once so the direct-reference path cannot bypass gates
that the scope-based path enforces:

```rust
// In src/retrieve.rs
/// Returns (admitted, reason_if_blocked). Applies the full retrieve contract
/// in order: workspace/repo partition, lifecycle + draft, thread expiry,
/// trust holdback. This is the sole admission gate — no parallel logic.
pub(crate) fn check_retrievable(
    m: &crate::memory::Memory,
    part: &QueryPartition,
    include_draft: bool,
    min_trust: Option<&str>,
    today: &str,
) -> (bool, Option<String>) {
    // 1. Workspace/repo partition + lifecycle + draft — same gate as query()
    if !base_filter(m, part, include_draft) {
        let reason = match m.status {
            Status::Quarantined => "quarantined",
            Status::Retracted   => "retracted",
            Status::Archived    => "archived",
            Status::Superseded  => "superseded",
            Status::Draft       => "draft",
            Status::Active      => {
                if m.scope.workspace != part.workspace { "wrong workspace" }
                else { "wrong repo" }
            }
        };
        return (false, Some(reason.to_owned()));
    }
    // 2. Thread expiry — same gate as query() (ScopeMatch arg is vestigial)
    if !thread_expiry(m, ScopeMatch::of(Dimension::Paths), today) {
        return (false, Some("thread expired (unverified or stale)".to_owned()));
    }
    // 3. Trust holdback — same gate as run_retrieve's pre-render filter
    let floor = holdback_floor(min_trust);
    if held_back(m, floor) {
        return (false, Some("held back (low trust ∧ high severity)".to_owned()));
    }
    (true, None)
}
```

Used by:
- **`retrieve_reference`** — calls `check_retrievable` as the single pre-render
gate, then computes staleness + reads body + renders.
- **`memory_show` MCP handler** — calls `check_retrievable` to set `consumable`,
`held_back_on_retrieve`, and `notes`.
- **`find_for_mcp` rows** — `find_for_mcp` results have already passed
`base_filter` + `thread_expiry` via `query()`, so only the holdback check is
needed for the `held_back_on_retrieve` flag: `held_back(m, holdback_floor(None))`.
No duplicate gate logic.

**Visibility bumps needed:**
- `holdback_floor` → `pub(crate)` (currently private)
- `held_back` → `pub(crate)` (currently private)

`base_filter`, `thread_expiry`, `ScopeMatch`, `Dimension`, `QueryPartition` are
already `pub(crate)`.

## 3. Tool definitions

### Structured MCP helpers (not CLI `run_*`)

The CLI `run_find` / `run_list` render to a writer and return `()`. The MCP
tools need structured data with pagination metadata (`total`, `offset`, `limit`,
`next_offset`) plus enriched fields (`key`, `held_back_on_retrieve`). Rather
than post-processing rendered strings or changing the CLI JSON contract,
add MCP-specific structured helpers that reuse the existing `load_query` →
`query` pipeline (no parallel implementation):

**`retrieve::find_for_mcp`** — returns rows + total, consumed by the
`memory_find` MCP handler which builds the pagination envelope.

```rust
// In src/retrieve.rs
pub(crate) struct FindForMcp {
    pub(crate) rows: Vec<serde_json::Value>,
    pub(crate) total: usize,
}

pub(crate) fn find_for_mcp(
    path: Option<PathBuf>,
    paths: Vec<String>,
    globs: Vec<String>,
    commands: Vec<String>,
    tags: Vec<String>,
    lifespan: Option<Lifespan>,
    free_query: Option<String>,
    type_f: Option<MemoryType>,
    status_f: Option<Status>,
    include_draft: bool,
    offset: usize,
    limit: Option<usize>,
) -> Result<FindForMcp> {
    let loaded = load_query(path, paths, globs, commands, tags,
        lifespan, free_query, type_f, status_f)?;
    let ranker = Bm25Ranker;
    let ranked = query(&loaded.mems, &loaded.q, &loaded.snap,
        include_draft, &loaded.root, &ranker);
    let total = ranked.len();
    // None = unbounded (handler applies its own cap); 0 = rejected.
    if limit == Some(0) { anyhow::bail!("--limit must be >= 1"); }
    let cap = limit.unwrap_or(usize::MAX);
    let visible: Vec<&Candidate<'_>> = ranked.iter()
        .skip(offset).take(cap).collect();
    let rows: Vec<serde_json::Value> = visible.iter().map(|c| {
        let m = c.memory;
        // find_for_mcp results already passed base_filter + thread_expiry
        // via query(); only the holdback check is needed for the flag.
        let held = held_back(m, holdback_floor(None));
        json!({
            "uid": m.uid,
            "key": m.key,
            "type": m.kind.as_str(),
            "status": m.status.as_str(),
            "staleness": c.staleness.label(),
            "trust": crate::memory::scrub_line(&m.trust_level),
            "severity": crate::memory::scrub_line(&m.severity),
            "spec": c.scope_match.map_or("-", |s| s.dim.label()),
            "title": crate::memory::scrub_line(&m.title),
            "held_back_on_retrieve": held,
        })
    }).collect();
    Ok(FindForMcp { rows, total })
}
```

**`memory::list_for_mcp`** — returns rows + total with pagination, consumed by
the `memory_list` MCP handler.

```rust
// In src/memory.rs — factored shared filtered-list helper
/// Returns all memories matching the standard filter + type axis, in default
/// sort order. Used by both list_rows (CLI) and list_for_mcp (MCP).
pub(crate) fn filtered_list(
    root: &Path,
    type_f: Option<MemoryType>,
    filter: &crate::listing::Filter,
) -> Result<Vec<Memory>> {
    let mut rows = crate::listing::retain(collect_all(root)?, filter, is_hidden, key);
    rows.retain(|m| type_f.is_none_or(|t| m.kind == t));
    sort_default(&mut rows);
    Ok(rows)
}
```

`list_rows` delegates to `filtered_list` for the core pipeline, then formats
(Table or Json). `list_for_mcp` calls `filtered_list`, paginates, and returns
`ListForMcp`. Zero duplication — the full filter contract (`listing::retain`:
substr over key+title, status validation, default hide-set, tag OR-match)
is shared.

**`memory::list_for_mcp`** — thin pagination wrapper over `filtered_list`:

```rust
// In src/memory.rs
pub(crate) struct ListForMcp {
    pub(crate) rows: Vec<serde_json::Value>,
    pub(crate) total: usize,
}

pub(crate) fn list_for_mcp(
    root: &Path,
    type_f: Option<MemoryType>,
    substr: Option<&str>,
    status: &[String],
    tags: &[String],
    offset: usize,
    limit: usize,  // resolved by handler (default 50, 0 → usize::MAX)
) -> Result<ListForMcp> {
    // Validate statuses (same gate as list_rows — listing::build doesn't)
    listing::validate_statuses(status, MEMORY_STATUSES)?;
    let args = crate::listing::ListArgs {
        substr: substr.map(str::to_owned),
        status: status.to_vec(),
        tags: tags.to_vec(),
        ..Default::default()
    };
    let (filter, _format) = crate::listing::build(args)?;
    let rows = filtered_list(root, type_f, &filter)?;
    let total = rows.len();
    let page: Vec<serde_json::Value> = json_rows(
        &rows.into_iter().skip(offset).take(limit).collect::<Vec<_>>()
    ).into_iter()
      .map(|r| serde_json::to_value(r).unwrap())
      .collect();
    Ok(ListForMcp { rows: page, total })
}
```

`DEFAULT_MEMORY_LIST_LIMIT = 50`. `filtered_list` delegates to `listing::retain`
for the full filter contract. `list_for_mcp` validates statuses via
`listing::validate_statuses` then builds the filter through `listing::build`
(substr lowercasing, regex compilation) — the same path as `list_rows`.

### ExtractFields extension

Add `opt_bool_field(name: &str) -> Option<bool>` to the existing `ExtractFields`
helper — needed for the `include_draft` boolean flag. Parses JSON `true`/`false`
via `serde_json::Value::as_bool`. Returns `None` for missing or non-boolean.

### MCP handler: memory_find

The handler calls `retrieve::find_for_mcp` (structured), then builds the
pagination envelope with `total`/`offset`/`limit`/`next_offset`:

```rust
"memory_find" => {
    let fields = ExtractFields::from_value(arguments, &[]);
    let limit = fields.opt_usize_field("limit");
    let has_selectors = fields.opt_str_field("query").is_some()
        || !fields.vec_str_field("path_scope").is_empty()
        || !fields.vec_str_field("glob").is_empty()
        || !fields.vec_str_field("command").is_empty()
        || !fields.vec_str_field("tag").is_empty()
        || fields.opt_str_field("type").is_some()
        || fields.opt_str_field("status").is_some()
        || fields.opt_str_field("lifespan").is_some();
    // No selectors + no explicit limit → default cap of 20
    let effective_limit = if !has_selectors && limit.is_none() {
        Some(20usize)
    } else {
        limit
    };
    let result = retrieve::find_for_mcp(
        Some(root.to_path_buf()),
        fields.vec_str_field("path_scope"),
        fields.vec_str_field("glob"),
        fields.vec_str_field("command"),
        fields.vec_str_field("tag"),
        parse_lifespan(fields.opt_str_field("lifespan"))?,
        fields.opt_str_field("query"),
        parse_memory_type(fields.opt_str_field("type"))?,
        parse_status(fields.opt_str_field("status"))?,
        fields.opt_bool_field("include_draft").unwrap_or(false),
        fields.opt_usize_field("offset").unwrap_or(0),
        effective_limit,
    )?;
    let offset = fields.opt_usize_field("offset").unwrap_or(0);
    let cap = effective_limit.unwrap_or(result.total);
    let next_offset = if offset + cap < result.total {
        Some(offset + cap)
    } else {
        None
    };
    Ok(serde_json::to_string_pretty(&json!({
        "kind": "memory_find",
        "rows": result.rows,
        "total": result.total,
        "offset": offset,
        "limit": cap,
        "next_offset": next_offset,
    }))?)
}
```

Three shared parse helpers (module-private in tools.rs):

| Helper | Input | Output | Unsafe? |
|---|---|---|---|
| `parse_memory_type(s: Option<String>)` | `"concept"` | `Some(MemoryType::Concept)` | None → None; bad value → bail |
| `parse_status(s: Option<String>)` | `"active"` | `Some(Status::Active)` | None → None; bad value → bail |
| `parse_lifespan(s: Option<String>)` | `"semantic"` | `Some(Lifespan::Semantic)` | None → None; bad value → bail |

**Error-wrapping requirement:** The parse helpers call `FromStr` or internal
parsers that produce errors like `"unknown lifespan {other:?}"`. These do NOT
start with `"invalid arguments: "`, so the MCP error mapper's prefix check
(`§2 Error mapping`, branch 2) would not match them, and they'd fall through
to `-32603` (Internal) instead of the correct `-32602` (Invalid params). Each
helper must wrap the inner error:

```rust
fn parse_lifespan(s: Option<String>) -> Result<Option<Lifespan>> {
    s.map(|v| Lifespan::from_str(&v)
        .map_err(|e| anyhow::anyhow!("invalid arguments: {e}")))
     .transpose()
}
```

The `"invalid arguments: "` prefix is load-bearing — see `map_review_error`
branch 2 in `tools.rs`. The same pattern applies to `parse_memory_type` and
`parse_status`.

### MCP handler: memory_retrieve

Two branches: `reference` present → `retrieve::retrieve_reference` (single-memory
through full holdback); else → `retrieve::run_retrieve` (scope-based search).
Mutual exclusivity enforced: `reference` with any query/scope probe → `-32602`.

```rust
"memory_retrieve" => {
    let fields = ExtractFields::from_value(arguments, &[]);
    let reference = fields.opt_str_field("reference");
    let include_draft = fields.opt_bool_field("include_draft").unwrap_or(false);

    // Validate min_trust before use — holdback_floor silently defaults on bad input
    let min_trust_str = fields.opt_str_field("min_trust");
    let min_trust = min_trust_str.as_deref().map(|s| {
        retrieve::parse_min_trust(s)
            .map_err(|e| anyhow::anyhow!("invalid arguments: {e}"))
    }).transpose()?;

    if let Some(ref_str) = reference {
        // Validate mutual exclusivity: reference alone, no probes
        let has_probes = fields.opt_str_field("query").is_some()
            || !fields.vec_str_field("path_scope").is_empty()
            || !fields.vec_str_field("glob").is_empty()
            || !fields.vec_str_field("command").is_empty()
            || !fields.vec_str_field("tag").is_empty()
            || fields.opt_str_field("type").is_some()
            || fields.opt_str_field("status").is_some()
            || fields.opt_str_field("lifespan").is_some();
        if has_probes {
            anyhow::bail!("invalid arguments: reference is mutually exclusive with query/path_scope/glob/command/tag/type/status/lifespan");
        }
        // Single-memory path: resolve → check_retrievable → staleness → render
        let mut buf = Vec::new();
        retrieve::retrieve_reference(
            &mut buf,
            root,
            &ref_str,
            include_draft,
            min_trust.as_deref(),
        )?;
        Ok(String::from_utf8(buf)?)
    } else {
        // Scope-based path: search → rank → holdback → framed blocks
        let mut buf = Vec::new();
        retrieve::run_retrieve(
            &mut buf,
            Some(root.to_path_buf()),
            fields.vec_str_field("path_scope"),
            fields.vec_str_field("glob"),
            fields.vec_str_field("command"),
            fields.vec_str_field("tag"),
            parse_lifespan(fields.opt_str_field("lifespan"))?,
            fields.opt_str_field("query"),
            parse_memory_type(fields.opt_str_field("type"))?,
            parse_status(fields.opt_str_field("status"))?,
            include_draft,
            fields.opt_usize_field("limit").unwrap_or(RETRIEVE_LIMIT_DEFAULT),
            min_trust.as_deref(),
            fields.opt_usize_field("offset").unwrap_or(0),
            crate::listing::Format::Table,
            None,  // expand (deferred per scope)
        )?;
        Ok(String::from_utf8(buf)?)
    }
}
```

### Hardcoded format per tool

| Tool | Format | Rationale |
|---|---|---|
| `memory_find` | JSON | Structured ranked rows — agent parses and selects candidates |
| `memory_retrieve` | Table | Security-framed data blocks with nonce/staleness — SPEC-007's "data, not instruction" render |
| `memory_show` | JSON | Full memory header + body + resolved wikilinks + relations + backlinks |
| `memory_list` | JSON | Index rows — agent parses and selects |

No `format` parameter exposed to the MCP caller. The format is baked per tool.

### Result contract: JSON inside text content, not structured content

The MCP protocol layer is unchanged — all tool results are returned as
`content: [{type: "text", text: "<JSON string>"}]`. The text content is
parseable JSON, but the MCP protocol itself does not carry an `outputSchema`
or structured content type. This is identical to how the 10 existing review
tools return their results. The skill updates' claim of "structured JSON"
means "machine-parseable JSON inside the text content" — the agent parses
`content[0].text` as JSON.

### Tool descriptions (agent guidance, not just metadata)

| Tool | Description (model-facing) |
|---|---|
| `memory_find` | "Discovery tool — metadata only, no bodies. Use first to probe context. Holdback-exempt: rows may include memories suppressed by `memory_retrieve`. Do not treat high-risk rows as consumable knowledge; use `memory_show` for inspection then `memory_retrieve` for safe recall. Requires at least one selector or defaults to 20-row cap." |
| `memory_retrieve` | "Agent-context recall with trust holdback. Returns security-framed data blocks (nonce + staleness + attribution). Low-trust ∧ high-severity memories are suppressed. Use after `memory_find` identified relevant candidates. Supply `reference` for single-memory recall through holdback." |
| `memory_show` | "Full memory inspection — header, body, relations, wikilinks, backlinks. Use only after selecting an exact uid via `memory_find`. For token efficiency, use `view: summary` to skip body, or `include_body: false`. Held-back memories (field `held_back_on_retrieve: true`) are shown with a metadata warning; do not treat as consumable knowledge." |
| `memory_list` | "Browse/index only — all memories, newest first, capped at 50 by default. Prefer scoped `memory_find` for targeted discovery." |

### Schema definitions

All 4 added to the `tools()` vec alongside the existing 10 review tools.

Tool: `memory_find`

```
memory_find(query?, path_scope[], glob[], command[], tag[],
            type?, status?, lifespan?, include_draft?,
            offset?, limit?)

  Lifespan enum: semantic | episodic | procedural | working | identity
  Type enum: concept | fact | pattern | signpost | system | thread
  Status enum: active | draft | superseded | retracted | archived | quarantined

  → JSON envelope with pagination metadata and per-row heldback flag:
  {
    "kind": "memory_find",
    "rows": [
      { "uid": "…", "key": "mem.pattern…", "type": "pattern",
        "status": "active", "staleness": "fresh",
        "trust": "high", "severity": "medium",
        "spec": "paths", "title": "…",
        "held_back_on_retrieve": false }
    ],
    "total": 42,        // pre-pagination candidate count
    "offset": 0,
    "limit": 20,
    "next_offset": 20
  }
```

Behaviour: when no selectors AND no explicit `limit`, defaults `limit` to 20.
Explicit `limit: 0` is rejected as `--limit must be >= 1`.

Tool: `memory_retrieve`

```
memory_retrieve(reference?, query?, path_scope[], glob[], command[], tag[],
                type?, status?, lifespan?, include_draft?,
                offset?, limit?, min_trust?)

  Lifespan: same as find; Type: same; Status: same
  min_trust: high | medium | low

  → Table: security-framed data blocks with nonce, staleness,
    attribution — same as doctrine memory retrieve
```

Behaviour: `reference` is a uid or key, mutually exclusive with query/scope
probes. When present, resolves that single memory and renders it through the
trust holdback + security-framed output — the safe exact-body path for agents
that selected a candidate via `memory_find`. `limit` defaults to 5, capped at
20. `min_trust` default medium.

Tool: `memory_show`

```
memory_show(reference!,
            view?,            // "summary" (default) | "full"
            include_body?,    // true (default) | false
            backlinks_limit?) // max backlinks to return (default 20, 0 = unlimited)

  → JSON: {
    "kind": "memory",
    "memory": {
      uid, key, type, …,          // same as show_json
      "held_back_on_retrieve": true,  // flagged if retrieve would suppress this
      "consumable": false,            // false when check_retrievable fails
      relations: […],
      wikilinks: […],
      backlinks: [ … ],
      backlinks_total: 13
    },
    "body": "…",
    "notes": "This memory is …"  // present when !consumable, explains why
  }
```

Default `view: summary` — body excluded by default. Use `view: full` to
include body. This is the PoLS choice: agents discover and inspect via
summary, then safely consume the exact body via
`memory_retrieve(reference)` (which goes through the trust holdback and
security-framed render).

`consumable` is false when `check_retrievable(memory, &snap.part, false, None, &snap.today)`
returns `(false, reason)` — i.e. partition mismatch, lifecycle blocked, thread expired,
or trust holdback suppressed. The `notes` field carries the reason string.

**Safe exact-body path:** For agents that have selected a candidate uid via
`memory_find` and want the body, the recommended path is
`memory_retrieve(reference: "<uid>")` — the `reference` parameter resolves
that single memory and renders it through the trust holdback + security-framed
output. This is cleaner than calling `memory_show` with `view: full` on a
held-back memory, because `memory_retrieve` enforces the holdback pre-render
before the body is read or framed.

Tool: `memory_list`

```
memory_list(type?, substr?, status[], tag[], limit?, offset?)

  → JSON envelope:
  {
    "kind": "memory",
    "rows": [
      { "uid": "…", "key": "…", "type": "pattern",
        "status": "active", "trust": "high", "title": "…" }
    ],
    "total": 200,
    "limit": 50,
    "next_offset": null  // null when all rows returned (no more pages)
  }
```

Behaviour: defaults `limit` to 50. Use `limit: 0` for all.

### MCP handler: memory_list

Calls `memory::list_for_mcp` (structured), then builds the pagination envelope:

```rust
"memory_list" => {
    let fields = ExtractFields::from_value(arguments, &[]);
    // Resolve limit before passing: default 50, 0 = all (unbounded)
    let limit_raw = fields.opt_usize_field("limit");
    let limit = match limit_raw {
        Some(0) => usize::MAX,
        None => 50,
        Some(n) => n,
    };
    let result = memory::list_for_mcp(
        root,
        parse_memory_type(fields.opt_str_field("type"))?,
        fields.opt_str_field("substr").as_deref(),
        &fields.vec_str_field("status"),
        &fields.vec_str_field("tag"),
        fields.opt_usize_field("offset").unwrap_or(0),
        limit,
    )?;
    let offset = fields.opt_usize_field("offset").unwrap_or(0);
    let next_offset = if offset + limit < result.total {
        Some(offset + limit)
    } else {
        None
    };
    Ok(serde_json::to_string_pretty(&json!({
        "kind": "memory",
        "rows": result.rows,
        "total": result.total,
        "offset": offset,
        "limit": if limit == usize::MAX { result.total } else { limit },
        "next_offset": next_offset,
    }))?)
}
```


### Backlink computation cost

Building the backlink index scans the full memory corpus (one `collect_all` per
`memory_show` call, shared between `check_retrievable` + `backlink_rows_for` —
no double scan). Acceptable for typical corpus sizes (<1000
memories, <100ms). If the corpus grows past 10k items, this should be cached or
moved to build-on-record. For now, every `memory_show` call pays this cost, same
as `doctrine memory backlinks <REF>`.

## 4. Backlinks enrichment in `memory_show`

The MCP handler for `memory_show` post-processes the JSON output from
`run_show` to inject a `backlinks` array:

```json
{
  "kind": "memory",
  "memory": {
    …existing show_json fields…,
    "relations": […],
    "wikilinks": […],
    "backlinks": [
      { "uid": "mem_xxx", "title": "…", "method": "wikilink|relation" }
    ]
  },
  "body": "…"
}
```

**DRY constraint:** The existing `memory::run_backlinks` already contains the
full backlink pipeline — target normalization, uid-prefix resolution,
wikilink-vs-relation distinction. Do not duplicate it in `tools.rs`.

Instead, factor a pure helper from `run_backlinks`'s internals. It accepts
pre-collected memories so callers that also need the `Memory` for
`check_retrievable` + `backlink_rows_for` avoid a double `collect_all` scan:

```rust
// In memory.rs — factored from run_backlinks
pub(crate) struct BacklinkRow {
    pub(crate) uid: String,
    pub(crate) memory_type: String,
    pub(crate) title: String,
    pub(crate) method: String,  // "wikilink" | actual relation label
}

pub(crate) fn backlink_rows_for(
    root: &Path,
    all: &[Memory],
    uid: &str,
) -> Result<Vec<BacklinkRow>> {
    // build wikilink + relation maps (with method provenance) from `all`,
    // call crate::links::backlinks_index, filter to uid targets,
    // resolve source memory titles, return Vec<BacklinkRow>
}
```

`run_backlinks` delegates to this helper for the core computation, then
formats/renders. The MCP `memory_show` handler also calls it — see below. Zero
duplication — the method provenance (wikilink vs relation label) is preserved
from the source.

### MCP memory_show handler

The handler does one `collect_all` scan, then enriches in two passes:

1. Call `run_show(&mut buf, root, reference, Format::Json)` — get JSON
2. Deserialize, extract `uid`
3. `let all = memory::collect_all(root)?;` — one scan
4. Resolve the `Memory`: `memory::resolve_memory_from_all(&all, &mref)?`
5. Freeze snapshot: `let snap = retrieve::freeze(root);`
6. Call `retrieve::check_retrievable(memory, &snap.part, false, None, &snap.today)` → set `consumable`,
   `held_back_on_retrieve`, `notes`
7. Call `memory::backlink_rows_for(root, &all, &uid)` — get typed backlinks
8. Apply `backlinks_limit` cap, inject `backlinks` array + `backlinks_total`
9. Re-serialize enriched JSON

Zero changes to `show_json` or `run_show`. The `backlink_rows_for` signature
change (`all: &[Memory]` instead of `root: &Path`) is the sole API shift —
callers that already have collected memories pass them in; `run_backlinks`
does `collect_all` first then delegates.

## 5. Skill surface updates

`retrieve-memory/SKILL.md` is already updated (from a previous iteration;
see below). `audit/SKILL.md` and `reviewing-memory/SKILL.md` are listed here
for application during implementation:

### `retrieve-memory/SKILL.md`

New preamble after the heading, before `## Two surfaces`:

```
## Tool preference

If your harness supports MCP tools and doctrine's MCP server is connected
(you see `memory_find`, `memory_retrieve`, `memory_show`, `memory_list` in
your tool list), **prefer these MCP tools over the CLI** — they return
machine-parseable JSON text in the MCP content block without spawning a
shell, and `memory_show` enriches results with resolved backlinks.

When MCP tools are not available (e.g. in a plain shell environment),
fall back to the `doctrine memory` CLI commands described below.
```

### `reviewing-memory/SKILL.md`

Single-line note at the top, after the heading:

> **MCP shortcut:** If the doctrine MCP server is connected, use `memory_show`
> via MCP tool instead of `doctrine memory show` for machine-parseable JSON results with
> backlinks.

### `audit/SKILL.md`

New preamble after the heading, before `## Audit lens`:

```
## Tool preference

If your harness supports MCP tools and doctrine's MCP server is connected
(you see `review_new`, `review_raise`, `review_dispose`, `review_verify`,
`review_prime`, `review_list`, `review_show`, `review_status` in your tool
list), **prefer these MCP tools over the CLI** — they return machine-parseable
JSON text in the MCP content block and eliminate shell overhead. Every review verb has
an MCP equivalent.
```

## 6. Code impact summary

| Path | Change |
|---|---|
| `src/retrieve.rs` | Add `writer: &mut impl Write` param to `run_find`, `run_retrieve`, `expand_graph`. Replace `write!(io::stdout(), …)` → `write!(writer, …)`. Add `retrieve_reference(writer, root, reference, include_draft, min_trust)` — delegates to `check_retrievable` for the full gate. Add `find_for_mcp(…) -> FindForMcp` — structured find reusing `load_query`→`query`, with `key`/`held_back_on_retrieve` fields. Add `check_retrievable(m, part, include_draft, min_trust, today) -> (bool, Option<String>)` — partition + lifecycle + thread expiry + holdback in one gate. Bump `holdback_floor`, `held_back`, `staleness`, `git_facts` to `pub(crate)`. |
| `src/memory.rs` | Add `writer: &mut impl Write` param to `run_show`, `run_list`. Replace `write!(io::stdout(), …)` → `write!(writer, …)`. Add `filtered_list(root, type_f, filter) -> Vec<Memory>` — shared filtered-list helper used by both `list_rows` (CLI) and `list_for_mcp` (MCP), reusing `listing::retain`. Add `list_for_mcp(…) -> ListForMcp` — paginated wrapper over `filtered_list`. Refactor `backlink_rows_for(root, all: &[Memory], uid) -> Vec<BacklinkRow>` — accepts pre-collected memories so callers avoid double `collect_all`. `BacklinkRow` + fields `pub(crate)`. Bump `resolve_memory_from_all` to `pub(crate)`. `list_rows` delegates to `filtered_list` (internal refactor, no behaviour change). |
| `src/main.rs` | 4 CLI call sites pass `&mut io::stdout()` as first arg. |
| `src/mcp_server/tools.rs` | Add 4 tool definitions with agent-facing descriptions. Add 4 `call_tool` match arms. `memory_find`: `find_for_mcp` + pagination envelope. `memory_retrieve`: validate mutual exclusivity (`reference` vs probes → `-32602`) + validate `min_trust` via `parse_min_trust` before branch; ref branch → `retrieve_reference`, scope branch → `run_retrieve`. `memory_show`: `run_show` JSON → deserialize → `collect_all` + `freeze` once → resolve Memory → `check_retrievable` + `backlink_rows_for` → enrich + re-serialize. `memory_list`: `list_for_mcp` + pagination envelope. Change `call_tool` return type to `String`. Wrap 10 review arms with `.map(|o| serde_json::to_string(&o)?)`. Change `handle_tools_call` Ok arm to `McpToolResult::text(out)` directly. Add `opt_bool_field` to `ExtractFields`. Add `parse_lifespan`, `parse_memory_type`, `parse_status` helpers. |
| `src/retrieve.rs` | Move `limit=0` reject into `run_retrieve`; cap at `RETRIEVE_LIMIT_MAX` in `run_retrieve` only (not `run_find`). |
| `src/mcp_server/protocol.rs` | Unchanged. |
| `src/mcp_server/mod.rs` | Unchanged. |
| `plugins/doctrine/skills/retrieve-memory/SKILL.md` | Add MCP tool preference preamble. |
| `plugins/doctrine/skills/reviewing-memory/SKILL.md` | Add MCP shortcut note. |
| `plugins/doctrine/skills/audit/SKILL.md` | Add MCP tool preference preamble. |

## 7. Verification alignment

### Existing evidence (must stay green, unchanged)

- All `retrieve.rs` tests (search pipeline, ranking, scope matching, trust
  holdback, staleness computation, run_find, run_retrieve)
- All `memory.rs` tests (show, list, record, verify, edit, tag, status,
  resolve-links, backlinks)
- All `mcp_server` tests (protocol round-trips, review tool dispatch)
- All CLI integration tests for `doctrine memory *` and `doctrine review *`

### New evidence

**Writer capture tests** (direct, not via CLI):
- `retrieve::run_find(&mut Vec<u8>, …)` writes expected JSON envelope
- `retrieve::run_retrieve(&mut Vec<u8>, …)` respects limit cap (9999 → 20)
- `retrieve::run_retrieve(&mut Vec<u8>, …)` rejects `limit=0`
- `retrieve::run_retrieve(&mut Vec<u8>, …)` respect trust holdback
- `memory::run_show(&mut Vec<u8>, …)` writes expected JSON shape
- `memory::run_list(&mut Vec<u8>, …)` writes expected JSON envelope

**MCP dispatch tests:**
- `memory_find` with no args returns capped 20 rows with pagination metadata (no selector required)
- `memory_find` with path/glob scope returns scoped results with pagination metadata
- `memory_find` rows include `key` and `held_back_on_retrieve` fields
- `memory_find` response includes `total`, `offset`, `limit`, `next_offset`
- `memory_retrieve` returns security-framed table blocks
- `memory_retrieve` with `min_trust: "high"` suppresses low-trust high-severity
- `memory_retrieve` with `limit: 9999` is capped to 20
- `memory_retrieve` with `limit: 0` returns error (`-32602`)
- `memory_retrieve` with `min_trust: "banana"` returns error (`-32602`)
- `memory_retrieve` with `reference` + query probe returns error (`-32602`, mutual exclusivity)
- `memory_retrieve` with `reference` alone returns security-framed block
- `memory_retrieve` with `reference` to quarantined memory returns error (check_retrievable gate)
- `memory_retrieve` with `reference` to draft memory without `include_draft` returns error
- `memory_show` with valid uid returns JSON with `held_back_on_retrieve`, `consumable`, `notes` fields
- `memory_show` with `view: summary` excludes body
- `memory_show` with `include_body: false` excludes body (metadata + links only)
- `memory_show` with `backlinks_limit: 5` returns at most 5 backlinks + `backlinks_total`
- `memory_show` with invalid uid returns error
- `memory_show` response `content[0].text` parses as JSON object (not quoted string)
- `memory_list` defaults to 50-row cap with pagination metadata
- `memory_list` with `limit: 0` returns full index

**Review MCP compat:**
- All 10 review MCP tools produce byte-identical responses after `call_tool → String`
- Review MCP response `content[0].text` parses as JSON object (double-encoding guard)

**Test updates:**
- VT-3 (`tool_list_has_10_tools`) updated to 14
- VT-3 tool list names test updated to include 4 new names

**Gates:**
- `cargo clippy` zero warnings
- `just check` green

Test style: unit-level writer-capture tests for `run_*` function changes;
integration-level MCP request/response cycle via `doctrine serve --mcp` against
the repo's own corpus for the dispatch tests.
