# Knowledge-record entity surface (SPEC-019)

## Context

**SPEC-019** defines the knowledge-record entity surface — four `record_kind`s
(assumption `ASM`, decision `DEC`, question `QUE`, constraint `CON`) riding the
shared entity engine (**SPEC-004**) as the epistemic-and-governance sibling of
the backlog (**SPEC-015**). No code shipped; this is forward-intent.

The relation surface is the forcing function: records need to participate in the
cross-corpus relation contract (**SPEC-018**, **ADR-010**) so they can link to
slices, specs, ADRs, and each other. IMP-050 and IMP-053 track the relation gaps;
IMP-051 tracks the cross-kind supersession verb for records.

This slice realises the full SPEC-019: entity scaffolding, per-kind lifecycle
vocabularies, typed facets, evidence structure, the outbound relation seam, and
the supersession verb for knowledge records.

**Sequenced after SL-095** so the relation foundation (including the `supersede`
verb) is bedded in before records extend it.

## Scope & Objectives

1. **Four engine Kinds** — bind `ASM`, `DEC`, `QUE`, `CON` onto the entity engine
   with independent reservation namespaces under `.doctrine/knowledge/<kind>/`.

2. **Per-kind lifecycle vocabularies** as specified:
   - Assumption: `held → testing → validated | invalidated | obsolete`
   - Decision: `proposed → accepted | rejected | superseded`
   - Question: `open → answered | obsolete`
   - Constraint: `active → waived | superseded | retired`
   - Per-kind `is_terminal` predicate for list hide-set.

3. **Per-kind typed `[facet]` blocks** + shared `[evidence]` structure.

4. **Prefix→kind resolution** on read path — one `doctrine knowledge` verb set
   dispatches across all four prefixes.

5. **Outbound relation seam** (IMP-050): records in `RELATION_RULES` with
   `relates_to` (record → any numbered entity) and `spawns` (record → backlog
   item). Reverse derived per ADR-004.

6. **Record↔record associative edges** (IMP-053): `informs` and `bears_on`
   labels for record→record relations.

7. **Cross-kind supersession verb for records** (IMP-051): extends the
   `doctrine supersede` verb (built in SL-095) to knowledge records with the
   SPEC-019 §6 allowed matrix.

8. **`doctrine knowledge` CLI surface** riding SPEC-013's uniform grammar.

## Non-Goals

- **No priority-engine change** — IMP-047 (trinary actionability, record→item
  gating edges) is out of scope. Records ship with all-`Terminal` partition.
- **No constraint facet extensions** — IDE-006 (owner + immutability/enforceability
  axis) is deferred.
- **No knowledge lint verb** — IDE-009 is deferred.
- **No DEC-vs-ADR guidance** — IDE-007 is deferred.
- **No knowledge-record memory signpost** — IMP-083 is a separate fast-follow.

## Affected Surface

- `src/relation.rs` — RELATION_RULES rows for record source kinds (`relates_to`,
  `spawns`, `informs`, `bears_on`); RelationLabel variants
- `src/knowledge.rs` — new: entity scaffolding, facet/evidence parse, lifecycle
  transition, outbound_for arm
- `src/entity.rs` or engine — new KINDS entries
- `src/main.rs` — `knowledge` CLI verb family
- `src/supersede.rs` — extend to knowledge records (IMP-051)
- `src/listing.rs` — knowledge list columns
- `.doctrine/knowledge/` — new entity trees
- SPEC-019 — status transition to `done`; possibly amendments from implementation
  findings

## Risks & Assumptions

- **R1 (SPEC-019 completeness):** the spec is forward-intent; implementation may
  surface gaps. `/consult` on any spec ambiguity rather than improvising.
- **R2 (supersede verb extension):** the governance supersede verb (SL-095)
  must be designed for extension to other kinds — the record supersede matrix
  (§6) adds cross-kind crossing.
- **R3 (behaviour preservation):** the entity engine is shared machinery; existing
  suites (slice, ADR, spec, backlog, memory) must stay green unchanged.
- **A1:** No records exist yet — no migration, no backwards compat.
- **A2:** IMP-047 (gating) ships later; records don't block work on launch.

## Verification / Closure Intent

- `doctrine knowledge new assumption "…"` scaffolds ASM-001 with correct facet.
- `doctrine knowledge list` shows records; `--status` filters per lifecycle vocab.
- `doctrine knowledge show ASM-001` renders identity, facet, evidence, relations.
- `doctrine link ASM-001 relates_to SL-095` succeeds.
- `doctrine link DEC-001 informs ASM-001` succeeds.
- `doctrine supersede DEC-001 DEC-002` flips DEC-001 to `superseded`.
- `doctrine inspect ASM-001` shows inbound + outbound relations.
- RELATION_RULES exact-coverage invariant updated and green.
- Existing suites green; `just gate` clean.
- SPEC-019 status → `done`.

## Summary

The knowledge-record entity surface realises SPEC-019: four record kinds riding
the shared engine with per-kind lifecycles, typed facets, and a full relation
seam over the SPEC-018 contract. The relation work (IMP-050/053/051) is the
forcing function — bedding in records extends the relation foundation SL-095
completes.

## Follow-Ups

- **IMP-047** — trinary actionability (records gate work without being actionable)
- **IMP-083** — knowledge-record memory signpost
- **IDE-006** — constraint facet extensions
- **IDE-007** — DEC-vs-ADR guidance
- **IDE-009** — knowledge lint verb
