# RSK-005: backlog_order adapter: duplicate ItemId in &[OrderInput] silently corrupts the NodeId bimap

Surfaced by the codex adversarial pass during SL-039 PHASE-02; deferred to backlog
rather than expanding the corrective phase (per
`mem.system.lifecycle.defer-needs-backlog-before-close`), filed at SL-039 close.

## The fragility

`src/backlog_order.rs` builds a `NodeId ↔ ItemId` bimap (`by_item` / `by_node`)
while folding `&[OrderInput]`. If two rows carry the **same `ItemId`**, `by_item`
silently overwrites (last write wins) while `by_node` retains **both** NodeId
entries — the two maps disagree, and the ordering output is corrupt without any
error. The pure adapter does not fail loud on this precondition violation.

## Why it's latent, not live

Design §5.4 scopes the adapter input to "one row per non-terminal item", and
PHASE-03's `project` (the only production caller, `src/backlog.rs` ~`:560`)
enforces distinct `ItemId`s at the projection boundary (DD4). So it **cannot arise
in production today**.

The risk is for any **future** adapter caller that bypasses `project` and feeds
`&[OrderInput]` directly — they inherit a silent-corruption footgun with no guard.

## Candidate fix (not scoped here)

Make the adapter fail loud on a duplicate `ItemId` precondition violation — e.g.
return an `Err` from the build path when a second row reuses an `ItemId`, rather
than half-updating the bimap. Cheap, turns silent corruption into a clear error.

Related: `mem.system.engine.cordage-rc-budget-closed-null`.
