# Implementation Plan SL-094: Semantic graph: zoom, pan, and crop-on-bounds for DOT/Graphviz SVG view

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Three phases, layered from pure → impure → integration. Each phase produces a
separately verifiable increment:

- **PHASE-01** — pure math: the viewport type and helpers, tested in isolation.
  No DOM, no browser, just arithmetic.
- **PHASE-02** — the feature stands alone: CSS changes + `graphPane()` wiring.
  Zoom/pan works interactively on the DOT graph, with all design guards in place.
- **PHASE-03** — persistence + navigation: app state makes the viewport survive
  re-renders, and focus change triggers centring/clamp rules.

## Sequencing & Rationale

**PHASE-01 first** because every other phase depends on the pure helpers.
`fitViewport` and `applyFocusChange` encode the design's viewport rules as
testable functions — catching edge cases here catches them cheaply. Placing
`GraphViewport` in a dedicated `viewport.ts` keeps the type and its logic
co-located; `svg.ts` remains for SVG DOM manipulation only.

**PHASE-02 second** because it delivers the user-facing behaviour — zoom/pan on
the DOT graph — as a standalone unit. The CSS changes (.graph-area restyle,
.transform-layer) and event handlers (wheel, drag) are tightly coupled to
`graphPane()`, so they ship together. Cross-mode guards and listener lifecycle
are critical here; verifying them in isolation before adding state persistence
keeps debugging bounded.

**PHASE-03 last** because it's thin wiring. The viewport state lifts into
`app.ts` with three fields and a callback. The focus-change logic is already
implemented and tested in PHASE-01's `applyFocusChange`; this phase just calls it
at the right time. The `lastRenderedFocusId` field is the only new concept — it
exists purely to compute `focusChanged` before calling `graphPane()`.

**Verification posture:** PHASE-01 uses VT (unit tests) for all criteria.
PHASE-02 and PHASE-03 use VA (agent checks) because zoom/pan behaviour is
interactive — automated DOM testing of wheel events and drag sequences is
brittle and low-signal. VA-1 in PHASE-02 gates the core feature; VA-1 through
VA-4 in PHASE-03 gate the persistence and navigation rules.

## Notes

- The design's follow-ups (pinch-to-zoom, resize/reflow, reset-to-fit,
  click-vs-drag disambiguation, zoom-to-selected) are out of scope for all
  phases.
- `plan.toml` is the single source of criteria truth; this file explains why.
