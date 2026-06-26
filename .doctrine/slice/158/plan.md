# Implementation Plan SL-158: Trinary actionability

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-158 implements ADR-017 (gating = an inbound `needs` on an unsettled record).
Four design deltas (D1/D2/D3/D6 in `design.md`) land across three phases. Canon
(SPEC-001/PRD-011/SPEC-019, plus the ADR-017 source-only premise correction)
moves at **reconcile**, not as a code phase — it is not planned here.

The phases are cut along **file seam + dependency**, not along design-decision id:

- **PHASE-01 — D1, `partition.rs`.** The foundational delta: the third status
  class. Everything else assumes records can be Gating.
- **PHASE-02 — D2, `dep_seq.rs`.** The gate widening that makes the inbound edge
  authorable. The trap the planner must not skip: ADR-017's prose claims the
  work-like gate is source-only; it is not — the *target* is gated work-like too,
  so without D2 `needs SL QUE` stays refused and the whole mechanism is inert.
- **PHASE-03 — D6 + D3, `relation.rs` (+ confirm).** Records author `references`
  (concerns), and the estimate/value-on-records round-trip that earns its keep
  via optionality propagation. Independent file; sequenced last because it is the
  smallest behavioural surface and its scoring assertion reads cleanest once the
  partition is settled.

## Sequencing & Rationale

**Why this order.** PHASE-01 → PHASE-02 is a hard dependency: VT-3/VT-4 (gate
blocks, settle→unblock) are only meaningful once an `open` QUE classifies
**Gating** (blocks) rather than today's Terminal (settled). D2's VT-6
(admissibility) needs only D2, but landing D1 first lets PHASE-02 verify the full
block→settle→unblock cycle in one phase rather than deferring it.

PHASE-03 (D6/D3) is **file-disjoint** from both predecessors (`relation.rs` vs
`partition.rs`/`dep_seq.rs`) and has no code dependency on them — it could in
principle run in parallel. It is sequenced last on purpose: D3's optionality
assertion (a sized record raising its referenced target's score) is clearest to
reason about and golden once the partition behaviour is locked, and keeping it
serial avoids a shared-index race for a slice this small.

**The no-code-change output flip.** `channels.rs`, `graph.rs`, `surface.rs`,
`view.rs`, `render.rs` are *not* edited — pole reads (`== Workable` / `!=
Terminal`) and `{:?}` Debug rendering absorb the new variant. But `survey --all`,
`explain`, and `inspect` render `StatusClass` for any minted node, so an
unsettled record's output flips Terminal → Gating. This is **intended**, not a
regression. VT-8 (the knowledge-in-priority golden) is deliberately bound to
PHASE-01 — same phase as the cause — over a records-only corpus that carries no
needs-edges or references, so PHASE-02/03 cannot churn the golden.

**Behaviour-preservation gate.** This slice touches shared machinery (the
priority partition). Per the project gate, the existing suites are the proof:
they must stay green **unchanged** except the two flips design pre-declares as
consumer revisions (the knowledge-vocabulary canary and the
terminal-never-workable assertion). Any other movement is a regression to stop on.

## Notes

- **Two test flips are by design, not regressions** —
  `every_knowledge_status_classifies_terminal_never_workable` and
  `knowledge_partitions_cover_the_real_vocabularies` (PHASE-01).
  `decision_accepted_diverges_hidden_from_status_class` **stays green**.
- **D6 was added after the codex adversarial pass** — self-verified only. The
  optionality wiring claim (graph.rs:163, role-blind label filter) is the one to
  watch in PHASE-03; a focused review there is cheap insurance.
- **Split out of scope:** shapes-roles → IDE-022; surface estimate/value in
  show/inspect → IMP-183; full cross-tier dep/seq → IMP-033 (D2 widens only to
  records). ADR-017 prose correction → reconcile.
- **Jail:** writes need `DOCTRINE_RESERVATION_FALLBACK=1`; RW doctrine = build
  target. Lint denies `as`/`unwrap`/`expect`/`print_stdout`/`format_push_string`
  (Vec<String>+concat house style). Path-limit commits (shared index).
