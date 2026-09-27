# ISS-360: Batch declare refuses an in-batch parent chain

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

A batched `declare` refuses when a node's parent is **also in the same batch** and
the chain runs two levels deep — declaring `A`, then `B` with `parent = A`, then
`C` with `parent = B` in one submission. Parent resolution appears to run against
the pre-batch state rather than against the batch's own accumulating result.

## Why it matters

The inquiry map's entire value is its **structure** — parent edges and blocking
edges. `RFC-026` E8.3 measured a real run at nine nodes and **zero edges of either
kind**, and this is one of the mechanical reasons why: the natural way to author a
tree is to declare it top-down in one batch, and that is exactly the shape the
engine refuses. The workaround — one submission per level — costs a revision per
level and is not obvious, so the path of least resistance is a flat map.

That makes this issue evidence in `QUE-218` (*Does the inquiry map earn a semantic
tier?*): the "under-featured vs under-used" fork cannot be read cleanly while a
mechanical obstacle sits in front of the feature.

## Evidence

- `019fc523-c616` — batch declare refused an in-batch parent chain two levels deep

## Shape of a fix

Resolve parents against the batch's running projection, not the pre-batch snapshot
— i.e. fold declarations in submission order and resolve each against the state
after its predecessors. A cycle check is then needed on the folded result, which
the flat form gets for free today.

## References

- `QUE-218` — does the inquiry map earn a semantic tier? (this is a confound in the evidence)
- `RFC-026` E8.3 — nine nodes, zero edges
- `DEC-063` — design-run mutation uses atomic sparse JSON declarations

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
