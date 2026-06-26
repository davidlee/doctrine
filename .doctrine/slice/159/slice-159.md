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

The **edge surface** (`supports` / `disputes`, EVD-`disputes`-INV, HYP
confirm/refute consuming EVD) is RFC-009 D3 — **open**, out of scope here. This
slice lands the kinds + lifecycles + the CON→INV replacement; new kinds inherit
existing `RECORD` edges (`shapes`, `spawns`, `governed_by`, `supersedes`) with no
`RELATION_RULES` change (per `rfc/009/edge-validation.md`).

## Scope & Objectives

1. **EVD kind** — `RecordKind::Evidence`; status set + hidden + terminal arrays
   per the lifecycle above; scaffold; `RECORD` grouping membership; template;
   facets noted for `/design` (`datum`, `provenance`, `confidence`, `supersedes`).
2. **HYP kind** — `RecordKind::Hypothesis`; status set + terminal arrays;
   scaffold; `RECORD` membership; template; facets (`proposition`, `predicts`,
   `tested_by`) noted for `/design`.
3. **CON → INV replacement** — rename `Constraint` → `Invariant` across the kind
   catalog (enum, KIND struct, status/hidden/terminal arrays, scaffold, prefix,
   `RECORD` grouping); migrate seed CON-001 → INV-001; rename
   `knowledge-constraint.toml` template → invariant; resolve `ConstraintSource`
   (`src/knowledge.rs:308`) — keep as `InvariantSource` or reshape (a `/design`
   decision). Update install docs / skills referencing the constraint kind.
4. **Governance axis** — the catalog change routes through a **Revision**
   (ADR-013). Per the agreed plan: **cut the Revision after design, settle it in
   reconciliation** — not authored at scope time.

## Non-Goals

- D3 edge modelling: `supports` / `disputes`, EVD-`disputes`-INV, HYP
  confirm/refute transition verbs. New kinds inherit `RECORD` edges only.
- D2 latent-taxonomy corpus survey (risk, mitigation, principle, procedure,
  interaction, responsibility, edge case, candidate solution).
- D4 concept-map reify / reified-concept (DEF/CPT) kind.
- D5 skill-uptake program beyond mechanical updates to keep existing skill/doc
  references coherent with the renamed/added kinds.
- Tier 2 (spec-as-graph). RFC-009 stays open; this slice does not close it.
- Closing RFC-009 or authoring its broader Revision.

## Affected Surface

- `src/knowledge.rs` — `RecordKind` enum + per-kind KIND/STATUSES/HIDDEN/TERMINAL
  arrays, scaffolds, `ConstraintSource`.
- `src/kinds.rs` — `RECORD` grouping const + its test.
- `install/templates/knowledge-*.toml` — add evidence + hypothesis; rename
  constraint → invariant.
- `install/using-doctrine.md`, `install/templates/seed-onboarding.md`, glossary —
  references to the constraint kind and the kind list.
- Seed record CON-001 → INV-001 migration.

## Risks / Assumptions / Open Questions

- **R1** — CON→INV is a destructive rename of a shipped kind. Behaviour-preservation
  gate: existing knowledge-record suites are the proof; migration must keep them
  green (adjusted for the rename, not broken). Seed CON-001 migration must not
  orphan inbound relations.
- **A1** — new kinds inherit `RECORD` edges with no `RELATION_RULES` change
  (asserted by `rfc/009/edge-validation.md`; verify in design).
- **OQ1** — `ConstraintSource` fate under INV: keep/rename/drop. `/design`.
- **OQ2** — prefix allocation for EVD / HYP / INV; INV must not recycle the CON
  prefix (RFC-009 D4 warns history would mislead).
- **OQ3** — migration mechanics for the shipped CON-001 seed (in-place rewrite vs
  supersede).

## Verification / Closure Intent

- New kinds creatable via the knowledge CLI with correct lifecycle status sets;
  invalid transitions rejected; gating partition (settled/unsettled) correct for
  EVD + HYP.
- CON fully retired: no `Constraint` kind authorable; INV authorable in its place;
  seed migrated; templates + docs coherent.
- `just gate` green; existing record suites green post-rename.
- Revision cut after design, disposed/settled through reconciliation; RV ledger
  clean at close.
