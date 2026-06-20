# Implementation Plan SL-125: Stamp provision source from primary worktree

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

One surgical fix in `src/worktree.rs`, so one phase. The change is small enough
that splitting would only add ceremony; the TDD red→green→refactor loop lives
inside PHASE-01.

## Sequencing & Rationale

**Why one phase.** The design (`design.md` §2/§4) isolates the defect to a single
role: `run_stamp_subagent` uses one `repo` value for both repo-binding (R1) and
the provision source (R2); only R2 is wrong. The fix touches one file — add the
`primary_worktree` helper and swap the source feeding `run_provision` — and leaves
R1 (binding anchor, `cwd_valid`, `classify_stamp`, every `StampRefusal`) untouched.
No cross-arm blast radius: the subprocess/coordination provisioning paths resolve
their source from the orchestrator process and never call `run_stamp_subagent`
(single caller `main.rs:4190`).

**TDD order inside PHASE-01.** Start red with VT-1 — the e2e test that spawns the
built binary with `current_dir` = the worker worktree (the exact Defect-C
condition: process == worker == fork). It fails today because `root::find` returns
the worker as the source and `verify_sibling_worktree` bails. Then green: add
`primary_worktree(cwd)` (git's first `worktree list --porcelain` entry) and route
the provision source through it. Add VT-2 (helper unit) and VT-4 (cross-repo
`bad-dir` pin — guards the codex BLOCKER from regressing). VT-3 is the existing
refusal suite, which must stay green untouched. Refactor: keep the helper cohesive
with the other git-worktree probes (`resolve_common_dir`, `is_linked_worktree`),
correct the stale defect-site comment (the false "hook fires inside the
orchestrator tree" claim).

**Verification boundary.** VT-1/2/3/4 are the in-suite proof. VH-1 (the fresh
IMP-046 harness probe) cannot run in-suite — it needs a real Claude session with
`isolation: worktree`; it is the final out-of-suite confirmation that the live
claude dispatch arm stamps without a hand-stamp.

## Notes

- Bare-repo layout is the one residual `source==fork` case; out of scope (dispatch
  never bare) and it fail-closes via the existing M3 `STAMP FAILED` path — no silent
  widening (design §7 A1).
- FU-1 (design §Follow-ups): if `.worktreeinclude` later carries per-worktree
  divergent untracked state, primary-as-source no longer suffices and the hook
  cannot name the orchestrator tree — a separate design, not this slice.
