# IMP-193: Retire doctrine validate once doctor proves fast enough

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Source

SL-168 design decision D9. `doctrine doctor` (SL-168) is built as a **strict
superset** of `doctrine validate` (it runs both halves — id-integrity +
relation-graph integrity — as checks #1/#2, design D8). So `validate` becomes
redundant at the UX surface.

SL-168 deliberately **keeps** `validate` in v1 (D9): it is already a thin,
non-duplicating composition over the shared check fns, and removal is a breaking
change to any CI/script calling `doctrine validate`. The slice scope lists
removing per-surface commands as out-of-scope.

## Trigger

Retire (or quiet-alias) `validate` **once `doctor`'s runtime is measured and
proven fast enough** to be the default integrity gate. Doctor currently re-walks
the corpus once per check (SL-168 R6 — no shared snapshot), so prove the cost is
acceptable first.

## Options when actioned

- **Quiet-alias** `validate` → `doctor`'s id+relation subset (back-compat,
  single entry point).
- **Strip** `validate` with a deprecation cycle.
- Pair with the SL-168 R6 shared-corpus-snapshot refactor if perf is the blocker.
