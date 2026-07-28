**A constant's prefix declares the layer it binds, and a layer may bind only
its own artefacts.** Two halves, and stating only the first is what let this
defect survive five recurrences in SL-233.

## Half 1 — a projection bound must not bind the record it projects from

A bound that exists to protect a *projection* (context budget, wire size,
display width) must never propagate into the *record* that projection reads
from. Truncating at write time destroys information no later reader can
recover, to buy space in a rendering that reader may not request.

**Check the cost before economising.** In SL-233 the store was gitignored
runtime state under `.doctrine/state/`, bounded by a retention constant —
neither committed nor multiplied. There was no cost to pay, so the lossy write
bought nothing at all.

## Half 2 — identity is bounded at admission, never at emission

Bound identity and closed vocabularies (ids, enum labels, event names) at
**admission** — the moment a value is created or accepted — and make exceeding
the bound a **refusal**. Never truncate them on the way out. Only gracefully
degrading prose may be elided at render time.

**Why this half is not optional:** a *correctly layered* render cap on an id
still destroys meaning. Two distinct ids sharing a 32-byte prefix truncate to
the same 32 bytes and render identically, so a change log stops naming which
subject changed. A truncated identity is a *wrong* identity, not a short one.
Half 1 cannot catch this — the constant was on the right layer.

## Why the name is not enough

The tell is usually in the identifier, and that is still not sufficient. In
SL-233 the constants were prefixed `ENVELOPE_*` — the envelope being the
projection — and the revision that *wrote the sentence* "a projection bound
must never propagate into the record it projects from" violated it in the very
next table, bounding the stored value "to `ENVELOPE_REASON_BYTES`". A visible
prefix and a stated rule both failed. What worked was a **grep-able test**:
no `ENVELOPE_*` identifier reachable from a storage-write or admission path,
demonstrated to fail when one is reintroduced.

## The fix shape

Separate the artefacts explicitly — stored record keeps full fidelity, rendered
form is bounded, a `--full` surface reads the store — rather than pick one cap
that serves both badly. Where a domain limit is genuinely wanted on stored
input, give it its **own** domain-layer constant enforced by refusal; do not
borrow the projection's.

## Two lessons about fixing recurrent defects

- **A containment check is not the fix.** SL-233 tried "prove every bounded
  container holds its contents with each scalar saturated *at* its cap". That
  verifies arithmetic while the premise is what's broken — it cannot detect
  that a cap sits at the wrong layer or is the wrong *kind* of bound. Two
  further instances sailed through it, one of them written inside the very
  criterion authored to stop the class.
- **Test a proposed mechanism by whether it retro-catches the recurrences that
  already happened.** A rule explaining only the newest instance is a patch
  wearing a rule's clothes. State the falsifier with the rule: another instance
  it does not catch means widen the rule, never patch the instance.

Related: [[mem.concept.doctrine.storage-model]], [[mem.fact.doctrine.storage-tiers]].
