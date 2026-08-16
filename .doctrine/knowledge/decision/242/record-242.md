# DEC-242: Skeleton-informed implementation with a blind test author

## Why this is not prototype promotion

Promoting a prototype is a sin for one mechanical reason: code written under
relaxed constraints gets grandfathered past a bar nobody ever re-derives. The
relaxations become invisible, because after the fact there is no way to tell
which parts were justified and which were merely tolerated.

Red-first breaks that — but only if the bar is authored independently. A test
suite written with the skeleton in context is not an independent bar; it is a
transcript of the skeleton with assertions around it, and it looks identical to
the real thing. That is why the blindness sits on the *test author* seat
specifically, and why it is the operative clause of this decision rather than a
refinement of it. A version of this arrangement without it says something
materially weaker.

## What the prototype is for

An oracle, not a source. Across two rounds in `proto/SL-238-types` it found four
defects in `design.md` — two rank-type errors that would have silently widened a
delete, a refusal message that had lost its subject, and a bound the design
claimed to reach and did not. Each surfaced because the design and the compiler
disagreed *audibly*. Preserving that audibility through implementation is the
whole design of this arrangement; every seat rule below follows from it.

## The tension this decision holds open

`RFC-026`'s `P8` (*rough design, cheap implementation, disposable trace*) names
probe-branch merging as the failure mode to watch for first, on the grounds that
it will look like efficiency. This decision does not dismiss that. It accepts
that the branch stops being disposable, and pays for the privilege with a seat
that cannot see it. Whether that price is sufficient is not yet known — `RFC-026`
`E10` is the evidence home, and this decision is a second field trial rather than
a settled practice.

The honest failure mode remains: if the blind seat is ever relaxed "just for this
phase", the arrangement collapses to cherry-picking and nothing in the artefacts
will record that it did.

## Seat rules

The three seats are not symmetric, and the asymmetry is the point.

- **Planner** — reads the prototype, and should: it is the only seat that can
  assess which phases the skeleton actually covers. Guarded, because a planner
  that shapes phases around the prototype's diff will under-plan §5, which has
  no prototype code at all.
- **Test author** — blind. Works from §7's assertion list and the design.
  A design gap it cannot resolve is a finding to raise, not a hole to fill
  from source.
- **Code author** — reads both, and is therefore the only seat where a
  design/prototype conflict can land. It reports; it does not adjudicate.

See [[mem.pattern.review.bind-scope-bar-and-never-self-rule]] — the same
principle this slice has been governed by since `RV-358`, applied to code
instead of to review findings.
