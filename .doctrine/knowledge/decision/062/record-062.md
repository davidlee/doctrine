# DEC-062: Resolved inquiries require explicit semantic disposition

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

Ordinary inquiry-map maintenance—adding, moving, pinning, deferring, or pruning
nodes—does not require human approval or knowledge-record creation. Material
structural changes remain visible through the run's change projection.

A node may transition to `resolved` only with an explicit semantic disposition:
create or link the durable record carrying its outcome, link an already-recorded
outcome, or mark the result intentionally non-durable with a concise reason.
The coordinator refuses a resolution lacking one of these alternatives.

Promoting a proposed choice or premise into accepted design truth requires
evidence of user acceptance at the semantic checkpoint. Inquiry-map edits do not
inherit that authority merely because they were accepted as valid mutations.
