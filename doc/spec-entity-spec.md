# Spec entity specification — design note

**Status: deferred. No action now.** This records the *shape* of the spec
entity so it can land without restructuring once Heresiarch grows a spec system.
It is the hairiest of the doc entities in the [glossary](glossary.md) and the
one whose design forces the registry question — so it is specified early to pin
the decisions the other notes ([relation-index](relation-index.md),
[drift-spec](drift-spec.md)) lean on.

Scope: the **spec family** — product requirements (`PRD-`), technical spec
(`SPEC-`), revision (`REV-`). They share one internal shape; the technical spec
(spec-driver's `SPEC-110`) is the worked example because it carries the most.

## The pathology

`SPEC-110.md` (spec-driver) is one 850-line file wearing three hats:

1. **Identity + flat fields** — frontmatter: `id`, `status`, `responsibilities[]`,
   `sources[]` (with nested `variants[]`).
2. **Prose** — sections 1–7 (~400 lines), pure narrative.
3. **An embedded mini-database** — four schema-tagged, versioned YAML blocks
   (lines 34–439, *before the H1*):
   - `spec.requirements@v1` — a **table of requirement entities** (`FR-001`…),
     each with `lifecycle`/`kind`/`category`/`acceptance_criteria[]`.
   - `spec.capabilities@v1` — capability entities, each carrying a `requirements[]`
     FK list, `summary`, `success_criteria[]`.
   - `spec.relationships@v1` — `requirements.primary[]` (those ids **again**) +
     `interactions[]` (typed edges to other specs).
   - `verification.coverage@v1` — a join table: `artefact × requirement × status`
     + `notes`.

The blocks are not "deeply nested config." They are **normalized relational rows
with primary and foreign keys**: `FR-001` is defined once, re-listed in
`relationships.primary[]`, FK'd from `capabilities`, FK'd from `coverage`, and
referenced from other specs, deltas, and the drift ledger.

## Diagnosis: relational data does not belong in a prose document

Format choice (YAML/TOML/RON) is irrelevant to the four real failures — all
caused by *embedding a queried registry in a hand-edited document*:

- **Parse-everything-to-query.** "What verifies `FR-001`? what's uncovered?"
  means parsing every 850-line file. Exactly `relation-index.md`'s bottleneck.
- **Unenforced referential integrity.** `SPEC-110`'s relationships block carries
  `spec: SPEC-TBD` four times — dangling foreign keys, committed. A typo'd
  `FR-099` parses fine in any format. Only a registry that validates keys on load
  catches this; no serialization syntax does.
- **Self-inflicted drift.** Requirements are stored **twice** — as `requirements`
  rows and as the `relationships.primary[]` id list. That duplication is the
  precise FM-vs-block mismatch [drift-spec](drift-spec.md) (`DL-048`) exists to
  clean up. Embedding *manufactures* the drift the ledger then tracks.
- **Prose-in-data + merge collisions.** `notes`, `summary`, `success_criteria`,
  `acceptance_criteria`, `description: ""` are prose holes inside the data; two
  deltas editing different requirements of one spec collide in one block.

## The decomposition

A spec is a **directory** (it already is — `sources.variants.path` points at
`contracts/*.md`). Normalize the three hats apart, reusing the slice
directory-entity shape and the shared reservation primitive (namespace
`spec/id/<n>`):

```
.doctrine/spec/110/
  spec-110.toml        # identity + flat fields
  spec-110.md          # sections 1–7, pure prose
  requirements.toml    # [[req]] rows (facets)
  requirements.md      # ### FR-001 → description, acceptance criteria (prose)
  capabilities.toml    # [[capability]] rows + requirements[] FK
  capabilities.md      # ### per capability → summary, success criteria (prose)
  coverage.toml        # [[entry]] join rows: requirement × artefact × status
  coverage.md          # ### per entry → notes (prose)
  interactions.toml    # [[edge]] cross-spec FK rows
```

### Mapping

| spec-driver block | Heresiarch artefact(s) | prose lifted to `.md` |
|---|---|---|
| `spec.requirements@v1` | `requirements.toml` `[[req]]` | `description`, `acceptance_criteria` |
| `spec.capabilities@v1` | `capabilities.toml` `[[capability]]` | `summary`, `success_criteria`, `responsibilities` |
| `verification.coverage@v1` | `coverage.toml` `[[entry]]` | `notes` |
| `spec.relationships@v1` · `interactions` | `interactions.toml` `[[edge]]` | — |
| `spec.relationships@v1` · `requirements.primary` | **derived, not stored** | — |

## Three rules

1. **Derive, don't store.** `relationships.primary[]` is *exactly the ids in
   `requirements.toml`* — compute it, never write it twice. This deletes the
   denormalization that drifts. Only `interactions[]` (genuine cross-spec edges)
   is irreducible.
2. **Tables, not trees.** Once prose is lifted into the `.md` siblings, every
   remaining structure is flat: arrays-of-tables in TOML, nothing deep. The
   "complex nesting" was prose + duplication; both are gone. No richer format
   needed.
3. **Foreign keys are the registry's job.** `coverage.requirement = "FR-005"`,
   `interactions.target = "SPEC-123"` are validated *on load* by the
   `relation-index.md` registry — which can then flag `SPEC-TBD` as dangling.
   Small per-table sister files are exactly what make that registry cheap.

## Requirement identity (central design decision)

A requirement is **referenced everywhere** (capabilities, coverage, other specs'
interactions, deltas, drift ledgers) — so it must have a stable, addressable id.
But it does **not** warrant its own directory or a global reservation.

**Decision: requirements are table rows with a compound key, not standalone
artefacts.** Identity is `(spec_id, local_id)` — `FR-001` within `spec-110/`,
fully qualified as `SPEC-110.FR-001` when referenced from outside. The local id
is hand-assigned and unique within the spec's `requirements.toml`; no
cross-spec reservation needed because the spec id already namespaces it.

- *Why not per-file* (`requirements/FR-001.toml`): hundreds of micro-files, and
  the compound key already gives the integrity that per-file existence would.
- *Why not global reservation*: the spec id namespaces requirement ids, so two
  specs both having `FR-001` never collide. Reservation is for the spec dir only.

The registry validates that every `(spec, req)` FK resolves to a row that exists.

## Metadata & table schemas

`spec-110.toml`:
```toml
id = 110
slug = "supekku-cli"
name = "supekku/cli Specification"
status = "draft"            # draft | active | deprecated | superseded
kind = "spec"               # prd | spec | rev
category = "unit"
c4_level = "code"
created = "2025-11-02"
updated = "2025-11-03"
responsibilities = [
  "Provide a unified command-line interface",
  "Orchestrate thin command layers that delegate to registries",
]

[[source]]
language = "rust"
identifier = "heresy/cli"
module = "heresy::cli"
```

`requirements.toml`:
```toml
[[req]]
id = "FR-001"
title = "CLI MUST provide a single unified entry point routing to all subcommands"
kind = "functional"          # functional | non-functional
category = "cli"
lifecycle = "pending"        # pending | live | retired
```

`coverage.toml` (the join table):
```toml
[[entry]]
requirement = "FR-002"       # local FK → requirements.toml
artefact = "VT-CLI-LIST-001"
kind = "VT"
status = "verified"          # verified | partial | uncovered
```

`interactions.toml`:
```toml
[[edge]]
target = "SPEC-123"          # cross-spec FK → registry-validated
type = "uses"                # uses | extends | conflicts-with | ...
description = "Uses SpecRegistry to load and filter specifications"
```

## Serde types

FK fields are plain strings; their integrity is the registry's, not the type's.

```rust
#[derive(Debug, Deserialize, Serialize)]
struct Spec {
    id: u32,
    slug: String,
    name: String,
    status: SpecStatus,
    kind: SpecKind,
    #[serde(default)]
    responsibilities: Vec<String>,
    #[serde(default, rename = "source")]
    sources: Vec<Source>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Requirement {
    id: String,                 // local: "FR-001"
    title: String,
    kind: ReqKind,
    #[serde(default)]
    category: Option<String>,
    lifecycle: ReqLifecycle,
}

#[derive(Debug, Deserialize, Serialize)]
struct CoverageEntry {
    requirement: String,        // FK (local) — registry-validated
    artefact: String,           // FK (external test id)
    kind: String,
    status: CoverageStatus,
}

#[derive(Debug, Deserialize, Serialize)]
struct Interaction {
    target: String,             // FK (cross-spec) — registry-validated
    #[serde(rename = "type")]
    kind: InteractionType,
    description: String,
}
```

`SpecStatus`, `ReqLifecycle`, `ReqKind`, `CoverageStatus`, `InteractionType` are
closed enums (`#[serde(rename_all = "kebab-case")]`) — fixed vocabulary, parse
errors on typos. The FK *strings* are open (any id is syntactically valid); the
*registry* decides whether they resolve.

## Lifecycle

- **Spec status** `draft → active → deprecated → superseded` — recorded, by hand,
  ungated in v1 (as slices/drift).
- **Requirement lifecycle** `pending → live → retired` — advanced by the *change
  process*, not edited in place: a delta/slice that implements `FR-001` flips it
  to `live` on completion (spec-driver's "complete delta updates requirement
  statuses"). The requirement's source of truth is `requirements.toml`; the
  change record is what *moves* it. Gating waits on the change lifecycle.

## Relationship to the other notes

- **[relation-index](relation-index.md)** — the registry this entity *needs*. The
  per-table sister files are the small-files precondition that note protects;
  SPEC-110 is the stress case that justifies "no eager parse, keep relations
  small." FK validation lives there.
- **[drift-spec](drift-spec.md)** — what embedding *causes*. Decomposing + deriving
  removes the duplication that produces FM-vs-block drift; the drift ledger then
  handles only genuine spec-vs-shipped mismatches, not self-inflicted ones.
- **[slices-spec](slices-spec.md) / reservation** — the directory-entity shape and
  the `mkdir` reservation are reused, not reinvented; spec is another caller.

## Out of scope

- **The registry / index, FK validation engine** — [relation-index](relation-index.md);
  not built until query load demands it.
- **Code↔spec sync adapters** (spec-driver's `sync`) — needs a code parser; later.
- **Coverage computation** (deriving uncovered requirements) — a registry query;
  the artefact only *stores* coverage rows, it doesn't compute gaps.
- **PRD/REV-specific fields** beyond the shared shape — folded in when those kinds
  are designed.

## Known risks

- **Cross-file integrity without a registry.** Until `relation-index` lands, FKs
  (`coverage.requirement`, `interactions.target`) are unvalidated — a `list`-time
  lint can warn on locally-unresolvable refs, but cross-spec refs need the
  registry. Accepted: same deferral as relation-index.
- **More files per spec.** A spec is now ~8 files, not 1. Mitigated: prose and
  data each diff/merge cleanly in their own file; the directory shape already
  exists. Net win on merge-collision surface.
- **Requirement renumbering.** Hand-assigned local ids mean a deleted `FR-003`
  leaves a gap or invites reuse. Treat ids as immutable (never reuse); a future
  lint flags reuse. Same monotonic discipline as slice ids.

## Open questions

1. **Capabilities — entity or grouping?** Capabilities are essentially named
   bundles of requirements + prose. Whether they earn their own table or collapse
   into a `group` tag on requirements is unresolved; kept as a table here because
   they carry their own prose and success criteria.
2. **Where the requirement lifecycle source of truth sits** once the change
   process exists — `requirements.toml` vs derived from completed changes. Decided
   with the change/delta lifecycle.

## Follow-ups

- **Glossary.** The spec family already lists `PRD/SPEC/REV`; add the requirement
  (`FR-`/`NF-`) and coverage (`VT-`) referends as sub-entities when this leaves
  deferred.
- **Generalise the entity machinery.** `heresy slice`'s scan/claim/scaffold +
  reservation namespace want kind-parameterising before spec (and drift) become
  callers — one entity engine, many kinds, not parallel implementations.
