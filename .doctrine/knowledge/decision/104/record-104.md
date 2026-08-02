# DEC-104: Heuristic sets are stage framing, not runbook checklists

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Why this is a separate record and not an amendment to DEC-103

DEC-103 says *when* an obligation must be delivered. DEC-104 says *what kind of
asset carries it*. They are different claims on different axes, and conflating
them is precisely the error DEC-104 corrects — SL-233's sketch read DEC-103's
"delivered at the moment it takes effect" as though it also entailed "therefore
a runbook step", and spent a revision building a nineteen-step gate on that
reading.

DEC-103 is untouched and remains `accepted` as ruled.

## The shape of the mistake, kept on the record

Three successive framings of the same content, each defeated by a better
question:

| framing | question it answered | what it produced |
|---|---|---|
| *is this stated with more authority elsewhere?* | DRY | four deletions, two of which dropped an obligation where it fires |
| *when must this arrive?* (DEC-103) | timing | nineteen steps on one edge |
| *what kind of thing is this?* (DEC-104) | destination | one step, and a fragment carrying the rest |

The second framing was a genuine improvement that nonetheless produced a worse
artefact than the third, because timing alone promotes everything into a gate.
Worth remembering: a ruling that repairs one defect can install another, and the
tell here was a count nobody could justify rather than an argument anybody could
refute.

## The discriminator, in one line

**Can this be truthfully completed at a specific boundary, with a finite result
whose completion is necessary before advancing?** If yes, it is a gate-worthy
act, and acts are steps. If no, it is a lens applied throughout the activity,
and lenses are framing.

Verification strength is a *second, orthogonal* question. `DischargeOutcome`
refuses to let a caller claim `verified`, so a step with no verifier can never
say more than `attested`. That grades a step's **evidence**; it does not decide
whether the obligation is a step. A step shipping no verifier must, however,
say why mandatory attestation is worth blocking the edge.

> **Amended 2026-08-01** (owner ruling, RV-325 round 3 F-3), before this
> decision governed any authored asset. The rule first read *"could a verifier
> ever corroborate this?"* — which is unreproducible, because *could ever*
> admits an imagined future verifier for almost any lens. Completability is
> decidable at the boundary and gives mechanically unverifiable but genuinely
> mandatory human acts — semantic scope reconciliation, knowledge capture —
> somewhere honest to sit. Every assignment the original made survives.

> **Amended 2026-08-01** (owner ruling, RV-325 round 11 F-17). Round 10 recast
> the discriminator as an **authoring gate** — *every 2a step states a truthful
> completion condition in its text; one that cannot is 2b* — decidable over the
> whole set before any run. That stands. What does not is the clause round 10
> attached to it, *and checkable in run state*: it **contradicted the paragraph
> above**, and applying the gate is what surfaced that. It demotes the two acts
> the round-3 amendment protects by name, and it condemns four of
> `exploring.toml`'s five shipped steps, only `explore.research` carrying a
> verifier. **The clause is withdrawn from the classification limb and relocated
> to collection**: a stated condition is *state-visible* when both its terms can
> be read from run state, the adherence signal is collected only over those, and
> the kit reports which obligations it could not reach. State-visibility grades a
> condition's evidence exactly as a verifier grades a discharge's — this
> paragraph, one level down. Applied over all nine 2a obligations, all nine state
> a condition and survive as 2a; five of nine are state-visible.

## What this does not settle

- **Overridability.** Neither 2a nor 2b is project-overridable today; the whole
  `design-prompts/` store is embedded-only, and the `.doctrine/hymns` overlay
  does not reach it. Both the shipped authoring rule and DEC-102 promise
  otherwise. Backlogged, with `design.md §7`'s rejection flagged for reopening.
- ~~**Whether the discriminator can be cashed out.**~~ **Settled by the
  2026-08-01 amendment.** No surviving 2a step ships a verifier, and under the
  original verifiability rule that made the sort unreproducible. Under
  completability it is a statement about evidence strength, not classification.
  The codicil carries the residual obligation.
- **The authoring rule's text.** `install/design-prompts/exploring.toml:8-13`
  still presents two branches and no 2a/2b account. IMP-374 carries a third
  clause for DEC-103 and now needs a fourth for this.

Related: [[DEC-101]] obligation runbooks · [[DEC-102]] craft overridable,
invariants sealed · [[DEC-103]] delivery at the point of effect · [[DEC-077]]
the closed fragment store · [[DEC-078]] `name@digest` fragment receipts.
