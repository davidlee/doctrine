## The shape

`invalidation_rows` reported a recorded act's death by differencing
`live_acts(before)` against `live_acts(after)`. The set is keyed
`(ActKind, DesignId)`, and `act_id` builds the id as `prefix + act.as_str()` —
a **pure function of the kind**. Both act stores replace by that same kind. So
a displaced act and the act displacing it land in the identical slot: present
on both sides, empty difference, no row. The derivation could not see a
replacement even in principle, while `live_acts`' own doc promised it did
(`ISS-367`).

## The rule

**A derived difference reports a change only while the derivation happens to be
injective over what changed.** Nobody states that property and no test guards
it, so it fails silently. Before trusting a before/after difference to report an
event, ask what the set is keyed on and whether the event preserves the key.

The repair is not a discriminator bolted onto the key (an ordinal, a digest) —
that fixes one derivation and leaves the class. Emit the row at the seam that
**performs** the write, which is the only code that knows a replacement
happened. Keep the derivation for the cases it can actually see, and say in its
doc which those are; the two sets are then disjoint and no dedup is owed.

## Where it bites

Any retain-then-push store whose replacement key is reproduced by the
replacement. Returning the displaced record from the retain site costs nothing
and hands the caller the fact. Let the caller build the row, so the store never
learns the log's vocabulary.

See [[mem.pattern.design-run.leaf-rule-binds-tests]].
