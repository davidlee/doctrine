# Implementation Plan SL-066: Revision entity: pending revise-intent and staged-delta vehicle

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Five phases ship `REV-NNN` as a standalone work-lifecycle kind on the change
axis, unifying IDE-003 (staged-delta vehicle) and IDE-010 (pending revise-intent
a slice can `needs`). The design is locked (three adversarial passes integrated —
design.md §11/§12/§13). The phase boundaries follow design §7, refined here.

The shape is the standard new-authored-entity build, but with one non-obvious
twist that drives the sequencing: a `KINDS` row is not a passive registration —
it is *read by three separate corpus-walk tables*, and a row added ahead of any
of its three arms makes the debug-build corpus scan panic or mis-classify the
moment a REV is minted (Opus pass G1–G3). That coupling is why PHASE-02 is larger
than "scaffold the kind" and why apply (PHASE-05) is deliberately last and narrow.

## Sequencing & Rationale

**PHASE-01 (canon first).** Governance moves before code (ADR-003 / §1 authoring
ethos — explicitly *not* IMP-047, which is the unbuilt trinary-actionability
improvement, G7). ADR-013 fixes Revision's home as a change-axis kind and records
the governance→work dependency rule (depend on the Revision, never the evergreen
doc — the SL-060 invariant). The two doc relocations (`entity-model.md`
§Adjudication, `spec-entity-spec.md`'s deferred `REV-`) close the open question the
slice was chartered to close. Doc-only, so it verifies by agent coherence review
+ user acceptance (VA/VH) — no test oracle fits prose.

**PHASE-02 (kind + spine + all three corpus-walk arms together).** The kind, its
backlog-style lifecycle (`proposed→started→done` +`abandoned`), the orthogonal
`approval` field (ADR-009 "approval is not lifecycle"), and the full wiring seam
(KINDS row, manifest dir, gitignore negation — the `adr` trap). The hard
constraint: the `KINDS` row and its **three** consumers land in one phase, or the
build goes RED the instant a REV exists — (G1) `priority::partition` needs a
dedicated REV row + `REV_STATUSES` const, else a `done` REV classifies
`Unrecognised != Terminal` and a dependent never unblocks (the inverse of the
whole point); (G2) `dep_seq_for` needs a REV arm or REV-as-source edges are
silently dropped; (G3) `outbound_for`'s fallthrough is `debug_assert!(false)`, so
its REV arm must exist even though the accessor it calls stays empty until
PHASE-03. This phase ships `new`/`show`/`status` only — `change add`/`approve`/
`apply` come later.

**PHASE-03 (payload + reciprocity).** The `[[change]]` table and its derived
inbound view. `revises` is a `TypedVerbOnly` relation (authored by `revision
change add`, never `doctrine link` — F2); the table carries two row shapes
because creation ops have no FK to key on yet (F3), and creation rows freeze
`new_label` at authoring time so membership churn between draft and apply cannot
silently change what lands (E4/M2). Reciprocity is *derived* (ADR-004) via the
`relation_graph` `in_edges` index and surfaces on `inspect`, never `show` (ADR-004
§3 reserves inbound completeness to the scan-backed surface). This phase fills the
`outbound_for` accessor stubbed in PHASE-02. OQ-1's open detail-column/vocab
questions resolve here against the real table shape.

**PHASE-04 (dep/seq surface).** The IDE-010 payoff: add REV to the work-like
predicate as both source and target, and exercise the partition + `dep_seq_for`
arms (already present from PHASE-02) end-to-end — `needs REV-N` blocks until REV-N
is terminal, then unblocks. The SL-060 governance-exclusion invariant must survive
(a `needs ADR-X` is still refused). Small phase by design; its arms were front-
loaded into PHASE-02 to keep the corpus scan sound.

**PHASE-05 (apply, last and narrow).** `revision apply` auto-lands `status` rows
*only* (E1/E5) — the honest consequence of external B1/B2: the creation seams
(`spec req add`/`spec new`) are non-transactional CLI handlers, so auto-applying
them would risk orphaned half-writes one commit cannot undo. Status rides the
engine-callable `requirement::set_status` (no refactor — G4), composes one RecDoc
per row (REC untouched), and runs a pre-flight all-or-nothing sweep with the
`from`-guard (the one silent-clobber the dropped drift re-prompt opens). Apply
refuses unless `approval=approved` — a forcing-function checkpoint, not actor
authz (E3, ADR-009 invoker-blind). `done` means every row landed: status-only
REVs reach `done`, manual-carrying REVs hold at `started` (M1). Everything else is
surfaced-for-manual. Apply is last because it depends on the payload (PHASE-03)
and the lifecycle/dep machinery (PHASE-02/04) being real first.

PHASE-06 (`/revise` skill + workflow integration) is **out of this slice** — an
explicit follow-up (scope Non-Goals; IDE-003 tail), authored once the kind exists.

## Notes

- The grain mismatch with SL-044 (one-act-one-commit) is deliberate: a Revision
  apply is N status acts in one commit, each REC self-describing for NF-003
  reconstructability (design §4.5).
- Two honour-system gaps are named-and-accepted, not oversights (design §9): no
  machine check that surfaced-for-manual rows actually landed before `done`, and
  no reversal of landed status rows when a partially-applied REV is `abandoned`
  (those deltas are real reconciliations).
- Auto-apply for `introduce`/`create` returns once transactional
  `spec::add_requirement`/`spec::create_spec` engine helpers exist (external B2) —
  additive, no model change. Tracked as a slice Follow-Up.
