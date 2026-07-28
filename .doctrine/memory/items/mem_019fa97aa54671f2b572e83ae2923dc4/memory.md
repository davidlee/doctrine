A bound that exists to protect a *projection* (context budget, wire size,
display width) must never propagate into the *record* that projection reads
from. Truncating at write time destroys information no later reader can
recover, in order to buy space in a rendering that reader may not request.

**The tell is usually in the name.** In SL-233 the constants were prefixed
`ENVELOPE_*` — the envelope being the projection — and they were applied to the
persisted change row's schema. The prefix already declared the scope; the
violation was visible in the identifier and still went unnoticed through two
revisions and an adversarial review round.

**Check the cost before economising.** The store there was gitignored runtime
state under `.doctrine/state/`, bounded by a retention constant: neither
committed to git nor multiplied at scale. There was no cost to pay, so the
lossy write bought nothing at all.

**The shape of the fix** is to separate the two artefacts explicitly — stored
row keeps full fidelity, rendered row is bounded, and a `--full` surface reads
the store — rather than to pick a single cap that serves both badly.

Related: [[mem.concept.doctrine.storage-model]], [[mem.fact.doctrine.storage-tiers]].
