# QUE-184: Managed design workflow state and obligation model

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

Should the managed design run preserve the current skill's stages as literal
stored states, normalize them into a smaller semantic FSM with point-in-time
obligations, or interpret an external step graph? Which states are durable run
position, which behaviours are obligations, and which loop transitions must the
CLI validate?

Answered by DEC-065 through DEC-067: use five coarse semantic stages, keep
inquiry, cursor, traversal, section, review, and obligation state orthogonal,
enforce only load-bearing boundaries, and support direct regression with
cumulative freshness-aware forward validation.
