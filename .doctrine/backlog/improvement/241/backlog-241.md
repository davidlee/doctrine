# IMP-241: Interestingness findings over the actionability graph — text-first probe

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

Design thread (2026-07-03) on actionability-graph visualisation. Corpus
measurement reframed the problem: 184 nodes, 4 `needs` edges, 41 distinct
scores across 182 rows, 168/181 adjacent pairs near-tied. The actionability
graph is **order-rich, edge-poor**; the current web view (d3-dag sugiyama in
`web/map/src/priority.ts`) spends both spatial axes on layout aesthetics over
a graph that has almost no shape. The user's real questions — what now, why
not X, what does settling X unlock, where are the contested orderings — are
answered by *findings*, not by a whole-graph picture.

This is an RFC-007 workstream (2) "Legible" item: bring the truth to where
attention already is, textually first.

## The probe

Implement an **interestingness-findings catalogue** as pure functions over
`PriorityGraph` (`src/priority/graph.rs`) — NOT the thin `ActionabilityView`
web projection, which carries only total score + rank + blockers and never
sees provenance or components. `PriorityGraph` is the honest substrate: the
cordage `Graph` (provenance: `EvictedEdge`, `Degraded` levels), the
`EntityKey↔NodeId` projection, per-node attrs (`base_score`, facets), the
leverage/optionality/score maps, and the dep/seq overlay handles. Findings
needing β-perturbation additionally need the **rebuild seam** (`build_from`
+ perturbed `[priority.estimate]` config) — est_cost bakes into base at node
mint, so instability is re-derivation, not a read. Substrate tiers per
finding: structural (fork/join/inversion/displacement/gating fan-out) ←
graph + score map; decomposition/plateau ← component maps; provenance ←
cordage provenance; instability ← rebuild seam. No visualisation. Surface
as text —
`survey --interesting` or a `findings` verb (final surface via design
conversation). Each finding: node subset + reason + magnitude, rendered one
line each, e.g.

```
Y-fork @ QUE-003: settles → {IMP-054, IMP-071, ISS-028}; arm order contested (flips at β<0.4)
```

### Finding catalogue (initial)

| finding | detection | value |
|---|---|---|
| fork (Y) | out-degree >1 in unlock direction | one settle opens N arms; arm order is a live decision |
| arm resequencing | fork whose arm order flips under estimate perturbation | contested, not dominated |
| order instability | recompute order at β=0 / β=1 (estimate interval bounds); flipped pairs | order is an artifact of a guess |
| value inversion | edge where score(blocked) ≫ score(blocker) | low-value item gating high-value item |
| displacement | \|constrained position − pure-score position\| large | constraints doing real work |
| gating fan-out | ADR-017 record (QUE/DEC/ASM/CON…) blocking ≥2 items | one answer unlocks a front |
| join / convergence | in-degree >1 | schedule risk; last-arriving prereq rules |
| provenance anomalies | cordage evictions, `Degraded` levels | already computed, never surfaced |
| plateau exposure | consecutive near-tie segments (gap < ε) in `next` order | shows where order is arbitrary and facet-setting would actually bite |

Structure-only findings (fork, join, inversion, gating fan-out, provenance)
work on today's sparse facet data; β-sensitivity findings starve until
estimates are set — acceptable, they activate as facet coverage grows.
β-perturbation is cheap: the engine is pure; rebuild ×3 per refresh.

## Design direction this probe validates (follow-on, not in scope)

If the findings read as useful, they become the **inclusion policy and
interface** for all actionability rendering:

- **Include by finding, not by existence** — default graph view = union of
  finding subgraphs + edge-connected components; isolated plateau nodes
  collapse to a scored list/count. Every rendered subgraph badges *why shown*.
- **Arc-strip linear view** — nodes on a line in `next` order; arcs above =
  `needs`, below = `after`; consecutive near-ties render as unordered
  segments (honest about the plateau); displacement badges; stacked
  score-component mini-bars (ADR-015); box width = est_cost.
- **Semantic synthesis, one rule** — the semantic graph never contributes
  ordering edges; it enters as lanes/colour (owning slice, spec lineage),
  on-focus ghost context (1-hop fulfils/implements/owning_slice), and value
  flow (fulfils + ADR-018 completion degree → frontier-to-requirement
  burndown).
- **Web** — pin y = server rank, x = score order (drop free sugiyama);
  hourglass cone focus mode; findings panel drives navigation.
- **CLI images** (kitty graphics protocol, DOT→PNG for focused cones) —
  deferred; text trees/bands expected sufficient at current density.

## Relations

- RFC-007 workstream (2) "Legible" — this item is one of the mints it calls
  for; sibling concerns: fold `explain` into `next`/`survey` (`--why`),
  coefficient what-if trace.
- ADR-015 (score components consumed by decomposition/instability findings),
  ADR-017 (gating-record findings), ADR-018 (value-flow follow-on).
- IMP-112 (estimate display in `show`) — adjacent; instability findings give
  estimates a consumer.
