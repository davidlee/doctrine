# Implementation Plan SL-170: Dispatch handover trust-gate

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-170 turns "verify, don't trust the worker self-report" from skill-prose into
a mechanical gate at the dispatch funnel. The design (locked, 2 adversarial
passes, INV-1..8) splits the work into two mechanism-independent sub-systems
delivered as four phases (design D3):

- **S1 — regression baseline-diff** (PHASE-02): catch a slice-caused regression
  that today ships as "env", by diffing full-suite failure-sets at the phase
  base `B` against the post-implementation tree `S`.
- **S3 — VT existence/shape gate** (PHASE-01 lift → PHASE-03 gate): catch a
  *mandated-but-missing* test, by lifting plan VT criteria into a structured,
  checkable model and asserting each mandated `test_file` exists and contains its
  `keywords`.
- **S6 — VT-status summary** (PHASE-04): make S3's verdict legible at handover,
  not at downstream audit.

The two failure modes this addresses both reduce to *noise getting ignored*
(SL-169's regression shipped as env; SL-168 F-1's pre-existing RED dismissed).
So the governing constraint across every phase is **zero false-fails** (design
principle 4): a gap surfaces as a visible non-halting state (`UNCHECKABLE` /
`WAIVED` / a warned-but-tolerated `persistent` baseline), never as a false
alarm that re-trains the operator to ignore the gate.

## Sequencing & Rationale

**Why this order.** PHASE-01 is the shared substrate: it lifts EN/EX/VT into the
parsed `PlanPhase` and adds the P2 structured fields that PHASE-03's gate reads.
PHASE-02 (S1) is genuinely independent — a new module (`regression.rs`) and a new
`check` subcommand touching neither `plan.rs` nor the gate — so it carries **no
dependency** and is file-disjoint from PHASE-01/03 (parallelizable under
dispatch). PHASE-03 depends on PHASE-01 (it consumes the lifted model). PHASE-04
depends on PHASE-03 (it renders that gate's verdict at conclude). So the only
hard chain is 01 → 03 → 04, with 02 free to run alongside.

**The dependency graph (for dispatch):**

```
PHASE-01 ──► PHASE-03 ──► PHASE-04
PHASE-02 (independent, file-disjoint — parallel with 01/03)
```

**Phase boundaries are file- and concern-cohesive, not mechanism-coupled** (D3).
S1 and S3 share no code; they share the slice's governance home and the funnel
files (`dispatch.rs`, `commands/`, the skills). Splitting at the phase boundary
keeps each phase a single reviewable concern while one slice carries the shared
ceremony.

**Behaviour-preservation is the PHASE-01 gate.** `plan.rs` is shared machinery;
the proof the additive lift broke nothing is the *existing* `Plan::parse` suite
passing UNEDITED (VT-2, design §3). Every new field is `#[serde(default)]`, so a
legacy plan with no EN/EX/VT and no structured fields round-trips to defaulted
empties.

**Dogfood (design §9).** This plan's own VT rows are the first to carry the P2
`test_file`/`keywords` mandate. The fields are inert (serde-ignored) until
PHASE-01 lands the model, after which `verify-vt SL-170` judges this very plan —
it must come back verify-vt-clean (no `Fail`) at the slice's own conclude. That
is both the acceptance of the P2 surface and the slice's self-test.

**The acceptance proof is the SL-169 replay** (design §9), split across the two
gates that would each independently have stopped the original false-green:
PHASE-02 VT-7 reconstructs the `e2e_standard_cli_golden` regression → `new` →
halt; PHASE-03 VT-4 reconstructs the absent conformance matrix → missing-keyword
`Fail` → halt.

## Notes

**Verdict / exit semantics — INV-7 governs.** Design §5.2 (line 146) and INV-7
both make the S1 gate halt on **`new ∪ changed`**, not `new` alone — the
`changed` bucket (codex F-1) catches a pre-existing-RED test regressed into a
*new failure mode* (same key, different signature). Two stale CLI bullet lines
in design §5.2/§5.4 still read "exit non-zero iff new ≠ ∅"; these predate the
codex F-1 integration and are superseded by INV-7. `plan.toml` EX-4 encodes the
authoritative `new ∪ changed`. (Flagged for the design's own housekeeping; not a
plan-blocking ambiguity.)

**PHASE-02 fingerprint accessor (selector resolution, INV-8).** The
run-fingerprint needs worker-marker state and doctrine-bin provenance. If
`worktree/marker.rs` lacks a read accessor for these, PHASE-02 adds one (an
SL-169 under-declaration). Sourcing them is an impl detail pinned at PHASE-02,
not a new design decision.

**PHASE-02 signature normaliser** — the volatile-token list (addresses,
durations, tmp paths, hashes) is an implementation detail pinned against captured
cargo-output fixtures (hermetic, SL-168 F-2), not live runs. Getting the
normaliser's breadth right is the chief PHASE-02 correctness risk (design R1): too
narrow → signature flaps → false `changed`; too broad → a real failure-mode change
hides in `persistent`. The fixture suite is the calibration surface.

**PHASE-04 reader altitude.** Conclude uses the **fs reader** (the worker's tests
are on the coord working fs at `S`; absent a mid-dispatch waiver the coord
`plan.toml` == the authored plan, so one tree reads both — design §5.4). INV-6 is
honoured here as a *cadence rule* — a mid-dispatch waiver must be committed onto
`dispatch/<slice>` before the gate. The un-skippable `prepare_review`
committed-graph reader (which would make the gate binary-enforced) stays a
**deferred follow-up**; the pure core already takes an injected reader, so the
seam is built for it.

**Predicted test paths** carried in the VT `test_file` fields
(`tests/e2e_check_regression.rs`, `tests/e2e_slice_verify_vt.rs`,
`tests/e2e_dispatch_verify_vt.rs`) are design-target predictions — adjust at impl
if the test lands in an existing file, and update the VT mandate to match (a
plan edit, never a silent skip).

**Out of scope (design §2 / Non-Goals):** `coverage_verify.rs` (granularity
mismatch — per-VT-cell matcher, not a full-suite failure-set); S2/S4/S5 from the
SL-169 PIR; semantic test-quality judgement; IMP-194's finding-granularity
generalization to gates/doctor (the diff is built general for it, but only the
test extractor is wired — D4).
