# REV REV-061 — reconcile SL-262

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

<!-- Why this revision: what authored truth needs to change and why, the scope of
     the staged delta, and (for ADR/POL/STD/prose rows) the before/after excerpts
     the structured payload only labels. Seeded at `revision new`. -->

Reconciles SL-262 (the turn envelope's derived `forward` edge replaces the
writerless `next_obligation`) into the specs it moved. Driven by RV-383 `F-4`
(design sec-6 *Spec revisions*).

### REQ-414 (PRD-019) — modify

The pending obligation is no longer *held*; it is derived on every read
(`DEC-290`).

- description before: "The framework holds the run's stage, pending obligation,
  refresh boundary, and submission identity, and resolves the next obligation
  from that state rather than relying on the agent to recall where it is."
- description after: "The framework holds the run's stage, refresh boundary, and
  submission identity, and derives the next obligation from that state on every
  read rather than storing it or relying on the agent to recall where it is."
- criterion 1 after: "The next obligation is derived from persisted run state on
  every read, not stored and not taken from conversational recall."
- criterion 2 after: "A run's stage and next obligation are readable without
  consulting the transcript."

### REQ-437 (SPEC-029) — modify

The forward edge is a new bounded, no-drop projection part (`DEC-293`).

- description after: "Named limits bound the ordinary projection's frontier,
  change summary, blockers, declaration detail, and forward edge, and hold
  against a run large enough to exceed each of them."
- criterion 2 after: "The bounding fixture exceeds every limit before
  projection, and a maximal forward edge per embedded runbook renders within the
  normal budget."

### SPEC-029 — modify responsibilities

The gate-contract-table responsibility lists the table's projections; the
envelope's forward edge joins them.

- before: "… the rules that discharge it, the refusal's remedy, the stage-entry
  receipt and the published stage diagram are projections of that single table
  …"
- after: "… the rules that discharge it, the refusal's remedy, the stage-entry
  receipt, the turn envelope's forward edge and the published stage diagram are
  projections of that single table …"

## Reconcile narrative (SL-262)

- [RV-383 F-4]: REQ-414 — obligation derived, not held (DEC-290).
- [RV-383 F-4]: REQ-437 — forward edge among the named limits; maximal-forward
  bounding case (DEC-293).
- [RV-383 F-4]: SPEC-029 responsibilities — forward edge is a projection of the
  gate contract table.
- DEC-293 supersedes the envelope half of SL-233 `EX-15`; no spec carries EX-15,
  so no row.
