# IMP-014: Cross-verb golden harness over listing.rs shared render surface

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

`src/listing.rs` is one of the two highest in-degree modules (11 importers — every
`list` verb leans on it for table / row / JSON rendering). It is pure formatting,
which is why the coupling is acceptable. The gap: the byte-exact golden coverage
(`mem.pattern.testing.black-box-cli-golden`) is **per-verb**. There is no test
pinning the *shared* render surface across all consumers at once.

The exposure: a change to `listing.rs` rendering can pass each verb's individual
golden (because each was regenerated against the new output) while silently
shifting the cross-verb consistency the shared module is supposed to guarantee.
The blast radius is wide and the regression is invisible to the existing suite —
the structural risk a chokepoint carries without a contract test.

## Proposed

Add a cross-verb golden harness that renders the *same* logical row-set through
every `listing.rs` consumer and pins the consistency in one place — so a format
change must be acknowledged at the shared surface, not slip through N
independently-regenerated per-verb goldens.

## Trigger (deferred-until-condition — see IMP-012)

Standing gap, but it *bites* when `src/listing.rs` rendering is edited. Path
trigger (pending IMP-012): `src/listing.rs`. Until then, this prose IS the trigger.

## Relations

Second customer of IMP-012 (architectural triggers). Sibling render concern:
IMP-013 (slice/spec list+show duplication).
