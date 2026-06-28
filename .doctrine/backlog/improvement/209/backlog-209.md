# IMP-209: Plan skill should author structured VT mandates so verify-vt has signal

## Context

Surfaced by the SL-171 audit (RV-190 F-4). `doctrine slice verify-vt` is meant to
give a mechanical S3 coverage signal at conclude/audit time, but it can only check
a VT that carries a **structured mandate** (a `test_file` / `keywords` it can match
against the source tree). The `/plan` skill authors VTs as prose-only `expects =
"..."` strings, so verify-vt reports every VT **UNCHECKABLE** — the gate is inert
project-wide.

On SL-171 this meant all 10 VTs (5×PHASE-01, 5×PHASE-02) came back UNCHECKABLE: the
gate could neither confirm PHASE-01's genuinely-strong coverage nor flag PHASE-02's
real test gap (the missing-pagination-tests finding, RV-190 F-3, which the audit
caught by hand and remediated). A coverage gate that never fires is worse than none
— it reads as a green check while signalling nothing.

## Proposal

Have the `/plan` skill author VTs with the structured fields verify-vt consumes
(at minimum the `test_file` glob + matching `keywords`/test-name hints), so the
gate has something to check. Confirm the exact field shape verify-vt expects (read
the verify-vt impl), then update the plan skill + plan.toml template/guidance so
VT authoring is structured-by-default. Optionally tighten conclude/audit to treat a
**bare** (mandate-less) VT as a warning rather than silent UNCHECKABLE.

## Scope notes

- Project-wide tooling/process gap, not specific to SL-171.
- SL-171's own plan was **not** backfilled retroactively (audit decision: marginal
  post-hoc value; the real coverage was confirmed by the existing tests + the
  audit-added pagination tests).
- Source: RV-190 (SL-171 audit) finding F-4; see also F-3 (the gap F-4's dead
  signal failed to surface).
