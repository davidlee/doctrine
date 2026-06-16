# Plan — SL-075

## Rationale

Three phases, ordered by dependency depth. PHASE-01 (HTML/CSS) establishes the
visual substrate that PHASE-02 (JS logic) acts on — legend dimming references the
`data-labels` attrs placed in PHASE-01, depth button rewiring keeps the control
functional after its HTML move, and the transparent SVG background must exist
before visual verification of tooltip and edge contrast. PHASE-03 wraps with
cross-theme visual verification and the D1 fallback gate.

All changes are confined to `web/map/` (5 files). No Rust changes. The backend
API surface is unchanged — all work is presentation and interaction.

## Phase boundaries

**PHASE-01 — HTML + CSS structural refactors.** Pure structural changes with one
coupled JS update: the depth selector HTML moves from sidebar to main pane, and
`wireDepthButtons` must be updated to query the new parent. This is included in
PHASE-01 rather than split across phases because a moved-but-unwired control is
broken and a dead control in place is misleading. All other changes in this phase
are CSS-only or HTML-only with no JS dependency.

**PHASE-02 — JS logic improvements.** All behaviour changes in one phase because
they share `app.js` and the sort comparator in `model.js` — splitting them across
phases would create artificial merge conflicts on the same lines. The phase is
small enough to be digestible: tooltips (3 lines in `dot.js`), sort (one map +
two comparators + 4 call-site changes), legend dimming (one function + one
call-site insertion), DRY refactor (one function signature change), and the
one-line depth button bugfix. D9 is deliberately in PHASE-02, not PHASE-01, to
keep the wiring change and behaviour change in their respective phases — the
depth button is functional after PHASE-01 (just not triggering re-render); D9
fixes the behaviour.

**PHASE-03 — Visual verification.** Manual verification in both themes. D1's
fallback gate is a binary check: if any node fill or edge line is illegible on
the dark background, the light-theme-only faint fill is restored unconditionally.
This is the only phase that may produce a code change (the D1 fallback), so it
runs before audit rather than being collapsed into PHASE-01.

## Sequencing

```
PHASE-01 ──▶ PHASE-02 ──▶ PHASE-03
```

No parallelism — PHASE-02 depends on PHASE-01's DOM structure, and PHASE-03
verifies both.
