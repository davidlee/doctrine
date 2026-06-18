# SL-099 Design: Memory read-path relations and data-model hardening

## Status

`design` — locked. All foundational decisions settled. Proceeding to adversarial review
then `/plan`.

## Context

Doctrine's memory system has an authored relation graph (180 `[[relation]]` rows across
~100 memories) and wikilink cross-references in body text — but neither surfaces at any
CLI read path. `memory show` and `memory retrieve` don't include relations. `inspect`
refuses memory refs. Wikilinks (`[[mem.…]]`) are never extracted or resolved.

This slice adds the read-path surfaces, wikilink extraction, backlinks, graph expansion,
missing `record` flags, the `lifespan` field, ageing in the sort key, suggested
relations, a `validate` command, and `verify --allow-dirty`.

Full scope: `slice-099.md` (7 objectives). Capability gap analysis: `capability-gaps.md`.
UX spike: `ux-spike.md`.

## Architecture

### Module layout

```
src/links.rs       NEW  leaf   wikilink extraction, resolution, backlinks index
src/validate.rs    NEW  leaf   pure validation predicates (dangling, stale, expiry)
src/memory.rs      cmd   widened schema (lifespan, provenance, review_by),
                             relations in show render, --allow-dirty in verify,
                             suggested relations in record, wikilink section
src/retrieve.rs    eng   lifespan_factor + ageing in sort_key,
                             --lifespan filter, --expand N BFS
src/lexical.rs     leaf  UNCHANGED (Bm25Ranker reused for suggested relations)
src/main.rs        cmd   new CLI handlers (resolve-links, backlinks, validate,
                             --expand), inspect pre-dispatch for memory refs,
                             new record/retrieve flags
```

All new modules live at the leaf tier (ADR-001): they import nothing from `memory`,
`retrieve`, or any engine/command module. `memory.rs` and `retrieve.rs` import from
them — dependency direction is leaf ← engine ← command, no cycles.

### Behaviour-preservation gate

Shared machinery that must stay green unchanged:

| Module | Why sensitive | Gate |
|---|---|---|
| `src/entity.rs` | `materialise_named`, `scan_named`, `write_fileset` — the entity engine | Existing test suites pass unchanged |
| `src/relation.rs` | Relation vocabulary leaf | Unchanged — no new relation labels |
| `src/catalog/` | `scan_entities`, `hydrate` — already parses `RawRelation` from memory.toml | Existing catalog tests pass unchanged |
| `src/retrieve.rs` | Sort key gains key-8 ageing modulation | Existing rank tests pass with default (unset) lifespan = 1.0 multiplier |
| `src/lexical.rs` | Bm25Ranker reused | Unchanged — no edits to lexical.rs |

New behaviour → new tests. Existing suites are proof.

## Design decisions

### D1 — Wikilink extractor in `src/links.rs` (leaf tier)

Pure regex extraction over body text. Imports nothing from `memory` or `retrieve`.
Exports:

```rust
/// A wikilink target extracted from body text.
pub(crate) struct Wikilink {
    pub(crate) target: String,  // "mem.pattern.cli.skinny" or "mem_018f3a..."
    pub(crate) is_uid: bool,
}

/// Extract [[mem.*]] wikilinks from body, skipping fenced + inline code.
/// Line-aware: state machine tracks inside/outside ``` fences, strips
/// inline backticks per line, regexes the remainder.
/// Pure — takes a string, returns Wikilinks. No memory/entity imports.
pub(crate) fn extract_wikilinks(body: &str) -> Vec<Wikilink>;

/// Resolve one wikilink target against a known-uid set + known-key→uid map.
/// Returns Ok(uid) if resolved, Err(target) if dangling.
/// Pure — takes string-keyed lookups. Callers build the maps from Memory.
pub(crate) fn resolve_wikilink(
    known_uids: &BTreeSet<&str>,
    key_to_uid: &BTreeMap<&str, &str>,
    target: &str,
    is_uid: bool,
) -> Result<String, String>;

/// Build reverse index: for each memory (keyed by uid), its wikilink targets
/// → inverted map of target→[source_uids]. Caller provides per-uid wikilink lists
/// and per-uid [[relation]] targets; deduplication happens here.
/// Pure — takes string-keyed data. No Memory type dependency.
pub(crate) fn backlinks_index(
    wikilinks_by_uid: &BTreeMap<&str, Vec<&Wikilink>>,
    relations_by_uid: &BTreeMap<&str, Vec<&str>>,  // target strings
) -> BTreeMap<String, BTreeSet<String>>;  // target → {source_uids}
```

**Leaf-tier purity.** All `links.rs` functions take string-keyed data (uids, body
strings, target strings). The Memory→string projection happens at call sites in
`memory.rs` / `retrieve.rs`. This keeps `links.rs` at the leaf tier with zero
imports from command or engine modules (ADR-001).

**Wikilink + relation coexistence.** Graph ops (backlinks, `--expand`) take the
deduplicated union of wikilink targets and `[[relation]]` targets. The link set is
treated as a single graph. Wikilinks carry contextual meaning inline in prose;
`[[relation]]` edges are introspectable, queryable structural connections.

**Code-block skipping.** Line-aware state machine: `in_fence: bool` tracks
`` ``` `` boundaries. Non-fence lines: strip inline `` ` ``-delimited spans, then
apply `\[\[mem[._][^\]]+\]\]` regex. Correct for current body sizes (a few KB each).

### D2 — `inspect` pre-dispatches `MemoryRef::parse`

`run_inspect` in `main.rs` currently calls `integrity::parse_canonical_ref(id)`,
which requires a numbered prefix (`SL-031`). Memory keys contain dots; memory
uids are 35-character hex strings — neither fits. `inspect_from` takes `u32`;
memory uids are strings, so a separate entry point is needed.

**Pre-dispatch:** try `MemoryRef::parse(arg)` first. If it matches:

1. Resolve the uid (uid prefix → full uid via `resolve_uid_prefix`; key → uid
   via symlink resolution).
2. Route to `relation_graph::memory_inspect_view(root: &Path, uid: &str)` which:
   - Reads the catalog scan via `scan_entities` (memories are already scanned
     as `ScannedEntity` with `RawRelation` edges).
   - Filters to memory entities, builds a relation-graph projection keyed by
     uid string (not `u32`).
   - Renders `outbound:` (the memory's own `[[relation]]` rows), `inbound:`
     (other memories' edges pointing to this uid), and `danglers:` (this
     memory's unresolvable targets).
   - Adds a `wikilinks:` section showing `[[mem.…]]` references extracted from
     this memory's body (resolved targets; dangling marked).

The format mirrors numbered-entity `inspect`. Memory relation labels are
free-form strings (`CatalogEdgeLabel::Raw`), not vocabulary-bound — rendered
verbatim.

A non-memory ref falls through to the existing `parse_canonical_ref` path unchanged.

### D3 — Template + Draft widening for new record flags

`Draft`, `RecordArgs`, and `render_memory_toml` all widen so new flags land in the
written TOML. Without this widening, a `--lifespan semantic` flag would parse
successfully but never reach the file — a silent data-loss bug.

**`Draft` gains:**

```rust
pub(crate) struct Draft<'a> {
    // … existing fields …
    pub(crate) lifespan: Option<Lifespan>,
    pub(crate) review_by: Option<&'a str>,
    pub(crate) sources: &'a [Provenance],
    pub(crate) trust_level: Option<&'a str>,   // overrides "medium" default
    pub(crate) severity: Option<&'a str>,       // overrides "none" default
}
```

**`RecordArgs` gains matching fields.** `run_record` maps CLI flags into `Draft`.

**`render_memory_toml` changes:**
- When `lifespan` is `Some(l)`, emit `lifespan = "<l.as_str()>"` line after
  `memory_type`. When `None`, omit the line entirely (not `lifespan = ""`).
- When `review_by` is `Some(d)`, emit `review_by = "<d>"`. When `None`, emit
  empty (existing behaviour — `review_by = ""`).
- Loop `sources` to emit `[[source]]` blocks (kind + ref + optional note).
  Empty sources → no blocks emitted.
- `trust_level`: use the provided value if `Some`, else `"medium"` (existing default).
- `severity`: use the provided value if `Some`, else `"none"` (existing default).

The `[trust]` and `[ranking]` blocks are already emitted by the template — the
new flags only override the default values, not add new blocks.

### D4 — Lifespan ageing: float multiplier table

The recency component of the sort key (key 8) is modulated by lifespan. The existing
sort key uses raw `days_between(reviewed, today)` — fewer days ranks higher. Lifespan
ageing applies a multiplier before the sort key sees the value.

**Multiplier table:**

| Lifespan | Factor | Mental model |
|---|---|---|
| `identity` | 0.0 | never ages — permanent recency |
| `semantic` | 0.1 | ~10 real days = 1 effective day |
| `procedural` | 0.33 | ~3 real days = 1 effective day |
| `episodic` | 1.0 | baseline — ages at wall-clock rate |
| `working` | 10.0 | 1 real day = 10 effective days |
| *(unset)* | 1.0 | baseline |

**Computation** (pure, in `src/retrieve.rs` next to `days_between`):

```rust
fn lifespan_factor(lifespan: Option<Lifespan>) -> f64 {
    match lifespan {
        Some(Lifespan::Identity) => 0.0,
        Some(Lifespan::Semantic) => 0.1,
        Some(Lifespan::Procedural) => 0.33,
        Some(Lifespan::Episodic) => 1.0,
        Some(Lifespan::Working) => 10.0,
        None => 1.0,
    }
}

/// Effective recency age for sort key 8.
/// Sentinel: i64::MAX (the missing-date marker from days_between) passes
/// through unchanged so missing dates always sort last regardless of factor.
/// Otherwise: real_days * factor → round → clamp to i64::MAX - 1 (avoids
/// sentinel collision for very old working memories).
fn effective_age(days: i64, lifespan: Option<Lifespan>) -> i64 {
    if days == i64::MAX {
        return i64::MAX;  // missing-date sentinel — never modulated
    }
    let factor = lifespan_factor(lifespan);
    let effective = (days as f64 * factor).round() as i64;
    effective.min(i64::MAX - 1)
}
```

`effective_age` replaces the raw `days_between(…).unwrap_or(i64::MAX)` in
`sort_key` key 8. The `as_conversions` ban is handled per `lexical::quantize`
precedent with a stacked `#[expect]`.

**Sort key 8 change (behaviour-preservation):** an unset lifespan multiplies by
1.0, so existing memories with no `lifespan` field sort identically to before.
The new `effective_age` function is the only change to the sort key.

### D5 — Suggested relations: reuse `Bm25Ranker`

After `run_record` writes a memory:

1. `collect_all(root)` — union of items/ + shipped/
2. Filter out the just-recorded uid
3. Build `LexicalCorpus::Raw(&docs)` over existing memories
4. Score the new memory's `lex_doc` projection with `Bm25Ranker` as query
5. Take top 5 by BM25 descending, deduplicate against existing `[[relation]]` targets
6. Print to **stderr**: `note: you might want to link to: …`
7. Skip silently if corpus < 1 memory

No new machinery in `lexical.rs` — `Bm25Ranker` is used as-is.

### D6 — `verify --allow-dirty`

**Without flag:** behaviour byte-identical to current. Dirty tree → bail with
"working tree is dirty: refusing to verify."

**With `--allow-dirty`:** Skip the dirty-tree bail. In `stamp_verification`:

```rust
// Without --allow-dirty: frame.commit (HEAD, always clean — dirty is refused)
// With --allow-dirty: frame.checkout_state_id when anchor is CheckoutState
let sha = if allow_dirty && frame.anchor_kind == AnchorKind::CheckoutState {
    &frame.checkout_state_id
} else {
    &frame.commit
};
git.insert("verified_sha", toml_edit::value(sha));
```

The `staleness` function (branch 1) already keys on `verified_sha` presence,
not anchor kind — a dirty-attested memory with `verified_sha` set and path-scoped
enters commit mode. The `anchor:` line in `show` already surfaces `checkout_state`
kind, so the attestation is transparent.

### D7 — `validate` uses catalog hydrate's target resolution

Three checks, all advisory (never writes). Output format: one finding per line,
`UID: CHECK: detail`.

1. **Dangling relations.** For each `[[relation]]` target: try `MemoryRef::parse`
   → resolve in memory store. If not memory-shaped, try
   `integrity::parse_canonical_ref` → check entity existence via the catalog scan.
   Report each dangling target:
   `mem_018f3a…: dangling: [[relation]] target "SL-999" not found`

2. **Stale verification.** If `verified_sha` is set and memory has `scope.paths`,
   run `git::commits_touching(root, &paths, &verified_sha, "HEAD")`. If ≥1
   commits touch those paths since verification, report stale:
   `mem_018f3a…: stale: verified_sha 3 commits behind HEAD on scoped paths`

3. **Draft expiry.** If `status == "draft"` and `review_by` is set and
   `days_between(review_by, today)` is `Some(d)` where `d < 0`, report expired:
   `mem_018f3a…: expired: draft past review_by 2026-06-01 (15 days ago)`

Corpus-wide if no REF given. Exit 0 if clean, exit 1 if warnings.

### D8 — Wikilinks surface in `show` and `backlinks`/`--expand`, not `retrieve`

`memory show` (text): new `wikilinks:` section listing resolved targets (dangling
marked with `(dangling)`). JSON: `"wikilinks": […]` array. `memory retrieve`
blocks are ranking-oriented — wikilinks would be noise there. `backlinks` and
`--expand` consume wikilinks as part of the link set.

### D9 — `--expand N` output format

BFS from matched memories following the union of wikilinks + `[[relation]]`
targets. Only memory→memory edges are traversed — cross-entity relation targets
are not followed (they appear in `inspect` only).

**Output:** Framed blocks concatenated, separated by blank lines. Each block
reuses `render_show` with a `depth: N` staleness line (depth ≥ 1).

**Cycle handling:** `visited: BTreeSet<String>` tracks visited uids. A cycle
(A → B → A) breaks silently — B is visited at depth 1, A is skipped at depth 2.

**Ordering at equal depth:** Nodes at the same depth are sorted by
`sort_default` (created desc, uid asc) — consistent with `memory list`.

**Depth-0:** The matched memories themselves (the `retrieve` survivors) are NOT
re-rendered as part of `--expand`. Only expanded nodes at depth ≥ 1 appear.
This avoids duplicating the retrieve output.

### D10 — `--provenance-source KIND:REF` parse

Split on the FIRST colon only. `KIND` = everything before `:`, `REF` =
everything after. Validated: `KIND` must be non-empty and match
`[a-z][a-z0-9-]*` (lowercase alphanumeric with hyphens).

Examples:
- `code:src/lexical.rs` → kind=`code`, ref=`src/lexical.rs`
- `url:https://example.com` → kind=`url`, ref=`https://example.com`
- `discussion:PR #42` → kind=`discussion`, ref=`PR #42`

### D11 — `resolve-links` is wikilinks-only

`memory resolve-links` extracts only wikilinks (`[[mem.…]]`) from body text,
not `[[relation]]` rows. This matches the scope doc: "regex-extract
`[[mem.<key>]]` / `[[mem_<uid>]]` from body." `backlinks` and `--expand` handle
the union of both — that's their role as graph ops.

`memory show` (text): new `wikilinks:` section listing resolved targets (dangling
marked with `(dangling)`). JSON: `"wikilinks": […]` array. `memory retrieve`
blocks are ranking-oriented — wikilinks would be noise there. `backlinks` and
`--expand` consume wikilinks as part of the link set.

## Current → Target behaviour

### Objective 1 — Surface `[[relation]]` rows

| Surface | Current | Target |
|---|---|---|
| `memory show` (text) | No relations | `relations:` block after `anchor:`, one `label → target` per line |
| `memory show --json` | No relations | `"relations": [{"label": "...", "target": "..."}]` |
| `memory retrieve` | No relations | `relations:` line in framed block header |
| `doctrine inspect` | Refuses memory refs | Pre-dispatches `MemoryRef::parse`, renders inbound + outbound edges |

### Objective 2 — Wikilinks, backlinks, `--expand`

| Verb | Behaviour |
|---|---|
| `memory resolve-links [REF]` | Regex-extract `[[mem.<key>]]` / `[[mem_<uid>]]` from body (skip code), resolve against store. Report resolved count, dangling count, dangling targets |
| `memory backlinks <REF>` | Reverse index from wikilinks + `[[relation]]` (deduped), return sources linking to REF. Table via shared column spine (`listing::render_columns`): `uid type title method` where method = `wikilink` or the `[[relation]]` label |
| `memory retrieve --expand N` | BFS from matched memories following union of wikilinks + relations up to depth N. **Memory→memory edges only** — cross-entity relation targets (to non-memory entities) are not traversed; they appear in `inspect` only. Framed blocks with `depth: N` header |
| `memory show` | New `wikilinks:` section. JSON: `"wikilinks"` array |

### Objective 3 — Record flag gaps

| Flag | TOML field | Default |
|---|---|---|
| `--lifespan semantic\|episodic\|procedural\|working\|identity` | `lifespan` | unset (absent from TOML) |
| `--review-by YYYY-MM-DD` | `review.review_by` | unset (empty string) |
| `--provenance-source KIND:REF` (repeatable) | `[[source]]` | empty |
| `--trust low\|medium\|high` | `trust.trust_level` | `medium` |
| `--severity critical\|high\|medium\|low\|none` | `ranking.severity` | `none` |

`Draft`, `RecordArgs`, `RawMemoryToml`, `RawSource`, `Memory`, and
`render_memory_toml` all widen. `RawSource` gains `kind`/`ref`/`note` fields
(currently an empty struct — widened from `struct RawSource {}`).

### Objective 4 — Lifespan field + filter + ageing

- New `Lifespan` enum: `Semantic`, `Episodic`, `Procedural`, `Working`, `Identity`
- `Memory` gains `lifespan: Option<Lifespan>`
- `--lifespan` filter on `find`/`retrieve`:
  - `QueryContext` gains `lifespan: Option<Lifespan>`
  - Applied in `query()` filter cascade after `base_filter` + `match_scope`:
    a `None` filter passes everything; `Some(l)` requires exact match
- Ageing: `effective_age(days, lifespan)` modulates key 8 of the 9-key sort key
- Sentinel `i64::MAX` (missing reviewed date) passes through unchanged — missing
  dates always sort last regardless of lifespan
- Unset lifespan → factor 1.0 → existing sort behaviour unchanged

### Objective 5 — Suggested relations on `record`

After `record`: score new body against existing corpus with `Bm25Ranker`, report
top 5 to stderr. Deduplicate against already-authored `[[relation]]` targets.

### Objective 6 — `memory validate [REF]`

Three checks: dangling relations, stale verification, draft expiry. Exit 1 on
warnings, 0 if clean. Corpus-wide if no REF given.

### Objective 7 — `verify --allow-dirty`

With flag: skip dirty-tree bail, stamp `checkout_state_id` in `verified_sha`.
Without flag: existing behaviour (refuse dirty).

## Schema changes

### New TOML fields

```toml
# Top-level: orthogonal to memory_type
lifespan = "semantic"  # optional — semantic|episodic|procedural|working|identity

# [review] block: new key
review_by = "2026-07-01"  # optional — scheduled review date

# New [[source]] array-of-tables (repeatable)
[[source]]
kind = "code"
ref = "src/lexical.rs"
note = ""  # optional
```

### New Rust types

```rust
// src/memory.rs
pub(crate) enum Lifespan {
    Semantic,
    Episodic,
    Procedural,
    Working,
    Identity,
}

pub(crate) struct Provenance {
    pub(crate) kind: String,   // "code", "doc", "discussion", …
    pub(crate) ref_: String,   // "src/lexical.rs", URL, …
    pub(crate) note: String,
}
```

### Widened validated Memory

`Memory` (the validated projection) gains three fields it currently ignores via `..`:

- `relations: Vec<RawRelation>` — pass-through from `RawMemoryToml.relations`.
  No validation on free-form labels (per `mem.pattern.link.memory-label-fork`).
  Used by `render_show`/`show_json`/inspect.
- `lifespan: Option<Lifespan>` — parsed from the `lifespan` TOML key.
- `sources: Vec<Provenance>` — parsed from `[[source]]` rows.

`RawSource` widens from the current empty `struct RawSource {}` to carry
`kind`, `ref`, and `note` fields.

### Template changes (`install/templates/memory.toml`)

Add `lifespan` (absent by default, no `{{lifespan}}` token), `review_by`
(substituted from `Draft`), and `[[source]]` block (looped from `Draft.sources`).

## CLI surface

### New verbs

```
doctrine memory resolve-links [REF]
doctrine memory backlinks <REF>
doctrine memory validate [REF]
```

### New flags on existing verbs

```
doctrine memory record
  --lifespan <LIFESPAN>
  --review-by <DATE>
  --provenance-source <KIND:REF>  (repeatable)
  --trust <LEVEL>
  --severity <LEVEL>

doctrine memory find/retrieve
  --lifespan <LIFESPAN>

doctrine memory retrieve
  --expand <N>

doctrine memory verify
  --allow-dirty

doctrine inspect
  (now accepts mem_<uid> / mem.<key>)
```

## Verification alignment

| Requirement | Test strategy |
|---|---|
| Relations in `show`/`retrieve` | Unit: `render_show` output contains `relations:` block when `RawRelation` present. Integration: `memory show <uid>` for a memory with known relations |
| Wikilink extraction | Unit: `extract_wikilinks` against bodies with/without wikilinks, fenced blocks, inline code |
| Backlinks correctness | Unit: `backlinks_index` returns correct reverse edges. Integration: `memory backlinks <REF>` |
| `--expand N` | Unit: BFS depth correct. Integration: `memory retrieve --expand 2` |
| `inspect` memory refs | Integration: `doctrine inspect mem_<uid>` renders inbound + outbound |
| New `record` flags round-trip | Unit: `render_memory_toml` → `Memory::parse` carries new fields |
| `--lifespan` filter | Unit: `match_scope`/`base_filter` with lifespan filter |
| Ageing in sort key | Unit: `effective_age` table-driven against all (lifespan, days) pairs. Unit: `rank` orders correctly with lifespan modulation |
| Suggested relations | Unit: BM25 scores non-zero for related body, zero for unrelated. Integration: record → stderr output |
| `validate` checks | Unit: pure predicate tests for each check. Integration: `memory validate` against fixtures |
| `--allow-dirty` | Unit: `stamp_verification` writes `checkout_state_id`. Integration: `verify --allow-dirty` on dirty tree |
| Behaviour-preservation | Existing test suites in `entity.rs`, `relation.rs`, `catalog/`, `retrieve.rs`, `lexical.rs` pass unchanged |
| Backward compatibility | Existing memories without new fields parse identically |

## Phase plan (provisional)

Will be detailed in `/plan`. Anticipated phases:

1. **PHASE-01 — Schema + template widening.** `Lifespan` enum, `Provenance` struct, widen `RawMemoryToml`/`Memory`/`Draft`/`RecordArgs`, template changes. New flags parse + round-trip.
2. **PHASE-02 — `src/links.rs` wikilink extractor.** `extract_wikilinks`, `resolve_wikilink`, `backlinks_index`. Pure, tested exhaustively.
3. **PHASE-03 — Show/retrieve surface widening.** Relations in `render_show`/`show_json`/retrieve blocks. Wikilinks in `show`. `resolve-links` + `backlinks` CLI verbs.
4. **PHASE-04 — `inspect` memory bridge.** Pre-dispatch `MemoryRef::parse` in `run_inspect`. `relation_graph::memory_inspect_view`.
5. **PHASE-05 — Lifespan filter + ageing.** `lifespan_factor`, `effective_age`, sort key modulation. `--lifespan` filter on `find`/`retrieve`.
6. **PHASE-06 — Suggested relations + `--expand N`.** Post-record BM25 scoring. BFS graph expansion in retrieve.
7. **PHASE-07 — `validate` + `--allow-dirty`.** Three validation checks. Dirty-tree stamp in verify.
8. **PHASE-08 — Integration + behaviour-preservation gate.** Cross-phase integration tests. Existing suite verification. `just gate` green.

## Open questions (non-blocking)

| # | Question | Disposition |
|---|---|---|
| OQ1 | `backlinks` output format: relation type column? | Yes — `method` column with `wikilink` or the `[[relation]]` label. Defers to implementation |
| OQ2 | `--expand N` output: tree or framed blocks? | Framed blocks with `depth: N` header. Reuses `render_show`, consistent with retrieve |
| OQ3 | Suggested relations deduplicate against existing relations? | Yes — `BTreeSet` of existing `(label, target)` pairs |
| OQ4 | `validate --json`? | Defer — human output first |
| OQ5 | Suggested relations score over full body text rather than title+summary+tags? | Defer — body indexing is a larger change; `lex_doc` projection is explicit in v1 |

## Adversarial review findings

| # | Finding | Severity | Resolution |
|---|---|---|---|
| F1 | `links.rs` took `&[Memory]` in signatures — leaf tier can't import command-tier `Memory` | Doctrinal | D1 revised: all `links.rs` functions take string-keyed data (uids, body strings, target strings). Memory→string projection at call sites |
| F2 | `Memory` doesn't carry `relations` through validation (ignored via `..`) | Missing detail | `Memory` gains `relations: Vec<RawRelation>` pass-through. `RawSource` widens from empty struct |
| F3 | `effective_age` could saturate to `i64::MAX` and collide with missing-date sentinel | Boundary | Sentinel passes through unchanged; valid ages clamp to `i64::MAX - 1` |
| F4 | `memory_inspect_view` unspecified — `inspect_from` takes `u32` not string uid | Underspecified | D2 detailed: separate entry point with string-keyed graph projection, own `outbound`/`inbound`/`danglers`/`wikilinks` sections |
| F5 | `validate` output format unspecified | Missing detail | D6 specifies `UID: CHECK: detail` per-line format with examples |
| F6 | `[[source]]` struct currently empty — needs `kind`/`ref`/`note` | Underspecified | Noted in schema changes section |
| F7 | `--lifespan` filter placement in `query()` not specified | Missing detail | Objective 4 now specifies: `QueryContext` gains `lifespan`, applied in filter cascade after `match_scope` |
| F8 | `--expand` BFS could traverse into non-memory entities | Underspecified | D9 added: BFS follows memory→memory edges only; cross-entity targets are `inspect`-only |
| F9 | `backlinks` output format unspecified | Missing detail | Objective 2 table now specifies shared column spine (`listing::render_columns`) |

## Walkthrough review findings (external adversarial pass)

| # | Finding | Severity | Disposition |
|---|---|---|---|
| W1 | `memory_inspect_view` placement ambiguous — tier violation risk if in `relation_graph.rs` | Medium | Resolved: placed in `memory.rs` (command tier), imports `relation_graph` + `catalog` downward. D2 updated |
| W2 | `--expand N` output underspecified — cycles, ordering, depth-0 handling | Medium | Resolved: D9 specifies visited set, sort_default, depth-0 exclusion |
| W3 | `validate` `commits_touching` subprocess cliff | Low | Noted. Design `commits_touching` for multi-path queries from the start so batching is a non-breaking follow-up |
| W4 | `--provenance-source` parse — colon ambiguity | Low | Resolved: D10 specifies first-colon split + KIND format validation |
| W5 | Suggested relations scores title+summary+tags only (lex_doc), not full body | Low | Noted in OQ5. Defer to follow-up |
| W6 | Wikilink regex cost extrapolation should be O(total_bytes) | Low | Resolved: D1 corrected to scale with total body bytes |
| W7 | `Memory` gains 3 new fields — test construction sites need updating | Low | Noted. Phase plan to account for test fixture updates |
| W8 | `resolve-links` vs `backlinks` asymmetry | Low | Resolved: D11 — `resolve-links` is wikilinks-only |
| W9 | Template + `Draft` widening for new flags — silent data-loss risk | Medium | Resolved: D3 specifies Draft/RecordArgs/render widening in detail |
| W10 | `--trust` / `--severity` — record pipeline may not write them | Medium | Resolved by D3 — template already emits both blocks; new flags override defaults |

## Governance alignment

| Authority | Requirement | Alignment |
|---|---|---|
| ADR-001 | Leaf ← engine ← command, no cycles | ✓ `links.rs`/`validate.rs` at leaf, `retrieve.rs` at engine, `memory.rs`/`main.rs` at command |
| ADR-004 | Relations stored outbound-only; reciprocity derived | ✓ Backlinks and inspect inbound are derived from outbound edges + wikilinks; no stored reverse edges |
| ADR-010 | Memory relations are Tier 3 (free-form labels, `CatalogEdgeLabel::Raw`) | ✓ Labels rendered verbatim; no vocabulary enforcement |
| Storage rule | Authored vs derived tiers | ✓ Wikilinks are derived (on-the-fly regex), never persisted |
| Behaviour-preservation gate | Shared machinery suites stay green unchanged | ✓ `entity.rs`, `relation.rs`, `catalog/`, `lexical.rs` untouched; `retrieve.rs` existing tests pass with unset lifespan = 1.0 |
| Pure/imperative split | No clock, rng, git, disk in pure layer | ✓ `effective_age`, `extract_wikilinks`, validation predicates are pure; git/clock live in shell |

## Risks

- **Wikilink regex cost** (~0.007s corpus-wide at 193 memories). Linear scaling — re-evaluate if >100ms.
- **Backward compatibility.** All new TOML fields optional with defaults. Existing memories parse unchanged (proven by existing `Memory::parse` tests).
- **`--allow-dirty` trust.** A dirty-tree attestation is less trustworthy than a commit-attested one. The flag makes the tradeoff explicit; the `anchor:` line surfaces the `checkout_state` kind.
- **Behaviour-preservation.** Unset lifespan = 1.0 multiplier = existing sort key unchanged. Existing test suites must pass with zero edits.
- **Corpus-wide validate cost.** `commits_touching` spawns a subprocess per memory — acceptable at current scale; flag as a follow-up if >100 memories with `verified_sha`.
