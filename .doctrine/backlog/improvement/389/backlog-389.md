# IMP-389: Derive the next traversal candidate

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

The cursor is **declared**, never **derived**. An agent picks a node and submits a
`traversal` declaration; nothing computes a candidate from graph state. So
"cursor unset" is a representable and durable condition, and the engine's only
response is to render `cursor unset STALE` and carry on.

In SL-243 the cursor was cleared when `inq-4` was disposed at revision 9 and was
still unset at revision 17 — eight revisions and four further decisions later,
including across a context break. The run kept working; the field just stopped
meaning anything.

## What already exists, and why this is a small delta

Most of the machinery is built. `frontier()` ranks eligible nodes by kinship,
posture and `needs_in_degree`, excludes blocked nodes, caps at seven and reports
an `omitted` count. `frontier_candidates` has a documented cursorless branch. The
ranking is real and the eligibility rules are correct.

What is missing is the last step: the engine offers a ranked list and requires the
caller to choose, rather than naming the head of it. That is a defensible design —
it keeps the agent in charge — but it means the traversal state degrades silently
when the agent simply never chooses, which is what happened.

## Prior art

hydra (RFC-026 E8.5) derives both: `ready` is every open head whose dependencies
are satisfied, `next` is the first ready head in pre-order. Neither is stored, so
an unset or stale cursor is **not representable** — the question "where am I?"
always has an answer computed from the graph.

## Shape

Cheapest useful version: a derived `next` on the envelope alongside `frontier`,
computed by the existing ranking, with the declared cursor overriding it when set.
The agent keeps its authority (a `user-pinned` cursor still wins outright); the
run stops having a hole where its focus should be.

Worth settling: whether a derived `next` should also suppress the `STALE` marker,
or whether staleness should stay visible as evidence the agent stopped steering —
the latter is more honest and was how the condition was noticed at all.

## Provenance

Argument and instrument: **RFC-026 E8**. Observed across CHR-049's induced
context break, where the fresh agent had to be told by hand that the cursor
needed re-picking — one of five facts the continuation prompt carried because the
run could not.
