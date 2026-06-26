# Review RV-169 — design of SL-162

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

This Inquisition interrogates the design of SL-162 — its architecture,
its fidelity to the sanctioned doctrine, and its own internal coherence.
The design stands accused of being the sole artifact of a slice that has
not yet been planned or executed. Judgement shall be rendered on its
worthiness to proceed.

### Lines of Interrogation

1. **Doctrinal alignment.** Does the design conform to ADR-001 (module
   layering), ADR-003 (canonical change loop), ADR-009 (lifecycle FSM),
   POL-001 (no clankspeak), and the AGENTS.md conventions?

2. **Design coherence.** Are the decisions (D1–D5) internally consistent?
   Do the invariants (INV-1) hold under scrutiny? Are the risks (R1–R4)
   honestly stated and adequately mitigated?

3. **Factual accuracy.** Does the design's description of the current
   state match reality? Are the counts, file paths, and relationship
   claims verifiable against the codebase?

4. **Scope integrity.** Is the scope cleanly bounded? Do the non-goals
   genuinely stay out of scope, or does the design silently reach beyond
   its own boundaries?

5. **Verification adequacy.** Are VT-1 through VH-1 sufficient to prove
   the design's intent? Are there gaps in the verification net?

6. **Precursor fidelity.** Does the design faithfully extend the
   CHR-014 pattern it claims to follow, or does it deviate in ways
   that introduce new risk?

### Sanctioned Doctrine Held Against the Accused

- ADR-001: leaf ← engine ← command, no cycles
- ADR-003: canonical change loop, audit/reconcile/close seam
- ADR-009: slice lifecycle FSM, conduct axis, no slice-level `review` state
- POL-001: engineering action figure cosplay clankspeak prohibited
- POL-002: platform independence from host-project conventions
- AGENTS.md: TDD, behaviour-preservation gate, DRY, small composable SRP
- `review-ledger.md`: adversarial review protocol

## Synthesis

### Judgement

**No heresy.** The design of SL-162 stands orthodox before the sanctioned
doctrine. It faithfully extends the CHR-014 pattern, respects ADR-001
layering (test-only helper, no cycle), honours the behaviour-preservation
gate (F4), and confines itself to a clean, bounded scope. The prose is free
of clankspeak taint. Two minor findings were raised, disposed, and
discharged — neither rises to blocker.

### Standing Findings Discharged

1. **F-1 (minor, tolerated):** Blanket `#![allow(dead_code)]` on the shared
   test helper module. Discharged as *tolerated* — the module is trivially
   small (two re-exports), the blanket allow is the standard Rust idiom for
   this pattern, and per-item suppression would add ceremony without safety
   gain.

2. **F-2 (minor, follow-up):** No CI-verifiable constructor test for the
   resolver. Discharged as *follow-up* — a test verifying `doctrine_bin()`
   returns an existing executable would add automated defence, but the
   design correctly classifies the cross-namespace proof as VH and does not
   claim otherwise.

### Penance Required Before Execution

None. The design is locked and ready for `/plan`. The execution must honour:

- **INV-1 strictly.** No `env!("CARGO_BIN_EXE")` outside the guard's
  assembled needle. The `doctrine_bin()` doc-comment mention stays in a
  `///` line (skipped by the guard's `//` filter).
- **The 59-file sweep is mechanical.** Every file: add `mod common;` where
  absent (58 files), add `fn bin() -> PathBuf { common::doctrine_bin() }`,
  replace `Command::new(BIN)` with `Command::new(bin())`, delete `const
  BIN`. The behaviour-preservation gate (`just gate`) is the proof of
  fidelity.
- **Guard rename + generalise.** `e2e_no_baked_manifest_dir.rs` →
  `e2e_no_baked_paths.rs`, adding the second assembled-fragment needle for
  `CARGO_BIN_EXE`. Verify both needles scan correctly.

### Standing Risks

- **R4 (dead_code suppression):** Tolerated. The blanket allow is fit for a
  two-line shared module. If common/mod.rs grows beyond the current scope,
  revisit.
- **VH-1 (cross-namespace proof):** Unverifiable in CI. The constructor
  test (verify `doctrine_bin()` returns an executable) is follow-up work
  — capture as backlog.

### Harvest

- Follow-up → backlog: constructor test for `doctrine_bin()` (verify
  returned path exists and is executable, CI-runnable).

The Inquisition finds no taint requiring `/consult` — the design may
proceed to plan. Let none say mercy was shown where heresy was found; let
all say no heresy was found to show mercy upon.

> **HERESIS URITOR; DOCTRINA MANET**
