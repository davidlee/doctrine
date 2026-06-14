# Implementation Plan SL-060: Cross-kind dep/seq capture: extend needs/after authoring beyond backlog

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-060 extends the typed dep/seq axis (`needs`/`after`) from backlog-only to
slices. The work splits into one canon move, three code phases, and one data-only
backfill. The decomposition follows the design's two structural facts: the
consumer (`priority/graph.rs`) is *already* cross-kind (DD-2) — one read-gate away;
and the capture surface is welded to backlog and must be *lifted*, not copied
(no parallel implementation). So the code is a lift (PHASE-02), an author surface
(PHASE-03), and a one-gate consumer generalisation (PHASE-04).

## Sequencing & Rationale

- **PHASE-01 first — canon moves before code (D6).** PRD-011 is amended to claim
  the cross-kind *capture* intent before any phase builds on it. Cross-kind
  capture is currently owned by neither spec (PRD-009 scopes `needs`/`after` to a
  backlog item; PRD-011 disclaims the capture seam). Building code on an unclaimed
  intent would leave canon trailing the implementation — the inverse of the
  project's canon-first posture. Non-code, so it gates nothing technically; it
  gates *legitimacy*.

- **PHASE-02 — the leaf lift, in isolation.** The shared `src/dep_seq.rs` is the
  smallest honest DRY move (D1). Doing it as its own phase with the
  behaviour-preservation gate (INV-2, byte-identical backlog incl. verb message
  text) proves the lift is mechanical *before* any new behaviour rides on it. If
  PHASE-02 stays green with no golden churn, the risk that the lift perturbed
  backlog (R2) is closed before the cross-kind surface complicates the picture.
  `promoted` deliberately does **not** move (R3) — it is a backlog projection;
  the leaf has no `promoted` field.

- **PHASE-03 before PHASE-04 — author before consume.** The author surface (verbs
  + validation + scaffold seed + slice read arm) is independently verifiable:
  `doctrine needs SL-A SL-B` writes the edge and `slice show` round-trips it,
  with no consumer involved. The D2 work-like target gate lands here, at author
  time — it is the single guard that keeps SL-060 from silently leaking IMP-047's
  governance-blocks-work topology (the consumer blocks on *any* non-terminal dep
  predecessor regardless of kind, so "allowed-but-inert" is false; E1/E4). The
  backlog verbs become thin delegates but **retain** their pre-write cycle refuse
  (E3) — only `append` delegates to the leaf; the generic slice verb defers cycle
  diagnosis to read-time (no cross-kind author-time oracle in scope).

- **PHASE-04 — the consumer, one gate.** With slices authoring edges, the engine
  dispatch (`dep_seq_for`, mirroring `outbound_for`) and the generalised read-gate
  surface them through the unchanged, already-kind-agnostic overlay machinery.
  Non-authoring kinds short-circuit with no disk read (F5) — the read loop now
  visits every kind, not five. Backlog stays byte-identical here too (priority
  goldens, `promoted` projection).

- **PHASE-05 last — backfill, data-only.** The strict leaf never creates the
  `[relationships]` table (D5); pre-existing slices in this dogfood repo lack it,
  so authoring `needs`/`after` *onto* them would hit the strict refuse until
  seeded. The backfill establishes the runtime INV-1 invariant for those slices.
  It runs last because it must match the seeded-table contract that PHASE-02/03
  lock, and because it is project-local ops (ASM-1) — kept out of the durable
  code, committed as a pure data diff. New phases scaffold the table from birth,
  so the backfill is a one-off, not an ongoing seam.

## Notes

- The phase boundaries are file-coherent: PHASE-02 owns `src/dep_seq.rs` +
  `src/backlog.rs` re-point; PHASE-03 owns `main.rs` verbs, `src/slice.rs`,
  `install/templates/slice.toml`; PHASE-04 owns the engine dispatch +
  `src/priority/graph.rs` read-gate. The emission path (`graph.rs:257-277`) is
  touched by nobody (DD-2).
- IMP-047 rides this slice's `dep_overlay` seam afterwards (labelled `gates`
  producer); SL-060 leaves the overlay open but unifies no producers (D7).
- The two non-code deliverables (PHASE-01 canon, PHASE-05 backfill) are real
  phases per F6 — not bundled silently into code phases.
