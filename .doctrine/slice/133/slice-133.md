# Multi-dimensional priority scoring for survey/next

## Context

`survey` and `next` currently sort by `actionability → consequence desc →
canonical-id asc`, where `consequence` is a raw inbound-reference count.
That count is blind to *what* depends on an item — a blocker gating 5 ideas
scores the same as one gating 5 high-value slices.

IMP-118 specifies a multi-dimensional scoring system that consumes the
authored `[value]`, `[estimate]`, and risk `[facet]` facets and applies
config-driven kind weights and tag coefficients from `doctrine.toml`.

## Scope & Objectives

### Dimensions

- **Value dimension**: `(value × kind_weight × Σ(tag_coeffs)) / estimate_midpoint`
- **Risk dimension**: `exposure × risk_coeff` (risk items only, 1..=16)
- Absent facets → dimension = 0.0 (value absent → 0.0; estimate absent → midpoint = 1.0)

### Two-pass computation

**Pass 1 — base score** (per-node, pure over config + authored facets):
`base = value_dimension + risk_dimension`. No graph access — each item's own
TOML + config suffices. Store on `NodeAttr`.

**Pass 2 — consequence** (post-graph-build, over assembled PriorityGraph):
Walk the graph, sum dependents' base scores across two edge classes:
- Reference/lineage edges (`in_edges`): `Σ base(dependent) × ref_edge_coeff`
- Dep edges (`out_edges` on needs overlay): `Σ base(dependent) × dep_edge_coeff`
- `score = base + consequence`

### Config

New `[priority]` section in `doctrine.toml`:
```toml
[priority.coefficients]
value = 1.0
risk  = 2.0

[priority.kind_weights]
improvement = 1.2
issue       = 1.5
idea        = 0.3

[priority.tag_coefficients]
"area:security" = 2.0
"area:cli"      = 1.5

[priority.consequence]
ref_edge_coeff = 1.0
dep_edge_coeff = 2.0
```

Defaults: kind_weights → 1.0, tag_coefficients → 1.0 (additive: identity
multiplier of 1.0 so absent tags contribute nothing). Unknown config keys
ignored (forward-compatible).

### Sort integration

- `survey`: `actionability → score desc → canonical-id asc`
- `next`: order_key-based (cordage-composed). Within-level tiebreaker is
  mint order `(base desc, canonical-id asc)` — consequence is deliberately
  excluded from structural ordering (would create feedback loop).
- `explain`: expose score and constituent dimensions.

### Pure/impure split

- Config parsing (`doctrine.toml` → `PriorityConfig`) — impure (disk)
- Base score computation — pure (config + authored facets)
- Consequence post-pass — pure (over built PriorityGraph)

### Soft-dependency on IMP-134

Tag coefficients default to 1.0 when absent — scoring ships with or without
tagging. The `after IMP-134` edge is a soft preference, not a hard blocker.

## Governance — ADR-015 (architectural feedback)

The scoring formula currently lives in IMP-118 prose (backlog body).
Durable policy must be ratified in a governing artifact before
implementation.

**Scope deliverable: ADR-015 — Multi-dimensional priority scoring**

ADR-015 captures durable policy:
- Dimension semantics (value, risk)
- Two-pass computation model (base + consequence)
- Config shape (`[priority]` section in `doctrine.toml`)
- Sort integration contract (`survey` / `next` / `explain`)

Tunable defaults remain implementation-owned:
- Coefficient values
- Kind-weight defaults
- Tag-coefficient examples

ADR-015 is authored during design phase and referenced by the
implementation plan — it separates "this is how Doctrine scores"
from "these are the starting numbers."

## Shared facet projection (architectural coupling risk)

`ScannedEntity` and `PriorityGraph` do not currently carry
estimate/value/risk/tag data. Both SL-132 (display) and SL-133 (scoring)
need this same data. A shared `EntityFacets` projection should be
established BEFORE either slice grows its own parser — avoid parallel
parsing of the same authored facets.

Suggested shape:
```rust
struct EntityFacets {
    estimate: Option<EstimateFacet>,
    value: Option<ValueFacet>,
    risk: Option<RiskFacet>,
    tags: Vec<String>,
}
```
Then `format_show`, `build_priority_graph`, and `explain` consume the
same projection.

## Non-Goals

- No per-item priority overrides (coefficients only, no escape hatch)
- No maintainability/cost dimension (future, coefficient defaults to 0.0)
- No history tracking (IDE-013, deferred)
- Risk dimension uses existing `RiskFacet.exposure()` — no new model

## Terrain

(Revised at design — risk-leaf extraction + build-seam config; see `design.md` §7 D2/D4.)

| File | Change |
|------|--------|
| `src/risk.rs` (new) | Extract risk facet types from `backlog` to a **leaf** (forced by ADR-001 — D2): `RiskLevel`, `RiskFacet`, `RawRiskFacet`, parse/validate, `exposure()` |
| `src/backlog.rs` | Re-use the leaf risk types (command→leaf); behaviour-preserving |
| `src/facet.rs` | `EntityFacets` gains `risk: Option<RiskFacet>` |
| `src/catalog/scan.rs` | `read_facets` reads the `[facet]` table; `ScannedEntity` gains risk |
| `src/priority/config.rs` (new) | `PriorityConfig` serde struct + impure `load(root)`; advisory-config clamp policy (`COEFF_MAX`, silent) — F-6/OQ-1 |
| `src/priority/graph.rs` | `NodeAttr` gains `base_score: BaseScore`; replace consequence pre-pass with base pre-pass; mint retie `(base desc, id asc)`; add consequence **post**-pass (ref-class `in_edges` over `CONSEQUENCE_LABELS`, dep-class `out_edges(dep_overlay)`); `PriorityGraph.consequence:u32 → score:f64` **+ stored `consequence:f64` map** (F-3); `is_finite` sanitize dims/total/consequence (F-2); `build_from` loads `&PriorityConfig` from `root` (covers all callers — F-4) |
| `src/priority/surface.rs` | `consequence:u32 → score:f64` across `SurveyRow`/`ActionabilityNode`/`ActionabilityBlock`; sort on score; `policy_version` v2→v3 |
| `src/priority/render.rs` | `survey` score column only (`next` has none — score via `ReasonKind::Score` reason line, F-8); `ReasonKind::Score{base,value_dim,risk_dim,consequence,total}` human + json |
| `.doctrine/adr/001/layering.toml` | **Binding tier-map (F-1, ADR-001 forcing fn):** add `risk = "leaf"`, `priority::config = "leaf"`; relax `facet` comment to permit the risk import — `just gate` green |
| `doctrine.toml` | New `[priority]` section |
| `.doctrine/adr/015/**` | **ADR-015** — durable scoring policy (authored this phase) |

Config loads **inside `build_from`** from its `root` arg (which already drives the impure
`dep_seq_for` reads), **not** `src/main.rs` — D4. This covers every `build_from` caller
(incl. the pre-scanned `actionability_block_from`, surface.rs:484) with no signature
change — F-4. Tag coefficients are a **stub** (`Σ = 1.0`, no scan read) this slice; the
seam exists in the formula, lit up once SL-136 lands tag storage — D5.

## Dependencies

- **needs**: SL-132 (display) — scoring is untestable and unshippable
  without a surface to render its output
- **after**: SL-134 (risk CLI), IMP-134 (tagging) — additive, not
  foundational; risk is authorable by hand, tags default to 1.0
- Config parsing precedent: `[conduct]` (src/slice.rs:443), `[dispatch]`
  (src/dispatch_config.rs:29)
- Risk model: `src/backlog.rs:382` — `RiskFacet`, `exposure()` → 1..=16
