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

1. **S1 — regression diff (orchestrator-side).** Capture a baseline failure-set at
   the phase base `B` *on the coordination tree* (pre-spawn), then diff the
   post-implementation failure-set (at `S`, same tree) against it. Any *new* failure
   is a slice regression regardless of which test binary or env var it surfaces
   under; the failures `B` and `S` share absorb every env artifact (the
   disambiguator). Pure classifier over two failure-sets keyed by an opaque
   finding-key; the classifier is **general** (IMP-194 later feeds layering/doctor
   finding-keys to the same diff) but only the **test** extractor is wired now.
   Thin shell runs the suite and caches the sha-keyed baseline.
2. **S3 — VT existence/shape gate.** Lift EN/EX/VT criteria from `plan.toml` into
   the parsed `PlanPhase` model, **plus structured `test_file` / `keywords` fields
   on VT rows** (P2 — free-text `expects` is too heterogeneous to parse reliably).
   Add a `doctrine slice verify-vt <id>` gate: every VT (mode `VT`) criterion whose
   structured mandate is present has its `test_file` exist and contain its
   `keywords`. Four verdicts — `Pass` / `Fail` (halts) / `Uncheckable` (no
   structured mandate) / `Waived` (human-authorized, rationale shown). Zero
   false-fails: the gate halts on `Fail` only.
3. **S6 — VT-status summary.** Emit a per-phase VT verdict summary at the dispatch
   conclude/handover step — the human-readable read-surface of S3's check, making
   gaps (incl. `Uncheckable` / `Waived`, rendered distinctly) visible at handover,
   not at audit.

## Non-Goals

- **S2** (per-kind golden guard on shared renderers), **S4** (split-lineage
  prevention), **S5** (design-time selector declaration) — separate backlog items
  from the same PIR; out of scope here.
- **Semantic** test-quality judgement — this gate asserts existence/shape, never
  that an assertion is correct or sufficient.
- Re-architecting the worker trust model or the funnel topology. We add gates to
  the existing import→verify→conclude cadence; we do not move where work lands.

## Affected surface (`/design` refined — design-target selectors recorded)

- `src/plan.rs` — lift EN/EX/VT into `PlanPhase` + structured `test_file`/`keywords`/
  `waived` VT fields (PHASE-01). All additive `#[serde(default)]` (behaviour-preserving).
- `src/regression.rs` *(new)* — pure `parse_failures` + `diff` + `RegressionDelta` +
  `render_delta` (PHASE-02).
- `src/vtgate.rs` *(new)* — pure `check_vt` + `check_phases` + `render_summary` +
  verdict constants (PHASE-03).
- `src/commands/**` — `check regression {capture,diff}`; `slice verify-vt <id>`
  (+ optional `waive-vt`).
- `src/dispatch.rs` — conclude: invoke verify-vt render (PHASE-04). `prepare_review`
  fold = deferred hardening option (seam built, wiring not).
- `plugins/doctrine/skills/{dispatch,handover,plan}/SKILL.md` — funnel cadence
  (capture/diff), handover VT block, `/plan` authoring discipline (`test_file`/`keywords`).
- Tests: `tests/e2e_*` + unit for the pure classifiers (hermetic fixtures, SL-168 F-2).
- **EXCLUDED:** `src/coverage_verify.rs` — granularity mismatch (per-VT-cell matcher,
  not a full-suite failure-set). S1 is a new module, not a coverage_verify edit
  (corrects the original coarse guess; design §2 / D-correction).

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
