# Prove an additive widening by reading one input two ways

## The situation

You widen a closed vocabulary — a new enum variant, a new mode, a new strategy —
and the justification for choosing *addition* over *modification* is that no
existing caller's behaviour moves. That justification is the whole reason the
additive route was chosen, and it is usually left as an assertion in a commit
message.

## The test that actually proves it

Take **one input** that both the old rule and the new rule can read, and assert
both readings **in the same test**:

    let both_present = /* the input that distinguishes the two rules */;

    assert_eq!(read_the_old_way(&both_present),  OldAnswer);
    assert_eq!(read_the_new_way(&both_present),  NewAnswer);

Two properties fall out of one test:

1. the old rule's contested cell is **pinned**, so a later "simplification" that
   folds the two rules together reds; and
2. the new rule is shown to be what moved the answer — not the input, not the
   surrounding machinery.

## Why the shared input matters

If the two assertions use different inputs, the test proves only that two
functions exist. The force comes from the bytes being identical: the *reading*
is the only variable.

It also acts as a design check. If you cannot write both assertions over one
input, you did not do an additive widening — you modified the existing rule and
gave it a flag. In SL-248 PHASE-09 this was explicit: the rejected alternative
was a dominance flag on the existing variant, and under that design one of the
two assertions could not have been written at all.

## Belt

Keep the pre-existing test of the old cell as well. It is cheap, and mutating
the old cell should red **both** it and the additivity test — one conviction
from the incumbent's own suite, one from the newcomer's.


## Related

- [[mem.pattern.testing.mutation-battery-synthetic-targets]] — how to convict
  the new rule and the old cell without the battery taking out its own runner.
