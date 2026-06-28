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

## Open questions — RESOLVED at `/design` (see design.md §7)

- **OQ-1 (S1 locus) → orchestrator-side** (D2). The trusted seam; both runs on the
  coord tree make env artifacts cancel into `persistent`. Cost (2nd suite run)
  mitigated by sha-keyed carry-forward → one run/batch steady-state.
- **OQ-2 (IMP-130 / IDE-008) → leave IMP-130; relate IDE-008** (D6). IMP-130 is the
  RV-116 close-source candidate-drift guard, orthogonal to VT completeness (its
  apparent overlap is a loose memory link). IDE-008 is S3's complementary twin
  (executable pass/fail at solo flip vs S3's structural existence/shape at dispatch
  handover); shares PHASE-01's plan-VT lift. Linked `SL-170 related IDE-008`.
- **OQ-3 (altitude) → one slice, phase-split** (D3). PHASE-01 lift → 02 S1 / 03 S3 /
  04 S6. Mechanism-independent but file- and governance-cohesive.
- **OQ-4 (env-artifact masking) → discharged by INV-1** (same-tree capture+diff).
  Resolved as a property of OQ-1=A, not a separate mechanism.
- **NEW (baseline-diff generalization, ex SL-168) → test-only now, general
  classifier** (D4). Gates/doctor need finding-granularity (stable finding identity)
  — real work, IMP-194's; the diff is built general so IMP-194 plugs in extractors.
- **S3 mechanism → P2 structured fields** (D5). Free-text `expects` too heterogeneous;
  P2 is the only zero-false-fail posture; forward-only, with `Uncheckable` surfaced.

## Escape valve (infeasible / mis-specified VT)

A VT proving incorrect or impossible mid-execution must NOT be silently skipped or
self-relaxed. Obstacle → `/consult` → human OK → revise (`test_file`/`keywords`
edit) or **waive** (`waived=true` + `waived_reason`, id + original mandate retained
— append, never renumber). Only a human-authorized plan edit can relax; worker and
dispatch-orchestrator cannot. A waiver is visible (`WAIVED` + reason), never a silent
pass (design §5.4).

## Verification / closure intent

A dispatched slice carrying (a) an injected regression masked as env, or (b) a
missing mandated VT, is caught at the handover/verify gate — not at downstream
audit. Verified by: unit tests on the pure baseline-diff and VT-existence
classifiers; e2e on the conclude summary surface.

## Summary

## Follow-Ups

- **IMP-194** extends the general S1 finding-key diff to gates (layering) + doctor —
  needs finding-granularity (stable finding identity); the diff seam is built for it.
- **`prepare_review` hardening (residual OQ):** make the S3 gate un-skippable in the
  binary by having `prepare_review` call `vtgate::check_phases` with a git-tree
  reader. Seam built (injected reader); wiring deferred pending cadence-trust signal.
- **Forward link to SL-057 / IDE-008:** the structured VT row is the join key that
  lets `verify::resolve` attach a runnable check per plan VT; the coverage graph
  already traces VT → phase → slice → REQ — continuous re-derivation of plan VTs
  back to originating requirements becomes reachable downstream (design §7).
