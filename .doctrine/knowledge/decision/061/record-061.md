# DEC-061: Inquiry map admits only prerequisite cross-links

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

Each inquiry node has one primary parent for decomposition and may carry zero or
more `needs` references to prerequisite node IDs. This is the only non-tree edge
admitted in SL-233 v1.

Blocked state and reverse “unblocks”/eligibility views are derived from `needs`
and node lifecycle. Lateral “related” links have no coordinator semantics and
are excluded; conditional discoveries add nodes when they become concrete
rather than pre-authoring an activation language.

Conceptual associations that become durable belong in the linked knowledge
record graph, not in an expanding provisional inquiry graph.
