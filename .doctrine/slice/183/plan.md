# Implementation Plan SL-183: macOS Seatbelt write-confinement arm

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four phases take the locked design (D-mac1..4, RV-203-discharged) to a green,
parity-verified macOS Seatbelt write-floor. The spine is the design's **pure/impure
split** (RV-203 F-1): a thin impure resolver feeds two pure builders. The plan
front-loads the one piece of **deferred empirical work** the inquisition surfaced —
the final DUTMP/xcrun_db profile and SBPL regex semantics were *design-decided but
never probed* (F-2/F-3) — so no Rust commits to a profile shape that hasn't been
pinned on a real host.

- **PHASE-01 — Confirmation probe.** Discharges the RV-203 F-2/F-3 obligation with a
  disposable shell rig (the same posture as the discharged RSK-014 H2 probes). Pins
  the exact final profile (ordering, canary preservation, xcrun-tool-works) and the
  anchored-regex boundary before the builders encode it.
- **PHASE-02 — Pure builders.** `seatbelt_profile` + `sandbox_exec_argv` as pure
  functions (resolved inputs → String/Vec), with the STD-001 named-constant catalog.
  Unit-testable in isolation.
- **PHASE-03 — Impure resolver + wiring.** `resolve_inputs` (cwd→git derivation,
  realpath, getconf, policy read) with the six **fail-closed** branches (F-4), and
  the `select_jailer` macOS branch that routes into SL-182's seam unchanged.
- **PHASE-04 — Parity + in-situ verification.** Behaviour-preservation (SL-182 suites
  green), the in-situ containment assertion (mirrors pass-2), and the degrade
  contract.

## Sequencing & Rationale

**Why probe-first.** The design's §5.1 profile is only *partly* probe-proven: the
base shape passed RSK-014 H2, but the DUTMP-deny + anchored-`xcrun_db`-allow were
added at OQ-mac4 resolution, after the probes closed. RV-203 (F-2/F-3) refused to let
that be labelled "proven." PHASE-01 pins it empirically so PHASE-02 encodes a
verified shape, not an asserted one. It is scheduled first because it is the only
phase **not** blocked on SL-182 — it is shell, runnable on any macOS host today.

**Why pure-before-impure.** The pure/impure split is ADR-001 and the F-1 remedy.
Building the pure layer first (PHASE-02) lets its tests run with zero I/O and zero
git/host dependence — the ordering invariant (F-A), the conditional network line, the
device-sink set, and the anchored regex are all exercised as string output. PHASE-03
then adds the thin impure shell that *feeds* those builders, so the fail-closed logic
(F-4's six branches) is tested against a pure, already-green target.

**Why integration is its own phase.** Parity is the slice's whole reason to exist
(reuse SL-182's `Decision`/`Target`/policy/funnel, fork only the builder). PHASE-04
isolates the behaviour-preservation gate (SL-182's suites green *unchanged*) and the
in-situ leg (needs a macOS host, R-mac4) from the unit-level implementation phases.

**The SL-182 dependency.** PHASE-02/03/04 are `needs SL-182` — they touch
`src/worktree/jail.rs` and the `select_jailer` fork point, which live in SL-182's
design but not yet on disk. As of planning, **SL-182 is `started`** (implementation
underway, advanced from `ready`), so the block is expected to lift soon; but the plan
does not assume it has. PHASE-01 can proceed regardless. The lifecycle move to
`ready`/`started` for SL-183's own phases waits on SL-182's `jail.rs` landing.

## Notes

- **Verification modes.** PHASE-01 and PHASE-04's in-situ legs are `VA`/`VH` — an
  automated test cannot judge a live macOS Seatbelt-nesting run; those are agent- and
  human-verified, mirroring how the RSK-014 probes were accepted. The pure-layer and
  fail-closed logic (PHASE-02/03) are `VT` — ordinary Rust unit tests.
- **VT mandates** point at `src/worktree/jail.rs` (SL-182's module, where the fork
  lives). If SL-182 lands the seam in a differently-named module, update the
  `test_file` mandates at `/phase-plan` time — the path is SL-182's to fix, not a
  stale premise in this plan.
- **Carry-forward from RV-203** (all folded into the phases above): F-2/F-3 →
  PHASE-01; F-1/F-9 → PHASE-02; F-4/F-6 → PHASE-03; behaviour-preservation → PHASE-04.
