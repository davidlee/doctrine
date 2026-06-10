# Record-time stderr nudge for hidden thread memories

## Context

`thread_expiry` (`src/retrieve.rs`, SL-008 design D6) excludes any `thread` that
is not `verified` ∧ `reviewed`≤14d from `find`/`retrieve`. `record` always
scaffolds `unverified`, so a freshly-recorded thread is invisible to scope
ranking — `list`/`show` only — until `verify` (which refuses a dirty tree). An
agent hit this and read it as a retrieval bug (SL-032 handover "blind spot").
The behaviour is correct canon (D6); the gap is discoverability. The
`/record-memory` skill already warns, but an agent recording via raw CLI without
the skill gets no signal. IMP-011 parks the runtime nudge; this slice closes it.

## Scope & Objectives

Emit a single non-blocking **stderr** line when `doctrine memory record
--type thread` succeeds, telling the user the thread will not surface in
`find`/`retrieve` until `verify`d on a clean tree. Direct prior art:
`run_record` already warns to stderr for the linked-worktree case
(`src/memory.rs:755`, SL-032 PHASE-04) — same shape, gated instead on
`memory_type == Thread`.

- Affected surface: `src/memory.rs` `run_record` (the shell), plus its tests.
- Fires for every `--type thread` record (always unverified → always hidden),
  including `--global` (a global thread is gated identically).

## Non-Goals

- **No change to `thread_expiry` or any read path** — D6 stands, the
  behaviour-preservation gate stays intact (existing memory suites unchanged).
- No new flag, no stdout change (machine-readable success line is untouched; the
  nudge is stderr only, like the worktree warning).
- No suppression/quiet flag in v1.

## Summary

(to be completed at close)

## Follow-Ups

(none anticipated)
