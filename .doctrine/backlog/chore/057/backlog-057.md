# CHR-057: Retract a needs edge

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`doctrine backlog needs <DEPENDENT> <PREREQUISITE>...` appends and cannot
remove. `doctrine unlink` reaches only tier-1 `[[relation]]` rows, and `needs`
is not one — it is a typed axis in `[relationships]` alongside `after` and
`triggers`, carrying per-edge payloads, which is why it is stored apart from
the uniform rows (SL-048, "the cut").

So a prerequisite that turns out not to be one has no exit. The storage rule
forbids hand-editing the TOML, and no verb retracts.

## How it surfaced

`ISS-314` `needs: IMP-392`, and `IMP-392`'s own body pinned the reason: it
waited on the concluded-pass marker **alone**, with the instruction *if the
marker is ever split out, re-point that edge*. The marker was split out and
landed (2026-08-07). The prerequisite is discharged, the edge cannot be, and
`ISS-314` reads blocked on delivered work.

That is not a bookkeeping nit: inbound `needs` on an unsettled record is the
actionability gate (ADR-017), so a stale edge suppresses an item from
`doctrine next` and `blockers` indefinitely.

## Shape

The obvious symmetry is `doctrine backlog needs --remove` (or a sibling verb),
validating that the edge exists and refusing silently-successful no-ops. `after`
has the same gap and the same fix; `triggers` likely too — check before
scoping to one axis.

Worth deciding at the same time: whether retraction should be *append-only with
a discharge marker* rather than a delete. A prerequisite that was real and got
satisfied is different from one that was wrong, and the audit trail is cheap.
Do not over-build it — the append-only reading is a genuine option, not the
obvious answer.

Related: `IMP-392`, `ISS-314`, `ADR-017`, `SL-048`.
