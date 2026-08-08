Now I have everything. Here is the research brief.

---

# Research Brief: Does Doctrine's outcome vocabulary distinguish found / not-found / executed / passed / failed?

## Answer

**No — Doctrine's existing outcome vocabulary conflates several critical distinctions.** The `RunOutcome` enum has two variants: `Unobtainable` (all failure-to-obtain fused into one) and `Ran { exit_ok, matched }` (run-to-completion). Persisted `CoverageStatus` has five values: `Planned`, `InProgress`, `Verified`, `Failed`, `Blocked`. The load-bearing safety property — that a missing subject or zero-test selection must never be indistinguishable from success — is **violated** today: `cargo test some_absent_name` (exits 0, runs 0 tests, no matcher) lands as `Verified`.

`Unobtainable` fuses at least five distinct conditions into `Blocked` with no way to distinguish them post-hoc: unresolved alias, missing binary/spawn failure, wall-clock timeout, empty `File(glob)` match set, and unparseable regex. There is no "not-found" status distinct from "could not run." Captured stdout/stderr are in-memory only and discarded after matcher evaluation — never persisted on `CoverageEntry`. The core is language-agnostic in production code; a `resolve(subject)` step would be entirely new, as would a resolver-command field on `VerificationConfig`. No test-topology concept exists in the codebase.

## Evidence

### A. The outcome vocabulary as built

**A.1 — `RunOutcome` definition and every construction site**

`src/coverage.rs:461-474`:
```rust
pub(crate) enum RunOutcome {
    /// The check could not be obtained/run at all (unresolved alias, spawn
    /// failure, timeout — the F-VII framing). Yields [`CoverageStatus::Blocked`].
    Unobtainable,
    /// The check ran to completion. `exit_ok` is the exit-code verdict; `matched`
    /// is the matcher verdict (`None` ⇒ no matcher present ⇒ exit-code-only).
    Ran {
        exit_ok: bool,
        matched: Option<bool>,
    },
}
```

**`Unobtainable` is produced at FIVE distinct sites** in `src/coverage_verify.rs`:

1. **Unresolvable check** — `run()` line ~163: `Err(_) => RunOutcome::Unobtainable`
2. **Spawn failure / empty argv** — `run_argv()` lines ~392-394:
   ```rust
   let Ok(mut child) = spawned else {
       return RunResult::Unobtainable;
   };
   ```
3. **Timeout** — `run_argv()` lines ~418-419:
   ```rust
   reap(&mut child, oh, eh);
   return RunResult::Unobtainable; // wall-clock timeout
   ```
4. **Empty/absent `File(glob)` match set or containment violation** — `outcome_for()` line ~278:
   ```rust
   None => return RunOutcome::Unobtainable,
   ```
5. **Unparseable regex** — `outcome_for()` line ~283:
   ```rust
   None => RunOutcome::Unobtainable,
   ```

These five code paths all produce the identical `RunOutcome::Unobtainable` → `CoverageStatus::Blocked`. **The persisted record (`CoverageStatus::Blocked`) cannot distinguish which of these five caused it.** There is no reason field, no error detail persisted on `CoverageEntry`.

**A.2 — Persisted record**

`src/coverage.rs:57-77` — the full `CoverageEntry` struct:
```rust
pub(crate) struct CoverageEntry {
    #[serde(flatten)]
    pub(crate) key: CoverageKey,
    pub(crate) status: CoverageStatus,
    pub(crate) git_anchor: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) attested_date: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) touched_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) check: Option<VtCheck>,
}
```

There is **no field** for stdout, stderr, exit code, error reason, or any diagnostic payload. `RunOutcome` has **no serde derives** (confirmed: `grep` for `Serialize.*RunOutcome|Deserialize.*RunOutcome` in `src/coverage.rs` returned zero matches). The captured `stdout: Vec<u8>` and `stderr: Vec<u8>` in `coverage_verify.rs`'s private `RunResult::Ran` are consumed by `outcome_for()` for matcher evaluation and then **dropped** — never stored, never persisted.

Positive control: the grep for `stdout|stderr|captured_output` in `src/coverage.rs` matched only `MatchSource::Stdout`/`MatchSource::Stderr` enum variants (the haystack *source* selection), not any storage of captured output.

### B. The not-found gap

**B.1 — No "subject does not exist" representation**

There is no `CoverageStatus` variant for "not-found" or "absent" distinct from `Blocked`. The `Blocked` status means "evidence unobtainable" — which could be a config error, a missing binary, a timeout, or a missing file. There is no way to encode "the test name is valid but the test does not exist in the test suite."

**B.2 — Concrete trace: `cargo test some_absent_name` with no matcher**

Walk through `src/coverage_verify.rs::run()`:

- `run_argv` spawns `cargo test some_absent_name`. Cargo exits 0, stdout contains "running 0 tests". `run_argv` returns `RunResult::Ran { exit_ok: true, stdout, stderr }`.
- `outcome_for` sees no matcher (`check.matcher.is_none()`) → returns `RunOutcome::Ran { exit_ok: true, matched: None }`.
- `derive_status` (`src/coverage.rs:481-496`):
  ```rust
  RunOutcome::Ran {
      exit_ok: true,
      matched: None | Some(true),
  } => CoverageStatus::Verified,
  ```
  **Result: `Verified`.** This is a false positive — zero tests ran, zero assertions checked.

**B.3 — Reporting surface**

`src/coverage_verify.rs:233-256` — `print_report`:
```rust
writeln!(
    out,
    "{}/{}/{}/{}: {old}→{new}{flag}",
    e.key.slice, e.key.requirement, e.key.contributing_change, e.key.mode,
)?;
```

The only signal is the `[exit-code-only]` flag (for literal-command cells with no matcher). For `cargo test some_absent_name`, the report would print:

```
SL-057/REQ-200/SL-057/VT: planned→verified [exit-code-only]
```

There is **no test-count check, no zero-tests-run detection**, no post-hoc audit trail. A human reading `verified [exit-code-only]` has no way to know whether one test or zero tests ran.

### C. Adapter boundary fit

**C.1 — Language-agnosticism**

Production code in `src/verify.rs` and `src/coverage_verify.rs` contains **zero** language- or framework-specific facts. The `cargo test` references are:
- `DEFAULT_REGRESSION` at `src/verify.rs:161` — a project-level default, overridable in `doctrine.toml`
- All other `cargo` hits are in `#[cfg(test)]` blocks or doc comments

Positive control: grep for `pytest|jest|vitest|go test|mocha|rspec` in both files returned zero matches. The production logic operates purely on argv `Vec<String>`.

**C.2 — No existing `resolve(subject)` seam**

The existing `verify::resolve()` (`src/verify.rs:119-150`) resolves the **command** (`argv` + `MatchSource`), not the **subject** (test name). A "does this test exist?" step answered before execution would be entirely new — there is no existing seam for it.

**C.3 — `VerificationConfig` schema**

`VerificationConfig` (`src/verify.rs:40-70`) declares: `command`, `default_source`, `timeout_secs`, `aliases`, `quick`, `commit`, `gate`, `regression`, `prove`. A resolver command ("given a subject name, does it exist?") would need a **new field** — none of the existing fields carry a resolver-location concept.

### D. Topology vs proof mode

Doctrine carries **no test-topology concept** (unit / integration / system / acceptance). The word "topology" in the codebase refers to **git worktree topology** (linked worktrees, branch structure, `is_linked_worktree`, `worktree_topology` in `src/worktree/jail.rs`, etc.).

Positive control: grep for `topology` in `src/` found 100+ matches — every single one is about git/worktree topology. Grep for `unit.test|integration.test|system.test|acceptance.test` in `src/` found zero matches as a test classification scheme. The proof-mode system (`VT`/`VA`/`VH`) is the only classification axis.

## Judgement

**The core design tension is this:** `RunOutcome` is a tri-state (unobtainable / passed / failed) trying to carry a five-state reality (not-found / not-runnable / executed-passed / executed-failed / zero-tests). The `Unobtainable` arm fuses "cannot resolve the command" with "cannot find the subject" with "command ran but found nothing" — and the last of these is actively dangerous because it lands in `Blocked` when it should arguably be a distinct "not-found" or "empty" signal.

The zero-tests-exit-zero case is the sharpest safety violation: it lands `Verified` with no matcher, meaning the only guard is the human reading `[exit-code-only]` in the report. There is no programmatic post-condition checking test count.

The `derive_status` function at `src/coverage.rs:481` is the single point of truth for the mapping. Adding a "zero tests ran" detection would need to happen either at the `RunOutcome` level (a new variant, or a new field on `Ran`) or at the `derive_status` level (a new `CoverageStatus`). Either is a schema change that cascades to `CoverageStatus` (persisted), `ForgetOutcome`, the `drift` decision tree, and the closure gate.

The resolver command is entirely new ground — no schema, no seam, no existing pattern. The closest analogue is `verify::resolve` for commands, but a subject resolver answers a different question (existence, not argv-construction).

## Verdict table

| Distinction | Distinguishable today? | Evidence |
|---|---|---|
| `found` (subject exists) | **No** — indistinguishable from any `Verified` | `derive_status` maps `Ran{exit_ok:true,matched:None}` → `Verified` regardless of test count |
| `not-found` (subject absent) | **No** — lands `Verified` (with no matcher) or `Blocked` (with matcher miss) | `cargo test absent` exits 0 → `Verified`; File glob with no matches → `Unobtainable` → `Blocked` |
| `executed` (check ran) | **Yes** — `Ran` variant exists | `RunOutcome::Ran` vs `Unobtainable` |
| `passed` | **Yes** — `Verified` | `derive_status`: `Ran{exit_ok:true, matched:None\|Some(true)}` |
| `failed` | **Yes** — `Failed` | `derive_status`: `Ran{exit_ok:false}` or `Ran{exit_ok:true,matched:Some(false)}` |
| `unobtainable` (cannot run at all) | **Yes** — `Blocked` | `derive_status`: `Unobtainable → Blocked` |
| *Why* unobtainable | **No** — all five causes fused into `Blocked` | No reason/error persisted on `CoverageEntry` |
| zero-tests-ran vs one-or-more | **No** — both land `Verified` (exit-code-only) | No test-count post-condition check |

## Limits

- I did not trace every call site of `derive_status` beyond `coverage_verify.rs` — there may be other producers of `RunOutcome` I missed. However, the only construction sites for `RunOutcome` are in `coverage_verify.rs` (the shell that runs commands) — there are no other modules that construct `RunOutcome` values.
- I inferred that `RunResult` (the private enum in `coverage_verify.rs` holding captured stdout/stderr) is never persisted from the fact that it has no serde derives and is private to that module, plus the `CoverageEntry` struct lacks any stdout/stderr field. This is a structural guarantee rather than something I traced at runtime.
- The "zero tests" detection would depend on the test runner's output format, which **is** language-specific. Doctrine's current language-agnosticism would break if it tried to parse test counts from output. This is an inherent tension in the "minimal adapter contract" design space.

## Positive controls run

- **Assertion: `RunOutcome` has no serde derives.** Control: `grep` for `Serialize.*RunOutcome\|Deserialize.*RunOutcome` in `src/coverage.rs` matched zero lines. Same grep for `#[derive` on the `RunOutcome` definition confirmed only `Debug, Clone, PartialEq, Eq`.
- **Assertion: `CoverageEntry` has no stdout/stderr field.** Control: `grep` for `stdout\|stderr\|captured_output` in `src/coverage.rs` matched only `MatchSource` variant references (the haystack *source*), zero storage fields.
- **Assertion: No test-topology concept exists.** Control: `grep` for `topology` in `src/` matched 100+ hits, all about git/worktree topology. `grep` for `unit.test\|integration.test\|system.test\|acceptance.test` in `src/` matched zero hits as a test classification scheme.
- **Assertion: No language-specific facts in verify/coverage_verify production code.** Control: `grep` for `pytest\|jest\|vitest\|go test\|mocha\|rspec` matched zero in both files. The only `cargo test` hits in verify.rs outside test code are the `DEFAULT_REGRESSION` constant (overridable) and its doc comment.
