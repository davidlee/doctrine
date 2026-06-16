# Plan — SL-078

## Rationale

Single phase: both work items are independent, file-disjoint, and small enough
to execute in one pass without internal sequencing constraints. CHR-006 touches
only spec/docs territory (`.doctrine/spec/`, `.doctrine/requirement/`); CHR-008
touches only test code (`src/main.rs`). No dependency between them — parallel
execution is possible but serial within one phase is simpler and avoids any
commit-interleaving complexity for a <2h total change.

## Sequencing

`PHASE-01` covers the full scope. The entrance criteria are design-locked and
plan-consistent (standard gate). Exit criteria pin the four concrete outcomes:
clean spec output, clean requirement output, passing test, clean lint.
