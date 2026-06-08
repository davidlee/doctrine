# Technical-spec system support: descent, decomposition & integrity

## Context

SL-015 shipped the spec entity with a `tech` subtype: identity, requirements as
peer entities, membership labels, on-demand `spec show` reassembly, and the
`spec validate` FK gate (`mem.system.spec.composition-seam`). It also landed the
tech-only flat fields the *how* needs at rest: `c4_level` (closed
context|container|component|code enum, rendered) and `[[source]]` code anchors
(language / identifier / optional module, rendered). What it did **not** ship is
the relational spine PRD-012 v1 requires: a spec has no way to descend from the
product capability it realises, no way to decompose into a single-parent tree,
and the registry runs no decomposition integrity (cycle detection was explicitly
"deferred to the feature DAG").

PRD-012 ("Technical Specifications") settled the v1 surface: descent, C4 level,
single-parent acyclic decomposition, typed peer interactions, code anchors, and
lineage-via-supersession — with the importer (OQ-2) and dedicated transform
verbs (OQ-4) reserved. SL-021 (tech-spec corpus backfill) named this work as its
own prereq slice: decomposition as a *first-class structured field* (a
single-valued outbound `parent`/`decomposes` FK under ADR-004 outbound-only),
**not** a free-text `interactions` edge — `interactions` stays for peer relations
(`uses`/`calls`). This slice builds that spine so SL-021 can author against a
complete tech surface.

This is the entity-model + integrity delta only. The folder hoist
(`.doctrine/spec/{product,tech}/` → top-level) and the corpus content are
separate work (an ADR/migration and SL-021 respectively).

## Scope & Objectives

Close the gap between the SL-015 tech surface and PRD-012 v1, requirement by
requirement. Build only what is missing; reuse the shipped seams unchanged
(behaviour-preservation gate — the SL-015 suites stay green).

1. **Cross-family descent (REQ-082 / FR-002).** Add a structured outbound
   relation on a tech spec naming the product capability it realises, storing the
   target's durable peer id only (`PRD-NNN`), never a compound or owner-qualified
   key. `spec validate` confirms the target is an existing product spec; `spec
   show` renders it. The spec does not restate product intent — the relation is a
   pointer, not prose.

2. **Single-parent decomposition (REQ-083 / FR-003).** Add a single-valued
   outbound `parent` (decomposition) FK on a tech spec, stored once on the child
   (ADR-004). A parent's children are **derived** by registry scan, never stored.
   `spec show` renders the parent and the derived children; `spec validate`
   confirms the parent resolves to an existing tech spec.

3. **Decomposition integrity (REQ-087 / NF-001).** `spec validate` enforces the
   tree: a second parent is rejected (structurally precluded by a single-valued
   field, but a malformed multi-value/self-parent must still hard-fail), and a
   cycle in the parent chain is a hard finding returning non-zero. This adds the
   parent-chain cycle detection the registry deferred.

4. **Peer-interaction target-kind correctness (REQ-084 / FR-004).** Distinguish
   an *invalid target kind* (a peer `interactions` edge pointing at a product
   spec — the target exists but the edge type is wrong) from a *dangling
   reference* (target id resolves to nothing). Today a `PRD-*` interaction target
   is merely "absent from the tech_specs set" and reported as dangling; PRD-012
   §6 requires the kind distinction.

5. **Lineage & orphan integrity on supersession (REQ-086 / FR-006).** A tech spec
   retired or superseded through the shared lifecycle records recoverable lineage
   (what became what) rather than being deleted/rewritten in place, and never
   silently orphans its decomposition children: a child whose parent is
   superseded/removed without explicit re-parenting is an integrity finding.
   v1 records supersession lineage minimally; the merge/split *representation*
   (OQ-4) and Theseus identity threshold (OQ-3) stay reserved.

Authoring stays hand-edited TOML validated by `spec validate` and reassembled by
`spec show` — the established v1 pattern (`c4_level`, `[[source]]`, and
`interactions` all hand-authored, no producer verb). Whether decomposition/
descent warrant a thin producer verb vs. hand-edit is a design decision (see OQ
below), but the default and smaller path is no new command surface. Update the
`spec-tech.toml` scaffold/template to document the new fields.

## Non-Goals

- **Importer / hand↔import convergence code (REQ-088 / NF-002).** The importer's
  source and shape are unresolved (OQ-2). The code anchor (`[[source]]`) is
  already the single convergence seam; v1 builds no import path. The convergence
  requirement constrains a future importer, not this slice — satisfied-by-design
  (one anchor seam), no code.
- **Dedicated transform verbs (OQ-4).** Merge/split operations, automatic child
  re-parenting, and lineage edge representation beyond minimal supersession are
  reserved. v1 represents merge/split as manual supersession with recorded
  lineage.
- **Theseus identity policy (OQ-3).** The in-place-evolution vs. supersede
  threshold is reserved.
- **Drift ledger.** Recording a spec↔code mismatch is the drift capability's
  surface. This slice only ensures the anchor *data* a drift pass reads is
  present and resolvable in shape; it creates no drift record and resolves no
  anchor against live source.
- **Evergreen-altitude enforcement (REQ-089 / NF-003).** Holding specs at durable
  architectural altitude (not single-change design) is an authoring discipline
  enforced in `/design` and `spec-tech` SKILL guidance, not a code gate here.
- **C4 level & code anchors (REQ-081 / REQ-085).** Already shipped in SL-015;
  in scope here only insofar as render/validate touch them — not re-built.
- **Corpus content & folder hoist.** SL-021 and a separate ADR/migration.
- **PRD↔SPEC reverse view materialisation.** The reverse ("which tech specs
  realise this PRD") is derived by registry scan under ADR-004; a cached/indexed
  reverse view is later work.

## Affected surface

- `src/spec.rs` — `Spec` struct gains the descent and decomposition fields
  (parse + `spec show` render); render of derived children. Touches the parse /
  render seams only; existing fields unchanged.
- `src/registry.rs` — the FK/integrity gate: resolve descent target kind
  (product), resolve parent target kind (tech), parent-chain cycle detection,
  derived-children scan, orphan-on-supersession check, invalid-target-kind vs.
  dangling distinction for interactions.
- `install/templates/spec-tech.toml` — document the new fields in the scaffold
  comment block (mirrors the `c4_level` / `[[source]]` comments). Embedded
  template — heed the rust-embed re-embed footgun.
- Possibly `src/cli.rs` / a `spec` subcommand — only if design elects a producer
  verb over hand-edit (default: no).
- Tests in `src/spec.rs` / `src/registry.rs` and any e2e harness covering
  `validate` / `show`.

## Risks, assumptions, open questions

- **Verb vs. hand-edit (design).** Decomposition/descent as hand-authored TOML
  fields (consistent with the existing v1 pattern, smaller) vs. a thin producer
  verb (`spec parent`/`spec realises`) that prevents malformed edits. Default
  hand-edit; settle in `/design`.
- **Descent field shape (design).** Field name and serialisation for the
  cross-family pointer (`realises`/`descends_from` = `"PRD-NNN"`). Must store the
  durable peer id only (REQ-082) and not collide with the tech-only
  `interactions` edge (which is spec→spec peer, not cross-family). One flat field
  vs. a typed relation row.
- **Lineage representation (bounded by OQ-3/OQ-4).** v1 must record *enough*
  lineage for supersession to be recoverable without pre-empting the reserved
  merge/split representation. Risk of over-building into OQ-4. Keep minimal:
  likely a `superseded_by`/`supersedes` durable-id pointer, no lineage facet.
- **Cycle detection placement.** The registry scan currently builds id sets +
  an edge list with no graph traversal; adding parent-chain cycle detection is
  the first DAG-shaped check. Keep it local to decomposition (do not generalise
  into a "feature DAG" prematurely).
- **Orphan semantics.** "Child not orphaned on supersession" needs a precise
  rule: is a child of a *superseded* (not removed) parent an orphan, or only a
  child of a *removed* parent? PRD §6 implies a child pointing at a superseded
  parent is an orphan the guard flags. Confirm in `/design`.
- **Behaviour-preservation.** The SL-015 spec/registry suites are the proof the
  shared machinery is unchanged — they must stay green unedited.
- **Assumption:** `c4_level` enum, `[[source]]` shape, and the membership/label
  seam are final for the duration (PRD-012 settled).
- **Assumption:** PRD-012 v1 narrowing (importer + transform verbs reserved)
  holds; no expansion mid-slice.

## Verification / closure intent

- Each in-scope requirement (REQ-082, -083, -084, -086, -087) traces to a test:
  descent stored/rendered/validated; parent stored-once + children derived;
  cycle + second-parent + self-parent rejected non-zero; PRD-targeted peer edge
  reported as invalid-kind not dangling; supersession preserves lineage and an
  orphaned child is flagged.
- `doctrine spec validate` is green on a well-formed tech corpus and non-zero on
  each crafted violation; `spec show` reassembles a tech spec with descent,
  parent, derived children, peers, and anchors as one readable whole.
- SL-015 spec/registry suites pass unchanged (behaviour-preservation gate).
- `just check` green; clippy zero warnings; storage rule honoured (structured
  relations in TOML, no derived data — children, reverse view — persisted).
- SL-021 is unblocked: the tech surface is complete enough to backfill against.

## Summary

## Follow-Ups
