# Review RV-126 — reconciliation of SL-132

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

This is a **conformance audit** of SL-132 — a self-audit driving both roles
(`--as auditor` for raise/verify, `--as responder` for dispose).

**Lines of attack:**

1. **Design conformance.** Does every design decision (D1–D9) materialise in
   the implementation? Specifically: D1 (full DoctrineToml parse), D2 (confidence
   formula correctness), D3 (absent → no row, golden byte-identical), D4
   (no value confidence), D5 (EntityFacets wraps existing data), D7 (module-level
   gate removed, function-level gates added), D9 (ADR-001 leaf tier entry).

2. **Verification gate.** Do all 12 VT items hold? VT-5 (golden absent-facet
   byte-identical) as the pivotal test — if this passes, the change is strictly
   additive. VT-11/12 (shell integration) and VT-1 through VT-4 (unit formatter
   correctness). VT-6 (no dead_code), VT-7 (JSON unchanged), VT-8 (just gate).

3. **ADR-001 layering.** `src/facet.rs` is leaf tier, imports only estimate +
   value (both leaf). `src/value.rs` format_value_normal is pure — no I/O, no
   clock. `run_show` resolves units/confidence in the shell tier; passed as pure
   values to `format_show` (engine). The pure/impure boundary follows ADR-001.

4. **Code quality.** No regression — `load_conduct` helper preserved for status
   transitions. No dead_code warnings. No parallel implementation (reuse of
   existing `resolve_unit`, `resolve_confidence`, `SliceDoc` fields). EntityFacets
   is a pure aggregation struct, not a new parse path.

5. **Risk surface.** `format_estimate_confidence` uses `debug_assert` for bound
   validity — resolved by `resolve_confidence` at the shell boundary before
   calling, but a future caller that bypasses that guard would hit a release-mode
   panic. `format_show` now accepts 9 parameters — the `too_many_arguments`
   clippy expect is reasoned, but worth inspecting alternatives.

**Invariants held:**

- No estimate/value line when facets are absent (D3).
- Byte-identical output for unauthored slices (VT-5 as gate).
- JSON output unchanged (VT-7).
- No new parse path (D5).
- Pure/impure split preserved (ADR-001).
- Existing tests green and unchanged (VT-7, VT-8).

## Synthesis

SL-132 is a clean, tight implementation. The change is ~510 lines across 7 files
and every design decision (D1–D9) materialises faithfully.

**Design conformance is complete.** D1 (full `DoctrineToml` parse) replaces
`load_conduct` in the single `run_show` call path — a one-line upgrade that
yields both `ConductConfig` (via `.conduct`) and estimation/value config
(via `.estimation`, `.value`). No extra disk read. D2 (confidence formula) is
identical to the design: `lo = lower + lower_pct × width`, `hi = lower + upper_pct
× width`. D3 (absent → no row) is enforced by `if let Some(…)` guards; VT-5
proves byte-identical output for unauthored slices. D4 (no value confidence) is
respected — `format_value_normal` renders a single magnitude. D5 (EntityFacets
wraps existing data) holds: no new parse path introduced. D7 (module-level gate
removed, function-level gates added) is verified; `format_estimate_normal` and
`format_estimate_verbose` survive with function-level `#[expect(dead_code)]`.
D9 (ADR-001 leaf tier) — the `facet = "leaf"` entry exists in `layering.toml`.

**Verification is exhaustive.** All 12 VT items pass. VT-5 (golden absent-facet)
was built first per plan and passes exactly. VT-1 through VT-4 cover
present/absent combinations for both facets. VT-9 and VT-10 cover edge cases
(custom confidence bounds, zero-width estimate). VT-11/12 cover shell integration
(fixture slice + malformed config). VT-6/7/8 confirm zero dead_code warnings,
JSON unchanged, and `just gate` green. The full suite (2194 unit/doc tests)
passes with zero failures attributable to SL-132.

**ADR-001 layering is sound.** `src/facet.rs` imports only `estimate` and `value`
(both leaf). `format_value_normal` is pure — no I/O, no clock. `run_show`
resolves units and confidence in the shell tier, passing pure values to
`format_show`. The boundary is clean.

**No regression.** `load_conduct` remains in `slice.rs` for status transitions.
`format_estimate_normal` and `format_estimate_verbose` are preserved for future
verbose display mode. The NF-001 allowlist was updated with `facet.rs`.

**Standing risks.** (a) `format_estimate_confidence` uses `debug_assert` for
bound validity — protected by `resolve_confidence` in the current call path, but
a future caller bypassing the shell would hit release-mode panics (F-2,
tolerated). (b) `format_value_normal` uses `{:.1}` for magnitude formatting;
integer values show one decimal place, diverging slightly from the design's
illustrative prose (F-1, aligned — the settled formatting is correct and tested).

**Tradeoffs consciously accepted.** The `too_many_arguments` clippy expect on
`format_show` reflects an intentional choice to avoid a separate `DisplayCtx`
struct — `EntityFacets` already bundles facet data, and a second bundling struct
for display params would add indirection without reducing coupling.

**Verdict:** Conformant. Ready for reconciliation and close.

## Reconciliation Brief

No spec/governance changes required. Both findings (F-1, F-2) were aligned —
no design, spec, or ADR amendments needed.

### Per-slice (direct edit)

None.

### Governance/spec (REV)

None.
