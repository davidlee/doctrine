I now have all evidence needed. Let me compile the research brief.

---

# Research Brief: How Verification Is Owned in Doctrine

## Answer

**Criterion identity** lives in `src/plan.rs` as `VerificationCriterion.id` (`VT-1`, `VA-1`, `VH-1`), with mode encoded in the prefix — there is no separate mode field. **What must be demonstrated** lives in `VerificationCriterion.expects` (free prose, heterogeneous by design) alongside optional structured fields `test_file`, `keywords`, `patterns`. **Proof binding** is currently two separate things that do NOT meet: (a) for the VT existence/shape gate (`vtgate.rs`), the binding is `test_file` + `keywords` + `patterns` — pure text inspection, never execution; (b) for the coverage substrate (`coverage.rs`), the binding is a `VtCheck` recipe (alias XOR command + matcher) that the verifier (`coverage_verify.rs`) actually runs. The **executable command identity** lives in `src/verify.rs::VerificationConfig` — a project-declared `command`, `aliases`, and `default_source`. **Expected result** lives in `MatchSource` + `Matcher` (stdout/stderr/file glob with pattern and regex flag). **Git anchor, result, output digest** are on `CoverageEntry`: `git_anchor: String`, `status: CoverageStatus`, and captured output is in-memory only (not persisted on the entry).

**VT vs VA/VH**: VT evidence is continuously re-derivable (re-run the check); VA/VH are point-in-time attestations that decay via the memory git-anchor staleness seam (`IsStale`). VT records lean `Planned` at record time until the verifier runs; VA/VH carry an `attested_date`. **What treats mere string presence as proof**: `vtgate.rs::check_vt` — the plan-phase VT gate. It NEVER executes anything; it checks raw substring presence of `keywords` and line-anchored `patterns` regex matches over unstripped source, gated by whether the file appears in the slice's `modified_files` set. **Already sufficient**: the coverage `VtCheck` + `VerificationConfig` + `coverage_verify::run` pipeline IS a complete criterion→binding→evidence model — it resolves, runs, captures output, evaluates matchers, and persists `CoverageStatus`. The plan-phase `vtgate` is a lighter, cheaper gate operating on source text alone, and the two currently have no shared identity wiring.

## Section A: Fact-Ownership Table

| Fact | File | Type/Field | Notes |
|---|---|---|---|
| Criterion identity | `src/plan.rs:84-99` | `VerificationCriterion.id` | `VT-1`, `VA-1`, `VH-1` — prefix IS the mode |
| Semantic statement of what must be demonstrated | `src/plan.rs:83-84` | `VerificationCriterion.expects` | Free text; "just prose" in RFC-023 |
| Structured mandate (text presence) | `src/plan.rs:86-93` | `test_file`, `keywords`, `patterns` | Consumed by `vtgate.rs`; NOT executed |
| VT-check recipe (executable binding) | `src/coverage.rs:342-355` | `VtCheck` | `alias` XOR `command`, `extra_args`, `Matcher` |
| Executable command identity (project config) | `src/verify.rs:40-73` | `VerificationConfig` | `command`, `aliases`, `default_source`, `timeout_secs` |
| Pure resolution of check→runnable argv | `src/verify.rs:108-137` | `resolve()` | Base argv precedence: alias → literal command → project default |
| Match source | `src/coverage.rs:408-416` | `MatchSource` | `Stdout`, `Stderr`, `File(glob)` |
| Expected result / matcher | `src/coverage.rs:386-396` | `Matcher` | `source`, `pattern`, `regex` |
| Verdict from run outcome (pure) | `src/coverage.rs:470-487` | `derive_status()` | `Unobtainable→Blocked`, `exit_ok+matched→Verified/Failed` |
| Git anchor on evidence | `src/coverage.rs:54` | `CoverageEntry.git_anchor` | Stamped at record time; re-stamped on VT re-derivation |
| Result/output digest | `src/coverage_verify.rs:84-92` | `RunResult::Ran { stdout, stderr }` | In-memory only; NOT persisted on the entry |
| VT vs VA/VH persistence | `src/coverage_store.rs:101-113` | `record()` | VT→`Planned`+no date; VA/VH→caller status+`attested_date` |
| VT vs VA/VH freshness | `src/coverage.rs:137-147` | `IsStale` | VT re-derivable; VA/VH decay via `commits_touching` |
| VT existence/shape gate (text-only) | `src/vtgate.rs` | `check_vt()` | Substring/keyword/pattern over RAW source; NEVER executes |
| Plan VT shape lint | `src/plan.rs:222-260` | `check_vt_shape()` | Flags `BareTestFile`, `BareKeywords`, `MissingWaiverReason` |
| Closure gate drift predicate | `src/slice.rs:1930-1956` | `undischarged_drift()` | Reads `Composite` + authored `ReqStatus`; refuses on residual drift |
| Per-requirement composite coverage | `src/coverage.rs:149-159` | `composite()` | Pure fold; derived, never stored |
| Drift verdict | `src/coverage.rs:241-276` | `drift()` | Pure; `Coherent`/`Divergent(reason)`/`Indeterminate` |
| Coverage record write path | `src/coverage_store.rs:90-142` | `record()` | Fail-fast: validates + resolves BEFORE any write |
| Coverage verify (execution) | `src/coverage_verify.rs:106-153` | `run()` | Global argv dedup; spawns subprocess; captures stdout/stderr |
| Waived escape valve | `src/plan.rs:91-93` | `waived`, `waived_reason` | Short-circuits before fs read in `vtgate` |

### VT mandate shape in plan.toml vs what vtgate does

The `VerificationCriterion` in `src/plan.rs:84-99` holds:
- `id` — the immutable doc-local handle (mode in prefix)
- `expects` — free-text expectation
- `test_file` — single repo-relative path (the mandated file)
- `keywords` — `Vec<String>`, literal substrings
- `patterns` — `Vec<String>`, line-anchored regexes
- `waived` / `waived_reason` — escape valve

`vtgate.rs::check_vt` (`src/vtgate.rs:58-108`) applies these in order:

1. **Waived short-circuits first** — returns `Waived` before any fs read
2. **No `test_file`** → `Uncheckable` (nothing to grep)
3. **File doesn't read** → `Fail` (missing)
4. **File NOT in `modified_files`** → `Unattributable` (keyword signal predates slice work)
5. **Keyword absent** from raw source, or **pattern matches no line** → `Fail`
6. Otherwise → `Pass`

**It NEVER executes anything.** No subprocess, no test runner. Raw substring over unstripped source. `patterns` is line-anchored regex via `regex_lite`. Comment/string stripping is explicitly rejected (POL-002: host-language convention). The threat model is worker omission, not adversary.

### What is already sufficient

The **coverage verification pipeline** (`coverage_verify::run` + `coverage_store::record` + `verify::resolve` + `coverage::derive_status`) is a complete criterion→binding→evidence model:

1. **Criterion identity**: `CoverageKey` = `(slice, requirement, contributing_change, mode)`
2. **Binding**: `VtCheck` → `verify::resolve()` → `Resolved { argv, source }`
3. **Evidence**: `RunResult` → `RunOutcome` → `derive_status()` → `CoverageStatus` + `git_anchor`

This owns the full loop: config, resolution, execution, matcher evaluation, status derivation, persistence, and continuous re-derivation. It is POL-002-clean — the engine knows only `VerificationConfig`; the project declares the actual command.

## Section B: DEC-101 Runbook Discharge Model

### B.1 — How a discharge binds to the definition

A discharge binds `Step::material`'s **SHA-256 digest** (computed shell-side), NOT the step's id. From `src/design_run/runbook.rs:287-297`:

```rust
pub(crate) fn material(&self) -> String {
    let argv = self.verify();
    let mut parts = vec![
        framed(RUNBOOK_STEP_DIGEST_VERSION),  // "runbook-step.v1"
        framed(&self.id),
        framed(&self.text),
        framed(if self.required { "true" } else { "false" }),
        framed(&argv.map_or(0, <[String]>::len).to_string()),
    ];
    parts.extend(argv.unwrap_or(&[]).iter().map(|element| framed(element)));
    parts.concat()
}
```

The encoding is **netstring-framed** (`len:value` per field), version-tagged (`runbook-step.v1`), covering: version, id, text, required flag, verify argv arity and each element. Deliberately NOT separator-joined — `text` can contain any byte, so a separator join would let different definitions encode identically.

The `Discharge` record (`src/design_run/runbook.rs:397-417`) stores: `version`, `runbook`, `step`, `digest`, `revision`, `outcome`, and conditional `reason`/`exit`/`output`.

### B.2 — How a discharge becomes stale

`Runbook::live_discharge` (`src/design_run/runbook.rs:564-576`) checks three conditions:

```rust
held.names(self.key, step.id.as_str())
    && held.version == RUNBOOK_STEP_DIGEST_VERSION
    && &held.digest == digest
```

Any edit to `id`, `text`, `required`, or `verify` changes `Step::material()`, producing a different digest, making the discharge stale. The encoding **is** versioned: `RUNBOOK_STEP_DIGEST_VERSION = "runbook-step.v1"` — changing it later invalidates all prior records visibly (a stale record reports as stale, not silently compared against incomparable bytes).

### B.3 — Attested vs Verified

`DischargeOutcome` (`src/design_run/runbook.rs:382-395`) has three arms, **two** on the wire:

```rust
pub(crate) enum DischargeOutcome {
    Attested,   // agent says it did the work; no check corroborates
    Verified,   // attestation corroborated by a check that exited zero
    Skipped,    // step could not be done, with stated reason
}
```

A caller may say `attested` or `skipped` and may **never** say `verified`. `Verified` is reachable ONLY through `Discharge::verified()` constructor, which requires `exit: i32` and `output: String`. From `src/design_run/runbook.rs:71-73`:

> "A verifier result is exactly the kind of fact Doctrine must derive rather than accept on a caller's word."

A step with no `verify` field cannot produce a `Verified` discharge because `Discharge::verified()` is the only route to that outcome and it requires an exit code — the rendering inherits the constructor's invariant (`src/design_run/runbook.rs:471-485`).

### B.4 — The verify field: argv array, execution, constraint

`Step::verify` is `Option<Vec<String>>` — an **argv array**, never a shell string (`src/design_run/runbook.rs:244-257`):

> "Never a shell string: `{repo_root}` can contain spaces and shell metacharacters, so substituting into a command string turns data into syntax."

The four placeholders are `{slice}`, `{run}`, `{repo_root}`, `{step}` — a closed vocabulary. `interpolate()` (`src/design_run/runbook.rs:97-107`) substitutes each element independently, preserving element count so a value with spaces arrives as one argument.

**Execution is shell-side.** From DEC-101 evidence (`src/design_run/run.rs:87-96`): results enter through `DerivedInput`, never through caller-authored payload. The pure core executes nothing. Validation is owned (`EX-3`): `Runbook::parse` validates the entire domain before anything executes.

### B.5 — Scope fence from Condition vocabulary

From DEC-101's restated consequence (2026-08-02):

> "The runbook does NOT derive the incumbent conditions and MUST NOT. … narrowing a term from an OPEN vocabulary into a CLOSED one is a TYPE ERROR, and that is the whole objection."

The `Condition` enum is payload-free and `satisfies()` is existential (`src/design_run/gate.rs:354`). Runbook steps are an open set — an author adds one and nothing determines which `Condition` it discharges — so mapping into the closed ten cannot be total or stable. Instead, `RunbookStanding` is a **third derived input** beside `ReviewStanding` guarding its own edge (`src/design_run/gate.rs:1603,1626`). It deliberately does NOT join `cumulative_conditions` (`src/design_run/gate.rs:1628-1635`). The scope fence is architectural: the open vocabulary (runbook steps) stays open, the closed vocabulary (Conditions) stays closed, and NO mapping is attempted between them.

### B.6 — Is this reusable as the criterion/binding/evidence model?

**Structurally specific to design-run stage advancement. Not directly reusable, but the encoding pattern IS the precedent.** The reasons:

1. **The discharge is tightly coupled to the stage machine.** `RunbookKey` variants are design-run stages (`Exploring`, `Inquiring`, `Drafting`, `Reviewing`); `RunbookStanding` carries `cursor`, `outstanding`, `stale`, `regressed` — concepts specific to ordered obligation progression with a cursor.

2. **The digest binds TO a step definition, not a verification criterion.** The `Step::material()` encoding covers `id`+`text`+`required`+`verify` — these are obligation-checkbox fields, not verification-criterion fields. A coverage criterion's identity is `CoverageKey` (a different 4-tuple), its definition is `VtCheck` (a different shape), and its evidence is `CoverageStatus` + `git_anchor` (a different record).

3. **The staleness mechanism IS the reusable part.** The netstring-framed, version-tagged digest encoding — bind to definition content rather than id, stale on any field change, versioned for forward compatibility — is exactly the pattern needed for criterion/binding/evidence. The coverage substrate already has `IsStale` as the same concept (fresh/stale via git anchor). The two are different *scopes* (runbook step vs coverage cell) but the same *pattern*.

4. **The executable half IS already reused.** The `doctrine verify` family (`src/commands/verify.rs`) is the runbook's own verifier surface; the coverage verifier (`coverage_verify.rs`) is a parallel verifier surface. Both use exit-code + output contracts. Both are argv arrays, never shell strings. The patterns converge but the surfaces don't share code.

5. **What's missing to unify them:** The coverage `VtCheck` does not currently digest-bind to its own definition. A VT entry's `check` field is persisted verbatim but there is no digest of the check recipe that would go stale when the check definition changes. The `CoverageEntry` carries `git_anchor` for staleness of the *evidence*, not staleness of the *binding*. The runbook's innovation — binding the discharge to the *definition's digest* so an edit to the definition invalidates the discharge — has no coverage-side analogue.

## Section C: Where the Seams Already Are

### C.1 — SPEC-002's executable-verification contract

REQ-254 through REQ-257 (`src/verify.rs` is the anchor, per SPEC-002's source table):

| Requirement | What it guarantees |
|---|---|
| **REQ-254** (FR-007) | "Carry a runnable executable check identity on VT coverage entries" — the `VtCheck` field on `CoverageEntry` |
| **REQ-255** (FR-008) | "Derive observed VT coverage status from a real command run" — `coverage_verify::run()` spawns subprocess, captures output, folds through `derive_status` |
| **REQ-256** (FR-009) | "Provide a production write and withdraw path for observed coverage entries" — `coverage_store::record()` / `forget()` |
| **REQ-257** (NF-004) | "Keep the verification command contract project-agnostic" — `VerificationConfig` in `src/verify.rs`, POL-002-clean: engine knows config shape, project declares the actual command |

The project-declared command registry IS `VerificationConfig` in `src/verify.rs:40-73`:
- `command: Option<Vec<String>>` — project-default base argv
- `aliases: BTreeMap<String, Vec<String>>` — named base argvs
- `default_source: Option<MatchSource>` — default matcher source
- `timeout_secs: Option<u64>` — run timeout

Resolution is pure: `verify::resolve()` (`src/verify.rs:108-137`) folds config + check into `Resolved { argv, source }`.

### C.2 — Zero-test selection vs "passed"

**Yes, this is distinguished.** The critical safety property is maintained in `coverage_verify::run()` via `RunOutcome::Unobtainable` → `Blocked`:

From `src/coverage.rs:470-487`:
```rust
pub(crate) fn derive_status(outcome: &RunOutcome) -> CoverageStatus {
    match outcome {
        RunOutcome::Unobtainable => CoverageStatus::Blocked,
        RunOutcome::Ran { exit_ok: false, .. }
        | RunOutcome::Ran { exit_ok: true, matched: Some(false) } => CoverageStatus::Failed,
        RunOutcome::Ran { exit_ok: true, matched: None | Some(true) } => CoverageStatus::Verified,
    }
}
```

However, **this only applies to the coverage verifier, not vtgate.** The vtgate (`src/vtgate.rs`) is a text-inspection gate — it has no concept of "zero tests selected" because it never runs anything. It only checks text presence. A keyword in a comment satisfies it.

The zero-test-selection problem specifically: if a coverage VT check resolves to `cargo test mymod` and `mymod` has zero tests, `cargo test` exits 0 with "running 0 tests". With NO matcher, `exit_ok: true, matched: None` → `Verified`. **This IS a real gap in the current model.** The `Matcher` is the defence — an entry without a matcher on an alias/default-base is rejected by `coverage::valid()` (MatcherRequired, `src/coverage.rs:502-513`), but a literal `command` with no matcher is accepted and exits 0 with zero tests → `Verified`. This is DESIGNED: the exit-code-only path (`src/coverage_verify.rs:155` flags it `[exit-code-only]`) is surfaced for audit but not blocked.

### C.3 — Git anchor recording and invalidation

**Recording**: `coverage_store::record()` (`src/coverage_store.rs:117`) stamps `git_anchor: git::head_sha(root).unwrap_or_default()` at record time. `coverage_verify::run()` (`src/coverage_verify.rs:148`) re-stamps on a `Ran` outcome; `Unobtainable`/`Blocked` keep the prior anchor (F-VIII).

**Invalidation**: `IsStale` (`src/coverage.rs:137-147`) is produced by `coverage_scan` from `git::commits_touching` against `CoverageEntry.touched_paths`. The shell resolves staleness once per scan; the pure folds (`composite`, `drift`) receive `(CoverageEntry, IsStale)` cells with staleness already resolved. A stale attestation is **surfaced, never auto-demoted** (REQ-115/NF-002).

## Unknowns / Low Confidence

1. **The zero-test-selection gap is real but its severity is unclear.** The exit-code-only path exists by design (D3/A — literal command with no matcher). Whether this is acceptable depends on whether the project's `cargo test` selects tests or is a global run. The matcher-required rule for aliases/default-base closes the gap for those paths; the literal-command path trusts the author.

2. **REQ-254–257 have empty acceptance criteria in SPEC-002.** The requirements exist as active FR/NF entries but their body content (beyond the title) is not populated. This may indicate they were created as placeholders for the SL-057 work which shipped, but the spec wasn't backfilled.

3. **The coverage VtCheck does not digest-bind to its own definition** — I cannot find any mechanism that would make a coverage entry go stale when its `check` recipe changes (as opposed to when the evidence's git anchor goes stale). This is an inference from the absence of a `VtCheck` digest field on `CoverageEntry`.

4. **Whether `doctrine slice verify-vt` has a consumer in the worker commit gate** — I did not trace the full call chain through `worker_commit`. I can see it exists as a CLI verb and is used in plan hardening, but whether it fires automatically at commit time is unverified.

## Positive Controls Run

| Absence Claim | Control Search | Result |
|---|---|---|
| "vtgate never executes a test" | `grep -n 'Command::new\|spawn\|subprocess\|std::process' src/vtgate.rs` | 0 matches — confirmed. No spawn, no subprocess, no Command in vtgate.rs |
| "No digest of VtCheck on CoverageEntry" | `grep -n 'digest\|fingerprint\|material' src/coverage.rs` | Only match is `RoundTrip` test; no digest/make_digest/material method on `CoverageEntry` or `VtCheck` |
| "vtgate only does text inspection" | `grep -n 'contains\|is_match\|regex\|lines()' src/vtgate.rs` | `contains()` at line 105, `lines().any(\|line\| re.is_match(line))` at line 112 — confirmed substring + regex over raw source |
| "FAMILIES includes verify" | `grep -n '"verify"' src/commands/cli.rs` | Line 934 — confirmed in FAMILIES |
