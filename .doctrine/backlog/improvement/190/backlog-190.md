# IMP-190: /audit skill — signpost worktree-fork review refusal + parent-tree promotion

The `/audit` skill body never warns that `doctrine review` verbs refuse a
worktree fork (IMP-024: the turn baton lives in the parent tree's gitignored
state). That constraint lives only in `review-ledger.md §6` (parent-tree
caveat), read late. When a slice was built in an isolated worktree (the normal
case for code slices), the audit cannot open its RV there — the slice code +
lifecycle status live on the branch while review state must live on the parent
tree (edge). Resolving the split (promote branch → parent tree so it carries
code + `audit` status before the RV opens) required a user consult during
/audit SL-163; the skill offers no recipe.

**Fix:** add a one-line pointer near the top of `/audit` (and/or the
review-ledger trigger): "If the slice was built on a branch/worktree, land it on
the parent tree before opening the RV — review verbs refuse forks (IMP-024)."
Optionally a short promotion recipe (`git merge --no-ff <branch>` into the parent
tree, or the dispatch candidate path for dispatched slices).

Cross-ref: RFC-011 case-notes; review-ledger.md §6; IMP-024.
