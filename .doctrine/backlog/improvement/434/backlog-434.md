# IMP-434: Plan sites VT test_file by subject, not by the tier that owns the oracle

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`DEC-140` (accepted 2026-08-04) already decides this: a `VT` row's `test_file`
and its `expects` must name a tier that can actually produce the row's subject.
`/plan` does not operationalise it, and the cost is now measured — `SL-251`'s
plan got `test_file` wrong **four times in one slice**, always the same way, and
every time it was caught downstream by a phase planner or a worker rather than
at planning.

- `PHASE-01/VT-1` — moved at plan review.
- `PHASE-03/VT-3` — split at plan review; a sibling `tests.rs` could not reach
  pin 4's samples.
- `PHASE-04/VT-1` — authored at `src/design_run/tests.rs`, but the pin must name
  `crate::knowledge` and `crate::commands`. `design_run` is leaf tier, so that is
  a leaf → command edge and `just check` would have redded.
- `PHASE-06/VT-1` — authored at `src/design_run/payload_contract.rs`, but the
  golden must render over the *real* extern table, which only
  `crate::commands::design::extern_contracts()` can build.

## The mechanism

A criterion is authored where its **subject matter** lives, while
`tests/architecture_layering.rs` sites tests by **import legality**. The two
disagree whenever a leaf-tier subject needs a command-tier oracle. Nothing
surfaces the disagreement at authoring time; it appears only when someone sits
down to write the test, by which point the plan is approved.

## Why this is worth a gate rather than continued vigilance

Three of the four would merely have redded the build — loud, cheap, self-
announcing. `PHASE-06/VT-1` is the one that argues for a fix: sited in the leaf
it would have gone **green** while pinning the *shipped, published* document to
a two-row test fixture, publishing a contract that describes a knowledge region
which does not exist. A defect class that usually fails loudly and occasionally
fails silently is exactly the kind worth mechanising, because vigilance is
calibrated by the usual case.

## Candidate remedies, unranked and not designed

- A `/plan` step that resolves each `VT`'s `expects` to the tier owning its
  oracle *before* writing `test_file`.
- A `doctrine` check reading a plan's `test_file` values against
  `layering.toml` and the row's own keywords.
- Making `DEC-140` a checked rule at `slice plan` time rather than a decision
  agents are expected to recall.

Observed while driving `SL-251` PHASE-04..07.
