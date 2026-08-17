# ISS-442: backlog inspect drops label-only relations

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`doctrine backlog inspect <ITEM>` silently omits `[[relation]]` rows whose label
is not one of the axes it enumerates. For a backlog item the practical casualty
is `related` — which is the **only** additional label a backlog source can
author, so the omission is total rather than partial.

The verb reports success, the row is on disk, and the reader shows nothing. It
reads as a no-op write.

## Reproduction

```
doctrine backlog new issue One --slug one     # ISS-001
doctrine backlog new issue Two --slug two     # ISS-002
doctrine link ISS-001 related ISS-002         # → "linked: ISS-001 related ISS-002"

doctrine backlog inspect ISS-001
# ISS-001 — One
# one · issue · open
# created … · updated …
#                                    ← no `relationships:` block AT ALL

doctrine inspect ISS-001
# outbound:
#   related: ISS-002                 ← the edge is there
```

The `relationships:` header is gated on the same axis set, so when `related` is
the only edge the section does not merely lose a row — it disappears, and the
item reads as having no relations.

**Second, smaller drop in the same renderer:** the `descriptor` on a
`references(concerns)` edge. `doctrine inspect` renders
`references(concerns): ISS-002 — "the seam"`; `backlog inspect` renders
`references(concerns): ISS-002`.

## Cause

`src/backlog.rs:1746-1790` renders relations by enumerating a **hand-maintained
allowlist of axes** — `fulfils` (inbound), `drift`, `needs`, `after`,
`triggers`, and `references` split by its three roles. The `if` at `:1768-1776`
gates the whole section on that same list. `src/relation.rs:45` carries 21
labels; anything outside the allowlist is never read and never rendered.

So this is not a filter on role or on validity — it is a closed list that does
not derive from the relation table, and a label added to the vocabulary does not
appear here.

## Why it matters beyond the missing line

- **STD-003 (no silent skip — a degraded read is disclosed).** A read verb that
  drops rows it does not recognise, without saying so, is the exact shape the
  standard forbids. The natural verify-my-edge command reports a false absence.
- **STD-001 (single-source named constants).** The axis list is a second,
  hand-maintained copy of a vocabulary that already exists in `relation.rs`, and
  it has already drifted from it.
- `doctrine inspect` is correct throughout, so the fix is a renderer defect in
  one place, not a model problem.

## Bound

Small. Either render the unrecognised rows generically (label + target, the
shape `inspect` already uses) or derive the axis set from `RelationLabel` so a
new label surfaces by construction. The second is the STD-001 answer and is
barely larger.

Worth a red test that authors a `related` edge and asserts it renders — the
class of bug that returns the moment someone adds label 22.

## Provenance

Reported by an agent mid-task, whose `link` looked like a no-op write.
Reproduced and characterised 2026-08-17 against `edge` at `113821d71`.
