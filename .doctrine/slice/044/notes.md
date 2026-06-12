# SL-044 notes — reconcile writer + closure gate

Durable implementation notes. Progress lives in the runtime phase sheets; this
is the keep-after-close residue.

## RV-004 reconciliation audit (close-ready)

Conformance audit driven off an external adversarial review (Opus,
*revision-required*, 2 blockers). All findings re-verified against source, raised
on RV-004, dispositioned terminal. Fixes in commit `36156fd` + the F-5 reorder.

Key outcomes (full prose in `review/004/review-004.md` `## Synthesis`):

- **NF-001 wall proof (F-1, blocker).** VT-5 `written_status_is_verdict_independent`
  was vacuous — it asserted `select_status(fixed, _) == fixed` (identity) and never
  called `run()`, so the laundering surface was untested. Rewired through the real
  `run()`: vary on-disk coverage (≥3 verdicts), hold `--to` fixed, assert the
  on-disk authored status == `--to`. VT-6 supplied the harness.
- **`select_status` honesty (F-2).** Signature isolation constrains the fn body,
  NOT the call site. The invariant is the three layers + VT-5, not a compiler proof.
  Docstring corrected to stop overclaiming.
- **Discharge for-R (F-3).** `rec_discharges` clause (b) now matches
  `d.requirement == req` before `d.to == authored`. Without it a multi-delta
  hand-authored REC could discharge R's drift on another requirement's coinciding
  `to`. Regression: `multi_delta_rec_does_not_discharge_via_foreign_requirement`.
- **One `distinct_keys` (F-4).** Twin removed; `coverage::distinct_keys` is the
  single deduper for the writer's `evidence_ref` and the gate's residual keys.
- **Write-ahead ordering (F-5).** accept/revise materialise the REC BEFORE
  `set_status` (NF-003: status never moves without its REC). redesign keeps
  transition-first — writes no requirement status (F7), and its guarded back-edge
  can refuse, where REC-first would orphan ledger entries. Asymmetry documented at
  the `run()` seam.

## Standing risks (consciously accepted, none block close)

- Redesign torn window (transition succeeds, REC mint fails → slice in design, no
  REC) — tolerated; no authored status moves.
- `--note` has three deliberate semantics across the new surface (spec discards;
  reconcile accept/revise prompts; redesign forwards into the transition record).
  Unifying pass is optional, not a defect (F-9).
- Closure-seam classifier not extracted (F-7) — two call sites don't yet earn it.
- Per-req `scan_coverage` corpus walks at close (RSK-006) and the ISS-006
  slug-symlink double-walk persist; `distinct_keys` only de-dupes the *symptom*.
