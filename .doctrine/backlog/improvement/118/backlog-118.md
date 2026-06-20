# IMP-118: Multi-dimensional priority scoring for survey/next

## Context

The item-level authored-priority slot feeding `survey` / `next` is
currently empty in `priority/surface.rs`. The existing sort chain is
`actionability → consequence desc → canonical-id asc`, where
`consequence` is a raw inbound-reference count (how many other entities
reference this one). That count is blind to *what* depends on it — a
blocker gating 5 ideas scores the same as one gating 5 high-value
slices.

## Precedence

Slot precedence is fixed by SPEC-001 D10, REQ-054 (FR-006), and PRD-011 OQ-001.

## Architecture

### Model

One composite priority score, computed from multiple **dimensions**.
Each dimension has a configurable coefficient in `doctrine.toml`; no
per-item priority overrides. Change the coefficients, reorder the
graph.

### Dimensions

Each dimension is a pre-computed scalar from authored facets and config:

| dimension | input | formula |
|-----------|-------|---------|
| value     | `[value] value` (f64), `[estimate]` (lower/upper bounds), kind, tags | `(value × kind_weight × Σ(tag_coeffs)) / estimate_midpoint` |
| risk      | `[facet] likelihood × impact` (1..=16, risk-only) | `exposure × risk_coeff` |

Additional dimensions (e.g. maintainability) are anticipated but out of
scope for the initial implementation. Their coefficient defaults to 0.0
(future-proof: parse unknown keys, multiply by zero).

When a required facet is absent:
- Value absent → value dimension is 0.0
- Estimate absent → no discount (midpoint = 1.0)
- Risk facet absent (non-risk items) → risk dimension is 0.0

### Coefficients

All coefficients live in `doctrine.toml` under `[priority]`:

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
```

Kind weights and tag coefficients default to 1.0 when not configured
(a missing entry = neutral). Tags are matched by exact string against
the item's `tags` list. Tag coefficients are **additive**: an item with
`area:security` (2.0) and `area:cli` (1.5) has a tag multiplier of
`1.0 + (2.0 - 1.0) + (1.5 - 1.0) = 2.5`. The identity multiplier is
always 1.0 (the neutral) so that absent tags contribute nothing.

### Two-pass computation

**Pass 1 — base score** (per-node, pre-graph, pure over config +
authored facets):

```
base = value_dimension + risk_dimension
```

No graph access needed — each item's own TOML + `doctrine.toml` config
suffices. Computed once during the scan, stored on `NodeAttr`.

**Pass 2 — consequence** (post-graph-build, over the assembled
`PriorityGraph`):

For each node, sum the **base scores** of its dependents across two edge
classes, each with its own coefficient. Dep edge orientation is
`prereq→src` (B→A flip): if A needs B, the edge is B→A. Consequence
walks OUTWARD on dep edges (`out_edges`) — from blocker to blocked — and
INWARD on reference edges (`in_edges`) — from target to referrer.

```
consequence(node) =
    Σ base(dependent) × ref_edge_coeff   for each dependent reaching node via reference/lineage edges
  + Σ base(dependent) × dep_edge_coeff   for each dependent reaching node via dep (needs) edges
```

For reference edges (referrer→target), `in_edges(ref_overlay, node)` gives
the referrers. For dep edges (prereq→src), `out_edges(dep_overlay, node)`
gives the nodes blocked on this one.

The reference/lineage edge class is the existing `CONSEQUENCE_LABELS`
set (`Specs`, `Requirements`, `Slices`, `DescendsFrom`, `Parent`,
`Members`). The dep edge class is the `needs` overlay — items that
*block on* this node.

A dependent's base score is contributed **once per edge class** per
distinct dependent: each reference overlay checks `in_edges` on its
overlay, summing dependents' base scores; the dep overlay separately
checks `out_edges`. Same dependent via two reference overlays contributes
twice (once per overlay) — the overlay IS the edge class. Duplicate edges
within one overlay are deduped by cordage's `BTreeSet` adjacency.

Coefficients in config:

```toml
[priority.consequence]
ref_edge_coeff = 1.0
dep_edge_coeff = 2.0
```

**Final score:**

```
score = base + consequence
```

No fixpoint problem — base depends only on own facets, consequence
depends only on dependents' base scores. One scan, one graph walk.

### Integration into graph build

The current build pipeline:

1. Scan entities
2. Consequence pre-pass (raw count of inbound refs)
3. Mint in `(consequence desc, canonical-id asc)` order
4. Emit dep/seq/reference edges
5. Build cordage graph

The replacement:

1. Scan entities
2. **Base-score pre-pass** — compute `base` for each scanned entity from
   config + authored facets; store on `NodeAttr`
3. Mint in `(base desc, canonical-id asc)` order
4. Emit dep/seq/reference edges
5. Build cordage graph
6. **Consequence post-pass** — walk the graph, compute consequence from
   dependents' base scores across both edge classes
7. Store score (`base + consequence`) on `PriorityGraph`

### Sort integration

**`survey`:** `actionability(Actionable > Blocked) → score desc →
canonical-id asc`

**`next`:** order_key-based (cordage-composed). The within-level
tiebreaker is the mint order, which is now `(base desc, canonical-id
asc)` — consequence is NOT embedded in the cordage ordering. This is
intentional: consequence is a ranking signal (how much depends on this),
not a structural ordering constraint. The structural order (`needs`
levels) already captures blocking relationships; adding consequence to
the within-level sort would create a feedback loop between graph
topology and dependency ordering.

**`explain`:** exposes the score and its constituent dimensions.

### Pure/impure split

- Config parsing (`doctrine.toml` → `PriorityConfig`) — impure (disk read)
- Base score computation — pure (over config + authored facets)
- Consequence post-pass — pure (over the already-built `PriorityGraph`)
- `PriorityGraph` carries the config as an owned field (parsed once, read
  by pure passes)

### Future dimensions

New dimensions are added by:
1. Adding a coefficient field to `PriorityConfig`
2. Adding an authored facet (or reusing an existing one)
3. Adding the dimension to the base-score formula

Unknown config keys are ignored (forward-compatible). A dimension whose
coefficient is 0.0 is effectively disabled.
