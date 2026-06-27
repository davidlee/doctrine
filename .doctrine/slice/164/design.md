# SL-164 Design — Memory write verbs + onboarding in MCP

## Architecture overview

### Current behaviour
- MCP server exposes 15 tools: 10 review (7 write + 3 read) and 5 memory (all read)
- No MCP surface for memory write verbs — agents fall back to `bash`
- Boot snapshot hardcodes `/retrieving-memory` for 2 signpost memories; no MCP-aware path

### Target behaviour
- 3 new MCP tools: `memory_record`, `memory_edit`, `doctrine_onboard` → 15 → 18
- `memory_record` + `memory_edit`: dispatch to existing `memory::run_*` functions,
  following the review write-tool pattern
- `doctrine_onboard`: self-describing tool returning CLI→MCP mapping + 2 bundled memory
  bodies, so an MCP agent discovers mappings without parsing skill files
- Boot footer updated to MCP-first with `/retrieving-memory` fallback
- No new memory verb functionality — ride existing engine seams

### Module boundaries (ADR-001)
- **Command tier** (`src/mcp_server/tools.rs`): 3 new `McpTool` definitions, `call_tool`
  dispatch arms, `doctrine_onboard` markdown renderer
- **Engine tier** (`src/memory.rs`): minimal change — add `writer: &mut dyn Write`
  parameter to `run_record` and `run_edit` (aligns with existing `run_show`/`run_list`
  pattern). CLI call sites pass `&mut io::stdout()`; MCP passes a `Vec<u8>` buffer.
- **Authored** (`.doctrine/boot-footer.md`): prose update only
- `src/mcp_server/protocol.rs`: no changes needed

### Error mapping
- Reuse the existing `map_review_error` function in `tools.rs`. It already handles:
  1. `"Tool not found: "` prefix → `-32601`
  2. `"invalid arguments: "` prefix → `-32602` (Invalid params)
  3. `ReviewError` downcast by variant identity → structured codes
  4. Catch-all → `-32603`
- Memory dispatch arms wrap argument-level errors with `"invalid arguments: "` prefix
  so they hit branch 2. Non-argument errors fall through to `-32603`.
- No new error enum needed.

---

## `memory_record` tool

### JSON Schema

```json
{
  "type": "object",
  "properties": {
    "title":       { "type": "string", "description": "Memory title (required)" },
    "memory_type": { "type": "string", "enum": ["concept","fact","pattern","signpost","system","thread"], "description": "Memory kind (required)" },
    "key":         { "type": "string", "description": "Optional durable key (e.g. mem.pattern.cli.skinny)" },
    "summary":     { "type": "string", "description": "One-line summary" },
    "trust_level": { "type": "string", "enum": ["high","medium","low"], "description": "Trust level (default: medium)" },
    "severity":    { "type": "string", "enum": ["high","medium","low"], "description": "Severity (default: medium)" },
    "tags":        { "type": "array", "items": { "type": "string" }, "description": "Tags" },
    "paths":       { "type": "array", "items": { "type": "string" }, "description": "File path scopes" },
    "globs":       { "type": "array", "items": { "type": "string" }, "description": "Glob scopes" },
    "commands":    { "type": "array", "items": { "type": "string" }, "description": "Command scopes" },
    "lifespan":    { "type": "string", "enum": ["semantic","episodic","procedural","working","identity"], "description": "Lifespan threshold" },
    "status":      { "type": "string", "enum": ["active","draft","superseded","retracted","archived","quarantined"], "description": "Initial status (default: active)" },
    "repo":        { "type": "string", "description": "Explicit repo identity override" },
    "global":      { "type": "boolean", "description": "Record as global orientation master" }
  },
  "required": ["title", "memory_type"]
}
```

### Dispatch

```rust
"memory_record" => {
    #[derive(Deserialize)]
    struct RecordParams {
        title: String,
        memory_type: String,
        key: Option<String>,
        summary: Option<String>,
        trust_level: Option<String>,
        severity: Option<String>,
        tags: Option<Vec<String>>,
        paths: Option<Vec<String>>,
        globs: Option<Vec<String>>,
        commands: Option<Vec<String>>,
        lifespan: Option<String>,
        status: Option<String>,
        repo: Option<String>,
        global: Option<bool>,
    }
    let p: RecordParams = serde_json::from_value(arguments)
        .map_err(|e| anyhow::anyhow!("invalid arguments: {e:#}"))?;
    let memory_type = MemoryType::parse(&p.memory_type)
        .map_err(|e| anyhow::anyhow!("invalid arguments: {e}"))?;
    let args = RecordArgs {
        title: &p.title,
        memory_type,
        key: p.key.as_deref(),
        summary: p.summary.as_deref(),
        trust_level: p.trust_level.as_deref(),
        severity: p.severity.as_deref(),
        tags: &p.tags.unwrap_or_default(),
        paths: &p.paths.unwrap_or_default(),
        globs: &p.globs.unwrap_or_default(),
        commands: &p.commands.unwrap_or_default(),
        lifespan: parse_optional_lifespan(p.lifespan.as_deref())?,
        status: parse_optional_status(p.status.as_deref())?,
        repo: p.repo.as_deref(),
        global: p.global.unwrap_or(false),
        review_by: None,
        sources: &[],
    };
    let mut buf = Vec::new();
    memory::run_record(Some(root.to_path_buf()), &args, &mut buf)
        .context("invalid arguments")?;
    Ok(String::from_utf8(buf)?)
}
```

Argument errors from `run_record` (e.g. "Title must not be empty") are wrapped with
`"invalid arguments"` via `.context()`, so `map_review_error` branch 2 maps them to
`-32602`. `RecordArgs` borrows `&str` slices from `RecordParams` — both live to the
end of the match arm, so lifetimes are sound.

### Response
Returns the `run_record` confirmation message captured from the writer buffer:
`"Recorded memory mem_... (key): /path/to/dir\n"`.

---

## `memory_edit` tool

### JSON Schema

```json
{
  "type": "object",
  "properties": {
    "reference":  { "type": "string", "description": "Memory reference: uid or key (required)" },
    "title":      { "type": "string", "description": "New title" },
    "summary":    { "type": "string", "description": "New summary" },
    "status":     { "type": "string", "enum": ["active","draft","superseded","retracted","archived","quarantined"] },
    "lifespan":   { "type": "string", "enum": ["semantic","episodic","procedural","working","identity"] },
    "review_by":  { "type": "string" },
    "trust":      { "type": "string", "enum": ["high","medium","low"] },
    "severity":   { "type": "string", "enum": ["high","medium","low"] },
    "key":        { "type": "string", "description": "Set key (only if none exists — immutable once set)" },
    "path_scope": { "type": "array", "items": { "type": "string" } },
    "glob":       { "type": "array", "items": { "type": "string" } },
    "command":    { "type": "array", "items": { "type": "string" } }
  },
  "required": ["reference"]
}
```

### Dispatch

```rust
"memory_edit" => {
    #[derive(Deserialize)]
    struct EditParams {
        reference: String,
        title: Option<String>,
        summary: Option<String>,
        status: Option<String>,
        lifespan: Option<String>,
        review_by: Option<String>,
        trust: Option<String>,
        severity: Option<String>,
        key: Option<String>,
        path_scope: Option<Vec<String>>,
        glob: Option<Vec<String>>,
        command: Option<Vec<String>>,
    }
    let p: EditParams = serde_json::from_value(arguments)
        .map_err(|e| anyhow::anyhow!("invalid arguments: {e:#}"))?;
    let fields = EditFields {
        title: p.title,
        summary: p.summary,
        status: p.status,
        lifespan: p.lifespan,
        review_by: p.review_by,
        trust: p.trust,
        severity: p.severity,
        key: p.key,
        path_scope: p.path_scope,
        glob: p.glob,
        command: p.command,
    };
    let mut buf = Vec::new();
    memory::run_edit(Some(root.to_path_buf()), &p.reference, &fields, &mut buf)
        .context("invalid arguments")?;
    Ok(String::from_utf8(buf)?)
}
```

Argument errors (e.g. "key already set", empty title) are wrapped with
`"invalid arguments"` via `.context()` → mapped to `-32602`.
```

### Response
Returns `"Edited memory <reference>\n"`.

---

## `doctrine_onboard` tool

### Definition
- **Parameters:** none (`"type": "object", "properties": {}, "required": []`)
- **Output:** single text block containing rendered markdown with two sections

### Markdown content

**Section 1 — CLI→MCP mapping table** (static):

```
# Doctrine MCP Onboarding

## CLI → MCP Tool Mapping
When MCP tools are available, use these tools instead of CLI commands:

| CLI command | MCP tool | Notes |
|---|---|---|
| `doctrine review new` | `review_new` | |
| `doctrine review list` | `review_list` | |
| `doctrine review show <ref>` | `review_show` | `reference` param |
| `doctrine review raise` | `review_raise` | |
| `doctrine review dispose` | `review_dispose` | |
| `doctrine review verify` | `review_verify` | |
| `doctrine review contest` | `review_contest` | |
| `doctrine review withdraw` | `review_withdraw` | |
| `doctrine review status` | `review_status` | |
| `doctrine review prime` | `review_prime` | |
| `doctrine memory find` | `memory_find` | |
| `doctrine memory retrieve` | `memory_retrieve` | |
| `doctrine memory show <ref>` | `memory_show` | `reference` param |
| `doctrine memory list` | `memory_list` | |
| `doctrine memory validate` | `memory_validate` | |
| `doctrine memory record` | `memory_record` | |
| `doctrine memory edit` | `memory_edit` | |
```

**Section 2 — Bundled onboarding memories** (resolved at runtime):

```
## Onboarding Memories

=== MEMORY (mem.signpost.doctrine.overview) ===
... body resolved from memory store ...

=== MEMORY (mem.signpost.project.orientation) ===
... body resolved from memory store ...
```

Rendered using `retrieve::retrieve_reference` — the same path the `memory_retrieve`
MCP handler uses. Each memory body is rendered as a framed text block with the
`=== MEMORY (key) ===` header, then the body.

### Implementation

```rust
fn render_onboard(root: &Path) -> anyhow::Result<String> {
    let mut parts: Vec<String> = Vec::new();
    parts.push(ONBOARD_MAPPING_TABLE.to_owned());
    parts.push("\n## Onboarding Memories\n".to_owned());
    for key in &["mem.signpost.doctrine.overview", "mem.signpost.project.orientation"] {
        let mut buf = Vec::new();
        retrieve::retrieve_reference(&mut buf, root, key, false, None)?;
        parts.push(String::from_utf8(buf)?);
    }
    Ok(parts.concat())
}
```

Reuses `retrieve::retrieve_reference` (the same renderer as `memory_retrieve`). No new
engine code.

The `doctrine_onboard` tool appears in its own mapping table row (`doctrine onboard` →
`doctrine_onboard`) so agents discover it self-referentially.

---

## Engine change: `writer` parameter on `run_record` + `run_edit`

### Current signatures
```rust
pub(crate) fn run_record(path: Option<PathBuf>, args: &RecordArgs<'_>) -> Result<()>
pub(crate) fn run_edit(path: Option<PathBuf>, reference: &str, fields: &EditFields) -> anyhow::Result<()>
```

### New signatures
```rust
pub(crate) fn run_record(path: Option<PathBuf>, args: &RecordArgs<'_>, writer: &mut dyn Write) -> Result<()>
pub(crate) fn run_edit(path: Option<PathBuf>, reference: &str, fields: &EditFields, writer: &mut dyn Write) -> anyhow::Result<()>
```

### Changes
- `run_record`: replace `let mut stdout = io::stdout()` with `writer`; replace `writeln!(stdout, ...)` with `writeln!(writer, ...)`
- `run_edit`: replace `writeln!(io::stdout(), ...)` with `writeln!(writer, ...)`
- CLI call site (`src/memory.rs` ~line 482): pass `&mut io::stdout()`

Follows the existing pattern of `run_show`, `run_list`, `run_validate` which already
accept `writer: &mut impl Write`.

---

## Boot footer update

Replace `.doctrine/boot-footer.md` with:

```
Immediately on beginning your NEXT TURN:
If the MCP `doctrine_onboard` tool is available, call it to get onboarding context
in a single call. Otherwise, use /retrieving-memory skill to retrieve
`mem.signpost.doctrine.overview` and `mem.signpost.project.orientation`.
```

No code changes to `src/boot.rs` — the footer is user-authored prose injected at boot
time by the existing `FOOTER_REL` constant path.

---

## Verification impact

### VT-2 (tools/list count)
- E2E (`tests/e2e_mcp_server.rs`): `15` → `18`
- Unit (`tools.rs` test module): `15` → `18` in `tool_list_has_14_tools()`,
  `tools_list_response_structure()`
- Name assertions: add `memory_record`, `memory_edit`, `doctrine_onboard`

### New E2E tests
- `memory_record` + `memory_show` round-trip: record a memory, verify it appears in show
- `memory_edit` round-trip: record → edit title → verify changed
- `doctrine_onboard` returns non-empty text

### New unit tests
- `memory_record` with invalid `memory_type` returns `-32602`
- `memory_edit` with no `reference` returns `-32602`
- `memory_edit` with no flags returns `-32602` (at least one flag required)

---

## Code impact summary

| Path | Change |
|---|---|
| `src/mcp_server/tools.rs` | +3 `McpTool` definitions, +3 `call_tool` match arms, `doctrine_onboard` renderer, VT counts 15→18, memory arg-error wrapper |
| `src/memory.rs` | Add `writer: &mut dyn Write` to `run_record` + `run_edit` signatures, update call sites |
| `tests/e2e_mcp_server.rs` | VT-2 count update, name assertions, 3 new round-trip tests |
| `.doctrine/boot-footer.md` | MCP-first onboarding instruction with fallback |

## Remaining open questions
None — all design decisions resolved.
