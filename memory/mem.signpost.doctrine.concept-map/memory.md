# Doctrine concept map signpost

Concept maps (CM kind) are DSL-driven relationship diagrams that document
the architecture of the project's domain model. They live as authored
artifacts under `.doctrine/concept-map/nnn/` and can be exported to DOT,
Mermaid, or JSON for visualisation.

## When to create a concept map

Create a concept map when you need to document how entities relate to each
other at the domain level — boundaries, dependencies, data flow, or
taxonomic hierarchies. Concept maps are the "why" behind relation edges,
not the edges themselves.

## CLI

The CLI is the source of truth: `doctrine concept-map --help`, never guess.
Key verbs: `new`, `list`, `show <ID>`, `check`, `add`, `remove`,
`rename-node`, `export`.

See [[concept.doctrine.entity-engine]] for the underlying relation model,
[[signpost.doctrine.file-map]] for the directory layout,
and [[signpost.doctrine.relating-entities]] for CLI relation authoring.
