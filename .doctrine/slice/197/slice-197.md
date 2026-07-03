# Add concept (CPT) as a knowledge record kind

## Context

IMP-244 part 1: `knowledge new` currently supports six epistemic/governance kinds
(assumption, decision, question, constraint, evidence, hypothesis). There is no
neutral, addressable record for local terminology, distinctions, models, or
conceptual anchors. Adding `concept` (prefix `CPT`) fills that gap.

## Scope & Objectives

- Add `Concept` variant to `RecordKind` enum (clap ValueEnum — CLI auto-plumbed).
- Add `CONCEPT_KIND` const (dir `.doctrine/knowledge/concept`, prefix `CPT`).
- Add `CPT` to `kinds::RECORD` and all combined-constant sites (SEARCH_DEFAULT,
  ALL_KINDS, TAGGABLE, ADMISSIBLE_DEP_TARGETS).
- Add `KindRef` row to `integrity::KINDS`.
- Wire relation membership: CPT rides RECORD so it auto-inherits Shapes, Spawns,
  Supersedes, GovernedBy, References(concerns). Confirm these are semantically
  appropriate for a neutral concept; limit if needed.
- Define concept status vocabulary (decision needed — not epistemic, so likely
  different from existing 6 kinds).
- Concept facet: minimal or empty. IMP-244 says template sections are Definition,
  Notes, Distinguish from, Examples, Related — all prose in the md body, not
  structured TOML fields.
- Seed template (`install/templates/knowledge-concept.toml`).
- Update all touch sites: supersede policy, relation rules test pins, integrity
  test pins, kinds test pins, knowledge match arms (kind, as_str, ALL, statuses,
  hidden, terminal, facet, render, show, json, validate, resolve_ref).
- Tests: round-trip, seed, list, show, relation edges, priority partition.

## Non-Goals

- Part 2 of IMP-244 (per-edge relational descriptors) — separate item.
- New relation labels or graph semantics.
- Concept-map integration or web UI changes.
- Changes to `RelationLabel` vocabulary.

## Design Decisions

### Resolved

- **Concerns**: CPT must be added to the `references(role = concerns)` source-set
  in `relation.rs` — a concept can be about things ("concept X concerns SL-YYY").
  This is the concerns source-set literal list, separate from `kinds::RECORD`.

### Open

1. **Status vocabulary**: Neutral concept, not epistemic. Candidates: `draft`,
   `defined`, `refined`, `retired` — or simpler `active`, `draft`, `retired`.
2. **ConceptFacet shape**: Empty `#[derive(Default)]` or one `definition: Option<String>` summary field?
3. **Shapes / Spawns membership**: CPT in RECORD grants Shapes (concept shapes
   everything) and Spawns (concept spawns backlog items) — semantically loose
   for a neutral concept. Should CPT be excluded from those?
4. **Supersede policy**: Concepts evolve; supersession likely inappropriate.
   `None` (like HYP) is the starting assumption.

## Summary

Mechanical addition following the proven 6-kind pattern. ~15-20 code edits across
5-6 files + 1 new template. The engine is data-driven; `RecordKind::from_prefix`
auto-routes many dispatch sites. The design decisions above are small but need
explicit answers before implementation.

## Follow-Ups

- IMP-244 part 2 (relational descriptors) — separate slice.
