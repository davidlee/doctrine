# SPEC-004: Entity engine

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

The entity engine is the kind-agnostic substrate every authored Doctrine kind is
built on. One engine materialises every directory entity — slice, adr, spec,
requirement, memory, backlog item, and the governance kinds — from a `Kind`
descriptor. This container realises the shared identity, storage, and claim
mechanism; the per-kind surfaces (the slice, ADR, backlog, and governance-kind
components) are its children and carry only what is specific to their kind, citing
this container for everything shared.

Authoritative note: this spec captures the engine's shipped architecture; the
durable content seeded from `doc/entity-model.md` is now owned here, and that doc
is demoted to historical seed.

## Responsibilities

Mirrors the structured `responsibilities` list: own the kind-agnostic
materialiser, the storage-rule realisation for entities, the identity and
reference model, the atomic claim seam, the corpus-wide kind table, and the
runtime-state boundary.

### The storage rule's entity realisation

Each entity is a directory holding a fixed shape:

```text
<entity-dir>/
  <entity>-<id>.toml   # identity, lifecycle, owners, summary, typed references
  <entity>-<id>.md     # prose only
  <facet>.toml         # flat structured rows (arrays-of-tables)
  <facet>.md           # prose keyed by row id — only when a facet needs narrative
```

Structured, queryable data lives in the TOML identity file and typed facet tables;
the Markdown body is prose only. The tooling reads and round-trips TOML facets but
never parses prose structure: a prose template is a write-once scaffold applied by
token substitution, so a renamed or deleted heading is a harmless no-op, never an
error. Templates are sensible defaults; the TOML facets are owned, locked formats.

### Entity vs facet taxonomy

The model is a small set of durable authored entities with typed facets and tables
attached, rather than modelling every file, block, phase, or index row as a
first-class kind. A requirement is a peer entity (`REQ-NNN`) membered onto a spec
via a `members.toml` row, not a sub-kind. Payload-bearing relationships stay typed
tables (a spec's `members.toml`, tech `interactions.toml`); payload-free links use
the generic edge table.

### Identity and reference model

- **Canonical string id externally, numeric id internally** — `id = "SPEC-004"`
  with `number = 4`. Cross-entity references target the durable peer id; every
  entity is addressable on its own, so there is no compound owner-qualified key.
- **Membership labels are not identities.** A requirement's `FR-001` / `NF-001` is
  a per-membership label carried on the spec's `members.toml` edge, distinct from
  the requirement's durable `REQ-NNN` identity.

### The claim seam

A generic atomic claim (mkdir-backed) guards id allocation and named-entity
placement. The same seam serves both identity shapes; only the interpretation of
an existing claim differs by caller — a numbered caller treats it as a lost race
and recomputes, a named caller treats it as a duplicate and errors. The claim is a
swappable backend; the local-filesystem implementation is the one in use.

### Family-specific status vocabulary

Lifecycle vocabulary is family-specific, not one global dialect; the word is
`status`. Each kind carries its own status set (slices, backlog items, ADRs, and
specs each have their own). Approval, where modelled, is a separate field, never
folded into `status`. The corpus-wide kind table (`integrity::KINDS`) enumerates
each kind's directory, prefix, and its stateful status set.

### Runtime-state boundary

Mutable runtime state — session, handoff, review-index caches, phase-tracking
churn — lives under a separate `.doctrine/state/` tree and is not part of the
authored entity taxonomy. Mutating verbs write the row-and-prose companions
edit-preservingly via `toml_edit`, never a full reserialize, so comments, inert
tables, and unknown keys survive a status transition.

## Concerns

- **Behaviour preservation.** The engine is shared machinery; changes to it are
  proven by the existing per-kind suites staying green, since every kind rides the
  same materialiser.
- **TOML integrity at scaffold time.** User-supplied free text spliced into a TOML
  literal must be escaped through the serializer, or a later read of the entity
  tree fails to parse.
- **Single dispatch site.** The kind→engine boundary is intentionally one place to
  grep; dispatch is data-driven, not polymorphic.

## Hypotheses

- **A `Kind` is data, not a trait.** With a small, compile-time-known set of kinds
  and no plugin story, a data descriptor (`{ dir, prefix, scaffold fn }`) handed to
  the materialiser is preferred over trait-object polymorphism; the seam to revisit
  only if third-party kinds ever arrive.
- **Generalise only as far as the second identity shape forces.** The engine serves
  numbered and named identity through one materialiser because a named caller
  (memory) forced it — not via a speculative identity-strategy framework.

## Decisions

- **D1 — three Rust layers: Raw → Entity → Registry.** A tolerant raw parse
  preserves unknown keys; the entity layer is the validated model (typed ids, soft
  enums, normalized paths); the registry resolves references and reports FK
  diagnostics. Schemas are generated from the Rust types, not hand-authored
  externally.
- **D2 — shared mechanism lives here, kind specifics descend.** Identity, claim,
  storage realisation, and the render pipeline are owned by this container; each
  per-kind component cites this container for them and restates none of it.
- **D3 — runtime state is held physically apart** from the authored taxonomy under
  `.doctrine/state/`, so disposable progress never contaminates committed truth.
