# CHR-046: Author product + tech specs and requirements for the graph subsystem (semantic + actionability graph)

## Motivation

Doctrine's graph functionality — the **semantic graph** (entities + outbound
relations, ADR-004 / ADR-010 / ADR-016, SPEC-018) and the **actionability graph**
(priority/scoring engine, PRD-011 / SPEC-001 / ADR-015 / ADR-017 / ADR-018),
plus the web explorer that renders both (`web/map/`) — has grown across many
slices and ADRs but is **not coherently governed by a spec set that describes the
graph as a capability in its own right.** Coverage is scattered: the priority
engine has a PRD+tech spec, the relation contract has a tech spec, but the graph
*as the user-/agent-facing thing* (what it shows, what it's for, how the views
compose) has no durable product intent, and the web explorer has no tech spec.

`bundle-graph-context.local.sh` was written to assemble exactly this context for
external spec-drafting — evidence the material is ready to be pulled into
first-class specs.

## Scope

Put spec-coverage discipline into practice on the graph:

1. Assess current coverage and choose spec boundaries (product altitude + C4
   level) for the graph subsystem — ideally using [[IMP-295]]'s support if
   available; otherwise do it by hand and feed findings back to IMP-295.
2. Author / extend the **product spec(s)** — the durable product intent for the
   graph: what it represents, who consumes it, the views (semantic /
   actionability / concept map) and their purpose, success measures.
3. Author / extend the **tech spec(s)** — architecture and mechanism of the graph
   builders and the web explorer, downstream of the product spec, sitting
   correctly relative to the existing SPEC-001 / SPEC-018 boundaries.
4. Capture / place the attendant **requirements**, related to the existing corpus
   (needs/refines links to PRD-011, SPEC-001, SPEC-018, ADR-004/010/015/016/017/018).

Likely spawns one or more slices; this item is the intake/driver.

## Neighbours & inputs

- [[IMP-295]] — the systemic spec-coverage support this exercises; sequence after
  it where its skill/tooling helps, but do not block on it.
- `bundle-graph-context.local.sh` — the assembled context pack (gitignored).
- Existing coverage: PRD-011, SPEC-001, SPEC-018, PRD-014, SPEC-020,
  ADR-004/010/015/016/017/018; web source under `web/map/`.
- `IDE-015` — bridge concept map to relation graph (a graph-view gap the specs
  should acknowledge).
- [[CHR-024]] — relation-model design review overlaps the semantic-graph half;
  coordinate boundaries so the two don't duplicate.
