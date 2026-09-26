# IMP-493: Doctor checks RV status matches last turn

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

A `doctrine doctor` check that an RV finding's stored `status` equals the state
its last `[[finding.turn]]` row implies (RFC-032 D1 turn journal, landed by
SL-268). Deferred from SL-268 by DEC-322; SL-268 discloses out-of-vocabulary
values at the read site instead (DEC-319). Waits on IMP-491 (no spec governs
doctor) and on SL-268's journal landing. Legacy pre-journal findings carry no
turns and must not fire.
