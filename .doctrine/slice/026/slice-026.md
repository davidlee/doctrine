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

The emitter **rides SL-025** (uniform list/show/filter/**render** contract,
in-flight) — it is a new JSON *render target* on that shared spine, not a parallel
renderer. SL-025 is therefore an execution dependency.

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
2. A **new read-only CLI command** (working name `emit-lazyspec-brief`; final shape
   may be a render mode of SL-025's surface — see Open Questions) that projects
   doctrine entities into conformant JSON.

**Projection rules (the contract this slice owns)**

- **Node set:** `spec` entities → `composed-spec` nodes (`virtual: true`, body =
  assembled members); `slice`, `adr`, `plan` → their own nodes. **Requirements are
  NOT standalone nodes** — they render inline in the composed-spec body as
  `FR-`/`NF-` labelled entries (per the composition seam).
- **Every entity carries `validate_ignore: true`** (doctrine owns validation;
  `rules = []` does not empty lazyspec's rule set). **Emitted types are
  non-singleton** so `TypeConstraintChecker` stays satisfied — these two are
  load-bearing, from the brief.
- **Edges flatten** to lazyspec's four `RelationType`s
  (Implements/Supersedes/Blocks/RelatedTo); exotic edges → `RelatedTo`. Reciprocity
  is derived at projection time (ADR-004 — edges stored outbound-only).
- **Composed-spec body assembled inline** from `members.toml` + `interactions.toml`.

**Affected surface (concrete)**

- Read: `src/spec.rs`, `src/requirement.rs`, `src/registry.rs` (composition layer).
- New verb at the command layer, riding the SL-025 render spine.
- JSON serialization (serde). Layering held: leaf ← engine ← command (ADR-001); the
  command is the impure shell, projection logic stays pure (date/uid injected).

**Verification / closure intent**

- JSON **conformance tests** pin the §3 schema golden-file style — schemas are
  version-fragile, same medicine as `mem.pattern.parse.toml-error-classification-fragile`.
- Field-level check against the brief's DocMeta map (every emitted field has a
  lazyspec home). lazyspec can't run in this repo, so conformance is schema + golden
  file, not a live round-trip.
- RO proof: the command is pure read + serialize — no mutation path exists to test.

### Assumptions

- SL-025's render contract is settled (design locked even if code is 3/6) — this
  slice's design can proceed against it now.
- The in-flight JSON substrate is SL-025's render spine, not a separate seam.

### Risks / Open Questions

- **Edge → RelationType mapping** — how `descends_from` (spec→PRD, the what→how
  descent), `interactions` (spec→spec), and slice relations land in the four
  lazyspec types. `/design` decides; consequential because the lazyspec graph shows
  **Implements only** (brief §6).
- **Command shape** — standalone `emit-lazyspec-brief` vs a `--format lazyspec`
  render mode of SL-025's `show`/`list`. Resolve in `/design` once SL-025's final
  render contract is visible.
- **Execution gated on SL-025** (3/6). Design unblocked; execution waits.

## Non-Goals

- The lazyspec fork — `StoreBackend::Doctrine`, cold-cache materialization,
  editor-`e` gating, the `.lazyspec.toml` preset (piece 4, lives in `../lazyspec`).
- doctrine mutation verbs — projection is read-only.
- Requirements as standalone lazyspec nodes.
- A parallel JSON renderer — must ride SL-025.
- Graph fidelity beyond an implements-tree (a known lazyspec-v1 limitation, not
  doctrine's concern here).

## Summary

One coherent change: doctrine emits a conformance-tested, read-only JSON projection
of its entities — composed specs (requirements inline) plus slices/adrs/plans —
consumable by a lazyspec doctrine backend, riding SL-025's render spine.

## Follow-Ups

- **Piece 4 (`../lazyspec`):** the doctrine backend fork off this slice's wire
  format + the shipped `.lazyspec.toml` preset.
- **Later:** selectively re-enable mutations as doctrine grows lifecycle/transition
  verbs, mapping onto lazyspec's `DocumentStore` writes.
- **v1 limitation to revisit:** lazyspec's graph renders `Implements` only —
  doctrine's `blocks`/`supersedes`/descent edges are invisible *as a graph* until
  lazyspec's graph view widens (a v2 upstream ask to lazyspec).
