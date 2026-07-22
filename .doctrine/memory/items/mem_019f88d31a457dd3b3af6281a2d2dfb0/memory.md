# Materialise a merge-tree conflict into a linked worktree: cwd=worktree, LF index-info, on-branch checkout

Problem: you have a `git merge-tree --write-tree` **Conflict** (`T_c` + an unmerged
stage table) and want to hand it to an operator to resolve+commit as a genuine
2-parent merge — WITHOUT running `git merge` (which is config-perturbable via
`branch.<n>.mergeOptions`, so it is a *different* engine; SL-212 D2). Probe-verified
on git 2.54 (`git::materialise_conflict_worktree`, `src/git.rs`).

**1. Run every plumbing command with `cwd = the linked worktree`.** From inside a
linked worktree, `git rev-parse --git-path {MERGE_HEAD,MERGE_MODE,MERGE_MSG,index}`
resolve to the **per-worktree** git dir (`…/.git/worktrees/<n>/<name>`) automatically —
NOT the common dir. So git uses the worktree-private index and merge metadata with
**no explicit `GIT_INDEX_FILE`** and no ScratchIndex (contrast
[[mem.pattern.tooling.tempfile-dev-only-use-git-dir-scratch-index]], which wants a
*throwaway* index; here you want the worktree's OWN index so the operator's
`git commit` sees it). `--git-dir` == `--absolute-git-dir` == the per-worktree dir.

**2. Sequence:**
- `git read-tree --reset -u <T_c>` — index+worktree become `T_c` EXACTLY, including
  **removal of paths `T_c` deletes** (a plain `read-tree` + `checkout-index -af`
  leaves a mechanically-deleted path behind → the operator's `git add` re-adds it →
  spurious downstream refusal; `--reset -u` gives `write-tree == T_c`. RV-289 F-2).
- Rewrite each conflicted path to unmerged stages 1/2/3 via ONE
  `git update-index --index-info` fed **LF records** `mode SP oid SP stage TAB path`
  (a `0 <zero-oid> TAB path` line first to drop stage 0). **`-z --index-info` does
  NOT work for this** — it accepts only the stage-less ls-tree form, so NUL records
  with a stage field are dropped (0–1 rows land instead of 3). Because the record is
  LF/TAB-delimited, a conflict path containing a raw LF/TAB byte can't be
  represented → defensive refuse (byte-safety for such paths is a downstream
  `-z`-diff concern, not this rewrite).
- Write `MERGE_HEAD`=source_oid, an empty `MERGE_MODE`, and `MERGE_MSG` via
  `rev-parse --git-path`. HEAD stays at base (read-tree does not move it), so the
  operator's `git commit` yields ordered parents `[HEAD(base), MERGE_HEAD(source)]`.

**3. Check the worktree out ON THE BRANCH, not detached.** `git worktree add <path>
<FULL-refname>` (e.g. `refs/heads/candidate/…`) **detaches HEAD** → the operator's
commit does NOT advance the branch ref (see
[[mem.pattern.dispatch.candidate-worktree-detached-head]]). Pass the branch
**shortname** (`refs/heads/` stripped) instead → on-branch → the commit **advances
the ref**, which any downstream verb reading `resolve_commit(<ref>)` (e.g. the SL-212
ingest verb) depends on. This is the difference between ingest reading the resolved
merge `R` and reading `base` and refusing "not committed".
