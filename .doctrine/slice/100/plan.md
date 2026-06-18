# Implementation Plan SL-100: Memory lifecycle verbs and agent UX hardening

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four phases deliver the three write verbs (`tag`, `status`, `edit`) then the
agent skills that document them. The verbs land before the skills because the
skill prose must only cite verbs/flags that exist (PHASE-04 VA-1). Each verb
phase ends green with its own tests; existing backlog/memory suites stay green
unchanged (behaviour-preservation gate, design § Architecture).

## Sequencing & Rationale

**PHASE-01 — tag leaf extraction + `memory tag`.** Sequenced first because it is
the only phase carrying a shared-machinery edit: `normalize_tag` moves from
`src/backlog.rs` to a new `src/tag.rs` leaf (ADR-001), and backlog must keep its
tag tests green unchanged. Doing the extraction first isolates that risk from the
verb work. The phase has no code dependency on status/edit, so it could run
file-disjoint in parallel with PHASE-02 if dispatched — but the plan orders it
first to retire the extraction risk before building on the tag seam.

**PHASE-02 — `memory status` + the pure transition core.** Must precede PHASE-03:
`edit --status` delegates to `memory_status_transition`, the pure core authored
here. The core is thin — it composes the existing `dep_seq::apply_status`
(verified pure, `src/dep_seq.rs:287`) under a `Status::parse` vocabulary gate; no
new IO seam is invented. `status` itself uses the `set_authored_status` IO shell;
`edit` will call the pure core directly on its own held document so the single-
write / single-`updated`-stamp invariant holds (design D2, RV-086 F-2). `--by`
superseded reuses `append_memory_relation` (ADR-004 §5 carve-out).

**PHASE-03 — `memory edit`.** Depends on PHASE-02 for the status core. Everything
else is local: `apply_edit` is a pure multi-field mutation over a held
`DocumentMut`, stamping `updated` once. The `--key` Option guard (late-bind iff
`None`) and `normalize_key` parity with `record` are the sharp edges (RV-086 F-1,
F-3); scope arrays replace, not append (R3, deferred follow-up).

**PHASE-04 — skills.** Last, because it documents verbs PHASE-01..03 deliver.
Updates `record-memory` and `retrieve-memory`; authors `reviewing-memory` and
`dreaming`. Deferred features (`--lifespan ""` clear, scope-append) must NOT
appear in the prose — VA-1 guards that.

## Notes

- Phase boundaries are verb-disjoint; only PHASE-03 has a hard intra-slice code
  dependency (PHASE-02's transition core). PHASE-01 is independent.
- Deferred to follow-ups (design R2/R3, OQ1/OQ3): `edit --lifespan ""` to clear,
  and scope-array append. Out of scope for every phase here.
- `dreaming` is an addition beyond `slice-100.md`'s original four skill items but
  is fully specified in design D4 — the plan carries the design, not the older
  scope draft.
