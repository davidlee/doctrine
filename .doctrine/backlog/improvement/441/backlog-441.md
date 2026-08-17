# IMP-441: Relation targets do not resolve across peer corpora

**Speculative — parked deliberately.** Carried so the cost is written down, not
because a decision is pending.

A relation resolves only within the corpus that authors it. Two doctrine-governed
repos side by side therefore cannot link at all:

```
$ doctrine link EVD-002 disputes EVD-019
Error: `EVD-019` does not resolve to an entity
  (no EVD-019 at …/oubliette/.doctrine/knowledge/evidence/019)
```

So every citation across the boundary degrades to a sentence — invisible to
`doctrine backlinks`, `doctrine relation census`, the concept map, and every
reachability question. `oubliette:ADR-003` clause 3 documents the refusal verbatim
rather than prescribing a command that fails, and states the cost: the disputed
`EVD-019`/`oubliette:EVD-002` pair is held together by prose on both records, and
nothing enumerates what disputes what across the boundary.

## Why it stays parked

The refusal may well be right. An unqualified id is genuinely ambiguous once there
are two corpora, and a target resolving against a sibling checkout's presence on
disk would be worse than prose — `POL-002`, and `ADR-022` clause 1 refuses that
shape outright. Three consumers have now wanted the edge (`IMP-016`, closed;
`oubliette:ADR-003`; `ADR-022`) and all three were served adequately by a
sentence. Reachability across a repository boundary is a real loss and so far a
cheap one.

What would move it: a third corpus, or a question that cannot be answered by
reading two documents.

## Shape, if ever built

Sketches, none chosen:

- a qualified reference form (`<peer>:EVD-019`) resolving through the peer
  declaration `ADR-022` clause 1 requires, never through relative layout on disk;
- an outbound-only unresolved-target relation, carried in the graph and rendered
  as a citation, verified by nothing — cheap, honest, and enough for `backlinks`;
- keep the refusal permanently, and give prose citation a recognised form the
  tooling can at least enumerate.

Likely `RFC` territory before any of it — it touches `ADR-004` (relations stored
outbound-only, reciprocity derived), `SPEC-018`, and the reference-form rules in
`glossary.md`.

## Boundary

**`ISS-443` is not this item and must not wait on it.** That a peer-intended
target *silently resolves to an unrelated local record* is a live wrong-write with
a shape-check fix that needs no cross-corpus resolution at all. It was split out
precisely so parking this speculative feature does not park the footgun with it.

Otherwise independent of `CHR-069`, `CHR-070` and `IMP-440`, all of which are
written to work without it. `oubliette:CHR-012` names this a doctrine feature
request, not a fix on that side.
