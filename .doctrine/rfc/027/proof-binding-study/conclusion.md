# Brief 04 — conclusion: proof bindings and verifier adapters

Study of `scratch/2026-08-08/04-proof-binding-adapters-brief.md`, an external
(GPT) brief in the design/proof pack dated 2026-08-08.

Evidence: `raw/thread-a-ownership.md` + `thread-a-verification.md` (who owns
criterion / binding / evidence today), `raw/thread-b-adapter-boundary.md` +
`thread-b-verification.md` (does the outcome vocabulary distinguish
found / not-found / executed / passed / failed).

Every claim below was checked against primary sources; where a delegated thread
and the corpus disagreed, the corpus won and the correction is in the matching
verification note.

## Recommendation

**Extend the SPEC-002 seam. Do not build a new criterion/binding/evidence
model, and do not reject the abstraction.**

Brief 04 proposes an architecture Doctrine substantially already has. Its own
anti-complexity boundary — what core should know (criterion identity, proof
mode, binding identity, adapter + opaque subject, resolve/execute outcome, Git
anchors, freshness) and what it should not (Rust/pytest/Go/Jest syntax) — is
met by the shipped seam, with four specific exceptions.

## What the brief asked for, against what exists

| Brief 04 asks for | Incumbent | Verdict |
|---|---|---|
| criterion identity distinct from test locator | `CoverageKey` 4-tuple vs `VtCheck` recipe | **already separate** |
| binding revisable without criterion mutation | `check: Option<VtCheck>`, additive and replaceable | **already true** |
| project-declared, language-neutral command registry | `VerificationConfig` + pure `verify::resolve()` | **already exists** (SPEC-002) |
| missing subject ≠ green | `Unobtainable → Blocked`; matcher mandatory *except* literal-command | **partial — see R2** |
| VA/VH freshness distinct from VT | `REQ-115` git-anchor decay; `attested_date`; `Verified` unforgeable | **already true** |
| `unit/integration/system/acceptance` must not drive core | no topology concept exists at all | **satisfied by construction** |
| core must not parse test syntax | `verify.rs` / `coverage_verify.rs` operate on `Vec<String>` argv | **already true** |

## The four repairs that are actually earned

### R1 — Bind a coverage entry to its check definition's digest

`CoverageEntry` persists `check: Option<VtCheck>` **verbatim and undigested**, so
a coverage entry goes stale only when its *evidence's* git anchor ages. Change
the binding — swap the matcher pattern, change the command, retarget the alias —
and the prior `Verified` stands unchallenged.

This is brief 04 § B's correction case, and today it fails.

**The mechanism already exists one subsystem over.** `Step::material()`
(`src/design_run/runbook.rs:324-335`) builds a netstring-framed, version-tagged
digest over a step's whole definition, so any edit makes the discharge stale by
construction. `DEC-101`: *"An id solves reference, not equivalence."*

Positive control: `digest` appears 34× in `runbook.rs`, **0× in `coverage.rs`**.

### R2 — Remove the literal-command exemption from the matcher rule

`coverage::valid` (`src/coverage.rs:542`) requires a non-empty matcher **unless a
literal command is set**. So a literal command that exits 0 having selected zero
tests lands `Verified` — flagged only as `[exit-code-only]` in the report, for a
human to notice.

Brief 04's constraint (*"selecting zero tests must never satisfy an executable
criterion"*) is therefore met for aliased and default-base entries and **not** for
literal-command entries.

**This is the cheapest repair in the study.** Making the matcher mandatory for
every mode closes the hole with no schema change, no new outcome variant, and no
language knowledge in core: the author must state what success looks like in the
output. It is a tightening of one validation function.

It is also the right answer to the tension below.

### R3 — Give `Unobtainable` its cause

`RunOutcome::Unobtainable` fuses at least six conditions — resolve error, spawn
failure, wall-clock timeout, empty/unreadable glob match set, unparseable
hand-edited regex, empty argv — into `CoverageStatus::Blocked`, and
`CoverageEntry` persists no reason, exit code, or captured output. A `Blocked`
cell cannot be triaged without re-running it.

**Again the fix exists nearby**: `FailureSet::Unobtainable { why: String }`
(`src/regression.rs:44`) carries its cause.

### R4 — Fill in `REQ-254`–`REQ-257`

The four SPEC-002 requirements governing this seam are **title-only** — no
statement, no acceptance criteria, in either `spec req show` or the woven
`spec show SPEC-002`. Positive control: `REQ-113`/`REQ-114`/`REQ-115` in the same
spec carry full statements and 3–6 acceptance criteria each.

R1–R3 are not conformance-checkable until this is repaired, and "reuse the
SPEC-002 seam" presently means reusing code governed by requirements that assert
nothing.

## The tension the brief has to own

Brief 04's `H13` says core must not parse test syntax. But **the strong form of
its own safety property requires exactly that.**

You can distinguish "ran zero tests" from "ran and passed" two ways:

- **Parse the runner's output.** Strong. `src/regression.rs` INV-5 does this —
  *"NEVER a silent empty set … so a compile error / panic / format change at `S`
  can never read as 'zero failures = green'. This is the load-bearing inversion
  against the SL-169 ship-as-env regression."* It greps `test result:` and
  `running `, so it is cargo-bound — the thing `H13` forbids in core.
- **Require a matcher.** Language-neutral, and what `coverage.rs` does — but
  currently exempts literal commands.

**R2 resolves this without choosing.** A mandatory matcher pushes the
language-specific knowledge out of core and into project-authored config, where
`REQ-257` (project-agnostic command contract) already says it belongs. The
project declares `test result: ok` or `passed=` or whatever its runner emits;
core stays neutral and still refuses to call an unmatched run green.

So `H13` survives — but only if the matcher is unconditional.

## The study's actual finding

Across both threads, one pattern recurs four times: **Doctrine solves a problem
well in one subsystem and does not carry it to its neighbour.**

| Property | Has it | Lacks it |
|---|---|---|
| Definition-staleness via digest | `design_run/runbook.rs` | `coverage.rs` |
| Failure carrying its cause | `regression.rs` (`why: String`) | `coverage.rs` |
| Zero-result never reads as green | `regression.rs` (INV-5) | `coverage.rs` |
| Callee cannot self-declare success | `runbook.rs` (`Discharge::verified`) | `coverage.rs` |

The mechanisms exist and are good. They are unevenly distributed. That is a
cheaper problem than the one brief 04 set out to solve, and a different one.

## What this study did NOT establish

Stated plainly so nobody reads more into it than it earned:

- **No cross-ecosystem probe was run** (brief 04 § D). Nothing here demonstrates
  the seam against pytest, Go, or Jest. The language-neutrality claim rests on
  reading the code, not on exercising it.
- **The zero-test trace was reasoned, not executed.** `cargo test <absent>` →
  `Verified` follows from `derive_status`, which is verified; the premise that
  cargo exits 0 on an empty filter selection was not run here.
- **Independent test authorship (§ F) was not investigated.** It is RFC-023
  territory and the brief itself says not to block the core model on it.
- **No adapter prototype was built.** The brief asks for one only if needed to
  establish feasibility; the seam already exists, so it was not.

## Smallest next step

Not a slice. **R2 alone**, as a backlog item: drop the literal-command exemption
in `coverage::valid`, with the existing `MatcherRequired` refusal doing the work.
It closes a live false-green path, needs no schema change, and its cost is
bounded by however many existing entries rely on the exemption — a corpus query,
not a design question.

R1 and R3 are schema changes to a persisted type and should be governed by the
SPEC-002 amendment that R4 requires. R4 should precede them.

## Relation to brief 03

Brief 03's obligation-graph study (`../obligation-study/`) found RFC-027's `H9`
unsupported — obligation-level dependency does not move the actionable frontier.
It also surfaced that `EX` criteria already are the obligations, and that no
authored field links a `VT` row to the `EX` row it proves.

That missing `EX`→`VT` link is the same defect as R1 seen from the other end:
the criterion and its binding have no shared identity. Both studies converge on
it independently.
