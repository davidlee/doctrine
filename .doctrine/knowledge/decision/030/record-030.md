# DEC-030: V1 observation queries provide collection search, not analytics

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

The v1 observation query surface supports collection retrieval:

- exact filters over UUID, kind, recorded-time range, and registered typed
  facet fields;
- lexical search over the human-authored summary and detail fields;
- AND semantics between distinct filter fields and OR semantics between
  repeated values of the same field; and
- deterministic newest-first ordering, with UUID as the stable tie-breaker.

The authored observation corpus is authoritative. A query may scan it directly;
any derived index is disposable, optional, and must not change query semantics.

V1 does not provide aggregation, counts or grouping, similarity search,
clustering, causal inference, relevance ranking, or token-efficiency analysis.
Those are reporting and analysis concerns for a follow-up capability.

## Rationale

Structured filtering and plain lexical search are sufficient to retrieve related
raw signals, reduce accidental duplication, and supply later consumers with a
usable collection interface. Adding analytical semantics now would couple the
capture primitive to unproven reporting needs and enlarge the slice beyond its
agreed boundary.

Keeping a corpus scan correct by construction also avoids making an index a
second source of truth. Indexing can be introduced later as a transparent
performance optimisation.

## Consequences

- CLI and MCP collection reads expose the same filter and ordering semantics.
- Typed facet registration determines which facet paths are filterable; the
  query interface does not interpret an untyped metadata bag.
- Lexical matching is deliberately retrieval-oriented rather than a promise of
  ranked full-text search.
- Analytical consumers build over the collection interface without mutating the
  raw observation records or redefining their storage contract.
