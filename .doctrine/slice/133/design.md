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

- **ADR-001 (layering, no cycles).** A leaf (`facet`) and the engine
  (`priority::graph`) must read risk data. The risk model lives in `backlog`
  (**command**) — a leaf/engine→command import is an upward violation. Risk types
  **must** move to a leaf (decisive, §7 D2).
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

Three pure stages bracketed by two impure reads (config load, scan):

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
  consequence(node) = Σ base(dep).total() × edge_coeff
        ref-class: in_edges over CONSEQUENCE_LABELS overlays × ref_edge_coeff
        dep-class: out_edges(dep_overlay)  (needs B→A flip) × dep_edge_coeff
  score(node)       = base(node).total() + consequence(node)
```

`value_dim = (value × kind_weight × Σ tag_coeff) / estimate_midpoint`
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
    consequence: ConsequenceCoeffs,             // { ref_edge_coeff: f64=1.0, dep_edge_coeff: f64=2.0 }
}
pub(crate) fn load(root: &Path) -> PriorityConfig;   // impure; missing [priority] → all defaults
```

Unknown keys ignored (no `deny_unknown`) → forward-compatible. Lookups
(`kind_weight(kind)`, `tag_coeff(tag)`) return `1.0` on absence.

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

**Build seam** (`graph::build`): now `load`s config and threads it; `build_from`
gains `config: &PriorityConfig`. Surface fns (`survey`/`next`/`explain`) keep their
`(root)` signatures — config is loaded inside the wrapper alongside the scan.

**`PriorityGraph`** field changes:
- `consequence: BTreeMap<EntityKey, u32>` → `score: BTreeMap<EntityKey, f64>` (the
  post-pass result `base + consequence`), plus the per-node `base_score` lives on
  `NodeAttr`. The raw weighted `consequence` is recoverable as `score − base` (kept as
  a derived value for `explain`, not a stored third map — but see OQ-2).

`NodeAttr` gains `base_score: BaseScore` (the split — `value_dim`/`risk_dim`).
`base_score.total()` is the value the mint comparator and consequence post-pass
consume.

**Surfaces** retype `consequence: u32 → score: f64` across `SurveyRow`,
`ActionabilityNode`, `ActionabilityBlock`. `ReasonKind::Consequence { inbound: u32 }`
→ `ReasonKind::Score { base, value_dim, risk_dim, consequence, total }` (all `f64`).

### 5.3 Data, State & Ownership

- **Authored** (read-only here): `[estimate]`/`[value]`/`[facet]` in entity TOMLs;
  `[priority]` in `doctrine.toml` (new — authored config, hand-edited).
- **Derived** (in-memory, rebuilt each command): `base_score` per node, `score` map,
  consequence. Nothing persisted. ADR-004 honoured — no reverse edges stored.
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
   after `build()`, with `ref_by_label` + `dep_overlay` still in scope) — for each
   node N, sum its dependents' `base_score.total()`:
   - **ref class**: a referencer→target edge is emitted `src→dst`, so N's dependents
     are the **`in_edges(ov, N)`** sources — iterated over the **`CONSEQUENCE_LABELS`
     subset** of ref overlays ONLY (work/lineage; `reviews`/`owning_slice` bookkeeping
     stay excluded, preserving the pre-SL-133 consequence semantics). Weighted
     `× ref_edge_coeff`.
   - **dep class**: `A.needs=[B]` emits `edge(dep_overlay, B→A)` — the prereq B is the
     edge **source**, so B's dependents are the **`out_edges(dep_overlay, B)`** targets
     (NOT in_edges). Weighted `× dep_edge_coeff`.
   - `score(N) = base(N).total() + Σ`. Store in `PriorityGraph.score`. (Multi-label
     double-counting across overlays matches the old per-edge `+=1` tally — unchanged.)

**Sort integration:**
- `survey`: `actionability → score desc (total_cmp) → canonical-id asc`.
- `next`: cordage `order_key`; the within-level tiebreaker is the **mint order**, now
  `(base desc, id asc)`. Consequence is **deliberately excluded** from structural
  ordering — including a graph-derived quantity in the structural tiebreak would
  couple ordering to the very edges it orders (feedback loop). Mint uses `base` only.
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
- **I2 — NaN-free.** Inputs are finite: `value`/`exposure` are bounded ints;
  `kind_weight`/`tag_coeff`/edge coeffs are finite config (a non-finite authored coeff
  is rejected/clamped at load — see edge cases). `estimate_midpoint` is guarded
  `max(ε, mid)` so the division never produces ∞/NaN. ⇒ no NaN reaches a comparator.
- **I3 — consequence excluded from structure.** Mint and `next` order on `base` only;
  consequence influences only the `survey` *display* sort and `explain`. (Prevents the
  feedback loop; keeps the structural order a pure function of authored dep/seq.)
- **I4 — additive identity for absent facets.** A node with no authored facets scores
  `base = 0` and contributes `0` to dependents' consequence — exactly the pre-SL-133
  "unweighted" floor, so unauthored corpora degrade gracefully.
- **Edge: non-finite / negative authored coeff.** `load` clamps each coefficient to a
  finite non-negative `f64` (NaN/∞ → default; negative → 0.0) so I2 holds and a typo
  can't invert ordering. Logged? — no; silent clamp (config is advisory). *(OQ-1.)*
- **Edge: part-assessed risk.** `exposure` already returns 0 unless *both* axes
  present — assessment is all-or-nothing (existing contract, preserved).
- **Edge: estimate midpoint of 0.** Cannot occur (a valid estimate has positive
  bounds); the `max(ε, mid)` guard is belt-and-braces.
- **Edge: dangling/free-text dep target.** Contributes no edge (existing resolve-only
  discipline) ⇒ no phantom consequence.

## 6. Open Questions & Unknowns

- **OQ-1 — clamp telemetry.** Should `load` warn on a clamped/garbage coefficient, or
  clamp silently? Leaning silent (config is advisory, matches `dispatch_config`'s
  tolerant parse). Revisit if it bites.
- **OQ-2 — store consequence separately or derive `score − base`?** Storing only the
  final `score` map keeps state minimal; `explain` recomputes the consequence term as
  `score − base`. Acceptable (both are exact `f64` from the same pass) but slightly
  implicit. Alt: store a second `consequence: BTreeMap<…, f64>`. Leaning derive.
- **OQ-3 — follow-up: collapse the two facet parse paths.** SL-132 left scan
  (`read_facets`) and the show path (`SliceDoc` serde) parsing the same facets twice.
  Unifying `ScannedEntity` onto a single `EntityFacets` carrier is a cohesion cleanup
  out of scope here — capture as a backlog improvement.

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
  implementation of the validator.
- **D3 — `EntityFacets` is the pure base-score input (carry risk on it now; defer
  unifying the parse paths).** Satisfies the scope's "build_priority_graph consumes
  EntityFacets" intent without disturbing SL-132's show path (behaviour-preservation).
  Collapsing the two parse paths is OQ-3, a separate cleanup. Alt: loose fields only —
  loses the shared projection; Alt: unify now — reworks done code, bigger blast radius.
- **D4 — Load config at the `graph::build` seam, not `main.rs`.** `build` already owns
  the impure touches (scan, `dep_seq_for`); reading `[priority]` there keeps one impure
  entry and leaves `survey`/`next`/`explain` on their `(root)` signatures. (Deviates
  from the scope's "main.rs parses config" — the build seam is more cohesive.) Alt:
  thread a `PriorityConfig` from `main` through every surface fn — more plumbing, no
  benefit.
- **D5 — Tag-coeff seam present but fed empty (Σ = 1.0) this slice.** Honours the soft
  `after IMP-134`: the formula carries the tag term from day one but reads no tags
  until SL-136 lands tag storage. Avoids coding scan against SL-136's unratified
  storage shape. Lighting it up later is a localized scan read, not a redesign.
- **D6 — `f64::total_cmp` for every score comparison; NaN-free by construction
  (I2).** Total order + clamped finite inputs. Alt: `partial_cmp().unwrap_or(Equal)` —
  rejected, hides a NaN bug as a silent tie.
- **D7 — `ReasonKind::Score { base, value_dim, risk_dim, consequence, total }`
  replaces `Consequence { inbound }`.** `explain` is the transparency surface; the raw
  inbound count is no longer the ranking quantity, so it is replaced by the dimensional
  breakdown. `survey`/`next` rows show only the single `score` column.

**ADR-015 boundary** — ratifies durable policy: dimension semantics, the two-pass
model, the `[priority]` config shape + forward-compat rule, and the sort contract
(survey/next/explain, incl. consequence-excluded-from-structure). Implementation-owned
(not in the ADR, tunable freely): the coefficient numbers, kind-weight defaults,
tag-coeff examples, and the `total_cmp`/ε/clamp mechanics.

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
  retie; consequence post-pass + `score` map.
- **P4 — surfaces.** Retype `consequence → score`; `Score` reason; render columns;
  goldens.
- **P5 — ADR-015 + `doctrine.toml` `[priority]` seed.**

Verification (criteria firm up in `/plan`):
- **VT-1** — `risk::exposure` parity: the extracted leaf reproduces the former
  `backlog` results (existing tests pass post-move, unchanged).
- **VT-2** — `base_score` is pure & correct: value-only, risk-only, both, neither;
  absent estimate → midpoint 1.0; kind_weight/tag_coeff defaults applied.
- **VT-3** — config: missing `[priority]` → all defaults; partial section → per-field
  defaults; unknown key ignored; non-finite/negative coeff clamped (I2).
- **VT-4** — consequence post-pass **directions** (the F1/F2 fixes): a `needs`
  dependent's base flows to its prerequisite via `out_edges(dep_overlay)` (× dep coeff);
  a referencer's base flows to its target via `in_edges` over the `CONSEQUENCE_LABELS`
  overlays only (× ref coeff); a `reviews`/`owning_slice` edge contributes **0** (subset
  exclusion); a dangling target contributes 0; ADR-004 (no stored reverse) upheld.
- **VT-5** — **reordering scenario** (the point of the slice): a blocker gating one
  high-value slice outranks a blocker gating five ideas, where the old inbound-count
  ranked them opposite.
- **VT-6** — determinism: equal scores tiebreak canonical-id asc; no NaN reaches a
  comparator (property/targeted test over the guards).
- **VT-7** — `next` structural order ignores consequence (mint = base only); `survey`
  display sort uses score.
- **VA-1** — `explain --json` exposes `{ base, value_dim, risk_dim, consequence,
  total }`; human render reads correctly.
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
