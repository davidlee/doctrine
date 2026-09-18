A finding names one arm of a two-arm claim. The repair's scope then silently
inherits the finding's scope, satisfies the arm that was attacked, and leaves its
twin — while looking finished, and while being verified against the tree before
disposing.

Observed four times in one design review (`RV-370`, `SL-246`), each caught only
by a later round:

| finding | arm attacked | twin left behind |
|---|---|---|
| `F-13` / `F-18` | the text arm's markers | the JSON arm had no producer |
| `F-23` | the `Facets` level | the marker was unreachable at `Full` |
| `F-30` | the JSON arm's `Full` route | the text arm got the requirement and no mechanism |
| `F-31` | the record-level empty states | the BLOCK-level empty state, on both arms |

None was carelessness. Every one of those repairs was verified against the tree
before its disposition was written. The defect is structural: the finding frames
the scope, and a repair that answers the finding reads as complete.

## The counter, which costs one question

On **every** disposition, before writing it:

> Which arm did the finding name, and what is its twin?

"Arm" is whatever dimension the claim is quantified over — text vs JSON, one
level vs every level, per-record vs per-block, one caller vs every caller. If the
claim has a `for all` in it, the finding almost certainly named one member.

## The second-order version

When a review has named the *class* — as `RV-370` did three rounds before
`F-31` — fixing the next instance is not enough. **Sweep the class or it will
return as another finding.** `F-31` was found by a reviewer taking a
disposition's own sentence ("the third time a repair discharged one arm") at its
word and asking where the fourth was. The fifth and sixth (`X5`, `I6`) were then
found by sweeping rather than waiting.

## The sibling, and how it differs

[[mem.pattern.review.repair-closes-a-subset-of-the-stated-class]] is the same
genus: a repair closing one member and leaving the rest. The difference is where
the rest are written down.

- **That one**: the class is *enumerated in the artefact* — three vacuity modes
  in one sentence — and the repair closes one while the other two sit three
  sentences below their own statement. The counter is to **re-read the class
  statement** the repair came from.
- **This one**: nothing is enumerated. The twin is implicit in the claim's
  *quantification* — "on both arms", "at every level", "per record" — so there
  is no sentence to re-read. The counter is to **ask what the dimension is**,
  because the artefact will not tell you.

Both fail the same way from a reader's side: the repair answers the finding
exactly, and reads as finished.

See [[mem.pattern.doctrine.tdd-loop]] for the analogous discipline in code, and
`/feedback`'s step 3 (Generalize), which states the rule and is the step that
was being skipped in all four instances.
