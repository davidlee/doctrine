# Design: SL-141 — Entity full-text search

## Architecture

New command-tier module `src/search.rs` (ADR-001 — command → engine → leaf).
Singular responsibility: orchestrate BM25 full-text search over entity `.md`
bodies via the existing `lexical` leaf.

```
search.rs
  ├── KindSelector::resolve(…)     → KindSelector  (pure)
  ├── entity_lex_doc(ent)          → LexDoc        (pure adapter)
  ├── build_corpus(kinds, root)    → Vec<LexDoc>   (impure: reads .md)
  ├── snippet(body, query)         → Option<String> (pure, uses lexical::tokenize_with_spans)
  ├── output_table / output_json   (listing facade)
  └── run(query, opts, root)       → Result<()>    (shell)

lexical.rs
  └── tokenize_with_spans          → Vec<TokenSpan>  (NEW — shared span authority)
      tokenize                     → Vec<String>      (now a projection over tokenize_with_spans)
```

Dependencies: `lexical` (leaf — `Bm25Ranker`, `LexDoc`, `LexicalCorpus`,
`tokenize`), `listing` (leaf — table primitives), `entity` (engine),
`catalog::scan` (command), `integrity` (command). No upward edges: command
→ engine is allowed, command → command is within-tier. No new crates.

### Tangle impact (ADR-001 ratchet)

`search.rs` introduces 3 new command→command edges into the existing 32-module
command SCC (baseline frozen at 120 cyclic edges in `layering.toml`):

| edge | direction |
|------|----------|
| `search` → `catalog::scan` | command → command |
| `search` → `integrity` | command → command |
| `commands::cli` → `search` | command → command (dispatch wiring) |

All three connect into the SCC, so they increment the cyclic edge count.
**Mitigation:** before implementation, a pre-commit cleanup removes 3
command→command edges elsewhere to keep the 120 ceiling, or the baseline is
bumped to 123 with an explicit `[tangle_baseline]` increase in the PR.

### catalog::scan change

`scan_entities` gains a `ScanMode` parameter to avoid reading `.md` bytes in
consumers that don't need them (inspect, priority, graph, catalog dump):

```rust
pub(crate) struct ScanMode {
    pub include_bodies: bool,
}

pub(crate) fn scan_entities(
    root: &Path,
    diagnostics: &mut Vec<CatalogDiagnostic>,
    mode: ScanMode,
) -> anyhow::Result<Vec<ScannedEntity>> {
```

`ScannedEntity` gains `pub body: Option<String>`. Read from
`entity::id_path(root, kref.kind, id, Ext::Md)`. Body-read policy:
- **Missing file** → `body = None` (no diagnostic — many kinds legitimately
  lack `.md` bodies).
- **Non-missing read error** (permission, I/O) or **invalid UTF-8** →
  `CatalogDiagnostic::Warning` at the body path + `body = None`. Scan continues;
  one corrupt body never poisons the full search.
When `include_bodies` is false, field is `None`.

~10 existing call sites pass `ScanMode { include_bodies: false }` — zero
behaviour change. Only `search::build_corpus` passes `true`.

`catalog::hydrate::CatalogEntity` gains `body: Option<String>`, carried
verbatim. Serialization excluded by default from catalog JSON dump (explicit
`--include-bodies` could follow later; out of scope).

## Data flow

```
1. CLI → SearchArgs
2. KindSelector::resolve(default, --kind, --with, --no) → valid prefixes
3. scan_entities(root, diagnostics, ScanMode { include_bodies: true })
   → Vec<ScannedEntity>
4. Filter by KindSelector (prefix match against entity key)
5. Build LexDoc per entity:
     id   = canonical ref string ("SL-023")
     text = title + " " + body.unwrap_or_default()
6. LexicalCorpus::Raw(&docs), Bm25Ranker.score(query, corpus, &all_ids)
7. Sort descending by score, tiebreak canonical id ascending
8. Apply --limit / --offset / --page
9. Format output (table or JSON, optional --context snippets)
```

Key properties:

- **Corpus = all matching entities** — BM25 fits IDF over the full corpus, not
  just query-hits. Identical pattern to `retrieve::run_find`.
- **No scope/staleness/exact-key dimensions** — unlike memory retrieve, entity
  search is purely lexical. Single sort component: BM25 score.
- **Empty body → title-only.** Missing `.md` → `body = None` → text bag is
  title only. BM25 handles short docs naturally (zero TF for most terms).
- **No dedup concerns.** Entity ids are unique by construction (canonical refs).
  Reading by explicit `entity::id_path` not `read_dir` walk avoids the symlink
  alias gotcha.
- **Deterministic.** Permutation-invariant fit corpus + canonical-id tiebreak
  → byte-identical output for same query on same corpus.

## Kind selector

Three flags define the effective set:

| flag | behaviour |
|------|-----------|
| `--kinds sl,adr,backlog` | Replace default — no default kinds included |
| `--with rfc,review` | Add to effective set (repeatable) |
| `--no ide,rsk` | Remove from effective set (repeatable) |

Evaluation order: default set → `--kinds` replaces → `--with` adds → `--no`
removes. `--kinds` and absent-`--kinds` are mutually exclusive in effect:
`--kinds` means the default is discarded.

Values: canonical kind prefixes (`sl`, `adr`, `prd`, `spec`, `rfc`, `iss`,
`imp`, `chr`, `rsk`, `ide`, `rv`, `rec`, `req`, `rev`, `cm`, `asm`, `dec`,
`que`, `con`) plus group aliases:

| alias | expands to |
|-------|-----------|
| `backlog` | `iss`, `imp`, `chr`, `rsk`, `ide` |
| `governance` | `adr`, `pol`, `std` |
| `specs` | `prd`, `spec` |
| `knowledge` | `asm`, `dec`, `que`, `con` |
| `all` | every numbered kind in `integrity::KINDS` (excl. memory) |

Unknown prefix → hard error listing valid prefixes + aliases.

### Default set

```rust
const DEFAULT_SEARCH_KINDS: &[&str] = &[
    "sl", "adr", "prd", "spec", "rfc",
    "iss", "imp", "chr", "rsk", "ide",
    "asm", "dec", "que", "con",
];
```

Explicit exclusions from default: `pol`, `std`, `rv`, `rec`, `req`, `rev`,
`cm`. Rationale: REQ bodies are thin; RV/REC are process byproducts; POL/STD
are rare; REV is change-axis metadata; CM is graph not prose.

No status filtering on results — status is display-only (table column, JSON
field). Follow-up could add `--status` filter.

## CLI surface

```
doctrine search [OPTIONS] <QUERY>

Arguments:
  <QUERY>  Free-text lexical query (required)

Options:
  --kinds <KINDS>   Replace default search kinds (comma-separated prefixes/aliases)
  --with <KINDS>    Add kinds to the effective set (repeatable)
  --no <KINDS>      Remove kinds from the effective set (repeatable)
  --format <FMT>    Output format [default: table] [possible values: table, json]
  --context         Show body snippet for each result
  --limit <LIMIT>   Max results to show
  --offset <OFFSET> Skip first N results [default: 0]
  -p, --path <PATH> Explicit project root (default: auto-detect)
  --color <COLOR>   Control colour output [default: auto]
  -h, --help
```

Pagination: `--limit` defaults to 20, capped at 100. `--offset` skips the
first N results (default 0).

## Output formats

### Table

```
QUERY: "auth token"  (3 results, 0.004s)

ID        KIND     STATUS    SCORE    TITLE
SL-023    slice    done      984231   Auth token rotation logic
ADR-005   adr      accepted  872140   Adversarial review protocol
```

Columns: canonical ID, kind label, status, BM25 score (u32 quantized), title.

`--context` interleaves snippet rows:

```
ID        KIND     STATUS    SCORE    TITLE
SL-023    slice    done      984231   Auth token rotation logic
                                       ...token expiry check in middleware, rotating auth tokens...
ADR-005   adr      accepted  872140   Adversarial review protocol
                                       ...the adversarial review protocol requires every finding...
```

Snippet extraction uses shared lexical machinery. `lexical.rs` exposes a new
`tokenize_with_spans(&str) -> Vec<TokenSpan>` — the ONE authority for
split-on-non-alphanumeric tokenization with byte-offset spans. `tokenize`
becomes the token-only projection:

```rust
pub(crate) struct TokenSpan {
    pub token: String,   // lowercased
    pub start: usize,    // byte offset in original
    pub end: usize,      // exclusive
}

pub(crate) fn tokenize_with_spans(s: &str) -> Vec<TokenSpan> { ... }
pub(crate) fn tokenize(s: &str) -> Vec<String> {
    tokenize_with_spans(s).into_iter().map(|ts| ts.token).collect()
}
```

The snippet function:
1. `let body_spans = tokenize_with_spans(body);`
2. Let `query_tokens: BTreeSet<String>` = `tokenize(query)`.
3. Find first `body_spans` entry where `ts.token` ∈ `query_tokens`.
4. Extract window: `body[ts.start.saturating_sub(40)..(ts.end + 40)]`,
   with "..." ellipsis at boundaries.

No match → first 120 chars of original body. Empty query/body → None.

Proof tests: `tokenize(s) == tokenize_with_spans(s).map(|ts| ts.token)` for
varied inputs (behaviour preservation); Unicode multi-byte boundaries don't
split inside a codepoint; existing `lexical` tests pass unchanged.

### JSON

```json
{
  "query": "auth token",
  "count": 3,
  "results": [
    {
      "id": "SL-023",
      "kind": "slice",
      "status": "done",
      "score": 984231,
      "title": "Auth token rotation logic"
    }
  ]
}
```

`--context` adds `"snippet"` field.

## Verification alignment

### New tests (unit, `src/search.rs`)

- `KindSelector::resolve` — default-only, replace, add, remove, combined,
  unknown prefix error, group alias expansion (expand then validate)
- `entity_lex_doc` — id canonical, title present, body concatenated, None body
  handled
- `snippet` — uses `tokenize_with_spans` for one-authority span extraction;
  match → correct byte-offset window; no match → body prefix; empty inputs → None;
  Unicode multi-byte boundary correctness
- zero-score suppression: row with score 0 is dropped from results
- `build_corpus` — with a seeded project dir: reads bodies correctly, handles
  missing body files (None), permission/UTF-8 errors (diagnostic + None),
  filters by kind selector

### Existing tests (behaviour preservation)

- `catalog::scan` tests pass unchanged — `ScanMode { include_bodies: false }`
  yields `body: None` in all existing assertions
- `catalog::hydrate` tests pass — `body: None` is additive, doesn't break
  existing shape assertions. New test: `CatalogEntity` with non-empty body
  serializes to JSON without `"body"` key (serde skip guard).
- `lexical` unit tests pass unchanged — no change to `Bm25Ranker` or `LexDoc`
- `retrieve` tests pass unchanged — no change to memory path

### Integration tests

- `doctrine search "auth" --format json` returns valid JSON with expected shape
- `doctrine search "nonexistent_token"` returns zero results (all scores 0 → suppressed)
- `doctrine search "auth" --kind adr --format table` includes ADR column headers
- `doctrine search --help` renders without error (follows existing CLI help-test pattern)
- `doctrine search "auth" --with req --no adr` additive/subtractive logic
- `doctrine search "auth" --context --format json` returns valid JSON with snippet fields
- `doctrine search "auth" --format table` prints table without panicking
- `just gate` green — zero clippy warnings, layering gate passes

## Risks & mitigations

| risk | mitigation |
|------|-----------|
| Disk I/O: reading all `.md` during scan doubles touches | Already reads all `.toml`; `.md` files are small. `ScanMode` gates it per-caller |
| Memory: all body text in one `Vec<LexDoc>` | Entity corpora are small (hundreds, not millions). Revisit if projects grow 10x |
| Snippet quality: naive token-matching window is crude | Acceptable for v1; follow-up: sentence boundaries, proper highlighting |
| `--kind` values coupling to `integrity::KINDS` | KindSelector validates against `integrity::kind_by_prefix` — new kinds auto-register |
| `spec` alias collision with SPEC prefix | Renamed to `specs` (plural). D7 records the decision. |

## Open questions

1. **Template noise (follow-up):** many entities start from templates with
   boilerplate text ("## Context", "## Scope & Objectives"). These tokens
   match every query containing "scope" or "context". Follow-up: detect and
   strip template boilerplate before indexing, or score-penalise tokens that
   appear in >90% of bodies.

2. **MCP surface (follow-up):** memory has `mcp_find`/`mcp_retrieve` for agent
   consumption. Entity search could expose a similar MCP handler for agents
   querying the corpus.

3. **Body field in catalog JSON (out of scope):** `CatalogEntity.body` is
   serialized but excluded by default. A future `catalog scan --include-bodies`
   flag could expose it for debugging.

## Design decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | `ScanMode` opt-in for bodies | Don't pay disk I/O where not needed; one walk, no duplication |
| D2 | BM25 fit corpus = all matching entities | Correct IDF computation; memory retrieve pattern |
| D3 | No status filtering in v1 | Simpler surface; status is display-only; follow-up can add `--status` |
| D4 | `--kind` replaces default, `--with`/`--no` modify | Follows existing doctrine flag philosophy (flags are authoritative) |
| D5 | Snippet via token-window, no sentence boundaries | Simple, testable; sentence awareness is a follow-up |

| D7 | Canonical-id tiebreak | Deterministic output; stable across invocations on same corpus |
| D8 | Suppress zero-score rows | Entity search has no scope-filter pre-pass; 0-score-everything is noise |
| D9 | `specs` alias (plural) avoids SPEC prefix collision | `--kind spec` would resolve to [prd, spec], user may have meant tech-spec only. `specs` is unambiguous. `--kind spec --no prd` for tech-specs only. |
