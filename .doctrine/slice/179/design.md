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
- **D3 (Q3=B, tightened post-review) — `ObservedBlocked` discharges only with a
  fresh human (`VH`) `Verified` cell, and the REC cites both keys.** The bar is a
  fresh **VH** Verified cell on the req (NOT VT or VA — only a human sign-off is the
  accountable "the unobtainable thing actually works"; VT is the blocked check
  itself, VA is an agent, neither is the manual confirmation). The accept-REC's
  `evidence_ref` must cite **both** the Blocked key and the confirming VH key
  (self-documenting; codex M5). Machine-checkable, honours PRD-013 (blocked never
  defaults to verified) and NF-001 (human attests via a recorded cell). *Alt
  rejected:* mode-agnostic `any_fresh_verified` (a foreign VT/VA cell for a
  different contribution could satisfy it without being the manual confirmation);
  status-quo discharge; rationale-prose/flag (theatre, not machine-checkable).
- **D4 (codex B3) — withdrawal over a live contradiction requires a recorded act.**
  `drift` short-circuits `Retired`/`Superseded` to `Coherent` *before* inspecting
  coverage (`coverage.rs:287`), and requirement status is free any-to-any (D-B6) —
  so flipping a req to `Retired` lets a slice close over its live `Failed`/`Blocked`
  cell, then reactivate the req later, with no evidence-citing act. The **closure
  gate** (not `drift` itself — the read-view semantics stay) must treat a withdrawn
  gate-set req that *still carries a live `Failed`/`Blocked` cell* as undischarged
  **unless** a slice-owned `revise`/`redesign` REC cites those evidence keys. So
  withdrawal-as-resolution becomes a recorded reconciliation act, not a silent
  status flip. *Alt rejected:* trusting the bare status flip (the exact
  evidence-erasure this slice kills, by another verb).

## 3. Current vs target behaviour

| Scenario (req in gate set) | Current | Target |
|---|---|---|
| live `Failed` cell, no REC | refuse (Divergent) | **refuse** (ObservedFailure) |
| live `Failed` cell + accept-REC (3 clauses) | **discharges → closes** | **refuse** — not dischargeable |
| live `Blocked` cell + accept-REC, no VH Verified | discharges → closes | **refuse** — needs confirming VH cell |
| live `Blocked` + fresh `VH` Verified + accept-REC citing both keys | discharges → closes | **discharges → closes** |
| live `Failed`/`Blocked` cell + req flipped to `Retired`/`Superseded`, no REC | discharges (drift→Coherent) | **refuse** — needs a recorded withdrawal REC (D4) |
| status-lag (`EvidenceOutrunsAuthored`) + accept-REC | discharges | discharges (**unchanged**) |
| `Indeterminate` + accept-REC with empty residual keys | discharges (vacuous clause-c) | **refuse** — empty-evidence accept forbidden (M7) |
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
- **Reconcile note (RV-198 F-1):** no edit landed — both clauses satisfied unedited.
  `:393` `Verdict::Divergent(r) => Some(r.label())` already auto-renders the new
  `observed-failure`/`observed-blocked` labels, and `:111` `observed_state()` keeps
  the combined `any_failed_or_blocked` health summary. The "undelivered" conformance
  selector was a stale design over-declaration absorbed by existing delegation.

### 4.4 `slice.rs` — closure gate (§2)
**Control flow pinned (codex M10): `undischarged_drift` classifies once per req and
decides; `rec_discharges` stays the unchanged 3-clause REC predicate.** Per gate-set
req, compute `verdict = drift(authored, &composite)` then:
  - `Divergent(ObservedFailure)` → **always undischarged** (push; never call
    `rec_discharges`).
  - `Divergent(ObservedBlocked)` → undischarged unless `composite.has_fresh_vh()`
    **and** `rec_discharges(...)` **and** the REC cites the confirming VH key (D3).
  - `Divergent(EvidenceOutrunsAuthored)` → unchanged `rec_discharges`, **plus** the
    M7 guard.
  - `Indeterminate` → unchanged `rec_discharges`, **plus** the M7 guard: if
    `residual_keys` is empty, accept cannot discharge (clause (c) is otherwise
    vacuously true, `slice.rs:1373`) → undischarged.
  - `Coherent` → normally skip, **except the D4 withdrawal check**: if `authored ∈
    {Retired, Superseded}` AND `composite.any_failed() || composite.any_blocked()`
    (a live contradiction the withdrawal short-circuited), require a slice-owned
    `revise`/`redesign` REC citing those keys; absent → undischarged.
- `rec_discharges` (`:1352`): **signature unchanged** — it remains the 3-clause REC
  predicate. The reason-branching, the VH bar, the M7 empty-keys guard, and the D4
  withdrawal check all live in `undischarged_drift`. (Keeps the bool predicate pure
  and the policy in the gate; NF-001 — a refuse decision, never a status write.)
- New `Composite` helper: `has_fresh_vh()` (a fresh `Verified` cell with `mode ==
  VH`) — distinct from the mode-agnostic `any_fresh_verified()` (D3).
- `UndischargedReq` (`:1320`): carry the `DivergentReason` (+ a withdrawal-marker
  variant) so the bail copy is per-req accurate.
- bail copy (`:841-849`): three registers —
  - failure: "REQ-X has a Failed coverage cell — re-derive it (VT: `coverage verify
    SL-N`) or re-attest it (VA/VH: `coverage record`), or withdraw the requirement;
    a Failed cell is not accept-dischargeable." (Remedy is mode-aware: VT cells
    re-derive via `verify`; VA/VH cells are overwritten via `record` same-key — F4.)
  - blocked-no-VH: "REQ-X is Blocked with no human confirmation — record a VH
    Verified attestation that it works, then an accept-REC citing both the blocked
    and confirming keys; or use the withdrawal path."
  - withdrawal-without-REC (D4): "REQ-X is withdrawn but still carries a live
    Failed/Blocked cell — record a slice-owned revise/redesign REC citing the
    evidence keys; a bare status flip cannot retire a live contradiction."
  - blocked-with-VH / lag: existing accept-REC recipe (SL-178 legibility
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
  requirement withdrawn via a recorded act;
- a `Blocked` cell is acceptable **only** when the requirement also carries fresh
  confirming **human (VH)** `Verified` evidence cited by the REC;
- withdrawing a requirement that still carries a live `Failed`/`Blocked` cell is
  itself a reconciliation act — a recorded `revise`/`redesign` REC citing the
  evidence, not a bare status flip (D4);
- the closure gate is the **`done`** path; `abandoned` is a distinct
  giving-up terminal, explicitly **not** gated on coverage (codex M6 — clarify the
  "terminal status" wording so it does not imply abandon-gating).

Candidate touch: **REQ-113** (gate refuses undischarged residual drift) gains
clauses (or companion requirements) for Failed un-acceptability, the VH-Blocked bar,
and the withdrawal-act rule. The REV is **shaped after this design locks**
(PHASE-01); spec authorizes the code, so governance lands first. Routed per ADR-013
(governance edit → Revision).

## 6. Phasing (shape — `/plan` sets criteria)

- **PHASE-01 — governance:** author + approve + apply the D8/REQ-113 REV (Failed
  un-acceptable, VH-Blocked bar, withdrawal-act rule, abandon-not-gated
  clarification). Also seed SL-179's own `[gate].extra_reqs` with the REV targets so
  the dogfood is non-vacuous (codex M8/B1).
- **PHASE-02 — verdict model:** §4.1–4.3 (`coverage.rs`, `reconcile.rs`,
  `coverage_view.rs`) + the `has_fresh_vh()` helper. Behaviour-preserving except the
  named reason split.
- **PHASE-03 — closure gate:** §4.4 (`slice.rs`) — Failed hard-refuse, VH-Blocked
  bar, M7 empty-keys guard, D4 withdrawal check, per-reason bail copy.
- **PHASE-04 — forget guard:** §4.5 (`coverage_store.rs`). File-disjoint from
  PHASE-03; /plan may parallelize or merge.

## 7. Verification / closure intent

- VT: live `Failed` on a gate-set req refuses `reconcile→done`; **no** accept-REC
  discharges it (regression vs the current accept path).
- VT: `Blocked` + fresh `VH` Verified + accept-REC citing both keys → closes;
  `Blocked` with only a VT/VA Verified, or no Verified → refuses (D3).
- VT: req flipped to `Retired`/`Superseded` over a live Failed/Blocked cell refuses
  unless a slice-owned revise/redesign REC cites the evidence keys (D4).
- VT: `Indeterminate`/lag accept with **empty** residual keys → refuses (M7).
- VA: the reconcile writer still cannot observe `Composite`/`Verdict` at
  `select_status` after the reason split (NF-001 verdict-independence, codex M9).
- VA: SL-179's own close seeds a declared cross-slice Failed cell and proves the
  **candidate binary** refuses (non-vacuous dogfood, codex M8).
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
  attestation. *(Superseded by D3 — the bar is now VH-only + cite-both-keys.)*

### 9.1 External adversarial pass — codex GPT-5.5 (integrated)

Read-only review of the design + gate/coverage/reconcile surface. Disposition:

**Folded into the design:**
- **B3 → D4** — withdrawal-over-contradiction was a real escape (req flipped to
  Retired → drift Coherent → close → reactivate). Now requires a recorded
  revise/redesign REC.
- **M5 → D3** — Blocked bar tightened from mode-agnostic `any_fresh_verified` to a
  fresh **VH** cell + REC citing both keys.
- **M7** — vacuous discharge: accept with empty `residual_keys` now refused.
- **M9** — added NF-001 verdict-independence VA after the reason split.
- **M10** — control flow pinned: classify in `undischarged_drift`; `rec_discharges`
  signature unchanged.
- **M8 / B1-dogfood** — PHASE-01 seeds SL-179's own `[gate].extra_reqs`; final VA
  seeds a declared cross-slice Failed cell and proves the candidate binary refuses.
- **M6** — `abandoned` is not gated (giving-up ≠ closing); clarified in the D8 REV.

**Verified mostly-moot:**
- **B2** — `redesign` empty-`status_delta` keeps a req out of the `reconciled` gate
  term, but `redesign` drives the ADR-009 back-edge (`reconcile.rs:14,265`) → the
  slice returns to `design` and cannot close that way. Residual is the gate-set
  breadth theme (B1).

**Deferred to follow-up backlog (out of RSK-008 scope):**
- **B1** → **RSK-012** — closure gate-set scope is per-slice; a foreign Failed req
  can be omitted by not declaring it (no *silent* leak — un-declaring is a reviewed
  toml edit; per-slice scope is deliberate, SL-044 D-B2).
- **M4** → **RSK-013** — `scan_coverage` silently skips malformed/unreadable
  `coverage.toml`; closure needs a strict (fail-closed) scan mode. A genuine silent
  gap, but broader machinery than this slice.
