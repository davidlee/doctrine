# Review RV-146 — reconciliation of SL-145

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Mode:** conformance self-audit. **Surface reviewed:** the authored source on
`edge` (commit `73c56a8` + the VT-4 fixture in `relation_graph.rs`) — not a
dispatched candidate; SL-145 was a solo single-phase slice.

**What this audit probes — does the shipped change match design D1/D2/D3 and the
plan's EX-1..EX-7 / VT-1/VT-2/VH-1, and only that:**

1. **Scope discipline** — did the slice stay a *table-only* widening? No new
   label, no role grammar, no consumer/graph-effect reaction (D3), no migration.
   Any code touched beyond the two `sources` sets + their test goldens is drift.
2. **The two widenings (EX-1/EX-2)** — `governed_by` sources gained BACKLOG with
   `Kinds(GOV)` intact; `related` widened by *extending the existing `[SL,RFC]`
   row* (D1), not a new row, target still `AnyNumbered`.
3. **Monotonicity / behaviour-preservation (EX-6)** — the only golden churn is
   the intended refusal flips; the rest of the relation/relation_graph/integrity/
   inspect suite is green unchanged. `just gate` clean (EX-7).
4. **Goldens tell the truth (EX-3/EX-4/EX-5)** — read_block emits the backlog
   edges + keeps `requirements` illegal; VT-2 `Related` set + its doc-comment
   widened; positive `validate_link` + a backlog target-gate negative present;
   `lookup` flipped to `is_some`.
5. **VH-1 end-to-end** — the CLI loop (link → TOML row → inspect outbound +
   derived inbound → unlink) was exercised on a fresh dev binary (recorded in
   `73c56a8`); confirm no authored edges leaked onto real entities.
6. **Doc honesty** — design §10 flagged a stale SL-095 test comment
   (`relation.rs:1717`, "new BACKLOG/SLICE row") as an optional non-blocking
   tidy; check whether it now lies post-D1.

Invariants held against: the four in the domain_map (monotonic widening,
unchanged target gates, kind-generic seams, single-row D1).

## Synthesis

**Closure story.** SL-145 is a textbook minimal slice: design D1/D2/D3 reduced
RFC-003 Axis A to a single legal-set widening, and the implementation is exactly
that — BACKLOG (ISS/IMP/CHR/RSK/IDE) added to the `sources` of `governed_by` and
of the existing `related` `[SL,RFC]` row in `RELATION_RULES`, plus the goldens
that encoded the old refusal. Every plan exit criterion (EX-1..EX-7) and
verification (VT-1/VT-2/VH-1) is satisfied; conformance is 1:1 with design. All
six findings landed `aligned` save F-6 (a cosmetic doc nit), and none was a
blocker.

The behaviour-preservation gate held: `just gate` exit 0, churn confined to the
intended refusal flips. The monotonicity invariant — widening `sources` only
grows the legal pair set — is proven by the untouched suite staying green. The
target gates (`Kinds(GOV)` for governed_by, `AnyNumbered` for related) are intact
and explicitly re-asserted by a new backlog-source negative test. The read/write
seams stayed kind-generic (no backlog accessor, no command allowlist), so R1 (a
hidden consumer assuming backlog never carries these labels) is disproven by the
green suite, not just argued.

**Standing risks.** None material. The slice deliberately ships permit-only (D3):
no consumer reacts to the new edges (priority overlay, `/close`, transitive walks
are untouched). That is the intended Layer-1 boundary, not a gap. F-5 (review
outlet) remains open against RFC-003 Axis B by design (D2), not deferred drift
from this slice.

**Tradeoffs consciously accepted.** F-6 fixed in-audit: the stale SL-095 test
comment at `relation.rs:1717` was tidied to name both SL-095 and SL-145
accurately (fix-now, in audit scope, `just check` green). No design or governance
artifact tells an untruth post-implementation — nothing for `/reconcile` to write.

## Reconciliation Brief

### Per-slice (direct edit)
- **None.** design.md / plan.toml / slice-145.md already match the shipped
  implementation 1:1 (conformance findings F-1..F-5 all `aligned`). The lone code
  nit (F-6) was fixed in-audit, not deferred.

### Governance/spec (REV)
- **None.** No finding touches an ADR, POL, STD, spec, or requirement. ADR-004 /
  ADR-010 / SPEC-018 were the conformance oracles and the slice respected all
  three without amendment.

`/reconcile` has no write surface to action — the brief is empty by construction.
Hand straight through to status `reconcile`, then `/reconcile` confirms the
no-op and routes to `/close`.

## Reconciliation Outcome

All six findings (F-1..F-6) were `verified` — no dispositions required amendment.
No writes needed on either surface:

### Direct edits applied
- **None.** per-slice artefacts (design.md, plan.toml, slice-145.md) match the
  shipped implementation 1:1; conformance findings F-1..F-5 all `aligned`.

### REVs completed
- **None.** No finding touched an ADR, POL, STD, spec, or requirement. No
  governance/spec artefact needed amendment.

### Fixed in-audit
- RV-146 F-6 (`fix-now`): stale SL-095 test comment at `relation.rs:1717` tidied
  in-place during audit; `just check` exit 0.

Reconcile pass complete — handoff to /close.
