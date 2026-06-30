# Closure gate hardens on live Failed coverage cell; close the forget evidence-erasure leak

## Context

Origin: RSK-008 (likelihood medium, impact high). A live `Failed` coverage cell
is observed contradiction, yet a slice can reach `done` with that contradiction
unaddressed via **two leaks**:

1. **`coverage forget` (evidence-erasure).** Erasing the Failed cell's 4-tuple
   drops its requirement out of the closure-gate set (`covered ∪ declared ∪
   reconciled` — `src/slice.rs` `gate_requirement_set`) unless also
   `declared`/`reconciled`. The drift gate (`undischarged_drift`, `slice.rs:833`)
   then never sees the contradiction → silent `done`. This defeats SPEC-002 **D8**
   ("closed-with-unreconciled-drift is unrepresentable"): the override is meant to
   be a *recorded* REC, but `forget` is *unrecorded* erasure.

2. **`accept` REC over a Failed cell.** `drift` folds `Failed | Blocked` into one
   `Divergent(ObservedContradiction)` (`coverage.rs` `any_failed_or_blocked`), and
   `rec_discharges` (`slice.rs:1352`) treats it identically to
   `EvidenceOutrunsAuthored` — so an `accept` REC discharges a live failing test.
   Per SPEC-002 D8 + SL-044 D-B1/NF-001 this is **by design** (accept = "affirm
   authored status against evidence"; `--to` is independent human input). The risk
   is that accepting an *active contradiction* is qualitatively worse than
   accepting status-lag.

**Resolution principle (decided at preflight).** Split the two contradiction
sources, which `drift` currently lumps:

- **`Failed`** = a check *ran and contradicted*. No credible close-over case →
  **hard refuse**, not accept-dischargeable. Fix it (`coverage verify` re-derives
  Failed→Verified) or withdraw the req (Retire/Supersede already short-circuit to
  Coherent in `drift`).
- **`Blocked`** = evidence *unobtainable* (check can't run). PRD-013 names this a
  first-class failure mode; `coverage_verify` F-VII already keeps it distinct from
  Failed. Its designed resolution is a human VH attestation — i.e. a recorded
  override. Keep the override path, but **stricter** for contradiction than for
  lag (explicit, rationale-bearing). Honours NF-001 (human still decides).

This split also avoids a trap: `any_failed_or_blocked` short-circuits *before*
`any_fresh_verified`, so a req with a Blocked VT + a fresh Verified VH trips the
gate today. A blanket hard-refuse would force the operator to `forget` the
Blocked cell to close — the very erasure this slice kills.

## Scope & Objectives

1. **Close the `forget` leak.** A live `Failed`/`Blocked` cell must not be
   erasable in a way the closure gate can't see. Direction (design to settle the
   exact shape): refuse `coverage forget` of a Failed/Blocked cell, or admit it
   only through a recorded reconciliation path — never silent, never gate-evading.

2. **Hard-gate live `Failed` at closure.** On `reconcile → done`, a live `Failed`
   cell on any gate-set requirement refuses — and is **not** accept-dischargeable.
   Requires distinguishing `Failed` from `Blocked` at the discharge predicate
   (today `rec_discharges` is blind to the `DivergentReason`).

3. **Preserve a stricter override for `Blocked`.** A `Blocked` cell remains
   reconcilable via the recorded-override (accept-REC) path, but only with a fresh
   **human (VH)** `Verified` cell on the req and the REC citing both keys (design
   D3).

3a. **Withdrawal over a live contradiction is a recorded act (design D4).** Flipping
   a req to `Retired`/`Superseded` while it carries a live `Failed`/`Blocked` cell
   refuses close unless a slice-owned `revise`/`redesign` REC cites the evidence —
   closing the "withdraw to escape" leak the original remedy left open.

4. **Govern the D8 amendment via a REV.** SPEC-002 D8 currently treats all
   residual drift uniformly. Distinguishing `Failed` (un-acceptable) from
   `Blocked` (acceptable-strict) amends D8. The REV is **shaped after design
   locks** (anticipated PHASE-01) — design first, then route the governance edit
   through a Revision, then implement.

## Non-Goals

- No change to the `EvidenceOutrunsAuthored` / status-lag accept path — that
  discharge stays as-is (SL-044 D-B1).
- No reverse-index / perf rework of `gate_requirement_set` reverse scan
  (RSK-006 owns that).
- No new coverage status variants; no `coverage record`/`verify` semantics change
  beyond what the gate needs to read.
- Not reintroducing coverage→status auto-derivation (NF-001 stays intact — the
  hard refuse is a *gate*, not a status write).

## Affected surface (coarse — `/design` refines)

- `src/slice.rs` — closure gate (`undischarged_drift`, `rec_discharges`,
  `gate_requirement_set`, `run_status` seam).
- `src/coverage.rs` — `drift` / `DivergentReason` / `Composite` predicates the
  gate reads (the Failed-vs-Blocked distinction lives here).
- `src/coverage_store.rs` — `forget` / `run_forget` (the erasure leak).
- `src/coverage_verify.rs` — Failed/Blocked derivation (F-VII) for consistency.
- SPEC-002 `spec-002.md` D8 — amended via REV (governance surface).

## Risks / Assumptions / Open Questions

- **OQ-1 — forget fix shape. RESOLVED (design D2):** refuse outright on
  Failed/Blocked; wrong-key garbage remedied by a reviewed hand-edit (git-auditable).
  No `--force` (it would re-open the leak).
- **OQ-2 — Blocked override strictness. RESOLVED (design D3):** an accept-REC
  discharges `ObservedBlocked` only if the req also carries a fresh `Verified` cell.
- **OQ-3 — distinction site. RESOLVED (design D1):** split `DivergentReason` into
  `ObservedFailure` + `ObservedBlocked` (one named source the gate reads), not a
  parallel composite predicate.
- **Assumption:** behaviour-preservation gate applies — existing coverage/close
  suites must stay green where behaviour is unchanged (the lag-accept path).
- **Assumption:** "live" excludes withdrawn-status reqs (Retire/Supersede already
  short-circuit `drift` to Coherent) — those need no special-casing.

## Verification / Closure intent

- A live `Failed` cell on a gate-set req refuses `reconcile → done`, and **no**
  `accept` REC discharges it (regression test against the current accept path).
- `coverage forget` of a Failed/Blocked cell cannot silently clear the gate
  (refused, or recorded such that the gate still sees it).
- A `Blocked` cell remains closeable via the stricter recorded-override path.
- The status-lag (`EvidenceOutrunsAuthored`) accept path is unchanged (existing
  SL-044 suites green).
- SPEC-002 D8 amended via a `done` REV; the slice's own closure gate (the very
  machinery it edits) passes for SL-179.

## Follow-Ups

- **RSK-012** — closure gate-set scope is per-slice; a foreign Failed req can be
  omitted by not declaring it (deferred from the SL-179 codex pass; no silent leak,
  broader gate-set-breadth concern).
- **RSK-013** — `scan_coverage` silently skips malformed/unreadable `coverage.toml`;
  closure needs a strict fail-closed scan mode (deferred from the codex pass).
- Further durable findings captured at `/notes` / close.
