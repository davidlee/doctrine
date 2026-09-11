# Design-run cluster: entry point

Start here for work on the **managed design run** — the engine behind
`doctrine design start|apply|resume|materialise`, governed by **SPEC-029**
(*Design run engine*) and built by **SL-233**.

This is a map, not a status board. Live state is always queryable:

```
doctrine backlog list -t cluster:design-run --by sequence
doctrine inspect RFC-031            # the program's related slices / records
doctrine next                       # gated items are absent by design
doctrine blockers <ID>              # names the gate
```

## Where things live

| need | home |
|---|---|
| **plan: priority order, tracks, fitness bar** | **RFC-031** (*Design run fitness*) |
| evidence / measurement baseline | **RFC-026** items E8, E8.3, E8.5–E8.7 |
| inquiry map's purpose | **RFC-030** (*Inquiry map as a general interview substrate*) |
| the engine contract | **SPEC-029** |

RFC-031 was minted 2026-09-11 at the user's direction, **reversing** the
2026-08-15 assessment that no RFC was warranted: slices came (SL-251, SL-256)
but the cluster grew (~27 → ~38 open) and runs kept stalling, so a single home
for order and a fitness bar was missing. Put plan and priority changes in
RFC-031, not here and not in RFC-026.

## Tracks (defined in RFC-031 — membership only, never status)

T1 **Truthful apply** (**SL-259**) → T2 section edit loop → T3 contract
legibility residue → T4 map surfaced then judged → T5 review integration →
T6 durability / engine quality (opportunistic). Read RFC-031 for item lists
and rationale.

`area:design-run` means the **engine**; `area:design` means the **stage/skill**.

## The two gates

Both are `QUE` records, so the block is real — `next` and `blockers` honour them
(ADR-017).

- **QUE-219** — *Submission strictness against snapshot forward-compatibility.*
  Gates ISS-290/327/328/333. Settled as the **first design decision of SL-259**.
  Bound by `mem.fact.design-run.snapshot-outlives-the-binary`.
- **QUE-218** — *Does the inquiry map earn a semantic tier?* Gates
  IMP-386..389. Settled by **CHR-065** (`needs ISS-299`): surface the map, then
  re-measure against E8.3's baseline of 9 nodes / 0 edges / 17 revisions.

## Gotchas

- A slice cannot `references` a `QUE`; relate the QUE from RFC-031 (`related`).
- Two duplicate pairs remain unmerged: `ISS-348` → `ISS-320`, `IMP-357` →
  `IMP-364` (later filer wrote a full record; folding is authoring work).
  `ISS-346` was closed as a duplicate of `ISS-333`.
- The cluster tag is the dedup instrument — check
  `doctrine backlog list -t cluster:design-run` before filing, and tag new items
  with it (several post-2026-08-15 items were filed untagged).

## Related

- [[mem.fact.design-run.snapshot-outlives-the-binary]] — the constraint behind QUE-219
- [[mem.signpost.doctrine.backlog]] — backlog verbs
- [[mem.signpost.doctrine.knowledge]] — knowledge-record verbs
