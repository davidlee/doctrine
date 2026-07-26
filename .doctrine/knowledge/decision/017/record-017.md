# DEC-017: Observation processing state is consumer-owned

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

An observation carries no consumed, analysed, processed, triaged, or aggregated
state. Processing state is owned by each consumer and stored outside the immutable
observation.

Consumers may maintain receipts, checkpoints, or analysis-run manifests appropriate
to their guarantees. Durable outputs may cite source observation UUIDs directly or
through a durable analysis-run manifest.

## Rationale

Consumption is not an intrinsic property of an observation. It is relative to a
consumer, purpose, processor version, and run; it may happen repeatedly and produce
several independent outputs. Storing one consumption facet on the observation would
collapse those histories, require mutation, and create contention on otherwise
merge-independent records.

Separating source signals from consumer state also permits operational bookkeeping
to remain derived or disposable while significant analytical provenance is made
durable only when needed.

## Consequences

- SL-231 does not define processing lifecycle fields or consumption facets.
- The observation reader provides stable UUIDs and deterministic enumeration from
  which consumers can maintain their own progress.
- Reporting and aggregation follow-ups choose their required delivery semantics
  and state model rather than inheriting a premature boolean.
- Promotion or authored analytical outputs do not mutate their source observations.
