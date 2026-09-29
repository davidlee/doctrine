# Notes SL-275: Memory search: default page, retrieval floor, and a zero-evidence signal

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-29 · PHASE-02 + RV-412 · 3544168c7

### Produced
- PHASE-01: `SEARCH_LIMIT_DEFAULT`; `memory::resolve_limit` + `memory::page_offset`
  (shared by Search/Retrieve arms); `run_search`/`search_for_mcp` take `usize`;
  `format_truncation_notice` next-row continuation; MCP edge `limit 0` refusal,
  `checked_add` `next_offset`; `temp_project_with_memories(n)` fixture.
- PHASE-02: `QueryContext::has_free_text`, `Candidate::has_evidence`, floor in
  `query()`; `listing::format_no_match_notice` + retrieve-private
  `no_match_notice` (scrubs; keyed pre-holdback); MCP `memory_search`
  find/browse description; `record_fact` / `search_out` / `retrieve_out` test
  helpers. Live: `memory search bwrap` 565 → 10 rows.
- PHASE-03: shipped guidance — `retrieve-memory` step 1 (find vs browse, empty =
  no match, page 20 / `next_offset`), `record-memory` §7 (own terms or `--page`),
  `dreaming` sweep (`--limit`/`--page`), `using-doctrine.md` rows 23/33/48
  ("free text is floored; selectors alone browse"). Also fixed a stale `find`
  → `search` in retrieve-memory procedure step 3.
- RV-412 (code review PHASE-01/02, pi-research): concluded; 6 fixed
  (`a2bd4d332`), F-7 tolerated. ISS-503 filed (priority page-offset overflow).

### Learned
- `plan.toml` PHASE-01 header had been swallowed by a template comment; the plan
  loader accepted the orphan keys silently (friction obs `01a0eac4`).
- `resolve_limit` absorbed the duplicated `--limit 0` check from both arms, beyond
  the design's `page_offset` extraction — same edge, one site.

- `listing.rs` is a leaf (no `crate::memory` edge), so the notice formatter
  takes pre-scrubbed text; the scrub lives in the one retrieve-side caller.
- `query_bare_query_keeps_all_active_ranked_lexically` pinned SL-008 D20's
  keep-all; renamed and its count flipped to 1 under DEC-348 (ordering witness
  kept). The only pre-existing test the floor changed.

### Verification (agent)
- PHASE-02 VA-1: MCP `memory_search` description + `query`/`limit` schema text
  read against design sec-4 — states free text floored to lexical/exact-key
  evidence, empty rows = no match, selectors alone browse in severity order,
  default page 20. Pass.
- PHASE-03 VA-1: the four guidance edits read against design sec-2 "Shipped
  guidance" — each clause present and matching live behaviour (MCP param is
  `offset`/`next_offset`, CLI `--page`). ADR-024 sweep: no entity ids in added
  shipped lines. Pass.

### Open
- Slice ready for `/audit`.
- ISS-503 — out of slice scope.
- CHR-171 — deferred spec-text drift (design sec-5).

## Design review passes

- **RV-410** (codex `gpt-6-sol`, high): 8 findings, all fix-now, all verified,
  pass concluded 2026-09-29. F-2 (retrieve `--page` from uncapped limit) and F-4
  (MCP `offset + cap` overflow) are live defects today, not design-only.
- **Further pass: none needed.** The two blockers and F-8 all sat in one small
  surface — continuation arithmetic — and the final verify probed its edge cases
  (offset ≥ total, total 0, aligned/unaligned, final page) across all three
  `format_truncation_notice` callers. The rest of the design is a filter predicate
  plus deletions, which is covered by the behaviour-preservation suites. What a
  further pass *would* probe: the MCP retrieve path's reuse of the table renderer
  under the notice (`tools.rs:967-990`), and whether `has_free_text` should use the
  ranker's tokenizer rather than `lexical::tokenize` (identical today, by design D2).
  Both belong to implementation review, not design.
