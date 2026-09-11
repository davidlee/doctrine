# ISS-450: needs edge at node creation emits no change row

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

A `needs` edge declared on a **new** node lands in the graph and emits no
`needs_added` row. The same edge declared against an **existing** node emits one.

Witnessed on this repo's own `SL-259` design run, 2026-09-11, while populating
the exploring-stage inquiry graph:

```text
rev 2  (5 nodes, 2 carrying needs)   → 5 node_created rows, 0 needs_added
rev 3  (10 nodes, 8 carrying needs)  → 10 node_created rows, 0 needs_added
rev 4  (1 existing node, 1 needs)    → 1 needs_added row
```

Ten edges landed silently. They *are* in the graph — `design show` reports all
ten as blockers with the right reasons — so this is not a dropped key. It is a
material change with no row.

The `node_created` row renders `parent=` but has no `needs` term, so the
creation row does not subsume the edge either: nothing in the log says the edge
exists.

## Why it matters

`SL-256` established the contract that a material change emits its row, and an
agent's only observable on a successful apply is the rows. Ten silent edges is
the same failure mode as `ISS-355` (*successful apply prints no change row*),
surviving inside the fix for it — and the edges are load-bearing: they are what
`blocked` and the frontier are computed from, so an agent that mis-declares one
at creation gets no signal and discovers it later as an unexpected blocker.

`NeedsAdded` is a member of `ChangeEvent::EMITTABLE`, so this is a missed
emission at the creation branch, not an absent vocabulary.

## Where

`src/design_run/run.rs:1294` — the create branch returns a lone `NodeCreated`
row before reaching the needs diff at `run.rs:1330`, which only the update path
below it runs. The create branch builds `parent` and `provenance` terms and no
`needs` term, which is the other half: neither a row nor a term records the edge.

## References

Found while designing `SL-259` (*Truthful apply*), whose leg 2 is "success
implies rows that tell the truth". Same class as `ISS-367` (*`live_acts` is
blind to same-kind replacement, so `ActInvalidated` under-reports*) — a row the
contract promises and the code does not emit — and the same shape as `ISS-327`
(*a branch entered by subject state reads a key the other branch ignores*).
