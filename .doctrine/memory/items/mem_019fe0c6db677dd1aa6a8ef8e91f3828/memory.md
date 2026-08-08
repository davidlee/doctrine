# Pin a table's mapping by differencing the layer below it

When a lookup table decides whether input is accepted, the obvious test — *does
the checker agree with the table?* — proves only that the table equals itself. It
stays green under a table whose rows are **swapped**, because both sides move
together. `RV-349` `F-4` found exactly that: an inventory pin (*the table's key
set equals the wire type's serde key set*) is identical whether `body` maps to
`sec-` or to `cp-`.

## The shape that works

Two facts per cell, from two different layers:

- **effectful** — measured on the layer *below* the check, which does not consult
  the table. A differential: run a base input and the base plus the thing under
  test, and compare outcomes. Different outcome ⇒ the input did something.
- **refused** — measured on the full admission path, which does consult it.

Assert **exactly one** holds. Both false is silent acceptance; both true is a
table refusing something its own engine honours. A swapped table now fails twice,
in opposite directions, and neither failure can be papered over by editing the
table.

Landed as `SL-249` `PHASE-02`'s `I10` over
(`Declaration` wire key × design-run subject kind); the effectful side reads
`design_run::run::declare`, which is one layer under `Batch::validate` where the
check sits.

## Three things it needs to be honest

1. **Both vocabularies read from their own source.** Iterate the table's rows and
   the id enum's `ALL`, never a written-out cell list — otherwise coverage is an
   inventory that reads exhaustive and is not.
2. **A per-input value fixture that FAILS when missing**, not one that skips. A
   new key with no fixture must show up as a failing cell.
3. **A value that differs from the default.** A differential cannot see a key set
   to the value the engine already assumes when it is absent — `provenance:
   agent-proposed` and `reviewer: human` are both invisible that way.

## Prove it with controls, because this test passes trivially when broken

Three run on `SL-249` `PHASE-02` and each fired with the right message: swap one
row's home (fails at both the old and new kind), short-circuit the check (every
inert cell fails), and drop a state-fixture row (the cell it covered fails).
Same discipline as
[[mem.pattern.harness.grep-negative-needs-positive-control]] — a green assertion
you never saw fail is not evidence.

## The quantifier you are choosing, whether or not you say so

A cell is *some input at this coordinate*, not *every input*. Where behaviour also
varies along an axis the matrix does not range over — in `SL-249`'s case the
subject's create-versus-update state — the fixture silently picks the answer.
State the choice per row with its citation and record what is left uncovered
(`DEC-183`, `ISS-327`), or the matrix reads as wider than it is.

Related: [[mem.pattern.review.sweep-defect-class-not-instance]].

The nearest neighbour is
[[mem.pattern.testing.migration-oracle-restates-not-derives-from-ssot]] — *never derive
the expectation from the source of truth under test*. That is the same instinct
one step earlier: it says where the expectation may not come from, and this says
where it can come from instead when there is nowhere to restate it from.
