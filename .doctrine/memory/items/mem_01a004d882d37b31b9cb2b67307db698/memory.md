# Close the class with an instrument, then sweep what it subsumed

Two moves, and the second is the one that gets skipped.

Evidence: SL-251 design, RV-357 `F-6`, five review rounds on one verification
pin. Each round repaired the case it was shown — the assertion's direction, then
its domain (struct fixtures only), then empty containers, then an untagged
`Shape` no key led to. Every repair was correct and each lasted exactly one
round, because each was written as a *case*.

## 1 — What terminates it is an instrument, not a fifth case

The sequence ended when the rule stopped being a list and became a **recursive
descent matching on the model with no wildcard arm** — so a new variant of the
model is a compile error at the checker rather than a silently unchecked path.
Enumerating instances loses to enumerating *the type*: the model is finite,
already written, and the compiler will hold it.

The tell that you are still writing cases: the rule names the places a property
currently appears ("every `Seq` and `Map` **key**"). The general form names where
it is *declared* and lets the model say where that is.

Corollary caught in the act: the coverage assertion written to close the class
was itself first scoped to *rows*, and one member of the model was not a row. A
generalisation fitted to the instances you have seen is still an instance.

## 2 — Then sweep the neighbouring text the new rule subsumed

The general statement does not delete the specific ones. In SL-251 the descent
table specified the `Named`-edge check, and the paragraph three screens down
specified it again at length — the exact duplication the design elsewhere
condemns, introduced by the fix for the duplication class.

Three things to check after any generalising repair, all within a screen of the
edit:

1. **Said twice?** Whatever the new rule covers, find its old statement and
   delete or demote it to motivation.
2. **Residue still accurate?** A rule that now reaches further makes the
   artefact's own "where this does not reach" paragraph incomplete. Extend it,
   with the wider claim actually verified rather than assumed.
3. **Discipline applied to the new assertion?** A new set-equality is subject to
   the same positive-control rule as the ones it replaces. Arguing that its two
   sides derive independently is not carrying the control.

## Why it kept going out unchecked

Because an external reviewer was available, each repair went out to be checked
instead of being checked first. Adversarial review is for what you cannot see;
it is not a substitute for reading your own edit against the enumeration it came
from. The self-check found a larger hole than any of the four contests — an
entire arm of the model with no defined check at all.

Inward form: [[mem.pattern.review.repair-closes-a-subset-of-the-stated-class]] —
check the repair against the class statement it came from.
Outward form: [[mem.pattern.review.sweep-defect-class-not-instance]] — sweep the
codebase for siblings.
This one is the *third* form: sweep the artefact for what the repair made
redundant or made wrong.