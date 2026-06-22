# Notes SL-140: Unify cordage traversal: reachable/spine_path/extend_chains

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Implementation (commit 0b7d65ea)

Single-phase, plan/phase-sheet ceremony waived (small leaf refactor, design.md
the governing spec). Behaviour-preserving — no test assertions changed.

- `crates/cordage/src/query.rs`: added `walk_bfs(start, neighbours) -> Vec<NodeId>`
  (discovery-order BFS, generic over `I: IntoIterator<Item = NodeId>`). `reachable`
  → `walk_bfs(..).into_iter().skip(1).collect()`; `spine_path` →
  `walk_bfs(.., single_parent)` reversed. `cone_on_overlay` unchanged + doc comment
  recording why it does not ride `walk_bfs` (D2).

### Gate (all green)

- `cargo test -p cordage` — 14+8+8+9+6(cond_fold)+8(golden_net)+10+… all pass,
  zero assertion changes. Baseline captured green before the edit.
- `cargo clippy -p cordage` — zero warnings.
- `just check` — exit 0 (workspace).

### Equivalence (proven, R3)

- `reachable`: `start` is `walk_bfs` index 0, never re-emitted (visited seeded);
  `skip(1)` drops exactly it ⇒ strict set. `Direction::None` ⇒ empty neighbours ⇒
  `[start]` ⇒ ∅.
- `spine_path`: `single_parent` yields ≤1 ⇒ linear discovery `node→…→root`;
  `reverse` ⇒ ancestor-first; cycle re-entry excluded by visited (was `break`).

### Residual / follow-up

- R2: no *direct* test of `spine_path` on a cyclic `AtMostOne` Reject overlay —
  covered transitively via shared `walk_bfs` (whose cycle-safety is asserted by
  `reachable`'s a↔b test). Optional characterization test; out of this slice's gate.

## Close-out (trimmed audit/reconcile, option 2)

No design drift to reconcile: implementation matches design.md §5 exactly (one
`walk_bfs`, two thin callers, cone explicit + documented). Evidence = green gate
(above), proven equivalence (R3), behaviour-preservation via two independent
reachable oracles.

RV ledger (minimal): one finding, R2 → **CHR-022** (linked `slices SL-140`),
dispositioned *deferred* (out of slice scope, behaviour covered transitively).
No other open findings. Public API unchanged ⇒ no downstream reconciliation.

Transitions: started → audit → reconcile → done.
