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

## 2026-06-17 — Penance applied (RV-046 synthesis)

All seven F-1 through F-7 findings from RV-046 applied:

- **F-1 (fix-now):** PHASE-03 EX-2: ADR `proposed` is plain (unmapped), not
  yellow; `accepted` is green. VT-4: complete `status_hue` mapped set (13
  tokens across green/yellow/red).
- **F-2 (design-wrong):** Removed "case-insensitive" claim from design.md §5
  and plan.toml VT-3. clap accepts lowercase value enum tokens only without
  `ignore_case = true`. Memory `mem.fact.clap.colorchoice-case-sensitive` records
  the gotcha.
- **F-3 (design-wrong):** Narrowed VT-2 to direct `Never`/`Always` tests +
  `Auto` delegates to `stdout_color_enabled`; existing `color_enabled` tests
  cover the NO_COLOR/tty matrix. Design §7 updated.
- **F-4 (fix-now):** Added EX-9 to PHASE-03: `main.rs` resolves
  `tty::resolve_color(cli.color)` at each status command arm and passes
  `color: bool` into the five `run_status` handlers.
- **F-5 (fix-now):** Replaced PHASE-03 VA-1/VA-2/VA-3 with temp-root fixture
  commands using actual CLI shapes and bound fixture ids.
- **F-6 (design-wrong):** Replaced install/reconcile/corpus with
  standard/knowledge/revision in slice-079.md affected-surface and closure-intent
  sections (three locations).
- **F-7 (fix-now):** Added VT-5 to PHASE-01: integration evidence for
  `--color=always`/`--color=never` on at least one existing `CommonListArgs`
  surface.

`doctrine validate` passes clean. Plan is now executable — ready for
`/phase-plan`.
