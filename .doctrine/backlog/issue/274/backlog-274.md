# ISS-274: record-boundary leaves index and worktree stale

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`doctrine dispatch record-boundary` lands its boundary row as a working-tree-free
commit on the coordination branch, but does not update the coordination
worktree's index or working tree to match. The file exists in `HEAD` and not on
disk, so the tree is left reporting a **staged deletion** of the row that was
just written.

Observed in the SL-233 coordination worktree, recording PHASE-01's boundary:

```
$ doctrine dispatch record-boundary --slice 233 --phase PHASE-01 \
    --code-start 2f70392e5 --code-end 9bc1e729a
$ git status --porcelain
D  .doctrine/dispatch/233/boundaries.toml
$ doctrine dispatch tree-state --slice 233
{ "slice": 233, "clean": false,
  "staged": [".doctrine/dispatch/233/boundaries.toml"] }
```

The row itself is correct and committed (`8fc865297`). Only the tree state is
wrong.

## Why it matters

It is a staged deletion of committed coordination state sitting in a tree where
multiple agents share one index. `dispatch commit` refuses an unnamed deletion
before touching the index (ISS-234), which contains the blast radius — but a
raw path-limited `git commit` by anyone else in that tree carries the deletion
into their commit, silently reverting the funnel row. That is exactly the
funnel-reversion signature `dispatch hook-check` exists to refuse, produced by
the funnel's own verb.

It also makes the tree read dirty to every subsequent `tree-state` call, which
is the funnel's clean-check — so the noise lands on the next reader as an
anomaly to diagnose, not an artefact to ignore.

Recovery is a path-limited restore:

```
git restore --source=HEAD --staged --worktree -- .doctrine/dispatch/<N>/boundaries.toml
```

## Shape of a fix

Sync the index and worktree to the commit the verb just made, the way any
working-tree-free write must if the tree it writes behind is live. Worth
checking the sibling verbs that land working-tree-free commits on the same
branch — `dispatch_conclude_phase` lands a boundary row by the same mechanism
and is the more travelled path, so if it shares the defect it is the higher
priority of the two.

Found while concluding SL-233's coordinator-only PHASE-01 by hand — the interim
posture recorded in [[IMP-346]].
