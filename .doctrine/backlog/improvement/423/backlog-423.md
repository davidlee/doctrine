# IMP-423: Extract a slice-agnostic dispatch loop template from LOOP.md

`LOOP.md` is the dispatch-loop rulebook an orchestrator follows while driving a
slice's phases through workers. It works, and it has been written twice.

## The situation at `SL-248`'s landing

`LOOP.md` was edited **on both branches** and the two edits are for different
slices:

- on `sl-248`, it grew across ten phases of capsule work (~583 lines changed
  over the slice; `RV-352` `F-4`);
- on `edge`, it was edited for a slice that has **since closed**.

So the merge is not a case of picking a side — neither side is the future. Both
are slice-customised instances of a rulebook that has no template.

## What to do

Take the **slice-agnostic** parts from both sides and land them as a template:
something copied into a slice at dispatch time and customised there, rather than
a single file that accretes one slice's specifics and then has to be reconciled
with another's. Slice-specific rules stay in the copy; the template carries what
every dispatch loop needs.

## Why it is worth doing rather than tolerating

Every dispatch slice currently re-derives the same rules, and the evidence that
they are re-derived rather than remembered is that `LOOP.md` reached ~560 lines
against its own stated ~200-line cap, with compression owed at slice end and not
taken. A template makes the cap enforceable, because the thing being capped is
finally the same thing twice.

Related: `SL-248` adjudicated `LOOP.md` as a `scope-relevant` selector —
orchestrator infrastructure, touched but never delivered by the slice that
touched it. That adjudication is the symptom this item is the cure for: a file
every slice edits and no slice owns.

Originates from `SL-248` `RV-352` reconcile (`F-4`) and the owner's call at
landing.

## Resolution

Delivered with `IMP-424`, which had to merge the two lineages anyway. `LOOP.md`
stays at the repo root and is now an explicit template: its header says to copy
it to `.doctrine/slice/<N>/LOOP.md`, bake `<N>` in, customise § *Where this
runs* — the one section named as non-portable — and fire the loop at the copy,
folding anything durable the copy learns back into the root file at close.

The cap is now enforceable in the sense this item wanted: 298 lines against
`edge`'s 294 and `sl-248`'s 557, carrying both lineages, because the reusable
half went to the memory corpus behind § *Method* rather than accreting in a file
read every firing. The stated aim was ~220; see `IMP-424` for what the last ~78
would cost.
