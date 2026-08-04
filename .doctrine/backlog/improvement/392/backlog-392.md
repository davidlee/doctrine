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
- ~~Minting an `RV` on entry to `reviewing`~~ — **done in `SL-244`** (user,
  2026-08-04, on `RV-344` `F-1`). `DEC-125` says the `RV` arrives on entry to
  `reviewing` and does not say what puts it there; this list used to claim it,
  and `SL-244`'s `sec-3` needs the pass to exist in order to bind a disposition
  to one. That slice mints it through `DEC-086`'s journalled intent — the seam
  `DEC-125`'s own rationale cites — so the review is a first-class artefact from
  the start without waiting on this item.
- The design run resolving and deriving over an `RV` — it references none today
  (`IntegratedReview.id` is a `DesignId`, not a canonical ref).
- Removing `Finding` and its `fnd-` declaration path, with the e2e suites that
  encode it migrated deliberately rather than green-chased.
- The derived outstanding-findings summary **by severity** that `DEC-126`'s
  `review-disposition-attested` is informed by.

## Three things `SL-244` specified against this item and cannot enforce without it

All three need this record open, so they land together or not at all. `SL-244`'s
`sec-3` records them as one gap with three consequences; this is the delivering
end of it. **Superseded reading:** an earlier version of this section conditioned
`Waived` on nothing being outstanding, with `withdraw` as the escape. `DEC-138`
reversed that — see below.

- **Cumulative reach.** `review-disposition-attested` is labelled cumulative
  while its `Artefact` coverage means only the disposition record's own content
  invalidates it. Binding it to the finding set is what makes the label true.
- **`Conducted`'s blocker predicate.** `DEC-138` satisfies the arm while the `RV`
  carries no finding that is both `blocker`-severity **and** in `open` or
  `contested` state. Note this is *not* `derived_status`'s `await` — that is
  severity-blind (`review.rs:1009-1022`) and its own doc calls it a display
  summary, never a gate. Nor is it D-C9b's `doc_unresolved_blockers`
  (`review.rs:1490`), which is the same filter minus the state restriction and so
  counts `answered`. The one-state difference is the whole point: the responder
  must always hold an act that clears.
- **`Conducted`'s admissibility — the one addition, not just an exposure.** The
  arm may only be claimed over an `RV` carrying a **concluded-pass marker**, and
  no such marker exists. Without it, `Conducted` is satisfied on entry to
  `reviewing`, because `DEC-138`'s predicate is universally quantified over the
  finding set and never reads `status`, so it is **vacuously true** on an empty
  ledger. (Corrected 2026-08-04, `RV-344` `F-8`: this used to be argued from *an
  empty ledger reads `(Done, None)`*. That reading is wrong — `ADR-007` D-C8
  fixes an empty ledger at `active`/`raiser` so no implementation mistakes
  no-findings-yet for completion, and `ISS-314` records the incumbent
  `derived_status` violating it. Fixing `ISS-314` leaves this hazard exactly
  where it is.) Findings cannot supply the signal — a clean pass is
  structurally identical to one never run — and the `## Synthesis` prose is
  refused because a gate parsing authored prose is `SL-244` `sec-2`'s
  fourth-prose-loader risk. So this item adds structured state saying a pass
  concluded, alongside the section reference it is already adding.

`DEC-138` also carries the arms themselves into a home: `SL-244` gives
`CheckpointAct` a fourth optional slot for `Conducted { review } | Waived
{ reason }`, which `DEC-125` fixed and no type held. That part is `SL-244`'s to
build, not this item's — noted so the two are not confused.

**The `Waived` arm asks nothing of this item.** It is unconditional under
`DEC-138`: a user act, findings left standing on the `RV`, reason in the change
log. So until this item lands the interim is the same on both arms, rather than
one strict path and one lax one.

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
