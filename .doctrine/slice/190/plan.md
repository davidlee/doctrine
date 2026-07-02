# Implementation Plan SL-190: Dispatch orchestrator state-visibility verbs

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Six phases deliver IDE-027's "now" half: CLI-first state-visibility verbs for the
dispatch/worktree surface. Each phase = a pure engine core + a thin command shell
(ADR-001), TDD red/green/refactor. The design (`design.md`) is canon; this plan
sequences its code-impact table into shippable units, each a clean conventional
commit. The RV-214 rework (composite grounded on committed `phase/*` refs, per-phase
resolver, status-only writer) is baked into the phase criteria.

Three verb-groups, one shared refactor:
- **§1 phase-status** (PHASE-01 core → PHASE-02 query → PHASE-03 reconcile) — the #1
  token sink; the split-brain query + the primary-tree reconcile.
- **worktree inventory** (PHASE-04 gc-oracle lift → PHASE-05 `worktree list`).
- **selector doctor** (PHASE-06) — independent advisory.

## Sequencing & Rationale

**Why PHASE-01 first, alone.** The pure `resolve_phase_truth` resolver is the
foundation both §1 verbs consume. Landing it standalone — total function, exhaustive
table tests, zero IO — closes the RV-214 F-2/F-4 blockers (per-phase maps, total
table) as verifiable engine code before any shell touches it. Everything downstream
depends on it; nothing depends on the shell.

**Why query (02) before reconcile (03).** Read before write. PHASE-02 builds the
input-gathering machinery (phase-ref `for-each-ref` helper, registry-cache read,
coord/local per-phase reads) that PHASE-03's reconcile reuses. Splitting read from
write is deliberate (IMP-191 just *removed* a reader/writer overload — not
reintroducing one) and gives two clean commits. PHASE-03 carries the two hardest
RV-214 fixes: the status-only writer (F-3 — `set_phase_status` is unusable here, it
mutates the registry it reads) and the live-coord refusal (F-5 — the cross-tree
write hazard confined to a refusable condition, not "sole-writer-as-lock").

**Why the gc-oracle lift (04) is its own phase, before inventory (05).** The lift is
a shared-machinery change to `gc.rs` — the behaviour-preservation gate applies (the
gc suite must stay green *unchanged*). Isolating it makes that gate unambiguous:
PHASE-04's diff is the extraction + target parameter, proven by (a) gc green
unchanged and (b) new non-HEAD-target tests. PHASE-05 then *consumes* the generalized
oracle for the `landed` column. Conflating them would blur which test proves which
claim (the exact RV-214 F-6 conflation we're correcting).

**Why selector doctor (06) is independent.** It touches `conformance.rs` +
`slice.rs` only, shares nothing with §1's runtime machinery. Ordered last as the
lowest-risk, self-contained advisory. Its predicate's *home* is fixed
(`conformance.rs`, F-7); only which slice lands it first vs SL-180 stays open (OQ-2)
— resolved via the SL-180 relation at execution time, not here.

**Dispatch/parallelism note.** Phases are mostly serial by file-overlap: 02/03/06 all
touch `slice.rs` + `commands/cli.rs`; 04/05 both touch `src/worktree/`. Not a
file-disjoint fan-out candidate — SL-190 is small enough to drive serially. If
dispatched, the natural batches are {01}, {02}, {03}, {04}, {05}, {06} in order.

## Notes

- **Verification posture.** Pure cores (resolver, classify_worktree, diagnose_selector,
  the oracle) are the primary evidence — total functions with exhaustive case tables,
  unit-tested inline. Shells get e2e coverage for exit codes, JSON shape, and the
  cross-tree fixtures. Every VT carries a `test_file` + `keywords` mandate so the S3
  `verify-vt` gate can check it at handover.
- **New test files** (`e2e_slice_phase_status.rs`, `e2e_slice_reconcile_phases.rs`,
  `e2e_worktree_list.rs`, `e2e_slice_selector_doctor.rs`) are created by their phases;
  the VT mandate names them so the gate confirms existence.
- **POL-002.** No host-build coupling anywhere: binary-freshness and provisioning
  checks stay excluded (design Non-Goals); `dispatch/<slice>` is ADR-012 doctrine
  topology (RV-214 cleared it).
- **Follow-up IDE-028** (auto primary-sheet-push in the Record beat) is out of scope
  — this slice ships the manual `reconcile-phases` it would later automate.
