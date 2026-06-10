# IMP-013: Lift shared list+show row/table/JSON shape across slice and spec verbs

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

`src/slice.rs` and `src/spec.rs` carry parallel list/show implementations: both
read metas (via `meta.rs`), assemble per-kind rows, render a table (via
`listing.rs`), and emit JSON. The shape is duplicated; the per-kind row
*content* is not.

**Do not lift this now.** The parallelism is real but the bodies diverge where it
matters: `slice` decorates rows with drift / divergence / phase-rollup state
(SL-009) that `spec` has no analog for; `spec` dispatches a product/tech subtype
that `slice` handles differently. A shared row-builder today would need a
configuration surface more complex than the duplication it removes — premature
DRY across two kinds that may keep diverging.

## Trigger (deferred-until-condition — see IMP-012)

Fires when a phase **next reshapes the list or show rendering** in either
`src/slice.rs` or `src/spec.rs`. At that edit, the cost of touching one side
makes lifting the shared shape (row assembly + table + JSON, parameterized by a
per-kind column/decoration spec) cheaper than re-diverging. `listing.rs` already
owns the table/row primitives — the lift is the row-assembly + JSON layer above
it, not the table itself.

Path trigger (pending IMP-012's structural field): `src/slice.rs`, `src/spec.rs`
— list/show fns. Until IMP-012 ships, this prose IS the trigger.

## Relations

First customer of IMP-012 (architectural triggers). Sibling render concern:
IMP-014 (listing.rs cross-verb golden gap).
