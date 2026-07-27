# QUE-180: Semantic checkpoint admission and acceptance

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

At which design-run transitions must the semantic result be dispositioned as a
new or existing DEC/QUE/ASM, an explicitly retained unresolved result, or a
non-durable exchange? How is user acceptance represented without approval
ceremony after every inquiry-map edit?

Answered by DEC-062: semantic closure is gated at `resolved`; ordinary map
mutation is approval-free. Every resolution links or creates its durable
outcome, links an existing outcome, or explicitly records a non-durable result.
Only promotion into accepted design truth requires user acceptance.
