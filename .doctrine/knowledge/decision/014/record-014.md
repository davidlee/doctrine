# DEC-014: Observation schemas use a thin core and typed facets

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

The reusable observation schema consists of:

1. a small stable core shared by every observation;
2. optional, registered facets with closed typed schemas; and
3. a typed payload owned by the observation kind.

Facets are not arbitrary key-value metadata. Each facet has explicit semantics,
validation, and versioning. Observation kinds select the facets they admit or
require.

## Rationale

A rich universal envelope would encode friction-capture assumptions into unrelated
future observation kinds. A payload-only model would duplicate cross-cutting
structures such as execution context and measurements, frustrating reliable
cross-kind filtering later.

Typed facets preserve reuse and queryability without turning the format into an
unvalidated metadata bag. In particular, SL-231 can define reusable execution and
measurement facets without requiring every observation to describe an agent run.

## Boundary

The stable-core field set, initial facet schemas, and friction payload remain design
questions for SL-231. Reporting, aggregation, and a universal cross-ledger query
surface remain outside the slice even though the schema must not foreclose them.
