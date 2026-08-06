# DEC-165: Governance amendment does not gate the wire fix

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The narrower reading of the precondition

SL-249's scope card states objective 4 as a precondition:

> a write seam covering seven kinds cannot derive its per-kind field sets from a
> four-kind spec, and inventing them would be *invention presented as
> derivation*.

That is sound where it applies. It applies to objective 1 (`knowledge edit` and
its per-kind facet subverbs) and to the facet half of objective 2 (a facet
payload on `CreateRecord`). Both need per-kind facet field sets, and those are
ungoverned for `EVD`, `HYP` and `CPT`.

It does not apply to the other half of the slice:

- **Objective 3** — the inert-key refusal — is a correspondence between
  `Declaration`'s field set and the design-run *subject* kinds (`inq-`, `sec-`,
  `att-`, `fnd-`, `cp-`). It never touches a knowledge record kind.
- **The prose half of objective 2** — a prose slot on `CreateRecord` — is
  kind-blind.

## Why that half is the urgent half

The loss this slice exists to close was prose. On SL-248's design run six
checkpoint dispositions reached for the nearest-looking key and sent their prose
as `body`. `Declaration::body` is *section* prose, consumed only behind a guard
on `derived.section_digests`, so on a `cp-` subject it was accepted and
discarded. DEC-155 through DEC-160 minted hollow.

Objective 3 would have refused all six at submission. The prose slot gives them
the key they were reaching for. Together they close the observed incident, and
together they need no knowledge governance whatsoever.

## Why the precondition dissolves rather than merely narrowing

The rulings objective 4 needs — SL-249's `OQ-4` (is `ConceptFacet`'s emptiness
designed or an omission?) and the `EVD`/`HYP` facet contracts — are taken in
this design run either way. The REV is the artefact that *records* them, and
ADR-013 lands it at reconcile regardless of what order the code ships in.

So objective 4 was never a mechanical gate. The rulings were, and they are made
here. What remains is a phase-ordering choice, and the ordering that serves the
slice's motivation is: rule now, ship the wire fix first, land the REV at
reconcile.

## The instance that proves the cost

This record was itself minted hollow by the disposition that created it — the
ruling was fully in hand as the checkpoint was written, and there was no slot to
carry it. Filling it required hand-editing both tiers, because `doctrine
knowledge` has no `edit` verb. Captured as a friction observation
(`019fd6c8-ecbf-7b71-9001-a4ba464daf48`).
