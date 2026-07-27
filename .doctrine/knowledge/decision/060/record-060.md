# DEC-060: Inquiry lifecycle separates status cursor blocking and direction

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

Inquiry nodes carry only the lifecycle status vocabulary `open`, `resolved`,
`deferred`, or `pruned`. The run's current cursor is a separate
`active_node`; an open node is blocked as a derived condition when one of its
declared dependencies remains unresolved.

User pinning and breadth/depth direction are traversal policy, not lifecycle
states. Keeping these axes orthogonal prevents contradictory combinations such
as an “active” node that is not the cursor or a stored “blocked” node whose last
dependency has already resolved.

Read projections may render active, blocked, and user-directed badges, but they
derive them from the snapshot rather than persisting duplicate status.
