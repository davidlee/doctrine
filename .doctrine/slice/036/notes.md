# Notes — SL-036 cordage graph core

## Design-stage assessment after adversarial round 2 (2026-06-10, commit c726044)

Verdict given to user: foundation sound, fit for purpose; residual risk is not
correctness but the unused API. Three open concerns flagged (observations, not
integrated findings — candidates for round 3 or the adapter slice):

- **Explanation path enumeration blowup.** `Explanation.paths` enumerates every
  chain to root (`Vec<Vec<NodeId>>`); a diamond lattice has exponentially many
  paths in depth — bites even at ~50 nodes. Neither external reviewer landed it
  (GPT m36 adjacent). Fix direction: return the predecessor sub-DAG (or direct
  + one canonical chain); policy enumerates on demand.
- **Degraded taint is maximally conservative.** One dep cycle near the root
  degrades everything downstream in `U`. Correct under REQ-076 but blunt; the
  gentler still-not-false alternative is condensation ordering (SCC members tie
  at equal level, NodeId-broken). Core-internal change, no interface impact —
  defer until a consumer complains, but expect it.
- **Pre-consumer API churn (risk R2 realised).** Validating consumer is a
  fixture suite. Opaque-handle-heavy API (OverlayId/OrderSpec/ChannelSpec/seed
  maps) means semantically-wrong-but-valid wiring compiles. Expect a
  usage-driven interface rev when adapter/policy slices land; cheap while
  workspace-internal. Recommendation given: lock after one diminishing round —
  remaining unknowns are findable by the first consumer, not by more review.

Performance explicitly assessed a non-concern at H1/H2 scale (worst cases
microseconds at hundreds of nodes); do not spend review budget there.

## Round-2 process facts

- 55 external findings (GPT-5.5: 41, Opus: 14) deduped to F10–F29 (20
  integrated, 3 rejected with reasons) — full source map in design.md §10.
- F11 (per-layer lexicographic order_key unsound: level equality ≠
  incomparability) was self-found during integration — neither reviewer caught
  it. Lesson: integrating a fix is itself a review pass on adjacent machinery.
- F19 confirmed by checking SPEC-001 D4/D5 directly: design pass 1 kept the
  (rank,age)-MIN parent while spec eviction removes the min (weakest) — i.e.
  round 1 kept the weakest. Cross-checking the parent spec caught what both
  externals only smelled.
- Upstream wording note parked in design.md §6: SPEC-001 D9/D10 "seq *rank*
  within a dep-eligible set" should read seq-*topology*; rank is eviction
  strength only. Post-lock SPEC revision, same channel as T1.

## Round-3 outcome (2026-06-10)

NOT a diminishing round: 15 combined external findings (web + GPT-5.5 + Opus,
user-deduped) → F30–F44, all accepted (2 partial, 1 alternative fix), 0
rejected, none rehash; plus self-found F45 (the F11 pattern again — found
re-deriving F34's machinery). Four blockers in two interaction families:

- **F30** — pass-1 arity eviction broke authored Reject cycles before pass-2
  detection saw them (diagnostic silently lost). Fix: Reject detection on the
  authored pre-arity set; authored SCC = the one cycle concept.
- **F31/F32/F33** — Degraded/taint mis-scoped three ways: seeded from non-spec
  overlays, exclusion-from-U ambiguous (taint-defeating under the natural
  reading), suffix-by-NodeId violated surviving U edges (I2 literally false).
  Fix: spec-scoped seeds, intra-SCC-only exclusion, `Degraded(u32)` carrying
  U-level so the suffix respects surviving edges.

Lesson: both families are *interaction* bugs between individually-sound parts
(pass pipeline; degradation × ordering) — per-section review missed them three
times. Known-open list updated: **taint conservatism partially addressed**
(suffix now edge-respecting, non-spec overlays excluded); residual
conservatism = F30's authored-SCC degradation when arity already broke the
cycle, and full-downstream taint extent — both deliberate, revisit on consumer
complaint. Path-enumeration blowup and API churn remain open, untouched by
round 3 (no reviewer sharpened them).

Round 3 was billed FINAL, but it found 4 blockers — recommendation to user:
one more cheap external pass over the round-3 rewrites (pass 2/3/4 + the
propagation contract) before lock; the blocker trend has not yet hit zero.

## Round-4 outcome (2026-06-10, GPT-5.5 via codex MCP, run in-session)

User authorised running round 4 directly. 3 findings → F46–F48, all accepted:

- **F46** — round 3's F30 "one cycle concept" call reversed with evidence:
  authored-SCC keying of pass-3 exclusion/pass-4 taint destroyed surviving
  valid resolved edges when arity had already broken the cycle. Now: authored
  SCC → diagnostic only; post-arity SCC → order degradation. (This was the
  alternative I weighed and rejected for simplicity at F30 integration — the
  residual-conservatism note from round 3 is RESOLVED, not residual.)
- **F47** — explain()'s "chains to root" was impossible on cyclic Reject views
  (no root). Chains now end at roots or degraded-SCC entry; SCC members are
  endpoints only; in-SCC nodes get [[n]].
- **F48** — I1 wording over-claimed traversal-view acyclicity; tightened.

Trend: blockers 2 (round 2+self) → 4 (round 3) → 1 (round 4), and round 4's
blocker was a choice-cost, not a machinery bug. Remaining known-open: path
enumeration blowup (F47 bounds termination, NOT combinatorics — predecessor
sub-DAG still the fix direction), full-downstream taint extent, pre-consumer
API churn. Assessment: diminishing reached; findable-by-review surface looks
exhausted — remaining unknowns belong to the first consumer. Recommend lock.

## PHASE-01 implementation (2026-06-10) — crate skeleton & model contract

Shipped `crates/cordage` as the second workspace member: full §5.2 declarative
vocabulary, `GraphBuilder → build() → Graph`, build-input validation
(F14/F22/F38), BTree adjacency with a derived reverse index, and `out_edges`/
`in_edges`. **No resolution passes** (arity/cycle/U/order_key) — those are
PHASE-02+. 16 tests across construction / build_validation / adjacency; whole
`just check` green.

Load-bearing decisions (also in the phase sheet):
- **D(A-2) flat crate-root public API.** All public types in `lib.rs` root
  (`cordage::NodeId`, not `cordage::model::NodeId`). Forced by `pub_use` deny —
  the only route to a flat re-export-free API — and it dodges
  `module_name_repetitions` (`graph::Graph`). The payoff is **path stability**:
  PHASE-02+ logic moves into *private* modules (child modules read the root's
  private fields), public type paths never move.
- **Opaque-token discipline.** `NodeId`/`OverlayId` have no public ctor and no
  ordinal accessor; tests mint foreign ids from a *sibling* builder. Value types
  (`EdgeAttrs`/`OverlayConfig`/`ChannelSpec`) expose `new` + accessors.
- **Adjacency Ord is explicit (F21).** Private `OutEdge`/`InEdge` with hand-written
  `Ord` over `(dst,rank,age)` / `(src,rank,age)`; `BTreeSet` membership gives the
  A-4 identical-edge dedupe for free (key spans all fields). Kept as two structs
  deliberately — collapsing them would re-hide the per-direction key.
- **OrderSpec validated then discarded** this phase (storing an unread field trips
  `dead_code`); PHASE-03 re-stores it as the first consumer of order composition.

Carry-forward for PHASE-02 (`/phase-plan` reading):
- `GraphBuilder` keeps `overlays: Vec<OverlayConfig>` (config contents currently
  read only via accessors); the arity pass will read `.arity()`/`.cycle_policy()`.
- `Graph` stores only `out`/`incoming` indices today. Pass-1/2 will need node
  count + per-overlay configs threaded into `Graph` (or computed in `build()`
  before the indices) — neither is stored yet.
- `BuildError` has no `Display`/`Error` impl yet (deferred — would need a `NodeId`
  Display or a `use_debug`-denied `{:?}`); add when a consumer propagates via `?`.

## PHASE-02 implementation (2026-06-11) — build passes 1–2

Shipped `build()` passes 1 (arity) and 2 (per-overlay cycle resolution) in a new
private `src/resolve.rs`, plus the public provenance vocabulary (`EdgeRef`,
`EvictReason`, `EvictedEdge`, `CycleDiagnostic`, `Provenance`, `Graph::provenance()`).
`out_edges`/`in_edges` now read the **resolved** view. 14 black-box resolution
tests; whole `just check` green. Authored→resolved transform lives entirely in
`resolve()` — `build()` validates, calls `resolve`, then feeds the resolved flat
list to the existing `build_indices`.

Load-bearing decisions (also in the phase sheet):
- **Single internal `Edge` ordered by the F17 eviction key `(rank,age,src,dst)`.**
  Per-overlay working set is `BTreeSet<Edge>`, so `.min()`/`.max()` select by the
  eviction key *directly* (F37 satisfied by construction — the adjacency `OutEdge`/
  `InEdge` keys never drive selection) and set membership dedupes identical edges.
- **The F30/F46 two-SCC split** (the round-3/4 trap): authored pre-arity SCC →
  `CycleDiagnostic` (always surfaces the authoring error, even after arity breaks
  the cycle); post-arity SCC → degraded marks. Computed as two distinct
  `cyclic_components` calls — authored set vs the pass-1 output. Confirmed by the
  `arity_breaks_authored_reject_cycle_diagnostic_still_emitted` test.
- **Tarjan SCC over `BTreeMap<NodeId,_>` state**, adjacency walked in `BTreeSet`
  order → deterministic discovery with no ordinal Vec-indexing (sidesteps the repo
  `indexing-slicing` + `as`-cast bans). Self-loop = single-node SCC marked cyclic
  only when an `n→n` edge exists (F20). Recursion borrow dodged by copying the
  `&'a` adjacency ref out of `&mut self` before recursing.
- **`degraded_sccs` stored on `Graph` (D-1, user-confirmed)** to satisfy EX-2
  literally; write-only this phase → `#[expect(dead_code, reason=…)]`. **PHASE-03
  removes the expect when pass-3/4 first reads it** — do not add a PHASE-02 test
  that reads the field (would make the expect unfulfilled).
- **`configs`/`node_count` NOT stored on `Graph`** — passes 1–2 need configs only
  at build time (threaded as a `resolve()` arg) and never need `node_count` (arity/
  cycle concern only edge-touched nodes). The PHASE-01 carry-forward anticipated
  threading them into `Graph`; deferred to PHASE-03/04 which are their real
  consumers (`node_count` for the longest-path level, `configs` for `spine_path`).

Carry-forward for PHASE-03 (passes 3–4, order composition):
- The resolved per-overlay edge sets are in `Graph.out`/`incoming` (Evict overlays
  acyclic; Reject overlays may stay cyclic — the traversal view). `degraded_sccs`
  holds the post-arity cyclic SCCs of Reject overlays, keyed by overlay — the taint
  seeds, after filtering to OrderSpec-referenced overlays (F31).
- `OrderSpec` is still validated-then-discarded (PHASE-01). PHASE-03 must re-store
  it (the first consumer) plus `node_count` (level recurrence is total over ALL
  nodes) and `configs` (if `spine_path` lands here vs PHASE-04).
- `EvictReason::UnionCycleVsLayer` already declared; pass-3 is its first producer.
- Provenance sort is `(overlay, edge)` for evictions / `(overlay, nodes)` for
  cycles — a reporting sort distinct from the F17 selection key. Keep pass-3
  `UnionCycleVsLayer` evictions flowing through the same `sort_provenance`.

## PHASE-03 implementation (2026-06-11) — build passes 3–4, order surface

Shipped passes 3 (compose `U`) and 4 (`order_key` materialization) plus the
public ordering surface (`Level`, `OrderKey`, `Graph::order_key`, `Graph::ordered`).
9 black-box tests in `tests/ordering.rs` (VT-1..9); whole `just check` green.

Seam: `build()` now re-stores `OrderSpec` + `node_count` on `Graph` and calls a
private `Graph::compose_order()` that READS `self.degraded_sccs` → cleared the
`#[expect(dead_code)]` cleanly (D1 satisfied). `degraded_sccs`/`order_spec` stay
stored for PHASE-04/05.

Load-bearing decisions:
- **`U` is a `BTreeSet<Edge>` (resolve.rs `Edge`, F17 Ord)** — `cyclic_components`/
  `participates`/`.min()` reused verbatim. No second Tarjan, no second key.
- **`compose_order` takes `&self.out` (the resolved adjacency)**, not a re-derived
  edge list. `overlay_edges()` lifts an overlay's resolved edges back into `Edge`.
  Keeps `Edge` private to resolve.rs; resolve is a descendant module so it reads
  `OutEdge`/`NodeId`/`OrderKey` private fields directly (no public widening).
- **D2 resolved → oriented `U` edge** for the F17 eviction key AND the `EvictedEdge`
  provenance. All VT fixtures use `Along` (oriented ≡ authored), so no VT forced the
  authored-orientation re-map; `Against` orientation is implemented but untested by a
  VT — first `Against` consumer should add coverage. STOP-condition did not trigger.
- **Intra-SCC exclusion reuses `participates(&edge, degraded[ov])`** on the authored
  node pair (orientation-independent); boundary edges enter `U` so taint crosses.
- **Levels = memoised longest-path recursion** (`level_of`), total over
  `0..node_count` (isolated → 0, no sentinel). u32 via `.saturating_add(1)` (no `as`).
- **Taint = DFS over `U` forward adjacency from spec-referenced degraded SCC seeds**;
  empty-seed short-circuit. `Degraded > Finite` from enum variant order; `(level,
  node)` from `OrderKey` field order — both derive-`Ord`, no hand-written cmp.
- **Pass-3 evictions merged then re-sorted** via the now-`pub(crate)` `sort_provenance`
  (idempotent full re-sort), so `UnionCycleVsLayer` interleaves with passes 1–2 by
  `(overlay, edge)`.

Carry-forward for PHASE-04 (reachability & channel evaluation):
- `spine_path` deferred here (design assigns P04); `configs` still NOT stored on
  `Graph` (no consumer yet — PHASE-04 `spine_path`/`evaluate` are the first; thread
  then). `node_count`/`order_spec`/`order_keys`/`degraded_sccs` now all on `Graph`.
- `reachable`/`evaluate` read the per-overlay traversal view (`out`/`incoming`),
  which is cycle-safe over Reject overlays by design (I1/F47) — do NOT route them
  through `U`.

## PHASE-04 implementation (2026-06-11) — query surface T1–T4

Storage seam + traversal half of the phase. New private `src/query.rs` (the
query-time sibling of build-time `resolve.rs`), public channel-result types, and
`reachable`/`spine_path` on `Graph`. `tests/reachability.rs` (8 tests) green;
whole suite still green (behaviour preservation — T1 only adds a field).

- **T1 storage seam:** `overlays: Vec<OverlayConfig>` re-stored on `Graph` in
  `build()` (moved `self.overlays` into the struct after the `resolve` borrow
  ends — no clone). First consumer is `spine_path`'s `AtMostOne` gate. The brief
  dead-code window (field added before first read) carried a self-clearing
  `#[expect(dead_code)]`, removed the moment `spine_path` read it (T4).
- **T2 public types:** `Channel`/`ChannelDiagnostic`/`ChannelDiagReason` declared
  FLAT in lib.rs (A-2, no `pub use`), doc-comments mirror §5.4. `Channel` carries
  NO combinator (F40 partial — spec stays caller-side). `contributors(node)`
  returns `&BTreeSet` via a `static EMPTY` fallback (no panic, no alloc).
- **T3 `mod query` + `reachable(ov,n,dir) -> BTreeSet<NodeId>`:** BFS over the
  resolved adjacency, `start` seeded into `visited` but never into `reached` →
  STRICT (I6/F8) holds even when `start` is cyclically reachable. `Along`→`out`
  dst, `Against`→`incoming` src, `None`→∅ (F25). Foreign ids→∅ (F14). Visited set
  bounds a degraded Reject cycle (F12). → VT-9.
- **T4 `spine_path(ov,n) -> Option<Vec<NodeId>>`:** `None` unless the stored
  config is `AtMostOne` (F23); else follow `single_parent` (pass-1 left ≤1
  in-edge) up `incoming`. **Orientation pinned: root → … → node** (ancestor-first)
  — chosen for natural reading order, asserted in the test. Cycle-safe (visited
  re-entry break). → spine half of VT-1.

### T5–T7 (2026-06-11) — evaluate, REQ-080 seam, refactor

`evaluate` + the REQ-080 doc-seam landed; `tests/channels.rs` (9 tests, VT-1
channel half + VT-2..8) green, whole `just check` green (exit 0). PHASE-04
complete.

- **T5 `evaluate(spec, &seeds) -> Channel`, the F34 split.** Per node over
  `0..node_count`: `reach = query::reachable(...)`; `fold_node` dispatches on
  combinator. Fold set: idempotent (`Any`/`All`/`Max`) fold `{n} ∪ reach` via
  `idempotent_members` (`once(n).chain(reach)` — `n` never in strict reach, no
  dedup); `CountDistinct` folds STRICT `reach` only. **`Direction::None` needs no
  special branch** — `reachable(_,None)=∅` collapses the idempotent fold set to
  `{n}` (own seed, F35) and the CountDistinct set to `∅` (absent, F35). One match
  arm, no per-direction conditionals.
- **Seed contract single-sourced in `vet_seeds`** (T7 target, done up front):
  splits the seed map into effective seeds + diagnostics in ONE pass.
  UnknownSeedNode checked BEFORE the variant check → wins (F41). Diagnostics come
  out NodeId-sorted for free (seeds is a `BTreeMap`). Effective = known node AND
  in-domain variant; everything else contributes nothing to any fold.
- **Absence vs identity (F16/F45):** a fold returns `None` ⇒ node absent from
  `values` (no fabricated identity escapes). `Count(0)`/`Flag(false)` only when a
  present-but-false seed exists in the fold set — `true_witnesses` returns
  `Some(∅)` (present, no trues) vs `None` (nothing present) to carry exactly that
  distinction.
- **Contributors (F21/F43):** Any→present-true witnesses; All→present-false set
  if false / present-true set if true; Max→single argmax (`value>best || (==  &&
  node<best_node)` — min-NodeId tie, iteration-order-independent); CountDistinct→
  the counted set. Only non-empty contributor sets are stored (accessor masks
  missing→empty).
- **T6 REQ-080 seam:** `Combinator` doc-marked as the one curated extension point
  — a fresh channel is `(combinator, overlay, direction)` in the caller's hand, no
  core edit; adding a *variant* is the only core-touching channel change. VT-8
  composes two distinct fresh channels (Any/Against "flagged ancestor", Max/Along
  "max priority") over one graph, no new variant.
- **T7 refactor:** extracted `true_witnesses(members, effective) -> Option<
  BTreeSet>` — the present-true gather shared verbatim by `fold_any` (idempotent
  members) and `fold_count` (strict reach), differing only in fold set and output
  projection (`Flag(!empty)` vs `Count(len)`). `fold_all`/`fold_max` stay distinct
  (two-set / scalar-argmax shapes don't share cleanly). Diagnostic precedence was
  already single-point in `vet_seeds`.

EX-1..7 all met; VT-1..9 all covered. Ready for `/audit`.

## PHASE-05 implementation (2026-06-11) — T1–T4: the explain verb

Shipped `explain(n) -> Explanation` (the role-agnostic structured account, §5.4)
plus its tests. `tests/explain.rs` (8 tests, VT-1 + VT-2) green; whole `just check`
green. **Bounded subagent scope: T1–T4 only.** T5–T8 (golden net:
permutation/determinism/naive-oracle/denylist) DEFERRED to a later subagent —
NOT started.

Load-bearing decisions:
- **`Explanation` FLAT in lib.rs** (A-2, no `pub use`), declared right after the
  `Channel` block. Accessors only, no `# Errors` (infallible). Fields
  `{node, order_key, paths: BTreeMap<OverlayId, Vec<Vec<NodeId>>>, evicted:
  Vec<EvictedEdge>}` exactly per §5.4 — no String prose, no role fields (F13).
- **`predecessor_paths` free fn in query.rs** (the heavy multi-parent walk); thin
  `Graph::explain` threads `&self.incoming`/`&self.degraded_sccs`/`&self.provenance`
  and reads `order_keys`. Mirrors the PHASE-04 free-fn + thin-wrapper seam.
- **Multi-parent DFS, NOT spine_path's single chain.** `chains_to_root` branches on
  each predecessor (`predecessors()` returns the full in-edge src list, adjacency
  order) → `Vec<Vec<NodeId>>`. `extend_chains` carries a node-last `suffix`,
  `.reverse()`s each completed chain to root→…→node (same orientation trick as
  spine_path).
- **F47 termination via `degraded_sccs`, not via the visited set.** A predecessor
  that is a degraded-post-arity-SCC member is appended as the chain ENDPOINT and
  the walk stops (never recurses into the cycle). `node` itself ∈ an SCC → `[[node]]`
  (early return in `chains_to_root`). The visited set is a *defensive* secondary
  guard for any residual cycle not SCC-keyed — finiteness does not depend on it
  (the SCC endpoints already bound a degraded Reject view). Confirmed by the
  a↔b + a→x fixture: `explain(x).paths[ov] = [[a,x]]`, `explain(a) = [[a]]`.
- **`paths` keys every overlay present in `incoming`** (A8 settled): a node that is
  a root on an overlay still appears as `[[n]]` (present, not absent). An overlay
  with zero edges anywhere is absent — no node touches it. Pinned by the
  two-overlay root fixture.
- **F26 endpoint filter in the assembler:** `evicted` = `provenance.evictions()`
  filtered to `src == n || dst == n`, **preserving the existing `(overlay, edge)`
  provenance sort** (filter is order-preserving). VT-2 fixture: n as dst (p_lo→n)
  and n as src (n→child) both present, unrelated (u→w) absent.
- **`order_key` fallback:** `order_keys.get(n)` is total over all real nodes; a
  foreign id falls back to `Finite(0)` (infallible, F14) rather than panicking.

Carry-forward for the T5–T8 subagent:
- `tests/golden_net.rs` (or split) still to author. ZERO-DEP CONTRACT BINDING —
  hand-rolled permutation loop + naive oracle, NO proptest/quickcheck. Denylist
  scan must self-exclude (A4) and resolve root via `CARGO_MANIFEST_DIR` (A3/A4).
- For byte-identical compare (T5): `Explanation` derives `PartialEq`/`Eq`/`Clone`/
  `Debug`; `Channel`/`Provenance`/`OrderKey` already do. No derive gaps found for
  the explain surface.
- `extend_chains` has 7 params (one under clippy's default ceiling) — if T8's
  refactor touches it, bundle the `(incoming, degraded_sccs, overlay)` context into
  a struct rather than adding an 8th arg.

### T5–T8 (2026-06-11) — the golden net

Closed the slice-wide verification net: `tests/golden_net.rs` (8 tests) +
`tests/denylist.rs` (3 tests). Full cordage suite 75 tests, whole `just check`
green; plain `cargo clippy -p cordage` zero warnings. ZERO new deps — the
`[dependencies]`-free Cargo.toml contract held; everything hand-rolled.

- **T5 permutation + build-twice determinism (VT-3/VT-4).** Fixtures are
  *ordinal* edge specs (`EdgeSpec{src,dst,rank,age}` indexing into a fixed minted
  node vector) so a permutation loop can shuffle insertion order while node
  identity stays stable. `permutations(n)` is **Heap's algorithm** hand-rolled
  (no `itertools`) — a self-guarding test asserts it yields exactly n! distinct
  perms (n≤5 ⇒ ≤120). `assert_permutation_invariant` builds every perm and
  byte-compares `order_key`(all)/`Provenance`/`Explanation`(all)/a fixed
  `Channel` against the first perm as reference. Build-twice (REQ-077) compares
  two distinct `Graph`s incl. Max-channel contributor traces. All public types
  already derived `PartialEq`/`Eq`/`Clone` — **no derive additions needed** (the
  T5 carry-forward worry was unfounded).
- **No public `OverlayId` ctor** → `cordage_overlay()` mints a throwaway builder
  to recover the ordinal-0 id (every single-overlay fixture's overlay). Mint
  order is fixed so this is stable.
- **T6 naive oracle (VT-3/R1/REQ-076).** Genuinely independent of the production
  recursive Tarjan: SCCs by **mutual reachability closure** (BFS `closure()` per
  node; u,v same SCC iff u→\*v ∧ v→\*u), topo by an edge-respect check on
  `ordered()`. The engine's diagnosed `provenance().cycles()` node-sets are
  mapped to ordinals and asserted equal to the oracle SCCs over a fixed family
  (diamond / 2-cycle / 3-cycle+tail / two disjoint 2-cycles / self-loop). Topo
  witness: on acyclic views every edge u→v has `pos(u)<pos(v)` in `ordered()`.
- **REQ-076 cyclic-view witness — the subtle one (was a test bug first).** On a
  Reject cycle {0,1} with surviving edge 0→2, ALL THREE nodes go `Degraded`
  (full-downstream taint — documented conservatism), so there is NO `Finite` node
  to compare against. The real no-false-topo proof: the surviving acyclic edge is
  still order-respected (0 precedes 2 in `ordered()`) because **taint sets the
  level *tag*, not the longest-path *depth*** (F33: `materialize_keys` reads
  `longest_levels(U)` for the depth, then overlays the taint tag — so degraded
  nodes keep their edge-respecting depth ordering). First test cut wrongly
  expected node 2 to stay Finite; corrected.
- **T7 denylist scan (VT-5/REQ-079/EX-4).** `tests/denylist.rs` walks the whole
  crate from `CARGO_MANIFEST_DIR` (NOT cwd) — code, docs (README), manifest,
  tests — hand-rolled recursive `collect_files` (no `walkdir`), whole-word
  case-insensitive matcher (hand-rolled `find_from` windowed search + word-byte
  boundary; no `regex`, no `as`, no indexing-slicing). A guard asserts the walk
  actually finds `src/lib.rs` (green must mean clean, not "root resolution
  broke"); a second test plants a token to prove the matcher is live; a third
  pins self-exclusion.
- **Denylist curation (A3 call, no /consult).** Listed the concrete Appendix B
  domain nouns + time/scheduling/commitment/urgency terms; INCLUDED `backlog`
  (doctrine entity noun) and reworded the one disclaiming use in lib.rs
  (`no …/backlog vocabulary` → `…/domain vocabulary`); EXCLUDED bare `product` —
  "product-neutral" is the crate's own boundary self-description, not domain
  semantics, so matching it would be a false positive. Documented in the test
  rustdoc; flagged for /audit.
- **Self-match guard doubled (A4):** skip own basename AND assemble every literal
  from string fragments at runtime, so the test file's bytes never spell a token
  contiguously. Either guard suffices; both kept against a future rename/copy.

EX-1..5 met; VT-1..5 all covered. Phase NOT flipped to completed — that's /audit's
call. Slice rollup 4/5 → ready for /audit.
