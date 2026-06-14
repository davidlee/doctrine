# Dispatch orchestrator on shared main pays the concurrency cost; workers stay isolated. Review whether the coordination branch should run on its own worktree

Retrospective from SL-060: a 4-phase serial dispatch driven to completion while
1-2 OTHER agents (SL-061/62/63, IMP-023) committed to the SAME `main` worktree the
whole time. The funnel's correctness held — but the cost landed almost entirely on
the **orchestrator**, not the workers.

## What worked
- **Worker isolation is sound.** Each worker forked rung-3 from an explicit `B`
  into its own `.worktrees/sl060-pNN` and returned a clean disjoint delta. No
  worker ever saw the contention. All three came back green, `S^==B`, single
  non-merge, R-5 clear.
- **Re-anchoring works.** HEAD moved between every spawn and import (foreign
  commits). Re-capturing HEAD and re-checking disjointness (`B..S` vs `B..H`) before
  each import was enough — every batch imported onto the moved HEAD cleanly via
  `git checkout S -- <paths>` and committed without `-a`.

## Where the contention actually bit (all orchestrator-side, on shared main)
1. **Dirty foreign INDEX once blocked the funnel.** A concurrent agent had staged
   (uncommitted) changes in the shared index; committing would have swept them in.
   Commit-without-`-a` does NOT cover a pre-staged index → had to PAUSE the drive and
   wait for the index to clear, then re-anchor. (Polled ~30s; cleared on the first
   check.)
2. **Inline `.doctrine/` writes (PHASE-05 backfill) collided with foreign WIP.** The
   backfill is non-delegable (R-5 forbids a worker writing authored trees), so the
   orchestrator wrote 60+ slice TOMLs ON the shared main — colliding with a foreign
   agent's uncommitted edit to slice-061 and sweeping a foreign UNTRACKED slice-063
   into the commit (caught + amended). See
   [[mem.pattern.dispatch.glob-add-sweeps-foreign-untracked-on-shared-main]].
3. **rtk masks git exit codes / stat-proxies diff** on shared main, making the
   funnel guards fiddly — output-content checks only, `git checkout S` not
   `diff|apply`. [[mem.pattern.tooling.git-cat-file-e-exit-masked-use-ls-tree]],
   [[mem.pattern.dispatch.rtk-git-diff-stat-use-checkout-import]].

## The open question for the orchestration model
Workers get isolation; the **coordination branch does not** — it rides the shared
`main` working tree where humans + other agents are live. Most of the incident-time
this run went to defending the orchestrator's own commits/index/untracked surface
against that. Worth reviewing: should `/dispatch` run the COORDINATION branch on its
OWN dedicated worktree (a clean checkout of `main`, no foreign WIP), funnel worker
deltas there, and fast-forward/push `main` only at well-defined sync points? That
would move the whole "re-anchor + don't-sweep-foreign" burden off the per-batch hot
path. Trade-off: an extra worktree + a main-sync step, and inline non-delegable
`.doctrine/` writes (backfill, status, notes, memory) would happen in the clean
coordination tree instead of on contended main.

Net: serial dispatch is ROBUST under concurrency (it completed unattended, correct),
but the shared-main coordination posture makes it more expensive and error-prone than
it needs to be. Related: [[mem.system.coordination.concurrent-design-shared-main-worktree]],
[[mem.pattern.dispatch.three-way-import-onto-moved-shared-main]],
[[mem.pattern.dispatch.reanchor-base-on-disjoint-head-move]].
