# DEC-067: Regress directly and revalidate forward gates cumulatively

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

A managed design run may regress directly from a later stage to any earlier
stage when the submission supplies a concise reason. Backward movement is not
forced through adjacent stages.

Forward return evaluates every intervening DEC-066 boundary gate in order. Gate
evidence is tied to the relevant input fingerprint—such as the accepted inquiry
basis, aligned section content, materialised design hash, or reviewed revision.
Unchanged evidence remains reusable; changed inputs make only dependent
evidence stale.

A request may return to a later target stage atomically when every intervening
gate remains satisfied. Otherwise it stops at the earliest unsatisfied boundary
and the turn envelope names the refresh obligation. Thus gate-by-gate semantic
clearance never becomes compulsory gate-by-gate human repetition.

This is a fixed small classifier/freshness model, not a generalized dependency
or hierarchical-state-machine framework.
