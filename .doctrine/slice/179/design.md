# SL-179 — Closure gate hardens on live Failed coverage cell; close the forget evidence-erasure leak

Origin: RSK-008. Governs SPEC-002 (`concerns`), `governed_by` ADR-009.

## 1. Problem

A live `Failed` coverage cell is observed contradiction, yet a slice can reach
`done` with it unaddressed via two leaks:

1. **`coverage forget` (evidence-erasure).** Erasing the cell's 4-tuple drops its
   requirement out of the closure-gate set (`covered ∪ declared ∪ reconciled`,
   `slice.rs gate_requirement_set`) unless also declared/reconciled — so the drift
   gate (`undischarged_drift`) never sees the contradiction → silent `done`. Defeats
   SPEC-002 **D8** ("closed-with-unreconciled-drift is unrepresentable"): the
   override is meant to be a *recorded* REC; `forget` is *unrecorded* erasure.
2. **`accept` REC over a Failed cell.** `drift` folds `Failed | Blocked` into one
   `Divergent(ObservedContradiction)`; `rec_discharges` treats it identically to
   status-lag, so an `accept` REC discharges a live failing test.

## 2. Resolution principle (locked at preflight/design)

Split the two contradiction sources `drift` currently lumps:

- **`Failed`** = a check *ran and contradicted*. No credible close-over case →
  **hard refuse**, not accept-dischargeable. Remedy: fix it (`coverage verify`
  re-derives Failed→Verified) or withdraw the req (Retire/Supersede already
  short-circuit `drift`→Coherent).
- **`Blocked`** = evidence *unobtainable*. PRD-013 first-class failure mode;
  `coverage_verify` F-VII already keeps it distinct from Failed. Acceptable via a
  **stricter** recorded override: an accept-REC discharges it *only if the req also
  carries a fresh `Verified` cell* (the manual confirmation, recorded as VH/VA).
- **`EvidenceOutrunsAuthored`** (status-lag) → **unchanged** (SL-044 D-B1).

This split also avoids a trap: `any_failed_or_blocked` short-circuits *before*
`any_fresh_verified`, so a Blocked-VT + Verified-VH req trips the gate today. A
blanket hard-refuse would force the operator to `forget` the Blocked cell to close
— the very erasure this slice kills.

**NF-001 holds.** Every gate decision below is a *refuse-the-transition* read of
coverage, never a write of authored `ReqStatus` from coverage. The human's `--to`
at reconcile stays the sole status writer (info-flow wall at the function
signature — SL-044 §5.6, `mem_019eb9a8…`). `slice → coverage` is the established
ADR-001 downward edge (`slice.rs:1229`); no new cross-module coupling.

### Decisions

- **D1 (Q1=A) — the Failed/Blocked distinction lives in `DivergentReason`.** Split
  `ObservedContradiction` into `ObservedFailure` + `ObservedBlocked` — one named
  source the gate already reads, no risk of a parallel predicate disagreeing with
  the verdict reason. *Alt rejected:* composite-only predicates (two notions of
  "what kind of contradiction" that can drift apart); gate-local threading (messier
  hybrid).
- **D2 (Q2=A) — `forget` refuses outright on `Failed`/`Blocked`.** The CLI offers
  *zero* silent-erasure path. Legit status changes are not `forget` (VT → `coverage
  verify` re-derives; VA/VH → `coverage record` upserts same key). The only
  legit forget-of-contradiction is wrong-key garbage (authoring error); its remedy
  is a reviewed, committed hand-edit of the authored `coverage.toml` — git-visible,
  durable, peer-reviewable. *Alt rejected:* `--force` escape (re-opens the exact
  leak under a new name; terminal loudness is not durable evidence).
- **D3 (Q3=B) — `ObservedBlocked` discharges only with a fresh `Verified` cell.**
  Machine-checkable (`composite.any_fresh_verified()`), reuses existing
  coverage-cell machinery, honours PRD-013 (blocked never defaults to verified) and
  NF-001 (human attests via a recorded cell). *Alt rejected:* status-quo discharge
  (Blocked as cheap as lag); rationale-prose/flag (theatre, not machine-checkable).

## 3. Current vs target behaviour

| Scenario (req in gate set) | Current | Target |
|---|---|---|
| live `Failed` cell, no REC | refuse (Divergent) | **refuse** (ObservedFailure) |
| live `Failed` cell + accept-REC (3 clauses) | **discharges → closes** | **refuse** — not dischargeable |
| live `Blocked` cell + accept-REC, no Verified | discharges → closes | **refuse** — needs confirming Verified |
| live `Blocked` + fresh `Verified` + accept-REC | discharges → closes | **discharges → closes** |
| status-lag (`EvidenceOutrunsAuthored`) + accept-REC | discharges | discharges (**unchanged**) |
| `coverage forget` a `Failed`/`Blocked` cell | erases silently → gate blind | **refused** |
| `coverage forget` a `Planned`/`Verified` cell | erases | erases (**unchanged**) |

## 4. Code impact

### 4.1 `coverage.rs` — verdict model (§1 foundation)
- `DivergentReason`: drop `ObservedContradiction`; add `ObservedFailure`,
  `ObservedBlocked`.
- `Composite`: add `any_failed()` / `any_blocked()`; keep `any_failed_or_blocked()`
  as `any_failed() || any_blocked()` (view summary convenience).
- `drift`: replace the single `any_failed_or_blocked` arm with ordered precedence —
  `any_failed() ⇒ ObservedFailure` then `any_blocked() ⇒ ObservedBlocked` (Failed
  outranks Blocked).
- `DivergentReason::label`: `"observed-failure"` / `"observed-blocked"`.
- Tests (`coverage.rs:861-905,966`): update expected reasons.

### 4.2 `reconcile.rs` — prompt register
- `divergent_label` (`:106`): two arms — failure ("a check ran and contradicted")
  vs blocked ("evidence unobtainable; confirm with a Verified attestation or
  withdraw"). Test `:637` updates.

### 4.3 `coverage_view.rs` — read view
- `:111` / `:393`: render the two new reasons (keep combined health summary via
  `any_failed_or_blocked` if the column is coarse; sharpen the verdict cell).

### 4.4 `slice.rs` — closure gate (§2)
- `undischarged_drift` (`:1287`): branch on the verdict reason —
  - `ObservedFailure` → always undischarged (push, never call `rec_discharges`).
  - `ObservedBlocked` → undischarged unless `composite.any_fresh_verified()` **and**
    `rec_discharges(...)`.
  - `EvidenceOutrunsAuthored` / `Indeterminate` → unchanged (`rec_discharges`).
- `rec_discharges` (`:1352`): signature gains the verdict (or reason) + `&Composite`
  it needs for the Blocked branch. Returns bool (a refuse decision — no status
  write; NF-001).
- `UndischargedReq` (`:1320`): carry the `DivergentReason` so the bail copy is
  per-req accurate.
- bail copy (`:841-849`): three registers —
  - failure: "REQ-X has a Failed coverage cell — re-derive it (VT: `coverage verify
    SL-N`) or re-attest it (VA/VH: `coverage record`), or withdraw the requirement;
    a Failed cell is not accept-dischargeable." (Remedy is mode-aware: VT cells
    re-derive via `verify`; VA/VH cells are overwritten via `record` same-key — F4.)
  - blocked-no-verified: "REQ-X is Blocked with no confirming evidence — record a
    VH/VA Verified attestation, then accept-REC; or withdraw."
  - blocked-with-verified / lag: existing accept-REC recipe (SL-178 legibility
    preserved).

### 4.5 `coverage_store.rs` — forget guard (§3)
- `forget` (`:160`): guard atomically (removal-then-return can't be post-checked).
  Return an outcome enum:
  ```rust
  enum ForgetOutcome { Erased(CoverageKey, CoverageStatus), Refused(CoverageStatus), NotFound }
  ```
  `Failed`/`Blocked` ⇒ `Refused` (no remove, no save). Erasable predicate named once
  (a pure helper; status compared via the `CoverageStatus` enum — STD-001, no string
  literals).
- `run_forget` (`:366`): match — `Refused` bails naming the cell + remedies;
  `Erased`/`NotFound` as today.
- test `forget_removes_the_keyed_cell…` (`:765`): retarget to a Planned/Verified
  cell; add refusal tests.

## 5. Governance — the D8 REV (PHASE-01)

SPEC-002 **D8** today: closure gate default-refuses residual drift, *with a recorded
override (a REC recording accepted residual drift)*. The amendment **narrows what is
acceptable residual drift**:
- a `Failed` cell is **not** acceptable — it must be fixed (cell → Verified) or the
  requirement withdrawn;
- a `Blocked` cell is acceptable **only** when the requirement also carries fresh
  confirming `Verified` evidence.

Candidate touch: **REQ-113** (gate refuses undischarged residual drift) may gain a
clause or companion requirement on Failed un-acceptability. The REV is **shaped
after this design locks** (PHASE-01); spec authorizes the code, so governance lands
first. Routed per ADR-013 (governance edit → Revision).

## 6. Phasing (shape — `/plan` sets criteria)

- **PHASE-01 — governance:** author + approve + apply the D8/REQ-113 REV.
- **PHASE-02 — verdict model:** §4.1–4.3 (`coverage.rs`, `reconcile.rs`,
  `coverage_view.rs`). Behaviour-preserving except the named reason split.
- **PHASE-03 — closure gate:** §4.4 (`slice.rs`).
- **PHASE-04 — forget guard:** §4.5 (`coverage_store.rs`). File-disjoint from
  PHASE-03; /plan may parallelize or merge.

## 7. Verification / closure intent

- VT: live `Failed` on a gate-set req refuses `reconcile→done`; **no** accept-REC
  discharges it (regression vs the current accept path).
- VT: `Blocked` + fresh `Verified` + accept-REC → closes; `Blocked` without
  `Verified` → refuses.
- VT: status-lag accept path unchanged. The existing discharge tests (VT-4
  `vt4_matching_accept_rec_discharges_the_drift`, VT-5) exercise
  `EvidenceOutrunsAuthored` (a `Verified` cell on a `Pending` req), **not** a
  contradiction — they stay green untouched. No existing test discharges a
  `Failed`/`Blocked` cell, so the hard-refuse *adds* tests rather than flipping
  them (behaviour-preservation proof; adversarial F2).
- VT: `coverage forget` of `Failed`/`Blocked` refused; of `Planned`/`Verified`
  still works.
- VT: `drift`/label/prompt goldens updated for the two new reasons.
- VA/dogfood: SL-179's own close passes the hardened gate.

## 8. Risks / open questions

- **R1 — golden churn.** The reason split touches every drift/label/prompt golden.
  Bounded, expected (behaviour-preservation gate); not a correctness risk.
- **R2 — Blocked-without-Verified deadlock.** A genuinely-unobtainable req with no
  way to confirm cannot close — *by design* (withdraw the req or escalate
  `reconcile→design`). Documented in the bail copy.
- **R3 — wrong-key garbage Failed cell.** Removable only by reviewed hand-edit (D2).
  Accepted: rare authoring error, git-auditable remedy.
- **R4 — stale Failed/Blocked treated as live (fail-closed).** `any_failed()` /
  `any_blocked()` ignore staleness (as `any_failed_or_blocked` does today). A stale
  Failed VT cell refuses until re-derived — the operator runs `coverage verify`,
  which either re-confirms or flips it. Conservative and behaviour-preserving
  (adversarial F5).
- **OQ (carried to /plan):** whether PHASE-03 and PHASE-04 merge (file-disjoint,
  both small).

## 9. Adversarial review (internal pass — integrated)

Hostile pass against the code, findings folded above:
- **F1 — `forget` blast radius contained.** Only `run_forget` + two unit tests call
  `forget`; the `ForgetOutcome` enum change is bounded.
- **F2 — behaviour-preservation claim verified** (see §7): existing discharge tests
  are lag-based; none discharge a contradiction cell.
- **F3 — NF-001 intact.** Reason-consumers in `reconcile.rs` are display-only
  (`divergent_label`/`build_prompt`); `select_status` takes no `Verdict`. The gate
  reading `Composite`/`Verdict` for a *refuse* decision is not a status derivation —
  it returns `bool`, never writes `ReqStatus`. `slice → coverage` is the established
  ADR-001 downward edge.
- **F4 — bail copy made mode-aware** (§4.4): VT re-derive vs VA/VH re-attest.
- **F5 — stale-cell fail-closed documented** (R4).
- **Trust note — `any_fresh_verified` for Blocked discharge.** A falsely-recorded VH
  Verified cell could satisfy the bar — but that is the same human-attestation trust
  model as every VH cell (attributable, git-anchored, dated). Not a new hole; the
  bar is "honest recorded confirmation," consistent with NF-001. Predicate is
  mode-agnostic (any fresh `Verified`); the canonical case is a VH/VA manual
  attestation.
