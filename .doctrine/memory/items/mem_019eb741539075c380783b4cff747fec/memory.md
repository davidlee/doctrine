# RV review verbs refuse on a worktree fork — drive the audit from the parent tree or merge-first

RV review **turn-verbs** (`raise`/`dispose`/`amend`/`verify`/`contest`/`reopen`/
`withdraw`/`conclude`), plus `status`, `prime` and `unlock`, refuse on a git worktree fork — the turn baton lives in the **parent tree's
gitignored runtime state**, which a fork cannot co-write (IMP-024). `review new`
*succeeds* on a fork (it only writes the authored ledger files; `show` and `list`
are unguarded too), which is the trap: a successful `new` is no proof you are
outside a fork; the next turn verb fails with:

> review verbs are not supported on a worktree fork (IMP-024): the turn baton
> lives in the parent tree's gitignored state, which a fork cannot co-write. Run
> `review` from the parent or coordination tree.

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

## It is not only *forks* — it is every ordinary worktree (confirmed SL-251, 2026-08-16)

The refusal message says "worktree fork", and the guard's own doc comment says
"the test is the ROLE, not mere linkage". Both undersell the reach. Read the
code (`src/review/turn.rs` `resolve_review_root` → `worktree` `classify_worktree_role`;
re-verified 2026-09-26, SL-268):

```rust
if classify_worktree_role(branch, linked) == "fork" { bail!(…) }
// classify_worktree_role: !linked → "primary"
//                         branch is `dispatch/<all-digits>` → "coord"
//                         everything else → "fork"
```

So the **only** linked worktree that may drive an RV is one whose branch is
`dispatch/<NNN>` with a numeric suffix. `ISS-275` widened the guard exactly that
far and no further. A plain `git worktree add -b close/SL-251 …` is classified
`fork` and refused, and there is no env escape hatch.

**This contradicts `AGENTS.md` in this repo**, which says: *"If auditing /
closing a feature, land it on a worktree and push to main."* You cannot. Take
the merge-first branch of the resolution above: merge into the primary tree's
own branch (which is not a checkout, so the "primary stays on edge" rule holds)
and drive `/audit` → `/reconcile` → `/close` there. Do not rename a branch to
`dispatch/<NNN>` to unlock the verbs — it would work, and it lies about the
tree's role to every dispatch verb afterwards.

Cost when it bites late: a full merge, build, and conformance run in the
worktree, then relocating the whole audit. Check the branch shape *before*
building the tree you plan to audit in.
