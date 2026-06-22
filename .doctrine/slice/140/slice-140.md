# Unify cordage traversal: reachable/spine_path/extend_chains

## Context

IMP-020 identifies three diverged traversal implementations in
`crates/cordage/src/query.rs` that share the same BFS + visited-set +
frontier pattern but differ in their termination and mapping logic:

- `reachable` — BFS outward from a start node, collecting a reachable set
- `spine_path` — walks a single-parent chain upward, collecting a path
- the predecessor-cone walk (`cone_on_overlay`) — BFS upward, building a
  `node ↦ {preds}` adjacency map

Each duplicates the visited-set / frontier / neighbour-lookup loop. The
walk logic should be unified behind a single traversal primitive
parameterised by direction, termination predicate, and fold function.

`extend_chains` is referenced in IMP-020's title but is not present in the
current source — it was likely a historical name or a planned abstraction
that was never written. The three implementations above are the active
divergence.

## Scope & Objectives

- Extract a single `walk` primitive in `query.rs` that subsumes the BFS
  visited-set/frontier loop shared by all three traversals.
- Re-implement `reachable`, `spine_path`, and `cone_on_overlay` as
  thin callers of the unified walk.
- Public API (`Graph::reachable`, `Graph::spine_path`) is unchanged —
  this is an internal refactor.
- Existing cordage tests pass unchanged (behaviour-preservation gate).

## Non-Goals

- No new traversal capabilities (direction, label filtering, etc.) — pure
  unification.
- No changes to channel propagation (`evaluate`) or condensation logic.
- No changes to the doctrine CLI/relation-graph layer.
- No public API additions to the cordage crate.

## Summary

| File | Change |
|------|--------|
| `crates/cordage/src/query.rs` | Add `walk(…)` primitive; refactor `reachable`, `spine_path`, `cone_on_overlay` onto it |
| `crates/cordage/tests/reachability.rs` | Existing suite must stay green unchanged |

## Risks

- **RSK-010 (base drift):** cordage is a shared leaf — changes here affect
  doctrine's priority engine. The behaviour-preservation gate (existing
  tests green unchanged) is the safety net.
- Determinism: the walk must preserve `BTreeSet`/`BTreeMap` iteration order
  (already the case — no Hash-based collections in the current code).

## Verification / Closure intent

- `cargo test -p cordage` green with zero changes to test assertions
- `just check` green workspace-wide
- clippy zero-warnings on cordage
