# CHR-068: Stale tangle baseline in architecture_layering doc comment

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`tests/architecture_layering.rs:8` and `:22` state `Tangle baseline: leaf=0,
engine=0, command=120` and `Command tangle (120 cyclic edges)`. The authoritative
baseline is `.doctrine/adr/001/layering.toml:190` — `command = 76` (SL-204
PHASE-04 took it 99 → 76, per the comment at `:180`).

The gate passes against 76; only the header comment is stale. Found by SL-238's
type prototype while checking the design's claim that the baseline holds at 76 —
the design is right and the test file's comment is wrong. It is a footgun for
anyone grepping the test file for the number rather than reading `layering.toml`.
