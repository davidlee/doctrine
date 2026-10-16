# REQ-170: Install is idempotent by ownership: user-owned files write-if-absent, framework-owned artefacts converge and are reported, dirs created-if-missing, the gitignore block rewritten

## Statement

Two consecutive project installs with the same binary leave the tree
byte-identical after the second; user-owned files are never overwritten or
removed; every framework-owned write, relink, unlink and prune is reported.

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->
