# Implementation Plan SL-015: Spec entity v1: product + technical specs

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.

## Overview

Six phases land the materialised-graph design (specs as thin aggregate roots,
requirements as peer entities, membership as a labelled edge) on the unchanged
`entity.rs` engine. The spine is dependency-ordered: the durable atom first, then
the aggregate roots that reference it, then the verbs that write / read / lint the
edges between them, then the canon reconciliation that retires the overturned
model. Every phase ends green and holds the behaviour-preservation gate
(entity/slice/adr/memory/skills suites unchanged) — the gate is the proof, so it
is an entrance criterion at the start and a verification at each step.

## Sequencing & Rationale

**Why this order — each phase is the precondition of the next, never a barrier.**

- **PHASE-01 (requirement)** is first because it is the durable thing everything
  else points at. It is the smallest, most foundational caller — a pure mirror of
  `adr.rs` — and it is where the second identity shape exercises the engine's
  claim seam. Landing it alone proves the "new caller, engine unchanged" thesis
  before any spec leans on it. It deliberately ships **no CLI** (requirement is
  spec-mediated, §5.2): the atom is real before the workflow that creates it.

- **PHASE-02 (spec new + list)** adds the two aggregate-root subtypes and the
  read-only list surface. It is sequenced second because `spec req add` (P3) needs
  a spec dir with a **seeded-empty** `members.toml` to append to — so the scaffold
  must exist first. `list` rides here (not later) because it is a pure mirror of
  `adr list` and proves the shared-`Meta` reuse is genuinely additive (the `title`
  field + `render_table` for `#members`) the moment specs exist, with `#members`
  reading the seeded 0.

- **PHASE-03 (spec req add)** is the integration point — the first phase that
  joins the two trees. It depends on both P1 (the requirement kind to reserve) and
  P2 (a spec with a members file to append to). It is isolated as its own phase
  because the two-tree non-atomicity, the auto-label assignment, and the
  edit-preserving append are the genuinely novel mechanics — the orphan/torn-write
  contract (C5) and canonical-prefix resolution (C4) both land and are tested here.

- **PHASE-04 (show / render)** comes before validate because render is the
  headline D5 deliverable — the only readable whole — and it depends only on what
  P1+P3 produced (requirement entities + member rows). Proving the pure local
  reassembly early de-risks the load-bearing read path; its purity (no write, no
  cross-corpus scan) is exactly what makes D9's "cannot go stale" claim hold.

- **PHASE-05 (validate / registry seed)** is the safety net over the same parsed
  data, sequenced after the writes (P3) and the read (P4) so there is a real
  corpus to lint. It introduces `registry.rs` minimally (FK-existence + uniqueness
  + orphan only; no cache, no cycle detection). The scoped-vs-corpus split
  (external review item 3) and the hard orphan severity (C5) are the subtle
  contracts proven here.

- **PHASE-06 (canon sweep + skills + close)** is last because the four-file doc
  rewrite should describe **shipped reality**, not anticipated design — the canon
  reconciles to what the prior phases actually built. It discharges inquisition
  C1/C3 (the widened sweep + the corrected false-witness against `relation-index`
  and the additive `glossary` edit), drops the "not yet structural" skill caveats,
  and harvests durable findings. Its acceptance test is the design's own grep gate.

**Coherent-subset boundary (D7).** The `feature` entity, the dependency DAG +
cycle validation, multi-frame grouping, the relation-index *cache*, coverage
computation, and `spec req link` (reuse an existing requirement under another spec)
are **designed-deferred** — forward-compatible because they are all edges over
already-reserved requirements. No phase here builds them; PHASE-06's notes record
them as the named next-slice surface.

## Notes

- **No registry in v1**, so `[specs]`/`[requirements]` stay empty in `plan.toml`
  (the spec/requirement registry this slice seeds does not yet index the plan).
- **The gate is the proof.** `entity.rs` is never modified; the existing suites
  staying green unchanged is the behaviour-preservation evidence, asserted as a VT
  in every phase that adds a caller (P1/P2).
- **Disposition provenance.** C1–C6 + Q4 (inquisition) and the five external-review
  repairs are integrated in `design.md` §10; this plan threads each into the phase
  whose criteria enforce it — C2→P1/P2, C4→P3, C5→P3/P5, C6→P2, C3/C1→P6,
  review-item-3 (scoped validate)→P5, review-item-4 (tech-only interactions)→P5,
  review-item-5 (`spec req link` deferral)→P6 notes.
