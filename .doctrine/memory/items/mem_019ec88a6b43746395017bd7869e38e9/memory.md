# prepare-review advances dispatch/<slice> via plumbing (no checkout) — a later working-tree commit on the coord branch reverts journal.toml

`dispatch sync --prepare-review` commits the CAS journal onto `dispatch/<slice>`
with **plumbing** (`commit_journal`: `tree_with_file` + `commit_tree` +
`update_ref_cas`) — it advances the branch ref WITHOUT touching the working tree
or index. So in the coordination worktree, after prepare-review, **HEAD is ahead
of the index/working tree** by the two `journal: prepare-review` commits.

**Trap:** authoring an ordinary working-tree commit (e.g. `notes.md`) on the coord
branch *after* prepare-review builds the commit tree from the stale index — which
lacks `.doctrine/dispatch/<slice>/journal.toml` — so the new commit **reverts/deletes
journal.toml**, breaking stage-2 integrate (`/close` then reports "no prepared
journal — run prepare-review first").

**Avoid:** do conclude-time authored commits (notes/status) BEFORE prepare-review,
or after prepare-review run `git checkout HEAD -- .` (resync the working tree to the
advanced HEAD) before committing. **Recover:** restore from the applied-status
journal commit — `git checkout <journal-commit> -- .doctrine/dispatch/<slice>/journal.toml`
then commit (it is gitignored coordination tier; force-add). The deliverable
`review/*`/`phase/*` refs are cut from the pre-journal tip and are unaffected.

See [[mem.pattern.dispatch.candidate-build-seam]] and
[[mem.pattern.dispatch.sync-tree-reads-ledger-not-worktree]].
