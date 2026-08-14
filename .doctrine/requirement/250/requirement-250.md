# REQ-250: `worktree land --fork BR` lands a solo isolated-worktree branch onto the coordination branch structurally non-squash (`git merge --no-ff`, never `--squash`); refuse a marker-bearing (`dispatch-fork`) or worktree-gone fork and abort a conflicted merge before refusing. Orchestrator-classed, solo-only.

## Statement

> **AMENDED — FALSIFIED (SL-254, 2026-08-14).** The title line above retains the
> SL-056 statement. Its "refuse a marker-bearing (`dispatch-fork`)" clause is
> falsified: the disk marker is gone (`DEC-207`), so `land` can no longer read
> one. The refusal is now decided by BRANCH SHAPE via
> `src/worktree/shared.rs::is_dispatch_fork_branch`. (Design §5.2.3 prescribed
> `classify_worktree_role` returning `"fork"` as the substitute; that was wrong
> as written — a solo isolation branch also classifies as `fork`, which would
> have made `land` refuse its own use case. `is_dispatch_fork_branch` is what
> shipped and what this requirement now states.) The title is retained
> unchanged; the statement below is what must hold.

`doctrine worktree land --fork BR` lands a SOLO isolated-worktree branch onto the
coordination branch:

1. **Structurally non-squash** — `git merge --no-ff`, never `--squash`, so the
   fork's commits survive as commits.
2. **Refuse a dispatch worker fork.** `BR` is a dispatch worker fork iff
   `is_dispatch_fork_branch(BR)` — the branch name carries the `dispatch/` prefix
   AND its suffix is not a slice number. A numeric suffix (`dispatch/254`) is the
   COORDINATION branch, not a worker fork; a solo isolation branch (`feat/x`)
   carries no `dispatch/` prefix and is landable. No cross-tree read of any kind
   participates: the marker was the only signal that asked about ANOTHER tree,
   and the environment leg that replaced it answers only for THIS process, so it
   cannot answer for the fork being landed.
3. **Refuse a worktree-gone fork**, with `worktree-gone` gating the
   `dispatch-fork` leg (the worktree-less case is named first).
4. **Abort before refusing.** A conflicted merge is `git merge --abort`-ed FIRST
   — restoring a clean tree — and only THEN refused (`merge-conflict`). If the
   abort itself fails, the refusal names `MERGE_HEAD`, the unmerged paths, and
   the manual remedy.
5. **Orchestrator-classed, solo-only** — refused under worker mode by
   `worker_guard`.

## Rationale

Landing is the solo path's counterpart to the dispatch funnel's import: a solo
fork's history is the deliverable, so squashing it would destroy the record the
non-`--ff` merge exists to preserve. The dispatch-fork refusal keeps the two
paths from crossing — a worker fork must funnel through `import`, where the
`.doctrine/` / `.claude/` belt applies. Deciding that by branch shape rather than
by a stamped artefact means the verb needs no privileged read of the fork's tree
and cannot be fooled by a tree that was never stamped.
