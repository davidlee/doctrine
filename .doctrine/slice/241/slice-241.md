# Capsule spike rig

## Context

RFC-025 proposes replacing the shared-worktree dispatch trust model with
execution capsules. The high-judgment groundwork is banked under
`.doctrine/rfc/025/`: `mechanism-census.md` (delete/transform/keep verdicts),
`red-team.md` (RT-1 verify-capsule blocker; RT-2/RT-3 binding constraints),
`probe-specs.md` (P-C1/P-C2/P-C3 experiment designs, 16-row hostile matrix),
plus QUE-200 (ingestion mechanism, settled only by probe evidence).

This slice builds and runs the probe rig. **The deliverable is evidence, not
product** — EVD records against QUE-200, measurements, and a go/no-go on the
capsule authority model. The rig is disposable scaffolding; nothing it
contains migrates into dispatch machinery.

## Scope & Objectives

- A rig (shell scripts under `scripts/spike-capsule/`) implementing the three
  probe specs: capsule provisioning (clone + manifest), the bwrap confinement
  profile, scripted hostile probes, harvest/ingestion (quarantine-fetch M-A
  and bundle M-B side by side), admission via the **existing** `candidate`
  verbs (probe-specs DQ-1), verify-capsule execution (RT-1).
- Execution of P-C1 (cost baseline), P-C2 (confinement matrix), P-C3
  (hostile matrix, both mechanisms) against a scratch clone of this repo.
- Evidence: EVD records linked to QUE-200 (`supports`/`disputes`), the
  measurement table from probe-specs § Measurements, and probe logs kept
  under `.doctrine/rfc/025/evidence/`.

## Non-Goals

- No changes to dispatch/worktree machinery, no new CLI verbs, no migration
  of any census row — that is post-spike REV/slice work.
- No FR-007 (confined-orchestrator) work; held pending per census B8.
- No RFC-025 prose edits (separate cleanup pass, #1 in the programme).
- No sandbox-profile productization (network tightening etc. noted, not built).
- Rust source changes are out of scope *by default*; if a rig step is blocked
  on a missing read verb or a candidate-verb precondition, `/consult` before
  touching `src/` (see risks).

## Risks / Assumptions / Open questions

- **R1 (highest):** DQ-1 requires admission through `candidate create`/
  `admit`, but the provenance gate (REQ-316) refuses sources that are not
  `Verified` stage-1 journal rows — the candidate layer presumes dispatch
  ledger state. The rig must either drive enough real dispatch machinery to
  mint journal rows for the scratch slice, or this friction is itself a
  finding (the capsule pipeline's binding point into the candidate layer).
  Expected first `/consult`.
- **A1:** nested bwrap works inside the jail (ADR-008 D-B3 precedent;
  `pi-spawn-confined.sh` is the seed).
- **A2:** headless `claude -p` runs with the jail's `~/.claude` credential
  arrangement inside the capsule sandbox.
- **A3:** worker token cost is external to the orchestrating session
  (separate process), so probe *execution* is context-cheap for the driver;
  log volume is the context cost to manage.
- **OQ-1:** where probe evidence logs live long-term (RT-9 archive-tier
  question, small-scale instance). v0: committed text summaries + gitignored
  raw logs.

## Verification / closure intent

Done when: every P-C1/P-C2 row and every P-C3 matrix row has a recorded
pass/partial/fail for both M-A and M-B (or a consulted deviation), EVD
records exist and are linked to QUE-200, the measurement table is filled,
and a go/no-go summary lands in `.doctrine/rfc/025/`. A failed row is a
finding, never a silent rig edit (probe-specs § Order and gating).

## Summary

## Follow-Ups

- QUE-200 settle (`knowledge status`) once EVD suffices.
- RFC-025 cleanup pass folding spike evidence + census B1 note + RT-3/RT-9
  rulings.
- REV scoping for census DELETE rows if go.
