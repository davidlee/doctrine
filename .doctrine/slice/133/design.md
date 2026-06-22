# Design SL-133: Multi-dimensional priority scoring for survey/next

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-132, ADR-001, IMP-118); doc-local refs bare — OQ-1 (§6), D1 (§7),
     VT-1 (§9). -->

## 1. Design Problem

`survey` and `next` rank by `actionability → consequence desc → canonical-id asc`,
where `consequence` is a raw **inbound-reference count** (`priority::graph` pre-pass,
`BTreeMap<EntityKey, u32>`). That count is blind to *what* depends on an item: a
blocker gating five throwaway ideas outranks nothing against one gating five
high-value slices — both score "5".

IMP-118 specifies a multi-dimensional replacement that consumes the authored
`[estimate]`, `[value]`, and risk `[facet]` data plus config-driven kind weights and
tag coefficients, and propagates a *weighted* consequence along the dependency graph.
The formula currently lives only in IMP-118 prose; this slice ratifies it as durable
policy (**ADR-015**) and implements it.

## 2. Current State

- **Build pipeline** (`src/priority/graph.rs`, `build_from`): scan → **consequence
  pre-pass** (tally inbound `CONSEQUENCE_LABELS` references into `BTreeMap<…, u32>`) →
  mint nodes in `(consequence desc, id asc)` order → emit edges → `OrderSpec` →
  `builder.build()`. `NodeAttr` carries `{ kind, status, promoted, title }`.
  `PriorityGraph` carries `consequence: BTreeMap<EntityKey, u32>`.
- **Scan** (`src/catalog/scan.rs`, `ScannedEntity` + `read_facets`): already reads
  `[estimate]`/`[value]` per entity (loose `Option<EstimateFacet>`/`Option<ValueFacet>`
  fields). Does **not** read the risk `[facet]` or tags.
- **EntityFacets** (`src/facet.rs`, **leaf**): `{ estimate, value }` only; built
  separately in the *show* path (SL-132), display-only. Not yet a scan carrier.
- **Risk model** (`src/backlog.rs`, **command** tier): `RiskFacet`, `RiskLevel`,
  `RawRiskFacet`, `validate_facet`, `exposure(Option<&RiskFacet>) -> u8` (=
  `likelihood × impact`, 1..=16, else 0). Private to `backlog`.
- **Surfaces** (`surface.rs`/`render.rs`/`view.rs`): `SurveyRow.consequence: u32`,
  `ActionabilityNode.consequence: u32`, `ActionabilityBlock.consequence: u32`,
  `ReasonKind::Consequence { inbound: u32 }`. `survey` sorts by the comparator at
  `surface.rs:137`; `next` consumes cordage `order_key` (mint order is the tier-3
  fallback); `explain` emits the consequence reason.
- **Config**: `[priority]` does not exist in `doctrine.toml`. Precedent for a typed
  section: `dispatch_config.rs` (serde struct, `#[serde(default)]`, defaults via
  helper fns, unknown keys ignored).
- **Layering** (`ADR-001`, `.doctrine/adr/001/layering.toml`): `estimate`/`value`/
  `facet`/`projection` are **leaf**; `priority::graph` is **engine**; `backlog` and
  `catalog::scan` are **command**.

## 3. Forces & Constraints

- **ADR-001 (layering, no cycles) — and its BINDING tier map.** A leaf (`facet`) and
  the engine (`priority::graph`) must read risk data. The risk model lives in `backlog`
  (**command**) — a leaf/engine→command import is an upward violation. Risk types
  **must** move to a leaf (decisive, §7 D2). Because `.doctrine/adr/001/layering.toml` is
  the binding tier map consumed by `just gate` (NOT mere convention), the slice **must**
  amend it in-slice: add `risk = "leaf"`, classify the new `priority::config` (leaf), and
  update the `facet = "leaf"` entry whose comment currently reads "imports only estimate +
  value" to permit the risk import. Omitting this fails the gate (RV-121/SL-132 was caught
  on exactly this). See §7 D2.
- **Pure/impure split.** Base-score computation must be pure over `(facets, config,
  kind)`; the consequence post-pass pure over the built graph; only config load + scan
  touch disk. The base pass must run **before** mint (it feeds the mint tiebreaker), so
  it cannot depend on the assembled graph — which it doesn't (per-node only).
- **Behaviour-preservation gate.** Moving risk types out of `backlog` and adding a
  scan read must keep the existing suites green unchanged.
- **Determinism.** Scores are `f64`; mint order and the `survey` comparator rank on
  them. Ordering must be total and reproducible (no NaN, stable tiebreak).
- **Soft dependencies.** Tags (`IMP-134`/`SL-136`) are additive: scoring ships with
  the tag-coefficient seam present but fed an **empty** tag set (Σ = identity 1.0)
  until SL-136 lands tag storage (§7 D5). Risk (`SL-134` CLI) is hand-authorable today;
  the facet model already exists.
- **ADR-004 (relations outbound-only).** The consequence post-pass derives dependents
  in-memory from outbound edges already on the built graph; it stores no reverse field.

## 4. Guiding Principles

- Ride existing seams; no parallel facet parser, no parallel risk model.
- Push impurity to the edges; keep the formula a pure function of declared inputs.
- Durable policy in the ADR; tunable numbers in code.
- The score is **explainable** — every dimension is recoverable via `explain`.

## 5. Proposed Design

### 5.1 System Model

Three pure stages bracketed by impure reads — config load + scan + the per-entity
`dep_seq_for` already performed inside `build_from` (graph.rs:221):

```
load(root) ───────────────► PriorityConfig          [impure: doctrine.toml]
scan_entities(root) ──────► [ScannedEntity{ estimate, value, risk }]  [impure: disk]
        │                            (+risk read added to read_facets)
        ▼  pre-pass (pure, per-node)
  base_score(&EntityFacets, kind, &PriorityConfig) -> f64
        │   = value_dim + risk_dim
        ▼
  NodeAttr.base_score   ──► mint order (base desc, id asc)   ──► edges ──► build()
        │
        ▼  post-pass (pure, over the built PriorityGraph)
  leverage(N)    = dep_coeff · Σ_{D ∈ needs-dependents(N)} ( base(D) + leverage(D) )
                   — RECURSIVE over the acyclic `needs` backbone (reverse-topo DP)
  optionality(N) = ref_coeff · Σ_{R ∈ ref-referencers(N)} base(R)
                   — ONE-HOP over CONSEQUENCE_LABELS overlays (Reject, cyclic-safe)
  consequence(N) = leverage(N) + optionality(N)
  score(N)       = base(N) + consequence(N)
```

Two structurally-distinct consequence mechanisms (ADR-015 thesis: an *enabler* accrues
a coefficient-weighted share of the value it unlocks — the **value of optionality**):
- **`needs`-leverage — recursive.** Completing N unblocks its whole downstream cone, so
  N accrues the *decayed* intrinsic value of everything that transitively needs it. Safe
  to recurse because the `needs` overlay is the acyclic ordering backbone (§3, D8).
  `dep_coeff ∈ [0,1]` (where `0`, and any value clamped from `≤0`, explicitly DISABLES
  leverage) is a per-hop **retention** factor: above `0`, along a *single* path a dependent
  at depth k contributes `dep_coeff^k · base`, so depth alone decays. It does **not** bound
  total magnitude — a reconvergent fan reaches a node by many paths, so leverage is a finite
  **path-sum** that can exceed any single `base` and grow with breadth (accepted, §5.5).
  Finiteness is structural (a single DP sweep over a finite DAG), not a property of
  `dep_coeff`; the only unbounded risk is `f64` overflow to `∞`, fenced by `COEFF_MAX`
  (input) + `is_finite` (output, I2(b)) — not by `dep_coeff ≤ 1` (F-1/RV-132).
- **`ref`/lineage-optionality — one-hop.** Associative "this unlocks the option of that"
  — a flat single-hop share. NOT recursed: these overlays are `Reject` (cyclic-capable),
  and recursion over them has no termination guarantee. `ref_coeff ≥ 0` is a flat weight
  (no compounding, so its magnitude is unconstrained). (D9.)
- **`after`/seq — NOT a weight contributor.** Seq stays a *structural* ordering
  constraint (cordage's `OrderSpec`), realized strictly (B strictly before A, `base` the
  continuous under-signal — the strict-`<` / ULP form, no manufactured ties). Deferred as
  a weight class; escalate only on evidence the dumb constraint mis-sequences (D10, OQ-4).

`value_dim = (coefficients.value × value × kind_weight × Σ tag_coeff) / estimate_midpoint`
`risk_dim  = exposure × risk_coeff` (presence-gated by the `[facet]`; non-risk → 0)

Absent facets collapse to the identity: value absent → `value_dim = 0`; estimate
absent → `estimate_midpoint = 1.0`; risk facet absent → `risk_dim = 0`; tags absent
(always, this slice) → `Σ tag_coeff = 1.0`.

### 5.2 Interfaces & Contracts

**New leaf `src/risk.rs`** (extracted verbatim from `backlog.rs`, behaviour-preserving):

```rust
pub(crate) enum RiskLevel { Low, Medium, High, Critical }
pub(crate) struct RiskFacet { /* likelihood, impact, origin, controls */ }
pub(crate) struct RawRiskFacet { /* tolerant parse layer */ }
pub(crate) fn parse_optional(t: Option<&toml::value::Table>) -> anyhow::Result<Option<RiskFacet>>;
pub(crate) fn exposure(facet: Option<&RiskFacet>) -> u8;   // 1..=16, else 0
```

`backlog` re-uses these (command→leaf, legal); its public behaviour is unchanged.

**`EntityFacets` (`src/facet.rs`, leaf) gains risk:**

```rust
pub(crate) struct EntityFacets {
    pub estimate: Option<EstimateFacet>,
    pub value: Option<ValueFacet>,
    pub risk: Option<RiskFacet>,   // SL-133
}
```

**`ScannedEntity` (`catalog/scan.rs`) gains a risk field;** `read_facets` reads the
`[facet]` table via `risk::parse_optional` with the same per-facet isolation as
estimate/value (a malformed present facet drops to `None` + an `Error` diagnostic,
siblings intact).

**Config (`src/priority/config.rs`, new):**

```rust
#[derive(Deserialize, Default)] #[serde(default)]
pub(crate) struct PriorityConfig {
    coefficients: Coefficients,                 // { value: f64=1.0, risk: f64=2.0 }
    kind_weights: BTreeMap<String, f64>,        // lookup default 1.0
    tag_coefficients: BTreeMap<String, f64>,    // lookup default 1.0 (unused this slice)
    consequence: ConsequenceCoeffs,             // { dep_coeff, ref_coeff } — see below
}
// dep_coeff: RECURSIVE retention, clamped to [0, 1] at load (0 = explicit disable). ≤1 makes
//   each hop *along a path* a decay (dep_coeff^k at depth k); >1 would let a single path
//   amplify with depth.
//   The [0,1] DOMAIN is scoring policy (ADR-015 owns role+domain — F-4/RV-132); it does NOT
//   bound total leverage under fan-out (a path-sum, F-1/RV-132) — overflow is fenced by
//   COEFF_MAX, not the domain. ≤0 disables leverage. Default VALUE illustrative (impl owns
//   the number), e.g. 0.5.
// ref_coeff: FLAT one-hop weight, clamped finite-non-negative ≤ COEFF_MAX (no
//   compounding ⇒ magnitude unconstrained beyond the overflow guard). Default e.g. 1.0.
// (No seq coeff — seq is a structural constraint, not a weight class, D10.)
pub(crate) fn load(root: &Path) -> PriorityConfig;   // impure; missing [priority] → all defaults
```

Unknown keys ignored (no `deny_unknown`) → forward-compatible. Lookups
(`kind_weight(kind)`, `tag_coeff(tag)`) return `1.0` on absence.

**`load` policy (deliberate, advisory-config — NOT inherited from `dispatch_config`).**
`dispatch_config` *hard-errors* malformed values (e.g. test `unknown_harness_is_error`);
we choose differently because `[priority]` is advisory tuning, not a correctness gate:
- *Missing* `[priority]` / missing field → default (per `#[serde(default)]`).
- *Unknown key* → ignored (forward-compat).
- *Malformed value* (wrong type / non-finite / negative coefficient) → **clamped**, not
  fatal: NaN/±∞ → field default, negative → `0.0`, and every coefficient is bounded to a
  finite sane max (`COEFF_MAX`) at load so downstream products cannot overflow to `∞`
  (F-2, I2(a)). Silent (no telemetry) — config is advisory; OQ-1 resolved silent.

**Scoring (`priority::graph` or a sibling pure `priority::score`, engine):**

```rust
pub(crate) struct BaseScore { pub value_dim: f64, pub risk_dim: f64 }
impl BaseScore { pub fn total(&self) -> f64 { self.value_dim + self.risk_dim } }

fn base_score(f: &EntityFacets, kind: &entity::Kind, cfg: &PriorityConfig) -> BaseScore;
// pure; NaN-free by construction (§5.5). Returns the SPLIT (not a bare sum) so
// `explain` can surface value_dim / risk_dim. `kind_weight` keys on the kind's
// canonical name (entity::Kind carries a distinct Kind per backlog sub-kind —
// ISSUE_KIND/IMPROVEMENT_KIND/IDEA_KIND/… — so improvement/issue/idea resolve
// directly, no ItemKind needed).
```

**Build seam** (`graph::build_from`): `build_from` already takes `root` and already
performs impure `dep_seq_for` reads (graph.rs:221) — so it `load`s `PriorityConfig` from
that same `root` there, rather than threading a param or reaching `main.rs` (D4). This
covers **every** `build_from` caller with NO signature change: `build` (scan→build_from),
the `survey`/`next`/`explain` wrappers, AND the pre-scanned `actionability_block_from`
(surface.rs:484, the `inspect` actionability-block path), which already retains `root`
solely for `dep_seq_for` and now feeds config from the same handle. No caller is left on
undocumented default config (F-4).

**`PriorityGraph`** field changes:
- `consequence: BTreeMap<EntityKey, u32>` → the post-pass stores its two `f64`
  constituents directly: `leverage: BTreeMap<EntityKey, f64>` (recursive `needs` term)
  and `optionality: BTreeMap<EntityKey, f64>` (one-hop `ref` term), plus the final
  `score: BTreeMap<EntityKey, f64>`. `consequence = leverage + optionality` and
  `score = base + consequence` are exact sums of stored values — `explain` reads all
  three constituents directly, never recovering any term by subtraction (FP cancellation
  is inexact; published fields must be accurate — OQ-2 resolved, store not derive).
  Per-node `base_score` lives on `NodeAttr`.

`NodeAttr` gains `base_score: BaseScore` (the split — `value_dim`/`risk_dim`).
`base_score.total()` is the value the mint comparator and consequence post-pass
consume.

**Surfaces** retype `consequence: u32 → score: f64` across `SurveyRow`,
`ActionabilityNode`, `ActionabilityBlock`. `ReasonKind::Consequence { inbound: u32 }`
→ `ReasonKind::Score { base, value_dim, risk_dim, leverage, optionality, total }` (all
`f64`). `explain` surfaces the two consequence mechanisms separately (recursive
`leverage` vs one-hop `optionality`) so a large number is diagnosable — `consequence =
leverage + optionality`, `total = base + consequence`.

### 5.3 Data, State & Ownership

- **Authored** (read-only here): `[estimate]`/`[value]`/`[facet]` in entity TOMLs;
  `[priority]` in `doctrine.toml` (new — authored config, hand-edited).
- **Derived** (in-memory, rebuilt each command): `base_score` per node, and the `score`
  + `consequence` `f64` maps on `PriorityGraph` (both stored at compute time, §5.2/§5.4).
  Nothing persisted to disk. ADR-004 honoured — no reverse edges stored.
- **Ownership**: the risk leaf owns risk types; `facet` owns the aggregation;
  `priority::config` owns the config schema + load; `priority::graph` owns the build
  pipeline and the two pure scoring passes; `surface`/`render` own presentation.

### 5.4 Lifecycle, Operations & Dynamics

Revised `build_from` order (the mint-order ↔ consequence cycle is broken by moving
consequence to a **post**-pass):

1. **Scan** (caller-supplied) — entities + outbound edges + status + title +
   estimate/value/**risk**.
2. **Base pre-pass** (pure) — `base_score` per node from its own facets + config +
   kind into a `BTreeMap<EntityKey, BaseScore>`. *(Replaces the consequence pre-pass;
   runs before mint because it feeds the tiebreaker, and needs no graph.)*
3. **Mint** in `(base.total() desc via f64::total_cmp, id asc)`. *(Was `consequence
   desc`.)* The `BaseScore` is carried onto `NodeAttr` at the 3c attrs pass.
4. **Edges** — unchanged (ref/lineage overlays + dep/seq overlays).
5. **`OrderSpec` + build** — unchanged.
6. **Consequence post-pass** (pure, over the built graph; runs inside `build_from`
   after `build()`, with `ref_by_label` + `dep_overlay` still in scope) — two mechanisms:
   - **`needs`-leverage — recursive DP.** `A.needs=[B]` emits `edge(dep_overlay, B→A)`,
     so `out_edges(dep_overlay, N)` are N's dependents. Compute
     `leverage(N) = dep_coeff · Σ_{D ∈ out_edges(dep_overlay,N)} (base(D).total() + leverage(D))`
     by visiting nodes in **reverse `graph.ordered()` order** (dependents before their
     prerequisites), so each `leverage(D)` is already resolved when N is reached — a
     single O(V+E) sweep, no fixpoint. Termination rests on the `needs` backbone being
     acyclic (§3): the `dep_overlay` is `Reject`, so a genuine `needs` cycle is *diagnosed*
     (`dep_cycles()` / REQ-076), not silent. **Safety net — explicit dep-component graph
     (F-2/RV-132).** Partition nodes into components: each diagnosed `dep_overlay` cycle from
     `provenance().cycles()` (filtered to `dep_overlay`; `.nodes()` is the SCC member set) is
     one component, every other node a singleton. (`degraded_sccs` is cordage-private, but
     `provenance().cycles()` is the public Reject-cycle surface — no new accessor needed.)
     Condense: an `out_edges(dep_overlay)` edge whose endpoints share a component is
     **intra-component → contributes 0**; only edges to dependents in *other* components
     carry leverage. DP the components in reverse topological order; a component's leverage =
     `dep_coeff · Σ_{D ∈ external dependents} (base(D) + leverage(D))` — each external
     dependent counted once for the component — and **every member receives that same
     component value**. So a malformed cycle halts the DP, yields finite equal leverage for
     its members, and is surfaced as the authoring error it is (I5).
   - **`ref`-optionality — one-hop.** A referencer→target edge is `src→dst`, so N's
     referencers are `in_edges(ov, N)` over the **`CONSEQUENCE_LABELS` subset** of ref
     overlays ONLY (`reviews`/`owning_slice` bookkeeping excluded — pre-SL-133 semantics).
     `optionality(N) = ref_coeff · Σ_{R ∈ in_edges} base(R).total()`. **No recursion** —
     these overlays are `Reject` (cyclic-capable), so a one-hop sum is the only
     termination-safe read. (Multi-label double-counting across overlays between the same
     pair matches the old per-edge `+=1` tally — unchanged.)
   - `consequence(N) = leverage(N) + optionality(N)`; `score(N) = base(N).total() +
     consequence(N)`. `leverage`, `optionality`, `score` are each `is_finite`-sanitized
     before storage (I2(b)) and stored on `PriorityGraph` (§5.2).

**Sort integration:**
- `survey`: `actionability → score desc (total_cmp) → canonical-id asc`.
- `next`: ordering changes **wherever the precedence partial-order is silent** — which is
  more than "between disconnected molecules." The visible set is the *actionable frontier*,
  and an actionable item has by construction **no unsatisfied `needs`** (a pending
  prerequisite would block it), so **no `needs`-precedence relates two actionable items.**
  The only structural relation that can survive among them is `after` (soft seq —
  non-blocking, so two seq-related items can both be actionable). Precedence is therefore a
  **partial** order: it totally-orders items along a single seq *chain*, but leaves
  **incomparable** any two items with no seq path between them — separate components, AND
  sibling arms of a branch (a Y whose two arms share an upstream but have no edge to each
  other: ordered within each arm, silent between them). Wherever the order is silent, the
  differentiator is **`score` desc** (`total_cmp`), then `id`. **Algorithm (F-3/RV-132).**
  An induced-frontier topological sort over the actionable set: the precedence relation is
  the **surviving** seq edges only — the `seq_overlay` edges minus `provenance().evictions()`
  for that overlay (raw `seq_overlay` edges include ones cordage *evicted* to linearize an
  `Evict` cycle; replaying them raw would re-impose an evicted contradiction or miss a
  transitive constraint). Among nodes whose surviving seq-predecessors are all emitted, pick
  the max by `(score desc, id asc)` — a Kahn-style frontier sort with `score` as the
  ready-set priority. **Cordage's `order_key` is NOT usable for this (RV-132 F-3 raiser).**
  `OrderKey` is `(Level, NodeId)` (cordage lib.rs:353) and ranks by longest-path **level
  before** the mint/`NodeId` tiebreak, so a high-`score` item promoted to level ≥1 by a
  surviving seq predecessor sorts below *every* low-`score` level-0 item — which breaks the
  "score wherever seq is silent" contract between two seq-incomparable items. So `next` runs
  its **own** frontier sort; cordage supplies the constraint *source* (surviving seq edges =
  overlay − evictions), not the ordering. This makes `next`
  leverage-aware — a ready item that
  unblocks a large valuable cone leads, even with a modest own `base` — while the only
  thing structure overrides is genuine same-chain sequencing.
- `explain`: emits the full `Score` breakdown.

`ActionabilityView.policy_version` bumps `"priority.v2" → "priority.v3"` — the
ranking contract changed (count → weighted score).

**Config dynamics:** absent `[priority]` → all defaults → behaviour is "weighted by
value/risk with unit kind weights". An operator tunes coefficients without code
change. Unknown keys tolerated (forward-compat with future dimensions, e.g. a
maintainability coefficient defaulting to 0.0).

### 5.5 Invariants, Assumptions & Edge Cases

- **I1 — total order.** All score comparisons use `f64::total_cmp`; final tiebreak is
  canonical-id asc. Equal scores ⇒ deterministic id order.
- **I2 — NaN/∞-free, by construction at BOTH ends.** (a) *Inputs*: `value`/`exposure`
  are bounded ints; every config coefficient is clamped finite-non-negative AND bounded to
  `COEFF_MAX` at load (§5.2); `estimate_midpoint` is guarded `max(ε, mid)`. (b) *Outputs*:
  clamped inputs alone do NOT suffice — a product of finite-but-large coeffs can still
  overflow to `+∞`, and a downstream `∞ − ∞` would mint a `NaN`. So `base_score`
  **sanitizes every computed dimension and total** (`value_dim`, `risk_dim`, `total()`)
  and the consequence post-pass sanitizes `consequence`/`score` with `is_finite` before
  storage/comparison (non-finite → `0.0`). ⇒ no `∞`/`NaN` can reach the mint comparator,
  the `survey` sort, or `explain`. (VT-6.)
- **I3 — consequence excluded from *mint*, not from *display*.** The distinction is
  mint-time vs display-time. **Mint** order (cordage's tier-3 structural fallback, which
  feeds graph construction) uses `base` ONLY — a graph-derived quantity in the structural
  tiebreak would couple ordering to the very edges it orders (feedback loop), and `score`
  isn't even available at mint time (it's a post-build pass). **Display** order is a
  pure post-hoc sort over the already-built graph: `survey` and `next` both rank by
  `score`. No feedback — re-sorting built output by a derived quantity introduces no
  cycle. (This is why `next` can be score-aware between molecules without violating the
  no-feedback rule: the score sort happens *after* build, the mint/structural order does
  not see it.)
- **I4 — additive identity for absent facets.** A node with no authored facets scores
  `base = 0` and contributes `0` to dependents' consequence — exactly the pre-SL-133
  "unweighted" floor, so unauthored corpora degrade gracefully.
- **I5 — leverage recursion terminates, always.** The recursive `needs`-leverage DP runs
  over the `dep_overlay`, the acyclic ordering backbone. Reverse-`ordered()` traversal
  guarantees dependents resolve first (single sweep, no fixpoint). A *diagnosed* `needs`
  cycle (malformed data; `Reject` surfaces it) is condensed — intra-SCC edges contribute
  0 — so the DP halts and the cycle is reported, never chased. Finiteness is **structural**
  — a single DP sweep over a finite DAG is always finite — *not* a convergence property of
  `dep_coeff`: under fan-out leverage is a path-sum that `dep_coeff ≤ 1` decays per-path but
  does not bound (F-1/RV-132). `>1` is clamped at load to keep each hop a *retention*
  (per-path decay), the recursive-vs-flat policy domain (ADR-015); overflow to `∞` is fenced
  by `COEFF_MAX` + `is_finite` (I2(b)), not by the domain. The one-hop
  `ref`-optionality term needs no termination argument (it never recurses). (VT-4, VT-8.)
- **Reconvergence is multiplicative — accepted as leverage.** In a `needs` diamond
  (N gated-by {B, C}, both gated-by D), D's base reaches N through *both* paths, so D is
  counted twice in `leverage(N)`. This is the honest meaning of "total value of the
  downstream cone" — a node fronting a wide reconvergent fan genuinely has more leverage.
  Accepted, not guarded; documented in ADR-015 and `explain` (the `leverage` term is
  visible). (If it ever surprises, a path-dedup variant is the escalation — not this slice.)
- **Edge: non-finite / negative / huge authored coeff.** `load` clamps each coefficient
  finite-non-negative and ≤ `COEFF_MAX` (NaN/∞ → default; negative → 0.0; over-max → max)
  so products stay finite and a typo can't invert or overflow ordering. Silent (config is
  advisory) — OQ-1 resolved silent; §5.2 owns the full load policy.
- **Edge: part-assessed risk.** `exposure` already returns 0 unless *both* axes
  present — assessment is all-or-nothing (existing contract, preserved).
- **Edge: estimate midpoint of 0.** Cannot occur (a valid estimate has positive
  bounds); the `max(ε, mid)` guard is belt-and-braces.
- **Edge: dangling/free-text dep target.** Contributes no edge (existing resolve-only
  discipline) ⇒ no phantom consequence.

## 6. Open Questions & Unknowns

- **OQ-1 — clamp telemetry. RESOLVED (silent).** `load` clamps silently — `[priority]`
  is advisory tuning, not a correctness gate. This is a *deliberate new policy*, NOT
  inherited from `dispatch_config` (which hard-errors malformed values); the full load
  contract is specified in §5.2. `explain` already exposes the live dimensions, so a
  surprising rank is diagnosable without clamp logging.
- **OQ-2 — store vs derive consequence. RESOLVED (store).** `PriorityGraph` stores
  `consequence: BTreeMap<…, f64>` from the post-pass directly (the weighted Σ exists
  exactly pre-summation). `score − base` is rejected: it is floating-point cancellation,
  not exact in general, and `explain`'s published `consequence` field must be accurate.
  §5.2 / §5.4 step 6 updated accordingly.
- **OQ-3 — follow-up: collapse the two facet parse paths.** SL-132 left scan
  (`read_facets`) and the show path (`SliceDoc` serde) parsing the same facets twice.
  Unifying `ScannedEntity` onto a single `EntityFacets` carrier is a cohesion cleanup
  out of scope here — capture as a backlog improvement.
- **OQ-4 — seq as a weight class (deferred, evidence-gated).** Seq ships as a structural
  constraint (D10). IF the dumb constraint mis-sequences in practice, re-introduce `after`
  as a diminished, **rank-modulated** optionality weight (coefficient `< dep_coeff`, with
  the edge `rank` scaling the share). Acyclic-by-eviction (`Evict`), so it *could* recurse
  — but start one-hop. Not this slice; revisit on evidence.
- **OQ-5 — `next` fully score-driven / true-graph view (forward).** This slice makes
  `next` score-aware *between molecules* (§5.4). A larger question — whether the whole
  actionability ordering becomes score-primary with structure as a constraint layer, and
  how the web view renders score on one axis of a true graph — is downstream. The design
  keeps score exposed as first-class node data so that view is unblocked.

## 7. Decisions, Rationale & Alternatives

- **D1 — Two-pass: base pre-pass (pure, per-node) + consequence post-pass (pure, over
  built graph).** The old consequence pre-pass tallied a count with no graph. The new
  consequence needs each dependent's *base*, which needs the graph built — so it moves
  after `build()`. Base moves to a per-node pre-pass that feeds mint. Alt: single pass
  with a fixpoint — rejected (consequence-excluded-from-structure, I3, makes a fixpoint
  unnecessary and the two passes are strictly ordered, no iteration).
- **D2 — Extract risk types to a leaf `src/risk.rs` (forced by ADR-001).** The risk
  model is `backlog`-private (command tier); a leaf (`facet`) and engine
  (`priority::graph`) must read it. Importing upward violates layering. Mirrors the
  estimate/value leaf precedent (SL-103). Alt: expose `backlog::parse_risk` — rejected,
  upward dependency. Alt: re-parse risk inline in scan — rejected, parallel
  implementation of the validator. **Binding tier-map edits are part of this slice**
  (`.doctrine/adr/001/layering.toml`, consumed by `just gate`): add `risk = "leaf"`;
  classify `priority::config = "leaf"` (pure serde struct + a `std::fs` `load`, no
  internal module deps — mirrors `fsutil`/`facet_write`, leaves that perform IO); and
  relax the `facet` entry comment ("imports only estimate + value") to permit the risk
  import. Without these `just gate` fails (§3, the F-1 forcing function).
- **D3 — `EntityFacets` is the pure base-score input (carry risk on it now; defer
  unifying the parse paths).** Satisfies the scope's "build_priority_graph consumes
  EntityFacets" intent without disturbing SL-132's show path (behaviour-preservation).
  Collapsing the two parse paths is OQ-3, a separate cleanup. Alt: loose fields only —
  loses the shared projection; Alt: unify now — reworks done code, bigger blast radius.
- **D4 — Load config inside `build_from`, not `main.rs`.** `build_from` already takes
  `root` and already performs impure `dep_seq_for` reads (graph.rs:221) — so it `load`s
  `[priority]` from that same `root`. More cohesive than threading a `PriorityConfig`
  param and, crucially, covers **every** `build_from` caller with no signature change —
  including the pre-scanned `actionability_block_from` (surface.rs:484), which would
  otherwise miss a threaded param (F-4). `survey`/`next`/`explain` keep their `(root)`
  signatures. (Deviates from the scope's "main.rs parses config" — the build seam is more
  cohesive.) Alt: thread `PriorityConfig` from `main` through every surface fn — more
  plumbing, easy to miss a caller. Alt: a separate `config: &PriorityConfig` param on
  `build_from` — same miss-a-caller risk (F-4).
- **D5 — Tag-coeff seam present but fed empty (Σ = 1.0) this slice.** Honours the soft
  `after IMP-134`: the formula carries the tag term from day one but reads no tags
  until SL-136 lands tag storage. Avoids coding scan against SL-136's unratified
  storage shape. Lighting it up later is a localized scan read, not a redesign.
- **D6 — `f64::total_cmp` for every score comparison; NaN/∞-free by construction
  (I2).** Total order + clamped finite inputs **+ `is_finite` sanitization of every
  computed dimension / total / consequence** (not inputs alone — finite inputs can still
  overflow a product to `∞`; I2(b), F-2). Alt: `partial_cmp().unwrap_or(Equal)` —
  rejected, hides a NaN bug as a silent tie.
- **D7 — `ReasonKind::Score { base, value_dim, risk_dim, leverage, optionality, total }`
  replaces `Consequence { inbound }`.** `explain` is the transparency surface; the raw
  inbound count is no longer the ranking quantity, so it is replaced by the full
  breakdown — including the two consequence mechanisms split out (`leverage` vs
  `optionality`) so a large number is attributable. Render contract: **`survey`** adds a
  `score` column (`SurveyRow` retyped `consequence: u32 → score: f64`). **`next` now also
  gains a `score` column** — it is no longer column-less (reversing the earlier F-8 call):
  because `next` is now *ordered by* score between incomparable items (§5.4), the ranking
  quantity must be visible on the row, so `NextRow` gains `score: f64` and `NEXT_COLS` a
  column. (`explain` still carries the full reason breakdown.)

- **D8 — `needs`-leverage is RECURSIVE over the acyclic backbone.** Consequence's core
  is the transitive value a node unlocks, not a one-hop count — a deep blocker gating a
  cheap chore that gates ten valuable slices *is* highly consequential. Computable as a
  single-sweep reverse-topological DP precisely because the `needs` overlay is the
  acyclic ordering backbone (a real cycle is a diagnosed authoring error, condensed as a
  safety net). Alt: one-hop sum-of-base (original IMP-118 prose) — rejected, blind to
  downstream leverage past one hop. Alt: full-graph fixpoint — unnecessary given
  acyclicity, and unstable on cycles.
- **D9 — `ref`/lineage-optionality is ONE-HOP, deliberately.** Lineage overlays are
  `Reject` (cyclic-capable: `related` loops, lineage diamonds), so recursion has no
  termination guarantee. Semantically lineage is associative ("unlocks the option of"),
  for which a flat single-hop share is the right model anyway. So the two consequence
  classes differ by *structure*: `needs` recurses (acyclic + causal), `ref` doesn't
  (cyclic + associative). This is the clean resolution of the cycle problem — it's a
  constraint on *which edges accumulate weight recursively*, not a wrinkle in the math.
- **D10 — `after`/seq is a STRUCTURAL constraint, not a weight class.** Seq stays in
  cordage's `OrderSpec`, enforced strictly (B `<` A, not `≤` — the ULP/`next_down` form),
  so it sequences without manufacturing ties and needs no weight aggregation. Modelling
  seq as a score *clamp* was the tempting-but-wrong path (non-strict ⇒ ties); strict
  structural precedence (which cordage already realizes) is both simpler and tie-free.
  Escalation path if the dumb constraint ever mis-sequences: re-introduce seq as a
  *diminished, rank-modulated optionality weight* (lower than `needs`) — deferred, evidence-
  gated (OQ-4). Coefficients are asymmetric by structure: `dep_coeff ∈ [0,1]` (recursive
  retention — per-path decay, though fan-out still sums, F-1), `ref_coeff` flat-non-negative
  (one-hop, no compounding).

**ADR-015 boundary** — opens with the **thesis**: an *enabler* accrues a
coefficient-weighted share of the value it unlocks (the value of optionality); `score`
is a reusable ordering primitive exposed as first-class node data (forward: a true-graph
web actionability view orders one axis by it). Ratifies durable policy: dimension
semantics; the two-pass model; **consequence = recursive `needs`-leverage (D8) +
one-hop `ref`-optionality (D9)**; seq-as-structural-constraint (D10); the
mint-vs-display ordering rule (I3); the `[priority]` config shape + forward-compat rule;
the sort contract (survey/next/explain); **and the coefficient role/domain split —
`dep_coeff` a recursive retention factor in `[0,1]`, `ref_coeff` a flat non-negative one-hop
weight, seq no weight class — because the domains (not just the values) encode the
recursive-vs-one-hop policy that makes D8/D9 valid (F-4/RV-132).** Implementation-owned (not
in the ADR, tunable freely): the coefficient *default numbers* (e.g. `dep_coeff = 0.5` — the
value, never the `[0,1]` domain), kind-weight defaults, tag-coeff examples, `COEFF_MAX`, and
the `total_cmp` / silent-clamp / condensation mechanics.

## 8. Risks & Mitigations

- **R1 — risk extraction breaks backlog suites.** Mitigation: pure move + re-export,
  behaviour-preserving; the existing backlog risk/exposure tests are the proof and stay
  green unchanged (behaviour-preservation gate).
- **R2 — golden/snapshot churn.** `survey`/`next`/`explain` output changes shape
  (score column, Score reason). Mitigation: update goldens deliberately in the surface
  phase; assert the *new* contract, not the old count.
- **R3 — ordering regressions invisible to unit tests.** Mitigation: scenario tests
  with hand-computed scores (small fixtures: one high-value gating slice vs one gating
  ideas) asserting the *reordering* the slice exists to produce (VT-5).
- **R4 — config silently mis-tunes ordering.** Mitigation: clamp + defaults (I2);
  `explain` exposes the live dimensions so a surprising rank is diagnosable.

## 9. Quality Engineering & Validation

Phasing (provisional, for `/plan`):
- **P1 — risk leaf extraction.** Move risk types to `src/risk.rs`; `backlog`
  re-uses; `EntityFacets` gains `risk`. Behaviour-preserving.
- **P2 — scan + config.** `read_facets` reads `[facet]`; `priority::config` +
  `load`; thread into `build`/`build_from`.
- **P3 — scoring passes.** `base_score` pre-pass + `NodeAttr.base_score` + mint
  retie; consequence post-pass — recursive `needs`-leverage DP (reverse-`ordered()`,
  SCC-condensed) + one-hop `ref`-optionality; `leverage`/`optionality`/`score` maps.
- **P4 — surfaces.** Retype `consequence → score`; `Score` reason (leverage/optionality
  split); `survey` + `next` score columns; `next` frontier sort `(score, id)` + seq
  precedence; goldens.
- **P5 — ADR-015 + `doctrine.toml` `[priority]` seed.**

Verification (criteria firm up in `/plan`):
- **VT-1** — `risk::exposure` parity: the extracted leaf reproduces the former
  `backlog` results (existing tests pass post-move, unchanged).
- **VT-1b** — scan-seam per-facet isolation preserved (F-7): existing catalog/scan
  malformed-facet cases stay green unchanged, **plus** a new case proving a malformed
  `[facet]` (risk) drops only `risk` to `None` + an `Error` diagnostic while sibling
  `estimate`/`value` survive intact — the contract the new `read_facets` risk read must
  preserve.
- **VT-2** — `base_score` is pure & correct: value-only, risk-only, both, neither;
  absent estimate → midpoint 1.0; kind_weight/tag_coeff defaults applied.
- **VT-3** — config: missing `[priority]` → all defaults; partial section → per-field
  defaults; unknown key ignored; non-finite/negative/over-`COEFF_MAX` coeff clamped
  (I2(a)); a malformed *value* clamps and does NOT hard-error — the deliberate
  advisory-config policy (§5.2), distinct from `dispatch_config` (F-6).
- **VT-4** — consequence post-pass **directions & classes** (F1/F2 + D8/D9): (a)
  `needs`-leverage flows `out_edges(dep_overlay)` (prereq accrues dependents); (b)
  `ref`-optionality flows `in_edges` over `CONSEQUENCE_LABELS` overlays only, one-hop;
  (c) a `reviews`/`owning_slice` edge contributes **0** (subset exclusion); (d) a dangling
  target contributes 0; ADR-004 (no stored reverse) upheld.
- **VT-4b** — **leverage is recursive (D8)**: a 3-deep `needs` chain A←B←C (C needs B
  needs A) gives A `leverage = dep_coeff·(base(B)+leverage(B)) = dep_coeff·base(B) +
  dep_coeff²·base(C)` — i.e. depth-k decay `dep_coeff^k`; a reconvergent diamond
  double-counts the shared leaf through both paths (accepted). Contrast: `ref` is one-hop
  (no transitive accumulation).
- **VT-5** — **reordering scenario** (the point of the slice): a blocker gating one
  high-value slice outranks a blocker gating five ideas, where the old inbound-count
  ranked them opposite; AND a deep blocker gating a cheap chore that gates a valuable cone
  outranks a shallow blocker fronting one modest item (recursive-leverage proof).
- **VT-6** — determinism + finite outputs: equal scores tiebreak canonical-id asc; AND
  feeding near-`f64::MAX` coefficients proves no `∞`/`NaN` reaches mint, the `survey`
  sort, or `explain` — i.e. `base_score` and the post-pass `is_finite`-sanitize the
  computed dims/total/leverage/optionality, not just the inputs (I2(b), F-2).
- **VT-7** — ordering split (I3): **mint** uses `base` only (consequence excluded from the
  structural tier-3 fallback); **`survey`** display sorts by `score`; **`next`** sorts the
  actionable frontier by `(score desc, id)` with surviving `after`-seq applied as strict
  precedence — proven by: a Y-fixture (two seq-incomparable ready arms order by score), a
  same-chain seq pair (keeps structural order regardless of score), AND an evicted/cyclic
  seq case (an `Evict`-broken seq edge does NOT re-impose precedence — proving the sort reads
  *surviving* edges, not raw `seq_overlay`; F-3).
- **VT-8** — **leverage terminates on malformed data, condensation specified (I5, F-2)**:
  (a) a self-loop (`A needs A`) — the singleton-vs-cycle boundary — yields finite
  `leverage(A)`; (b) a multi-member SCC `A↔B` with an external dependent `C` (`C needs B`):
  the `{A,B}` component is read from `provenance().cycles()`, intra-component edges
  contribute 0, `base(C)+leverage(C)` flows to the component **once**, and `A` and `B` report
  the same finite component leverage. The DP halts and every node's leverage is finite.
- **VA-1** — `explain --json` exposes `{ base, value_dim, risk_dim, leverage,
  optionality, total }`; human render reads correctly.
- Goldens (`survey`/`next`/`explain` human + `--json`) updated to the score contract.

## 10. Review Notes

**Internal adversarial pass (2026-06-21).** Verified two correctness-critical facts
against source before locking, then found two bugs in the first draft:

- **Verified:** cordage exposes both `out_edges`/`in_edges`
  (`crates/cordage/src/lib.rs:768,783`); backlog sub-kinds are distinct `entity::Kind`
  rows (`ISSUE_KIND`/`IMPROVEMENT_KIND`/`IDEA_KIND`/…), so config kind-weights resolve
  without an `ItemKind` (worry dissolved).
- **F1 (fixed) — dep-class edge direction.** First draft walked `dep_overlay`
  *in_edges*; correct is **`out_edges`** (the `needs` B→A flip puts the prereq on the
  edge source). §5.2/§5.4/VT-4 corrected.
- **F2 (fixed) — ref-class label set.** First draft used all `REF_LABELS`,
  re-including `reviews`/`owning_slice`; restored to the **`CONSEQUENCE_LABELS`**
  subset to preserve pre-SL-133 consequence semantics. §5.2/§5.4/VT-4 corrected.
- **F3 (fixed) — `base_score` returns `BaseScore { value_dim, risk_dim }`** (split,
  not bare sum) so `explain` can surface dimensions.
- **F4/F6 (fixed)** — base computed into a map pre-mint then carried onto `NodeAttr`;
  `policy_version` bumps `v2→v3`.

Open after the pass: OQ-1 (clamp telemetry), OQ-2 (store vs derive consequence),
OQ-3 (parse-path unification follow-up). No governance conflict surfaced (ADR-001
layering *drives* D2; ADR-004 upheld; ADR-015 authored this phase).

**External inquisition RV-130 (2026-06-21, codex/GPT-5.5).** 8 findings (1 blocker,
3 major, 4 minor) against this design; the §10 internal pass was treated as the
accused's own alibi. The clean spine (edge directions, layering *direction*, ADR-004,
no parallel validator) survived. All 8 reconciled here, all `design-wrong` (no code
yet — the artifact was the defect):
- **F-1 (blocker)** — binding tier-map (`layering.toml`) edits made in-slice: §3, D2,
  Terrain. `risk = "leaf"`, `priority::config = "leaf"`, `facet` comment relaxed.
- **F-2 (major)** — I2 made true at *both* ends: outputs `is_finite`-sanitized +
  `COEFF_MAX` input bound. §5.2, §5.5 I2/edge, D6, VT-6.
- **F-3 (major)** — OQ-2 closed by **storing** `consequence: f64` (not `score − base`).
  §5.2, §5.4 step 6, §6 OQ-2.
- **F-4 (major)** — every `build_from` caller covered by loading config inside
  `build_from` from `root`; `actionability_block_from` (surface.rs:484) named. §5.2, D4.
- **F-5 (minor)** — §5.1 impurity boundary now counts `dep_seq_for`.
- **F-6 (minor)** — clamp owned as deliberate advisory-config policy; `dispatch_config`
  miscitation dropped. §5.2, §6 OQ-1, VT-3.
- **F-7 (minor)** — scan-seam isolation pinned by VT-1b.
- **F-8 (minor)** — D7 render contract reconciled with view types (`next` has no score
  column; reason line only). **SUPERSEDED below** — the consequence-model revision makes
  `next` score-ordered, so `next` now *does* carry a score column.

**Consequence-model revision (2026-06-21, design dialogue — POST-RV-130).** RV-130 and
the internal pass above reviewed a **one-hop** consequence (flat sum of direct
dependents' base, symmetric coeffs). A subsequent design conversation replaced it; the
mechanics below are NOT yet externally reviewed:
- **Recursive `needs`-leverage + one-hop `ref`-optionality (D8/D9).** Consequence splits
  into a transitive, depth-decayed leverage term over the acyclic `needs` backbone, and a
  flat one-hop optionality term over the cyclic-capable lineage overlays. Resolves the
  cycle question as "which edges may accumulate *recursively*" (only the acyclic backbone),
  not a calculation wrinkle. Coefficients become asymmetric: `dep_coeff ∈ [0,1]` retention,
  `ref_coeff` flat.
- **Seq stays a structural constraint (D10).** The score-clamp temptation is tie-prone
  only when non-strict; strict (`<`/ULP) precedence — which cordage's `OrderSpec` already
  realizes — sequences without ties, so seq needs no weight class. Escalation deferred
  (OQ-4).
- **Ordering is mint-vs-display (I3 refined).** Consequence excluded from *mint* (feedback
  + not-yet-computed), but `survey`/`next` *display* sorts use `score`. `next` orders the
  actionable frontier by `score` wherever the precedence partial-order is silent —
  including sibling Y-arms within one component, not just disconnected molecules (VT-7).
- **Forward.** Score is exposed as first-class node data for an eventual true-graph web
  actionability view (OQ-5).

**Re-review needed:** the recursive DP (termination/condensation, depth decay,
reconvergence), the asymmetric coefficient domains, and the molecule/Y ordering of `next`
postdate RV-130 — they want a fresh external adversarial pass before planning.
