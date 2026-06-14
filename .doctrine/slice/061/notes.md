# Notes SL-061: Rewire /code-review and /inquisition onto the RV review ledger

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## 2026-06-14 - external design inquisition and plan lock

- Ran the current prose `/inquisition` flow against SL-061 design + scope under
  ADR-007; recorded the findings in gitignored `inquisition.md`.
- Integrated three findings before lock: preserve `/audit` anti-escape +
  phase-sheet harvest mechanics in the shared extraction; make the ledger/prose
  trigger operational so `/code-review` cannot route around RV for durable
  reviews; remove stale scope language that still treated the inquisition facet
  as open.
- Advanced lifecycle `design -> plan -> ready`, authored `plan.toml` / `plan.md`,
  and materialised PHASE-01..PHASE-04 runtime sheets.
- Verification run: `doctrine slice show SL-061`, `doctrine slice phases 61`,
  and `git diff --check` for the touched SL-061 authored docs. No `just gate`
  run yet; no production code has been modified in this unit.
- Uncommitted work remains in SL-061 authored docs/plan/status/notes. Unrelated
  dirty workspace entries existed during the pass and were left untouched.
