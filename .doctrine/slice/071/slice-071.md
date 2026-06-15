# Entity corpus scanner / hydrator substrate

## Context

Doctrine already walks its authored entity corpus in several places —
`relation_graph::scan_entities` (for `inspect` and `priority`), `spec::build_registry`
(for `spec validate`), and the `integrity` checker. These walkers are overlapping but
narrow: each hydrates only the fields its consumer needs, handles errors only via
`anyhow::bail` (fail-fast, no diagnostic accumulation), and loses source context
(where a relation row came from, what file a fact was authored in).

The brief asks for one reusable corpus scanner that hydrates all authored entity
TOML into typed, graph-ready records with source spans, relation target
classification, and diagnostics — so consumers (`inspect`, `priority`, `spec`
validation, future coverage/agent-context) project and filter one shared corpus
rather than walking `.doctrine/` independently.

The existing reusable half — `relation_graph::scan_entities` / `ScannedEntity` —
is the right starting point. It already walks `integrity::KINDS`, reads status,
title, and outbound edges per entity. The gap is diagnostic collection, source
tracking, target classification, and a presentation-neutral graph view.

Naming tension: `src/corpus.rs` already exists for the *memory* corpus sync
(SL-018). The entity scanner will live in a new module — name TBD during design.

## Scope & Objectives

1. **One reusable scan entry point.** A single function that walks `.doctrine/`
   entity files via `integrity::KINDS`, hydrates every entity into typed records,
   and returns entities + edges + diagnostics in one pass.

2. **Richer hydration.** Beyond what `ScannedEntity` already carries (key, kind,
   status, title, outbound edges), the hydrated records should carry:
   - Source path and span for every fact (entity identity, each relation row)
   - Relation target classification: resolved entity / unresolved ref / free text
   - Any authored metadata that has a canonical reader (e.g., common TOML headers)

3. **Diagnostics, not panics.** Collect diagnostics for malformed TOML, unknown
   kinds, duplicate identity, invalid relation shape, unknown relation labels
   (if vocabulary is enforced), dangling references, and ambiguous references.
   Include file path, entity key, severity, and a human-readable message.
   Never fail-fast on a single bad entity.

4. **Presentation-neutral graph view.** Add a `CorpusGraph` with `nodes` and
   `edges`, supporting `outgoing`, `incoming`, and `neighbours` queries. The graph
   is pure — projection of the hydrated corpus, no disk or rendering dependency.

5. **CLI exposure.** Add a small debug/development command (`doctrine corpus scan`
   / `doctrine corpus graph`) with `--json` output for validating the shape.
   Not a polished user-facing feature; developer inspection only.

6. **Refactor, don't duplicate.** `relation_graph::scan_entities` should use the
   new scanner (or the scanner should be extracted from it). Existing
   `inspect`/`priority` behaviour must be preserved — the behaviour-preservation
   gate applies.

## Non-Goals

- Concept map UI, D2/Graphviz/SVG/browser rendering
- Graph database introduction
- Inferring speculative relations from prose
- A second independent kind registry — use `integrity::KINDS`
- Redesigning Doctrine's entity model
- Cross-repository or remote corpus scanning
- Polished user-facing CLI (debug/dev only)

## Affected Surface

- **New module** (name TBD): scanner + hydrator + diagnostics + graph view
- **Refactor target:** `src/relation_graph.rs` — `scan_entities` should consume the
  new scanner or be subsumed by it
- **CLI:** `src/main.rs` — new `doctrine corpus` subcommand
- **Existing consumers:** `relation_graph::build_relation_graph`, `priority::graph::build`,
  `spec::build_registry` — may benefit from the new scanner (opt-in, not forced)
- **Preserved:** `src/integrity.rs` (KINDS table), `src/entity.rs` (scan_ids, Kind),
  `src/relation.rs` (RelationEdge, RelationLabel, TargetSpec), `src/projection.rs`

## Risks

- **Behaviour drift in `inspect`/`priority`.** The scanner refactoring must
  produce identical output. Mitigation: the existing tests are the gate; run them
  unchanged.
- **Naming collision.** `src/corpus.rs` is a memory corpus sync module. The entity
  scanner needs a distinct name — resolved during design.
- **Overbuilding.** The brief's graph view (`outgoing`/`incoming`/`neighbours` with
  `depth`) could balloon. Mitigation: implement corpus + edge hydration first;
  graph projection is a follow-up phase or minimal viable implementation.
- **Diagnostic volume.** Collecting every diagnostic across all entities could
  produce verbose output. Mitigation: diagnostics are structured (severity levels);
  consumers filter.

## Verification / Closure Intent

- `cargo test` passes with the existing `inspect`/`priority` suites unchanged
- New fixture tests cover: valid entity hydration, multiple kinds, relation
  extraction, target resolution, dangling ref diagnostics, duplicate id detection,
  malformed TOML handling
- `doctrine corpus scan --json` produces valid JSON with entities, edges, diagnostics
- `doctrine corpus graph` produces a valid graph projection
- No parallel KINDS walker remains — `scan_entities` consumes the new scanner
- `cargo clippy` zero warnings
- `just gate` passes

## Summary

Extract a reusable entity corpus scanner/hydrator from the existing
`relation_graph::scan_entities` walk, adding source spans, target classification,
diagnostic accumulation, and a presentation-neutral graph view. Expose it via
`doctrine corpus` CLI for developer inspection. Keep existing behaviour intact.

## Follow-Ups

- Coverage scanner (`coverage_scan.rs`) could consume this corpus
- Future agent-context selection could project from this corpus
- The concept mapper (if ever built) should consume this, not walk `.doctrine/`
  independently
