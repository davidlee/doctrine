# DEC-045: Resolve observation corrections per control

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

Correction resolution validates and applies controls individually rather than
making every control in a weakly connected component inert when one is invalid.
Controls target primary observations only.

Malformed, dangling, kind-incompatible, cyclic, and losing conflicting controls
are individually diagnostic and inert. Repeated retractions and repeated
supersessions to the same replacement are idempotent. Retraction dominates
supersession for the same target. Distinct successors are ordered by the
canonical control key `(recorded_at, uid)`; the earliest valid edge is effective
and later alternatives are diagnostic. Supersession edges are considered in
that order, and an edge that would introduce a cycle is inert without cancelling
earlier valid edges.

This preserves the current view against appended invalid material: an invalid
control cannot resurrect an observation or cancel a valid correction. Correction
of a mistaken chain operates on the public observations in that chain; the
ledger does not introduce controls over controls.
