# Actionability interestingness findings

## Context

RFC-007 workstream (2) "Legible": the actionability graph is correct but
invisible; `next`/`survey` read as a flat, opaque, order-rich/edge-poor list
(184 nodes, 4 `needs` edges, 41 distinct scores). The user's real questions —
what now, why not X, what does settling X unlock, where are orderings contested
— are answered by *findings*, not by a whole-graph picture.

Originates from **IMP-241** (the detailed probe card: finding catalogue,
detection, per-finding substrate tier map). This slice is the text-first probe
that validates whether findings-over-picture reads as useful **before** any
visualisation or rendering-policy work is committed.

## Scope & Objectives

Implement an **interestingness-findings catalogue** as pure functions over the
priority engine, surfaced as text (one line per finding). No visualisation.

- **Substrate** (corrected from the card's original `ActionabilityView` cite):
  the honest substrate is `PriorityGraph` (`src/priority/graph.rs`) — cordage
  `Graph` + provenance, projection, `attrs` (base_score, facets),
  leverage/optionality/score maps, overlay handles — plus `PriorityConfig` for
  perturbation. `ActionabilityView` is the thin web projection (total score +
  rank + blockers only) and is insufficient for decomposition/provenance/β
  findings.
- **Finding layer is pure**, impurity injected. Likely signature shape:
  `fn findings(&PriorityGraph, &PriorityConfig, rebuild: impl Fn(β) -> PriorityGraph)`.
  The ×3 perturbation rebuild lives in the thin shell (`build_from` with
  perturbed `[priority.estimate]` config), not a read over the built graph —
  `est_cost` bakes into `base_score` at node mint, so instability is a
  re-derivation, not a re-read.
- **Per-finding substrate tier** (the design decision the signature encodes):
  structural (fork/join/inversion/displacement) ← graph + score map;
  decomposition/plateau ← component maps; provenance (evictions/`Degraded`) ←
  cordage provenance; instability ← rebuild seam.
- **Surface**: `survey --interesting` flag vs a new `findings` verb — decided in
  `/design`. Each finding renders one line: node subset + reason + magnitude.

Initial catalogue (from IMP-241): fork (Y), arm resequencing, order instability,
value inversion, displacement, gating fan-out, join/convergence, provenance
anomalies, plateau exposure.

## Non-Goals

- **No visualisation** — no web changes, no CLI images (kitty/DOT). Text only.
- **The follow-on rendering policy is out of scope**: arc-strip linear view,
  include-by-finding graph inclusion, semantic synthesis (lanes/colour/value
  flow), web sugiyama drop, focus modes. Those are what this probe *validates*,
  not what it builds.
- No changes to the scoring model, partition, or cordage. Read-only over the
  existing engine.
- β-sensitivity findings may ship "starved" (activate as facet coverage grows) —
  acceptable per the card.

## Affected surface

(Finalised in `design.md` code-impact; recorded as `design-target` selectors.)

- `src/priority/findings.rs` — NEW pure engine module: `Finding` enum + `impl` +
  detectors + thresholds.
- `src/priority/order.rs` — NEW pure engine module: `frontier_order` +
  `surviving_seq_predecessors` extracted from `surface.rs` (reused by `next` +
  detectors).
- `src/priority/graph.rs` — extract `build_from_with_cfg` (the β rebuild seam).
- `src/priority/surface.rs` — impure shell `fn findings(root)` + `beta_endpoints`;
  `next` reuses `order.rs`.
- `src/priority/render.rs` — `findings_human` + `findings_json`.
- `src/priority/mod.rs` — `run_findings` dispatch entry.
- `src/commands/cli.rs` — `findings` verb (match arm + members list).

## Risks / Assumptions / Open Questions

- **OQ-1 (surface) — RESOLVED:** new `findings` verb (design D2). Findings are
  aggregate/relational, not per-node rows; fold-into-`survey` is a follow-on.
- **OQ-2 (β semantics) — RESOLVED:** β ≡ `cfg.estimate.skew` (SL-172);
  `est_cost = floor_eps(lower + skew·(upper−lower))`. Estimates DO carry an
  interval (`lower`/`upper`) — the earlier point-value assumption was wrong.
  Sweep = endpoints {0,1} over one scan (design D4).
- **Gotcha** — cordage `Graph` has no public `edge_count`/`node_count`; fork/join
  degree detection iterates `out_edges` per overlay (mem
  `fact.cordage.graph-no-public-edge-count`).
- **Gotcha** — any DP/walk over the SCC-condensed graph needs the explicit
  component-DAG topo order (mem `pattern.priority.scc-condensation-dp-order`).
- **Risk** — provenance reasons (`EvictedEdge`/`CycleDegraded`) are emitted on
  the `explain` path today, not attached to the built graph; the finding layer
  must reach cordage provenance directly. ISS-003 (cordage `explain()`
  foreign-node bug, RFC-007) is adjacent — may bite if the provenance seam is
  shared.
- **Assumption** — probe-grade ambition: prove the findings read as useful, not
  build the production rendering layer.

## Verification / Closure intent

- Pure finding functions with unit coverage over crafted `PriorityGraph`
  fixtures — each catalogued finding detected on a positive fixture, absent on a
  negative one.
- Perturbation-based instability findings verified via the injected rebuild seam
  (deterministic, no disk in the pure layer).
- Text surface produces the one-line-per-finding render; golden/`--json`
  coverage consistent with the existing `survey`/`next` render discipline
  (render source-of-truth in view types, never recomputed in the renderer).
- Closure judged on: does the findings output, run against the live corpus, read
  as more useful than the flat list — the probe question. Design conversation
  records the verdict and whether the follow-on rendering work is warranted.

## Follow-Ups

- Rendering-policy follow-on (arc-strip, include-by-finding, semantic synthesis,
  web) — mint as separate items if the probe validates.
- Fold `explain` into `next`/`survey` (`--why`), coefficient what-if trace —
  sibling RFC-007 (2) concerns.
