# Implementation Plan SL-171: next columns, facets, and pagination

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-171 is a read-surface upgrade to `doctrine next` (design.md is canon). Two
phases, split on the natural seam between *what is shown* and *how much is shown*:

- **PHASE-01 — facet columns + `--columns`.** The column model: carry
  estimate/value/tags facets through graph → view → surface, render them, wire
  `--columns`, drop `unblocks`.
- **PHASE-02 — pagination.** `--limit`/`--offset`/`--page` + the truncation
  footer, including the shared-fn lift.

## Sequencing & Rationale

**Why this order.** PHASE-02 depends on PHASE-01 only loosely (pagination clips
whatever rows PHASE-01 renders), but PHASE-01 is the larger behaviour change and
the one the slice exists for. Landing the column model first means PHASE-02's
footer tests assert against the *final* row shape, not an interim one.

**Why two phases, not three.** Facet *carriage* (NodeAttr/NextRow/surface) and
facet *rendering* (NEXT_COLS/cells/`--columns`) are tightly coupled — the
carriage exists solely to feed the columns. Splitting them would yield a PHASE-01
whose only verification is an internal struct populated, which is implementation,
not behaviour. Merged, PHASE-01 is driven by behaviour tests (`next --columns
estimate,value` shows the facet) per the red/green/refactor discipline. Pagination
is genuinely separable — independent flags, independent footer machinery, its own
division-by-zero edge — so it earns its own phase.

**Why the lift sits in PHASE-02.** `format_truncation_notice` is `next`'s footer
consumer; moving it to `listing` only earns its keep once `next` calls it.
Bundling the pure move with its first new caller keeps the behaviour-preservation
proof (retrieve/find/memory-list goldens unchanged) adjacent to the change that
risks it.

**TDD seam per phase.** PHASE-01: a failing test that `next --columns
estimate,value` shows a seeded facet drives carriage + render together; the
`--json` byte-identity golden (VT-5) fences the no-leak invariant red-first.
PHASE-02: a failing `--limit 2` footer test, then the `--limit 0 --offset N`
no-panic test (the F1 guard) drives the guard in green-first.

## Notes

- **Immutable retrieve goldens (PHASE-02 VT-3).** The lift is a pure move; if any
  retrieve/find/memory-list truncation golden shifts, the move was not pure — stop
  and reconcile, do not edit the golden to match.
- **`next` goldens are this slice's to change (PHASE-01).** Existing `next_human`
  goldens assert the old layout (incl. `unblocks`); updating them is expected.
  This is distinct from the behaviour-preservation gate above, which binds shared
  machinery, not `next`'s own surface.
- **External adversarial pass: done (DeepSeek), findings integrated.** 1 BLOCKER
  (lifted-fn self-guard against `page_size==0`), 3 formatting/layering MAJORs, and
  the tags-gate decision (D7 reversed to visible-slice) are folded into design.md §10
  and the criteria above. One MAJOR (keep `unblocks` selectable) was rejected — `next`
  had no `--columns` before this slice, so there is nothing to break; the drop holds.
- Design carries the full decision ledger (D1–D7) and both adversarial passes
  (self F1–F6, external F7–F13 + minors); this plan does not restate them.
