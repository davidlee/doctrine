# QUE-181: Bounded delegation contract for design v1

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

What is the smallest v1 protocol for assigning one bounded inquiry obligation
to another session or agent and accepting an attributed proposal while the
coordinating run retains global transition authority and avoids
harness-specific spawning or write brokering?

Answered by DEC-068: delegates receive attributed bounded assignment envelopes
and return proposals only. The coordinating run remains sole writer and refuses
silent application or rebasing of stale proposals.
