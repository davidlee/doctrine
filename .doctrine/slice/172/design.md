# Design SL-172: Priority cost skew and bare-estimate anchor

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

`value_dim = (coeff.value × value × kind_weight × tag_term) / est_cost`
(ADR-015 §1). Two defects in `est_cost`:

1. **Bare-item inversion (ISS-057).** A value-bearing item with no estimate is
   priced at `est_cost = 1.0` — the cheapest possible cost — so it outranks items
   carrying honest estimates. The estimator is penalised. Observed on IMP-120.
2. **Midpoint mis-prices every estimated item.** Software estimate→actual is
   right-skewed (long tail): expected cost > midpoint. `(lower+upper)/2`
   systematically under-costs *all* estimated items; the bare inversion is the
   loud symptom of the same modelling error.

Goal: a cost model where (a) known items are priced toward the upper tail, and
(b) an unknown cost reads as *expensive* (≈ "worse than the worst known"), not
free — without assuming any fixed estimate scale.

## 2. Current State

`base_score` (`src/priority/graph.rs:72-105`) computes the divisor inline:

```rust
let est_mid = match f.estimate {
    Some(ref e) => { let m = f64::midpoint(e.lower, e.upper);
                     if m < EPSILON { EPSILON } else { m } }
    None => 1.0,
};
… cfg.coefficients.value * v.value * kw * tag_term / est_mid
```

`base_score` is per-node-pure, graph-free, run in the `build_from` base pre-pass
(2b) before mint; it feeds the mint tiebreaker (ADR-015 §2). `PriorityConfig`
(`src/priority/config.rs`) loads a `[priority]` table; `est_mid = 1.0` for the
absent case is **authored ADR-015 §1 policy**, not an impl gap.

## 3. Forces & Constraints

- **ADR-015 §1** decides the cost term *and* the `absent ⇒ 1.0` rule → both
  amend at the governance tier.
- **ADR-015 §2** pins the base pre-pass as "from its **own** authored facets …
  no graph access … safe as the graph mint tiebreaker." A corpus-wide anchor
  introduces a non-local input — must stay *pre-graph* and *deterministic* so the
  load-bearing invariant (mint safety) holds; the §2 *wording* amends.
- **ADR-015 §4** config contract is role-grouped sub-tables → add, don't overload.
- **SPEC-020 REQ-310 / FR-011** declares the estimate confidence band drives "**no**
  predicate, **aggregation**, normalization, or validation in v1." This design
  *intentionally lifts* that deferral — `max_upper` is an aggregation over the
  bounds. The lift is authorized by consult (this design session reframes the v1
  line as v0.1) and recorded by the REV at reconcile; SPEC-020 is amended
  alongside ADR-015, not silently outrun.
- **Pure/imperative split** (AGENTS.md): no disk/clock/rng in the pure layer —
  the corpus aggregate is computed in the impure scan shell and passed in
  (date/uid pattern).
- **Behaviour-preservation gate**: shared scoring machinery — existing suites are
  the proof; goldens recompute deliberately, never silently.
- **No fixed scale**: estimate bounds are free floats — the anchor must be
  data-driven, not a hard-coded 1–6 assumption.

## 4. Guiding Principles

- Unknown cost ≠ zero cost. An absent estimate is *maximal* uncertainty.
- One cost model for known and unknown items — the long-tail thesis applies to
  both; don't special-case bare items with a second philosophy.
- `β = 0.5` reproduces today's midpoint exactly — a clean migration anchor and a
  legible "off switch" for the skew.
- Knobs are operator-tunable and fail safe (clamp, never error) — matches the
  existing `[priority]` config posture.

## 5. Proposed Design

### 5.1 System Model

Replace the inline divisor with a pure `est_cost`, fed a precomputed
corpus-anchor context:

```text
est_cost(item) =
    has_estimate:  lower + β·(upper − lower)         # β skew ∈ [0,1]; 0.5 ≡ midpoint
    bare:          max_upper(corpus) + margin         # data-driven anchor
    (empty corpus, no estimates anywhere): 1.0        # legacy fallback
value_dim = (coeff.value × value × kind_weight × tag_term) / est_cost
```

`max_upper` = max `upper` bound across every estimated entity **that is not
`Terminal`** (`partition::status_class(kind, status) != Terminal`), all kinds.
Excluding terminal items is load-bearing: surfaces (`survey`/`next`) filter out
closed/promoted rows (`surface.rs:124-145`), so anchoring on *all* scanned
entities would let a **closed** huge-`upper` item inflate the anchor and sink
*visible* bare items — visible ranking depending on invisible entities. The
classifier is the canonical pure one (kind + status, both on `ScannedEntity`) —
no parallel predicate, and it runs pre-graph. Kind-comparability already lives in
`kind_weight`, so the anchor stays global across kinds.

Because `est_cost_i = lower_i + β(upper_i−lower_i) ≤ upper_i ≤ max_upper`, a bare
item with `margin > 0` costs ≥ every (non-terminal) estimated item — the inversion
is gone, while value still multiplies through (a high-value bare item can still
outrank a low-value estimated one — proportional, not a hard floor).

### 5.2 Interfaces & Contracts

`src/priority/graph.rs` — new pure helpers + threaded context:

```rust
const EPSILON: f64 = 1e-12;
fn floor_eps(x: f64) -> f64 { if x < EPSILON { EPSILON } else { x } }

/// Precomputed corpus cost-anchor (the absent-estimate divisor). Pure input to
/// `base_score`; computed once in the impure scan shell (date/uid pattern).
struct CostCtx { absent: f64 }

fn est_cost(est: Option<&EstimateFacet>, ctx: &CostCtx, ec: &config::EstimateCost) -> f64 {
    match est {
        Some(e) => floor_eps(e.lower + ec.skew * (e.upper - e.lower)),
        None    => floor_eps(ctx.absent),
    }
}

// signature gains the context:
fn base_score(f: &EntityFacets, kind: &entity::Kind,
              cfg: &config::PriorityConfig, ctx: &CostCtx) -> BaseScore
```

`base_score` has exactly one non-test caller (`build_from`, `graph.rs:234`) — the
signature change is fully contained; test sites read through `build`/`attrs`.

`src/priority/config.rs` — new role sub-table:

```rust
#[derive(…)] struct EstimateCost { skew: f64, margin: f64 }
impl Default for EstimateCost { fn default() -> Self { Self { skew: 0.65, margin: 1.0 } } }
// PriorityConfig gains `estimate: EstimateCost`
// load_from_table: table.get("estimate") → f64_or(t,"skew",0.65), f64_or(t,"margin",1.0)
// clamp(): estimate.skew → clamp [0.0,1.0];  estimate.margin → max(0.0)
//          (NaN/inf already route to default via the existing path)
```

`[priority.estimate]` TOML — `skew` (default 0.65), `margin` (default 1.0).

`src/commands/config.rs` — the first-party `doctrine config show/set/get/unset`
hardcode the known `[priority]` keys (`config.rs:174-427`). Extend each to
recognise `estimate.skew` / `estimate.margin`, so the operator surface matches the
authorable schema (manual-TOML-only would be a silent gap). `show` gathers the two
new rows; `set/get/unset` route them through the same validated path.

### 5.3 Data, State & Ownership

- `CostCtx` is build-scoped, owned by `build_from`, borrowed read-only by every
  `base_score` call. Single `f64` scalar — no per-node state.
- `EstimateCost` is owned by `PriorityConfig`, loaded once per build.
- No authored-file or derived-index change; this is read-time scoring only.

### 5.4 Lifecycle, Operations & Dynamics

`build_from` (`src/priority/graph.rs:219+`), inserted **before** the 2b base
pre-pass:

```rust
let max_upper = scanned.iter()
    .filter(|e| partition::status_class(e.kind, e.status.as_deref())
                != partition::StatusClass::Terminal)
    .filter_map(|e| e.estimate.as_ref().map(|x| x.upper))
    .reduce(f64::max);
let ctx = CostCtx { absent: match max_upper {
    Some(m) => m + cfg.estimate.margin,
    None    => 1.0,
}};
// 2b loop: base_score(&facets, entity.kind, &cfg, &ctx)
```

The anchor population (non-terminal) ≠ the mint population (all scanned). That is
deliberate: terminal items are still *scored* (so `inspect`/`explain` report a
number) but must not *price* the anchor. `base_score` itself stays per-node; only
the shared anchor input is corpus-derived.

Same `scanned` slice the pre-pass already folds — one extra O(n) pass, no new
scan. `ctx` is fixed before mint → mint stays deterministic; consequence
post-pass unchanged.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1** `β = 0.5` ⇒ `est_cost = midpoint` exactly (legacy equivalence).
- **INV-2** `margin > 0` ⇒ every bare item's `est_cost` ≥ every **non-terminal**
  estimated item's `est_cost` (anchor dominance). Strict for non-pathological
  magnitudes; at `upper ≈ 2⁵³` the `+margin` is below f64 granularity and the
  relation degrades to equality (same class as the `margin = 0` tie) — acceptable
  for espresso-shot–scale points, asserted as `≥` not `>`.
- **INV-3** `est_cost ≥ EPSILON` always (no div-by-zero) — `floor_eps` on both
  branches preserves the current guard for non-positive bounds.
- **INV-4** base pre-pass remains graph-free and deterministic (mint safety).
- **Edge — empty corpus**: no estimate anywhere ⇒ `absent = 1.0` (legacy; no
  inversion possible since every item is bare).
- **Edge — non-positive bounds** (`lower + β·span ≤ 0`): floored to EPSILON.
- **Edge — value absent**: `value_dim = 0` regardless of `est_cost` (unchanged);
  the cost model only bites value-bearing items.
- **Edge — NaN bounds**: `reduce(f64::max)` ignores NaN operands; an all-NaN
  corpus yields NaN `absent`, but the existing `if raw.is_finite() { raw } else
  { 0.0 }` guard (`graph.rs:94`) collapses the resulting `value_dim` to 0.0 — no
  NaN escapes. Preserved, not newly relied upon.
- **Edge — `margin = 0`**: clamp permits it (operator choice); INV-2 then weakens
  to `≥` (a bare item can *tie* the single worst-`upper` estimated item). Default
  `1` keeps strict dominance; documented degeneracy, not a defect.
- **Assumption**: anchoring on `max(upper)` (not `max(est_cost)`) is intentional
  — independent of β and still dominant (upper ≥ est_cost).
- **Assumption**: the EstimateFacet invariant `lower ≤ upper` is owned by the
  estimate facet (set-time), not re-validated here; inverted bounds (if any) stay
  bounded via `floor_eps`.
- **Accepted limitation — percentile-blind blend**: SPEC-020 reads bounds as a
  percentile band (`lower_confidence`/`upper_confidence`, default P10/P90). A fixed
  linear `β` is blind to that band, so `β = 0.65` targets a different latent
  quantile if an operator retunes the band. Accepted: `score` is an advisory
  heuristic, not a calibrated estimator; a band-aware estimator is a future knob.
  The SPEC-020 amendment (REV) records this explicitly.

## 6. Open Questions & Unknowns

- **OQ-1 (resolved → consult + reconcile)**: the governance change spans
  **ADR-015 §1+§2+§4 AND SPEC-020 REQ-310/FR-011** (the v1 aggregation deferral,
  intentionally lifted). Direction authorized by consult in this design session;
  the REV (ADR-013) is authored once the design locks and *landed at* `/reconcile`
  as the durable record. Amend, not supersede — both are refinements of intact
  recent decisions.
- **OQ-2 (accepted property)**: corpus coupling makes scores *relative* — adding
  a big-`upper` item raises `max_upper`, lowering every bare item's score.
  Deterministic per build; documented, not a defect.
- **OQ-3 (accepted)**: additive `margin` is scale-sensitive (decisive on 1–6,
  weak on 0–100). Default `1`; a multiplicative margin is a future knob if needed.

## 7. Decisions, Rationale & Alternatives

- **D1 — skew on by default (β=0.65), global re-price.** The long-tail thesis
  applies to known items too; a bare-only fix would leave the general mis-pricing
  live. Alt (β=0.5 default, opt-in): rejected — preserves the wrong central
  tendency. β=0.5 remains available as the off switch.
- **D2 — data-driven anchor `max_upper + margin`.** Self-scaling; needs no fixed
  scale config (code is scale-agnostic). Alt (fixed `M_absent` scalar / explicit
  scale bounds): rejected — reintroduces a scale assumption the corpus already
  carries.
- **D3 — `[priority.estimate]` sub-table.** Cohesive with `est_cost`, parallel to
  `[priority.consequence]`; a clean §4 *addition*. Alt (fold into
  `[priority.coefficients]`): rejected — β/margin are not multipliers.
- **D4 — global across kinds, but non-terminal only.** Cost is points,
  kind-independent (kind-comparability is in `kind_weight`), so the anchor spans
  kinds. But it folds only `status_class != Terminal` items, so closed/done work
  cannot poison live ranking (codex F3). Alt (per-kind): rejected — keyed
  aggregate + per-kind empty fallback, no benefit. Alt (all-scanned): rejected —
  the poison defect.
- **D5 — amend, not supersede; ADR-015 + SPEC-020.** Both refinements of intact
  decisions; SPEC-020's v1 aggregation deferral is deliberately lifted (consult).
- **D6 — extend `doctrine config` to the new keys.** The config CLI is the
  operator surface; manual-TOML-only would be a silent gap (codex F4).
- **D7 — no migration artifact.** `score` is recomputed every read from authored
  facets — no persisted ranking to migrate. The β=0.65 upgrade behaviour-change is
  the intended D1 re-price; β=0.5 is the documented off-switch (rebuts codex F5's
  migration ask).

## 8. Risks & Mitigations

- **R1 — golden churn masks a real regression.** Mitigation: recompute goldens
  *after* the targeted unit tests (INV-1..3) pass, and eyeball each fixture delta
  for direction (bare items sink, estimated items' relative order stable under
  β-only change). Never blind-accept. Blast radius confirmed by audit of
  score/order-asserting tests: the three named goldens recompute;
  `tests/e2e_inspect_golden.rs` is expected **unchanged** (value-free fixtures →
  `score: 0.0`) and is promoted to a design-target only if a fixture actually
  moves; `e2e_estimate_non_blocking` / `e2e_help_families_golden` assert no
  score/order and are unaffected.
- **R2 — ADR-015 §2 locality + SPEC-020 aggregation divergence read as drift.**
  Mitigation: the REV amends §2 wording and SPEC-020 REQ-310/FR-011 explicitly;
  design names the load-bearing invariant (pre-graph determinism) that actually
  constrains mint, and the non-terminal anchor population that bounds the coupling.
- **R3 — clamp gap on skew.** Mitigation: explicit `[0,1]` clamp test; `f64_or`
  + `clamp` already cover NaN/inf/negative.

## 9. Quality Engineering & Validation

Unit (inline `#[cfg(test)]`, `src/priority/graph.rs`):
- retire `base_score_absent_estimate_uses_midpoint_one`;
- `base_score_absent_estimate_anchored_to_max_upper_plus_margin`
- `est_cost_skew_half_equals_legacy_midpoint` (INV-1)
- `est_cost_skew_pulls_toward_upper` (β=0.65)
- `bare_item_not_below_equal_value_estimated` (INV-2, `≥`, ordering)
- `terminal_item_excluded_from_anchor` (codex F3 — closed huge-`upper` item does
  NOT move a live bare item's score)
- `empty_corpus_bare_falls_back_to_one`
- `est_cost_floored_on_nonpositive_bounds` (INV-3)
- **migrate the existing midpoint-coupled assertions** at `graph.rs:1142-1175`
  (`base_score_all_facets_present` etc.) to the skewed `est_cost` — guaranteed
  movers, not optional.

Config (`src/priority/config.rs`):
- `estimate_defaults_skew_065_margin_1`
- `skew_clamped_to_unit_interval`, `margin_clamped_nonneg`

Explain / config CLI (guaranteed movers):
- `src/priority/surface.rs` `va1_explain_exposes_full_score_breakdown`
  (`~1052-1089`) recomputes under skew (the `value 10.0` breakdown → skewed). Note
  `channels::value_dim` (`channels.rs:193`) only *reads* `base_score.value_dim` —
  single source of truth, no parallel cost math to update.
- `src/commands/config.rs` tests for the two new keys via `show/set/get/unset`.

E2E goldens recompute (deliberate, reviewed): `tests/e2e_priority_golden.rs`,
`tests/e2e_priority_cross_kind.rs`.
**Removed from scope** (codex F6): `tests/e2e_backlog_list_order_golden.rs` —
backlog list sorts by kind ordinal, "explicitly NOT a priority claim"
(`backlog.rs:451-456`); `est_cost` does not move it.
Verify-unchanged (no edit expected): `tests/e2e_inspect_golden.rs` (value-free
corpus) — escalates to a design-target edit only if its fixtures shift.

Verification modes: VT throughout (pure deterministic scoring); golden deltas VH-
reviewed for direction.

## 10. Review Notes

**Internal self-review** — integrated: `base_score` single non-test caller
(contained); golden blast radius audited; NaN / `margin=0` edges; INV-2 magnitude.

**External adversarial pass — codex (GPT-5.5), 2026-06-28.** Triage:
- **F3 (MAJOR, accepted)** — anchor poisoned by terminal/excluded entities. Fixed:
  fold `status_class != Terminal` only (§5.1/§5.4, D4).
- **F2 (BLOCKER, accepted)** — governance wider than ADR-015: SPEC-020 REQ-310 /
  FR-011 defers band aggregation to "v1". Resolved via consult (deliberate lift,
  v1→v0.1) + REV spanning both, landed at reconcile (§3, OQ-1, D5).
- **F4 (MAJOR, accepted)** — `doctrine config` CLI blind to new keys. Fixed: extend
  it (§5.2, D6).
- **F1 (MINOR, accepted)** — INV-2 not strict at f64 granularity. Weakened to `≥`
  (INV-2).
- **F6 (MINOR, partly accepted)** — dropped over-scoped `e2e_backlog_list_order`
  golden; added the real movers (`graph.rs:1142-1175`, explain `va1`). Rebutted the
  "parallel midpoint math" claim — `channels::value_dim` is read-only over
  `base_score` (single source).
- **F5 (rebutted in part)** — no migration artifact exists (scores recomputed every
  read; D7). Percentile-blindness accepted as a heuristic limitation, recorded in
  the SPEC-020 REV (§5.5).
