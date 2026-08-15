# DEC-236: Unresolvable refs keep a count-only signpost on the listing surface

## Decision

`DEC-232` routes every `needs`/`after` ref-integrity failure off `backlog list`
and into `doctrine doctor`. That is right, and it leaves the class mentioned
nowhere on the surface where the work happens — today a malformed ref at least
produces a footer line.

So when `backlog list --by sequence` drops one or more unresolvable refs, it
emits a **count-only advisory to stderr**, naming no individual ref and pointing
at `doctrine doctor`:

```
backlog list: 3 authored needs/after refs name nothing — run `doctrine doctor`
```

## Why this does not reopen `DEC-232`

The footer's contract — *only what is needed to understand the rendered content*
— is untouched, because the advisory is not in the footer. stderr already
carries advisories in this exact voice beside the `Ordering::Degraded` cycle
warning (`src/backlog.rs:1159-1162`), and `--json` routes it there on the same
terms as the footer blocks.

Three properties keep it a signpost rather than a second report:

- **It names no ref.** The report stays `doctor`'s.
- **The count is over distinct `(dependent, axis, ref)` occurrences**, matching
  the undeduplicated `doctor` check rather than the footer's per-pair dedup. The
  axis is part of the key: `needs` and `after` are stored as separate authored
  arrays, so the same ref declared on both is two edges and two repairs. An
  earlier wording of this bullet said `(dependent, ref)` pairs while also
  requiring the both-axes case to count twice, which the stated key cannot do
  (`RV-358` `F-7`). The intent was always the two-repair count; only the key was
  mis-spelled.
- **It fires only under `--by sequence`**, because `--by id` never composes and
  so never probes. Adding a probe to `--by id` purely to emit the line would buy
  a warning at the cost of the read that order mode exists to avoid.

## Why it is worth the surface

The realistic failure is not "the user never runs `doctor`". It is that a target
gets **deleted out from under a live ref**, and the surface the user is working
on is `backlog list`. Measured 2026-08-16, the corpus carries **zero**
unresolvable authored refs — every authoring path already gates on
`kinds::ensure_ref_resolves` — so this is drift insurance, and the cheapest
possible form of it.

## Standing

Recorded because **no accepted decision covers it**. `DEC-231`…`DEC-235` say
nothing about a pointer, and `SL-238` §9 carries it as a drafting overrun. This
record is that overrun given a durable home, so a reviewer holding the five
decisions can find the sixth rather than inferring it from prose.

Related: `SL-238` §2 (the advisory), §4 (`BoundaryProbe.unresolved`), §5 (the
drift framing), §7 (five VTs). `STD-003` governs the adjacent but distinct case
of a degraded *read*; a ref that names nothing is a referential defect, not a
read failure, so the standard is not what compels this.
