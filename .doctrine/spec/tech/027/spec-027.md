# SPEC-027: Graph projection and CLI emitter

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

This component is the **presentation-neutral projection of the entity graph and
its terminal consumption surface**: `CatalogGraph` (`src/catalog/graph.rs`) —
the pure read model that turns a hydrated `Catalog` into nodes, edges, and
units — plus the pure DOT emitter over it (`src/catalog/dot.rs`) and the
`doctrine graph` verb that wires them to stdout (`src/commands/graph.rs`). It
realises **PRD-016**'s semantic graph view on the pipe-composable CLI surface,
the consumption gap RFC-001 names and SL-226 closed.

It is a **component** of the entity engine container (**SPEC-004**): the
projection's input is `catalog::hydrate::Catalog`, and everything upstream of
that — the corpus scan, hydration, and entity identity — is the parent's. It is
**not** a component of the web explorer (SPEC-025), and the distinction is
load-bearing: `CatalogGraph::from_catalog` has **two** consumers — this spec's
`doctrine graph` verb and SPEC-025's `/api/graph` — so nesting it under the web
container would make the terminal a consumer of the web explorer's internals.
SPEC-025 says as much itself: it *consumes* `CatalogGraph` and disclaims owning
it. The two specs are peers descending from one capability, joined by an
interaction edge, not by containment.

**Consumes, does not own.** The relation vocabulary, roles, and tier partition
this projection carries are **SPEC-018**'s; they arrive already validated on
`CatalogEdge` and are projected verbatim, never re-derived or re-validated here.
Priority, actionability, blocker state, and their explanations are **PRD-011**'s
via SPEC-001, and this component neither computes nor renders them — the
`doctrine graph` verb emits the *semantic* graph only. Estimation and value
units pass through as an opaque top-level block sourced from SPEC-020's facet.

**Boundary against PRD-011's CLI surfaces.** `survey`, `next`, `explain`,
`blockers`, and `inspect` are terminal renderings of PRD-011's computation and
stay with it (PRD-016 §2, OQ-003 settled). This spec's CLI claim is the semantic
graph emitter alone; a future `--view actionability` altitude on the same verb
would be a boundary change, not a free extension.

## Responsibilities

Mirrors the structured `responsibilities` list.

### The projection: `CatalogGraph`

`CatalogGraph::from_catalog` is a **pure** projection — no disk reads, no
`cordage` dependency, no clock — which is what makes the whole read model
unit-testable without a filesystem and cheap enough for a one-shot CLI
invocation to build from a cold scan. It yields three fields:

- `nodes` — a `BTreeMap` keyed by `CatalogKey` (a numbered entity key, or a
  memory uid), each node carrying only what a presentation surface needs: title,
  optional status, kind label, and optional memory type. Deterministic key order
  is a property of the map, not of a sort applied downstream.
- `edges` — a **flat list**, not an adjacency structure. Adjacency is rebuilt
  on demand by the operations that need it.
- `units` — the project-wide estimation/value display units, projected verbatim
  from the source catalog so that every consuming surface resolves one unit
  block rather than each re-deriving its own.

**Dangling targets survive the projection.** An edge whose target is an
unresolved canonical reference or unvalidated free text stays in `edges` with no
corresponding node. This is deliberate: a broken reference is information, and
dropping it would make the projection quietly lie about the corpus. Every
consumer must therefore handle an edge with no target node — the DOT emitter by
rendering a ghost node, a JSON consumer by tolerating a target that indexes
nothing. The complementary asymmetry: outbound traversal includes dangling
edges; inbound traversal excludes them, because an edge with no resolved target
cannot point *at* anything.

### The projection algebra

Four filters, each **consuming `self` and returning `Self`**, so a pipeline is
an ownership chain rather than a mutation sequence:

- **`filter_kinds`** — keep nodes whose uppercase kind prefix is in the given
  set; drop incident edges. Prefixes arrive already uppercase and already
  validated; validation is the command layer's job, not the engine's.
- **`filter_label`** — keep edges whose label name matches; **never drops a
  node**. A bare `references` match keeps roled `references` edges, so filtering
  by the label does not require enumerating its roles.
- **`exclude_memory`** — drop every memory-keyed node and its incident edges.
  Memory is off by default on the CLI surface.
- **`drop_isolated`** — a *terminal* operation: drop nodes with zero incident
  edges, where an edge incidents a node if it sources from it or resolves to it.

Node-dropping filters share one internal rule: an edge survives only if its
source survives **and**, when its target is resolved, its target survives.
Dangling-target edges live and die with their source alone.

**`neighbourhood(focus, depth)`** is the ego-view: an undirected,
breadth-bounded BFS returning the owned subgraph within `depth` hops of the
focus. It traverses both directions — an entity's neighbourhood includes what
points at it, not only what it points to — by building outbound and inbound
adjacency in one pass over the edge list. **Boundary nodes are included but not
expanded**: a node at distance `depth` appears in the result, but its incident
edges are not collected. Without that rule the rendered ego-view would show
half-edges to nodes outside the bound.

### The DOT emitter

`catalog::dot::render` is the second pure layer: a projected graph and an
optional focus in, a DOT string out, with no filesystem access and no dependency
on an external renderer. Its contract is **determinism** — byte-identical on
repeat calls for the same input, which is what makes its output diffable and
testable rather than merely visually inspectable. Determinism comes from two
sources: node order is the `BTreeMap`'s key order, and edges are sorted on an
explicit total order (source, display label, target, then original index as the
final tiebreak).

The emitted document carries: styled real nodes (fill/font by kind prefix, box
shape, a tooltip of id, title, kind, and status); **ghost nodes** for each
distinct unresolved or unvalidated target, dashed and prefixed to keep them
distinguishable from real ids; and labelled edges coloured by relation label,
with roled `references` edges displayed as `references(role)` rather than
collapsed to the bare label. The focus node, when given, is distinguished by
pen weight only — never by shape or colour, which already encode kind.

Styling is a **structural port** of the web explorer's frontend DOT builder, not
a pixel-parity clone. Two renderings of one graph is an accepted, bounded
divergence risk; converging them into one shared emitter is not a goal of this
spec.

### The command surface

`doctrine graph [FOCUS]` is a thin ADR-001 shell holding **no projection
logic**. Its order of operations is itself part of the contract, because the
filters do not commute:

1. Scan and project the full graph.
2. Validate every `--kind` against the kind registry (plus the memory pseudo-kind),
   failing with the legal set rather than silently ignoring an unknown prefix.
3. Resolve `FOCUS` to a `CatalogKey` — a memory key by prefix, otherwise a
   canonical entity reference.
4. **Diagnose focus exclusion before projecting.** A focus absent from the
   corpus, a focus filtered out by `--kind`, and a memory focus without
   `--include-memory` are three distinct named errors. Without this check each
   would surface as an empty graph — the failure mode where the tool reports
   "nothing here" when the truth is "you excluded what you asked for".
5. Apply the pipeline: kinds, then memory exclusion, then label, then either the
   focus neighbourhood or (label-filtered, unfocused) `drop_isolated`.
6. Emit — DOT via the emitter, or JSON via the projection's own serialisation.

**One graph contract, two surfaces.** `--format json` serialises the same
`CatalogGraph` that `/api/graph` serves. The shared shape is the point: an agent
piping `doctrine graph --format json` and a frontend fetching `/api/graph` parse
one payload, and a change to the projection's serialisation is a change to both
surfaces at once.

## Concerns

- **Cost is a full scan per invocation.** The verb has no cache and no
  incremental path; every call hydrates the whole corpus. This is acceptable
  because the same scan already backs `map serve` start-up, but it sets the
  ceiling on corpus size before the CLI surface needs an index.
- **Divergence between the Rust and TypeScript DOT emitters.** Two independent
  renderings of one graph will drift under maintenance. Mitigated by keeping the
  Rust emitter structural and explicitly *not* chasing pixel parity; the JSON
  contract, not the DOT output, is the surface where equivalence is guaranteed.
- **Whole-corpus density.** Unfocused, unfiltered DOT over a mature corpus can
  be too dense to read. The projection algebra is the mitigation — focus, depth,
  kind, and label are the intended entry points, not an afterthought.
- **Silent-empty failure.** The pipeline can legitimately produce an empty graph
  (an over-narrow filter). Focus exclusion is diagnosed explicitly precisely
  because it is the case where empty means *mistake*, not *no data*.
- **Kind-set coupling.** `--kind` validates against the shared kind registry; a
  new entity kind becomes filterable with no change here, but a kind prefix
  colliding with the memory pseudo-kind would be ambiguous at this surface.

## Hypotheses

- **The projection is cheap enough to be one-shot.** Hydration and projection
  cost is dominated by the corpus scan that `map serve` already pays at
  start-up, so a CLI verb can rebuild from cold without a cache. Falsified if
  invocation latency becomes the reason to reach for the web explorer.
- **Presentation-neutrality holds under a second consumer.** The read model
  carries no CLI- or web-specific shaping, so a third surface should be able to
  consume it without widening it. The two current consumers are the evidence;
  the hypothesis is that a third does not force a fork.
- **A structural DOT port is sufficient.** Terminal consumers want the graph's
  shape piped into Graphviz, not the explorer's visual identity reproduced
  exactly. Falsified if users routinely post-process the DOT to restore
  styling the frontend already has.

## Decisions

- **D1 — The projection parents to the entity engine, not the web explorer.**
  `CatalogGraph` has two consumers of equal standing. Parenting it under
  SPEC-025 would invert the dependency for the CLI; parenting it at SPEC-003
  would skip the container rung. SPEC-004 owns the `Catalog` it projects.
- **D2 — Purity is a contract, not an implementation detail.** No disk, no
  cordage, no clock in either the projection or the DOT emitter. This is what
  keeps the read model testable without fixtures on disk and reusable by any
  surface, and it is the invariant SPEC-025 relies on when it calls the
  projection pure.
- **D3 — Dangling edges are projected, not dropped.** An unresolved or
  unvalidated target stays in the edge list without a node. Consumers absorb the
  asymmetry (the DOT emitter renders a ghost node); the alternative silently
  understates the corpus's real reference state.
- **D4 — Filters consume `self`.** The algebra is an ownership chain, which
  makes an accidental mid-pipeline reuse of a partially filtered graph a compile
  error rather than a subtle wrong answer.
- **D5 — Boundary nodes are included but not expanded.** The ego-view shows what
  sits at the edge of the requested depth without dragging in that node's own
  neighbourhood, so `--depth` bounds the result predictably.
- **D6 — Focus exclusion is an error, never an empty graph.** Distinguishing
  "not in the corpus", "excluded by `--kind`", and "memory needs
  `--include-memory`" costs three messages and removes the tool's worst failure
  mode.
- **D7 — `--format json` reuses the projection's serialisation.** The CLI does
  not define a second payload shape. One graph contract serves the terminal and
  the web explorer, and neither can drift without the other noticing.
- **D8 — `GraphFormat` is a dedicated enum, not the shared listing format.** The
  emitter's formats are `dot | json`; the corpus-listing format vocabulary is
  `table | json`. Sharing the type would force one surface to carry a variant it
  cannot honour.

## Open Questions

- OQ-1 — Should the verb grow further view altitudes (`--view actionability`,
  `--view coverage`, per SL-226's follow-ups)? An actionability altitude would
  render PRD-011's derived view on the terminal, which PRD-016 §2 currently
  assigns to PRD-011. The option surface was deliberately left open for it, but
  taking it is a product-boundary decision, not an implementation one.
- OQ-2 — Should the Rust and TypeScript DOT emitters converge on one source of
  truth (a shared emitter, or the frontend consuming server-rendered DOT for the
  semantic view as it already does for concept maps)? Deferred while the
  divergence stays cosmetic.
- OQ-3 — Should additional diagram formats (mermaid, d2) be emitted here or
  live behind a separate interchange surface? PRD-016 §2 demotes *static file
  interchange for external tools* while explicitly preserving this verb's
  on-demand stdout emission; where a second on-demand format falls against that
  line is unsettled.
