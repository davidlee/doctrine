# lazyspec read-only projection

## Context

lazyspec (a Rust TUI spec framework, https://github.com/jkaloger/lazyspec, local
checkout `../lazyspec`) is being bolted on
as a **read-only** front-end for doctrine. Research is complete — brief at
`../lazyspec/research/lazyspec-doctrine-integration-brief.md`; decisions and
constraints in memory `mem.thread.lazyspec.frontend-integration`.

The integration is four pieces. This slice is the **doctrine-side pieces 2 + 3,
merged**: the JSON wire format *and* its producer. They are one deliverable — a
locked schema with no producer is an untested spec, and the producer's output *is*
the schema. Piece 1 (research) is done; piece 4 (the lazyspec fork) is a separate
`../lazyspec` change, out of scope here.

The emitter **rides SL-025** (uniform list/show/filter/render contract, **done +
closed**) by reusing its read APIs + `listing::canonical_id` — *not* its
`{kind, rows}` envelope. The Brief is a cross-kind aggregate in lazyspec's
vocabulary, so it is its own shape (a new `export lazyspec` command), not a
`Format` variant. "No parallel renderer" is honoured at the read/compose layer,
not at the serde layer. SL-025 is landed — no execution dependency remains.

Governing canon: ADR-001 (leaf ← engine ← command, no cycles), ADR-004 (relations
stored outbound-only; reciprocity derived), the pure/imperative split (no
clock/rng/git/disk in the pure layer). Composition model per
`mem.system.spec.composition-seam` (SL-015): a spec composes requirements via
`members.toml` (FK + mobile `FR-`/`NF-` label + order) and spec→spec edges via
`interactions.toml`.

## Scope & Objectives

**What changes**

1. A **locked JSON wire format** (brief §3): `meta` + `entities[]` + `types[]`.
   Conformance-tested so drift is caught before it reaches lazyspec.
2. A **new read-only CLI command** `doctrine export lazyspec` — named for its target
   so it never masquerades as native doctrine output — projecting the corpus into
   conformant JSON.

**Projection rules (the contract this slice owns; full detail in design §5.3)**

- **Node set:** `slice` (SL); `spec` → two virtual types **product-spec** (PRD) +
  **tech-spec** (SPEC), requirements inline; `adr` (ADR); `backlog` → **five** types
  by item_kind (ISS/IMP/CHR/RSK/IDE); `plan` → a **synthetic** child node
  (`PLAN-NNN`, plan not being a reserved entity). **Requirements are NOT
  standalone nodes** — inline in spec bodies as `FR-`/`NF-` labelled entries.
- **Every entity carries `validate_ignore: true`** (doctrine owns validation;
  `rules = []` does not empty lazyspec's rule set). **Emitted types are
  non-singleton** so `TypeConstraintChecker` stays satisfied — these two are
  load-bearing, from the brief.
- **Edges flatten** to lazyspec's four `RelationType`s
  (Implements/Supersedes/Blocks/RelatedTo); exotic edges → `RelatedTo`. Reciprocity
  is derived at projection time (ADR-004 — edges stored outbound-only). Read through
  SL-048's unified relation seam (`relation::tier1_edges`) + a total
  `RelationLabel → RelationType` map — *not* per-kind structs (design §5.3, D7).
- **Composed-spec body assembled inline** from `members.toml` + `interactions.toml`.

**Affected surface (concrete)**

- Read: `src/spec.rs`, `src/requirement.rs` (composition layer), `src/relation.rs`
  (unified edge seam, SL-048), `src/state.rs` (`PhaseRollup`).
- New `src/lazyspec.rs` (wire structs + pure `project`) and a new `export lazyspec`
  verb at the command layer, riding the SL-025 render spine.
- JSON serialization (serde). Layering held: leaf ← engine ← command (ADR-001); the
  command is the impure shell, projection logic stays pure (date/uid injected).

**Verification / closure intent**

- JSON **conformance tests** pin the §3 schema golden-file style — schemas are
  version-fragile, same medicine as `mem.pattern.parse.toml-error-classification-fragile`.
- Field-level check against the brief's DocMeta map (every emitted field has a
  lazyspec home). lazyspec can't run in this repo, so conformance is schema + golden
  file, not a live round-trip.
- RO proof: the command is pure read + serialize — no mutation path exists to test.

### Assumptions / Dependencies

- SL-025 is landed (done + closed); its read APIs + `canonical_id` are production.
- **SL-028 landed (done).** Its lifecycle FSM is **9 states** (no `review`); the slice
  status map (design §5.3) is built on that set. Slice status is a free `String` with
  tolerated drift, so the map is total (default → `draft`).
- **SL-048 landed (done) — relation model migrated.** Cross-kind edges moved out of
  per-kind typed `Relationships` structs into a uniform `[[relation]]` block read via
  `relation::tier1_edges`/`targets_for`. The projection rides that seam (design §3, §5.3).
- SL-027 (done) DRY'd backlog test-fixtures into `write_fixture`/`Fixture`; **plus**
  `catalog::test_helpers` (`seed_slice`/`seed_adr`/`seed_requirement`/`seed_knowledge`/
  `relation_rows`) now exists — the golden corpus rides both, never re-rolling backlog
  TOML (re-opening ISS-001). Gaps: a backlog seed + a spec seed (design §9).

### Risks / Open Questions

Prior open questions are **resolved in design** (see design §7):
- **Edge → RelationType mapping** — settled, rebuilt on SL-048 (D7): a total
  `RelationLabel → RelationType` map (design §5.3), default → `related-to`;
  `descends_from`/`parent` → `implements` (graph-visible, D2); supersedes →
  `supersedes`; everything else → `related-to`. No `blocks` in v1 (dep/seq not
  projected).
- **Command shape** — settled (D1): aggregate `doctrine export lazyspec`, its own
  envelope, not a `Format` variant.
- **Node-set scope** — settled (D8): minimal v1 `{slice, spec, adr, backlog, plan}`;
  post-scope kinds (`POL`/`STD`/`RV`/`REC`/`REV`/`CM`/knowledge) deferred to **IMP-105**.

A round-2 inquisition (`inquisition.md`) verified the wire strings against lazyspec
source (OQ-3). A **round-3 re-validation** (2026-06-19, design §10) swept the whole
design after ~800 commits parked: lazyspec wire strings still exact; the relation
model (SL-048) and FSM (SL-028, 9-state) drift integrated; `catalog::test_helpers`
adopted; new kinds deferred (IMP-105). Residual risks tracked in design §8.

## Non-Goals

- The lazyspec fork — `StoreBackend::Doctrine`, cold-cache materialization,
  editor-`e` gating, the `.lazyspec.toml` preset (piece 4, lives in `../lazyspec`).
- doctrine mutation verbs — projection is read-only.
- Requirements as standalone lazyspec nodes.
- A parallel read/compose path — reuse SL-025's readers + `canonical_id`.
- Graph fidelity beyond an implements-tree (a known lazyspec-v1 limitation, not
  doctrine's concern here).

## Summary

One coherent change: doctrine emits a conformance-tested, read-only JSON projection
of its entities — specs (requirements inline) plus slices, adrs, backlog items, and
synthetic plan children — via `doctrine export lazyspec`, reusing SL-025's read APIs.

## Follow-Ups

- **Piece 4 (`../lazyspec`):** the doctrine backend fork off this slice's wire
  format + the shipped `.lazyspec.toml` preset. Its `materialize_doctrine_cache`
  must invoke `doctrine export lazyspec` — **renamed** from the brief's working
  `emit-lazyspec-brief --json` (D1, no-masquerade); the brief §7/§8 recipe still
  names the old form.
- **IMP-105 — extend the node set to post-scope kinds** (`POL`/`STD`/`RV`/`REC`/`REV`/
  `CM`/knowledge). Split out at round-3 re-validation (D8); rides this slice's wire
  format. Until then those kinds' inbound edges dangle harmlessly (design §5.5).
- **Later:** selectively re-enable mutations as doctrine grows lifecycle/transition
  verbs, mapping onto lazyspec's `DocumentStore` writes.
- **v1 limitation to revisit:** lazyspec's graph renders `Implements` only — so
  `descends_from`/`parent`/plan lineage shows as a DAG (mapped to `implements`, D2),
  but `blocks`/`supersedes`/`interactions` stay panel-only until lazyspec's graph
  view widens (a v2 upstream ask to lazyspec).
