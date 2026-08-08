# ISS-322: Run-minted review pass cannot name an externally conducted RV

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

A managed design run **mints its own `RV`** on entry to `reviewing`
(`commands/design.rs::review_pass_plan`, applied at `run.rs:483-489`), and
replaces it with another freshly minted one on a later entry — `ReviewPass` is
documented *"minted on entry to `reviewing` and **replaced, never reopened**"*.

There is no wire, verb or declaration that binds an **existing** `RV` to the run
as its pass. `ReviewDisposition::Conducted { review }` is checked against the
run's current pass and refuses anything else:

```
Error: the `review-disposed` act does not correspond to its rule: it disposes
`RV-346`, which is not the pass this run is on (`RV-347`)
```

## Why that is a misalignment, not just a constraint

The `reviewing` obligation *itself* offers the external route. Its "After the
pass" step says to offer the user **"a formal hostile pass via `/inquisition` or
a printed prompt for an external adversarial reviewer"**. An external reviewer
prompted that way works on an `RV` somebody created with `review new` — which
the run can never point at. So the documented workflow produces exactly the
artefact the gate structurally cannot accept.

The run's own minted `RV` meanwhile sits empty and unused, and the honest
disposition collapses to `Waived` however thorough the real review was.

## Observed

SL-248, design run `dr-019fd432-6e5e-7282-b425-19f07630a627`, 2026-08-08.
`RV-346` — "SL-248 design — rolling per-section external pass" — ran **nine
rounds over every section, 38 findings, every one carried to a terminal
disposition**, and concluded. `RV-347`, the run-minted pass, has 0 findings and
0 rounds and was never used. `Conducted` naming `RV-346` is refused; `Conducted`
naming `RV-347` is refused at admission for want of the concluded marker.

**The trap in the remaining route.** `Conducted { RV-347 }` becomes admissible
the moment anyone runs `doctrine review conclude RV-347` — concluding a ledger
that reviewed nothing. That is precisely the laundering `conclude`'s own
rationale warns about: *"a clean pass and a pass never run present identical
findings, so nothing derivable can tell them apart"*. The verb exists to make
that distinction recordable, and this is the one path that turns it into a way
to erase it. A reader later sees a conducted pass with zero findings and cannot
tell it from a genuinely clean one.

Correcting a claim carried in SL-248's handover chain: the `Conducted` arm being
*reachable* (`IMP-392`'s blocking leg landed, so `Waived` is not the only live
match arm) does **not** make it usable for an externally conducted review.
Reachability of the arm and applicability to a given `RV` are different
questions, and only the first had been checked.

## Sketch of a fix

Cheapest first.

1. **Let the run adopt an existing `RV` as its pass.** A declaration — or a flag
   on entry to `reviewing` — naming an `RV` that already targets this slice, in
   place of minting. The mint stays the default for the ordinary case.
2. **Do not mint when the slice already has a live design-facet `RV`.** Narrower
   and needs no new wire, but it guesses, and guessing wrong is worse than the
   refusal.
3. **At minimum, say so in the refusal.** The current message names the two
   `RV`s and stops. It should say that the run's pass is minted rather than
   chosen, and name the routes actually available — which is what turns a dead
   end into a decision.

Independently worth considering: whether `review conclude` should refuse a
ledger with zero rounds, so option 3's trap cannot be walked into at all.

Related: `DEC-125` (the two disposition arms), `DEC-138` (admissibility),
`IMP-392`, `ISS-320` (the other unactionable-refusal defect on this surface).
