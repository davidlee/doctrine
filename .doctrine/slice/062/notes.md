# SL-062 — implementation notes

Durable cross-phase findings from the dispatch run. Disposable phase sheets live
under `.doctrine/state/`; this file is the authored durable record.

## Dispatch mechanism observation (PHASE-01)

The `/dispatch` claude arm spawned a `dispatch-worker` via the `Agent` tool with
`isolation: worktree`. Observed: the worker's single commit `S` **integrated
directly onto `main`** on completion — no registered worktree remained
(`.git/worktrees/dispatch/HEAD` absent), and `main@{0}` in the reflog is the
worker's commit. The orchestrator did NOT run a separate `import`/one-commit step;
the delta was already on the coordination branch.

Outcome was nonetheless sound — the funnel's *goals* held even though its
sole-writer *mechanism* was bypassed:
- delta = exactly the 3 declared source files (`src/lifecycle.rs`, `src/slice.rs`,
  `src/main.rs`); **no foreign untracked sweep** (review/020, slice/063, memory
  items all still untracked, untouched);
- **R-5 clean** (no `.doctrine/`/`.claude/` touch);
- linear on HEAD (B=32dae47 → 3 foreign commits → S=7e4e071), no divergence;
- combined tree verified GREEN by the orchestrator (`just gate`, clippy `-D
  warnings` clean) AFTER landing — not trusting the worker's self-report.

Consequence for PHASE-02/03: the R-5 belt + verify run **post-landing**, not
pre-commit. Mitigation in the worker brief: stage ONLY declared files, never `git
add -A` (foreign untracked files sit in the shared tree); orchestrator verifies
each delta post-landing and would have to revert on a violation. Single
observation — confirm on PHASE-02 before recording as durable doctrine memory.

## PHASE-01 — re-home pure FSM into src/lifecycle.rs (DONE, verified green)

- New pure leaf `src/lifecycle.rs` (beside `conduct.rs`, ADR-009 axis-A/axis-B
  pairing): `enum Transition`, `fn classify`, `fn is_transition_terminal`,
  `fn crosses_closure_seam` + edge table. Pure `&str`-edge data, imports no kind
  module (ADR-001 no-cycle holds).
- STAYED in `slice.rs`: `transition_label` (P4), `is_terminal_status` (P3, distinct
  from `is_transition_terminal`), `SLICE_STATUSES`/`SliceStatus`/drift canary,
  `is_divergent`/`is_hidden`/`is_drifted`, `run_status`/`set_slice_status` (retarget
  imports to `lifecycle::*`).
- OQ-1 resolved: classify edge-case tests MOVED to `lifecycle.rs` (smaller, cohesive
  diff); the distinct-predicate canary stays in `slice.rs` importing
  `lifecycle::is_transition_terminal`.
- Behaviour-preservation gate held: slice FSM suite assertion text unchanged, only
  import paths shifted (F-E). Commit `7e4e071`.
