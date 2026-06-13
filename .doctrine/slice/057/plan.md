# Implementation Plan SL-057: Formal VT verification: executable check + coverage record surface (SPEC-002 test-run surface)

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Build the deferred SPEC-002 **test-run surface** in five phases that climb the
ADR-001 layering once: two pure leaves first (the verdict and the config
resolution), then two impure shells (the write path and the verifier), then the
CLI that exposes them. The cut follows the dependency grain — each phase depends
only on the ones before it — and keeps the irreducible value (a verdict provable
without a subprocess, design §4) landing first, where it is cheapest to test.

The slice realises the SPEC-002 deltas that lift its "Contracts deferred" line,
authored as requirement members at this plan step: **REQ-254** (runnable check
identity), **REQ-255** (status derived from a real run / continuous
re-derivation), **REQ-256** (production write/withdraw path), **REQ-257**
(project-agnostic verification contract), and a reaffirmation of the existing
**REQ-114** (NF-001 observed-tier confinement — not re-minted). They are wired to
the slice as tier-1 `requirements` relations.

## Sequencing & Rationale

**Why two pure leaves before any I/O.** The design's load-bearing claim (§4) is
that *running* a test is the only proof it is wired — but the *verdict* over a run
is a pure fold, and the *resolution* of a check into an argv is pure too. Landing
both before the shell means the subprocess layer (PHASE-04) is a thin orchestrator
over already-proven folds, not a place where logic and I/O tangle. It also lets
the verdict truth table, the matcher (D8 substring-vs-regex), and the whole
record-time `valid` reject matrix be exhaustively tested with no process or disk.

- **PHASE-01 — the verdict (coverage.rs).** The model (`VtCheck`/`Matcher`/
  `MatchSource`/`RunOutcome`) and the three pure fns: `derive_status` (the verdict
  table, INV-3), `evaluate_matcher` (D8), and `valid` (the record-time fail-fast:
  XOR, the D3/A anti-vacuity matcher rule, F-III glob-confinement, regex-parse).
  Additive on `CoverageEntry`, so old check-less `coverage.toml` files still parse.

- **PHASE-02 — one config reader (dtoml.rs + verify.rs).** The D2 decision: a
  single `doctrine.toml` reader rather than a second parser beside `conduct`'s.
  `conduct::parse` is re-expressed as a delegate, and its untouched-green suite is
  the R2 regression proof. `verify::resolve` turns a check + config into a runnable
  base (alias | literal | default) — the half of validity that `valid` structurally
  cannot decide (F-1: it has no config).

**Why the write path before the verifier.** The verifier *is* a writer — it loads,
re-derives, and saves observed cells. Giving it a tested `coverage_store` to stand
on (PHASE-03) keeps PHASE-04 focused on the run/dedup/derive orchestration rather
than also inventing file persistence. PHASE-03 also delivers `record`/`forget`,
the general observed-tier write/withdraw path (all modes — VT/VA/VH), which the
verifier (VT-only) does not provide.

- **PHASE-03 — the write path (coverage_store.rs).** The first production writer of
  `coverage.toml` (nothing writes it today — confirmed F-5, so this is a new seam,
  not a parallel implementation). `record` composes `valid`+`resolve` before
  writing and injects the clock (F-VI, no hidden `now`); `forget` is the loud
  transient-cell tool (F-IV). The dead_code suppression on `coverage.rs` is
  narrowed here as the first real consumers arrive — blanket retired, per-symbol
  expect kept only on the verdict fns still waiting on PHASE-04 (the blanket-masks-
  siblings lesson).

- **PHASE-04 — the verifier (coverage_verify.rs).** The slice's reason to exist:
  resolve → **global** argv dedup (F-2: per-slice is only the *write* unit, so
  `--all` must not re-run a shared suite once per slice) → one run per argv under
  `cwd=root` with a timeout (F-VII) → matcher eval over stdout/stderr/confined
  `File` globs (F-III/OQ-6) → `derive_status` → re-stamp the anchor *only* on a Ran
  outcome (F-VIII) → save → report (exit-code-only flags + the loud backfill line).
  The NF-001 guard drives `run()` end-to-end against the real write seam (INV-1).

**Why the CLI last.** The subcommand restructure (D4) is the only consciously
breaking change — bare `coverage <ref>` becomes `coverage show <ref>` because clap
cannot disambiguate a bare positional from the new subcommand names. Doing it last
means the breakage is a single, deliberate golden churn at the end, cleanly
separated from the behaviour-preservation gate.

- **PHASE-05 — the CLI surface.** Wire `record`/`verify`/`forget` (args-struct
  handler, R4) and relocate `show`. The phase carries the design's split
  behaviour-preservation claim (F-V): gate **(a)** — the SL-042/044 fold suites and
  the conduct suite stay green *byte-unchanged* — is the preservation proof; the
  **(b)** bare-`coverage` golden churn is conscious and explicitly outside (a). Ends
  on a green `just gate` workspace, zero clippy.

## Notes

- **Dogfood closure is a `/close` action, not a phase.** Once the machinery exists,
  SL-057 can record VT checks for its own requirements and `verify` them green at
  close, replacing the hand-authored backfill (design §9). Kept out of the phase
  plan so a green slice does not depend on an optional self-application.
- **Out of plan scope (design Non-Goals):** historical corpus backfill, "contracts"
  (per-file signature docs), running VA/VH, composite-precedence/multi-mode
  collapse (OQ-3), and the closure-gate-on-live-`Failed` guard (R5 / RSK-008). The
  first is captured as a follow-on at close; R5 is recorded as RSK-008.
- **No code without an approved plan.** Phases flip to `in_progress` only after
  `/phase-plan` expands the next runtime sheet and the design/plan/scope agree.
