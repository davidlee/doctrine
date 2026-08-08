# Objective 4's REV lands in a phase, not at reconcile

The structured facet carries the ruling. This is the part that does not fit a
field.

## Why the tension is real rather than procedural

`ADR-013` is not arbitrary about reconcile. A REV exists to *stage and approve*
spec deltas before they land, and reconcile is where a slice's findings are
written back — so governance amendments belong to the stage that already owns
writing settled truth. Landing one inside a phase genuinely does cut across
that.

What makes SL-249 different is that its amendment ships with a **control**, and
the control is code. `DEC-176` demanded an observable rather than an assertion
precisely because `SL-159` promised this same governance axis and never
delivered it; `D9` then spent five review rounds making the observable actually
bite. That control is a test in the repo, and a test in the repo is green or it
is red. There is no third state, and "red until reconcile" is the state in which
tests get deleted.

So the question is not *may a phase write governance* but *where can the control
be green*. Answer: wherever the prose is. The prose and its control cannot be
separated without giving up one of them.

## What this does not license

It does not make a phase a general vehicle for governance edits. The narrow rule
is the one in `consequences`: where a governance amendment's control is code,
the amendment lands where the control can be green. An amendment with no control
— or one whose control is a human reading — has no such pressure and belongs at
reconcile, as `ADR-013` says.

## The record's own irony

This record was minted hollow — `context`, `choice`, `rationale` and the rest
seeded empty — and its facet was filled by hand-editing
`record-182.toml`, because `doctrine knowledge` has no `edit` verb. That is
`IMP-403` lead 1 and the whole of SL-249 objective 1, demonstrated on the
decision that plans objective 4.
