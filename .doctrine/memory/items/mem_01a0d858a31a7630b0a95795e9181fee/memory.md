# The prepare-review plumbing advance leaves the coord worktree dirty, so its teardown refuses

`dispatch sync --prepare-review` commits the CAS journal onto `dispatch/<slice>`
with **plumbing** (`tree_with_file` + `commit_tree` + `update_ref_cas`): the
branch ref advances without touching the index or the working tree.

The documented conclude cadence says "remove coordination worktree directory
(KEEP the refs)". That step **fails** in the worktree it just prepared:

    $ git -C .dispatch/SL-265 status --porcelain
    D  .doctrine/dispatch/265/journal.toml      # STAGED deletion (index vs HEAD)
    $ git worktree remove .dispatch/SL-265
    fatal: ... contains modified or untracked files, use --force to delete it

Freshly-advanced HEAD has `journal.toml`; the stale index does not, so git reads
a staged deletion and gates the removal.

**Recover** (non-destructive — the file is HEAD's, not touched work): resync the
index/worktree to the advanced HEAD, then remove.

    git -C <coord> restore --source=HEAD --staged --worktree -- .doctrine/dispatch/<slice>/journal.toml
    git worktree remove <coord>

**Do not** reach for `git worktree remove --force`; the desync is the only
objection, and `--force` is the "drunk with a chainsaw" the dispatch skill
forbids.

The journal is safe: it lives on the `dispatch/<slice>` ref, and stage-2
`--integrate` tree-reads it from the branch tip (`read_path_at`), so no
worktree-local copy is load-bearing. This is the teardown-side twin of
[[mem.pattern.dispatch.prepare-review-plumbing-desync-reverts-journal]] (the
working-tree-commit variant). Observation
`01a0d851-dec4-7a91-91f2-61a1cb3437f1`.
