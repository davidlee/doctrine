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

## Governance note (architectural feedback)

The scoring formula currently lives in IMP-118 prose (backlog body). Before
design proceeds, the design MUST explicitly mark which parts are durable
policy vs. tunable implementation defaults:

- **Durable policy** (promote to SPEC/ADR): dimension semantics (value,
  risk), two-pass model, config shape, sort integration contract
- **Tunable defaults** (implementation-owned): coefficient values,
  kind_weight defaults, tag_coefficient examples

The first implementation may own its formula, but the design must draw the
line between "this is how doctrine scores" and "these are the starting
numbers."

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

| File | Change |
|------|--------|
| `src/priority/graph.rs:51` | `NodeAttr` — add `base_score: f64` |
| `src/priority/graph.rs:106` | Remove `counts_toward_consequence` |
| `src/priority/graph.rs:162` | Consequence pre-pass → replace with base-score pre-pass |
| `src/priority/graph.rs` | Add consequence post-pass over built graph |
| `src/priority/surface.rs:93,140` | Replace `consequence: u32` with `score: f64`, sort change |
| `src/priority/render.rs` | Display rendering for score in survey/next rows |
| `src/relation.rs:93` | `CONSEQUENCE_LABELS` — reuse for ref edge class |
| `doctrine.toml` | New `[priority]` section |
| `src/main.rs` | Parse `[priority]` from config |

## Dependencies

- **after**: SL-132 (display), SL-134 (risk CLI), IMP-134 (tagging)
- Config parsing precedent: `[conduct]` (src/slice.rs:443), `[dispatch]`
  (src/dispatch_config.rs:29)
- Risk model: `src/backlog.rs:382` — `RiskFacet`, `exposure()` → 1..=16
