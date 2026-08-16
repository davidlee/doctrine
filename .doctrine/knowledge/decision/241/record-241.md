## Decision

An act carrying a review disposition emits **two rows**: `ActRecorded` for the
recording, and `ReviewDisposed` for the disposition. This is the one place the
slice changes an existing observable rather than filling a missing one, and the
change is deliberate.

## Why

They are different claims about different things. *This act is now held by the
run* is what `ActRecorded` says, uniformly, for every act however it arrived.
*This is how the act disposed the current review pass* is what `ReviewDisposed`
says, and only the disposing arm has it to say. Collapsing them would mean the
log reports a recording under a name that describes something else — which is
the inference `ISS-355` objects to, reintroduced one arm deeper.

It is also the only reading under which `DEC-238`'s unified admit-store-emit
seam is actually unified. If the disposing arm suppressed its `ActRecorded`,
the seam would carry an exception keyed on a field of the record it is storing,
which is the asymmetry this slice exists to delete.

## The two alternatives, and why not

**Keep only `ReviewDisposed` on that arm.** Preserves current output exactly and
costs nothing today. It carves the original asymmetry back into the seam, and
leaves a reader inferring *the act was recorded* from a row about the pass.

**Fold the act kind into `ReviewDisposed` as a third term.** One row, no
information lost. Refused on two grounds. `DEC-237` already rejected extra terms
on the recording event, and the symmetric argument applies here. More concretely
it is unsafe: `ReviewDisposed` already carries up to two terms — `disposition`,
plus `reason` on the waived arm — so a third puts it level with
`WIDEST_PAYLOAD_EVENT`'s three-term exemplar (`StageMoved`), and `R5` records
that **neither** compile-time assert would catch a member that outgrew it,
because `WIDEST_ROW_BYTES` sums the budget rather than deriving the widest.

## Consequences

- The disposing path's row set goes from one row to two. Any test asserting an
  exact row set for it moves; that is a test update, not a semantic compromise.
- No pressure on `R5`: `ActRecorded` carries one term and `ReviewDisposed` is
  untouched.
- `every_material_event_kind_persists_a_change_row` needs the `every_event_fixture`
  ladder to drive `ActRecorded` on this arm as well as the others.
