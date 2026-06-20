# CHR-016: Burn down ADR-001 coverage→requirement layering wart

## Context

ADR-001 layering: the `coverage` → `requirement` edge is a wart, currently
carried by 6/10 baselined edges in the layering check.

## Approach

Extract the shared requirement types into a leaf module, mirroring the existing
`kinds` / `dep_seq` pattern, then drop the baseline entries. Behaviour-
preserving — the existing suites stay green unchanged (the behaviour-preservation
gate is the proof).
