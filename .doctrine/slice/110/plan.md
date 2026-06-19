# Implementation Plan SL-110: Web map UX polish: actionability/concept-map view interactions

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Five UX items + one scoped backend op on the web-map view-interaction surface.
The design (locked after two Codex passes) reduces them to three load-bearing,
unit-testable cores — `relabel_edge_in_dsl` (Rust), `focusTransition` and
`hoverDetailHtml` (TS pure fns) — with the rest being wiring and CSS. The phasing
follows those cores: each phase owns one testable seam plus its wiring, so red/
green/refactor has a real target and the manual (VH) surface stays small.

## Sequencing & Rationale

- **PHASE-01 first — relabel_edge backend.** Fully isolated (Rust only:
  `concept_map.rs`, `routes.rs`), file-disjoint from every frontend phase, and a
  hard dependency of PHASE-04. Pure-fn TDD with the sharpest correctness risk in
  the slice — the key-vs-label duplicate guard (Codex G1, the blocker). Landing
  it first de-risks the slice and unblocks item 4.

- **PHASE-02 — view-mode funnel + toggle (items 5 + 1) together.** Item 1's
  highlight is defined *relative to* item 5's derive ("called once early, right
  after the derive sets `state.viewMode`"), and both edits live in `renderView`.
  Splitting them would mean touching `renderView` twice and sequencing a
  placement dependency across phases; merging keeps the foundational frontend
  change cohesive. This is the riskiest frontend reasoning (the `focusChanged`
  gate that keeps the toggle alive — D1), so it goes early while context is warm.

- **PHASE-03 — hover tooltip (item 2).** Independent seam (`render.ts`/
  `priority.ts`); depends only on the locked design. Sequenced after 02 by
  convention, not necessity.

- **PHASE-04 — CM cell-select edit (item 4 frontend).** Entrance-gated on
  PHASE-01 (needs the `relabel_edge` server seam). Touches `app.ts` (shared with
  02) so it runs after the funnel work to avoid churning the same file mid-flight.

- **PHASE-05 — checkbox alignment (item 3).** Trivial CSS, independent, last.
  Could land any time; placed last so a visual-only change never blocks the
  substantive phases.

## Notes

- **Parallelism / file contention.** PHASE-01 (Rust) and PHASE-05 (`sidebar.css`)
  are file-disjoint from everything and from each other — dispatchable in
  parallel if desired. PHASE-02 → PHASE-04 share `app.ts`, and PHASE-02 →
  PHASE-03 share `priority.css` (item 1 renames `.view-btn`, item 2 adds
  `.priority-tooltip`), so that chain runs serially. Default execution is serial;
  this is polish, not a throughput problem.

- **Behaviour-preservation gate.** Existing `web/map` vitest suites and the
  `routes.rs`/`concept_map.rs` mutation tests must stay green unchanged across
  every phase, except `hoverPane`'s test in PHASE-03, which is intentionally
  updated to assert escaping rather than the old (partly-unescaped) markup — a
  fix, recorded as such, not a silent behaviour change.

- **Out of scope, captured.** The pre-existing `add_edge_to_dsl` label-vs-key
  duplicate gap (sibling of PHASE-01's guard) is ISS-025, deliberately not fixed
  here. Deep-linkable view mode (`?view=…`, behind D1) and the larger frontend
  cleanups (IMP-085/086/087/089) remain their own items.
