# Implementation Plan SL-127: Dispatch base freshness: ancestor-dominant ladder and mid-drive refresh

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Five phases deliver the locked design (design.md, LOCKED 2026-06-20) along its two
axes. Axis 1 (base selected correctly at setup) is PHASE-01 (the ladder fold) +
PHASE-02 (the plan-presence gate). Axis 2 (base advanceable + drift observable
mid-drive) is PHASE-03 (the `refresh-base` verb + the `trunk_drift` extract) +
PHASE-04 (the diagnostics that surface drift where it bites). PHASE-05 wires the
capability into the skills and retires the `DOCTRINE_TRUNK_REF=main` workaround.

Each phase is test-first (red → green → refactor) and ends green. Verification is
`VT` (by test) throughout, save the two `PHASE-05` skill criteria that no test can
judge (`VT-1` greps for the retired ritual; `VA-1` is the ADR-011 "verb not prose"
conformance check).

## Sequencing & Rationale

**Why this cut.** The phases follow the file seams, which keeps each phase a single
cohesive unit and makes later parallel dispatch possible:

- **PHASE-01 — `src/git.rs`.** The `freshest_descendant` fold + `trunk_ladder`
  rewire. Foundation: it changes shared machinery used by every ladder caller
  (`trunk_commit`, `trunk_entity_ids` minting), so it lands first and its
  behaviour-preservation is proven by the existing `trunk_ladder_explicit_*` and
  `e2e_trunk_minting` suites staying green (VT-2/VT-3).
- **PHASE-02 — `src/worktree.rs`.** The `coordinate` Create-path plan-presence
  gate, placed *before* the fork (design F6) so a bad base never creates a worktree
  to roll back. Logically rides on PHASE-01 (it backstops the diverged pick the
  fold cannot resolve), but touches a disjoint file.
- **PHASE-03 — `src/dispatch.rs` + CLI.** Axis-2 core. `trunk_drift` is extracted
  from `run_status` first (behaviour-preserving — the reuse the design found, no
  parallel drift impl), then the `refresh-base` verb is built on it. The codex C4
  revision is load-bearing here: the verb runs a **real `git merge` in the live
  coordination worktree**, not an object-db `merge_tree` (which yields no resolvable
  conflict state). Conflict is report-and-halt, leaving `MERGE_HEAD` for the
  operator — mirroring SPEC-021 stage-2 discipline.
- **PHASE-04 — `src/dispatch.rs`.** Diagnostics on the same surface, after PHASE-03
  because both consume `trunk_drift`. Deliberately *diagnostic only* (codex C5/C6):
  the candidate-create hint is appended, never a cause verdict; the `RefreshBase`
  guidance fires on a computed drift fact, not a vague flag.
- **PHASE-05 — skills.** Doc-only routing + ritual retirement, last because it
  asserts the verb exists (PHASE-03) and the env prefix is genuinely unnecessary
  (PHASE-01/02).

**Parallelism.** PHASE-01/02/03 are file-disjoint (`git.rs` / `worktree.rs` /
`dispatch.rs`) and could dispatch concurrently; PHASE-04 and PHASE-05 both depend on
PHASE-03 (the helper / the verb) and serialise after it. Default execution is serial
(one phase at a time); the disjointness is recorded for a possible `/dispatch`.

**Invariant carried through.** Every phase honours RV-030 F-1 (the projection's
pinned fork-point): `refresh-base` advances the base only by an explicit, recorded
operator action that regenerates the bundle — the same shape as the already-
sanctioned self-base step (SPEC-021) — never a silent live-tip reparent.

## Notes

- **OQ-1 (verb scope)** — PHASE-03 ships `refresh-base` as **merge-only**; the
  operator re-runs `dispatch sync --prepare-review` afterwards (the SL-122 two-step).
  Bundling regen into the verb stays a deferred option, not built here.
- **Governance touches → reconcile REV (ADR-013).** The ladder reorder refines
  **ADR-006 D3** (amendment / DEC); `refresh-base` warrants a **SPEC-012** mechanism
  REQ + a **SPEC-021** between-phase-cadence note. Authored at reconcile, coverage
  reconciled not inferred — not planned as code phases.
- **Follow-ups (not in this slice):** rollback-safety for *all* causes (RSK-010
  cand. e), Resume-preflight (codex C3), `refresh-base --abort`, and the
  `[dispatch] trunk_preference` config (**IMP-126**, folds with IMP-124).
