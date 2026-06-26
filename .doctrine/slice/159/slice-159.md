# Epistemic kind catalog: add EVD + HYP, replace CON with INV

## Context

RFC-009 (epistemic records as the human-facing relational substrate for design
ambiguity) carries three kind-catalog changes that are **locked in draft**,
independent of the RFC's still-open deliberation (D2 corpus survey, D3 edge bulk,
D4 concept-map reify, Tier 2). This slice carries only those locked changes:

- **EVD (evidence)** — a captured datum, with provenance, that `supports` /
  `disputes` other records. Replaces the rejected OBS catch-all (names a *role*,
  not a topic). Lifecycle `captured → confirmed | disputed | retracted`.
  Settled-for-gating: `{confirmed, retracted}`; unsettled: `captured`, `disputed`.
  `confirmed` is gating-inert but **not** lifecycle-final — may re-`disputed`
  (reopen) or be `superseded`; only `retracted` is terminal.
- **HYP (hypothesis)** — a testable proposed answer to a question. Distinct from
  QUE (the unsettled matter) and ASM (proceed-as-if vs let's-find-out). Lifecycle
  `proposed → confirmed | refuted` (both terminal).
- **CON → INV** — replace constraint ("boundary that must not be crossed") with
  invariant ("a property that must hold"). Near-duals; crisp-edge bar admits one
  framing; INV is the crisper, engineering-appropriate one. **Replace, not
  sibling** (sibling reintroduces the overlap D1 warns against).

EVD and HYP are **decoupled** (RFC-009 retracts the OBS↔HYP both-or-neither
coupling); each stands on its own merits. They compose (EVD is what settles a HYP)
but neither depends on the other.

The kinds land **fully modelled** (design decision D1): EVD's `supports` /
`disputes` edges are **in scope** — authored via `link` as `Writable` labels
(EVD→record), rendered in `show`, with transitions kept manual via `status`. Only
the broader RFC-009 D3 surface — the `shapes` epistemic-vs-affects *role* split —
stays open. New kinds also inherit the existing `RECORD` edges (`shapes`, `spawns`,
`governed_by`, `supersedes`); note the RFC's "no `RELATION_RULES` change" was
**wrong** — the `Shapes` target set and `GovernedBy` source set hardcode the record
list and must be edited.

Sequencing: this slice **rebases on SL-158** (Trinary actionability), which lands
first and turns the priority partition trinary. EVD/HYP therefore gate correctly on
arrival (a work item can `needs → EVD-captured`, blocked until `confirmed`).

## Scope & Objectives

_Full detail in `design.md` §5; this is the scope summary._

1. **EVD kind** — `RecordKind::Evidence`; statuses
   `captured, disputed, confirmed, retracted, superseded` (seed `captured`); facet
   `datum, provenance(Provenance enum), confidence(reuse Confidence)`. `confirmed`
   deliberately non-terminal (reopenable / supersedable).
2. **HYP kind** — `RecordKind::Hypothesis`; statuses `proposed, confirmed, refuted`;
   facet `proposition, predicts` (`tested_by` dropped — derived from the edge).
   Not supersedable (D7).
3. **CON → INV replacement** — faithful rename **+ `waived → relaxed`** (status) and
   facet `waiver_* → relaxation_*`; `ConstraintSource → InvariantSource` (variants
   kept). Migrate seed CON-001 → INV-001 by **in-place rewrite** (D6); rename tree
   dir + template.
4. **`supports` / `disputes` edges** — new `RelationLabel`s (full plumbing: enum,
   `name()`, parser, order pin, canaries), `sources:[EVD]`, `target:Kinds(RECORD)`,
   `Writable`; rendered in `knowledge show`/JSON.
5. **Catalog wiring across ~18 sites** — `kinds.rs`, `knowledge.rs`, `integrity.rs`,
   `priority/partition.rs` (trinary rows), `relation.rs` + `relation_graph.rs`,
   `supersede.rs` + `commands/supersede.rs`, `search.rs`, `tag.rs`, templates, docs,
   shipped memory, e2e goldens.
6. **Governance axis** — the catalog change routes through a **Revision** (ADR-013):
   **cut after design, settle in reconciliation** — not authored at scope time.

## Non-Goals

- The RFC-009 D3 `shapes` **role split** (epistemic-vs-affects) and concept-map
  edge types. `supports`/`disputes` ARE in scope; the shapes disambiguation is not.
- An evidence→status **automation** engine. `supports`/`disputes` are authored
  edges; HYP/EVD transitions stay manual via `status` (author's judgment).
- RSK as a `supports`/`disputes` target (RECORD-only; widen later if needed).
- D2 latent-taxonomy corpus survey (risk, mitigation, principle, procedure,
  interaction, responsibility, edge case, candidate solution).
- D4 concept-map reify / reified-concept (DEF/CPT) kind.
- D5 skill-uptake program beyond mechanical updates to keep existing skill/doc
  references coherent with the renamed/added kinds.
- Tier 2 (spec-as-graph). RFC-009 stays open; this slice does not close it.
- Closing RFC-009 or authoring its broader Revision.

## Affected Surface

The design-target selectors (`doctrine slice selector list SL-159`) are the
authoritative touch-set. Summary (~18 sites): `src/knowledge.rs`, `src/kinds.rs`,
`src/integrity.rs`, `src/priority/partition.rs`, `src/relation.rs`,
`src/relation_graph.rs`, `src/supersede.rs`, `src/commands/supersede.rs`,
`src/search.rs`, `src/tag.rs`; `install/templates/knowledge-*.toml` (add evidence +
hypothesis, rename constraint → invariant), `install/using-doctrine.md`,
`install/glossary.md`; `memory/mem.signpost.doctrine.knowledge`;
`tests/e2e_knowledge_cli_golden.rs`, `tests/e2e_memory_anchoring.rs`; the seed
CON-001 → INV-001 data migration.

## Risks / Assumptions / Open Questions

(Full register in `design.md` §6/§8.)

- **R1** — destructive rename of a shipped kind; behaviour-preservation gate —
  existing record suites must stay green (adjusted, not broken). Grep
  `Constraint|CON|waived` to zero before close.
- **R2** — SL-158 must land first (`git fetch . edge:main` before execute).
- **OQ1 (was: ConstraintSource fate)** — **resolved:** rename → `InvariantSource`,
  variants kept.
- **OQ2 (prefixes)** — **resolved:** `EVD` / `HYP` / `INV`; CON prefix removed, not
  recycled.
- **OQ3 (seed migration)** — **resolved:** in-place rewrite (D6).
- **OQ4** — does SL-158's `is_record` read `kinds::RECORD` (auto) or hardcode?
  Resolve at execution against merged SL-158 (design OQ-1).

## Verification / Closure Intent

- All 6 kinds creatable via CLI with correct lifecycle status sets; invalid
  transitions rejected; trinary gating correct for EVD/HYP (end-to-end `needs` test).
- `supports`/`disputes` authorable (EVD-only) and rendered in `show`/JSON.
- CON fully retired: no `Constraint` authorable; INV in its place; seed migrated;
  search/tag/templates/docs/goldens coherent.
- `just gate` green; existing record suites green post-rename.
- Revision cut after design, settled through reconciliation; RV ledger clean at close.
