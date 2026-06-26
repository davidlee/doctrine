# git worktree porcelain: prunable trails branch — block-accumulate

`git worktree list --porcelain` lays each worktree out as a blank-line-delimited
block, and the optional annotations (`prunable`, `locked`, `detached`) come **after**
the `branch` line:

```
worktree /tmp/dispatch-worker-153-PHASE-01
HEAD d94a36346498fc5587bdc7e151474b664565a6ce
branch refs/heads/worker/153/PHASE-01
prunable gitdir file points to non-existent location
```

A parser that returns the moment it matches the `branch` line therefore **never sees
`prunable`** — it reports a stale/pruned worktree as if it were live. This bit
`git::parse_worktree_for_ref` (SL-154 PHASE-02, design D9): the original returned
`Option<PathBuf>` on the branch match, dropping liveness.

## How to apply

- **Accumulate the whole block, then decide.** Settle a block (extract its fields,
  test the match) only at a block boundary — the next `worktree` line, a blank line,
  or EOF — not on the `branch` line. `parse_worktree_for_ref` uses a `WorktreeBlock`
  accumulator (`{ path, branch, prunable }`) and a `settle()` that yields the matched
  entry at the boundary.
- **Liveness = `!prunable && path.exists()`.** git keeps listing a worktree whose
  checkout was deleted (marking it `prunable`) until `git worktree prune` runs, so a
  name/ref match alone does not prove liveness. `git::live_worktree_for_ref` filters
  on both. Confirmed empirically: `git worktree add` then `rm -rf` the dir → still
  listed, now `prunable`, `path.exists()` false.
- The blank-line block-reset rule (M9) still holds: a `branch` line binds only to a
  `worktree` line in its own block.

Cousin of [[mem.pattern.dispatch.reset-keep-cant-resync-already-advanced-ref]] — both
are git plumbing whose output ordering / annotation semantics defeat a naive read.
