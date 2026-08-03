When a workflow step looks too loose or too human to derive, the instinct is to
leave it as an agent's claim — or to drop it. That instinct is usually reading
the wrong cause. The obstacle is almost never that the *judgement* is subjective.
It is that the judgement's **artefact** was never recorded as structured state,
so there is nothing to derive over.

Record the artefact and the shape changes completely:

- the human still judges; nothing is made mechanical that was not;
- the engine derives the condition from the artefact, uniformly, with no special
  "trust the caller here" branch;
- the CLI can render the artefact directly, instead of an agent paraphrasing it
  into a turn and garbling it;
- downstream consumers that have not been imagined yet get real state to use.

## The tell

A guard whose satisfaction is existential — *someone asserted this, about
something* — rather than a query over a named artefact. In doctrine's design-run
gate, `DerivedDesignFacts::satisfies` scanned evidence rows for a matching
condition and never checked what the evidence was *about*. Fingerprint binding
made such a claim **expire**; it never made one **true** (ISS-285).

## The worked case

`governing-context-recorded` and `initial-concerns-recorded` guard
`exploring → inquiring` and looked irreducibly subjective — did the agent really
consider the governing context? On inspection each names a **user interaction**
whose artefact was never built: a confirmed governance edge set, and a reviewed
inquiry graph with a declared blocking set. Both are ordinary structured state.
Both make the condition derivable. See DEC-120 (the derived/attested/claimed
kinds and the 2026-08-03 sharpening) and DEC-121 (the interactions themselves).

The incumbent proved it before it was named: `ReviewStanding::acceptance_current`
is `is_derived() == true` and means *"a user acceptance covers current content"*
— a subjective human act, derived, because SL-233 happened to record its artefact
on that one edge.

## The companion failure

SL-233 built the design-run machine and specified its user interactions in a
single bullet, then elaborated the mechanism around them at length. The
interaction design was most of the point of the exercise and got the least of
its attention — which is exactly why the two conditions above ended up as
payload-free names guarding an edge with no user contact on it at all.

**When a slice's value is mostly in an interaction, specify the interaction
first and let the mechanism follow it.** Mechanism specified around an unbuilt
interaction ossifies into placeholders that later work has to excavate.

Related: [[mem.pattern.design.classify-at-authoring-not-from-behaviour]].
