# SL-043 implementation notes

Durable cross-phase findings. Runtime task detail lives in the gitignored phase
sheets; this file survives to feed the audit at close-out.

## PHASE-01 — resolve.rs (commit 3687c17) — COMPLETE

Iterative Tarjan + iterative `level_of` + per-component eviction localization +
RSK-001 coverage VT. Behaviour-preserving: the 75 pre-existing tests stay green
unchanged; only `tests/ordering.rs` (+1) and `tests/scale_cliffs.rs` changed.

- **Executed in an isolated worktree subagent**; integrated to main by
  **cherry-pick** (the Agent worktree forked from a divergent base — merge-base
  was an old `chore` commit, not the plan commit — so a full merge was wrong;
  cordage was byte-identical at base & main, so the single-commit cherry-pick
  applied with exact context). Watch-out for the remaining phases.
- **Iterative Tarjan**: frame `(node, cursor)` + a `returned` child marker mirrors
  the recursion — first-visit init guarded by `returned.is_none()`, child-return
  folds `lowlink`, the parked cursor prevents re-walking successors. SCC emission
  ORDER may differ from the recursive form; that is sound by A-4 consumer
  order-insensitivity (provenance re-sorted, set-membership reads, edge-keyed min).
- **level_of**: push-children-then-revisit with an `expanded` flag; the
  `!cache.contains_key(parent)` guard makes shared ancestors (diamonds) resolve
  once. Relies on U being acyclic (pass 3 broke every U cycle).
- **Localization**: each cyclic component drives its own induced sub-edge-set to
  fixpoint (re-Tarjan only the shrinking sub-component). Evicted SET identical to
  the global loop by vertex-disjointness; provenance sorted ⇒ identical output.
- **G2 (layer-k invariant)** is enforced as a loud `debug_assert!(false, …)` STOP
  seam on the no-victim arm of `evict_layer_component`. It never fired across the
  suite. VT-1 is satisfied by that seam PLUS the unchanged `compose_order` goldens
  (golden_net.rs, ordering.rs) that depend on the invariant — the plan VT-1
  explicitly sanctioned this "OR the goldens" form. NOTE for audit: there is no
  dedicated *positive* fixture that constructs a layer-k cycle and asserts the
  invariant directly — acceptable per plan, but a candidate hardening if revisited.
- **Signals**: `deep_chain_overflows_*` → `deep_chain_builds_inside_target_scale`
  (default gate; 80k builds ~2s, asserts 80k nodes ordered — exercises BOTH Tarjan
  and level_of). New `many_small_cycles_*` gates (set-identity + linear in N).
  `eviction_fixpoint`/`dense_evict` stay `#[ignore]` = EXC-2 deferred residual.
- Gate: `cargo test -p cordage` green, `cargo clippy -p cordage` clean, `just
  check` green on main.

## PHASE-02 — query.rs evaluate (commit d97b311) — COMPLETE

Retire RSK-004: `evaluate`'s per-node `reachable` BFS replaced by ONE
direction-resolved condensation fold per call. Output byte-identical; the
behaviour-preservation suite stays green unchanged (channels, reachability,
golden_net, ordering incl. PHASE-01 RSK-001). REQ-078 + REQ-092 hardened.

- **Dispatched via the `/dispatch` funnel** (first real funnel run). Worker in an
  isolated rung-3 fork (`sl-043-p02`), forked from EXPLICIT coordination HEAD `B`
  (not session HEAD) so `S.parent == B` — imported as the net diff `B..S`, NOT
  cherry-pick. This is the corrected form of the PHASE-01 divergent-base gotcha;
  the worktree skill now mandates `worker` REQUIRES an explicit base (committed
  `1e208a2`, memory `mem.pattern.dispatch.fork-rung3-base-not-session-head`).
- **G1 is the dominant seam.** Partition AND condensation edges AND reverse-topo
  all derive from `neighbours(.., direction)` (Along=out.dst, Against=incoming.src,
  None=∅) — NEVER the forward build adjacency. `None` ⇒ every node a singleton (do
  NOT group stored `degraded_sccs`); Evict ⇒ singletons; Reject Along/Against ⇒
  stored `degraded_sccs[overlay]` grouped, rest singletons (total partition). C1.
- **Per-combinator fold** up the reverse-topo'd condensation: Max = single
  `(value,argmax)` min-NodeId tiebreak, whole SCC shares, fully O(V+E); Any/All =
  unioned witness/false set over `{n}∪reach`, contributor set superlinear (EXC-1);
  CountDistinct STRICT per-member `\{n}` (C2/F8/F34) — `count(b)≠count(c)`, NOT a
  shared SCC result; set-union accumulator makes diamonds a no-op.
- **Tests (NEW `tests/condensation_fold.rs`, 6):** the G1 matrix
  `{Along,Against,None}×{Max,CountDistinct}` over one degraded `Reject` SCC,
  asserting value AND contributor identity vs an independent per-node-BFS oracle —
  the `None`×cyclic + `Against`×cyclic cells (the surfaces the prior suite could not
  see, no SCC fixture existed). R4 strict-exclusion fixtures (cyclic `a⇄b`+down, and
  a diamond). `scale_cliffs::evaluate_scales_quadratically…` inverted →
  `…near_linearly…` (measured 2.1× for 2× nodes; quadratic ≈4×), no longer ignored.
- **Threading:** `query::evaluate` gained `degraded_sccs: &BTreeMap<OverlayId,
  Vec<BTreeSet<NodeId>>>`; passed `&self.degraded_sccs` at the single lib.rs call
  site. `Graph::evaluate` PUBLIC signature unchanged. Old fold cluster (`fold_node`
  et al.) removed — `evaluate` no longer calls it and the repo denies dead_code.
  Resolved view bundled into a small `Resolved<'_>` struct to stay under the clippy
  arg-count ceiling.
- **Gotcha (audit/PHASE-03 watch):** the forbidden-vocabulary denylist (SPEC-001
  App B, `tests/denylist.rs`) WHOLE-WORD matches `project` — `project_flag` /
  "projects" tripped it; renamed to `member_value` / "restricts". Any cordage
  identifier or doc-comment with `project`/`task`/`schedule`/`capacity` etc. will
  fail the denylist suite. `NodeId` wraps a private `u32` with no public ordinal
  ctor — tests iterate captured builder ids (`usize::try_from(node.0)`; `usize::from`
  exists for the u16 `OverlayId` but not u32 `NodeId`).
- Gate: `just check` green on the combined coordination tree (full workspace);
  worker fork verified green via `cargo test -p cordage` + `cargo clippy -p cordage`.
