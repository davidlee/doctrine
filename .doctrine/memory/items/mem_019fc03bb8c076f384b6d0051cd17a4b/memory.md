**Signature.** `git fsck` exits 16 with `error: Could not read <oid>` +
`failed to parse commit <oid> from object database for commit-graph`, while
`git verify-pack` passes on **every** pack and `multi-pack-index verify` is
clean. The objects are fine. The **commit-graph** is stale.

**Confirm it in one command** — if this exits 0, the graph is the sole cause:

```sh
git -C <repo> -c core.commitGraph=false fsck --no-progress --connectivity-only
```

**Why it happens.** `git clone --local` copies `objects/` wholesale, so the
clone inherits the source's commit-graph — a derived cache. The clone keeps a
**narrower ref set** than the source, so commits the graph names arrive
**unreachable**. They sit there as `dangling` and fsck is content. Then anything
that prunes — usually git's own auto-maintenance, triggered by a fetch — removes
them, and the inherited graph now names a commit that is gone.

Two conditions must coincide, which is why it stays hidden: the graph must cross
a clone boundary that narrows refs, **and** something must prune afterwards
without regenerating the graph. A normal `git gc` rewrites the graph in the same
run, so it is self-healing; the clone is where the pairing breaks.

**Fix, for any clone-and-measure harness** — do this at provisioning, not after:

```sh
rm -rf -- "$repo/.git/objects/info/commit-graph" \
          "$repo/.git/objects/info/commit-graphs"
git -C "$repo" config core.commitGraph false
git -C "$repo" config gc.auto 0            # also stops background repack/prune
git -C "$repo" config maintenance.auto false
```

The `gc.auto 0` half matters independently: if the harness asserts on **object
counts** (`count-objects`), a background prune moves the number the assertion
rests on. A measurement whose subject is being tidied up underneath it is not a
measurement.

**Do NOT reach for `git gc` on the source repo.** It is not the lever — new
dangling commits accumulate continuously (every branch delete, reset, stash,
reaped dispatch worker), so a gc today leaves the next clone exposed. And in a
shared repo it prunes unreachable commits that may be the only copy of work.

Measured in doctrine at SL-241 PHASE-05 (F-P05-28): cost about a session,
because the refusal token named the wrong subject entirely. See [[ISS-296]].
