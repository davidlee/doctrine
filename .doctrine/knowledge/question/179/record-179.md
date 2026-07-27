# QUE-179: Minimum useful inquiry-map schema

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

Which node statuses, provenance fields, and sparse cross-links are necessary to
make the active path and frontier inspectable without turning the provisional
inquiry map into a general graph language or imposing a material token tax?

Answered by DEC-059, DEC-060, and DEC-061: use a revisioned runtime TOML
snapshot; node lifecycle is `open|resolved|deferred|pruned`; cursor, blocking,
and user direction are orthogonal or derived; and `needs` is the sole non-tree
edge.
