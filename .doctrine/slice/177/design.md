# Design SL-177: Default value for valueless value-bearing kinds

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§8), F-1 (§10). -->

## 1. Design Problem

SL-176's `fulfils` **value-burndown** reduces a backlog item's priority by the
lifecycle-gated **raw `value` facet** of the slices fulfilling it
(`.doctrine/slice/176/design.md:297-311`, D-burndown-denomination):
`delivered(I) = Σ gate(status(src))·raw_value(src)`, `r(I) = clamp(delivered/raw_value(I),0,1)`.
The subtraction is value-denominated: it signals nothing for entities with **no
authored value**, which contribute `raw_value = 0` (`r(I)=0`, `delivered += 0`).

SL-176 (D-value-floor-sibling, user-locked 2026-06-29) defers the fix here: give
value-bearing kinds a **default value of 1.0** when none is authored, so burndown
has a denominator. **Sequencing-hard dependency: SL-177 needs SL-176** — it
retrofits SL-176's `raw_value` seam, which must exist first (RV-191 F-1).

## 2. Current State

`base_score` (`src/priority/graph.rs`, value-dim block ~L113) drops every
valueless entity — work or record — to `value_dim = 0`:

```rust
let raw = if let Some(ref v) = f.value {
    cfg.coefficients.value * v.value * kw * tag_term / cost
} else { 0.0 };
```

SL-176's burndown post-pass (being built in dispatch) reads the **raw authored
`value` facet** via its own `raw_value(..)` accessor — a *different seam* from
`base_score`. A default placed in `base_score` alone never reaches burndown
(RV-191 F-1, the cardinal correction below).

The value-bearing kind set already exists, function-locally, as `WORK_PREFIXES`
in `src/priority/surface.rs` (SL-089 D2): `["SL","ISS","IMP","CHR","RSK","IDE"]`.
**Distinct** from `dep_seq.rs::is_work_like` = same set **∪ REV** (RV-191 F-3):
a Revision is work-like for dep/seq but **not** value-bearing.

## 3. Forces & Constraints

- **STD-001 / no parallel impl.** One named source for the value-bearing set; no
  literal copies; do not conflate with `is_work_like`.
- **ADR-001 layering + cohesion.** `kinds.rs` (leaf) owns the kind set; the
  **default magnitude is priority-scoring policy**, homed in the priority tier
  beside the seam — *not* in `value.rs`, which stays authored-facet-pure
  (RV-191 F-4).
- **Single seam (RV-191 F-1).** The default is defined ONCE and consumed by every
  value read-site that feeds priority: `base_score`'s `value_dim` **and** SL-176's
  burndown `raw_value`.
- **Storage rule / A-1.** Applied at the scoring seam, never by mutating authored
  TOML.
- **Behaviour-preservation gate.** Scoped to genuinely-unrelated behaviour; tests
  that encode the *old* valueless-work==0 contract change by design (§9.1).

## 4. Guiding Principles

Default-when-absent, not a clamp. One shared seam, one named set, one home for the
constant. Tunability deferrable without rework.

## 5. Proposed Design

### 5.1 System Model

1. `kinds.rs` gains `VALUE_BEARING` + `is_value_bearing` (mirroring `BACKLOG` /
   `is_record`). **Not** named `WORK` (F-3).
2. The priority tier gains the **single shared accessor**
   `effective_raw_value(kind, &EntityFacets) -> Option<f64>` and the
   `DEFAULT_VALUE` const.
3. `base_score`'s `value_dim` consumes `effective_raw_value`.
4. SL-176's burndown post-pass `raw_value(..)` is **retrofitted** to consume
   `effective_raw_value` (the one line that makes the default reach burndown).
5. `surface.rs` drops its local `WORK_PREFIXES`, consumes `kinds::is_value_bearing`.

### 5.2 Interfaces & Contracts

`src/kinds.rs` (leaf):

```rust
/// Value-bearing kinds (SL-089 D2): a slice plus the five backlog kinds — the set
/// that carries a value facet and feeds priority value/burndown. A STRICT SUBSET
/// of dep_seq's `is_work_like`: value_bearing ⊂ work_like, parted by REV (a
/// Revision is work-like for dep/seq but NOT value-bearing). Governance and
/// knowledge records are excluded.
pub(crate) const VALUE_BEARING: &[&str] = &[SL, ISS, IMP, CHR, RSK, IDE];

pub(crate) fn is_value_bearing(prefix: &str) -> bool { VALUE_BEARING.contains(&prefix) }
```

`src/priority/graph.rs` (priority tier — the shared seam + its const):

```rust
/// Default raw value for a value-bearing entity that authors no `[value]` facet
/// (SL-177; SL-176 D-value-floor-sibling). A default-when-absent, NOT a min-clamp:
/// an authored value (incl. < 1.0 and 0.0) is returned untouched.
const DEFAULT_VALUE: f64 = 1.0;

/// The single definition of an entity's value for priority purposes. Authored
/// value wins; a value-bearing kind with no facet defaults to DEFAULT_VALUE; any
/// other valueless kind (records, governance) is None. Consumed by BOTH
/// `base_score`'s value_dim AND SL-176's burndown `raw_value` (RV-191 F-1).
fn effective_raw_value(kind: &entity::Kind, f: &EntityFacets) -> Option<f64> {
    f.value.as_ref().map(|v| v.value)
        .or_else(|| kinds::is_value_bearing(kind.prefix).then_some(DEFAULT_VALUE))
}
```

`src/value.rs`: **unchanged** by this slice (stays authored-facet-pure).

### 5.3 Data, State & Ownership

No new state, no storage change. The default exists only in the pure
`effective_raw_value` computation; the authored `[value]` facet (or its absence)
is the sole durable fact. Ownership: the kind set → `kinds.rs`; the default
magnitude + the accessor → priority tier (`graph.rs`); burndown is a *consumer*,
not an owner.

### 5.4 Lifecycle, Operations & Dynamics

`base_score` value-dim block:

```rust
let value_dim = {
    let raw = match effective_raw_value(kind, f) {
        Some(v) => {
            let cost = est_cost(f.estimate.as_ref().map(|e| (e.lower, e.upper)), ctx, &cfg.estimate);
            cfg.coefficients.value * v * cfg.kind_weight(kind.prefix) * tag_term / cost
        }
        None => 0.0, // records / governance with no value → still zero
    };
    if raw.is_finite() { raw } else { 0.0 }
};
```

SL-176 burndown retrofit (when SL-177 lands, the post-pass already built):
`raw_value(src)` / `raw_value(I)` call `effective_raw_value(kind, facets)`
(`.unwrap_or(0.0)` at the denominator guard, preserving SL-176's `raw_value>0`
branch — a defaulted `1.0` is `>0`, so valueless items now denominate).

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1 (no-clamp).** Authored `v` ⇒ `effective_raw_value == Some(v)`; the
  default never overrides a present value, incl. `v<1.0` and `v==0.0`.
- **INV-2 (record/REV exclusion).** `is_value_bearing(prefix)==false` for every
  record, governance, and **REV** kind ⇒ their valueless value stays `None`.
- **INV-3 (subset).** `VALUE_BEARING ⊂ is_work_like` (parted by REV); `VALUE_BEARING
  == [SL] ∪ BACKLOG` — both held by canary tests.
- **INV-4 (one seam).** Every priority value read-site routes through
  `effective_raw_value`; no site reads `f.value` raw for scoring/burndown.
- **Edge — authored 0.0 ≠ absent.** `Some(0.0)` ⇒ value `0`, distinct from a gap.
- **Edge — bounded surprise.** Unvalued **and** unestimated ⇒ cost = absent
  anchor (> largest real estimate) ⇒ `value_dim ≈ 1.0/large`, small. Only
  unvalued-but-cheaply-estimated items float (rare). (User; RV-191 F-5.)

## 6. Open Questions & Unknowns

- **OQ-1 (resolved — hard constant).** `DEFAULT_VALUE = 1.0`, not config-tunable;
  swap for a `cfg` field later with no seam-logic change.
- **OQ-2 (resolved — `VALUE_BEARING`).** Reuse the existing value-bearing set,
  promoted and renamed (NOT `WORK`); records/REV excluded.
- **OQ-3 (resolved — shared seam).** One `effective_raw_value` accessor feeds
  both `base_score` and burndown (RV-191 F-1).

## 7. Decisions, Rationale & Alternatives

- **D1 — Shared `effective_raw_value` seam (priority tier).** The default must
  reach every value read-site that feeds priority. A `base_score`-only default
  (original draft) silently failed burndown (RV-191 F-1). Alternative (default in
  two places) rejected: duplication drifts.
- **D2 — Name `VALUE_BEARING`/`is_value_bearing`, not `WORK`.** Avoids collision
  with `dep_seq::is_work_like` (∪REV) (RV-191 F-3). `is_work_like` untouched.
- **D3 — Apply at scoring seam, not authored TOML.** Storage honest (A-1).
- **D4 — Default-when-absent, not min-clamp.** Matches SL-176 ("governs the
  valueless case" only); authored sub-1.0 untouched.
- **D5 — `DEFAULT_VALUE` + accessor in priority tier, `value.rs` untouched.**
  Cohesion: it is scoring policy, not a facet property (RV-191 F-4).
- **D6 — Hard constant (OQ-1).**

## 8. Risks & Mitigations

- **R1 — `VALUE_BEARING` / actionability-node-set coincidence.** Identical today
  (both REV-less). If they diverge, split then (YAGNI). Low cost.
- **R2 — Sequencing (load-bearing).** SL-177 **needs SL-176**: the `raw_value`
  site must exist to retrofit. SL-177 lands second; interim, valueless items
  simply don't burn down (the soft posture SL-176 accepts). Relation authored via
  `doctrine needs`.
- **R3 — Test/golden blast radius.** Broad (§9.1). *Mitigation:* plan greps the
  full set and re-baselines goldens; preservation scoped to unrelated behaviour.
- **R4 — Standalone ordering shift.** Valueless work items gain a baseline
  `value_dim`; bounded by the estimate-anchor (§5.5 edge). Intended; user-acked.

## 9. Quality Engineering & Validation

TDD, red→green→refactor. New / asserted behaviour:

- **VT — value-bearing default (scoring).** Valueless `SL` + one backlog kind →
  `value_dim` equals the explicit-`value=1.0` computation.
- **VT — default reaches burndown.** A backlog item fulfilled by a **valueless**
  delivering-status slice → `delivered` reflects `1.0` and `r(I)>0` (the F-1
  regression guard — fails if burndown reads raw `f.value` instead of
  `effective_raw_value`). Pairs with SL-176's own burndown fixture.
- **VT — record/REV exclusion.** Valueless `ASM` and `REV` → value `None` →
  `value_dim == 0`; excluded from burndown.
- **VT — no-clamp.** Authored `value = 0.3` on `SL` → reflects `0.3`, not `1.0`.
- **VT — authored zero ≠ absent.** Authored `value = 0.0` → `value_dim == 0`.
- **VT — set canaries.** `VALUE_BEARING == [SL]+BACKLOG` and
  `VALUE_BEARING ⊂ is_work_like` (the REV gap).

### 9.1 Tests that change BY DESIGN (full blast radius — RV-191 F-2)

The behaviour-preservation gate covers only behaviour this slice does **not**
intend to change. Every fixture asserting the old valueless-work==0 contract is
**deliberately re-baselined** red→green. Known categories (plan enumerates
line-by-line via grep before touching code):

- **base_score unit tests** — `base_score_risk_only_value_absent` (ISS, no value,
  has risk: `value_dim 0→1.0`, `total 4.0→5.0`), `base_score_neither_facet_present`
  (ISS, no facets: `value_dim/total 0→1.0`).
- **graph.rs score-consequence tests** — valueless work fixtures asserting
  `score==0` (e.g. ~L987, L1161, L1264-1279); exact set is a plan grep.
- **e2e goldens** — `tests/e2e_priority_golden.rs` and
  `tests/e2e_inspect_golden.rs` (fixtures
  `tests/fixtures/sl071_inspect_sl00{1,3}_golden.json`) bake valueless SL/ISS/RSK
  scores; regenerate. `tests/e2e_priority_cross_kind.rs` is **untouched** — it
  asserts ordering/relations, not absolute scores.
- **in-tree test mod** — `src/priority/channels.rs` test fixtures also re-baseline.

`base_score_bare_item_empty_corpus_fallback_cost_one` (ISS, `value=3.0`) is
**unaffected** (authored value) — the standing no-clamp guard.

### 9.2 Genuine behaviour-preservation

- No consumer treats `value_dim == 0` as a "value-absent" sentinel (verified:
  flows only into `total` + render — `channels.rs:193`, `render.rs`). RV-191 F-6.
- `surface` actionability-view tests stay green after the `WORK_PREFIXES` →
  `kinds::is_value_bearing` promotion (set-preserving).

## 10. Review Notes

### Internal adversarial pass (2026-06-29)
- **AR-1 (fixed)** — false "suite unchanged" claim; two ISS value-absent tests
  encode the pre-change contract → reclassified as deliberate red→green (§9.1).
- **AR-2 (verified clean)** — no `value_dim==0` sentinel; lifting off zero is safe.
- **AR-3 / R4** — standalone ordering shift; bounded (§5.5 edge), user-acked.
- **AR-4** — `WORK`/value-bearing coupling; later superseded by F-3 rename.

### External adversarial pass — RV-191 (codex / GPT-5.5, read-only)
Five charges, all reconciled (synthesis in `review-191.md`):
- **F-1 (blocker, design-wrong)** — default at `base_score` only, never reaching
  SL-176 burndown's `raw_value`. **The cardinal correction**: shared
  `effective_raw_value` seam (D1). The slice would have passed its own tests and
  silently failed its purpose.
- **F-2 (major, fix-now)** — under-enumerated test/golden blast radius (§9.1).
- **F-3 (major, fix-now)** — `kinds::WORK` collides with `is_work_like` (∪REV);
  renamed `VALUE_BEARING` (D2).
- **F-4 (minor, design-wrong)** — `DEFAULT_VALUE` cohesion; rehomed to priority
  tier (D5).
- **F-5 (minor, follow-up)** — render authored-vs-effective legibility →
  **IMP-211**; bounded by the estimate-anchor.
