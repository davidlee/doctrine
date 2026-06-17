# Review RV-053 — reconciliation of SL-083

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reconciliation audit of SL-083 — pure structural refactor decomposing
web/map/app.js (1,406 lines, single IIFE) into five modules matching the SL-073
design intent. All six phases completed; code merged on main.

**Lines of attack:**

1. **Hard contract conformance** — Does the DI pattern hold? Are modules truly
   parameterised, or does graphPane compute model.neighbourhood internally? Are
   any data-kind or CSS custom-property contracts breached?
2. **Cleanup items F-5 through F-16** — Each item verified against the design
   spec (§4) for completeness and correctness.
3. **Exit criteria gate** — PHASE-06 EX-6 (manual checklist results recorded
   in notes/) is explicitly unmet; the handover.md checklist shows all 16 items
   as `✅/❌` with no pass/fail results. The design §5 says "Manual checklist
   results must be recorded in the slice notes before the slice can be accepted."
4. **Design-vs-implementation signature drift** — Several documented function
   signatures deviate from the as-built API (render.graphPane, render.relationshipTable,
   cm.renderDiagram). Functionally identical but the design artifact is now stale.
5. **Surface-level code quality** — ESLint, just check, module sizes, load order.

## Synthesis

SL-083 is a clean, faithful decomposition of the app.js God Object. The
implementation delivers on every major promise:

- **All 6 phases complete**, code merged on main, `just check` green (one
  pre-existing e2e_memory_sync test failure, unrelated).
- **Module sizes**: app.js 105 (target ~100), render.js 427 (target ~350),
  search.js 170 (target ~150), concept-map.js 198 (target ~400!), svg.js 105
  (target ~60). All within tolerance; concept-map.js undershoots significantly
  because the design estimated more complexity than was actually there.
- **Cleanup items**: F-5 (encodeAttr deleted) ✅, F-6 (NODE_STYLES lookup) ✅,
  F-7 (data-kind selectors) ✅, F-8 (bfsCore) ✅, F-9 (cacheElements) ✅,
  F-14 (wireHandlers factory) ✅, F-15 (declarative API body) ✅, F-16
  (safeStorage) ✅.
- **Hard contracts**: Globals-namespace script loading preserved ✅, XSS
  pipeline intact ✅, CM mutation pipeline intact ✅, SL-073 CSS palette
  untouched ✅, test.html self-contained ✅.

**One blocker**: the manual verification checklist (design §5, PHASE-06 EX-6)
was never executed. The 16-item checklist template in handover.md is present
but has zero pass/fail results. All automated tests pass, but the browser-based
acceptance gate remains closed. This is the only finding that gates closure.

**Design drift (minor)**: Four documented function signatures in design.md §2
diverge from the as-built API (F-2 through F-5). All are functionally benign —
the implementation is correct; the design artifact needs updating. All four
are dispositioned as "design-wrong" and the design.md should be reconciled
before (or as part of) closure.

**Standing risks**:
- render.graphPane computes model.neighbourhood internally (F-2, tolerated) —
  pragmatically identical to pre-computation but deviates from the DI hard
  contract. Low risk; accepted.
- No new test coverage for the module decomposition itself — the pre-existing
  test.html suite is the regression gate and it passes unchanged. Future test
  infrastructure (IMP-088) should add rendering-path coverage.

**Tradeoffs consciously accepted**:
- Terse code style in app.js and concept-map.js (single-line function bodies,
  chained var) trades readability for compactness. Accepted per PHASE-05/06
  precedent.
- render.graphPane reaching into model global trades DI purity for simplicity.
  Accepted with F-2 → tolerated.
