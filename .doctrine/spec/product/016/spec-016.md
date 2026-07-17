# PRD-016: Graph exploration

<!-- Reference forms: entity ids padded (REQ-059, ADR-004); doc-local refs bare
     (OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## 1. Intent

Doctrine's entity graph — the semantic layer of entities and outbound
relations (ADR-004/ADR-010/ADR-016, SPEC-018) and the derived actionability
layer PRD-011 computes over it — is correct but, on its own, invisible. An
agent or human cannot *see* the graph; they can only query slivers of it one
command at a time. RFC-001 names the pattern directly: doctrine's graph value
is gated on consumption surfaces, not on deeper internal modelling. RFC-007
sharpens it further for the actionability layer specifically: the calc is
correct, but "both ends are dark" — the graph has no faithful rendering, and
`next`/`survey` present as an opaque, flatly ordered list.

This capability is the graph's outward, explorable face: the **semantic
graph view** (what relates to what), the **actionability view** (what is
workable, blocked, or consequential, per PRD-011's derived ordering), the
**concept-map view** (a human-authored narrative diagram over a named
subset), and the **web explorer** (`doctrine map serve`) that renders all
three and lets an agent fetch the same read models as JSON. Its value is
legibility: a reviewer, an operator, or an agent can walk the corpus
visually or programmatically, understand why an item sits where it does,
and trust what they see without re-deriving it from raw TOML.

## 2. Scope

In scope:

- The **semantic graph view** — the entity+relation graph (nodes, edges,
  estimation/value units) as an explorable, focus-and-depth-bounded
  neighbourhood, kind-filterable, deep-linkable.
- The **actionability view** — a navigable rendering of PRD-011's derived
  priority/actionability ordering, presented as a distinct view from the
  semantic graph rather than folded into it.
- The **concept-map view** — rendering and structural editing of a
  human-authored DSL diagram over a named node/edge subset, with its own
  heuristic diagnostics, independent of the live relation graph.
- The **web explorer** (`doctrine map serve`) — the local HTTP server and
  bundled frontend that composes the three views: search, kind filtering,
  entity detail (rendered markdown body), relationship table, refresh, and
  a health probe.
- The **JSON read/write contract** each view exposes, so an agent consumes
  the same graph a human browses rather than a UI-only artefact.

Out of scope:

- Computing priority, actionability, blockers, or explanations — that
  synthesis is PRD-011's; this capability consumes its output and renders
  it, never re-derives or restates the policy.
- Authoring or validating relation edges on the semantic graph — SPEC-018
  owns the `link`/`unlink` write seam and its validation policy; the
  explorer's semantic and actionability views are read-only.
- The spec/requirement corpus itself and its authoring workflow — owned by
  PRD-002 (product specs) and PRD-012 (technical specs); this capability
  is itself an instance of their governance, not a restatement of it.
- Cross-validating the concept map against the live relation graph
  (drift/soundness detection) — the concept-map view renders and edits the
  authored DSL as its own narrative artefact; bridging it to the relation
  graph is an unbuilt, deliberately separate gap (IDE-015, RFC-001 surface
  (b)).
- Static graph file interchange (GraphML/Cypher/DOT-file export) for
  external tools — demoted to on-demand by RFC-002; this capability's
  interchange surface is the navigable JSON/SVG contract, not a file
  export pipeline.

Boundary: this capability owns the **outward, explorable presentation** of
doctrine's graph — what the three views show, who reads them, and how they
compose in one explorer. It does not own graph derivation, relation
authoring, or spec-corpus governance; each of those is computed, captured,
or governed upstream and merely rendered here.

## 3. Principles

- **The explorer renders; it does not compute.** Actionability ordering,
  blocker state, and explanations originate in PRD-011's derived surface;
  the web explorer and CLI display them faithfully and never re-derive or
  approximate the policy client-side.
- **One graph, three honest views.** The semantic graph, the actionability
  graph, and the concept map answer different questions and stay visually
  and structurally distinct — toggled, not merged into one omniscient
  diagram that would blur what is authored, what is derived, and what is
  narrative.
- **Every view a human sees, an agent can fetch.** Each view's read model
  is available as a stable JSON (or plain-text) HTTP response independent
  of the rendered HTML, so agent consumption never requires scraping a UI.
- **The concept map is authored narrative, not proven truth.** It renders
  what a human asserted about a subset of the graph; its diagnostics are
  heuristics over its own DSL, not a cross-check against the relation
  graph, until such a bridge exists as its own capability.
- **Local-first, no hosting dependency.** Exploration requires no external
  network egress, deployment, or build step at run time — the frontend
  ships embedded in the binary and the server binds loopback-only.
- **Navigation state is shareable.** Focus, depth, and view mode resolve
  from the URL, so a specific vantage point in the graph is a link, not a
  sequence of clicks to reproduce.

## 4. Requirements

The functional and quality requirements this capability must satisfy are
recorded as requirement entities and appear under the synthesized
Requirements section below. This section carries only the constraints and
invariants that bound every valid implementation.

Constraints:

- The explorer must not implement or duplicate PRD-011's actionability
  scoring, blocker resolution, or explanation logic; it consumes the
  derived view PRD-011 already computes.
- The HTTP server must bind loopback-only; no view or API route may be
  reachable from a non-local origin by default.
- Frontend assets must be embedded in the binary at build time; no view
  may depend on a separately deployed or hosted asset bundle.
- Concept-map mutation must go through the same DSL-editing seam the CLI
  uses (`doctrine concept-map add/remove/rename-node`), never a bespoke
  write path that could diverge from it.
- The semantic, actionability, and concept-map read models must each be
  independently fetchable as JSON without requiring the HTML shell.

Invariants:

- Every request handler renders from the current in-memory read models
  (catalog graph, priority graph, concept map); a view never re-scans the
  corpus on the read path — only an explicit refresh does.
- The three data stores backing the semantic and actionability views are
  replaced atomically together; no request observes one store refreshed
  ahead of the other two.
- A concept-map mutation is applied only when its optimistic-concurrency
  base hash matches the map's current DSL hash; a stale write is refused,
  never silently overwritten.
- The explorer never authors or mutates the relation graph, priority
  graph, or any entity's lifecycle state; its sole write surface is the
  concept-map DSL.

## 5. Success Measures

- An agent or human can name an entity and see its immediate relation
  neighbourhood rendered, without hand-composing an `inspect` query chain.
- An operator asking "what should I look at next, and why" can see the
  actionability ordering as a navigable diagram, not only a flat CLI list.
- Switching between the semantic and actionability views preserves the
  current focus, so a reviewer never loses their place changing questions.
- A human's hand-authored concept map stays editable and inspectable
  through the same explorer, without a separate tool or workflow.
- Every view's underlying data is fetchable as JSON, so an agent's
  consumption path does not require parsing rendered HTML or SVG.
- The explorer runs with zero external setup beyond the doctrine binary
  and (for semantic-view SVG rendering) a local Graphviz install, whose
  absence is surfaced by the health probe rather than a silent blank pane.
- A shared link (URL with focus/depth/view-mode) reproduces the same
  vantage point for a second viewer without narrated reproduction steps.

Acceptance gates:

- Focusing an entity and selecting a depth renders exactly that
  neighbourhood's nodes and edges in the semantic view.
- Toggling to the actionability view re-renders the same corpus as a
  derived-order diagram without dropping the active focus id.
- A concept-map edit applied through the explorer round-trips: reloading
  the map shows the mutation, and a stale-hash retry is rejected.
- Killing network access to the host does not degrade the explorer, since
  assets and API routes are all served from the local process.
- `/api/graph`, `/api/survey`, and `/api/concept-map/{id}` each return
  valid JSON usable without the HTML shell.

## 6. Behaviour

Primary flow — open the explorer: an operator runs `doctrine map serve`.
The server scans the corpus once, builds the catalog graph and the
priority graph, binds a loopback port, and (unless suppressed) opens a
browser at the map URL, optionally pre-focused on an entity and depth
passed on the command line.

Primary flow — explore the semantic graph: the explorer renders the
sidebar's search box, kind-filter checkboxes, and entity list from the
fetched graph. Selecting or focusing an entity requests a server-rendered
DOT→SVG diagram of that entity's neighbourhood at the active depth (0–3),
alongside a relationship table and, on hover, a detail pane. Kind filters
narrow the entity list and relationship table client-side without a new
fetch.

Primary flow — switch to the actionability view: selecting the
Actionability toggle re-renders the current focus against the derived
priority/actionability view instead of the raw relation graph, laid out
client-side as a directed-acyclic diagram rather than the server's
Graphviz rendering. The active focus, depth, and kind filter persist
across the switch; the view never issues a new priority computation, only
a new render of the same fetched view.

Primary flow — inspect an entity's markdown: focusing or hovering a node
requests that entity's rendered markdown body and displays it in the
markdown pane, so the graph diagram and the entity's prose sit side by
side.

Primary flow — explore and edit a concept map: focusing a concept-map
entity switches the graph area to that map's own diagram, parsed from its
DSL, with its diagnostics panel and edge table. An edit (add/remove edge,
rename node, relabel a relation) posts a mutation carrying the map's last
known DSL hash; the server applies it if the hash still matches and
returns the updated map, or a conflict if the map changed underneath the
editor.

Alternate flow — refresh: an operator triggers a refresh action. The
server re-scans the corpus and rebuilds the catalog graph and priority
graph together, replacing both atomically so no subsequent request sees a
partially updated pair.

Diagnostic flow — health check: a client may query the health endpoint,
which reports whether the SVG-rendering tool is available and whether the
in-memory graph is non-empty, so a broken local Graphviz install or an
empty scan surfaces as a named condition rather than a blank pane.

Edge cases and guards: an entity with no relations still resolves to a
single-node neighbourhood rather than an error; a request for an unknown
entity id or an unembedded asset path returns a self-describing not-found
response rather than a generic failure; a concept-map mutation against a
stale hash is refused rather than silently applied over a concurrent edit;
an oversized DOT payload to the render endpoint is rejected before it
reaches the external renderer.

## 7. Verification

Verification confirms that each view renders the correct read model for
its scope, that the three views stay independently navigable and
consistent under refresh, and that the explorer never crosses into
PRD-011's computation or SPEC-018's write seam.

Semantic-view correctness is proven by confirming the graph endpoint
emits the same nodes and edges the catalog scan produces, and that a
focus-and-depth request renders exactly the expected neighbourhood.
Actionability-view correctness is proven by confirming its endpoint
returns PRD-011's derived view unmodified and that the explorer performs
no independent scoring. Concept-map correctness is proven by confirming a
mutation round-trips through the same DSL-editing seam the CLI exposes,
that a stale base-hash write is refused, and that diagnostics reflect the
map's own DSL heuristics rather than the live relation graph. Refresh
atomicity is proven by confirming a concurrent reader never observes a
freshly rebuilt store paired with a stale one. Local-only serving is
proven by confirming the bound listener accepts loopback connections only
and that every asset route resolves from the embedded bundle with no
external fetch. Boundary preservation is proven by confirming this
capability's test suite contains no priority-scoring or relation-write
assertions — those remain PRD-011's and SPEC-018's respectively.

Where a check must reference a specific obligation, cite the durable
requirement entity (`REQ-NNN`), never a mobile membership label. Coverage
of the functional and quality requirements is tracked against those
entities, not duplicated here.

## 8. Open Questions

- OQ-001 — Should the concept-map view eventually cross-validate against
  the live relation graph (drift/soundness detection), and if so, is that
  a new channel of this capability or the separate bridge capability
  IDE-015 already names? RFC-001 sequences it last and speculative;
  this spec deliberately renders the concept map as authored narrative
  only until that boundary is decided.
- OQ-002 — Should static graph file interchange (GraphML/Cypher/DOT-file
  export for external tools like Gephi/Neo4j) become a channel of this
  capability if a concrete external-tool need appears, per RFC-002's
  on-demand deferral, or does it belong to a separate interchange spec?
- OQ-003 — Should `next`/`survey`'s CLI text rendering (RFC-007 workstream
  2 — folding `explain`'s decomposition inline, a `--why` flag, a
  what-if/trace mode) be governed by this capability, since it is graph
  legibility for the same actionability view, or does it stay scoped to
  PRD-011 as a CLI-surface concern of the computation it renders? The
  boundary drawn here (§2) treats the *web* actionability view as this
  capability's and leaves the *CLI* rendering question open.
- OQ-004 — What is this capability's C4/product-altitude boundary against
  a future tech spec for `map_server`/`web/map`: one tech spec spanning
  both, or a container-level split (server vs. frontend)? Blocks the
  downstream `/spec-tech` authoring CHR-046 also scopes.
- OQ-005 — Should the actionability view eventually render epistemic
  gating (RFC-007 workstream 3 — records blocking dependents) once
  populated, or is that automatic once PRD-011's derived view includes
  it, requiring no change here? Affects whether this spec needs a
  requirement for gating-edge legibility or inherits it for free.
