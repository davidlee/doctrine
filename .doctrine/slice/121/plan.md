# Implementation Plan SL-121: dispatch sync --integrate: clean exit state and legible outcome

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Three phases, strictly serial — each builds on the prior, and they overlap on
`src/dispatch.rs` + `src/git.rs`, so they cannot be dispatched in parallel.
PHASE-01 lands the shared probe; PHASE-02 is the engine change that consumes it
and fixes ISS-022/ISS-030/IMP-078; PHASE-03 makes the close verify honest and
implementable.

## Sequencing & Rationale

**PHASE-01 first — DRY foundation, lowest risk.** The worktree-aware advance
needs a "which worktree holds ref R" probe. Two duplicate porcelain parsers
already exist (`find_coordination_worktree`, `gather_fork_worktree`), the latter's
own doc admitting no shared helper. Extracting `worktree_for_ref` first means
PHASE-02 builds on a tested seam rather than a third copy. It is behaviour-
preserving (the ADR-006 gate: existing suites green unchanged), so it lands clean
and de-risks the engine phase. Doing it first, not folded into PHASE-02, keeps the
refactor's "no behaviour change" provable in isolation.

**PHASE-02 — the engine, where the bugs die.** All three backlog items share the
integrate exit path, so they fix together: the per-row classify-then-mechanism
structure (design §2.2) makes ISS-022 (index) and ISS-030 (worktree) impossible
under the supported placements, and the per-row disposition report (IMP-078) falls
out of the same loop. This phase carries the design's hard-won corrections from two
codex passes: exact-CAS classification on both legs (not ff-derived — B1), edge/
creation refusal preserved (B2), captured outcomes not bare errors (B3), the dirty
gate before the first `commit_journal` (M4), the None-leg post-CAS re-probe (R2),
and the §2.5 race guard. The VA-1 check exists because the §7 concurrency boundary
is a judgement a unit test cannot fully make — an agent confirms the residual races
are content-safe and reported.

**PHASE-03 last — depends on PHASE-02's journal output.** The close verify needs
the trunk row's `planned_new_oid`, which only exists after PHASE-02 journals it;
and the §3(b) read-surface gap (OQ-5: the admitted `close_target` OID has no stable
close-3a command) is resolved here, against the committed journal (tree-read, the
`sync-tree-reads-ledger-not-worktree` invariant). The SKILL doc rewrite then has a
real OID to diff. VH-1 is human because the close skill is a human/agent-run doc
step, not an automated path.

## Notes

- Execute in an **isolated worktree** (`/worktree` or `/dispatch`): a concurrent
  agent collided with the shared `main` working tree during this slice's design
  (branch switched out from under the session). Phases touching `src/` make that
  collision costlier — isolate.
- Phases are serial; do **not** parallel-dispatch (shared `dispatch.rs`/`git.rs`).
- Behaviour-preservation gate rides every phase: `e2e_dispatch_sync` and the two
  refactored callers' suites must stay green unchanged.
