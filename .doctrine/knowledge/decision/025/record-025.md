# DEC-025: Observation capture does not mutate Git

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Decision

`doctrine observation record` owns one atomic capture transaction:

1. validate explicit input;
2. allocate or accept the UUID and timestamp;
3. apply best-effort typed enrichment;
4. serialize and create the immutable file without overwrite;
5. resolve UUID replay or report identity conflict; and
6. return a receipt containing the UUID, path, warnings, and
   created-versus-replayed outcome.

The command does not stage, commit, push, or otherwise mutate Git. It does not
update a shared authored manifest or index, and Git metadata being unavailable
does not invalidate an otherwise valid capture.

## Rationale

Git publication is a distinct workflow transaction. Coupling it to capture would
add hook latency and failure, create one-observation commit spam, contend over the
shared index, and make it ambiguous whether a failed command had already preserved
the raw signal.

The returned path and UUID give the surrounding workflow everything needed to
include observations in a later path-scoped commit or task-end batch.

## Consequences

- Capture success means the observation file exists locally, not that it is
  published in repository history.
- RFC-011 dogfooding guidance may require returned paths to be included in the next
  coherent commit or a task-harvest batch.
- If operational evidence shows observations are routinely stranded, a separate
  flush or publish capability may be designed later; it is not implicit behaviour
  of `record`.
- Derived indexing may run separately but cannot become a precondition for capture.
