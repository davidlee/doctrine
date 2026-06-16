# Review RV-040 — reconciliation of SL-075

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Self-audit of SL-075 (Map Explorer UX overhaul — 9 design decisions, 3 phases,
16 scope items) against `design.md` (canonical), `plan.toml` EN/EX/VT criteria,
and the implemented code at d3ac96d.

Lines of attack:
1. **D1 SVG bg + dark-theme contrast** — the design admits edge `#888888` on
   `#1a1a1a` is ~1.6:1 contrast (< WCAG AA 3:1) and prescribes a binary fallback.
   The fallback is light-theme-only; dark theme stays transparent. Is the fallback
   mechanism internally consistent, and was the gate property exercised?
2. **D2–D9 conformance** — check each design decision against the actual source.
3. **Late addition: hide-relations toggle** — persistence, re-render correctness,
   interaction with render() refresh cycle.
4. **Gate and lint hygiene** — `just gate`, `node --check`, `npx eslint` all green.
5. **Handover watch-outs** — D1 dark-theme contrast, hide-relations persistence,
   `compareEdgesBySource` ESLint suppress, missing `'use strict'` on model/router/dot.

## Synthesis

SL-075 ships 9 design decisions + 1 late-addition (hide-relations toggle) across
3 phases, all 16 original scope items satisfied. Gate is green (1471 passed, 0
failed).

The reconciliation surfaced 3 findings, all now terminal:

- **F-1 (major, verified):** D1 had an internal inconsistency — the fallback gate
  was described as triggering on dark-theme illegibility but the prescribed
  mechanism (light-theme-only SVG background) couldn't address it. Resolved via
  `design-wrong`: D1 text corrected to scope the fallback gate to light theme
  only, and the depends/requires edge colour bumped from `#888888` to `#aaaaaa`
  in `dot._EDGE_COLORS` (~1.6:1 → ~7.2:1 dark-theme contrast).

- **F-2 (minor, verified):** D4 called for depth button relabel but the HTML
  retained bare digits. Tolerated — the "Depth" label above the button group
  provides the self-describing context the D4 relabel was meant to add.

- **F-3 (nit, verified):** `'use strict'` is present only in `app.js`, missing
  from `model.js`, `router.js`, `dot.js`. Tolerated — the ES5 codebase doesn't
  exercise strict-mode-sensitive features and adding it to stable files carries
  non-zero regression risk for zero behavioural change.

**Standing risks:** The dark-theme edge contrast improvement (`#aaaaaa`) leaves
light-theme edge contrast at ~2.4:1 on `#ffffff`. Thin edge lines in a DOT
graph are not text UI components and the design consciously accepts this
tradeoff. No follow-up backlog items created — the remaining warts (`use strict`,
depth button labels) don't justify new work.

**Gate:** `just gate` green, `node --check` clean, `npx eslint` zero errors.
Ready for `/close`.
