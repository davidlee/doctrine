# SL-141 close-out notes

## What was built

Entity full-text search via `doctrine search <query>`. Three phases:

- **PHASE-01**: Extracted `tokenize_with_spans(&str) -> Vec<TokenSpan>` from
  `tokenize` in `src/lexical.rs`. `tokenize` is now a thin projection. The
  shared span authority is the single source of truth for tokenization.
- **PHASE-02**: Added `ScanMode { include_bodies: bool }` to the catalog scan.
  Body reading is gated — only search pays the I/O cost. ~22 call sites
  mechanically updated.
- **PHASE-03**: New `src/search.rs` module with `KindSelector`,
  `entity_lex_doc`, snippet extraction, BM25 orchestration, and CLI wiring.
  ~680 lines.

## Key decisions

- **Default kind set**: slices, ADRs, specs, RFCs, all five backlog kinds.
  Knowledge records (ASM, DEC, QUE, CON) excluded from defaults but
  includable via `--with`. Reviews, revisions, RECs, concept maps excluded
  (process byproducts).
- **Group aliases**: `backlog` (5 kinds), `governance` (2), `specs` (2),
  `knowledge` (4), `all` (every prefix). Resolved at parse time.
- **Snippet context**: hardcoded at 40 chars (design said 80; plan narrowed
  to 40 during F-3 penance for consistency). Should become configurable.
- **JSON kind field**: uses lowercase prefix (`"sl"`) not kind label
  (`"slice"`). Extra fields (`total`, `prefix`, `status`) added for utility.

## Notable findings (from review ledgers)

### RV-140 (design inquisition, 6 findings)
- F-1 (major): snippet span-reconstruction algorithm underspecified →
  fixed by adding `tokenize_with_spans` as shared authority (PHASE-01).
- F-2 (minor): design misstated catalog::scan/integrity/listing tier labels
  → corrected in design.md.
- F-3 (minor): `spec` alias collision with SPEC prefix → renamed to `specs`.
- F-4 (minor): missing page-boundary edge tests → added to verification.
- F-5 (major): command-tier tangle ratchet analysis missing → documented
  and baseline bumped from 120→123.
- F-6 (minor): missing serde skip test → added.

### RV-141 (inquisition round 2, 4 findings)
- F-1 (major): two-pass snippet algorithm duplicated tokenizer → resolved
  by `tokenize_with_spans` shared authority.
- F-2 (major): pagination design contradicted memory-find/retrieve →
  resolved: search-specific SEARCH_LIMIT_DEFAULT=20, SEARCH_LIMIT_MAX=100.
- F-3 (minor): body-read error policy incomplete → specified: missing→None,
  error→diagnostic+None.
- F-4 (nit): help/flag precedence tests missing → added.

### RV-142 (audit reconciliation, 5 findings)
- F-1 (minor, tolerated): table-driven vs property test — form vs substance.
- F-2 (minor, tolerated): --json/--page flags in design not in plan →
  design trimmed to match plan.
- F-3 (nit, tolerated): context 40 vs 80 chars.
- F-4 (nit, tolerated): JSON kind=prefix vs kind=label.
- F-5 (minor, fix-now): snippet ellipsis off-by-one (`window_start + 1`)
  — genuine bug, fixed.

## Follow-up work (not blocking)

- **Property test for tokenize equivalence** — RV-142 F-1 tolerated,
  but a formal proptest would be more robust.
- **Template boilerplate detection** — RSK-003; common tokens like
  "scope", "context" inflate scores.
- **MCP surface** — like memory search has.
- **Persisted index** — for large corpora.
- **Hoist group aliases** to `integrity.rs`.
- **Configurable snippet context** — CLI flag for context window size.
- **--json shorthand, --page flag** — convenience UX from design spec.
