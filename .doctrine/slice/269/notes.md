# Notes SL-269: Review-capable worktree locus

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Review passes

After RV-406 (Codex adversarial pass, 8 findings, all disposed): no further pass
needed; user agreed. A further pass would only re-probe the rewritten sec-2/sec-3
failure paths (the post-CAS `mkdir` table, the `update_ref_cas` error split,
reseat's pre/post-commit cleanup boundary, the no-follow dangler walk). Those are
bounded and pinned by sec-7 tests. The residual risk is the accepted gap in
DEC-337, which review cannot reduce.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-28 · close · edbbc26c8

### Produced
- re-scoped as RFC-032 slice 4 (D9 → D7) (22dbfa6f8); ISS-484 fixed outside the slice (120b8b321, 0e2c52208)
- research distilled: `research/research.md` (gitignored); no contradiction with DEC-337/DEC-338/ASM-012
- design sec-1..sec-8 drafted, materialised, committed (commits after cfaebf913 through 3bed716b9); 26 design-target selectors recorded
- scope item 3 rewritten to DEC-338; item 7 adds ADR-007 D-C10; item 2 adds ISS-292 faults 1/3/4
- inq-12 (user) folds ISS-292 faults 1/3/4 into sec-3, superseding inq-5's out-of-scope ruling; SL-269 fulfils ISS-496, partly ISS-292
- review pass ledger RV-406 opened by the run, no findings yet; status reports it STALE (sections changed after it opened)
- PHASE-01..06 implemented (39b174ece..9dbe1b8ae); gate green at close
- RV-409 audit + REV-068 (ADR-007, PRD-005, SPEC-008) done; ISS-499 captured

### Learned
- mem.pattern.review.mcp-bypasses-worker-guard — review admission must live in `resolve_review_root`
- obs friction — `design apply`: a node and its checkpoint disposal cannot share one submission

### Open
- DEC-337, DEC-338 — accepted; implemented
- ASM-012 — design-run mint relies on the CLI worker guard (no MCP design tools)
- ISS-292 — faults 2, 5 and structured-edge rewrite stay out of scope

### Audit (RV-409, 2026-09-28)
- RV-409 opened and driven in the adopted capsule worktree with the slice binary (live demo of DEC-338 + DEC-337); 4 findings, all `design-wrong`, handed to /reconcile via its Reconciliation Brief
- gate green; `slice verify-vt` 25/25 PASS
- RSK-233 — clone-ref reservation writes into an ambient git repo above `TMPDIR` (PHASE-02/03 risk, unfixed)
- ISS-463 — the EPIPE-flaky e2e_observation test seen in PHASE-05 (already captured)
- residual: `worker_process()`'s production arm (`env_worker_set()`) has no unit test by construction; it is the only barrier for MCP review writes in a worker
- transition window: a pre-slice binary on edge ignores `reservation-local` refs; RV/RSK ids minted here (RV-409, RSK-233) could collide with edge mints before the merge
