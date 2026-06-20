# git reset --keep cannot resync a worktree whose branch already advanced under it

## What

`git reset --keep <oid>` and `--merge` only touch files that **differ between
`<oid>` and the current HEAD**. When a branch ref has already been advanced *under*
a live checkout (e.g. by a pure `git update-ref` / `update_ref_cas`), HEAD now
resolves to `<oid>` too — so `reset --keep <oid>` sees **zero diff** and leaves the
index/worktree stale (the ISS-022/030 phantom reverse-diff). Proven empirically:

```
update-ref refs/heads/main <c2>   # ref moved under the live checkout
git status → " D b.txt"            # desynced
git reset --keep   <c2> → still " D b.txt"   # NOT fixed (HEAD already == c2)
git reset --hard   <c2> → clean              # fixed
```

## How to apply

- To resync a worktree onto a ref that **already moved under it**, use
  `git reset --hard <oid>` — it sets index+tree to the commit unconditionally,
  immune to the HEAD-already-moved ordering.
- Gate it on a **clean tracked tree first** (`git status --porcelain
  --untracked-files=no` empty, i.e. `git::tree_clean`): under that precondition
  `reset --hard` discards nothing tracked and untracked files survive — it is
  content-safe. This is `git::resync_worktree_hard` (SL-121 PHASE-02), the None-leg
  post-CAS re-probe resync in `dispatch::advance_pure_ref`.
- The opposite ordering (worktree checks out the ref *then* you fast-forward it) is
  fine for `merge --ff-only`, which moves ref+index+tree together — that is the
  checked-out leg (`git::ff_advance_in_worktree`). `reset --keep` is only wrong
  when the ref moved *first*.

SL-121 design §2.2 originally named `reset --keep` for this resync; corrected to
`reset --hard` under the clean gate. See [[mem.pattern.dispatch.sync-tree-reads-ledger-not-worktree]].
