# QUE-184: Managed design workflow state and obligation model

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

Should the managed design run preserve the current skill's stages as literal
stored states, normalize them into a smaller semantic FSM with point-in-time
obligations, or interpret an external step graph? Which states are durable run
position, which behaviours are obligations, and which loop transitions must the
CLI validate?
