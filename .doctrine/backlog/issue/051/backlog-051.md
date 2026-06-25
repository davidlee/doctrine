# ISS-051: Solo auto-capture misses every slice's final-phase source-delta (off-by-one)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

RFC-004 v0.1 (SL-147) records each phase's source-delta into the conformance
registry. On the **solo / outside-dispatch** path the capture is bound to the
*next* phase's status transition: flipping `PHASE-N+1` to in_progress records
`PHASE-N`'s boundary. The final phase has no `PHASE-N+1` to fire it, so **the
last phase of every solo slice is never auto-captured**.

Found dogfooding SL-147 itself: `slice conformance 147` returned `incomplete —
PHASE-06 has no recorded source-delta row`. PHASE-06 was the last phase; its row
only appeared after a manual `slice record-delta` bootstrap.

Dispatch is unaffected — its funnel records each phase at the `integrate`
land-time beat, not off the next transition.

## Impact

Every solo slice silently drops its final phase's delta. `slice conformance`
fail-closes to `incomplete` (good — no false-clean), but conformance cannot run
at audit without a manual bootstrap, defeating the zero-tax intent.

## Direction

The final-phase boundary needs a trigger that does not depend on a successor
transition — capture on the phase→completed flip itself, or on the
`reconcile`/`close` transition, rather than on the next phase opening.
