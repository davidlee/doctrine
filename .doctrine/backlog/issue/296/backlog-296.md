# ISS-296: Local clones inherit a stale commit-graph

A `git clone --local` copies `objects/` wholesale, so it inherits the source
repo's **commit-graph** — a derived cache. The clone keeps a narrower ref set
than the source, so commits the graph names arrive **unreachable**. They survive
as `dangling` and `git fsck` is content — until something prunes them, at which
point the graph names a commit that is gone and `git fsck` exits 16 with
`failed to parse commit <oid> … for commit-graph`, on a repo whose objects and
packs are all intact.

## Measured (SL-241 PHASE-05, F-P05-28)

This repo carries a 6-file commit-graph chain and 334 dangling objects;
`c89b124a5b277d6bf182a44ad69d3efa723e53ba` is one of them, reachable from no ref
here. The spike-capsule heavy fixture is a narrow local clone of this repo
(`refs/heads` + `refs/tags` only), so it inherited both. Inside a probe run git's
auto-maintenance pruned the object and the inherited graph went stale, surfacing
as `harvest/fsck-failed` — attributed to the capsule rather than to the fixture.

It cost roughly a session to diagnose, because every packfile verified, the
multi-pack-index verified, and the refusal token pointed at the wrong subject.
The signature worth recognising: **`git fsck` fails, `verify-pack` passes on
every pack, and `-c core.commitGraph=false` makes the failure disappear.**

## Already fixed for the one consumer that hit it

`quiesce_clone` in `scripts/spike-capsule/control/pipeline.sh` — drops the
inherited graph and disables `gc.auto` / `maintenance.auto` on each provisioned
clone, so background housekeeping cannot mutate a repo whose refs and object
count are the observable.

## What is left to decide, and why it is an issue not a rig note

The rig is one consumer. Any local clone of this repo inherits the same
landmine — fixtures re-cut from main, capsule rigs, archive clones. Worth
deciding once:

- strip `objects/info/commit-graph*` in any clone-and-measure helper; or
- keep the source free of graphed-but-dangling commits; or
- accept it and document the signature above, so the next occurrence is
  recognised in minutes rather than a session.

## Explicitly NOT proposed: `git gc` on the primary repo

153 of the danglings are commits, 111 with no same-subject twin on any ref, many
already past the default 2-week expiry — including several `WIP on …` stash
commits (residue of the incidents AGENTS.md records). Most are probably reaped
dispatch-worker commits whose content landed under another message, but
subject-matching cannot prove that, and pruning is not the lever that fixes this:
new dangling commits accumulate here continuously, so a gc today leaves the next
clone exposed anyway.
