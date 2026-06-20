# Implementation Plan SL-123: Claude dispatch arm fail-closed base integrity

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Two phases, split on the trust boundary and the file surface (design §5):

- **PHASE-01 — code belt** (`src/worktree.rs` + `src/main.rs` CLI): the
  authoritative, orchestrator-side `verify-worker` belts (`not-isolated`,
  `branch-mismatch` via `--branch`). This is where correctness lives.
- **PHASE-02 — skill belt** (`dispatch-agent/SKILL.md` + the budget test): the
  fail-fast in-worker base-guard and the pre-funnel footer gate that *invokes* the
  PHASE-01 interface.

## Sequencing & Rationale

The cut is **dependency-first**, not file-partition-neat (codex plan review): the
ordering is forced by an interface dependency, and the "two clean file sets" framing
oversimplifies (PHASE-01 owns code + its own integration test; PHASE-02 owns the
skill + the e2e shrinkage test).

PHASE-01 precedes PHASE-02 because the skill documents and calls a real CLI
interface (`verify-worker --branch`, the five refusal tokens). Writing the skill
first would cite a verb that does not yet exist — and the budget test's presence
asserts (`not-isolated`, `branch-mismatch`) would have nothing to bind to. Code
first, prose second. Serial: PHASE-02 has a hard dependency on PHASE-01's interface
(EN-1), so even though the edited files don't overlap, they cannot run concurrently.

**PHASE-01 execution shape (realistic, not idealised TDD).** `classify_worker_verify`
and its goldens live in the SAME file (`src/worktree.rs`), and the signature change
ripples to every existing call site + test on the first edit — a naive "write the
new goldens red first" produces a *compile break*, not a meaningful red. Land it as
compile-preserving microsteps instead:
1. Extend the signature + enum and mechanically thread the two new args (placeholder
   `true`/`is_linked_worktree`) through all existing call sites and goldens so the
   crate compiles and the existing suite stays green (behaviour-preservation, VT-3).
2. THEN add the new goldens (VT-1/VT-2) red against the not-yet-wired branches, and
   implement the `NotIsolated` / `BranchMismatch` arms green.
3. Wire the shell `head_is_branch_tip` gather + the `--branch` CLI/executor sites,
   covered red→green by the integration test (VT-4).
Red/green/refactor holds per-microstep; the phase is not one big red.

PHASE-02 is mostly prose, so its load-bearing verification is VA/VH, not VT — a
test can assert the safety strings are *present* (VT-1) but not that they are
*correct and followable*. The VH explicitly guards against the codex-flagged
hazard (R3): a line-budget cap must not pressure the safety prose into ambiguity.
The budget bump is deliberate (design D3); the presence asserts, not the line
count, are the real gate.

## Notes

- `verify-worker` stays diagnostic-only (never removes a fork). `--branch` is
  optional (`head_is_branch_tip` defaults true) so the verb's existing contract is
  preserved for any non-`--branch` caller; the documented contract change is the
  primary-tree `Ok → not-isolated` flip (design R5), which no live caller hits.
- The mid-run-clobber / misplacement residuals are NOT this slice's code — they are
  contained by the existing harness-identical funnel import belt (`classify_import`
  `S^==B`); see design §5.1. Do not re-implement that here.
- Out of scope (do not drift): `dispatch/SKILL.md` router edits, `WorktreeCreate`
  pre-worker hook (IMP-072), `baseRef`→SHA pinning, Defect B (ISS-011).
