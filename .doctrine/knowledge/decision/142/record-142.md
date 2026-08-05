# DEC-142: VA-1 clause 2 reads structurally

## The question

SL-244 PHASE-06's `VA-1` asks three things of the nine contract narratives: each
says what the const does not, **restates nothing the renderer injects**, and
cites no repo-private id. Clause 2 has two available readings, and they disagree
on four of the nine assets.

The renderer's injection is exactly `contract <token> <kind>
[<coverage>|engine(<source>)] [observes(…)] <reach>` plus one `discharge:` line
(`src/design_run/prompt.rs:210-232`). A sweep with a positive control finds **no
asset containing any injected token**. But four assets discuss a *property* the
injection names — most sharply `section-attestations-current`, whose "editing
after a review costs that review" is the operational consequence of the currency
its condition name carries.

## The decision

Clause 2 reads as **states no structural fact the renderer states** — the
injected fields and the rendered discharge — not *touches no injected property*.
All nine narratives pass. No prose change is owed.

## Why

The injection exists to prevent one failure, named in `contract_block`'s own
doc: every structural field is injected "so the narrative can never restate them
and contradict them." Only a structural restatement can contradict. Prose about
what `per-section` *costs you* cannot drift from `per-section`; prose spelling
`per-section` in words can.

The strict reading also collides with `EX-3`'s other half — the asset carries the
fuller why — and the collision is not symmetric. Forbid an asset from naming the
consequences of its own binding and the coverage-heavy rows have nothing left
worth reading, which defeats the corpus PHASE-06 exists to ship.

## Boundary

This is a reading of clause 2 only. Clause 1 (says what the const does not) and
clause 3 (cites no repo-private id, per DEC-127; ISS-309 owns the pre-existing
corpus violations) are untouched, and a narrative that spelled `cumulative` or
`observes(...)` in prose would still fail.

Related: [[DEC-141]] — the other PHASE-06 ruling taken in the same consult.
