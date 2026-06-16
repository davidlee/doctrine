# Implementation Plan SL-077: Render requirement prose in spec show

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

Three phases, sequenced: extract the `read_spec` reader first (the foundation),
then build the requirement prose reader and wire it into `spec show`'s render
and JSON paths, and finally add the `prose` column to `spec req list`.

## Sequencing & Rationale

**PHASE-01 — Extract `read_spec` reader.** The slice scope bundles IMP-037
(spec reader consolidation) as the natural foundation for IMP-058 (requirement
prose rendering). Extracting `read_spec` eliminates two inline `from_str::<Spec>`
parse sites (`run_show` and `relation_edges`) and gives the module a single read
seam, mirroring `read_slice`'s `(parsed, raw_toml, prose_body)` shape. The
`build_registry` site keeps its inline parse — it carries non-trivial
`second_parent` error classification that doesn't belong in a general reader.
This phase is strictly a behaviour-preserving refactor: existing test suites
must pass unchanged, zero test-body diffs.

**PHASE-02 — Requirement prose reader + spec show render.** Adds
`requirement::load_with_prose(root, fk)` — a companion to the existing
`requirement::load` that reads both TOML and `.md` tiers. Scaffold detection:
if the `.md` body's sections (`## Statement` / `## Rationale`) contain only
`<!-- ... -->` comment placeholders and no authored text, it's classified as
`None`. This feeds into `render()` (a new line below each requirement's
structured facets in Table mode) and `show_json()` (a `body` key per member
requirement, absent when scaffold). The existing `render`/`show_json` tests
are extended additively — new assertions for the prose line, no rewrites.

**PHASE-03 — Prose column in `spec req list`.** Extends `ReqListRow` with a
`prose` field (`✓`/`—`), added to `REQ_COLUMNS` and `REQ_DEFAULT` as the 5th
column. `ReqJsonRow` gains `prose: bool` (absent on dangling rows, which have
no readable requirement to check). Uses `load_with_prose` from PHASE-02 —
each member in the roster now reads its `.md` body. Existing req_list tests
are extended additively.

Phases are strictly sequential: PHASE-02 depends on the shared read seam from
PHASE-01; PHASE-03 depends on `load_with_prose` from PHASE-02. No
parallelisation possible — each phase touches the same two files with additive
changes that compose cleanly in sequence.

## Non-Goals (re-stated from scope)

- No standalone `doctrine requirement show` verb
- No requirement authoring changes — read-only
- No validation or correctness rules

## Notes

- `build_registry` keeps its inline parse by design — the `second_parent`
  classification logic is non-trivial and not a fit for the general reader.
- Scaffold detection is structural: check that `## Statement` and `## Rationale`
  headings contain only HTML comments, no authored text. This avoids matching
  against the exact template bytes (fragile) while still detecting unfilled
  requirements.
- The `prose` column on `spec req list` is the first derived/observed column on
  that roster — all current columns are authored. This is a read-only derivation
  (no write, no cache) and doesn't violate INV-3.
