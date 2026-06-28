# Design SL-170: Dispatch handover trust-gate

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

SL-169's dispatch handed off "complete and green" while carrying its own
`e2e_standard_cli_golden` regressions, which the worker misattributed to the
`DOCTRINE_WORKER` env filter (SL-169 PIR §2.1). The downstream auditor had no
independent signal to distrust the handover. Two trust-gaps surfaced:

- **S1 — regression blindness.** The funnel "Verify" step re-derives pass/fail
  from *current* runs only — there is no baseline. A new failure is
  indistinguishable from a pre-existing/environmental one, so a slice-caused
  regression can be shipped as "env".
- **S3/S6 — completeness blindness.** The handover tests *what exists*, not *what
  was mandated*. SL-169 PHASE-05 VT-1 promised a parse-conformance matrix
  (`e2e_list_conformance.rs` covering `relation` + `census`); only the columns
  golden landed, the file existed, the gate stayed green.

Governing principle: **verify, don't trust the worker self-report.** This slice
moves that from convention into a mechanical gate at the dispatch funnel.

## 2. Current State

- **Funnel "Verify" is orchestrator-side prose** (`plugins/doctrine/skills/dispatch/SKILL.md`
  step 5: "run project verify; if RED, isolate offender per delta"). The agent runs
  the suite, eyeballs RED, isolates by hand. No mechanical baseline-diff. That
  hand-judgement is exactly what failed in SL-169 (and SL-168 F-1: a pre-existing-RED
  layering gate dismissed as noise).
- **`coverage_verify::run()`** re-derives *per-VT-cell* status via configured
  matchers (SL-057 contract). Its granularity is the VT coverage *set* — a small
  subset of the suite. It is **not** a full-suite failure-set runner.
- **`plan.rs::PlanPhase`** serde-drops `entrance_criteria` / `exit_criteria` /
  `verification`. The authored `verification` rows are `{id="VT-1", expects="<free
  text>"}` — mode is encoded in the id prefix; there is no structured mandated-file
  or keyword field. Nothing can structurally check VT completeness today.
- **`dispatch::prepare_review`** is the enforced conclude beat: it commits the
  boundaries ledger, derives the conformance registry, and `bail!`s on registry
  incompleteness — but checks ref/registry topology, never VT content.
- Empirically, authored `expects` strings are **wildly heterogeneous**: some name a
  `.rs` file, many name only a *suite* ("backlog suite", "full suite green"), many
  name only test-*fn* names, many are pure behavioural prose. No reliable mandated
  file+keyword structure exists in free text (D5).

## 3. Forces & Constraints

- **ADR-001 (layering).** Pure classifiers live in the leaf/engine tier (no clock,
  rng, git, disk); IO (suite-run, file reads, cache writes) lives in the thin
  command/shell — the date/uid injection pattern.
- **ADR-012 (dispatch topology).** The orchestrator is the sole writer and the
  trusted seam; the coordination worktree isolates the working tree, not the trunk
  ref. The gate runs orchestrator-side, on the coordination tree.
- **Behaviour-preservation gate.** Changing `plan.rs` (shared machinery) must keep
  the existing `Plan::parse` suite green unchanged.
- **The cry-wolf failure mode is the enemy.** SL-168 F-1 and SL-169 both failed by
  *noise getting ignored*. A false-failing gate re-introduces exactly that. The
  gate must not raise failures it cannot stand behind.
- **Storage tiers (the storage rule).** The mandate the gate checks must live in the
  *authored* tier (`plan.toml`), never the *disposable* phase sheet. The only
  runtime-tier artifact (the S1 baseline cache) must be a regenerable memo.
- **No parallel implementation.** Ride the existing `plan` model and `check` command
  group; share one pure core between every caller.

## 4. Guiding Principles

1. **Verify, don't trust.** The gate runs orchestrator-side and reads only
   authored + git state — never the worker's self-report, never disposable state.
2. **The mandate is independent of every party it judges.** VT criteria are
   authored at `/plan`, before dispatch, by neither the worker nor the
   dispatch-orchestrator. No self-grading; no mid-dispatch goalpost-moving.
3. **Green only by fulfilment, correction, or a visible recorded waiver.** Never by
   hiding a gap. `UNCHECKABLE` and `WAIVED` are surfaced, distinct, non-silent.
4. **Reliability over coverage.** Zero false-fails (P2, D5). A gap shows as a
   visible non-halting state, not a false alarm.
5. **General mechanism, narrow first wiring.** The S1 classifier is agnostic to
   what produced a failure; only the test extractor is wired now (D4).

## 5. Proposed Design

Two **mechanism-independent** sub-systems, delivered as four phases in one slice
(D3):

- **S1** — a regression baseline-diff over full-suite failure-sets (PHASE-02).
- **S3** — a VT existence/shape gate over a newly-modelled, structured plan VT
  (PHASE-01 lift → PHASE-03 gate).
- **S6** — the human-readable VT-status summary at handover (PHASE-04), the
  read-surface of S3.

### 5.1 System Model

```
                  AUTHORED (plan.toml)            GIT (B, S trees)
                        │                              │
   /plan author ───► VT criteria                 funnel pre-spawn B = HEAD
   (DEFINE)          (+ test_file/keywords)            │
                        │                              │
   ┌────────────────────┴──────────┐      ┌────────────┴───────────────┐
   │ S3/S6  vtgate (pure)           │      │ S1  regression (pure)       │
   │  check_vt / check_phases       │      │  parse_failures / diff      │
   │  render_summary                │      │  render_delta               │
   └────────────────────┬──────────┘      └────────────┬───────────────┘
                        │                              │
   shell: slice verify-vt <id>          shell: check regression capture|diff
                        │                              │
                 dispatch conclude  ◄── orchestrator (VERIFY) ──►  funnel verify
```

`leaf/engine` (pure): `regression::{parse_failures, diff, render_delta}`,
`plan::PlanPhase` (+ criteria), `vtgate::{check_vt, check_phases, render_summary}`.
`command/shell` (impure): suite-run, file reads, sha-keyed cache IO, CLI surfaces,
dispatch conclude hook, skill cadence.

### 5.2 Interfaces & Contracts

**S1 — `regression.rs` (pure):**

```rust
type FindingKey = String; // tests: "<target>::<test_name>" from the cargo failures: block.
                          // General — IMP-194 later feeds layering/doctor finding-keys to the SAME diff.

// A suite run yields EITHER a well-formed failure-set OR an unobtainable marker —
// never silently ∅ (R-A / INV-5). Mirrors coverage_verify's RunOutcome (F-VII).
enum FailureSet { Obtained(BTreeSet<FindingKey>), Unobtainable { why: String } }

struct RegressionDelta {
    new:        BTreeSet<FindingKey>, // current \ baseline = REGRESSIONS → gate halts
    fixed:      BTreeSet<FindingKey>, // baseline \ current = improvements (informational)
    persistent: BTreeSet<FindingKey>, // baseline ∩ current = pre-existing/env (IGNORED)
}

// Ok only when BOTH sides Obtained; an Unobtainable side is a hard Err (never ∅-pass).
fn diff(baseline: &FailureSet, current: &FailureSet) -> Result<RegressionDelta>;
fn parse_failures(suite_output: &str) -> FailureSet; // section-aware over cargo `failures:` blocks
fn render_delta(delta: &RegressionDelta, base: &str) -> String;
```

`new` = regressions regardless of which binary/env they surface under. `persistent`
= the failures B and S share = every env artifact (missing `web/map/dist` embed,
`DOCTRINE_WORKER`, stale bin, worker marker) — present in *both* sets, so they fall
here, never in `new`. This discharges OQ-4 mechanically — **but only when both runs
share the same test-selection** (INV-1): a differing `DOCTRINE_WORKER`/marker filter
changes *which tests run*, so an absent test reads as "fixed" and a newly-visible one
as "new". Same tree is necessary, not sufficient; same *invocation + filter state*
is required.

**The suite is the TEST suite, per-test granularity** — not the full `just gate`.
clippy/fmt/lint-js are pass/fail aggregates (coarse granularity = the SL-168 F-1
problem); they belong to IMP-194's finding-granularity extension (D4), not S1. S1
parses `cargo test` per-test output (section-aware: associate each `failures:` entry
to its target binary for a stable `FindingKey`).

**S3 — `plan.rs` model lift + `vtgate.rs` (pure):**

```rust
struct PlanPhase {
    id: String, name: String, objective: String,            // existing
    #[serde(default)] entrance_criteria: Vec<Criterion>,    // EN
    #[serde(default)] exit_criteria:     Vec<Criterion>,    // EX
    #[serde(default)] verification:      Vec<VerificationCriterion>, // VT/VA/VH
}
struct Criterion { id: String, #[serde(default)] text: String }
struct VerificationCriterion {
    id: String,                                  // "VT-1" — mode by prefix
    #[serde(default)] expects:  String,          // free-text, untouched
    #[serde(default)] test_file: Option<String>, // P2 structured mandate
    #[serde(default)] keywords:  Vec<String>,    // P2 structured mandate
    #[serde(default)] waived:    bool,           // escape valve
    #[serde(default)] waived_reason: Option<String>,
}

enum VtVerdict { Pass, Fail { reason: String }, Uncheckable, Waived { reason: String } }

// read_file injected (purity). waived checked first. VA/VH parsed but not gated.
fn check_vt(vt: &VerificationCriterion, read_file: &impl Fn(&str) -> Option<String>) -> VtVerdict;
fn check_phases(plan: &Plan, read_file: &impl Fn(&str) -> Option<String>) -> Vec<PhaseVtReport>;
fn render_summary(report: &[PhaseVtReport]) -> String;
```

Every plan field is `#[serde(default)]` → legacy plans parse with the new fields
defaulted; the existing `Plan::parse` tests stay green unchanged.

**CLI surfaces:**

- `doctrine check regression capture --base <B>` — run suite on coord tree @ B;
  write failure-set to `.doctrine/state/regression/baseline-<B>`; no-op if cached.
- `doctrine check regression diff --base <B>` — run suite @ S; load baseline-<B>;
  print `render_delta`; exit non-zero iff `new ≠ ∅`.
- `doctrine slice verify-vt <id>` — read `plan.toml`, fs-read mandated files, print
  `render_summary`; exit non-zero iff any `Fail`. `Uncheckable`/`Waived` non-halting.
- *(optional, may defer)* `doctrine slice waive-vt <id> PHASE-NN VT-n --reason "…"`
  — structured writer; baseline is a direct authored `plan.toml` edit.

`check regression` lives under the `check` group (sibling to `check
quick|commit|gate`) and is kept **general** — the funnel calls it, but it is reusable
for solo/CI and is IMP-194's extension point.

### 5.3 Data, State & Ownership

| concern | tier | owner | when |
|---|---|---|---|
| VT mandate (`test_file`/`keywords`) | **authored** (`plan.toml`) | plan author (`/plan`) | before dispatch |
| VT fulfilment (the test) | source delta | worker | during dispatch |
| VT/regression verdict | computed | orchestrator funnel | at verify/conclude |
| S1 baseline failure-set | **disposable runtime** (`.doctrine/state/regression/baseline-<sha>`) | orchestrator | per base |

**The gate reads only authored (`plan.toml`) + git state — never the disposable
phase sheet.** The phase sheet is derived from `plan.toml` at `/phase-plan` and is
the executor's scratchpad; it is never authoritative for the gate. The baseline
cache is a sha-keyed **memo**, regenerable by re-running the suite at that sha;
authoritative input is git. A cache in the disposable tier is correct *because* it
is derivable.

### 5.4 Lifecycle, Operations & Dynamics

**Funnel cadence (S1, dispatch skill):**

```
pre-spawn:  B = HEAD
            doctrine check regression capture --base <B>      # suite @ B, coord tree
              → write baseline-<B>;  NO-OP if cached (carry-forward)
  … workers run; import delta onto B (uncommitted) …
verify:     doctrine check regression diff --base <B>          # suite @ S, coord tree
              → RegressionDelta; EXIT NON-ZERO iff new ≠ ∅
commit:     (on green) ONE commit; HEAD → B'
              → the diff's CURRENT set @ S == the B' tree; persist as baseline-<B'>
```

**Carry-forward (OQ-1 cost mitigation):** S = (B + imported delta); after commit
HEAD = B' = that tree. So the diff step's current-set *is* the failure-set at B' —
persist it keyed by B', and the next batch's `capture --base B'` is a cache hit.
**Steady-state cost = one suite run per batch** (the diff), not two; only the first
batch pays the extra capture. A foreign commit / `refresh-base` changes the sha →
cache miss → honest re-capture.

**VT gate firing (S3):** the `/dispatch` conclude step runs `verify-vt` **in the
coord tree, BEFORE the coord worktree is removed** (cadence: verify-vt → on green
`prepare-review` → remove worktree). The worker's tests are on the coord working fs
at `S`, so the fs reader suffices; non-zero halts handover. **Two-tree caveat:** the
*mandate* (plan.toml) is authored state and a mid-dispatch waiver lands on the
authoring branch, not the coord fork. Absent a mid-dispatch waiver the coord
plan.toml == authored plan and one tree reads both. A mid-dispatch waiver requires
the orchestrator (sole writer of authored state) to propagate the plan.toml edit into
the coord tree before re-running the gate — else the waiver is invisible to it
(INV-6). Rejected:
folding into `prepare_review` now (would force git-tree blob reads — the delta is on
the coord branch, not primary — and mix VT-content into the ref-projection beat,
ADR-001 cohesion). The pure core takes an injected reader, so a future hardening can
have `prepare_review` call `check_phases` with a git-tree reader to make the gate
un-skippable in the binary (deferred follow-up).

**VT escape valve (infeasible/mis-specified mandate):** a VT that proves incorrect
or impossible mid-execution must **not** be silently skipped or self-relaxed.
Obstacle → `/consult` → human OK → revise-or-waive the authored plan:

- *mis-specified*: human edits `test_file`/`keywords` to the corrected mandate (per-
  slice authored edit, direct — not REV).
- *infeasible / no longer needed*: human **waives** with a recorded rationale — id
  stays (immutability: append, never renumber), original `expects`/`test_file`/
  `keywords` stay (auditability), `waived = true` + `waived_reason` appended.

Only a human-authorized plan edit (via `/consult`) can correct or waive a VT.
Neither worker nor dispatch-orchestrator may relax a mandate. A waiver is visible
(`WAIVED` + reason), never a silent pass.

**S6 at handover:** the conclude output and the `/handover` message embed
`render_summary` (and a one-line S1 status). `WAIVED` and `UNCHECKABLE` render
distinctly. Glyphs/labels are named constants (STD-001).

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1 (same env AND same selection).** S1 capture and diff run on the coord tree
  *with an identical suite invocation and identical test-selection/filter state*
  (`DOCTRINE_WORKER`/marker). Same tree alone is insufficient — a differing filter
  changes the test universe and breaks the cancellation property. The orchestrator
  normalises filter state (e.g. clears the worker marker) before both runs.
- **INV-2 (baseline source).** B is the funnel's live pre-spawn `HEAD`, **never** the
  conformance registry `code_start_oid` — sourcing from the binding would import the
  IMP-175 / IMP-192 / fork-land bugs into the regression gate.
- **INV-3 (authored mandate).** The gate reads only `plan.toml` + git; mandates are
  authored at `/plan`, independent of worker and dispatch-orchestrator.
- **INV-4 (green semantics).** The gate halts on `Fail` only. `Uncheckable` and
  `Waived` are visible, distinct, non-halting.
- **INV-5 (no silent ∅).** A non-completing or unparseable suite run is
  `Unobtainable` → a hard error (halt), never an empty failure-set. `diff` errs if
  either side is `Unobtainable`. This closes the false-green-at-S hole (a compile
  error / panic / format change at S must not read as "zero failures").
- **INV-6 (mandate currency at the gate).** `verify-vt` must read the *current
  authored* plan (incl. mid-dispatch waivers) + the *landed delta* tests. Default
  single-coord-tree read satisfies this absent a mid-dispatch waiver; a mid-dispatch
  waiver requires orchestrator propagation of plan.toml into the coord tree first.
- **A1.** A VT with `keywords` but no `test_file` is `Uncheckable` (nothing to grep).
- **A2.** `parse_failures` keys include the target/binary to disambiguate same-named
  tests across binaries (section-aware parse).
- **A3 (cache hygiene).** The baseline cache (`.doctrine/state/regression/`) is
  gitignored runtime state, written by the orchestrator, outside the worker delta —
  so it never appears in the `B..S` diff and the R-5 belt is unaffected.
- **Edge — flaky tests.** A flaky test poisons the diff (fail@B/pass@S → false
  `fixed`; pass@B/fail@S → false `new`/halt). Out of scope to *solve*; the
  new/fixed/persistent report makes a flaky halt diagnosable — a re-capture at the
  same sha exposes non-determinism (non-empty symmetric difference on an identical
  tree).
- **Edge — renamed pre-existing-failing test.** A renamed still-failing test reads as
  `fixed{old} + new{new}` → halts on `new`. Accepted (a renamed failing test
  warrants attention); documented limitation, not a bug.
- **Edge.** Empty plan / no VT criteria → empty report, exit zero. A phase with only
  VA/VH criteria → no VT lines (not gated).

## 6. Open Questions & Unknowns

All slice OQs are resolved (see §7):

- OQ-1 → orchestrator-side (D2). OQ-2 → IMP-130 orthogonal, IDE-008 related-not-
  subsumed (D6). OQ-3 → one slice, phase-split (D3). OQ-4 → discharged by INV-1.
  NEW (generalization) → test-only now, general classifier (D4). S3 mechanism → P2
  structured fields (D5).
- **Residual:** whether cadence-trust for the S3 gate is sufficient or it should be
  hardened into `prepare_review` (the seam is built; wiring deferred). Tracked as a
  follow-up, not a blocker.

## 7. Decisions, Rationale & Alternatives

- **D1 — A mechanical gate, not skill discipline.** The false-green slipped through
  human eyeballing; the fix must be a deterministic command with an exit code.
- **D2 — S1 orchestrator-side (OQ-1).** The orchestrator is the trusted seam
  (ADR-012); it captures B pre-spawn and runs both sets in one env, which *is* the
  env-artifact disambiguator (INV-1). Worker-side trusts the unreliable party and
  runs in the fork env where embed/marker/stale-bin artifacts live. Rejected.
- **D3 — One slice, phase-split (OQ-3).** S1 and S3/S6 are mechanism-independent but
  share files (`dispatch.rs`, `commands/`, the skills) and a governance home; two
  slices double ceremony. Separable at the phase boundary, cohesive at slice
  altitude. Rejected: two slices; S1-only.
- **D4 — Test-only wiring, general classifier (NEW/generalization).** SL-168 F-1
  (layering) / F-3 (doctor) are the same baseline-blindness, but catching them needs
  *finding granularity* (a new violation inside an already-RED aggregate gate is
  invisible at pass/fail granularity), which needs stable finding identity — real
  work, correctly IMP-194's. The `diff` operates on abstract finding-key sets from
  day one so IMP-194 plugs in extractors without reworking the diff. Rejected:
  cover gates/doctor now (balloons scope, gates the #1-do-now fix); test-specific
  classifier (parallel-impl with IMP-194).
- **D5 — P2 structured VT fields (S3 mechanism).** Free-text `expects` is too
  heterogeneous to extract a mandate reliably (§2), and the motivating SL-169 case
  needs the *keyword* half (the file existed; content was incomplete), so existence-
  only is insufficient and keyword-NLP is false-fail-prone. The failure mode that
  bit twice is *noise ignored*; only P2 has **zero false-fails**. Forward-only is
  acceptable: the gate's value is on *future* dispatches, and S6 surfaces
  `UNCHECKABLE` so under-specification is visible, not silently passed. Rejected:
  P1 free-text best-effort; P3 hybrid (inherits P1's fragility).
- **D6 — IMP-130 / IDE-008 (OQ-2).** IMP-130 is the RV-116 close-source candidate-
  drift guard — orthogonal to VT completeness; left alone (its apparent overlap is a
  loose memory link, not content). IDE-008 is S3's complementary twin (executable
  *pass/fail* at the solo `/execute` flip vs S3's structural *existence/shape* at the
  dispatch handover); they share the plan-VT-model lift. SL-170 PHASE-01 becomes
  IDE-008's substrate. Related (`enables`), not subsumed — IDE-008's solo baseline
  hits the binding family (IMP-175 etc.) that dispatch's clean pre-spawn B avoids.

**Forward link (deliberate, not scope):** the structured VT row is the join key that
lets SL-057's `verify::resolve` attach a *runnable* check per plan VT, and the
coverage graph already traces VT → phase → slice → REQ. PHASE-01's lift wires the
near end; continuous re-derivation of plan VTs back to originating requirements
becomes reachable (via IDE-008 / SL-057), 20 steps down the road.

## 8. Risks & Mitigations

- **R1 — `parse_failures` brittle to cargo output format.** Mitigation: parse the
  canonical `failures:` summary block (section-aware per target); unit-test against
  captured output fixtures (hermetic strings, not live runs — SL-168 F-2). Crucially,
  an *unrecognised/empty* parse of a run that DID execute is `Unobtainable` → hard
  error (INV-5), never a silent ∅-pass. The dangerous direction (false-green at S) is
  closed by construction, not by hoping the parse degrades safely.
- **R2 — P2 vacuous on legacy/unstructured plans (no completeness pressure).**
  Mitigation: S6 renders `UNCHECKABLE` distinctly at handover, converting absence
  into a visible signal; the `/plan` authoring discipline closes the gap forward.
- **R3 — Carry-forward staleness.** Mitigation: sha-keyed cache — any tree change is
  a new sha = cache miss = honest re-capture; INV-2 forbids the buggy registry source.
- **R4 — Gate cadence skipped (skill forgotten).** Mitigation: deterministic exit
  codes + the funnel cadence contract; the un-skippable `prepare_review` fold remains
  a built-seam follow-up if cadence-trust proves insufficient.
- **R5 — Worker-marker / stale-bin masks both runs.** Mitigation: INV-1 (same tree)
  means such artifacts cancel into `persistent`; the orchestrator clears the marker
  and forces a real rebuild before trusting any bin-shelling test (existing dispatch
  hygiene, mem.pattern.dispatch.claude-arm-isolation-fallback).

## 9. Quality Engineering & Validation

- **S1 unit:** `diff` partitions (new/fixed/persistent; empty cases; full overlap →
  all-persistent → green); the env-mask case (`baseline {embed_fail, X}`, `current
  {embed_fail, X, new_fail}` → `new = {new_fail}` only); `parse_failures` over
  captured cargo fixtures; carry-forward (diff's current-set persisted under B'
  equals a fresh capture at B').
- **S3 unit:** `check_vt` all four verdicts (Pass / Fail-absent / Fail-keyword /
  Uncheckable / Waived-short-circuit); `plan` new fields round-trip; **existing
  `Plan::parse` tests pass unedited** (behaviour-preservation).
- **S6 unit:** `render_summary` over a mixed report (waived reason surfaced;
  uncheckable distinct).
- **S1 unobtainable:** a suite run that does not complete / parses to nothing →
  `Unobtainable` → `diff` errs → non-zero (INV-5), NOT a green ∅. Explicit test.
- **e2e (hermetic fixtures, SL-168 F-2):** inject a failing test into a delta →
  `check regression diff` exits non-zero with it in `new`; a pre-existing failure at
  B → lands in `persistent`, exit zero. `verify-vt` over a fixture plan + fixture
  test files mixing Pass/Fail/Uncheckable/Waived → exit non-zero (the Fail), report
  lists all four with reasons. Conclude/handover emits the VT block.
- **SL-169 replay (the acceptance proof):** reconstruct the two original failures and
  show the gate catches *both*. (a) S1: the `e2e_standard_cli_golden` regressions are
  NEW failing tests → `new` → halt. (b) S3: a VT with `test_file =
  "tests/e2e_list_conformance.rs", keywords = ["relation","census"]` against a tree
  where the conformance matrix is absent → `Fail` (missing keyword) → halt. Either
  alone would have stopped the SL-169 false-green.
- **Dogfood:** SL-170's own `plan.toml` VT rows are the first to carry
  `test_file`/`keywords` — the forward-only P2 surface is exercised on this very
  slice (its VTs verify-vt-clean at its own conclude).

## 10. Review Notes

### Internal adversarial pass (2026-06-28) — findings integrated

- **F-A (critical) — silent ∅ = false-green at S.** A non-completing/unparseable
  suite run at S would yield `current = ∅` → `new = ∅` → false pass. Fixed: INV-5 +
  `FailureSet::Unobtainable` → hard error; `diff` errs on either side unobtainable.
  Mirrors `coverage_verify` F-VII.
- **F-B (critical) — "same tree" insufficient.** A differing `DOCTRINE_WORKER`/marker
  filter changes the test universe, breaking cancellation (absent tests look "fixed",
  new ones "new"). Fixed: INV-1 now requires identical invocation + filter state.
- **F-C (important) — unpinned suite.** The full gate folds clippy/fmt/lint at coarse
  pass/fail granularity (the F-1 problem). Fixed: S1's suite pinned to the *test
  suite, per-test*; aggregate gates are IMP-194 (D4).
- **F-D (important) — two-tree mandate currency.** A mid-dispatch waiver lands on the
  authoring branch, invisible to a coord-tree-only `verify-vt`. Fixed: INV-6 +
  §5.4 propagation requirement.
- **F-E (cadence) — gate vs worktree removal.** `verify-vt` must run before coord
  worktree removal. Fixed: §5.4 cadence made explicit.
- **F-F/G (minor, accepted) — flaky tests and renamed-failing tests** can mis-classify;
  documented as limitations (§5.5 Edges); the report aids diagnosis.
- **Validation strengthened:** SL-169-replay added as the acceptance proof; SL-170
  dogfoods P2 on its own plan (§9).

### Residual (non-blocking)

- `prepare_review` hardening to make S3 un-skippable in the binary (seam built,
  wiring deferred pending cadence-trust signal; §5.4, Follow-Ups).
- IMP-194 finding-granularity generalization to gates/doctor (D4).

### Prime remaining attack surfaces for an external pass

- `parse_failures` section-aware correctness across cargo target boundaries (R1).
- The INV-1 filter-normalisation step — is clearing the worker marker sufficient, or
  are there other selection-affecting env vars on the coord tree?
