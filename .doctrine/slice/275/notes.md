# Notes SL-275: Memory search: default page, retrieval floor, and a zero-evidence signal

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-29 · PHASE-01 · (this commit)

### Produced
- PHASE-01: `SEARCH_LIMIT_DEFAULT`; `memory::resolve_limit` + `memory::page_offset`
  (shared by Search/Retrieve arms); `run_search`/`search_for_mcp` take `usize`;
  `format_truncation_notice` next-row continuation; MCP edge `limit 0` refusal,
  `checked_add` `next_offset`; `temp_project_with_memories(n)` fixture.

### Learned
- `plan.toml` PHASE-01 header had been swallowed by a template comment; the plan
  loader accepted the orphan keys silently (friction obs `01a0eac4`).
- `resolve_limit` absorbed the duplicated `--limit 0` check from both arms, beyond
  the design's `page_offset` extraction — same edge, one site.

### Open
- MCP tool description reads "Default page: 20 rows." as an interim; the
  find/browse contract text is PHASE-02 EX-5.

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
