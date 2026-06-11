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
