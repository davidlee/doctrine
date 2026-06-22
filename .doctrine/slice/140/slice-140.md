# Unify cordage traversal: reachable/spine_path/extend_chains

## Context

IMP-020 identifies three diverged traversal implementations in
`crates/cordage/src/query.rs` that share the same BFS + visited-set +
frontier pattern but differ in their termination and mapping logic:

- `reachable` — BFS outward from a start node, collecting a reachable set
- `spine_path` — walks a single-parent chain upward, collecting a path
- the predecessor-cone walk (`cone_on_overlay`) — BFS upward, building a
  `node ↦ {preds}` adjacency map

Each re-asserts the visited-set / frontier loop. `reachable` and `spine_path`
share one genuine shape — breadth-first *discovery order* — and ride a single
primitive. `cone_on_overlay` does not: it records per-node predecessor sets as
map values and must terminate at degraded-SCC entries (record, don't expand),
neither expressible through a discovery-order primitive without a heavier
visitor+predicate abstraction that would obscure the SCC-endpoint logic. It
stays explicit, sharing the (already-factored) neighbour helpers, documented as
the deliberate exception (design D2). The neighbour-lookup helpers
(`neighbours`/`predecessors`/`single_parent`) are already factored out today.

`extend_chains` is referenced in IMP-020's title but is not present in the
current source — it was likely a historical name or a planned abstraction
that was never written. The three implementations above are the active
divergence.

## Scope & Objectives

- Extract a `walk_bfs(start, neighbours) -> Vec<NodeId>` discovery-order
  primitive in `query.rs` — the single locus of the visited/frontier/
  cycle-safety invariant (F12/F47).
- Re-implement `reachable` (strict set, `skip(1)`) and `spine_path`
  (reversed chain) as thin callers of `walk_bfs`.
- Leave `cone_on_overlay` explicit; add a doc comment recording why it is the
  deliberate exception (design D2).
- Public API (`Graph::reachable`, `Graph::spine_path`, `predecessor_cone`) is
  unchanged — this is an internal refactor.
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
| `crates/cordage/src/query.rs` | Add `walk_bfs(…)` discovery-order primitive; rewrite `reachable` + `spine_path` as thin callers; doc-comment `cone_on_overlay` as the deliberate non-rider |
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
