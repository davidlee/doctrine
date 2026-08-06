# DEC-174: EVD and HYP contracts are elevated unchanged

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The contracts, as SL-159 ruled them

From SL-159's design §5.3, verbatim in substance:

- `EvidenceFacet { datum: Option<String>, provenance: Option<Provenance>,
  confidence: Option<Confidence> }`. `Provenance` is a new closed set —
  `Inspection`, `Experiment`, `Reproduction`, `Citation` — with kebab serde,
  `as_str`, and a `KNOWN` drift-canary, deliberately mirroring `Basis`.
  `confidence` **reuses** the existing `Confidence` enum rather than minting a
  parallel one. `supports`/`disputes` are **edges, not facet fields**.
- `HypothesisFacet { proposition: Option<String>, predicts: Option<String> }`.
  The RFC's candidate `tested_by` is dropped (`D5`) — derivable from the inbound
  `supported_by`/`disputed_by` edges. *Don't store what the edge yields.*

Checked against `src/knowledge.rs`: `Provenance` is exactly that four-set. The
implementation has not drifted from the ruling, so elevating the ruling and
describing the code produce the same text — which is what makes this safe.

## Why R2 does not bite

`SL-249`'s `R2` warned that transcribing the structs would launder the
implementation into governance. The warning is right in general and does not
apply here, for the same reason it did not apply to concept in [[DEC-172]]: the
contracts originate in a *design decision*, with reasons, in a slice that
closed. The code is downstream of the ruling, not the source of it.

All three previously-ungoverned kinds now reach `SPEC-019` by the same route —
elevation with citation — and `R2` retires.

## What usage says

Measured 2026-08-06 over `.doctrine/knowledge/evidence/`:

- 26 `EVD` records; 14 carry a populated facet.
- Population is **strictly lock-step** — every record has all three fields or
  none. Not one partial. This is the same binary pattern the slice card measured
  on decisions (35/142) and questions (4/38), and it is the signature of a
  population that either engaged the structured tier or never touched it.
- `Provenance`: experiment 12, inspection 2. `Reproduction` and `citation`
  unused.
- `Confidence`: high 10, medium 4. `Low` unused.

`SL-159` left `OQ-2` open — *"Should `Provenance` carry a free-text escape (e.g.
an `other` + detail) or stay a closed 4-set? Default closed (crisp-edge bar);
`datum` holds detail. Revisit if it feels narrow in use."*

Half the set is unused. The trigger has not fired, and the answer on current
usage is no.

## The `HYP` gap

`HYP` has no records at all, so its contract is governed from the struct and the
design with nothing from practice. The REV should say so rather than let the
row imply the contract has been exercised.

That absence now has a recorded candidate explanation in [[HYP-001]], which
proposes that capture follows *routing* — and that no shipped skill routes
hypothesis work to the kind. If that is right, the zero is a fact about
discoverability, not about whether the contract is well-formed, and it is no
argument for delaying governance.

## A corroboration worth keeping

`EVD` sits at 54% populated against decision's 25% and question's 11%. The
plausible cause is directly on this slice's topic: evidence records are usually
authored by hand with the datum already in view, while decisions and questions
are minted by design-run checkpoints that have no slot to carry content.

That is `IMP-403` lead 2 corroborated from a direction the slice had not used —
a *between-kind* comparison rather than a within-kind field count.
