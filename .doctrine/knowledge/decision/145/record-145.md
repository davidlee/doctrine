# DEC-145: The composed knowledge read rides `doctrine inspect`

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## What was already there

`SL-246`'s scope says every kind's `show` emits outbound relations only. True,
but it reads as though no inbound surface exists. One does:

```
$ doctrine inspect SL-244
inbound:
  originated from: IDE-047, IMP-391, … ISS-315
  concerned by:    DEC-140, DEC-141, DEC-142, DEC-144, EVD-012
  shaped_by:       DEC-120, … EVD-012, QUE-206
```

Kind-agnostic, grouped by inbound label name, ids only. So this slice is not
building an inbound view — it is giving the existing one a content level. That
reframing is what the decision turns on.

(`EVD-012` appearing under two labels is a real sub-problem, but it belongs to
the label-curation question, not here.)

## Why not the per-kind `show` flag

It answers the question where an agent actually asks it. It also multiplies the
work by the number of composing kinds, wires the same derivation five-plus
times, and puts a byte-exact `SPEC-013` golden behind each copy. `backlog show`
already demonstrates both halves of that: `derive_fulfils_inbound`
(`src/backlog.rs:1368`, SL-176 PHASE-03) proves an entity `show` *can* carry a
derived inbound axis, and that it is bespoke per-kind work every time.

## Why not a third verb

`knowledge digest <ref>` is the cleaner name for what the reader wants. But it
either re-derives the inbound set — parallel implementation — or wraps
`inspect`, at which point the flag was the cheaper spelling of the same thing.

## The deciding argument

`inspect` already carries `--transitive --direction inbound --labels`. `IMP-398`
S5 — the recursive knowledge view — is that traversal with content attached.
Putting the content level on `inspect` means SL-246 objective 3 (*factor the
derivation so the closure extends it rather than replaces it*) has a real home
rather than a speculative one. On either alternative, S5 would have to be built
somewhere new.

## What this forecloses

The composed read stays an explicit second command. If the project later wants a
design's knowledge records rendered *by default* when someone reads the slice,
that is the per-kind arm and this decision would have to be revisited.

Shapes [[SL-246]]. Resolves that slice's `inq-1` (scope `OQ-1`).
