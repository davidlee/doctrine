# REQ-316: Candidate admission by immutable OID

## Statement

`candidate/<N>/<label>` branches are the safe interaction surface over immutable
evidence. Admission is by immutable OID, never by ref: `candidate create` performs a
Doctrine no-ff 3-way merge of `--source` into `--base`, recording immutable
`source_oid`, `base_oid`, and `merge_oid` (parents exactly `base_oid` + `source_oid`);
`admit` validates the tip descends from `merge_oid` and pins `admitted_oid`;
`integrate --trunk` targets `admitted_oid`, **never the live candidate tip**. If the
candidate ref drifts after admission, status reports it but the admitted OID is
unchanged. If trunk moved, integrate refuses with guidance to create a superseding
candidate — there is no close-time merge.

**Source provenance.** `candidate create` refuses a `--source` that is not a `Verified`
stage-1 prepare-review journal row — a candidate may only be built from verified
evidence — **with one exception for close-target repair propagation (REQ-317):** a
`close_target` create may source a recorded `candidate/<N>/<label>` row, provided that
row has `kind = audit` and `role ∈ {review_surface, close_target}` (`scratch` and
`experiment` sources are refused) and `status = Created` (a clean-merge row; a
hand-resolved `Conflicted` row is refused — a documented v1 limitation). The source
`target_ref` must resolve to **exactly one** recorded candidate row — a count-exact,
fail-closed match (duplicates refused). The candidate chain is then traced by bounded
recursion (depth ≤ 16); each hop applies the same row gate, and the chain must terminate
at a `Verified` journaled-evidence root (`review/<N>` or `phase/<N>-NN`) validated through
the *full* journaled gate — including the phase-hole refusal below when that root is a
`phase/<N>-NN` ref. The exception binds by lineage, not name: the resolved `source_oid`
must descend from the source row's recorded `merge_oid`, so a moved ref cannot launder
unrelated history (INV-6).

A `phase/<N>-NN` (code) close-target — whether named directly as the source or reached as
the traced chain root — additionally refuses when an **earlier** non-empty phase row
failed (an unresolved hole below the selected phase).

**Branch condition.** Whether `integrate --trunk` takes the candidate path is gated on
`candidate_active = the slice has ≥1 candidate row`. When candidate rows exist, integrate
**requires** a current `close_target` admission and does **not** fall back to raw evidence
or the phase chain. When no candidate workflow is active, the legacy path sources the
phase-chain tip from the journal (the highest `phase/<N>-NN`), **never `review/<N>`** — no
code path integrates trunk from the impl bundle.

## Rationale

Pinning "what lands on trunk" to an immutable OID makes the integrated artifact an
explicit, reviewable choice rather than whatever a mutable branch happens to point at
when close runs. The provenance gate keeps the chain honest — no candidate from
unverified evidence, no close-target straddling a failed phase. It also lets audit and
repair happen freely on the candidate without risking the evidence refs (SL-068).
