# IMP-392: Unify design-run findings onto the RV ledger

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`DEC-125` decides it: scrap the design run's runtime `Finding`
(`snapshot.rs:293-307`, written at `run.rs:1105-1156`) and enrich `RV` findings
with a **document-section reference**, so one model serves both.

`SL-244` specifies its gate conditions against the `RV`-backed model
(`DEC-126`). **This item delivers it.**

## What is here

- A section reference on an `RV` finding, carrying the **fingerprint** it was
  raised against — the hard part. `DEC-066` invalidation is snapshot-internal and
  `RV` has no fingerprint concept; the `design materialise` authored-watermark
  pattern is the shape to follow.
- Minting an `RV` on entry to `reviewing`, so the review is a first-class artefact
  from the start.
- The design run resolving and deriving over an `RV` — it references none today
  (`IntegratedReview.id` is a `DesignId`, not a canonical ref).
- Removing `Finding` and its `fnd-` declaration path, with the e2e suites that
  encode it migrated deliberately rather than green-chased.
- The derived outstanding-findings summary **by severity** that `DEC-126`'s
  `review-disposition-attested` is informed by.

## What this dissolves

`SL-233`'s design promised *"Findings remain runtime data unless promoted to
knowledge or the final authored review ledger"* (`slice/233/design.md:298-299`)
and the promotion leg was never built. Unification removes the need for it —
findings are born authored, so there is nothing to promote. Do not build a
promoter.

## Why it is not a tier violation

The run already crosses into authored state three ways: the authored watermark
guarding `design.md`, `DEC-086` checkpoint minting from inside `apply`, and the
project's own deliberate tier call for observations. See `DEC-125`.

Related: `DEC-125`, `DEC-126`, `DEC-124`, `DEC-066`, `DEC-086`, `ADR-007`,
`SL-244`, `SPEC-029`.
