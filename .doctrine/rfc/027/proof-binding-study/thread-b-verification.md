# Brief 04, thread B — verification note

Verification of `raw/thread-b-adapter-boundary.md` (pi-research, 2026-08-08)
against primary sources. The raw file is kept verbatim; corrections live here.

Question: does Doctrine's existing outcome vocabulary already distinguish
`found | not-found | executed | passed | failed`, or does it conflate them?

## Verdict

**Sound, and the best-evidenced of the four delegated threads.** It ran positive
controls throughout, reported them, and stated its limits honestly. Every
structural claim I checked held. **One material omission** — it missed the module
where Doctrine already solved the problem — which changes its concluding
judgement rather than its findings.

## Confirmed

- **`Unobtainable` fuses many causes into one.** Construction sites in
  `src/coverage_verify.rs`: resolve error (`:164`), `RunResult` passthrough
  (`:317`), empty-or-unreadable glob match set (`:340`), unparseable hand-edited
  regex (`:350`), empty argv (`:410`), spawn failure (`:419`), plus wall-clock
  timeout. All collapse to `CoverageStatus::Blocked`.
- **Nothing distinguishes them post-hoc.** `CoverageEntry` (`src/coverage.rs:55-74`)
  carries no reason, exit code, or captured output. `CoverageStatus`
  (`src/requirement.rs:141`) is exactly `Planned | InProgress | Verified |
  Failed | Blocked` — verified.
- **No test-topology concept exists.** Its positive control is good: `topology`
  matches 100+ times in `src/`, all git/worktree. So brief 04 § E's prior — that
  `unit|integration|system|acceptance` should not drive core machinery — is
  satisfied by construction today. Nothing to remove; only something not to add.
- **No `resolve(subject)` seam exists.** `verify::resolve()` resolves the
  *command* (argv + match source), never the *subject*. A subject resolver would
  be new, and `VerificationConfig` has no field that could carry one.
- **Production code is language-agnostic.** Its control (`pytest|jest|vitest|go
  test|mocha|rspec` → zero in `verify.rs` and `coverage_verify.rs`) holds. I
  separately checked `DEFAULT_REGRESSION = ["cargo", "test", "--no-fail-fast"]`
  (`src/verify.rs:161`) and it is **not** a POL-002 breach: each baked default is
  annotated *"Pure data — a host convention that informs (POL-002),
  client-overridable."* Mechanism is neutral; only convenience defaults name a
  runner.

## Material omission — the problem is already solved, in `src/regression.rs`

The thread concludes (§ B.3, and Limit 3):

> "There is **no test-count check, no zero-tests-run detection**, no post-hoc
> audit trail."
> …
> "The 'zero tests' detection would depend on the test runner's output format,
> which **is** language-specific. Doctrine's current language-agnosticism would
> break if it tried to parse test counts from output."

True **of the coverage subsystem**. False of Doctrine. `src/regression.rs`
already does exactly this, deliberately, as a named invariant:

> **INV-5** — *"A suite run yields EITHER a well-formed `FailureSet::Obtained` OR
> an `Unobtainable` marker — NEVER a silent empty set. A non-completing /
> unparseable run is `Unobtainable`, and `diff` returns `Err` if either side is
> `Unobtainable`, so a compile error / panic / format change at `S` can never
> read as 'zero failures = green'. This is the load-bearing inversion against the
> SL-169 ship-as-env regression."*

`parse_failures` (`src/regression.rs:111-120`) refuses `Obtained(∅)` unless the
output carries recognisable `cargo test` structure — it greps `test result:`,
`\nrunning `, `running `. And `FailureSet::Unobtainable { why: String }`
(`:44`) **carries its cause**, which is precisely the field `CoverageEntry`
lacks.

So Doctrine has, in one module, both properties the coverage seam is missing —
and paid the language-specificity price to get one of them.

## The real tension for brief 04

Not "can this be built" but **which of two incompatible guarantees to buy**:

| Approach | Where it lives | Strength | Cost |
|---|---|---|---|
| Parse runner output for structure | `src/regression.rs` INV-5 | Strong — zero-result cannot read as green | Language-bound; brief 04 § H13 forbids it in core |
| Require a matcher | `src/coverage.rs::valid` | Language-neutral | Weaker — literal-command entries are **exempt**, so exit-code-only cells go `Verified` |

The exempt case is the live hole. `derive_status` (`src/coverage.rs:481`) maps
`Ran { exit_ok: true, matched: None } => Verified`, so a literal command that
exits 0 having selected nothing lands `Verified`, flagged only as
`[exit-code-only]` in the report for a human to notice.

**A third option the thread did not consider**, and the cheapest: make the
matcher mandatory for *every* mode by removing the literal-command exemption.
That closes the hole with no new schema, no language knowledge in core, and no
new outcome variant — the author must state what success looks like in output.
It is a tightening of `coverage::valid`, not an architecture.

## Note on the zero-test trace

The thread's `cargo test some_absent_name → Verified` walk is **reasoned from
`derive_status`, not executed**. The reasoning is sound and the mapping is
verified, but the empirical premise (cargo exits 0 on a filter matching no
tests) was not run here. Treat as high-confidence, not measured.

## The pattern across this study, now four-for-four

Doctrine repeatedly solves a problem well in one subsystem and does not carry it
to its neighbour:

| Property | Has it | Lacks it |
|---|---|---|
| Definition-staleness via digest | `design_run/runbook.rs` | `coverage.rs` |
| Failure carrying its cause | `regression.rs` (`why: String`) | `coverage.rs` (bare `Unobtainable`) |
| Zero-result never reads as green | `regression.rs` (INV-5) | `coverage.rs` (exit-code-only) |
| Callee cannot self-declare success | `runbook.rs` (`Discharge::verified`) | `coverage.rs` (exit code is the claim) |

This — not a new criterion/binding/evidence ontology — is brief 04's actual
finding. The mechanisms exist and are good; they are unevenly distributed.
