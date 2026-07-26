# DEC-032: Late usage measurements are separate correlated observations

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

When a trustworthy usage measurement becomes available after its subject
observation was recorded, the recorder emits a separate immutable observation
carrying the `usage` facet. It correlates that measurement to the most precise
known subject: an observation UUID, turn, session, task, phase, or other
registered execution-context identity.

The later record does not patch, enrich, or supersede the earlier observation.
Its declared measurement scope determines what relationship a later consumer
may infer.

## Rationale

Token and elapsed-resource accounting commonly becomes available only after an
agent turn or process completes. Restricting usage to create-time enrichment
would systematically discard those measurements; overlay records would add
patch semantics to an otherwise immutable event corpus.

A separate correlated observation preserves the facts and their timing without
claiming that a session-wide or turn-wide measurement is the marginal cost of a
particular friction event.

## Consequences

- Harness adapters may emit usage observations after work completes.
- Correlation is optional and may be broader than a particular friction
  observation when that is all the source can support.
- Query and reporting consumers join correlated observations explicitly.
- Missing instrumentation produces no usage observation and does not impede
  ordinary capture.
