# Epistemic kind catalog: add EVD + HYP

## Context

RFC-009 (epistemic records as the human-facing relational substrate for design
ambiguity) carries three kind-catalog changes locked in draft. This slice carries
**two** — the additions EVD and HYP. The third (**CON → INV**) was **split out to
SL-160** (2026-06-27): its `waived → relaxed` semantics are unsettled and warrant
their own design pass rather than blocking these clean additions.

- **EVD (evidence)** — a captured datum, with provenance, that `supports` /
  `disputes` other records. Replaces the rejected OBS catch-all (names a *role*,
  not a topic). Lifecycle `captured → confirmed | disputed | retracted`.
  Settled-for-gating: `{confirmed, retracted}`; unsettled: `captured`, `disputed`.
  `confirmed` is gating-inert but **not** lifecycle-final — may re-`disputed`
  (reopen) or be `superseded`; only `retracted`/`superseded` are terminal.
- **HYP (hypothesis)** — a testable proposed answer to a question. Distinct from
  QUE (the unsettled matter) and ASM (proceed-as-if vs let's-find-out). Lifecycle
  `proposed → confirmed | refuted` (both terminal).

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

EVD's `supports`/`disputes` targets the `RECORD` family, which includes **CON in the
interim**; when SL-160 renames CON→INV, those edges carry through unchanged.

Sequencing: built on the **landed SL-158** (Trinary actionability) — EVD/HYP gate
correctly on arrival (a work item can `needs → EVD-captured`, blocked until
`confirmed`). **SL-160** (CON→INV) sequences `after` this slice — shared touch-site
files, serial edits.

## Scope & Objectives

_Full detail in `design.md` §5; this is the scope summary._

1. **EVD kind** — `RecordKind::Evidence`; statuses
   `captured, disputed, confirmed, retracted, superseded` (seed `captured`); facet
   `datum, provenance(Provenance enum), confidence(reuse Confidence)`. `confirmed`
   deliberately non-terminal (reopenable / supersedable).
2. **HYP kind** — `RecordKind::Hypothesis`; statuses `proposed, confirmed, refuted`;
   facet `proposition, predicts` (`tested_by` dropped — derived from the edge).
   Not supersedable (D7).
3. **`supports` / `disputes` edges** — new `RelationLabel`s (full plumbing: enum,
   `name()`, parser, order pin, canaries), `sources:[EVD]`, `target:Kinds(RECORD)`,
   `Writable`; rendered in `knowledge show`/JSON.
4. **Catalog wiring across ~17 sites** (4 → 6 kinds) — `kinds.rs`, `knowledge.rs`,
   `integrity.rs`, `priority/partition.rs` (trinary rows), `relation.rs` +
   `relation_graph.rs`, `catalog/scan.rs` (dispatch arm — **panic-grade**, codex-2
   F1) + `catalog/test_helpers.rs`, `supersede.rs` + `commands/supersede.rs`,
   `dep_seq.rs`, `search.rs`, `tag.rs`, two new templates, docs, shipped memory,
   e2e goldens.
5. **Governance axis** — routes through a **Revision** (ADR-013): cut after design,
   settle in reconciliation — not authored at scope time.

## Non-Goals

- **CON → INV** — split to SL-160.
- The RFC-009 D3 `shapes` **role split** (epistemic-vs-affects) and concept-map
  edge types. `supports`/`disputes` ARE in scope; the shapes disambiguation is not.
- An evidence→status **automation** engine. `supports`/`disputes` are authored
  edges; HYP/EVD transitions stay manual via `status` (author's judgment).
- RSK as a `supports`/`disputes` target (RECORD-only; widen later if needed).
- D2 latent-taxonomy corpus survey; D4 concept-map reify; Tier 2.
- The IMP-184 DRY refactor of the hardcoded prefix sites (add EVD/HYP at each site
  in place; centralisation is separate work).
- Closing RFC-009 or authoring its broader Revision.

## Affected Surface

The design-target selectors (`doctrine slice selector list 159`) are the
authoritative touch-set. Summary (~17 sites): `src/knowledge.rs`, `src/kinds.rs`,
`src/integrity.rs`, `src/priority/partition.rs`, `src/relation.rs`,
`src/relation_graph.rs`, `src/catalog/scan.rs`, `src/catalog/test_helpers.rs`,
`src/supersede.rs`, `src/commands/supersede.rs`, `src/commands/dep_seq.rs`,
`src/search.rs`, `src/tag.rs`; `install/templates/knowledge-evidence.toml` +
`…-hypothesis.toml` (two new), `install/using-doctrine.md`, `install/glossary.md`;
`memory/mem.signpost.doctrine.knowledge`; `tests/e2e_knowledge_cli_golden.rs`,
`tests/e2e_memory_anchoring.rs`. No seed migration (pure additions).

## Risks / Assumptions / Open Questions

(Full register in `design.md` §6/§8.)

- **R1** — a hardcoded literal record-prefix site is missed (no drift canary on
  `scan.rs`/`dep_seq.rs`/`search.rs`/`tag.rs`/`integrity.rs:817`); `scan.rs`
  omission is a debug-build panic. Grep every cluster
  (`mem.pattern.doctrine.record-kind-touch-sites`) before close.
- **R2** — SL-160 (CON→INV) edits the same lines; lands `after` this slice, serial.
- **R3** — `mem.signpost.doctrine.knowledge` (shipped) drifts; update + re-embed +
  `memory sync`.
- **OQ1** — `Provenance` closed 4-set vs free-text escape — default closed.
- **OQ2** — `is_record`/partition hardcode prefixes vs read `kinds::RECORD` — out
  of scope; IMP-184.

## Verification / Closure Intent

- All 6 kinds creatable via CLI with correct lifecycle status sets; invalid
  transitions rejected; trinary gating correct for EVD/HYP (end-to-end `needs` test).
- `supports`/`disputes` authorable (EVD-only) and rendered in `show`/JSON.
- EVD/HYP reachable by search/tag; templates/docs/goldens coherent; `scan.rs` arm
  present (no debug panic).
- `just gate` green; existing record suites green.
- Revision cut after design, settled through reconciliation; RV ledger clean at close.
