# Design SL-043: cordage scale & robustness hardening

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

SL-036 shipped the `cordage` graph core at H1/H2 scale (hundreds of nodes; all VT
fixtures ≤5 nodes). SPEC-001 H1 was then revised to **~tens of thousands of
nodes**, and a post-close probe (codex + Opus, 2026-06-11) **empirically refuted**
the implicit "build is O(V+E), non-recursive" guarantee. SL-038 captured that as a
durable harness — generators (`deep_chain`, `diamond`, `dense_evict`) + four
`#[ignore]`'d characterization reds + `examples/scale_harness.rs` — and filed four
risks. **This slice is the green/fix half**: make the core hold at target scale and
flip the reds. No new harness work; SL-038 owns that.

Four defects, two files (`resolve.rs` build-time, `query.rs` query-time):

- **RSK-003-primary** — two recursive sites overflow the 8MB stack (SIGABRT, not
  slowdown) at depth ~80k, **inside** target scale.
- **RSK-003-secondary** — eviction-to-fixpoint recomputes a whole-graph SCC pass
  per evicted edge → O(E·(V+E)).
- **RSK-004** — `evaluate()` runs a fresh reachable-BFS per node → O(V·(V+E)).
- **RSK-002** — `explain()` materialises every predecessor chain → 2^layers on
  diamond lattices.
- **RSK-001** — the `Against`-orientation re-map is correct but untested by any VT
  (coverage gap, not a defect).

## 2. Current State

| site | file:line | current behaviour | cost |
|---|---|---|---|
| `Tarjan::strongconnect` | resolve.rs:321 | recursive DFS, depth = graph depth | stack O(depth) → overflow |
| `level_of` | resolve.rs:545 | recursive memoised longest-path | stack O(depth) → overflow (independent of Tarjan; overflows on a clean acyclic chain) |
| `pass2_evict` | resolve.rs:198 | `loop { cyclic_components(all); evict global-F17-min participant }` | O(E·(V+E)) |
| `evict_layer_cycles` | resolve.rs:478 | same shape over `U`, `layer_k`-evictable only | O(E·(V+E)) |
| `evaluate` | query.rs:256 | `for n: reachable(n)` fresh BFS per node | O(V·(V+E)) |
| `predecessor_paths`/`extend_chains` | query.rs:80/138 | DFS enumerating **every** root→node chain; `suffix.clone()` per branch | exponential (2^layers) + O(path_len) copy/path |
| `orient` (Against) | resolve.rs:463 | correct src/dst swap | — (untested) |

Public surface touched: only `Explanation` (RSK-002). **No non-test consumer reads
`explain`/`paths()`** (confirmed: lib.rs internal wiring + cordage tests +
`examples/scale_harness.rs` demo only). `evaluate`'s signature is unchanged
(internal fold rewrite). Everything else is private-mechanism.

## 3. Forces & Constraints

- **Behaviour-preservation gate (project canon).** The existing suites (79 tests)
  are the proof and must stay green **unchanged** — *except* the four `explain`
  tests, which deliberately re-assert the new sub-DAG shape (the one sanctioned
  break, no real consumer). Determinism contracts F17/F21/F37 (eviction selection
  by the F17 key; provenance sorted by `(overlay, edge)`, never detection order)
  hold byte-for-byte.
- **Pure/imperative split (slices-spec §Arch).** All four fixes live in the pure
  layer; iterative rewrites use an **explicit `Vec` stack**, no clock/rng/IO.
- **ADR-001.** `cordage` is a leaf; **no new dependency** — std-only, hand-rolled.
- **SPEC-001 Appendix B (forbidden-core).** No time/urgency/scheduling/product
  vocabulary may enter the crate via any fix.
- **SPEC-001 D11 / F13.** `explain` returns **role-agnostic structure only**; the
  core privileges no rendering (policy interprets). F13 already ripped
  `blockers`/`spine` *out* of the public struct for this reason.
- **F47 / I1.** `Reject` traversal views may stay cyclic; `reachable`/`evaluate`/
  `explain` are cycle-safe **by contract** — no fix may narrow that to acyclic-only.
- **Published-grade (H4).** API choices must suit a future external/policy consumer,
  not just today's tests.

## 4. Guiding Principles

- **Target O(V+E) where the algorithm permits; name every exception.** Two
  inherent exceptions are accepted and documented, not papered over:
  - **EXC-1 — set-valued contributors (`Any`/`All`/`CountDistinct`).** Their
    output `contributors` (F43) is a *set* — `Any` true-witnesses, `All`
    present-false/true, `CountDistinct` the counted set — so even though the
    *value* folds in O(V+E), reproducing the contributor set needs a set-union
    fold, superlinear in set size (the output itself is up to O(V) per node;
    `contributors` is already O(V²) worst-case). **Only `Max` is fully O(V+E)** —
    its contributor is a single argmax (min-`NodeId` tiebreak), not a set.
  - **EXC-2 — dense single SCC eviction.** Strictly-sequential F17-min eviction
    (one edge per round, each round's victim depends on the prior) cannot be
    parallelised without risking a *different* evicted set → contract-bound
    superlinear on one pathological dense SCC. The common case (small sparse
    cycles) is effectively linear after localization.
- **Ride existing seams; no parallel implementation.** Iterative Tarjan is the
  single SCC primitive shared by RSK-003-primary *and* -secondary. RSK-004 reuses
  **build-time** `degraded_sccs` (already on `Graph`) rather than a second
  query-time Tarjan.
- **Preserve the determinism contract as the proof.** Where an optimisation
  reorders work, the *output* (sorted provenance, folded values, cone structure)
  must be identical — asserted, not assumed.

## 5. Proposed Design

### 5.1 System Model

Three independent fix clusters, phased by file/concern (§ Lifecycle). No new
modules, types relocations, or dependencies. The only public-type change is the
`Explanation.paths` → `Explanation.predecessors` reshape.

### 5.2 Interfaces & Contracts

**RSK-002 — `explain` output reshape (the only public change).**

```rust
// before:  paths:        BTreeMap<OverlayId, Vec<Vec<NodeId>>>      // enumerated chains
// after:   predecessors: BTreeMap<OverlayId, BTreeMap<NodeId, BTreeSet<NodeId>>>
//                                            node -> its in-cone predecessors
```

- The cone is the predecessor sub-DAG of the explained node `n`, per overlay:
  `node ↦ {immediate predecessors within the cone}`.
- **Roots** = cone keys with an empty pred-set.
- **F47 termination preserved structurally**: a degraded-SCC-entry node appears as
  a key with empty preds (endpoint, never walked through); if `n` is itself inside
  a degraded SCC its cone is `{n: {}}` (the old `[[n]]`). Policy reconstructs any
  chain/spine/witness by walking the cone — consistent with `spine_path` staying an
  *accessor*, not a field (D11/F13).
- Accessor rename `Explanation::paths()` → `predecessors()`; field doc updated.
- **Rejected**: bundling a canonical witness chain (re-introduces the privileged
  materialization D11/F13 forbid — policy derives it).

`Graph::evaluate` signature is **unchanged**; `query::evaluate` gains a
`degraded_sccs: &BTreeMap<OverlayId, Vec<BTreeSet<NodeId>>>` parameter (threaded
from `Graph`, already held).

### 5.3 Data, State & Ownership

- No new persisted/authored state. All work is pure transformation of the existing
  in-memory indices (`OutIndex`/`InIndex`/`degraded_sccs`).
- Iterative rewrites replace recursion-stack state with an explicit `Vec` work
  stack — same memoisation maps (`index`/`lowlink`/`cache`), same `BTreeMap`/
  `BTreeSet` determinism keys.

### 5.4 Lifecycle, Operations & Dynamics

**P1 — `resolve.rs` (RSK-003 both + RSK-001).**

- **Iterative Tarjan** — `strongconnect` becomes an explicit-stack DFS. Each stack
  frame carries `(node, successor-iterator-position)`; successors walked in
  `BTreeSet` order (discovery order **identical** to the recursive form → identical
  SCC output). Lowlink update on return mirrors the recursive `min`.
- **Iterative `level_of`** — explicit-stack post-order longest-path over `preds`,
  same memo `cache`; a node is finalised only after all parents resolved
  (push-children-then-revisit pattern). Identical levels.
- **Eviction localization** — one Tarjan → SCCs (vertex-disjoint). Process each
  cyclic component independently: evict its F17-min participating edge, **re-Tarjan
  only that shrinking sub-component** to fixpoint, then next component. Disjointness
  ⇒ the evicted **set** is identical to the global loop; provenance sorted ⇒
  identical output. Same treatment for `evict_layer_cycles` (only `layer_k` edges
  evictable; the localized component is the U sub-component). EXC-2 governs the
  dense-single-SCC residue.
- **RSK-001 VT** — a direct test that an `Against` `OrderLayer` produces the swapped
  (dst→src) oriented edge in `U` / the resulting order, characterizing the existing
  `orient` path. Passes on first write.

**P2 — `query.rs` (RSK-004).**

- **Condensation fold**, once per `evaluate`:
  1. SCC partition: `Evict` overlay ⇒ every node its own trivial SCC; `Reject` ⇒
     stored `degraded_sccs[overlay]` are the cyclic SCCs, all others singleton.
  2. Reverse-topological order over the condensation DAG (sinks first), O(V+E).
  3. Fold each SCC = combine(member seeds, already-folded successor-SCC results);
     every node in an SCC shares the SCC result (mutual reachability).
  - **Max** — own-seed included; whole SCC one `(value, argmax)`, min-`NodeId`
    tiebreak folded up. Fully **O(V+E)** (singleton contributor).
  - **Any / All** — own-seed included (`{n}∪reachable`); value folds in O(V+E),
    but `contributors` is a *set* unioned up the condensation → **EXC-1**.
  - **CountDistinct** — STRICT (own-seed excluded), witness *set* unioned; node
    `n`'s witnesses = `SCC_set \ {n} ∪ downstream` (the F8 strict-exclusion is an
    O(1) per-node removal). **EXC-1**.
- Output `values`/`contributors`/`diagnostics` identical to the per-node-BFS form
  (asserted against the pre-fix suite).

**P3 — `query.rs` + `lib.rs` (RSK-002).**

- Replace `predecessor_paths`/`chains_to_root`/`extend_chains` with a single
  cone-builder: BFS/DFS the in-edges from `n` up to roots / degraded-SCC entries,
  recording `node ↦ preds` adjacency, **visiting each cone node once** (visited
  set), no `suffix.clone()`, no enumeration. O(V+E).
- Rewrite `lib.rs` `Explanation` field + `explain()` assembly + the four
  `tests/explain.rs` cases to assert the cone.

### 5.5 Invariants, Assumptions & Edge Cases

- **Determinism (F17/F21/F37)** — eviction still selects the F17-min; provenance
  still sorted by `(overlay, edge)`. Localization changes *work order*, never output.
- **Cycle-safety (F47/I1)** — preserved: condensation handles cyclic Reject views;
  the cone builder stops at degraded-SCC entries exactly as today.
- **Strict reachability (F8)** — `n` never in its own reachable set, even cyclically;
  the CountDistinct fold subtracts `n` from its SCC witness set.
- **Empty/edge cases** — empty graph, single node, self-loop SCC, `Direction::None`
  (∅) all behave as before (existing tests cover; unchanged).
- **Iterative-rewrite equivalence** — assumed identical to recursion; *proven* by
  the unchanged build/resolution/ordering suites staying green.

## 6. Open Questions & Unknowns

- **OQ-1 — REQ pin (carry-over).** Exact REQ subset under SPEC-001 to record in
  `relationships.requirements` once perf numbers are in. Resolve at `/plan`.
- **OQ-2 — IMP-020 fold-in.** RSK-004 (evaluate walk) and RSK-002 (cone builder)
  both rewrite traversals IMP-020 wants consolidated (`reachable`/`spine_path`/
  `extend_chains` triplication). **Default: out** — folding the consolidation in
  widens scope from risk-closure to a traversal refactor. Revisit only if the
  rewrites naturally converge a shared helper; otherwise IMP-020 stays its own item.
- *(Resolved during design)* RSK-002 output shape → pure sub-DAG (D2). explain has
  no non-test consumer (the OQ-1 from scope is closed).

## 7. Decisions, Rationale & Alternatives

- **D1 — Fixes-only slice; SL-038 owns the harness.** The generators + reds + demo
  already exist; SL-043 implements fixes and flips signals.
- **D2 — RSK-002 = pure predecessor sub-DAG**, no bundled witness chain. *Rationale*:
  SPEC-001 D11/F13 mandate role-agnostic structure; the sub-DAG is that structure,
  is lossless, and is O(V+E). *Alternatives rejected*: sub-DAG + canonical witness
  (re-privileges a materialization); direct-preds + one chain (lossy).
- **D3 — RSK-004 = build-SCC condensation fold, combinator-split.** *Rationale*:
  cycle-safety is a standing contract (F47), condensation is the rigorous handling,
  and `degraded_sccs` is already computed at build → no query-time Tarjan. EXC-1
  (CountDistinct) accepted as inherent. *Alternative rejected*: assume acyclic
  overlays (narrows the contract); cache-reachable-sets (no asymptotic win).
- **D4 — RSK-003-secondary = per-SCC localization; retarget the signal.** *Rationale*:
  SCC disjointness makes localization provably output-identical; a single dense SCC
  is contract-bound superlinear (EXC-2). The `dense_evict` red can't become
  linear-green, so it stays an `#[ignore]` characterization (re-doc'd as the EXC-2
  exception) and a **new green gate proves the real win** — N independent small
  cycles evict ~linearly in N — plus an assertion the evicted **set** is unchanged.
  *Alternative rejected*: co-evict "disjoint minimals" for true linearity (risks a
  different evicted set → violates behaviour-preservation).
- **D5 — RSK-003-primary = iterative both sites.** Shared iterative Tarjan also
  serves D4's re-test. Mechanical; determinism preserved by BTreeSet walk order.
- **D6 — Phasing = 3, by file/concern** (P1 resolve.rs, P2 query.rs evaluate, P3
  query+lib explain). Dependency-ordered (Tarjan before localization); each phase
  flips its own red(s).

## 8. Risks & Mitigations

- **R1 — Iterative rewrite subtly diverges from recursion** (wrong SCCs/levels).
  *Mitigation*: the unchanged resolution/ordering/golden_net suites are the
  equivalence proof; they must stay green with zero edits.
- **R2 — Localization changes the evicted set** (not just order). *Mitigation*:
  explicit "evicted set unchanged vs pre-fix" assertion (D4); disjointness argument
  documented; provenance goldens unchanged.
- **R3 — `explain` test rewrite masks a regression** (the one sanctioned break).
  *Mitigation*: the new cone tests must assert the *same* reachable predecessor
  membership the old chains covered (roots, SCC-entry endpoints, `[[n]]` case),
  re-expressed as adjacency — not a weaker assertion.
- **R4 — CountDistinct strict-exclusion off-by-one** in the SCC fold. *Mitigation*:
  targeted cyclic-SCC + diamond fixtures asserting `n ∉ own witnesses` while
  `n ∈ predecessors'` witnesses.
- **R5 — Clippy ceilings** (the repo denies indexing-slicing, `as`, HashMap,
  unwrap-in-bins). *Mitigation*: explicit stacks use `Vec`/`BTreeMap`, `.get()`,
  guarded conversions — mirror the existing Tarjan's documented avoidances.

## 9. Quality Engineering & Validation

- **TDD per phase, red→green→refactor.** P1/P2 reds already exist (SL-038
  `#[ignore]`'d) — un-ignore / invert as each fix lands. P3 rewrites the four
  `explain` tests to the cone.
- **Signal flips:** `deep_chain_overflows` → assert build *succeeds* at 80k;
  `evaluate_scales_quadratically` → assert near-linear scaling (was quadratic);
  `explain_path_count_is_exponential` → replaced by a cone-shape/size assertion;
  `eviction_fixpoint` → `dense_evict` stays `#[ignore]` (EXC-2), **new**
  many-small-cycles linear gate added.
- **Cliff bounds budget for debug ≈10× release**
  (`mem.pattern.testing.debug-vs-release-scale-timing`); seed opaque ids from the
  builder, never `NodeId(0)` (`mem.pattern.cordage.opaque-ids-capture-from-builder`).
- **Behaviour-preservation:** full pre-existing suite green, unchanged, except the
  four `explain` tests.
- **Gate:** `cargo clippy` zero warnings (bins/lib only, not `--all-targets`);
  `just check` before each commit.

## 10. Review Notes

### Internal adversarial pass (2026-06-11)

- **A-1 (material, FIXED).** "Idempotent combinators are O(V+E)" was wrong:
  `Any`/`All` `contributors` are *sets* (F43), not just `CountDistinct` — so they
  need the same set-union fold and fall under EXC-1. Only `Max` (singleton argmax)
  is fully O(V+E). §4 EXC-1 and §5.4 corrected. The *value* still folds in O(V+E)
  for all idempotent combinators; the contributor *set* is what's superlinear.
- **A-2 (checked, holds — the linchpin).** RSK-004 reuses build-time
  `degraded_sccs` only if it captures *exactly* the cycles `evaluate`'s BFS would
  see. It does: `degraded_sccs[overlay] = cyclic_components(post-arity working set)`
  (resolve.rs:179) and the query traverses that same resolved per-overlay set →
  Evict overlays are acyclic (singletons), Reject cycles are precisely the stored
  SCCs. No cyclic view escapes the condensation. If this ever drifts the fold breaks
  silently — flagged for the verifier.
- **A-3 (checked, holds).** Eviction localization gives the identical evicted *set*:
  cyclic components are vertex-disjoint; evicting an edge in component C can only
  split C, never alter another component; the global loop's cross-component
  interleaving doesn't change per-component outcomes; provenance is sorted →
  identical output. The `dense_evict` residue (EXC-2) is one component that can't
  be split cheaply — contract-bound, not a localization failure.
- **A-4 (checked, holds).** `level_of` operates on `U`, acyclic by I1 — the
  iterative rewrite needs no cycle guard (unlike Tarjan); memo + post-order
  suffices. Determinism: `max` over parents is order-independent.
- **A-5 (checked, holds).** RSK-002 cone uses a *global* visited set (mark each
  node once) — that is the linearisation; the old per-path visited set
  (`visited.remove` on backtrack, query.rs:183) is exactly what made it
  exponential. Each node still records all its in-edges; degraded-SCC entries are
  recorded as empty-pred leaves.

### Carried risks into /plan
- R3 (explain test rewrite must not weaken assertions) and A-2 (degraded_sccs
  equivalence) are the two the phase verifier must guard explicitly.
