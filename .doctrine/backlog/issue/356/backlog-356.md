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

## Corrected diagnosis, 2026-09-27 — shared root cause with ISS-356 / ISS-360

Verified at the `cluster:design-run` triage (`RFC-031`). `ISS-356` and `ISS-360`
are one defect. `Batch::validate` (`src/design_run/submission.rs`) returns the
batch keyed by `DesignId` in a `BTreeMap`, and `DesignId` derives `Ord` over its
raw string — so a batch is folded in **lexicographic id order**:

- `ISS-360`: parents *do* resolve against the batch's running state — a chain
  `inq-1 ← inq-2 ← inq-3` lands in one batch. It fails only when ids do not
  sort parent-first as strings: `inq-9 ← inq-10` is refused `unknown node: inq-9`
  (probed against the e2e fixture).
- `ISS-356`: materialise already renders by `seq` (`SectionGroup::document_order`).
  The defect is upstream: new sections claim `seq` in that same string order, so
  `sec-10` numbers before `sec-2`. Half 1 of the original *Shape of a fix* is
  already true; half 2 is this.

One fix closes both. Two candidates:

- (a) fold in submission order — contradicts the documented "a batch has no
  order" contract on `Batch::validate` (DEC-063);
- (b) numeric-aware `Ord` for `DesignId` — keeps the contract; changes every
  id-sorted rendering, probably for the better.

Triage leaned (b); the user concurred tentatively on that recommendation. Not a
decision — settle it at the fixing slice's design.
