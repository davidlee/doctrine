# Review RV-050 — reconciliation of SL-079

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reconciliation + code-review of SL-079 (Finish the CLI colour story: deferred
surfaces + --color flag). Three phases implemented across 10 source files (~285
lines net), reviewed against the coordination-branch surface at
`refs/heads/dispatch/079` (tip `e82069b`). No candidate surface created — the
review/079 impl-bundle ref failed verification; the coordination branch is the
authoritative implementation surface.

**Lines of attack:**

1. **--color flag completeness** — verify every call site that resolves colour
   either passes `resolve_color(cli.color)` or passes through a `CommonListArgs`
   that accepts the injected `color: bool`. Any surface still calling
   `stdout_color_enabled()` directly is a fencepost bug.
2. **Priority routing conformance to D1** — `survey_human`/`next_human` must
   route through `render_columns` with `SURVEY_COLS`/`NEXT_COLS`, not through
   `render_table` directly.
3. **Status-line colour conformance to D2** — five surfaces (adr, policy,
   standard, knowledge, revision) must use `status_colored` with the shared
   `status_hue`; revision must emit two separate `status_colored` calls joined
   by `" → "`.
4. **Behaviour-preservation** — goldens must stay byte-identical under
   `color: false`; 1548 tests must stay green.
5. **Pure/imperative split** — D3: no `if_supports_color` in the pure layer;
   `status_colored` gated on injected `bool`.

## Synthesis

**Overall:** solid

**Synopsis:** SL-079 delivers on its three objectives cleanly. The column model
validation (IMP-038) is a tight `debug_assert!` at the right call point — catches
misconfigured defaults at test time with a clear message naming the offender and
the valid set. The `--color` flag (IMP-040) is a thin `resolve_color` wrapper that
respects the `Never > Always > auto-detect` precedence, and the injection seam is
consistent: `CommonListArgs::into_list_args(color:)` for list surfaces, direct
`RenderOpts` construction for priority, and `color: bool` parameter for status-line
handlers. The priority routing through `render_columns` (IMP-039a) replaces two
hand-built grids with `SURVEY_COLS`/`NEXT_COLS` column arrays — the empty-list
guard is preserved, and every cell inherits the shared colour/style pipeline for
free. The `status_colored` helper (IMP-039b) is a pure sibling to `status_hue`,
and the five status-line surfaces each wrap correctly (including revision's
dual-token `from → to` pattern).

One fencepost bug was found and fixed during review: `coverage_view::run` still
called `stdout_color_enabled()` directly, bypassing the `--color` flag. This was
an omission in the flag-wiring pass because `coverage_view` uses its own
`render_table` wrapper rather than `CommonListArgs`. Fixed in
`fix(SL-079): wire --color flag into coverage_view::run` on `dispatch/079`.

**Standing risks:**
- IMP-044 (RenderOpts migration for priority surfaces) remains deferred — the
  `&cols.iter().collect::<Vec<_>>()` allocation is harmless but acknowledged.
- `run_blockers`/`run_explain` accept a `RenderOpts` parameter they never use;
  this is a conscious uniformity tradeoff (symbol-prefixed `_render`).
- Three `e2e_adr_cli_golden` tests fail on worker-marked forks (Write-classed
  verbs blocked) — they pass on markerless coordination worktrees; not a
  regression.

**Tradeoffs consciously accepted:**
- `#[expect(dead_code)]` on `SURVEY_DEFAULT`/`NEXT_DEFAULT` — declared for
  IMP-038 validation parity but unused at render time (priority has no
  `--columns` surface). The attribute will fire an error if they ever become
  used, which is the desired signal.
- `status_hue` map unchanged — the existing conservative subset covers every
  token the five targeted surfaces emit. `proposed`/`superseded`/`deprecated`/
  `recommended`/`optional` stay grey by design.

**Haiku:**

Colour deferred blooms —
twelve call sites drink from one flag,
one missed, now found.


