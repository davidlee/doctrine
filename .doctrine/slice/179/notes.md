# SL-179 — implementation notes

Closure gate hardens on a live `Failed` coverage cell; closes the `forget`
evidence-erasure leak. Origin RSK-008. Governance: REV-017 (SPEC-002 D8 + REQ-113).

## Shape delivered

Two contradiction sources that `drift` used to lump as one `ObservedContradiction`
are now split and treated asymmetrically:

- **`Failed`** (a check ran and contradicted) — **hard refuse**, never
  accept-dischargeable. Fix (`coverage verify` re-derives Failed→Verified) or
  withdraw the req via a recorded act.
- **`Blocked`** (evidence unobtainable) — reconcilable, but **stricter than lag**:
  requires a fresh human (VH) `Verified` cell + an accept-REC citing both keys (D3).

## Phase rollup

- **PHASE-01 (`93a60a31`) — governance.** REV-017 narrowed SPEC-002 D8 + REQ-113 to
  four narrowings (Failed un-acceptable, VH-Blocked bar, withdrawal-as-recorded-act
  D4, `done`-gated-not-`abandoned` M6). Seeded `slice-179.toml [gate].extra_reqs=
  ["REQ-113"]` so the dogfood is non-vacuous (codex M8/B1).
- **PHASE-02 (`aa92b9b4`) — verdict model.** `DivergentReason` split into
  `ObservedFailure`/`ObservedBlocked`; drift Failed>Blocked precedence; new
  `Composite::{any_failed, any_blocked, has_fresh_vh}`; `reconcile::divergent_label`
  gains two arms. `coverage_view.rs` needed **no edit** — the `Verdict::Divergent(r)
  => r.label()` delegation (`:393`) auto-renders the new labels, and `observed_state`
  keeps the combined `any_failed_or_blocked` summary (audit F-1).
- **PHASE-03 (`bdc3ddca`) — closure gate (`slice.rs`).** Pure
  `classify_undischarged(authored, &composite, &residual_keys, owned_recs, req)
  -> Option<UndischargeReason>` carries all policy; `undischarged_drift` stays a thin
  impure loop; `rec_discharges` byte-frozen (codex M10). Four strands: Failure ⇒
  always undischarged; BlockedNoVh ⇒ `has_fresh_vh() && rec_discharges && cites VH
  key`; Lag/Indeterminate ⇒ `rec_discharges` + M7 empty-residual-keys guard; D4
  withdrawal ⇒ `withdrawal_discharged` scans `owned_recs` directly (redesign RECs
  carry empty `status_delta`, so `latest_owning_rec_for` can't see them — F2).
  Per-reason bail copy via `UndischargeReason::{header,remedy}` (STD-001).
- **PHASE-04 (`067542cb`) — forget guard (`coverage_store.rs`).** `forget ->
  ForgetOutcome { Erased(k,s), Refused(s), NotFound }`; refusal decided **before**
  mutation (atomic — removal-then-return can't be post-checked). Pure exhaustive
  `is_erasable(status)` ({Planned,InProgress,Verified} erasable; {Failed,Blocked}
  refused; no wildcard, no string literals). `run_forget`: Refused ⇒ `anyhow::bail!`
  (Err, exit 1) naming the 4-tuple + four remedies.

## Audit (RV-198) evidence

Clean audit, no blockers. `check gate` green; lag-discharge anchors (vt4/vt5/vt6 +
close-integration) unchanged (behaviour-preservation, design F2).

**Deferred VA dogfoods executed on the candidate `./target/debug/doctrine`:**
seeded a VT cell on REQ-113 with a failing check (`--command false` + unmatched
matcher), `coverage verify` → **Failed** (`Divergent: observed-failure`).

- **PHASE-04 VA-1 — MET.** `coverage forget` of the Failed cell refused (exit 1),
  named the 4-tuple + four remedies, cell remained (no remove/no save).
- **PHASE-03 VA-1 — MET.** `slice status SL-179 done` (from `reconcile`) refused,
  citing `REQ-113 (authored: active)` under "Failed coverage cell — a check ran and
  contradicted (not accept-dischargeable)" + mode-aware remedy. Refused ⇒ no
  transition. Scratch cell removed (untracked `coverage.toml`); tree clean.
- **PHASE-03 VA-2 — MET.** `reconcile::select_status(to, prior)` takes no
  `Verdict`/`Composite` — NF-001 verdict-independence wall intact.
- **PHASE-01 VA-1 — MET.** SPEC-002 D8 + REQ-113/FR-006 carry all four narrowings,
  consistent with design D3/D4; no uniform-residual wording remains.

**Findings (both `aligned`):** F-1 `coverage_view.rs` undelivered (delegation
absorbed the split — optional design §4.3 prose tidy at /reconcile); F-2 six
undeclared paths (all PHASE-01 governance/metadata, not code drift).

## Reusable findings (durable)

- `clippy::indexing-slicing` is `-D` here — `vec[pos]` after `position()` rejected
  even when provably valid; use `.get(pos)` and treat `None` as the miss arm (F-04a).
- Coverage forget-guard tests can't go through `record` (VT-check leans Planned,
  never Failed/Blocked) — the `seed_cell` fixture upserts an exact stored status
  directly (F-04b). The **CLI** equivalent for a live dogfood: record a VT cell with
  a failing `--command` + unmatched matcher, then `coverage verify` to derive Failed
  (see [[mem.pattern.audit.cli-seed-failed-coverage-cell]]).
