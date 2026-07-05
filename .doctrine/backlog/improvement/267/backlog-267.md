# IMP-267: close transition gate: refuse done when unmerged dispatch branches exist

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context (revised)

SL-197's implementation was properly merged into `main` via the dispatch
candidate merge (`bcd93294`). The close metadata commits (audit/reconcile/close)
were written to `edge`. After close, `edge` was never synced with `main` — the
primary worktree (where agents operate) lacked the CPT code that `main` carried.
RV-250 F-1 (original) was incorrect; the correction is documented in the
synthesis.

## What's needed

The `close` (or `done`) transition should verify that the primary worktree
(`edge`) is in sync with `main` after dispatch-integrated work. Options:
- Refuse `done` when `edge` has no path to `main`'s implementation commits
- Auto-merge `main` → `edge` as part of the close sequence
- At minimum, warn if `edge` lacks code changes present on `main`

## Notes

This is less urgent than the original F-1 framing suggested — the implementation
landed correctly on `main`. The gap is an edge/main sync hygiene issue, not a
missing-code emergency.
