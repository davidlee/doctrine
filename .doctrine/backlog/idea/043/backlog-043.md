# IDE-043: CLI graph emission and terminal ego-view

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

A CLI consumption surface for the entity/relation graph, in the RFC-001 sense
(graph value is gated on consumption surfaces):

- `doctrine graph --dot` — emit the semantic graph (or a filtered projection)
  as DOT on stdout, composable with the graphviz toolchain:
  `doctrine graph --dot | gvpr '<select/annotate a view>' | dot -Tsvg`.
- **Ego-view**: a verb to render one entity plus its nearest *n* relations
  (focus + depth bound, like the web explorer's neighbourhood view) and display
  it inline in a terminal image viewer (kitty graphics protocol / viu / sixel).
- Bonus: d2 / mermaid output formats.
- Bonus: higher-altitude views — C4-style, product-altitude, and a specs
  coverage map (what's governed vs dark).

## Relationship to prior art

- `concept-map export --format dot|mermaid|json` already exists but only for
  authored concept-map entities — not the live relation graph.
- The web explorer (PRD-016, SL-072/SL-073, `map serve`) already computes a
  focus-and-depth-bounded semantic-graph projection and renders it via
  Graphviz server-side (SL-094) — a CLI emitter is largely re-plumbing an
  existing projection to stdout, not new graph modelling.
- PRD-016 §2 demotes *static graph file interchange for external tools*
  (GraphML/Cypher/DOT-file export) to on-demand per RFC-002. This idea is
  distinguishable: it is an in-workflow, agent/human-facing consumption
  surface (pipe-composable DOT + terminal rendering), not an interchange
  pipeline — but adopting it means revisiting that PRD-016 boundary sentence.
- The specs-coverage-map bonus overlaps `/spec-coverage-assessment` (which
  produces a prose artefact); a graph rendering of the same could share its
  underlying model.
