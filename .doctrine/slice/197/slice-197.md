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
- Update all touch sites: relation rules + test pins, integrity test pins, kinds
  test pins, knowledge match arms (kind, as_str, ALL, statuses, hidden, terminal,
  facet, render, show, json, validate, resolve_ref), partition row. **Not** supersede
  policy — D4 (`_ => None` already excludes CPT; no `supersede.rs` edit).
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

All four opened questions resolved in `design.md` (D1–D4):

1. **Status vocabulary** → `[draft, active, retired]`, seed `draft`, hidden/terminal
   `{retired}`. Rejected `refined`/`defined` (no behavioural line vs `active`).
2. **ConceptFacet** → empty (unit-like, no fields). Sections are prose (IMP-244).
   Seed emits an empty `[facet]` header (scaffold-order invariant); `show` suppresses
   it at display.
3. **Shapes/Spawns** → keep via RECORD-ride (permissive edges; DRY the single source).
4. **Supersede** → None (no `supersede.rs` edit; `_ => None` + `validate_matrix`
   absence gate it, identical to HYP).

## Scope revision (see design.md §2 — D0)

Scope grew from "bare additive CPT" to **scoped DRY, two phases**. The record-kind
surface is guarded scatter, not centralised; PHASE-01 closes the unguarded gaps
(behaviour-preserving, 6 kinds), PHASE-02 adds CPT:

- **PHASE-01** — P2 single-entry the two relation test pins → `RECORD`-derived; P3
  canary the two unguarded unions (Shapes-target, `governed_by`-sources); P4 derive
  the two user-facing runtime record-kind-list messages (`dep_seq:84`,
  `resolve_ref`) from the vocab. Suites green **unchanged**.
- **PHASE-02** — add CPT (append `RECORD`, fill compiler-forced `knowledge.rs`,
  `KINDS` row, `partition.rs` row, three relation record-set appends, seed template,
  clap prose hand-adds).

Excluded (D0): the `RelationRule.sources` table-shape refactor — fights Rust const
unions, drags the relation behaviour-preservation gate for marginal value.
Dropped: P1 (derive partition rows from knowledge vocab) — **unsound**,
partition-terminal ≠ knowledge terminal/hidden (design §2).

Touch-set (design-target selectors): `kinds.rs`, `knowledge.rs`, `integrity.rs`,
`relation.rs`, `priority/partition.rs`, `commands/dep_seq.rs`, `commands/cli.rs`,
`install/templates/knowledge-concept.toml`, `install/using-doctrine.md`, and the two
re-pinned goldens `tests/e2e_validate_byte_exact_golden.rs` +
`tests/e2e_knowledge_cli_golden.rs` (added post-external-review). `supersede.rs`
stays scope-fence only (not edited — D4).

External review (codex/GPT-5.5) integrated into design.md §1/§3-D2/§4/§7: seed
emits an empty `[facet]` header (scaffold-order invariant), the four combined
constants are canary-forced manual adds (not auto-cascade), two goldens re-pin in
PHASE-02.

## Follow-Ups

- IMP-244 part 2 (relational descriptors) — separate slice.
