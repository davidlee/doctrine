## The shape

An audit that proves a boundary holds is often written as an **emptiness
claim** — "no mount resolves under this repository", "nothing crosses from
here", "this set is empty". It is the natural phrasing and it is the one most
likely to be **both wrong and green-looking**:

- if the claim is false because of a *legitimate* exception, the assertion reds
  and the reflex is to weaken it until it passes — which quietly deletes the
  audit;
- if it is true only because the enumeration missed something, it passes and
  says nothing.

Witnessed on SL-241 (F-P04-8): the census witness "nothing crosses from this
repository into the capsule" was false. The sandbox profile ro-binds the
control-plane runners at `/rig` from the repo's own tree — which is the design
working as intended (I4a: runners enter read-only from outside the writable
root). The emptiness claim would have been "fixed" by excluding the case it
existed to describe.

## The form that survives

Assert the **exact set**, with each member's reason:

> one repo-derived mount — the control-plane runners, read-only at `/rig`,
> carrying no `.git`

The admitted exception is now stated rather than hidden, the assertion reds when
a *new* member appears (which is the thing worth catching), and a reader can
check the reason instead of trusting the emptiness.

Companion to [[mem.pattern.harness.grep-negative-needs-positive-control]]: that
one says a negative result needs a positive control; this one says a negative
result should not have been phrased as a negative in the first place.
