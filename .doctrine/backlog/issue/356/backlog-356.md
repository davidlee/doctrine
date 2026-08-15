# ISS-356: Materialise renders sections in declaration order

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

`design materialise` emits sections in **creation order**, not in `seq` order.
Compounding it, a batched `declare` claims `seq` in **id-sorted** order, so a
ten-section batch lands as `sec-1, sec-10, sec-2, …` and the rendered `design.md`
reads `1, 2, 3, 4, 5, 10, 6, 7, 8, 9`.

There is no reorder verb. The id↔heading mismatch is permanent for the life of the
run — correcting it means re-minting the run.

## Why it matters

`design.md` is the run's **reader-facing** output: the artefact a human reviewer
and an external adversarial reviewer both read. A document whose sections are out
of order reads as careless, and the reader cannot tell whether the ordering is
meaningful or accidental.

It is also unrecoverable in-run, which makes it worse than a display bug: an agent
that batches its declares — the token-efficient way to declare — is silently
punished for it, and only finds out at `materialise`, after the batch is
irreversible.

## Evidence

- `019ff53c-902e` — materialise renders sections in declaration order (1..5, 10, 6..9)
- `019ff986-999b` — batched declare claims `seq` in id-sorted order

## Shape of a fix

Two independent halves; either alone is an improvement:

1. **Render by `seq`** rather than by creation order at materialise time.
2. **Assign `seq` by batch position** rather than by id sort at declare time.

A reorder verb (`design apply` act that rewrites `seq`) would additionally make it
recoverable, and is the only part that needs a payload-contract decision.

## References

- `IMP-393` — reader-facing design render for review
- `ISS-320` — re-adopting an edited `design.md` needs a section map nothing emits
