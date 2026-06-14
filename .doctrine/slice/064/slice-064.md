# Coordination-branch isolation: dedicated worktree + integration-sync seam for dispatch

## Context

Successor to SL-056. SL-056 made the **worker** side of `/dispatch` structurally
sound — orchestrator-owned fork, disk-marker identity, withheld coordination tier,
bwrap confinement. Each worker runs isolated and returns a clean disjoint delta.

The **coordination branch itself** got no such isolation. ADR-006 D8 pins the
coordination *branch* ("usually trunk in solo mode, a delta branch in team mode")
but is **silent on its working tree** — in practice the orchestrator runs on the
shared `main` working tree where humans and other agents commit live.

Two forces make that posture wrong:

1. **Contention cost (mem.system.dispatch.orchestrator-on-shared-main-contention-cost,
   SL-060 retro).** Serial dispatch completed correctly under concurrency, but the
   whole incident cost landed orchestrator-side, on shared main: a dirty foreign
   INDEX once blocked the funnel; inline non-delegable `.doctrine/` writes
   (R-5-forbidden to workers) collided with foreign WIP and swept an untracked
   foreign file; rtk masking git exit codes made every funnel guard fiddly. The
   re-anchor + don't-sweep-foreign burden rode the per-batch hot path.

2. **Team / PR workflow (primary product driver).** Most teams will not let an
   orchestrator commit dispatch output straight onto `main` — they want
   **feature-branch → PR review** before integration. A dedicated coordination
   worktree on its own branch is precisely the seam that makes dispatch output land
   on a **reviewable branch**, integrated at a controlled point, not YOLO'd onto
   trunk. The contention fix and the team-workflow fix are the **same mechanism**.

ADR-006 D8 already says globals "land on main at merge (convention, not
mechanism)". This slice turns that convention into an **actual integration-sync
mechanism** and pins the coordination branch's working-tree placement.

## Scope & Objectives

**O1 — ADR-006 D8 amendment (governs the code).** Pin coordination-branch
working-tree placement: the orchestrator runs the coordination branch in its **own
dedicated worktree** (a clean checkout of the integration base, free of foreign
WIP), not on the shared trunk working tree. Promote "globals land on main at merge
(convention)" to a defined **integration-sync** step. Preserve D2 (worker-sole-writer),
D7 (funnel discipline, coordination branch is the durable store), D9 (withheld-tier
provisioning) unchanged — this is a placement + sync refinement, not a rewrite.

**O2 — coordination worktree provisioning.** Reuse the SL-056 `worktree fork`
machinery to stand up the coordination tree (or define why it differs — it carries
the coordination/runtime tier that worker forks withhold). Decide lifecycle:
create at dispatch start, remove at integration/close (overlaps IMP-041).

**O3 — integration-sync seam.** The mechanism + cadence by which coordination-branch
commits reach the integration target. Spans the solo case (fast-forward/push trunk)
and the team case (leave a feature branch for PR; do **not** auto-merge). Default
posture: produce a reviewable branch, never push to a protected trunk by construction.

**O4 — `/dispatch` + `/worktree` skill alignment.** Rewire the funnel cadence to run
on the coordination worktree; fold IMP-043's moving-HEAD re-anchor down to the
integration-sync point only (it stops being a per-batch hot-path concern once the
coordination tree has no foreign writers).

## Non-Goals

- Not re-opening worker isolation, the disk marker, fork/import/gc verbs, or bwrap
  confinement (all SL-056, shipped).
- Not building a hosted-forge PR integration (GitHub/GitLab API). The seam emits a
  reviewable branch; *who opens the PR* is out of scope — leave the integration
  target policy-agnostic (ADR-006 D1).
- Not changing the worker-sole-writer invariant (D2) or the withheld-tier model (D9).
- Not a parallel-dispatch scheduler change — orthogonal.

## Affected surface

- `.doctrine/adr/006/` — D8 amendment (append; D-ids immutable).
- `/dispatch`, `/dispatch-subprocess`, `/dispatch-agent`, `/worktree` skills
  (source under `plugins/`, not the installed `.doctrine/skills` copy).
- Possibly `src/` worktree verb surface if the coordination tree needs a distinct
  provision path from the worker fork (TBD at design).

## Risks, assumptions, open questions

- **OQ-1 (carry to /design): integration-sync mechanism + cadence.** ff/push trunk
  vs leave-branch-for-PR; sync per-batch vs once at end. The team driver biases
  toward leave-a-branch as the default.
- **OQ-2: solo-vs-team applicability.** The SL-060 trigger was *serial* dispatch
  contended by *unrelated* agents — i.e. the trigger is "shared trunk has live
  foreign writers", not "parallel workers". Does the dedicated coordination worktree
  apply always, or only when foreign-writer contention / a PR posture is requested?
- **OQ-3: progress visibility.** Coordination on its own tree means `main` lags
  until sync — humans watching trunk see nothing mid-run. Acceptable cadence?
- **A-1:** the SL-056 fork/provision machinery generalises to a tier-carrying
  coordination tree without a new primitive (to be confirmed at design).
- **Cost:** an extra worktree + an integration-sync step; non-delegable `.doctrine/`
  writes move to the clean coordination tree (strictly better than contended main).

## Verification / closure intent

- ADR-006 D8 amendment accepted (adversarial review).
- A dispatch run drives the funnel on a dedicated coordination worktree, leaves a
  reviewable integration branch, and `main`'s working tree is never written by the
  orchestrator mid-run (the contention surfaces from SL-060 #1/#2 are unreachable
  by construction).
- IMP-043 re-anchor demoted to the sync point; IMP-041 cleanup ownership resolved.

## Follow-Ups

- IMP-043 (import `--allow-reanchor`) — subsumed/demoted by this slice.
- IMP-041 (worktree cleanup-after-merge ownership) — resolved here.
