# Spec entity specification — design note

**Status: SHIPPED (SL-015 v1).** This note recorded the *shape* of the spec
entity so it could land without restructuring once doctrine grew a spec system.
It landed in **SL-015** — `spec new`, `spec list`, `spec req add`, `spec show`,
`spec validate` plus the `requirement` peer entity — and this note is now
**reconciled to the as-built model**. The enduring thesis (relational data does
not belong in a prose document — § The pathology / § Diagnosis) is unchanged; the
**decomposition mechanics changed** from what the deferred draft proposed: a
requirement is a **reserved numeric peer entity (`REQ-NNN`)**, not a compound-key
facet row, and the seven-block facet richness collapsed to **members +
interactions** (the rest deferred). Where the old draft said `SPEC-110.FR-001`
(a compound key), the shipped model uses a durable FK `REQ-NNN` carried by a
spec-side membership row with a mobile `FR-`/`NF-` *label*.

Scope as shipped: the **spec family** — product requirements (`PRD-`) and
technical spec (`SPEC-`). Revision (`REV-`) is **deferred** (no subtype in v1).
The technical spec is the worked example because it carries the most (the only
subtype with `interactions`).

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
  catches this; no serialization syntax does. (Shipped: `spec validate`.)
- **Self-inflicted drift.** Requirements are stored **twice** — as `requirements`
  rows and as the `relationships.primary[]` id list. That duplication is the
  precise FM-vs-block mismatch [drift-spec](drift-spec.md) (`DL-048`) exists to
  clean up. Embedding *manufactures* the drift the ledger then tracks.
- **Prose-in-data + merge collisions.** `notes`, `summary`, `description: ""`
  are *narrative* holes inside the data; two deltas editing different
  requirements of one spec collide in one block. (The fix the slice took is
  stronger than the per-row split: a requirement is its **own entity directory** —
  see § Requirement identity — so two deltas touch two different trees, never one
  shared block. Testable lists — `acceptance_criteria` — stay structured TOML on
  that entity.)

## The decomposition (as shipped)

A spec is a **directory**, reusing the slice directory-entity shape and the shared
reservation primitive (per-subtype namespace `spec/{product,tech}/id/<n>` — see
§ Spec identity). The v1 fileset is **light** — identity + prose + the membership
edge file, plus (tech only) the spec→spec interactions file:

```
.doctrine/spec/tech/110/      # subtype folder (product/ | tech/)
  spec-110.toml        # identity + flat fields (title-keyed; see Metadata)
  spec-110.md          # prose body, pure narrative
  members.toml         # [[member]] rows: requirement FK + label + order
  interactions.toml    # [[edge]] spec→spec FK rows (TECH ONLY; absent on product)

.doctrine/requirement/7/      # the requirement PEER entity — its own tree
  requirement-7.toml   # id REQ-007, kind, status, description, acceptance_criteria
  requirement-7.md     # the requirement's prose statement
```

A requirement is **not** a facet row of the spec; it is a peer entity with its
own reserved numeric id (`REQ-007`) and directory. The spec *members* it — a
spec-side `members.toml` row holds the durable FK plus a sticky display label.

The seven-block facet richness of the deferred draft (`capabilities`,
`coverage`, `concerns`, `hypotheses`, `decisions`) is **not shipped in v1** —
deferred until a consumer forces each. v1 structures only what `spec show` /
`spec validate` need: the requirement entity, its membership edge, and the
tech-only spec→spec interaction edge. Prose-heavy material lives in the entity
`.md` bodies.

### Mapping (spec-driver block → as-built artefact)

| spec-driver block | doctrine artefact (v1) | status |
|---|---|---|
| `spec.requirements@v1` (a row table) | the **`requirement` peer entity** (`requirement/NNN/`), membered via `members.toml` `[[member]]` | shipped |
| `spec.relationships@v1` · `interactions` | `interactions.toml` `[[edge]]` (`type` + target spec + notes) — **tech only** | shipped |
| `spec.relationships@v1` · `requirements.primary` | the spec's `members.toml` rows (membership *is* the primary set) | shipped |
| `spec.relationships@v1` · `requirements.collaborators` | **dissolved** — cross-spec requirement reuse (`spec req link`) is deferred (no v1 verb); no `collaborators.toml` | deferred |
| `spec.capabilities@v1` | — | deferred |
| `verification.coverage@v1` | — | deferred |
| `spec.concerns / hypotheses / decisions` | entity `.md` prose | deferred-as-prose |

The old draft's three-way `relationships` (primary / collaborators / interactions)
collapses: **membership** subsumes `primary`; `collaborators` (cross-spec
requirement FK) is dissolved — sharing one requirement under a second spec is the
deferred `spec req link` verb (an additive mirror of `spec req add`'s membership
step), so v1 carries no `collaborators.toml`. Only the spec→spec `interactions`
edge survives as a structured cross-entity table.

## Identity rules (as shipped)

1. **A requirement is identified by its durable peer id, not a compound key.**
   The id `REQ-007` is reserved (engine `Fresh`, `mkdir` claim) and **immutable**.
   Every FK to a requirement — a spec's membership row, a future coverage/feature
   edge — stores that durable `REQ-NNN`. There is **no** `SPEC-110.FR-001`
   compound key: the requirement is not owned by, nor namespaced under, any spec.
2. **`FR-`/`NF-` are membership *labels*, not identities.** A spec's
   `members.toml` row carries `requirement = "REQ-007"` (durable FK) plus
   `label = "FR-001"` — a sticky, per-spec, human-facing display label
   (`FR-` functional, `NF-` quality). The label is **mobile**: it lives on the
   membership edge, so the same requirement membered by two specs can carry a
   different label in each, and a spec can renumber its labels without touching
   requirement identity. `order` is an advisory stable-sort key (gaps/dups
   cosmetic, not validated).
3. **Foreign keys are the registry's job.** `members[].requirement = "REQ-007"`
   and `interactions[].target = "SPEC-123"` are validated by `spec validate`
   (the `relation-index.md` registry seed) — dangling FKs, dangling tech-only
   interaction targets, duplicate labels within a spec, and orphan requirements
   (membered by no spec) are all hard findings. FK *strings* are open; the
   registry decides whether they resolve.

## Requirement identity (central design decision)

A requirement is **referenced from many places** (a spec's membership, future
coverage/feature edges, deltas, drift ledgers) — so it needs a stable, addressable
id. The deferred draft proposed a *compound-key facet row* `(spec_id, local_id)`;
the slice **overturned that**:

**Decision (as shipped): a requirement is a reserved numeric peer entity
`REQ-NNN`** — its own directory under `requirement/NNN/`, reserved through the
same `entity.rs` `Fresh` materialiser + `mkdir` claim that specs and slices use.
It is **spec-mediated** (no standalone CLI surface): `spec req add` reserves the
requirement *and* appends the membership row in one (two-tree) operation.

- *Why a peer entity, not a compound-key row:* reservation turns the merge-
  collision risk of hand-assigned `FR-` ids into an impossibility, and a durable
  identity that no spec owns is what lets membership (and its label) move without
  the address dangling. The integrity the compound key was meant to give now comes
  from the FK + `spec validate`, not from the key shape.
- *Why not per-file under the spec* (`requirements/FR-001.toml`): that re-imposes
  single-owner identity — the very thing the reframe removed.

The requirement parse struct (kebab-serde, title-keyed for the shared `Meta`):
`Requirement { id, title, slug, status, kind, description, tags,
acceptance_criteria }` with closed `ReqKind { Functional, Quality }` and
`ReqStatus { Pending, Active, Deprecated, Superseded }`.

## Spec identity (one family, two subtypes, separate folders)

`PRD`/`SPEC` are **one entity family with two shipped subtypes**, not one fused
number space and not unrelated kinds. They number and locate independently —
`spec/product/<n>/`, `spec/tech/<n>/` — so `PRD-011` and `SPEC-110` coexist with
no relation.

**Decision: each subtype gets its own folder + reservation namespace** —
`.doctrine/spec/{product,tech}/<n>/`, each its own monotonic numbering
(`spec/product/id/<n>`, `spec/tech/id/<n>`). The canonical key is `(subtype, n)`,
rendered `PRD-110` / `SPEC-110`; numeric ids are unique only **within** a subtype.
(`REV-` revision is deferred — not a v1 subtype.)

**Decision: the facet set is per-subtype, not shared.** They share the entity
*model* (the storage rule — identity TOML + prose body + facet edge files — and
the scaffold engine), **not** an identical fileset. Product carries `members.toml`
only (3-file scaffold); tech adds `interactions.toml` (4-file scaffold). The
engine's fileset-as-function (slice-003) supplies each subtype's own combination
as a descriptor.

## Metadata & table schemas (as shipped)

`spec-110.toml` — note the identity key is **`title`** (not `name`), so the shared
`meta.rs` `Meta` parses it (the `spec list` path):
```toml
id = 110
slug = "supekku-cli"
title = "supekku/cli Specification"
status = "draft"            # draft | active | deprecated | superseded
kind = "tech"               # product | tech
category = "unit"           # OPEN — Option<String>, deliberately not an enum
c4_level = "code"           # context | container | component | code (tech)
responsibilities = [
  "Provide a unified command-line interface",
  "Orchestrate thin command layers that delegate to registries",
]

[[source]]                  # tech-only code anchor (Source); seeded empty
language = "rust"
identifier = "doctrine/cli"
module = "doctrine::cli"    # optional
```

`members.toml` (the spec → requirement membership edge; the `spec req add`
append target — seeded empty at scaffold, appended edit-preservingly):
```toml
[[member]]
requirement = "REQ-007"     # durable FK → requirement peer entity (registry-validated)
label = "FR-001"            # sticky per-spec display label (FR- functional | NF- quality)
order = 1                   # advisory stable-sort key (gaps/dups cosmetic)
```

`interactions.toml` (spec→spec edges; **tech only**; array key is `[[edge]]`):
```toml
[[edge]]
target = "SPEC-123"          # cross-spec FK → registry-validated (must be a tech spec)
type = "uses"                # free-text type (not an enum)
notes = "Uses SpecRegistry to load and filter specifications"
```

The requirement entity's own `requirement-7.toml`:
```toml
id = 7
slug = "unified-entry-point"
title = "CLI MUST provide a single unified entry point routing to all subcommands"
kind = "functional"          # functional | quality  (ReqKind, kebab serde)
status = "pending"           # pending | active | deprecated | superseded
description = "..."          # the structured statement (rendered by `spec show`)
acceptance_criteria = [      # testable list — structured, queryable, not prose
  "Routes to every registered subcommand",
  "Unknown subcommand returns a diagnostic and a non-zero exit",
]
```

## Serde types (as shipped)

FK fields are plain strings; their integrity is the registry's, not the type's.

```rust
struct Spec {
    id: u32,
    slug: String,
    title: String,
    status: SpecStatus,
    kind: SpecSubtype,            // Product | Tech
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    category: Option<String>,    // OPEN by design (C6)
    #[serde(default)]
    c4_level: Option<C4Level>,   // Context | Container | Component | Code
    #[serde(default)]
    responsibilities: Vec<String>,
    #[serde(default, rename = "source")]
    sources: Vec<Source>,        // tech-only code anchors
}

struct Member {                  // members.toml [[member]]
    requirement: String,         // durable FK "REQ-NNN" — registry-validated
    label: String,               // "FR-001" / "NF-001" — sticky per-spec label
    #[serde(default)]
    order: i64,                  // advisory sort key
}

struct Interaction {             // interactions.toml [[edge]]
    target: String,              // cross-spec FK — registry-validated (tech)
    #[serde(rename = "type")]
    kind: String,                // free-text per schema; not an enum
    #[serde(default)]
    notes: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

struct Source {                  // tech-only code anchor
    language: String,
    identifier: String,
    #[serde(default)]
    module: Option<String>,
}
```

`Requirement` (the peer entity) is defined in § Requirement identity. `SpecStatus`,
`SpecSubtype`, `C4Level`, `ReqKind`, `ReqStatus` are closed enums
(`#[serde(rename_all = "kebab-case")]`) — fixed vocabulary, parse errors on typos.
`category` and Interaction `type` are deliberately **open** strings. These structs
are the **parse layer**; the registry (`spec validate`) resolves the FK strings.

## Reading a spec — `spec show` (the load-bearing reassembly)

A spec is only readable as a whole document through `spec show <spec-ref>` — a
**pure, local, ephemeral** reassembly (D8/D9): STDOUT only, no mutation, no
write, no cross-corpus scan. It composes: spec identity + flat fields → the spec's
own `spec-NNN.md` prose body **verbatim** → a **Requirements** section (each
`members.toml` row in `order`: heading `### FR-001 (REQ-007) — <title>`, then
kind, the requirement's structured `description` as its statement line, and
acceptance criteria — each member's requirement read by canonical FK
`REQ-NNN` → `requirement/NNN/`) → outbound **interactions** (tech only). Inbound
refs are **not** shown (a registry query — deferred). Because the render is a pure
function of present local state, it **cannot go stale**; a materialised
`*.rendered.md` is derived-tier and deferred (§ Lifecycle / D9).

## Validation — `spec validate` (the FK-integrity pass)

`spec validate [<spec-ref>]` is the cache-independent registry seed
(`src/registry.rs`). It scans the requirement + spec trees into id sets + an edge
list and checks — all **hard**, non-zero exit on any:

- every `members[].requirement` resolves to a requirement [dangling FK];
- every `interactions[].target` resolves to a **tech** spec [dangling, tech-only];
- `label` is unique within a spec's members [duplicate label];
- every requirement is membered by ≥1 spec [orphan = torn `spec req add`].

**Scoped vs corpus:** `spec validate <spec-ref>` checks only that spec's outbound
FKs + label uniqueness; the orphan check is corpus-level (a requirement's
membership is unknowable from one spec) and runs whole-corpus only. No id-collision
check (the `mkdir` reservation + git add handle it) and no cycle detection
(arrives with the feature DAG — deferred).

## Lifecycle

- **Spec status** `draft → active → deprecated → superseded` — recorded, by hand,
  ungated in v1 (as slices/drift).
- **Requirement status** `pending → active → deprecated → superseded` — advanced by
  the *change process*, not edited in place (a delta/slice implementing a
  requirement flips it on completion). The requirement's source of truth is its own
  `requirement-NNN.toml`.
- **Supersede / relocation.** Because requirement **identity is immutable and
  membership is mobile** (a label lives on the edge, not the id), relocating a
  requirement between specs is a membership move — re-point the `members.toml` row,
  the durable `REQ-NNN` address never dangles. A genuine replacement is *deprecate
  the old requirement + introduce a new one*, linked by a future `supersedes` edge;
  the durable id stays resolvable forever. (This **inverts** the deferred draft's
  Option A, which froze a compound key by forbidding moves — the peer-entity model
  makes moves safe instead of forbidding them.)

## Relationship to the other notes

- **[relation-index](relation-index.md)** — the registry this entity needs;
  `spec validate` is its shipped seed (FK validation; cache + cycle detection
  deferred). The small per-entity sister files are the small-files precondition it
  protects.
- **[drift-spec](drift-spec.md)** — what embedding *causes*. Decomposing into peer
  entities + edges removes the duplication that produces FM-vs-block drift.
- **[slices-spec](slices-spec.md) / reservation** — the directory-entity shape and
  the `mkdir` reservation are reused, not reinvented; spec + requirement are the
  engine's later callers.
- **[entity-model](entity-model.md)** — the umbrella taxonomy this sits under
  (fewer entity kinds, typed facet/edge files).

## Design-data vs runtime-state (a boundary the schema bundle exposes)

Not every `supekku:*` block is authored design data. Two kinds are **mutable
runtime state** and do **not** belong to this entity model:

- **`workflow.*`** (`state`, `sessions`, `handoff`, `review-*`) — agent-
  orchestration state, written continuously by tooling. Coordination, adjacent to
  the deferred transient-lease layer (reservation-spec § Deferred). Out of scope.
- **`phase.tracking`** — the one change-side artefact carrying live state;
  high-write and merge-churny. In doctrine it lives under `.doctrine/state/`
  (gitignored runtime tier), never in this authored model.

The line: **this model is for data authored once and referenced; state mutated on
every agent tick is a different problem, in a different tier.**

## Out of scope / deferred

- **The registry cache + cycle detection** — `spec validate` ships the FK-integrity
  seed; the cache and cycle detection arrive with the feature DAG.
- **`spec req link`** — reuse an existing requirement under a second spec (the old
  `collaborators` arm). An additive mirror of `spec req add`'s membership step.
- **Capabilities, coverage, concerns/hypotheses/decisions facets** — deferred until
  a consumer forces each; not v1.
- **Revision (`REV-`) subtype**, **PRD-specific fields** beyond the shared shape —
  folded in when those are designed.
- **Materialised `*.rendered.md`** (derived-tier), **code↔spec sync adapters**,
  **coverage computation** — all deferred.

## Known risks

- **Torn two-tree write (orphan requirement).** `spec req add` reserves a
  `REQ-NNN` then appends the `members.toml` row — two trees, not transactional. The
  one failure window (reserve OK, append fails) leaves an **orphan** requirement:
  the reserved dir is left uncommitted, operator-cleaned, and `spec validate` flags
  it **hard** (orphan = membered by no spec). The engine still guarantees the
  append step left no partial dir.
- **Requirement label collision within a spec.** Labels (`FR-`/`NF-`) are
  per-spec and auto-assigned `max+1` by kind (explicit `--label` honoured). Two
  branches both adding `FR-009` to one spec merge into a silent duplicate; the
  lever is detection — `spec validate`'s duplicate-label check is **hard**. Treat
  labels as stable once external refs exist.
- **Read-locality.** A requirement's facts span its own entity (row + prose) and
  its membership row in each spec. Bought for clean diffs/merges and parse-without-
  a-block-parser, recovered at read time by `spec show` (whole-spec reassembly).
  The split is the storage shape, not the reading shape.

## Open questions

1. **Capabilities — entity or grouping?** Deferred; unresolved whether they earn
   their own entity/edge or collapse into a `group` tag on requirements.
2. **Requirement lifecycle source of truth** once the change process exists —
   `requirement-NNN.toml` vs derived from completed changes. Decided with the
   change/delta lifecycle.
3. **`spec req link` mechanics** (cross-spec requirement reuse) — the membership
   model already admits it (a second `members.toml` row); the verb + any
   relabelling policy are pinned when first needed.
4. **PRD / REV facet sets** — product ships identity + members; the richer PRD set
   and the REV change-record facet are pinned when those subtypes are designed.

## Follow-ups

- **Glossary.** Add the requirement durable id (`REQ-`) and the membership labels
  (`FR-`/`NF-`) — done in this slice's canon sweep.
- **Locality recovery CLI.** `spec show <spec-ref>` (shipped) reassembles identity,
  prose, member requirements, and outbound interactions into one human view. A
  per-requirement `spec req show` (inbound + outbound refs for one requirement) is
  a deferred registry-query follow-up.
- **Entity engine.** The scaffold/scan/claim machinery was generalised *before*
  spec needed it (the SL-005 work); spec + requirement are later callers, each a
  kind-supplied fileset (tech ~4 files, product 3, requirement 2) with an optional
  reservation.
