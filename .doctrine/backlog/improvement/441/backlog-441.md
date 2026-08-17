# IMP-441: Relation targets do not resolve across corpora

A relation resolves only within the corpus that authors it. Two doctrine-governed
repos side by side therefore cannot link at all:

```
$ doctrine link EVD-002 disputes EVD-019
Error: `EVD-019` does not resolve to an entity
  (no EVD-019 at …/oubliette/.doctrine/knowledge/evidence/019)
```

So every citation across the boundary degrades to a sentence — invisible to
`doctrine backlinks`, `doctrine relation census`, the concept map, and every
reachability question. `oubliette:ADR-003` clause 3 documents the refusal
verbatim rather than prescribing a command that fails, and states the cost: the
disputed `EVD-019`/`oubliette:EVD-002` pair is held together by prose on both
records, and nothing enumerates what disputes what across the boundary.

**The asymmetry is the sharp part, and it points the other way.** That refusal is
what oubliette sees, because doctrine's `EVD-019` genuinely has no counterpart
there. Run the mirror image from doctrine's side and it does not refuse:

```
$ doctrine link EVD-019 disputed_by EVD-002
```

`EVD-002` resolves here — to *`claude -p` exposes token usage metrics without
usage billing*. The edge is written, it is false, and nothing reports it. A
missing target is a refusal; a **colliding** target is a confident wrong answer,
and `ADR-022` clause 3 records four such collisions among the ids needed to
describe this problem at all. So this item is not only about reachability: the
current state has a live footgun in it.

## Why it is worth raising now

`oubliette:ADR-003` is the **second** consumer to want this and the first to be
blocked by it. `IMP-016` (closed) is the earlier one, at a different altitude.
`ADR-022` is the third and fixes a prose form (`oubliette:EVD-002`) precisely
because there is no resolvable one. The refusal
may well be correct — an unqualified id is genuinely ambiguous once there are two
corpora, and a dangling target that resolves against a sibling checkout's
presence on disk would be worse than prose. What is missing is a decided
position, not necessarily a feature.

## Shape, if built

Sketches, none chosen:

- a qualified reference form (`<corpus>:EVD-019`) with the corpus declared once
  in `doctrine.toml`, so resolution never depends on relative layout on disk;
- an outbound-only unresolved-target relation, carried in the graph and rendered
  as a citation, verified by nothing — cheap, honest, and enough for `backlinks`;
- keep the refusal, and give prose citation a recognised form the tooling can at
  least enumerate.

Likely `RFC` territory before any of it — it touches `ADR-004` (relations stored
outbound-only, reciprocity derived) and the reference-form rules in
`glossary.md`.

## Boundary

Independent of `CHR-069`, `CHR-070` and `IMP-440`, all of which are written to
work without it. `oubliette:CHR-012` names this a doctrine feature request, not
a fix on that side.
