# DEC-083: Design apply owns recoverable knowledge checkpoints

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

`doctrine design apply` will accept a checkpoint declaration that can create
one DEC, QUE, or ASM through the existing knowledge engine, relate it to the
slice, and record its canonical reference as the semantic disposition of an
inquiry node.

The combined operation is atomic from the caller's perspective, not a claim of
cross-tier filesystem ACID. The imperative shell writes a narrow recovery
intent keyed by the apply submission id, validates the design-state candidate
and knowledge request before authored mutation, creates the record through the
existing entity engine, captures its returned canonical reference, and then
completes the runtime snapshot. A retry resumes the recorded operation instead
of creating another record.

Authored knowledge is never deleted or rolled back to repair a runtime failure.
If a crash occurs after authored creation but before Doctrine can durably prove
which record it created, the run enters an explicit reconciliation condition;
it must not silently retry creation. This protocol is deliberately specific to
managed-design checkpointing and does not introduce a generic transaction or
workflow framework.

This choice makes incremental durable knowledge capture part of the managed
operation rather than a separate agent ritual. It directly supports SL-233's
primary evaluation signal: timely, unprompted creation of useful DEC/QUE/ASM
records during a prolonged design interview.
