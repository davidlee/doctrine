# Implementation Plan SL-074: Concept Map Entity + CLI

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

The design introduces a first-class `concept-map` entity kind (CM-NNN) with a
compact line-oriented DSL (`Source > relationship > Target`) as the primary
authoring surface. The concept map is a standalone authored artifact — not a
hydrated corpus projection — that ships with CLI create/list/show/mutate/check/
export verbs and a reusable pure reader/parser/export API for future web
consumers (SL-072, SL-073). The work spans one new source file
(`src/concept_map.rs`), two scaffold templates, and CLI wiring into `main.rs`.

## Sequencing & Rationale

Four phases, strictly bottom-up. Each phase depends on everything before it,
and no phase touches a writable seam introduced by a later phase.

- **PHASE-01 (entity kind, scaffold, list/show)** is the bootstrapping surface:
  the kind is registered, templates are scaffolded, and the read-only CLI verbs
  (new/list/show) exist so an agent can create and inspect concept maps before
  any DSL machinery. The RELATION_RULES registration lands here so `doctrine
  link CM-001 …` is accepted from day one — no later phase needs to revisit it.
  Show emits raw DSL and structured metadata; the parsed edge/node tables are
  deferred to PHASE-02, but the flags are parsed here so the CLI contract is
  stable.

- **PHASE-02 (DSL parser + check)** builds the entire pure layer: `parse_dsl`
  turns the multiline string into nodes, edges, and diagnostics; `derive_node_key`
  normalises labels; `check` adds the heuristic diagnostics (SimilarNodeLabel,
  RelationDrift, CanonicalNodeCollision, EntityRefLike, SelfEdge) powered by a
  local Levenshtein implementation. This phase also extends `run_show` to render
  the parsed edge/node tables when `--edges`/`--nodes` are passed. It runs
  second because the mutation verbs (PHASE-03) need `parse_dsl` for duplicate
  detection and rename segment matching, and the export verbs (PHASE-04) need
  `ParsedConceptMap`. The pure functions are testable entirely from string
  fixtures — no filesystem or CLI integration needed for the core logic.

- **PHASE-03 (CLI mutations — add, remove, rename-node)** is the mutation
  surface: `run_add`, `run_remove`, `run_rename_node`. It depends on PHASE-02's
  `parse_dsl` for duplicate detection (add) and segment matching (rename), and
  on `toml_edit` (already in the dep tree) for edit-preserving TOML round-trips
  through `get_dsl`/`set_dsl` helpers. The `toml_edit` round-trip must preserve
  non-DSL content (`[[relation]]` rows, metadata fields) byte-identically
  modulo the intended DSL change. Rename operates on parsed segments, not naive
  substring replacement — it splits on `" > "`, matches the full trimmed
  segment text, and rewrites only that segment.

- **PHASE-04 (export renderers — DOT, Mermaid, JSON)** is the output surface.
  All three renderers are pure functions over `ParsedConceptMap`, making them
  trivially testable without filesystem fixtures. DOT is the primary format
  (consumed by SL-072's Graphviz bridge); Mermaid and JSON are secondary. Nodes
  and edges are sorted for deterministic output. The DOT renderer escapes
  special characters; the Mermaid renderer uses synthetic node ids to avoid
  collision with Mermaid reserved words. Empty maps produce valid output in all
  formats.

**Why these boundaries.** The phases isolate the pure/impure split: PHASE-02
is entirely pure (testable from strings), PHASE-03 adds the impure mutation
shell over `toml_edit`, and PHASE-04 is pure again (the CLI `run_export` is a
thin shell that reads TOML and calls a pure renderer). PHASE-01 is the
impure bootstrapping that establishes the entity on disk — it must come first
because every later phase reads from disk. The order is forced: no phase can
leapfrog its dependency.

**File-disjoint writable seams.** PHASE-01 touches `src/concept_map.rs`
(kind + scaffold), `src/main.rs` (Command enum + dispatch), `src/relation.rs`
(RELATION_RULES), and `install/assets/templates/`. PHASE-02 extends
`src/concept_map.rs` with pure parser + check + Levenshtein. PHASE-03 extends
it with mutation helpers + run_* handlers. PHASE-04 extends it with renderers
+ run_export. All phases grow the same file, but each phase's functions are
disjoint — the plan sequences them to avoid merge conflicts if parallelised,
but the single-file growth pattern strongly favours serial execution.

## Notes

- No new crate dependencies. The local Levenshtein (~20 lines) is built
  in PHASE-02. `toml_edit` is already in the dep tree.
- The `dsl` field is scaffolded as a multiline literal (`'''\n'''`) from day
  one (PHASE-01 template) — the first `add` appends without reformatting a
  basic string.
- Relation rules registration (PHASE-01 EX-7) follows the existing
  RELATION_RULES table pattern: a `RelationRule` entry with sources including
  `CONCEPT_MAP_KIND` and a label like `GovernedBy` (already exists) or a new
  `Contextualizes` label. If a new label is needed, it must be added to the
  `RelationLabel` enum and the rule table together, and the existing
  `RelationLabel` Ord test (labels_in_declaration_order) must stay green.
- The `CommonListArgs` pattern from `src/listing.rs` (`ListArgs` struct, `build`
  fn, `scan_and_format` in meta) is reused for `concept-map list` — no new list
  infrastructure.
- `--format json` on `show` reuses the existing `Format` enum
  (`table`/`json`), consistent with other Doctrine show commands.
- Node identity is by derived **key**, not authored label. The canonical
  collision diagnostic in `check()` surfaces when two different labels
  normalise to the same key. This is informational — the parser already
  resolves it (first-wins).
- DOT is the primary export format because it composes with SL-072's
  existing Graphviz bridge. Mermaid and JSON are secondary but equally
  tested.
