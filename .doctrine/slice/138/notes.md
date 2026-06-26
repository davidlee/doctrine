# Notes SL-138: Relation-transitive walk for inspect — analogous to blockers --transitive

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## PHASE-01 — cordage depth-bounded reachability primitive (done, green)

**Delivered.** `pub struct Reach { depths: BTreeMap<NodeId,usize>, truncated: bool }`
at the cordage crate root; public `Graph::reachable_bounded(overlay, node, direction,
max_depth: Option<usize>) -> Reach`; private `query::reachable_bounded`. `walk_bfs`
rewritten to thread depth (frontier carries `(NodeId, usize)`) and a cap, returning a
private `BfsWalk { order, depths, truncated }`. `query::reachable` re-expressed over
`reachable_bounded(.., None)`; `Graph::reachable` keeps delegating to it; `spine_path`
consumes `.order` and ignores depth.

**Behaviour-preservation (the #1 risk) held byte-for-byte.** The `None` path through
the new `walk_bfs` is iteration-identical to the pre-SL-138 walk (same order vec, same
visited-insert order), so `reachable`/`spine_path` are unchanged. Proof: existing
cordage suite + full workspace bin suites (blockers/inspect/relation) green UNCHANGED
under `just gate`.

**Depth/truncation semantics (design D6, F5).** `depths` = min-hop (BFS first-visit
wins; verified by the diamond VT where a node reachable at 2 and 3 hops records 2).
`truncated` ⟺ a node at the cap had a still-unvisited successor — genuinely deeper than
the cap by BFS ordering, never a false-positive on a within-cap node reached another
way. `start` excluded (only depth-0 entry, removed in `reachable_bounded`).
`Direction::None` / foreign overlay / foreign node → empty `Reach`.

**Verification.** `tests/reachability.rs` +7 VTs (15/15 green). `cargo test -p cordage`
green (incl. the denylist suite — new names `Reach`/`depths`/`truncated`/`max_depth`/
`reachable_bounded` are product-neutral). `just gate` green; clippy zero warnings; fmt
clean.

**For PHASE-02.** `Graph::reachable_bounded` is the engine seam `transitive_from` walks
(per overlay × per direction). `depths` is returned but unconsumed by the display today
(D6 — a future path/tree view consumes it with zero cordage rework). `predecessor_cone`
deliberately untouched (C3).

**Commit:** see `feat(SL-138): PHASE-01` (code + this notes/sheet, SL-138 paths only).
