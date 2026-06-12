# Dispatch funnel re-anchors B to a moved coordination HEAD on a disjointness proof

On a **shared** coordination branch (`main`), the import funnel's precond
(`HEAD == B`) and the `S^ == B` belt routinely trip — not from corruption, but
because HEAD legitimately moved off the captured `B`:

1. **The worker's own `/worktree` setup commits to the coordination branch.**
   First-ever fork adds `.worktrees/` to `.gitignore` and commits it **on
   `main`** (the worktree-skill safety step), moving HEAD to `B+1` before the
   worker delta even returns. (Pre-empt it: gitignore `.worktrees/` once, up
   front — then later workers skip the step.)
2. **Concurrent slices commit to `main` continuously.** Here SL-046 design/ADR
   work landed `.doctrine/` commits between every batch
   (`mem.system.coordination.concurrent-design-shared-main-worktree`).

**Do NOT blindly re-dispatch** (the skill's default for a moved HEAD) when the
move is provably benign — re-dispatch reproduces the *identical* delta and wastes
the worker run. Instead **re-anchor `B → current HEAD`** after proving the move
is disjoint from the import:

- `git diff --stat <old-B>..<new-HEAD> -- <each file in the delta>` is **empty**
  (the delta's target files are byte-identical at both bases), and
- the intervening commits touch only unrelated trees (`.doctrine/`, `.gitignore`).

Then the net diff `<old-B>..S` applies onto `<new-HEAD>` with the same result
re-dispatch would yield. This consciously **substitutes an explicit
disjointness proof for the mechanical `S^==B` belt** — sound because *you* (the
sole writer, worker-mode OFF) run it. Still mandatory: R-5 authored-tree belt,
combined-tree verify, branch-point guard, one commit per batch.

Corollary — the **precond "tree clean" check is per-delta-path, not absolute**
on a shared worktree: a concurrent slice's dirty file (e.g. SL-046's unstaged
`adr-010.md`) is fine to leave; guarantee cleanliness only for the delta's own
paths and stage explicitly (`git add <files>`, **never** `git add -a` /
`commit -a`, which would sweep the sibling's work into your commit).

See [[mem.pattern.dispatch.fork-rung3-base-not-session-head]] (fork from the
explicit base) and [[mem.system.coordination.concurrent-design-shared-main-worktree]].
