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
- **Prose-in-data + merge collisions.** `notes`, `summary`, `description: ""`
  are *narrative* holes inside the data; two deltas editing different
  requirements of one spec collide in one block. (The fix is the per-row split,
  not exiling every list: testable-criteria arrays — `acceptance_criteria`,
  `success_criteria` — are short structured strings and **stay** in TOML, where
  they remain queryable; only narrative lifts to prose. See § The decomposition.)

## The decomposition

A spec is a **directory** (it already is — `sources.variants.path` points at
`contracts/*.md`). Normalize the three hats apart, reusing the slice
directory-entity shape and the shared reservation primitive (per-kind namespace
`prd|spec|rev/id/<n>` — see § Spec identity):

The live schema bundle (Section 4, the artefact→blocks map) shows a tech spec
carries **seven** blocks, not the four `SPEC-110` happened to use:
`spec.requirements`, `spec.capabilities`, `spec.relationships`,
`verification.coverage`, plus `spec.concerns`, `spec.hypotheses`,
`spec.decisions`. Each table-shaped block becomes a `<name>.toml` + a
`<name>.md` prose sibling; the prose-heavy ones (`decisions`, `concerns`) are the
*strongest* case for the split, not the weakest.

```
.doctrine/spec/110/
  spec-110.toml        # identity + flat fields
  spec-110.md          # sections 1–7, pure prose
  requirements.{toml,md}   # [[req]] facets   | ### FR-001 → description, acceptance criteria
  capabilities.{toml,md}   # [[capability]]   | ### → summary, success criteria
  coverage.{toml,md}       # [[entry]] join   | ### → notes
  concerns.{toml,md}       # [[concern]]      | ### → narrative
  hypotheses.{toml,md}     # [[hypothesis]]   | ### → narrative
  decisions.{toml,md}      # [[decision]]     | ### → rationale, consequences
  interactions.toml        # [[edge]] spec→spec FK rows
  collaborators.toml       # [[collaborator]] cross-spec requirement FK rows
```

### Mapping

| spec-driver block | Heresiarch artefact(s) | prose lifted to `.md` |
|---|---|---|
| `spec.requirements@v1` | `requirements.toml` `[[req]]` (incl. `acceptance_criteria[]`) | `description` |
| `spec.capabilities@v1` | `capabilities.toml` `[[capability]]` (incl. `success_criteria[]`) | `summary`, `responsibilities` |
| `verification.coverage@v1` | `coverage.toml` `[[entry]]` | `notes` |
| `spec.concerns@v1` | `concerns.toml` `[[concern]]` | narrative |
| `spec.hypotheses@v1` | `hypotheses.toml` `[[hypothesis]]` | narrative |
| `spec.decisions@v1` | `decisions.toml` `[[decision]]` | `rationale`, `consequences` |
| `spec.relationships@v1` · `interactions` | `interactions.toml` `[[edge]]` (`type` + target spec + `notes`) | — |
| `spec.relationships@v1` · `requirements.collaborators` | `collaborators.toml` `[[collaborator]]` (fully-qualified cross-spec req FK) | — |
| `spec.relationships@v1` · `requirements.primary` | **derived, not stored** | — |

`relationships` is **three-way**, not two (this is the gap the first review
caught: `SPEC-110` has `collaborators: []`, hiding it). Per the
`spec.relationships@v1` schema: `primary[]` is *"requirement IDs owned by this
spec"* — exactly the rows in `requirements.toml`, so **derived**.
`collaborators[]` is *"collaborator requirement IDs from other specs"*
(e.g. `SPEC-200.FR-010`) — a fully-qualified, cross-spec, requirement-level FK
that is **neither** derivable (not local) **nor** a spec→spec `interaction` (those
are typed edges between specs, not requirement refs). It needs its own
registry-validated table; dropping it makes the decomposition lossy.

## Three rules

1. **Derive, don't store.** `relationships.primary[]` is *exactly the ids in
   `requirements.toml`* — compute it, never write it twice. This deletes the
   denormalization that drifts. Only `interactions[]` (genuine cross-spec edges)
   is irreducible.
2. **Tables, not trees.** Once prose is lifted into the `.md` siblings, every
   remaining structure is flat: arrays-of-tables in TOML, nothing deep. The
   "complex nesting" was prose + duplication; both are gone. No richer format
   needed.
3. **Foreign keys are the registry's job.** `coverage.requirement =
   "SPEC-110.FR-005"`, `interactions.target = "SPEC-123"` are validated *on load*
   by the `relation-index.md` registry — which can then flag `SPEC-TBD` as
   dangling. Small per-table sister files are exactly what make that registry
   cheap.
4. **One reference rule: bare local, qualified everywhere else.** A bare
   `FR-001`/`NF-002` appears in exactly one place — the `id` of its own row in
   `requirements.toml`. *Every* cross-table FK is **fully qualified**
   (`SPEC-110.FR-001`): coverage rows, collaborators, interaction targets,
   external refs. This matches the inherited schema (`verification.coverage`'s
   `requirement` is FQ-patterned; `relationships.primary[]`/`collaborators[]` are
   FQ) and removes the local/qualified ambiguity the first review flagged. The
   CLI may render a local shorthand in an owning-spec context; storage is always
   qualified. (Derived `primary[]` is the row ids **qualified** — derive still
   holds, it just prefixes the owning spec id.)

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

## Spec identity (kind-scoped, not a shared number space)

`PRD`/`SPEC`/`REV` are **three kinds, not one numbered family**. The schema
bundle's artefact map numbers and locates them independently — `product/PROD-xxx/`,
`specs/tech/SPEC-xxx/`, `revisions/RE-xxx.md` — so `PROD-011` and `SPEC-110`
coexist with no relation. A single `.doctrine/spec/<n>/` tree under one
`spec/id/<n>` sequence cannot reproduce that: it either fuses the sequences
(losing per-kind numbering) or collides `PRD-001` and `SPEC-001` on the same
`001/` directory.

**Decision: each kind is its own directory-entity kind.** Own dir
(`.doctrine/{prd,spec,rev}/<n>/`), own monotonic numbering, own reservation
namespace `<kind>/id/<n>`. The canonical key is `(kind, n)`, rendered
`PRD-110` / `SPEC-110` / `REV-001`; numeric ids are unique only **within** a
kind. The worked decomposition above is the `spec` kind (`.doctrine/spec/110/`).
This is exactly the engine's per-kind descriptor (slice-003): three descriptors,
no shared counter.

They share the **row+prose discipline**, not necessarily an identical fileset.
The bundle shows the shape diverges by kind — `prod` carries *no* fenced blocks
(frontmatter + prose only), `revision` is a single `revision.change`-style file,
only `spec` carries the seven table blocks. The fileset-as-function (slice-003)
absorbs that; pinning the PRD/REV filesets is left open (§ Open questions).

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
kind = "functional"          # functional | non-functional  (schema enum)
category = "cli"
lifecycle = "pending"        # enum from spec.requirements@v1 (e.g. pending, active, …)
acceptance_criteria = [      # testable list — structured, not prose
  "Routes to every registered subcommand",
  "Unknown subcommand returns a diagnostic and a non-zero exit",
]
```

(`lifecycle` and `kind` take their vocab from the `spec.requirements@v1` schema —
not reinvented here; `id`/`title`/`lifecycle`/`kind`/`description`/
`acceptance_criteria` are its required fields. `description` is the only one
lifted to prose; `acceptance_criteria` stays a structured array — it is a
testable list, queryable, and the per-row split already isolates merges, so it
need not become prose to dodge collisions.)

`coverage.toml` (the join table):
```toml
[[entry]]
requirement = "SPEC-110.FR-002"   # FK → fully-qualified (registry-validated)
artefact = "VT-CLI-LIST-001"
kind = "VT"
status = "verified"          # verified | partial | uncovered
```

`interactions.toml` (spec→spec edges; schema field `spec` → `target` here):
```toml
[[edge]]
target = "SPEC-123"          # cross-spec FK → registry-validated
type = "uses"                # schema: type + spec (+ notes/description), free-text type
notes = "Uses SpecRegistry to load and filter specifications"
```

`collaborators.toml` (cross-spec requirement FKs — the third arm of
`relationships`, requirement-level not spec-level):
```toml
[[collaborator]]
requirement = "SPEC-200.FR-010"   # fully-qualified external req FK → registry-validated
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
    id: String,                 // local: "FR-001" — the one place a bare id is stored
    title: String,
    kind: ReqKind,
    #[serde(default)]
    category: Option<String>,
    lifecycle: ReqLifecycle,
    #[serde(default)]
    acceptance_criteria: Vec<String>,   // testable list — stays structured
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
    kind: String,               // free-text per schema (calls/extends/…); not an enum
    #[serde(default)]
    notes: Option<String>,      // schema: notes + description both optional
    #[serde(default)]
    description: Option<String>,
}
```

`SpecStatus`, `ReqLifecycle`, `ReqKind`, `CoverageStatus` are closed enums
(`#[serde(rename_all = "kebab-case")]`) — fixed vocabulary, parse errors on
typos. Interaction `type` is **not** among them: the `spec.relationships` schema
makes it a free-text string (only `type` + `spec`/`target` are required, `notes`
and `description` optional), so it is a `String`. The FK *strings* are open (any
id is syntactically valid); the *registry* decides whether they resolve.

These structs are the **parse layer**. When the engine lands, the validated
*internal* model newtypes the FK strings (`SpecId`, `RequirementKey`) and parses
the open `type` — but that two-layer split is a build-time concern, not pinned
in this deferred note.

## Lifecycle

- **Spec status** `draft → active → deprecated → superseded` — recorded, by hand,
  ungated in v1 (as slices/drift).
- **Requirement lifecycle** — the enum is the `spec.requirements@v1` schema's
  (e.g. `pending`, `active`, …; *not* reinvented here). It is advanced by the
  *change process*, not edited in place: a delta/slice that implements `FR-001`
  flips it on completion (spec-driver's "complete delta updates requirement
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

## Design-data vs runtime-state (a boundary the schema bundle exposes)

Not every `supekku:*` block is authored design data. Two kinds the bundle
carries are **mutable runtime state** and do **not** belong to this entity model:

- **`workflow.*`** (`state`, `sessions`, `handoff`, `review-findings`,
  `review-index`) — agent-orchestration state, written continuously by tooling.
  This is coordination, adjacent to the deferred transient-lease layer
  (reservation-spec § Deferred), **not** a design artefact. Out of scope here.
- **`phase.tracking`** (`tasks_completed/total/blocked`, `started`/`completed`, a
  timestamped progress log) — the one *change-side* artefact that carries live
  state. It is high-write and merge-churny, unlike prose+facets. Flagged: when
  the phase entity is designed (the roadmap's IP+phases slice), its tracking
  state may want a different home/treatment than the static row+prose split — do
  not assume it fits this mould.

The line: **this model is for data authored once and referenced; state mutated
on every agent tick is a different problem.**

## Out of scope

- **The registry / index, FK validation engine** — [relation-index](relation-index.md).
  The *FK-validation pass* is an early, cache-independent deliverable triggered by
  the first cross-spec FK (relation-index § Two purposes); the *cache* is the part
  deferred to scale.
- **Code↔spec sync adapters** (spec-driver's `sync`) — needs a code parser; later.
- **Coverage computation** (deriving uncovered requirements) — a registry query;
  the artefact only *stores* coverage rows, it doesn't compute gaps.
- **PRD/REV-specific fields** beyond the shared shape — folded in when those kinds
  are designed.

## Known risks

- **Integrity value must co-land — it is not a later slice.** The headline win (FK
  validation catching `SPEC-TBD`-class dangling refs) arrives with the registry's
  FK-validation pass, which is **not** scale-gated — its trigger is *the first
  cross-spec FK authored* (relation-index § Two purposes). But this entity
  *introduces* the cross-spec tables (`collaborators`, `interactions`), so the
  trigger fires on its first real use. The pass therefore ships **in the same
  slice as the spec entity**, not after it: the minimum spec-landing bundle is
  `heresy spec new` · `spec req add` · `spec show` · **`spec validate`**. Shipping
  the tables without `validate` would pay the decomposition's file-count and
  read-locality cost up front while keeping the dangling-FK defect it was designed
  to remove — explicitly disallowed. The pass is cheap and cache-independent
  (relation-index § Two purposes), so co-landing is not a scope problem. Until the
  spec entity lands, there are no cross-spec refs to validate; the question is
  sequencing *within* its slice, and it is answered: together.
- **Row↔prose orphans (self-drift).** Each table entity is a `[[…]]` row *and* a
  `### id` prose heading — joined by id, not duplicated (the row carries facts,
  the prose carries narrative), but the *pairing* can desync under hand edits.
  Inherits drift-spec's mitigation verbatim: an atomic `heresy spec <table> add`
  that writes both, and a `list`-time orphan lint per table. The hairiest entity
  must not get the weakest drift guard. The atomic add must be **edit-preserving**
  (`toml_edit` / structured append), not a full serde reserialize — a reserialize
  drops hand comments and the unknown keys slices-spec promises to preserve. (The
  read-only `Meta` parse stays plain serde; only the mutating verbs need the
  edit-preserving document model — same caveat applies to `heresy drift add`.)
- **More files per spec.** A full tech spec is now ~13 files (identity + prose +
  seven block pairs + interactions/collaborators), not 1. Two costs: (a) the
  relation-index budget must count *files*, ~13× specs (handled there); (b)
  **read-locality** — understanding one requirement now spans its row, its prose,
  its coverage rows, and any capability FK'ing it. Accepted as a real cost (not
  zero), bought for clean diffs/merges and parse-without-a-block-parser, and
  recovered at read time by a `heresy spec req show <key>` that gathers the row,
  its prose section, its coverage rows, and inbound/outbound refs into one view
  (§ Follow-ups). The split is the storage shape, not the reading shape.
- **Requirement local-id collision across merges.** Ids are hand-assigned and
  semantic (`FR-`/`NF-`), so `max+1` does not apply and a `mkdir` claim cannot
  arbitrate a *row*. Two branches both adding `FR-009` merge cleanly into a silent
  duplicate. Lever is detection, not prevention: duplicate-`id` is a **hard** lint
  at **load over the merged file** (same shape as drift-spec § Known risks).
  Treat ids as immutable (never reuse a retired number).
- **Requirements moving between specs.** A spec split/merge could change a
  requirement's owning spec, making the compound key `SPEC-110.FR-001` an
  unstable global address — external links and audit trails would dangle on the
  move. **Decided (Option A): requirements never move.** A relocation is *retire
  the old id + introduce a new one*, linked by a `supersedes` edge; the old key
  stays resolvable forever (to the retired row), so `SPEC-110.FR-001` is a
  permanent address by construction. This is decided now — before external refs
  accumulate — because the alternative (a hidden internal UID + display key)
  introduces invisible ids too early, and retrofitting stability after refs pile
  up is the expensive path. Pairs with the immutability rule below (never reuse a
  retired number).

## Open questions

1. **Capabilities — entity or grouping?** Capabilities are essentially named
   bundles of requirements + prose. Whether they earn their own table or collapse
   into a `group` tag on requirements is unresolved; kept as a table here because
   they carry their own prose and success criteria.
2. **Where the requirement lifecycle source of truth sits** once the change
   process exists — `requirements.toml` vs derived from completed changes. Decided
   with the change/delta lifecycle.
3. **Spec-refactoring mechanics.** The *policy* is decided — requirements never
   move (§ Known risks, Option A: retire + reintroduce under `supersedes`). What
   stays open is the workflow that enacts a split/merge: how the `supersedes`
   edge is recorded, and whether a redirect from a retired key to its successor
   is surfaced in queries. Decided with the spec-refactoring workflow; the key
   stability it depends on is now guaranteed.
4. **PRD / REV filesets.** `spec` carries seven table blocks; the bundle shows
   `prod` carries none and `revision` is a single-block file (§ Spec identity).
   Each kind's exact fileset is pinned when that kind is designed; the
   fileset-as-function engine (slice-003) already admits the difference.

## Follow-ups

- **Glossary.** The spec family already lists `PRD/SPEC/REV`; add the requirement
  (`FR-`/`NF-`) and coverage (`VT-`) referends as sub-entities when this leaves
  deferred.
- **Locality recovery CLI.** `heresy spec show <SPEC-id>` and `heresy spec req
  show <SPEC-110.FR-001>` reassemble the decomposed pieces (identity, prose,
  table rows, inbound/outbound refs) into one human view — the read-locality
  mitigation (§ Known risks). `spec show` is part of the minimum landing bundle
  (§ Known risks, integrity).
- **Entity engine.** The scaffold/scan/claim machinery is generalised *before*
  spec needs it — extracted against the slice + design-doc callers (the roadmap's
  design-doc slice, which folds in the former slice-002), with a kind-supplied
  *fileset* (a spec scaffolds ~13 files, a slice 2) and an optional reservation
  (specs reserve a top-level id; sub-artefacts like requirements do not). Spec is
  the engine's *third* caller, not its justification.
