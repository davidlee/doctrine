# Publication seam

## Context

RFC-021 / ADR-019 pulled embedding, projection, and publication apart into
independent asset policies. Contract A (minimal projection) already landed as the
SPEC-009 / PRD-006 revision under REV-028: install now provisions a justified base
and leaves the rest framework-owned. That cut removes something real — an asset
that no longer lands in the repo is one a user can no longer open, copy, or
consult offline.

Contract B restores that control *without* reintroducing projection. It is
governed by PRD-017 (product) and SPEC-026 (tech, `descends_from` PRD-017, parent
SPEC-003), rooted on ADR-019. Contract B has three consumable surfaces:
**publication** (the declared exposure seam), the **library** read surface
(`list | tree | show`), and **federated search**. Both the library and search
treat publication as their source of truth — the library reads the publication
declaration, and federated search registers the library as one provider. So
publication must exist first.

This slice peels off **publication alone** — the seam the other two slices
consume. The library read verbs and search federation are explicitly out of
scope here and follow as later slices.

Grounding facts (verified this session):
- The only live anchor in SPEC-026 is entity search (`src/search.rs`); everything
  publication-side is greenfield.
- Install's text-only read accessor is `asset_text` (`src/install.rs:861`) over
  the `install/` RustEmbed root (`Assets`). Other embed roots: `plugins/`,
  `memory/` (`CorpusAssets`), `web/map/dist/`. SPEC-026 D3 intends to generalise
  `asset_text` into a neutral, binary-safe, byte-level asset-read seam owned by
  neither projection nor publication.

## Scope & Objectives

Deliver the publication *policy and resolution* mechanism — the declaration that
says what is public and how it is addressed and licensed — such that a later
slice can build `library list | tree | show` and a search provider directly on
top of it. Concretely (SPEC-026 decisions in parentheses):

- **Publication declaration as sole authority** (REQ-373 / PRD-017 REQ-359, D1).
  A publication declaration, distinct from install's projection manifest and
  parsed independently, is the sole authority for what is public. An asset that is
  embedded but undeclared is not public. Each declared entry binds a stable
  logical address to a backing source and carries its exposure metadata: content
  kind, title/summary, licence, and customization status.
- **Duplicate-address rejection at admission** (D9). Two entries claiming the same
  logical address are rejected when the declaration is admitted, never silently
  resolved by source precedence. (The hymn cascade's own overlay is not such a
  collision — that precedence is SPEC-023's, not the declaration's to arbitrate.)
- **Logical-address resolver over a source-adapter interface** (REQ-374, D2). A
  resolver maps each stable POSIX-like logical address to its backing source
  through an adapter interface, with the embedded-asset adapter as the production
  default. The resolver is written against the interface, not one fixed storage
  type, so logical addresses survive a source-root reorganisation and leak no
  embed key or filesystem detail. (Storage independence — PRD-017 REQ-369.)
- **Licence as an explicit declared field, fail-closed** (REQ-379 / PRD-017
  REQ-367, D6). Every entry declares a licence from the fixed set {MIT, GPL},
  validated at validate/build time; an asset whose licence cannot be established
  fails publication rather than being defaulted. Per-entry provenance (declared,
  or rule-suggested and confirmed) is recorded so the licence is auditable. Any
  rule-based classification only *assists* authoring — the declared field is the
  source of truth.
- **Byte-level read reachable through the resolver** (REQ-376, D3 — see fork
  below). The resolver resolves an address to its *bytes* (binary-safe), so the
  later `show` slice streams them faithfully. Whether this slice extracts the
  neutral asset-source seam now or rides `asset_text` provisionally is a design
  decision (Follow-Ups).

Closure intent: the seam is done when (1) a declaration is parsed and admitted as
sole authority with duplicate addresses rejected; (2) an address resolves to its
backing bytes through the adapter interface, independent of physical layout; (3)
licence validation fails closed against {MIT, GPL} at validate/build with recorded
provenance; (4) the customization-status field is present on every entry. Because
the user-facing read verbs are deferred, verification is primarily at the engine
layer plus the validate/build-time licence gate — see the verification open
question below.

## Non-Goals

- **Library read surface** — `library list | tree | show` (REQ-375). Next slice.
- **Search federation** — provider interface, coordinator, grouping, typed
  envelope, exposing entity/memory/library as providers (REQ-377, REQ-378,
  REQ-380). Later slice.
- **A working runtime-loaded adapter** (PRD-017 OQ-4). The adapter *interface*
  is in scope (it is what makes the resolver storage-independent); a functioning
  non-embedded adapter and external-package version/integrity are not.
- **Projection changes / SPEC-009** — Contract A already landed under REV-028.
  This slice must not couple publication to the projection path.
- **Addressing below whole-asset granularity** — fragment addressing is a C2
  concern (PRD-017 scope).
- **Materialization of any kind** — the seam reads/exposes bytes; it never writes
  to, installs into, or overwrites the project.

## Summary

First implementation slice of Contract B (RFC-021 C1): the publication seam that
the library and federated-search slices consume. Establishes the publication
declaration (sole authority, distinct from projection), a storage-independent
logical-address resolver over a source-adapter interface, and fail-closed licence
declaration — deferring the user-facing library verbs and search federation to
later slices.

## Follow-Ups

The four scoping forks are **resolved** in `design.md` (decisions D-A…D-F); the
durable, cross-slice items are recorded as knowledge records:

- **OQ-1 (PRD-017) → deferred.** The shipped declaration is **templates-only**
  this slice; whether the memory corpus is published-for-copy waits for the
  library slice that consumes it. Recorded as [[QUE-172]] (`references(concerns)`).
- **OQ-5 (PRD-017) → provisional.** Customization-status ships as the closed enum
  `{customizable, fixed}`, C2-owned. Recorded as [[ASM-003]].
- **D3 seam-extraction → extract now (D-B).** This slice extracts `asset_text`
  into a neutral leaf `asset_source` seam (binary-safe), no physical file
  relocation. REQ-376's neutral-ownership criterion is met here.
- **Verification without a read surface → one `publication validate` command (D-A,
  revised after codex RV-287).** No `doctrine library` *read* verb this slice, but a
  thin `publication validate` command ships as the production consumer: it admits the
  shipped manifest and emits every entry to a sink. It is required to satisfy
  `deny(unused)` (a test-only seam won't compile), and doubles as the observable
  licence/resolvability gate and the release-artifact probe. `library list|tree|show`
  stay deferred.

Deferred to later slices: `library list|tree|show` read UX and search federation;
physical relocation of published material into semantically-owned source roots
(ADR-019 downstream); a working runtime-loaded adapter (OQ-4). Tooling gaps
IMP-298 / IMP-299 are open context, not blockers.

Honoured `mem.pattern.doctrine.spec-prose-requirement-drift`: mechanism nouns
(manifest, resolver, adapter) live in `design.md` (per-slice tech tier), kept out
of the product-observable scope framing above.
