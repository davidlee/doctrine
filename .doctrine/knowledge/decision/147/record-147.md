# DEC-147: Selection is split from rendering; the record's caption is selection-supplied text

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## `--transitive` is a sibling, not a deeper setting

The tempting reading of objective 3 is that `IMP-398` S5 will one day pass a
larger depth to whatever `SL-246` builds. That is not the shape of the code.
`doctrine inspect` carries two views with two renderers:

| view | renderer | produced by |
|---|---|---|
| `InspectView` | `render_from` | `inspect_from` — `in_edges` per overlay, 1 hop |
| `TransitiveView` | `render_transitive_human` | `transitive_from` — `walk_transitive`, N hops |

`--transitive` is documented as relation-only, with no actionability block.
`walk_transitive` yields reachable-key groups per label; nothing in it resembles
`InspectView`'s inbound groups. So there is no depth parameter to widen.

## What the two cases actually share

One thing: *given some selected records and a level, render their content.*
Everything else differs — one selects by `in_edges` under a label allow-list, the
other by bounded reachability.

So the seam goes exactly there:

- **selection** (impure) — walk, filter by label, dedup, produce an ordered list
  of `(caption, record ref)`.
- **render** (pure) — that list plus the level, to a block.

This is the pure/imperative split the project already requires, applied to work
the 1-hop case has to do anyway. It is not generality bought on spec.

## The caption is the whole trick

At one hop a record's caption is its inbound label — `shaped_by`, `concerned by`.
At depth 3 there is no single label; there is a path. A renderer whose grouping
key is `RelationLabel` therefore *must* change when S5 arrives, which is the
"replaces" outcome objective 3 exists to prevent.

Making the caption selection-supplied text costs nothing now and is the entire
difference between extending and rewriting.

## Dedup proves the split is real

`EVD-012` appears on `SL-244` under both `shaped_by` and `concerned by` at a
single hop. Selection has to dedup today. At N hops the same record arrives at
several depths. Keeping dedup in selection means the renderer never learns that
traversal exists.

## What stays out

No depth parameter, no traversal abstraction, no halting rule for
knowledge→knowledge edges — and one is needed, since `Shapes` legally targets
the record kinds themselves (`src/relation.rs:530-531`), so a naive walk runs
through records without stopping. S5 owns that.

Shapes [[SL-246]]. Resolves that slice's `inq-3` (scope `OQ-5`). Downstream of
[[DEC-145]] and [[DEC-146]].
