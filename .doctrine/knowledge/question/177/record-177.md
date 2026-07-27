# QUE-177: Design-run recovery and freshness model

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

For SL-233, which design-run state must be persisted, which accepted semantic
state should be derived from linked records, and which freshness or idempotency
token should guard transitions and recovery across sessions?

Answered by DEC-057: surviving local runtime state provides exact continuation;
linked durable knowledge provides semantic reconstruction after runtime loss.
The resume projection should normally replace a prose handover, with prose
reserved for state the domain model cannot express.
