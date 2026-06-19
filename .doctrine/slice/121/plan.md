# Implementation Plan SL-121: dispatch sync --integrate: clean exit state and legible outcome

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four phases, strictly serial — each builds on the prior, and they overlap on
`src/dispatch.rs` + `src/git.rs`, so they cannot be dispatched in parallel.
**Execution order: PHASE-01 → PHASE-04 → PHASE-02 → PHASE-03** (PHASE-04 is
appended at the end of `plan.toml` per the never-renumber rule; its place in the
sequence is carried by PHASE-02's EN-2, not its file position). PHASE-01 lands the
shared worktree probe; PHASE-04 lands the IMP-075 journal-cycle extraction
(behaviour-pure); PHASE-02 is the engine change that consumes both and fixes
ISS-022/ISS-030/IMP-078; PHASE-03 makes the close verify honest and implementable.

Two behaviour-pure foundation refactors (PHASE-01 probe, PHASE-04 bracket) precede
the one behaviour-changing engine phase (PHASE-02) — so each refactor's "no
behaviour change" is provable in isolation (suites green unchanged), and PHASE-02's
diff is confined to integrate's injected `apply` closure plus the caller-side gate
and report.

## Sequencing & Rationale

**PHASE-04 — extract the journal bracket (IMP-075), behaviour-pure.** The folded
IMP-075 (design §2.6) collapses the commit-pre / apply-loop / commit-post cycle
duplicated by `prepare_review` and `integrate` into `with_journaled_projection`,
taking the per-row move as an injected closure. Done with both callers' CURRENT
bodies verbatim, it is behaviour-pure: the `e2e_dispatch_sync` prepare AND integrate
paths stay green unchanged (the ADR-006 gate). Landing it BEFORE PHASE-02 — not
folded into it — means PHASE-02's worktree rewrite touches only integrate's closure
body, leaving the delicate journal/CAS cycle proven-unchanged. This is the same
"extract pure first, change behaviour after" discipline as PHASE-01, and it is why
the fold is cheaper than refactoring `integrate` twice (the SL-121 bundling
rationale). The codex §2.6 pass bound the `apply` contract (refusals → `Ok(Some)`,
`Err` only fatal — B3); EX-4 carries it. Order vs PHASE-01 is free (the bracket is
journal-cycle, independent of the probe); both must precede PHASE-02.

**PHASE-01 — DRY foundation, lowest risk.** The worktree-aware advance
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
and the §2.5 race guard. With PHASE-04 already landed, this phase mutates only
integrate's injected `apply` closure (the advance) plus the caller-side dirty
pre-gate (before the bracket, M4) and the §4 report (after it) — the journal cycle
itself is untouched (EX-7). The VA-1 check exists because the §7 concurrency
boundary is a judgement a unit test cannot fully make — an agent confirms the
residual races are content-safe and reported.

**PHASE-03 last — depends on PHASE-02's journal output.** The close verify needs
the trunk row's `planned_new_oid`, which only exists after PHASE-02 journals it;
and the §3(b) read-surface gap (OQ-5: the admitted `close_target` OID has no stable
close-3a command) is resolved here, against the committed journal (tree-read, the
`sync-tree-reads-ledger-not-worktree` invariant). The SKILL doc rewrite then has a
real OID to diff. VH-1 is human because the close skill is a human/agent-run doc
step, not an automated path. VA-1 holds the IMP-102/103 seam-awareness: both stay
out of scope, but the tree-true step-3a must be cut so IMP-102's later `done`-gate
(refuse close when un-integrated) bolts onto the same check without re-cutting it —
the design-bound coordination from the SL-121↔IMP-075 fold.

## Notes

- Execute in an **isolated worktree** (`/worktree` or `/dispatch`): a concurrent
  agent collided with the shared `main` working tree during this slice's design
  (branch switched out from under the session). Phases touching `src/` make that
  collision costlier — isolate.
- Phases are serial; do **not** parallel-dispatch (shared `dispatch.rs`/`git.rs`).
  Execution order is **01 → 04 → 02 → 03** (PHASE-04 appended last in `plan.toml`
  but gated second via PHASE-02 EN-2).
- Behaviour-preservation gate rides the two refactor phases: PHASE-01 (both probe
  callers) and PHASE-04 (`e2e_dispatch_sync` prepare AND integrate paths) must stay
  green unchanged. PHASE-02 intentionally changes integrate behaviour — its proof is
  the new VTs, not the green-unchanged gate.
