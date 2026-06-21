# Split worktree.rs into a submodule folder

## Context

`worktree.rs` is 3049 lines (~2317 production) — the second-largest production
body in the repo. The 2026-06-19 architecture audit judged it a clear
SPLIT-CANDIDATE: a single file doing the work of a folder, bundling four distinct
concerns under one roof.

1. **Allowlist** — parsing/matching/violation reporting.
2. **Lifecycle** — six near-parallel state machines, each an
   `enum Refusal` / `classify_*` (pure) / `run_*` (shell) triplet:
   fork / provision / import / land / gc / coordinate.
3. **Marker / env worker-confinement** state.
4. **Subagent stamping + worker verify.**

The project demonstrably has a submodule-folder convention (`catalog/`,
`map_server/`, `priority/`, `estimate/`, each a `mod.rs` + topic files). This is
exactly the case it exists for — the file is folder-shaped already. The
pure/shell triplet structure makes the seams obvious and the split low-risk.

## Scope & Objectives

- Convert `worktree.rs` into a `worktree/` folder with `mod.rs` plus topic files.
  Final partition (decided in `/design`, D1): **per-machine** —
  `mod.rs` (command + dispatch + re-exports), `shared.rs`, `allowlist.rs`,
  `marker.rs`, and one file per lifecycle machine (`provision`/`import`/`land`/
  `coordinate`/`gc`/`fork`), `subagent.rs` (stamping + worker verify, D7), plus a
  `#[cfg(test)] test_helpers.rs`.
- Preserve the public surface (`worktree::*` paths callers use, e.g.
  `worktree::run_phases`'s neighbours) via `mod.rs` re-exports so no caller
  changes.
- Keep the pure `classify_*` / impure `run_*` split intact across the move (it is
  the existing seam, not a new one).

Closure intent: `worktree.rs` replaced by a `worktree/` folder; each concern in
its own file; public paths unchanged; existing suites green unchanged
(behaviour-preservation gate).

## Non-Goals

- The `worktree.rs:1742 → slice::run_phases` upward edge (a coupling-direction
  concern; out of scope here — note it, do not fix it in a cohesion split).
- Changing worktree behaviour, the lifecycle state machines, or the allowlist
  semantics.
- Splitting any other oversized module (main.rs → SL-115; memory.rs is a separate
  candidate not yet sliced).

## Summary

Lift `worktree.rs` into a `worktree/` submodule folder along its four existing
concern seams (allowlist / lifecycle / marker / subagent), following the
established convention. Pure mechanical cohesion split, behaviour-preserving.

## Follow-Ups

- `memory.rs` (~2884 production, multi-concern) is the next folder-split candidate
  from the audit — slice it if/when it earns priority.
- The `worktree → slice` upward edge may want its own coupling slice.
