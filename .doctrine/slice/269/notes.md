# Notes SL-269: Review-capable worktree locus

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-27 · design:reviewing · 0cf1eaf21

### Produced
- re-scoped as RFC-032 slice 4 (D9 → D7) (22dbfa6f8); ISS-484 fixed outside the slice (120b8b321, 0e2c52208)
- research distilled: `research/research.md` (gitignored); no contradiction with DEC-337/DEC-338/ASM-012
- design sec-1..sec-8 drafted, materialised, committed (commits after cfaebf913 through 3bed716b9); 26 design-target selectors recorded
- scope item 3 rewritten to DEC-338; item 7 adds ADR-007 D-C10; item 2 adds ISS-292 faults 1/3/4
- inq-12 (user) folds ISS-292 faults 1/3/4 into sec-3, superseding inq-5's out-of-scope ruling; SL-269 fulfils ISS-496, partly ISS-292
- review pass ledger RV-406 opened by the run, no findings yet; status reports it STALE (sections changed after it opened)
- no code yet; gate last green at 120b8b321

### Learned
- mem.pattern.review.mcp-bypasses-worker-guard — review admission must live in `resolve_review_root`
- obs friction — `design apply`: a node and its checkpoint disposal cannot share one submission

### Open
- DEC-337 — clone-wide local reservation (ref CAS + live-worktree scan)
- DEC-338 — review admission refused only in a worker process
- ASM-012 — design-run mint relies on the CLI worker guard (no MCP design tools)
- ISS-292 — faults 2, 5 and structured-edge rewrite stay out of scope
