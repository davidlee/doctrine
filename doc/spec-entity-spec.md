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
| `spec.requirements@v1` | `requirements.toml` `[[req]]` | `description`, `acceptance_criteria` |
| `spec.capabilities@v1` | `capabilities.toml` `[[capability]]` | `summary`, `success_criteria`, `responsibilities` |
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
kind = "functional"          # functional | non-functional  (schema enum)
category = "cli"
lifecycle = "pending"        # enum from spec.requirements@v1 (e.g. pending, active, …)
```

(`lifecycle` and `kind` take their vocab from the `spec.requirements@v1` schema —
not reinvented here; `id`/`title`/`lifecycle`/`kind`/`description`/
`acceptance_criteria` are its required fields, the last two lifted to prose.)

`coverage.toml` (the join table):
```toml
[[entry]]
requirement = "FR-002"       # local FK → requirements.toml
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

- **Integrity value is deferred — name the gate.** The headline win (FK validation
  catching `SPEC-TBD`-class dangling refs) does not arrive with the artefact; it
  arrives with the registry's FK-validation pass. That pass is **not** scale-gated
  — its trigger is *the first cross-spec FK authored* (relation-index § Two
  purposes), and the registry is its owner. Until then, only *intra*-spec refs are
  lint-checkable; cross-spec refs (`collaborators`, `interactions`) are
  unvalidated. So for that window the design carries the same defect it diagnosed,
  having paid the decomposition cost up front — honest, and bounded by a
  falsifiable trigger rather than a vague "feels slow."
- **Row↔prose orphans (self-drift).** Each table entity is a `[[…]]` row *and* a
  `### id` prose heading — joined by id, not duplicated (the row carries facts,
  the prose carries narrative), but the *pairing* can desync under hand edits.
  Inherits drift-spec's mitigation verbatim: an atomic `heresy spec <table> add`
  that writes both, and a `list`-time orphan lint per table. The hairiest entity
  must not get the weakest drift guard.
- **More files per spec.** A full tech spec is now ~13 files (identity + prose +
  seven block pairs + interactions/collaborators), not 1. Two costs: (a) the
  relation-index budget must count *files*, ~13× specs (handled there); (b)
  **read-locality** — understanding one requirement now spans its row, its prose,
  its coverage rows, and any capability FK'ing it. Accepted as a real cost (not
  zero), bought for clean diffs/merges and parse-without-a-block-parser.
- **Requirement local-id collision across merges.** Ids are hand-assigned and
  semantic (`FR-`/`NF-`), so `max+1` does not apply and a `mkdir` claim cannot
  arbitrate a *row*. Two branches both adding `FR-009` merge cleanly into a silent
  duplicate. Lever is detection, not prevention: duplicate-`id` is a **hard** lint
  at **load over the merged file** (same shape as drift-spec § Known risks).
  Treat ids as immutable (never reuse a retired number).
- **Requirements moving between specs.** A spec split/merge would change a
  requirement's owning spec, so the compound key `SPEC-110.FR-001` is **not** a
  stable global address across such a refactor — external links and audit trails
  dangle on the move. Open until a spec-refactoring workflow exists (§ Open
  questions); within-spec, ids are stable.

## Open questions

1. **Capabilities — entity or grouping?** Capabilities are essentially named
   bundles of requirements + prose. Whether they earn their own table or collapse
   into a `group` tag on requirements is unresolved; kept as a table here because
   they carry their own prose and success criteria.
2. **Where the requirement lifecycle source of truth sits** once the change
   process exists — `requirements.toml` vs derived from completed changes. Decided
   with the change/delta lifecycle.
3. **Do requirements move between specs** (spec split/merge)? If yes, the compound
   key is not a stable global address and external references need indirection;
   decided with the spec-refactoring workflow.

## Follow-ups

- **Glossary.** The spec family already lists `PRD/SPEC/REV`; add the requirement
  (`FR-`/`NF-`) and coverage (`VT-`) referends as sub-entities when this leaves
  deferred.
- **Entity engine.** The scaffold/scan/claim machinery is generalised *before*
  spec needs it — extracted against the slice + design-doc callers (the roadmap's
  design-doc slice, which folds in the former slice-002), with a kind-supplied
  *fileset* (a spec scaffolds ~13 files, a slice 2) and an optional reservation
  (specs reserve a top-level id; sub-artefacts like requirements do not). Spec is
  the engine's *third* caller, not its justification.
