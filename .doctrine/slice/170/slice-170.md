# Dispatch handover trust-gate

## Context

SL-169's dispatch handed off "complete and green" while carrying its own
`e2e_standard_cli_golden` regressions, which the worker misattributed to the
`DOCTRINE_WORKER` env filter. RV-184 caught it only via a clean re-run; the
downstream auditor had no independent signal to distrust the handover. Two
classes of trust-gap surfaced (SL-169 PIR §3):

- **Regression blindness (S1):** the funnel "Verify" step is orchestrator-side
  *prose* (dispatch SKILL.md step 5) — the agent runs the suite, eyeballs RED,
  isolates by hand. No baseline. A new failure is indistinguishable from a
  pre-existing/environmental one, so a slice-caused regression can be shipped as
  "env". (NB: `coverage_verify` is per-VT-cell matcher re-derivation, NOT a
  full-suite failure-set — it is the *wrong* home for S1; see Affected surface.)
- **Completeness blindness (S3/S6):** the handover tests *what exists*, not *what
  was mandated*. SL-169 PHASE-05 VT-1 promised a parse-conformance matrix; only
  the columns golden landed (the file existed; content was incomplete). The gate
  stayed green. `plan.rs` serde-drops EN/EX/VT criteria today, so nothing can
  check them structurally.

Governing principle (already half-doctrine on the claude arm,
[[mem.pattern.dispatch.claude-arm-isolation-fallback]]): **verify, don't trust the
worker self-report.** This slice moves that from convention into the gate.

## Scope & Objectives

1. **S1 — regression diff.** Capture a baseline failure-set at the phase base `B`
   *in the same environment the verify runs in*, then diff the post-implementation
   failure-set against it. Any *new* failure is a slice regression regardless of
   which test binary or env var it surfaces under. Pure classifier over two
   failure-sets; thin shell captures them.
2. **S3 — VT existence/shape gate.** Lift EN/EX/VT criteria from `plan.toml` into
   the parsed `PlanPhase` model, then add a structural check at the conclude/
   prepare-review gate: every VT (mode `VT`) criterion's mandated test file exists
   and contains the criterion's mandated keywords. Structural, not semantic —
   "does `e2e_list_conformance.rs` mention `relation` and `census`?", not "is the
   assertion correct?".
3. **S6 — VT-status summary.** Emit a per-phase VT pass/exist summary at the
   dispatch conclude/handover step — the human-readable read-surface of S3's
   check, making gaps visible at handover, not at audit.

## Non-Goals

- **S2** (per-kind golden guard on shared renderers), **S4** (split-lineage
  prevention), **S5** (design-time selector declaration) — separate backlog items
  from the same PIR; out of scope here.
- **Semantic** test-quality judgement — this gate asserts existence/shape, never
  that an assertion is correct or sufficient.
- Re-architecting the worker trust model or the funnel topology. We add gates to
  the existing import→verify→conclude cadence; we do not move where work lands.

## Affected surface (coarse — `/design` refines)

- `src/plan.rs` — lift EN/EX/VT criteria into `PlanPhase` (currently serde-dropped).
- `src/coverage_verify.rs` — baseline-diff seam in `run()`; carry delta metadata.
- `src/dispatch.rs` — `prepare_review()` completeness gate; conclude VT summary.
- `src/commands/` — possible new `doctrine slice verify-vt <id>` surface.
- `plugins/doctrine/skills/dispatch/**`, `skills/handover/**` — funnel cadence +
  handover message format.
- Tests: `tests/e2e_*` for conclude/verify behaviour; unit for the pure classifiers.

## Risks, assumptions, open questions

- **OQ-1 (S1 locus):** baseline capture worker-side (self-correct) vs
  orchestrator-side (verify-don't-trust the unreliable party). Orchestrator is the
  trusted seam but pays a second full-suite run. `/design` decides.
- **OQ-2 (IMP-130 overlap):** [[mem.pattern.audit.dispatched-phase-green-but-incomplete]]
  links IMP-130, which covers S3's territory at audit time. Dedupe / relate at
  `/design` — this slice may *subsume* or *complement* it.
- **OQ-3 (one slice or split):** S1 (regression) is mechanism-independent of
  S3/S6 (completeness, which share the plan-VT-model dependency). Could be a
  phase split within SL-170 or two slices. `/design` decides altitude.
- **OQ-4 (env-artifact masking):** baseline MUST be captured in the verify
  environment, else fork artifacts (embed gap, `DOCTRINE_WORKER`, stale bin —
  [[mem.pattern.dispatch.worker-fork-missing-gitignored-embed]]) read as
  regressions. The baseline-on-`B` run is exactly the disambiguator.
- **Assumption:** `plan.toml` VT criteria carry enough text (mandated file +
  keywords) to drive a structural match. To confirm at `/design`.

## Verification / closure intent

A dispatched slice carrying (a) an injected regression masked as env, or (b) a
missing mandated VT, is caught at the handover/verify gate — not at downstream
audit. Verified by: unit tests on the pure baseline-diff and VT-existence
classifiers; e2e on the conclude summary surface.

## Summary

## Follow-Ups
