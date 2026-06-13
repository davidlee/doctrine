# Dispatch import under rtk: git diff is stat-proxied, use git checkout to import

The rtk hook rewrites `git diff` to a compact **stat summary** (`path | N +++---`),
NOT a raw unified patch. So the canonical funnel import idiom
`git diff B..S -- path | git apply --3way` **fails** with `error: No valid patches
in input` — git apply receives the stat text, not a patch.

## How to apply (dispatch orchestrator, import step)

For a **file-disjoint** worker delta (the disjointness already proven against the
coordination HEAD), the net diff `B..S` for each changed path equals S's blob when
HEAD's version of that path == B's version. So import directly:

```
git checkout <S> -- $(git diff --name-only B..S)
```

This stages S's exact content for every changed path — identical to applying the net
diff, and it round-trips the rtk proxy cleanly. Verify fidelity with
`git diff <S> -- <paths>` (empty == working tree matches S).

When you genuinely need the **raw patch** (e.g. a non-disjoint 3-way apply, or to
read what changed), bypass the proxy: `rtk proxy git diff B..S -- path` returns the
unfiltered unified diff. `git diff --name-only` / `--name-status` are unaffected (no
patch body), so the R-5 belt and disjointness checks work as written.

Hit repeatedly across SL-048 dispatch (every PHASE-02..04 import). Companion to
[[mem.pattern.dispatch.three-way-import-onto-moved-shared-main]] and
[[mem.pattern.dispatch.reanchor-base-on-disjoint-head-move]] — both assume a working
`git apply`/diff; under rtk the checkout-import is the reliable substitute for
disjoint batches.
