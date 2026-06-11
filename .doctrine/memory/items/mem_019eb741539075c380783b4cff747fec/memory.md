# RV review verbs refuse on a worktree fork — drive the audit from the parent tree or merge-first

RV review **turn-verbs** (`raise`/`dispose`/`verify`/`contest`/`withdraw`/`status`)
refuse on a git worktree fork — the turn baton lives in the **parent tree's
gitignored runtime state**, which a fork cannot co-write (IMP-024). `review new`
*succeeds* on a fork (it only writes the authored ledger files), which is the
trap; the next turn verb fails with:

> review verbs are not supported on a worktree fork (IMP-024): the turn baton
> lives in the parent tree's gitignored state, which a fork cannot co-write. Run
> `review` from the parent tree.

**Why it bites `/audit` + `/close`:** when a slice's implementation lives on a
worktree branch (e.g. dispatch funnel work), you cannot drive the audit RV ledger
from that worktree.

**Resolution — pick one:**
- Drive the RV from the **parent tree** (the primary checkout) — it just needs the
  slice as a `--target`; evidence can be gathered from the worktree separately.
- Or, when the worktree branch has **settled and is disjoint**, `git merge` it into
  the parent's checkout **FIRST**, then run `/audit` + `/close` coherently in one
  tree where baton, code, and `notes.md` sit together (then commit/push per plan).
  Re-run the gate on the merged tree before raising findings.

If you ran `review new` on the fork by mistake, `rm -rf` the stray uncommitted
`.doctrine/review/NNN*` from the worktree before re-creating it in the parent.

Discovered during SL-042 `/audit`: work on the `sl-042-coord` worktree; merged to
`main` (disjoint from the settled SL-043) to drive RV-003. Companion:
[[mem.pattern.dispatch.fork-rung3-base-not-session-head]],
[[mem.pattern.build.clean-head-worktree-binary-vs-stash]].
