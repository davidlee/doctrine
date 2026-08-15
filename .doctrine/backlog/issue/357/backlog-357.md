# ISS-357: governance-confirmed marker is snapshot-only

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

`design resume` / `design show` print a `governance-confirmed — current` line. That
line is computed from the **run's own snapshot** of governance, taken when the run
confirmed it. It cannot see the live governance edge, so an ADR accepted, a policy
amended, or a standard added *after* confirmation leaves the marker still reading
`current`.

## Why it matters

The marker's only job is to answer "is the governance I designed against still the
governance in force?" — and it answers a strictly weaker question: "did I confirm
governance at some point?". A long-running design run (the norm: SL-233 ran 17+
revisions over days) is exactly where the live edge is most likely to have moved,
and exactly where the marker is most trusted.

A stale-but-green marker is worse than no marker. It converts an unanswered
question into a confidently wrong answer.

## Evidence

- `019ffa7e-248d` — governance-confirmed marker is snapshot-only (no edge-drift signal)

## Shape of a fix

The snapshot is the right thing to *design against* — that is the point of pinning
it. What is missing is a **drift signal** alongside it: compare the snapshot's
governance digest against the live corpus at read time and render a third state
(`confirmed — drifted, N entities changed`) rather than collapsing to `current`.

## References

- `SPEC-029` — Design run engine
- `mem.fact.design-run.snapshot-outlives-the-binary` — the snapshot's compatibility constraint
