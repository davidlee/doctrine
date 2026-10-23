# CHR-182: fresh_inputs test helper leaves a registered git worktree

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`tests/common/mod.rs` `fresh_inputs` runs `git worktree add --detach` into a
`TempDir` it returns. Dropping the `TempDir` deletes the directory but not the
worktree registration in `src`'s `.git/worktrees/`, so `src` keeps a stale
entry until `git worktree prune`. Harmless while `src` is itself a temp repo;
a hazard if a caller ever passes a persistent repo. Fix: a guard that runs
`git worktree remove --force` on drop. Surfaced in SL-285 (notes § Harvest).
