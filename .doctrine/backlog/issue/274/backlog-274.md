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

## 2026-07-29 — the sibling check is answered: it is FOUR verbs, not one

The "worth checking the sibling verbs" note above is now measured. Driving
SL-233 PHASE-14 through the full claude-arm funnel, the stale index appeared
after **every** funnel-writing verb:

| verb | staged artefact |
|---|---|
| `dispatch_import` | the imported source files + `funnel.toml` |
| `dispatch verify` | `funnel.toml` |
| `dispatch_conclude_phase` | `boundaries.toml` + `funnel.toml` |
| `dispatch_reap` | `funnel.toml` |

`dispatch_conclude_phase` **does** share the defect, so by this item's own
prioritisation it is the higher-priority leg.

**It is not always a pure deletion.** After `conclude` the staged diff was
`2 insertions(+), 11 deletions(-)` — a *modification reversion* of the funnel
row plus deletion of the boundary row. That matters because a reader checking
only for `D ` in `git status --porcelain` will not recognise it.

### It caused a real bad commit, and the trap has a specific shape

The predicted blast radius landed. `slice record-delta` had just been run (it is
required on the claude arm, ISS-241) and legitimately writes *somewhere*, so a
single staged `funnel.toml` read as "that must be record-delta's write". It was
`dispatch verify`'s stale index. The path-limited `git commit` reverted PHASE-14
from `verified` to `imported` and deleted the whole `[phase.verify]` row.

Caught only because the diffstat shape was wrong for an additive record;
restored from the pre-commit tip.

**The stale index is most dangerous when a legitimate write is expected in the
same window** — that is what defeats "is this mine?" reasoning. Two mitigations
short of the real fix, both cheap: have the funnel verbs clear their own index
entry, or have `record-delta` report the path it wrote.

Standing rule until then: read every staged diff before committing in a
coordination tree, and never infer authorship from the path alone.
`doctrine dispatch commit` structurally refuses the reversion signature and is
the correct verb for orchestrator writes there — a raw `git commit` is not.

Second confirmation from SL-233 PHASE-14.
