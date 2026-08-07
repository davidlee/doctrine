`doctrine backlog needs <DEPENDENT> <PREREQUISITE>...` **appends only**. There is
no `--remove`, no `--clear`, and no sibling verb.

`doctrine unlink` does not reach it either: it removes tier-1 `[[relation]]`
rows, and `needs` is not one. It is a typed axis in `[relationships]` beside
`after` and `triggers`, kept apart from the uniform rows because those axes carry
per-edge payloads (SL-048, "the cut"). Attempting it fails with *"`needs` is not
a relation label authorable by ISS via `link`"*, which reads like a permissions
problem and is really an absent feature.

**So do not go hunting.** If a prerequisite turns out to have been wrong, or gets
discharged by work landing elsewhere, the edge stays. The storage rule forbids
reaching for the TOML by hand.

**Why it matters rather than merely annoying:** an inbound `needs` on an unsettled
record is the actionability gate (ADR-017). A stale edge suppresses its item from
`doctrine next` and `blockers` indefinitely, and the item looks blocked on work
that already shipped.

**What to do instead:** record the discharge in prose on both items — the
prerequisite's body and the dependent's — so a reader who meets the edge finds
the correction. Then check whether `CHR-057` (*Retract a needs edge*) has landed;
it is the fix, and it also covers `after`, which has the identical gap.

Found 2026-08-07 when the concluded-pass marker was carved out of `IMP-392` and
`ISS-314` was left gated on a delivered prerequisite — `IMP-392`'s own body had
even left the instruction *"if the marker is ever split out, re-point that
edge"*, which turned out not to be a thing anyone could do.
