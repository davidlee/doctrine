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

## Rationale

Pinning "what lands on trunk" to an immutable OID makes the integrated artifact an
explicit, reviewable choice rather than whatever a mutable branch happens to point at
when close runs. It also lets audit and repair happen freely on the candidate without
risking the evidence refs (SL-068).

**Branch condition.** Whether `integrate --trunk` takes the candidate path is gated on
`candidate_active = the slice has ≥1 candidate row`. When candidate rows exist, integrate
**requires** a current `close_target` admission and does **not** fall back to raw evidence
or the phase chain. When no candidate workflow is active, the legacy path sources the
phase-chain tip from the journal (the highest `phase/<N>-NN`), **never `review/<N>`** — no
code path integrates trunk from the impl bundle.
