# Review RV-002 — reconciliation of SL-043

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance audit of SL-043 (cordage scale & robustness hardening) — three
phases implemented via the dispatch funnel, all green on main @ 9627b0e. The
slice retired four scale/robustness defects (RSK-001/002/003/004) by rewriting
`resolve.rs` (build-time) and `query.rs` (query-time) to hold at the
tens-of-thousands target, with one sanctioned public break.

This audit reconciles the implementation against `design.md`, `plan.toml`, and
SPEC-001, not against a spec engine (doctrine has none). The lines of attack are
the load-bearing claims the design's three adversarial passes flagged for the
verifier to guard, plus the residuals and divergences closure must own:

**Mandatory guards (design §10 — "verifier MUST guard explicitly"):**
- **G1** — the condensation fold partition AND its condensation-DAG edges +
  reverse-topo are direction-resolved (`{Along,Against,None}`), never the forward
  build adjacency. The `None×cyclic` (C1) and `Against×cyclic` cells are silent
  corruption invisible to the pre-existing suite. Demands the mandated fixture
  matrix `{Along,Against,None}×{Max,CountDistinct}` vs an independent per-node-BFS
  oracle.
- **G2** — the layer-k localization precondition (every U-cycle at layer k holds
  a `layer_k` edge): asserted directly, OR via the unchanged `compose_order`
  goldens that depend on it.
- **G3** — the four rewritten `explain` cone tests assert the *same*
  reachable-predecessor membership the old chains covered, not a weaker shape.

**Named exceptions to confirm are honestly bounded:**
- **EXC-1** — set-valued contributors (`Any`/`All`/`CountDistinct`) are
  superlinear in set size; only `Max` is fully O(V+E). Accepted inherent.
- **EXC-2** — dense single-SCC eviction stays superlinear (scope-bound/deferred,
  not an inherent floor); `eviction_fixpoint` stays `#[ignore]` as the marker.

**Behaviour-preservation (R1):** the pre-existing suite is the equivalence proof
— it must be green UNCHANGED except the four sanctioned `explain` rewrites.

**Closure divergences:** the slice's defining risk records (RSK-001..004) and the
adjacent ISS-003 / IMP-020 backlog items must be reconciled to code-truth before
close, and the `proposed → … → done` lifecycle (currently `proposed ⚠ 3/3`) must
be driven via the transition verb, not hand-edited.

Evidence gathered: `just check` green (full workspace); `cargo test -p cordage`
green with exactly one `#[ignore]` (`eviction_fixpoint_scales_superlinearly`,
EXC-2); `cargo clippy -p cordage` zero warnings; public reshape
`Explanation.paths()` → `predecessors()` confirmed landed (lib.rs:281/301).

## Synthesis

**Verdict: audit-ready for close.** All nine findings terminal (verified), no
unresolved blocker, the close-gate (D-C9b) will not refuse. The implementation
matches `design.md`, `plan.toml`, and SPEC-001; the three adversarial-review
guards the design flagged for the verifier all hold, the two named complexity
exceptions are honestly bounded, and the behaviour-preservation gate — the
load-bearing equivalence proof — is green unchanged but for the one sanctioned
break.

**The closure story.** SL-043 retired a four-defect risk cluster
(RSK-001/002/003/004) by rewriting two files to hold at the tens-of-thousands
target: iterative Tarjan + iterative `level_of` + per-component eviction
localization in `resolve.rs`; a direction-resolved condensation fold replacing
the per-node BFS in `query.rs evaluate`; and a linear predecessor-cone builder
replacing the exponential chain enumeration in `query.rs`/`lib.rs explain`. The
silent-overflow and exponential cliffs are gone — `deep_chain(80k)` builds, the
quadratic/exponential signals are inverted to linear gates, and the SIGABRT sites
are iterative. The equivalence argument is consumer order-insensitivity (A-4),
not byte-identical emission order, and the unchanged suite is its backstop.

**The three guards (G1/G2/G3) — all held.** G1 (the #1 guard): the condensation
fold's partition AND its DAG edges/reverse-topo are direction-resolved, proven by
the mandated `{Along,Against,None}×{Max,CountDistinct}` matrix vs an independent
per-node-BFS oracle — including the `None×cyclic` and `Against×cyclic` cells the
prior suite could not see. G2: the layer-k localization precondition is guarded by
the OR-goldens form plan VT-1 sanctioned (no positive fixture — a candidate
hardening, not a gate). G3: the explain cone tests re-assert the same predecessor
membership as the retired chains, with `golden_net.rs` permutation-invariance as
witness.

**Standing risks consciously accepted.**
- **EXC-1** (tolerated) — set-valued contributors for `Any`/`All`/`CountDistinct`
  are superlinear in set size; only `Max` is fully O(V+E). Inherent in the output
  shape (F43), named not papered over.
- **EXC-2** (tolerated) — one dense single SCC still re-Tarjans per eviction round
  (O(E·(V+E)) within that component). Scope-bound/deferred, NOT an inherent floor
  (codex R2 corrected the earlier over-claim); decremental SCC maintenance could
  hold the same evicted set sub-quadratically but is not worth the std-only /
  determinism / complexity cost. Captured by the lone remaining `#[ignore]` as the
  deferred-residual marker; OQ-3 sets the conditional trigger to file an IMP only
  if a dense-SCC workload ever appears.
- **G2 positive fixture** (tolerated) — the direct layer-k invariant assertion is
  a candidate hardening if revisited; the OR-goldens form is sanctioned.

**Reconciliations performed in-audit.** RSK-001/002/003/004 transitioned to
`resolved/mitigated` (they were code-retired but still read `open` — ledger now
matches code-truth). ISS-003 left open and flagged for rescope: SL-043's
`paths()`→`predecessors()` reshape changed the surface it cites (its doc-vs-
behaviour framing references a field that no longer exists; the foreign-node
lone-endpoint behaviour persists as `{node:{}}`). IMP-020 (OQ-2 traversal
triplication) correctly stayed its own open item — the P2/P3 rewrites did not
naturally converge a shared helper, so no fold-in occurred.

**Process note (not a slice finding).** This audit's RV was opened in the parent
tree, not the dispatch worktree fork: the review baton lives in the parent tree's
gitignored runtime state, so `review raise` refuses under a worker fork
(IMP-024). An orphaned RV-002 created in the fork is unreachable and unneeded;
this RV-002 in the parent tree is the canonical one. IMP-024 owns the fix.

**Lifecycle.** SL-043 currently reads `proposed ⚠ 3/3` (the SL-009 hand-status vs
rollup divergence). Reconcile via the lifecycle-transition verb at `/close`
(`doctrine slice status 43 …` through audit→reconcile→done), not by hand-editing
`slice-043.toml`. Hand off to `/close`.
