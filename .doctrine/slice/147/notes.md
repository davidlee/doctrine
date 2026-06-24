# Notes SL-147: Audit path-conformance delta

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Audit harvest (RV-157, 2026-06-24)

Audited the candidate surface `candidate/147/review-001` (`ad0a75f5`) off the
impl bundle `review/147` (`186c2ef5`). `just check` green. 5 findings, all
terminal; no blocker. Synthesis + reconciliation brief in `review-157.md`.

Durable carries (→ reconcile / close):

- **R-D5 / F-1 — design D5 double-write topology stale.** Shipped recording is
  arm-asymmetric: claude `dispatch record-boundary` (`dispatch.rs:552-560`)
  double-writes the committed ref-cut ledger AND the arm-neutral conformance
  registry in one call; codex/pi (no `record-boundary`) uses the separate
  `slice record-delta`. Skills reconciled in P06; `design.md` D5 not. Decision
  intact. → reconcile: per-slice direct edit to design.md D5.
- **F-2 — primary_worktree home pointer stale.** Resolver relocated to
  `git::primary_worktree` (ADR-001-clean leaf) in P02; design D5/R5/F-5/
  OQ-conf-3/D7 still cite `worktree::subagent::primary_worktree`. Decision
  intact. → reconcile: per-slice direct edit.
- **F-3 — bundle cut from stale `main` (edge +28).** Overlaps RFC-005/ISS-025
  in review/dispatch/state. → /close stage-2: promote `edge`→`main`, merge onto
  current edge before integrate; expect conflicts.
- **F-4 (aligned)** — layering.toml rows orchestrator-authored in funnel
  (sanctioned, handover L74-76); not a worker R-5 leak.
- **F-5 (aligned)** — `slice conformance 147` not runnable live: jail-wide
  `icu_provider` E0599 on any fresh-target bin build (unrelated to SL-147) +
  shared-target stale-bin footgun. Verb correctness held via green `just check`
  + recorded P06 EX-2 dogfood.
