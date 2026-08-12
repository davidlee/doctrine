## The pattern

An adversarial reviewer asked *what is wrong with this* will return defects. It
will keep returning correct defects round after round and never write the fix,
because finding the fix is not the job you gave it. Every repair then gets built
by the author, against someone else's diagnosis, and author-built repairs of a
defect the author did not see have a poor survival rate — they close the instance
they were shown and narrow the class the defect belonged to.

Ask instead: **what shape would you write if this were your design?** State the
role change explicitly — you are not reviewing this, you are co-authoring it —
and ask for text you can apply rather than a list of what is missing.

## The evidence it came from

`SL-253`'s `RV-354`, five rounds on one blocker finding (`F-1`):

- Rounds 1–4 as review: four correct diagnoses, zero remedies. All four repairs
  author-built. All four contested.
- The only finding on that ledger to close cleanly (`F-2`) closed on a remedy the
  reviewer had written out verbatim, unasked.
- Round 5 as collaboration: a remedy, plus a defect four review rounds had missed
  (a callback protocol pinned invocation order and bound no return to the
  occurrence that produced it), plus two assertions correctly demoted from
  "tested" to "structural, with the signature as evidence".

## The cost, which is not optional

**Collaboration and verification are mutually exclusive uses of one reviewer.**
Having co-authored the remedy, that reviewer can no longer adjudicate it — it
would be verifying its own work, and it has spent the independence that made its
verdict worth having. In `SL-253` the reviewer said so itself and declined to
verify, which was correct.

So: spend the round deliberately. A co-authored repair has a second *author* and
no second *opinion*, and those are different things. If verification is required,
it must come from a reviewer who has not been spent.

## How to apply

- Use it when the diagnoses are landing and the repairs are not — that asymmetry
  is the signal, not the number of rounds.
- Say the role change in the prompt, and ask for the shape rather than the gap.
  Ask it to say where your current text is already right; a review framing cannot
  give you that.
- Keep one reviewer unspent if a verified verdict is needed downstream.
- **Record which you got.** A repair with a second author must never be filed as
  a verified one. Write the provenance next to the repair, including the fact
  that independence was traded for it.

Related: [[mem.pattern.testing.classify-the-expectation-before-trusting-the-assertion]]
— same family, about trusting where an assertion's expectation came from.

## How this composes with the no-self-ruling rule

[[mem.pattern.review.bind-scope-bar-and-never-self-rule]] says: when you find a
defect in your own remediation, do not rule on it yourself — put it to the
external reviewer, because authoring the bar disqualifies you from ruling on your
own compliance with it.

That rule and this pattern point at the same reviewer and ask for different
things, so the order matters. **Ruling is the default use; co-authoring is the
one you spend.** Ask for a ruling and the reviewer stays able to give you another
one. Ask it to write the remedy and it has joined the authorship the first rule
disqualifies from ruling — for exactly the reason that rule gives.

So: exhaust the ruling use first, and switch to co-authoring only when the
diagnoses are landing and the repairs are not. After the switch, route any
verdict you still need to a reviewer who has not been spent.
