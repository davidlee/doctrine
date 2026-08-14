# CHR-063: sweep the memory corpus of retired dispatch mechanisms

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`SL-254` collapsed dispatch onto one confined-subprocess arm and deleted four
mechanisms outright: the disk worker marker, the `SubagentStart` stamp, the
`worktree pretooluse` wall, and the gated `worker_commit` MCP tool. The
project-local memory corpus was not swept with them.

## The measurement (`RV-356` `F-5`, taken 2026-08-14)

Items under `.doctrine/memory/items/` mentioning any of `SubagentStart`,
`stamp-subagent`, `dispatch-agent`, `worker_commit`, `arm-spawn`, `pretooluse`:

| corpus | matching items |
|---|---|
| project-local (`.doctrine/memory/items/`) | **104 live** (105 files, one already retracted) |
| shipped (`memory/`) | **0** |

The shipped corpus being clean at zero is the important half: nothing `doctrine
install` seeds into a client project carries a retired mechanism. The exposure is
local to this repo's own agents.

## Already done at `SL-254`'s reconcile — do not redo

One item was retracted in the reconcile pass, not deferred here:
`mem.signpost.doctrine.dispatch-claude-arm-wrong-base`
(`mem_019ee28ee9ee7d608a22dba762fdcc26`). It was the only one of the 105 **indexed
in the boot snapshot**, so every agent met it on every boot without retrieving
anything. Its entire subject — the `/dispatch-agent` in-session Claude arm — was
deleted by `SL-254`, which is why it was retracted rather than re-anchored. Its
absence from the snapshot's Memory index was confirmed after `doctrine boot`.

## What this card asks for

A `/reviewing-memory` sweep of the remaining 104. This is **per-item judgement,
not a find-and-replace** — the grep above is a candidate list, not a verdict list.
Three dispositions, and the split matters:

- **retract** — the item's whole subject is a deleted mechanism (as above). The
  memory has no true residue.
- **re-anchor** — the item teaches something still true, but names the mechanism
  as its example or vehicle. Rewrite against the post-`SL-254` shape: one script,
  one arm, `DOCTRINE_WORKER` set by the confining argv.
- **leave** — the item is explicitly historical, or the mention is a citation of
  what changed rather than an assertion that it exists. Retracting these loses the
  record of *why* the shape is what it is.

Expect the third bucket to be non-trivial: `SL-254`'s own knowledge records
(`DEC-203` … `DEC-217`) legitimately name the mechanisms they retired.

## Why it was deferred rather than done in the reconcile pass

Reconcile writes the changes the audit brief names, against artefacts the slice
owns. A 104-item corpus sweep with per-item retract/re-anchor judgement is a
different shape of work with its own skill (`/reviewing-memory`), and folding it
in would have held `SL-254`'s close open on unrelated judgement calls. Owner
ruling of 2026-08-14: mint a card. Not urgent — nothing gates on it — but it does
decay: the further the corpus drifts, the harder each item is to adjudicate.

## Provenance

- `RV-356` `F-5` — the audit finding and its measurement.
- `SL-254` — the slice that deleted the mechanisms.
- Recorded in `RV-356`'s `## Reconciliation Outcome` as deferred-to-card.
