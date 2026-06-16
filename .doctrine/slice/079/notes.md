# Notes SL-079: Finish the CLI colour story: deferred surfaces + --color flag

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## 2026-06-16 — RV-046 plan inquisition

- Opened `RV-046` against `SL-079` with facet `plan` and raiser label
  `inquisitor`; review status is `done · await=none`.
- Seven findings were raised and terminally verified. The plan should not move
  to `/phase-plan` until the penance in `RV-046` synthesis is applied.
- Main corrections required: fix PHASE-03 `proposed` colour criteria; reconcile
  the `clap::ColorChoice` case-sensitivity claim; make `resolve_color`
  verification executable; name the status-line color injection seam; replace
  live mutable VA commands with fixture-safe checks; reconcile stale
  install/reconcile/corpus scope text; add PHASE-01 integration evidence for
  `--color=always|never` on an existing list surface.
- Recorded memory `mem.fact.clap.colorchoice-case-sensitive` for the clap parser
  gotcha. It is unverified because the working tree is dirty.
- Verification run: `doctrine validate` passed (`validate: corpus clean`). No
  code tests were run; this was a plan/review-only task.
- Review and memory changes are uncommitted; the worktree already contained
  unrelated dirty files before this review.
