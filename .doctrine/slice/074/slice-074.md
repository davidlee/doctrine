# Concept Map Entity + CLI

## Context

Doctrine has a growing entity-relation corpus. SL-072 (map server) and SL-073
(map frontend) will provide interactive visual exploration of that corpus. But
there is no mechanism for *authored* concept maps — curated graphs that express
how a human wants concepts, entities, and relationships to be understood in a
bounded context.

This slice adds a first-class `concept-map` entity kind with a compact
line-oriented DSL (`thing > relationship > other thing`) as the primary
authoring surface. The map is a standalone authored artifact, not the Doctrine
corpus graph itself. It is the future substrate for the web map explorer's
concept-map view and, later, for composing authored edges with hydrated Doctrine
entity edges.

## Scope & Objectives

1. **Entity kind `concept-map`.** New numbered entity kind under
   `.doctrine/concept-map/<nnn>/` with `CM-<nnn>` canonical ids. Template:
   `concept-map-<nnn>.toml` (authored, includes `dsl` payload) and optional
   `concept-map-<nnn>.md` (prose rationale).

2. **TOML schema.** `id`, `title`, `status` (draft | accepted | superseded),
   `description`, `dsl` multi-line string. Plus `[[relation]]` rows for linking
   to other Doctrine entities (specs, ADRs, etc.).

3. **DSL parser.** Parse `source > relation > target` lines. Trim whitespace,
   ignore empty lines, allow `#` comments. Require exactly three non-empty
   segments per line. Return parsed nodes (key + label) and edges (from, rel,
   to, line origin) plus diagnostics.

4. **CLI create.** `doctrine concept-map new <slug> --title <title>` — creates
   the next available concept-map entity with an empty `dsl` block.

5. **CLI list.** `doctrine concept-map list` — tabular listing of known concept
   maps (id, status, title).

6. **CLI show.** `doctrine concept-map show <ID>` — prints metadata and DSL.
   Optional `--edges` and `--nodes` flags for parsed views.

7. **CLI add edge.** `doctrine concept-map add <ID> "Source" "rel" "Target"` —
   appends one DSL line. Rejects empty segments. Warns on duplicate edges or
   likely spelling drift.

8. **CLI remove edge.** `doctrine concept-map remove <ID> "Source" "rel"
   "Target"` — removes the matching DSL line (exact trim match).

9. **CLI rename node.** `doctrine concept-map rename-node <ID> "Old Label" "New
   Label"` — rewrites matching source/target terms across all DSL lines.
   Case-insensitive exact-label match after trimming. Reports count of rewritten
   occurrences. Optional `--case-sensitive` and `--dry-run` flags.

10. **CLI check.** `doctrine concept-map check <ID>` — validates malformed DSL
    lines, empty labels, duplicate exact edges, self-edges (warning), likely
    duplicate concepts, relation spelling drift, and unknown entity-looking refs
    (warning only).

11. **CLI export.** `doctrine concept-map export <ID> --format <fmt>` — renders
    the parsed map. Initial formats: `dot`, `mermaid`, `json`. DOT is the
    primary format (consumable by SL-072's Graphviz bridge and the SL-073
    frontend).

12. **Reader API.** A reusable reader/projection layer independent of CLI
    concerns: `AuthoredConceptMap` (from TOML) → `ParsedConceptMap` (nodes,
    edges, diagnostics) → format renderers. This is the API the future web
    endpoints will call.

## Non-Goals

- Web explorer integration (SL-073 already covers the frontend; concept maps
  will be consumed as a data source in a follow-up slice)
- `focus` projection (neighbourhood query at depth — follow-up slice)
- Graphviz SVG rendering (SL-072 owns the Graphviz bridge; export produces DOT
  text, which SL-072 can already render)
- `serve` command (SL-072 owns `map serve`; concept map serving composes with
  it later)
- Doctrine corpus graph composition (entity-ref resolution in concept-map
  nodes — future slice)
- Rich alias system, visual layout editor, collaborative editing
- Semantic relation ontology
- Mandatory DAG enforcement (cycles are allowed)

## Affected Surface

- **New:** `src/commands/concept_map.rs` — CLI entry and command handlers
- **New:** `src/concept_map.rs` — entity kind, Kind constant, reader/parser,
  pure rendering, CLI-agnostic projection
- **New:** `install/assets/templates/concept-map.toml` — entity scaffold
  template
- **New:** `install/assets/templates/concept-map.md` — optional prose scaffold
- **Existing consumers:** `src/entity.rs` (Kind-agnostic engine — read-only
  usage), `src/main.rs` (new `concept-map` subcommand)
- **Cargo.toml:** no new dependencies expected

## Risks

- **DSL parsing fragility.** The `>` delimiter may appear in node labels.
  Mitigation: first-pass split on ` > `, then trim; accept that `>` in labels
  is ambiguous. Document the convention.
- **Node rename scope.** Case-insensitive matching could produce surprising
  rewrites when labels differ only by case. Mitigation: `--case-sensitive` flag
  for explicit control.
- **Unknown entity refs.** Entity-looking labels like `PRD-010` won't resolve in
  v1. Mitigation: check warns but doesn't fail. Future composition adds
  resolution.
- **Export format fidelity.** DOT and Mermaid have different expressiveness.
  Mitigation: DOT is primary; Mermaid and JSON are derived from the same parsed
  model.

## Verification / Closure Intent

- `doctrine concept-map new <slug>` creates a valid entity with empty DSL
- `doctrine concept-map list` shows all concept maps
- `doctrine concept-map show CM-001` prints metadata and DSL
- `doctrine concept-map add CM-001 "A" "relates" "B"` appends the line
- `doctrine concept-map remove CM-001 "A" "relates" "B"` removes it
- `doctrine concept-map rename-node CM-001 "Old" "New"` rewrites all
  occurrences
- `doctrine concept-map check CM-001` reports diagnostics
- `doctrine concept-map export CM-001 --format dot` produces valid DOT
- Round-trip: add edge, show, check, export all agree
- `cargo test` passes (unit + integration)
- `cargo clippy` zero warnings
- `just gate` passes

## Design Approach

The concept map is a simple authored artifact. No graph database. No index
caches. No cordage integration in v1. The reader parses the DSL on every
read — maps are small enough that this is benign. If performance matters later,
add a parse-cache behind the reader.

The reader is pure over TOML text — no disk, clock, git, or root. The CLI shell
reads the file and passes it in. This preserves the pure/imperative split and
makes the reader testable without filesystem fixtures.

Node keys are derived from labels by lowercasing and collapsing whitespace to
hyphens: `"User Story"` → `"user-story"`. Canonical node identity is by key, not
label. This supports future entity resolution (`PRD-010` stays
`PRD-010`-derived but resolves to the Doctrine entity).

## Follow-Ups

- `focus` command: neighbourhood projection at configurable depth
- `serve` integration: concept maps as a data source in the SL-072 map server
- Entity-ref resolution: `PRD-010` in a concept map resolves to the Doctrine
  entity
- Relation rename: `rename-rel` command
- Graphviz SVG export: consume DOT via SL-072's Graphviz bridge
- Web explorer concept-map view: SL-073 consumes the reader API
