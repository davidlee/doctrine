# DEC-063: Design-run mutation uses atomic sparse JSON declarations

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

The canonical SL-233 v1 mutation input is a JSON batch of domain-specific sparse
declarations. Redeclaring an existing subject changes only supplied fields;
omitted subjects and fields persist. An empty collection explicitly clears that
collection, and `null` explicitly clears a nullable field.

The batch is unordered and atomic. Duplicate declarations for one subject,
unknown fields, illegal lifecycle transitions, parent/dependency cycles, or a
resolved node without the DEC-062 disposition refuse the whole candidate state.
There is no deletion operation: `pruned` preserves the visible history.

Every submission carries the expected run revision for stale-writer refusal and
a submission identity for idempotent retry. A reused identity with a different
payload is refused. The pure core applies declarations to a current snapshot and
returns either a validated candidate plus material delta or structured
refusals.

A compact DSL may later parse into these same declaration types. It is not a
second mutation protocol and is outside v1.
