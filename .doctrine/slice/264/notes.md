# Notes SL-264: Inquiry-map growth without re-attestation

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Triage (2026-09-25)

Recorded to discharge the `explore.triage` runbook step. Detail lives in
`research/research.md` (gitignored) and the scope's own risks section — this is
the index, not a copy.

### Open questions

- **OQ-1 — the fix family.** Narrow the two attested rows' binding, or mint a
  derived "added since acceptance" condition? `R4` makes narrowing alone unsafe.
  *Load-bearing.*
- **OQ-2 — two staleness causes, not one.** `initial-concerns-recorded` goes
  stale via `CoverageStale` (coverage) and `ConfirmationStale` (the `confirms`
  link); only the first is a coverage question.
- **OQ-3 — does the human act survive?** Whether a shape change voids the
  sufficiency acceptance. `RFC-031`'s recorded position (2026-09-25): it
  survives.
- **OQ-4 — blocking, or warning?** The strength of the "map moved ⇒ re-declare"
  obligation. `DEC-101` supplies the warning precedent, `DEC-120` the case
  against silence.
- **OQ-5 — the no-op's rows.** Does an edge-free `needs: null` emit nothing?
  `REQ-478` does not settle whether a no-op is a "recorded mutation".
- **OQ-6 — stored snapshots.** Does the narrowed reading apply uniformly over
  the seventeen live runs?

### Risks

- **R1** loosening invalidation is a truthfulness change — the `RFC-031` T1 class
  inverted: a condition that should have invalidated and did not is a silent lie.
- **R2** seventeen live snapshots change semantics underneath; `DEC-059` prefers a
  reading-side change.
- **R4** the derived guard is coupled to the attested rows' staleness. The crux.

### Assumptions carried

- **A1** growth is permitted, unrefused and exercised; the disincentive is cost.
- **A2** `ISS-481` is wiring only — `apply_collection` already clears on `Null`
  (`submission.rs:66-71`), and `SPEC-029` Responsibilities[13] plus `DEC-063`
  already require it.

### Shaping decisions (so far)

- The map's growth obligation belongs in the `inquiry.md` lens, not the
  `inquiring` runbook (`DEC-104`) — landed as `IMP-470`.
- `SL-264` carries `IMP-469` + `ISS-481`; `IMP-386` (backward cascade) and
  `ISS-450` (the create branch) are fenced out deliberately.
- The `DEC-062` reconciliation is a new or superseding `DEC`: `revises` targets
  `&[SPEC, PRD, REQ, ADR, POL, STD]` and cannot reach a `DEC`.

### Constraining governance

See `research/research.md` § Thread 1. Load-bearing: `SPEC-029`
Responsibilities[13] (already requires the sparse fix), `DEC-062` (the conflict),
`DEC-063`, `DEC-120` (the principle this slice creates an exception to),
`DEC-101` (stale-warns precedent), `DEC-121`, `DEC-126`, `DEC-067`; `ADR-001`;
`ADR-019` + `DEC-127` (the shipped stage table is generated); `STD-001`.
Verified: `REQ-427` does **not** reach gate attestations, so no REV is owed.

## Review passes

Written after the first pass (2026-09-25, revision 24) to discharge `review.passes`.
A further pass, if one is wanted, would probe:

- **The `DEC-300` refinement.** The decision says *derived*; the design implements the
  growth obligation as an **attested-by-agent** condition (`blocking-set-current`),
  because the act already carries coverage and `DEC-126`'s ledger prefers attested
  where an act exists. It avoids a new `EngineSource` and a new derivation rule. The
  substance is unchanged, but this is a deviation from a recorded decision's wording
  and is the single most arguable choice in the design.
- **The comparison's API shape.** Whether the joiners-blind comparison belongs as a new
  method beside `ContentCoverage::diff` or as a parameter of it — the design says "a new
  method" without arguing against the alternative.
- **`submission.rs`'s involvement.** `sec-5` hedges ("where one is owed"). A second pass
  should establish whether the new row needs a key-home/state entry at all.
- **The new row's reach.** `Cumulative` means it re-derives at every edge above
  `inquiring→drafting`. That is what keeps the guard alive, but it also means a map
  addition during `drafting` blocks the next edge. Deliberate, worth a second look.
- **Test placement.** Whether the flipped pin in `src/design_run/tests.rs` plus new unit
  rows is sufficient, or an e2e is owed (no e2e file carries the gate table today).
- **`IDE-057`.** Whether a traversal-only apply owing no change row interacts with
  `REQ-478`'s "every mutation the run records" once `blocking-set-current` makes
  traversal-shaped state load-bearing.

A second pass would not re-probe: the mechanism mapping in `research/research.md`
(`material()`, `materials()`, the digest's binding, the two staleness causes) — all
verified against source during design.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-25 · reviewing · 3e5b85e00

### Produced
- RV-385 — re-homed RV-386's seven live findings (F-1..F-7) onto the run's pass; commits f491f490e, 3e5b85e00
- 01a0d83e — friction record: SL-264 as an ISS-322 recurrence after ISS-476

### Learned
- mem_01a0d17f827772b096e836f95a2887c4 — pass_stale is a lamp, not a gate; raise on the run's pass RV, never a second

### Open
- ISS-322 — a run-minted pass cannot bind an externally conducted RV
