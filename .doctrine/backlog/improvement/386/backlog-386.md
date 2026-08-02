# IMP-386: Cascade staleness from a changed answer

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

The inquiry map gates **forward** and never propagates **backward**. `needs`
edges stop you working on X until Y settles (`is_blocked`, `blockers()`, the
`blocked` total, frontier exclusion). Nothing does the reverse: when Y's answer
*changes*, X is not reconsidered, flagged, or even counted.

So a design run can hold five accepted decisions of which two are quietly
premised on a third that has since been re-answered, and no surface anywhere says
so. The map knows the dependency exists — it is the same edge — and uses it in
one direction only.

## Not to be confused with what already exists

Doctrine does have invalidation machinery, and it is good: DEC-066's rule that a
changed section fingerprint invalidates the attestations and reviews bound to it
(`ChangeEvent::EvidenceInvalidated`, `ReviewInvalidated`, `invalidation_rows` in
`src/design_run/run.rs`). That operates on the **drafting/review axis** — prose
and the evidence bound to it.

Nothing equivalent exists on the **inquiry axis**. The concept is already proven
in this codebase; it was never extended to the graph.

## Prior art

hydra (RFC-026 E8.5) cascades over the *union* of `parent` and `blocked_by`
edges: re-answering a head reopens its children and everything blocked by it,
recursively, and requires a cycle check on that union. Reopened heads carry their
`prior` answer so the follow-up is *"does this still hold?"* rather than a cold
re-ask.

## Shape

Two decisions are load-bearing and neither is obvious:

1. **Which edges cascade.** `needs` clearly. `parent` is arguable — doctrine's
   parent edge is narrative nesting, and in SL-243 it went unused entirely
   (RFC-026 E8.3), so cascading it may propagate nothing or may propagate too
   much.
2. **Cascade or flag.** Automatically reopening resolved nodes is destructive and
   surprising; marking them *stale* and surfacing the count is weaker but
   reversible. hydra chose reopen; it can afford to because it retains `prior`.

**Cascade without [[ISS-303]] is the destructive form** — reopening a node today
discards its disposition, so an automatic cascade would silently delete the
rationale of every downstream decision. ISS-303 first, or cascade only as a
staleness flag.

## Provenance

Argument and instrument: **RFC-026 E8**. Verified absent by grepping
`src/design_run/` for cascade/reopen/invalidation and finding only the
section-and-evidence axis.
