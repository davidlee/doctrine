# Dispatch funnel import: 3-way net-diff onto moved shared main, stage-only-delta, commit without -a

On a shared `main` worktree with live concurrent agents, HEAD moves between when
you spawn a dispatch worker and when you import its result, and foreign untracked/
dirty files appear (`mem.system.coordination.concurrent-design-shared-main-worktree`).
The import pattern that holds:

1. Re-capture **`B = git rev-parse HEAD`** IMMEDIATELY pre-spawn. Fork the worker
   from explicit `B`, never session HEAD (`mem.pattern.dispatch.fork-rung3-base-not-session-head`)
   — so the fork is pristine at `B` regardless of where main races to.
2. Worker returns `S` (its fork HEAD, `S^ == B`, single non-merge).
3. Validate the delta: `S^ == B`; one non-merge commit; R-5 belt
   (`git diff --name-only B..S | grep '^\.doctrine/'` ⇒ HALT — workers emit source
   only); check foreign commits `B..HEAD` didn't touch your delta files (conflict
   risk).
4. Import = **`git diff B..S | git apply --3way --index`** — the worker's NET diff,
   3-way-merged onto the moved HEAD. Clean apply + green combined-tree verify +
   precise staging is sound on a moved base. A real `--3way` conflict ⇒ report+halt
   (your changed-path analysis was wrong).
5. `--index` stages ONLY your delta. Commit WITHOUT `-a` / `git add -A` so foreign
   untracked/dirty files (other agents' WIP) are never swept into your commit.

Distinct from `mem.pattern.dispatch.reanchor-base-on-disjoint-head-move` (re-anchor
B on a disjointness proof) — this is the import mechanism, not the re-anchor
trigger. A foreign untracked file trips the worker's `/worktree` commit-before-spawn
guard; either wait for a settled tree or override when forking from an explicit
committed `B` makes the guard's "absent from fork" concern moot.
