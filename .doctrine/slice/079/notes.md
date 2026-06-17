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

## 2026-06-17 — Implementation complete (dispatch, self-orchestrated)

All three phases implemented directly in worktree forks (no codex subprocess):

- **PHASE-01** — IMP-038 (debug_assert! in select_columns) + IMP-040
  (resolve_color, --color flag, into_list_args(color:) injection). 89 + 22 -
  lines. All 1551 unit tests pass; clippy clean.
- **PHASE-02** — IMP-039a (SURVEY_COLS/NEXT_COLS routed through
  render_columns). 103 + 40 - lines. RSK-1 confirmed: 16/16 e2e priority
  goldens byte-identical under color:false.
- **PHASE-03** — IMP-039b (status_colored helper + 5 status-bearing writeln!
  sites). 96 + 17 - lines. 4 new status_colored tests (1555 total pass).

Total delta: ~285 lines across 10 files. Funneled as three commits on
`dispatch/079` via cherry-pick (S^ == B in all cases). Coordination branch
at `e82069b`.

Outstanding:
- 3 e2e_adr_cli_golden tests fail on worker forks (Write-classed verbs
  blocked by marker). They pass on the coordination worktree (markerless).
  Not a regression.
- IMP-044 (RenderOpts migration for priority) deferred per scope.
- IMP-056 (coverage kebab-case) deferred per scope.

## 2026-06-17 — RV-050 reconciliation + code-review (audit)

- Opened RV-050 against SL-079 with facet `reconciliation`; primed with 6
  areas, 12 tracked paths, 5 invariants, 2 risks.
- Three findings raised: F-1 (major) — `coverage_view::run` bypassed --color
  flag; F-2 (minor) — `run_blockers`/`run_explain` accept unused RenderOpts;
  F-3 (nit) — doubled deref in debug_assert! error message.
- **F-1 fix-now:** `coverage_view::run()` at line 452 was still calling
  `stdout_color_enabled()` directly — the only surface missed in the PHASE-01
  flag-wiring pass. Fixed by adding `color: bool` parameter to
  `coverage_view::run()` and passing `resolve_color(cli.color)` from main.rs.
  Committed as `fix(SL-079): wire --color flag into coverage_view::run` on
  `dispatch/079` (commit `7ce4f37`).
- **F-2 tolerated:** Design explicitly says blockers/explain are prose;
  `_render` prefix signals unused per Rust convention.
- **F-3 aligned:** Style observation only — `*d` vs `**d` in closure produces
  identical code.
- Review status: `done · await=none`. No unresolved blockers.
- `cargo test coverage_view` — 9/9 pass post-fix.
