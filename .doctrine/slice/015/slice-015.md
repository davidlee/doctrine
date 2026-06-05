# Spec entity v1: product + technical specs

## Context

Doctrine has product/technical specs as *intent only*: the `/spec-product` and
`/spec-tech` skills exist but both declare "Not yet structural — author prose
under `doc/*` by hand." The glossary already reserves the id schemes
(`PRD-001`, `SPEC-001`, `REV-001`) and two deferred design notes have already
worked the shape end-to-end:

- [`doc/spec-entity-spec.md`](../../../doc/spec-entity-spec.md) — the spec entity
  itself, worked to serde structs: three subtypes (`product`/`tech`/`revision`),
  separate folders + per-subtype facet sets, requirement-as-compound-key-row
  (`SPEC-110.FR-001`), derive-don't-store, FK validation = the registry's job.
- [`doc/entity-model.md`](../../../doc/entity-model.md) — the umbrella taxonomy
  the note hangs off (storage rule, entity-vs-facet, generic-edge-table +
  typed exceptions, three-layer Rust model, `status` not `state`).

This slice makes specs **first-class entities** — modelled after Spec-Driver's
product/technical specs but in the Doctrine way: simpler, cleaner, no less
powerful. It rides the shipped scaffold engine (`src/entity.rs`, SL-003) the way
ADR (SL-006) did — spec is the engine's next caller, not a reason to fork it.

The two design notes are the **seed, not the contract.** The user's mandate is
explicitly to *challenge* the decomposition (the ~13-files-per-tech-spec split,
whether all seven tech facet tables earn their place) before it locks. That
contest happens in `/design` → `/inquisition`; this scope fixes *what* lands, not
the internal shape.

## Context Bundles & Sources

Where a `/design` agent should read in, grouped by authority:

**Governing design (committed, the seed):**
- [`doc/spec-entity-spec.md`](../../../doc/spec-entity-spec.md) — the worked spec
  entity: decomposition, requirement identity, serde structs, known risks, open
  questions. *The primary read; the thing `/design` challenges.*
- [`doc/entity-model.md`](../../../doc/entity-model.md) — umbrella taxonomy +
  adjudication (storage rule, entity/facet, edges, three-layer model).

**Supporting doc notes the spec note leans on (committed):**
- [`doc/relation-index.md`](../../../doc/relation-index.md) — the registry / FK
  validation; the *in-scope* `spec validate` rides its cache-independent pass.
- [`doc/drift-spec.md`](../../../doc/drift-spec.md) — row↔prose orphan mitigation
  this slice inherits verbatim.
- [`doc/slices-spec.md`](../../../doc/slices-spec.md) — the directory-entity shape
  + reservation primitive reused, not reinvented.
- [`doc/reservation-spec.md`](../../../doc/reservation-spec.md) — the per-subtype
  `mkdir` reservation namespace.
- [`doc/glossary.md`](../../../doc/glossary.md) — the reserved id schemes
  (`PRD-`/`SPEC-`/`REV-`, `FR-`/`NF-`, `VT-`).

**Reference entities (code — the reuse seams, do not fork):**
- `src/entity.rs` — the kind-parameterised scaffold engine (SL-003, done).
- `src/adr.rs` + [`slice/006/design.md`](../006/design.md) — the worked
  "new entity rides the engine unchanged" precedent; `src/spec.rs` mirrors it.
- `src/slice.rs` — the other substrate caller.

**External source corpus (read-only, `/workspace/spec-driver/`):**
- `.spec-driver/tech/SPEC-110/` — the worked tech spec (the 7-block pathology).
- `.spec-driver/tech/SPEC-134/` — a `stub`-status tech spec.
- `.spec-driver/product/PROD-008/` + `requirements/FR-00{1,2,3}.md` — product
  spec with standalone-file requirements.

**Local research (GITIGNORED, disposable — `scratch/`):**
- `scratch/spec-driver-schemas.local.md` — the full Spec-Driver block schemas
  (275KB; the source the spec note's mappings derive from). Sample, don't dump.

## Scope & Objectives

**Subtypes: `product` + `tech`.** Two folders, two per-subtype facet sets, one
shared entity model.

- **Product spec** — the *what/why*. The light subtype: identity TOML + prose
  body (+ a light requirements/capabilities set, exact combination decided in
  `/design`). No cross-spec FK tables.
- **Technical spec** — the *how*. The heavy subtype: identity TOML + prose +
  the facet tables surviving the `/design` challenge (the note's candidate set:
  requirements, capabilities, coverage, concerns, hypotheses, decisions, plus
  the cross-spec `interactions` / `collaborators` FK tables).

**Minimum landing bundle** (the note's integrity floor — non-negotiable because
tech is in scope): `doctrine spec new · spec req add · spec show · spec validate`.

- `spec new <subtype>` — scaffold the subtype's fileset via the entity engine.
- `spec req add` — atomic, **edit-preserving** (`toml_edit`) write of a
  requirement row + its prose companion (the row↔prose orphan guard).
- `spec show` — read-locality recovery: reassemble identity + prose + facet rows
  + inbound/outbound refs into one human view.
- `spec validate` — **FK-validation pass** over cross-spec refs; flags dangling
  keys (`SPEC-TBD`-class) and duplicate local ids. The headline win; must co-land
  with the tables that introduce cross-spec refs — shipping the decomposition
  without it pays the file-count cost while keeping the defect it removes (note
  § Known risks).

**Reuse, don't fork.** `src/spec.rs` mirrors `src/adr.rs`/`src/slice.rs` over the
shared `src/entity.rs` substrate; the fileset-as-function descriptor supplies each
subtype's combination. Resist a premature spec/slice/ADR abstraction — extract
only genuinely-shared substrate, as SL-006 did.

### Affected surface

- `src/spec.rs` *(new)* — the subtype-parameterised entity caller.
- `src/entity.rs` — extend the fileset descriptor for the two subtypes; **do not
  fork** the engine.
- FK-validation module *(new — the relation-index seed)* — the parse→validate
  pass over facet FK strings (`RawSpecToml → SpecEntity → FK diagnostics`).
- `src/cli` / command wiring — the `spec` subcommand surface.
- `install/templates` — prose scaffolds for each subtype's fileset.
- `.doctrine/spec/{product,tech}/<n>/` — the new entity trees; per-subtype
  reservation namespace (`spec/product/id/<n>`, `spec/tech/id/<n>`).
- `doc/spec-entity-spec.md`, `doc/entity-model.md` — design source; updated if the
  `/design` challenge moves decisions.
- `.claude/skills/spec-product`, `spec-tech` — drop the "not yet structural"
  caveat once the CLI exists.
- glossary — promote `FR-`/`NF-` (requirement) and `VT-` (coverage) sub-entities.

## Non-Goals

- **`revision` subtype.** Its home is explicitly open — `entity-model.md` pushes
  back on it being a spec subtype at all (closer to the change side). Excluded;
  resolved when the change/delta lifecycle is designed.
- **The relation-index *cache*.** Only the cache-independent **FK-validation
  pass** lands here (its trigger — first cross-spec FK — fires inside this slice).
  The scale-gated index/cache half of `relation-index.md` stays deferred.
- **Code↔spec sync adapters** (Spec-Driver's `sync`) — needs a code parser; later.
- **Coverage gap computation** (deriving uncovered requirements) — a registry
  query; v1 only *stores* coverage rows.
- **Spec-Driver corpus importer** — the migration note's job (`entity-model.md`
  § Migration), not this slice.
- **Spec-Driver ceremony, dropped wholesale:** slot-system symlink trees
  (by-slug/by-package/by-c4-level), registry JSON, audit-gate automation,
  contract-variant directories. This is the "simpler/cleaner" trim.
- **Spec lifecycle transitions / approval gating** — `status` hand-edited and
  ungated in v1, as slices/ADRs ship today.

## Risks, Assumptions & Open Questions

**Assumptions (carried):**
- `src/entity.rs` admits a new caller with a per-subtype fileset descriptor with
  no engine change — supported by SL-003 (done) and SL-006's "rides the engine
  unchanged." Exact API verified in `/design`, not now.
- Requirement identity is `(spec_id, local_id)`, hand-assigned, immutable;
  requirements **never move** (note's Option A — retire + reintroduce via
  `supersedes`). Carried as decided.

**Risks (inherited from the note):**
- **Read-locality.** A full tech spec is ~13 files; understanding one requirement
  spans its row, prose, coverage, and capability FKs. `spec show` is the
  mitigation; the *exact* file count is contestable in `/design`.
- **Row↔prose orphans (self-drift).** Each table entity is a `[[…]]` row + a
  `### id` prose heading; hand edits desync. Mitigation: atomic edit-preserving
  `add` + a `list`-time orphan lint.
- **Requirement local-id collision across merges.** Hand-assigned semantic ids;
  two branches both adding `FR-009` merge into a silent duplicate. Lever is
  detection: duplicate-id is a hard lint at load over the merged file.
- **Behaviour-preservation gate.** Extending `src/entity.rs` touches shared
  machinery — existing slice/ADR/memory suites must stay green unchanged.

**Open questions (deferred to `/design`):**
1. **Decomposition challenge (central).** Does tech genuinely warrant the full
   seven-table split + two FK tables, or does a lighter set lose no power? This
   is the user's "challenge. TBD." — the design's primary contest.
2. **Capabilities — entity or grouping?** Own table vs a `group` tag on
   requirements (note § Open questions 1).
3. **Product facet set.** Exact combination for the light subtype.
4. **FK-validation home.** Part of `src/spec.rs`, or a standalone registry module
   this slice introduces (the relation-index seam).

## Verification / Closure Intent

"Done" is judged by:
- `spec new product` and `spec new tech` scaffold their respective filesets via
  the engine; product light, tech the agreed facet set.
- `spec req add` writes row + prose companion atomically and edit-preservingly
  (round-trips without dropping comments / unknown keys).
- `spec show <id>` reassembles the decomposed pieces into one view.
- `spec validate` catches a deliberately-dangling cross-spec FK and a duplicate
  local id; passes a clean corpus.
- Existing slice/ADR/memory suites green **unchanged** (behaviour-preservation).
- `cargo clippy` zero warnings (bins/lib); `just check` clean.
- TDD red/green/refactor throughout; design locked via `/inquisition` before plan.

## Follow-Ups

- `revision` subtype, once the change/delta lifecycle is designed.
- relation-index *cache* (scale-gated half) + full coverage-gap queries.
- Spec-Driver corpus importer.
- Code↔spec sync adapters.
- Spec lifecycle transitions / approval (pairs with the absent slice-lifecycle
  transition gap, CLAUDE.md known gaps).
