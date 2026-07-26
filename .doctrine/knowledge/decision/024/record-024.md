# DEC-024: Observation retries use caller-stable UUIDs, never content deduplication

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

Observation creation uses create-only UUID idempotency:

- the recorder generates a fresh UUIDv7 unless the caller supplies a UUID;
- a caller or transport retry supplies the same stable UUID;
- storage never overwrites an existing observation;
- replay of the same UUID with the same caller intent returns the existing
  observation successfully; and
- replay of the same UUID with different caller intent returns an identity
  conflict.

Caller intent comprises the observation kind, typed payload, and explicit facets.
Automatically enriched values are fixed by the first successful write and are not
recomputed when an idempotent replay returns that record.

Different UUIDs are never deduplicated, regardless of content similarity.

## Rationale

Identical summaries may describe distinct occurrences; content deduplication would
erase the frequency signal protected by DEC-013. UUID replay distinguishes a
transport retry from a new occurrence without a shared idempotency registry or a
pre-capture duplicate search.

Create-only storage also prevents an idempotency mechanism from becoming a hidden
mutation path.

## Consequences

- CLI and MCP accept an optional caller-supplied UUID.
- MCP transports should reuse a stable UUID across retries where they can.
- A normal invocation without a supplied UUID always records a new occurrence.
- Identity conflict is a hard error because silently accepting different intent
  under one UUID would corrupt the ledger.
