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
  measurement table from probe-specs § Measurements, all committed under
  `.doctrine/rfc/025/evidence/`; the raw run logs they summarise stay in the
  runtime tier at `.doctrine/state/rfc-025/raw/`.

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

- **R1 — RESOLVED during design; it became findings, not a blocker.** Traced to
  F1–F5 in `design.md` § 2.1. The gate (REQ-316) does not validate the *result*,
  it validates that a staging ritual completed (F4); and the candidate verbs
  read their journal out of the coordination branch, so they are structurally
  inseparable from the staging the capsule model removes (F5). Disposition: the
  rig reports the coupling rather than minting a synthetic `Verified` row —
  forging that row would hand-roll exactly what DQ-1 protects. The matrix splits
  instead (D8): every row runs the four-stage pipeline, and H10/H16 *additionally*
  run a scaffolded sub-probe against the real candidate layer — their pipeline leg
  proves the capsule model refuses a stale second result (stage-4 CAS), their
  sub-probe leg is an incumbent regression check that counts toward nothing
  (RV-340 F-9; design § 5.6). No `/consult` was needed; the operator ruled on the
  split directly.
- **R7 (new, RV-340 F-9) — conflict/staleness *resolution* is out of evidence.**
  The spike proves refusal, not admission. QUE-202 owns the gap and outlives this
  slice; the go/no-go must say so rather than reading 16/16.
- **R2:** the conform stage's reliance on `slice conformance --against …
  --strict`. Its `--strict` semantics may differ from the import belt's in some
  edge case; the rig skeleton probes this first. A genuine gap is a `/consult`,
  not an improvised `src/` change.
- **A1:** nested bwrap works inside the jail (ADR-008 D-B3 precedent;
  `pi-spawn-confined.sh` is the seed).
- **A2:** headless `claude -p` runs with the jail's `~/.claude` credential
  arrangement inside the capsule sandbox. Tested early as a standalone smoke,
  split into network-reachability and authentication assertions.
- **A3:** worker token cost is external to the orchestrating session
  (separate process), so probe *execution* is context-cheap for the driver;
  log volume is the context cost to manage.
- **OQ-1:** where probe evidence logs live long-term (RT-9 archive-tier
  question, small-scale instance). v0: committed text summaries + raw logs in
  the runtime state tier (design § 5.3, as amended).

## Verification / closure intent

Done when: every P-C1/P-C2 row and every P-C3 matrix row has a recorded
pass/partial/fail for both M-A and M-B (or a consulted deviation), EVD
records exist and are linked to QUE-200, the measurement table is filled,
and a go/no-go summary lands in `.doctrine/rfc/025/`. A failed row is a
finding, never a silent rig edit (probe-specs § Order and gating).

The go/no-go is **scoped**, not absolute: go on Linux/bwrap, for a client of
this build shape, with model-level rows proven portable and env-conditional
rows outstanding for macOS. Writing the scope in is what stops the downstream
REV over-claiming (design.md § 9).

## Summary

## Follow-Ups

- QUE-200 settle (`knowledge status`) once EVD suffices.
- RFC-025 cleanup pass folding spike evidence + census B1 note + RT-3/RT-9
  rulings.
- REV scoping for census DELETE rows if go.
