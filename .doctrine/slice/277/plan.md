# Implementation Plan SL-277: Jev relevance trial

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML.

## Overview

Ten phases build `crates/doctrine-jev` bottom-up. PHASE-01 to PHASE-08 are
offline and spend nothing: every test runs against a fake transport. PHASE-09
and PHASE-10 are the live trial, each live step under a cap the user approves.

## Sequencing & Rationale

- **Codec before client (PHASE-01, PHASE-02).** The request and response
  shapes, the outcome table and the Choice invariant are pure and carry most of
  the trial's correctness risk, so they land first with no network code. The
  client then adds only transport, credential, retry and the model pin.
- **One attempt boundary (PHASE-03).** Spend reservation, the egress
  allow-list and the send log all guard the same moment, the send, so they are
  one function built once. RV-415 `F-17`'s criterion sketch (a control-routed
  finding held on the ledger) is transcribed here as PHASE-03/EX-4: its
  placement constraint names the phase that builds the transport boundary and
  send log.
- **Corpus, then sectioner and packer (PHASE-04, PHASE-05).** Both need only
  PHASE-01 and touch different modules, but both add a line to `src/lib.rs`, so
  they run serially by default; running them in parallel costs a trivial merge.
- **Labels after corpus (PHASE-06).** Classifying unreadable against absent
  needs `catalog scan`'s diagnostics, which PHASE-04 parses. The fixture is
  generated and committed here, so later phases test against real labels.
- **Rankers join everything (PHASE-07)**, then the report, eval and probe
  commands (PHASE-08). Everything PHASE-09 runs — the probe, section-as-state
  point scoring and the C-arm subsample — exists and is tested offline first,
  so the live phases operate code rather than write it. PHASE-07 is the
  largest phase; its parts share one candidate type, so it stays whole. The
  dry-run plan at the end of PHASE-08 replaces the design's pre-plan cost
  estimate before any money is spent.
- **Live last, split in two (PHASE-09, PHASE-10).** The probe must calibrate
  the estimator and fix how a context-limit 422 is recognised (the docs don't
  describe one), and the agreement check picks batched or section-as-state
  scoring, before the full eval spends its budget. The label spot-check sits
  in PHASE-10 so it reviews the committed fixture just before it is used.

## Notes

- **Plan-level refinement of design sec-7:** the crate is lib + bin
  (`src/lib.rs` plus the `main.rs` shell) because the design's `tests/` suites
  cannot link a bin-only package (the E0433 limit noted in
  `crates/doctrine-control/Cargo.toml`). The rubric constants get their own
  `src/rubric.rs`; the design fixes their being one module, not its name.
- **Test placement:** pure modules carry `#[cfg(test)]` suites in the unit
  under test; filesystem- or git-heavy behaviour (egress, snapshot, the full
  fake run) lives in `tests/`.
- **Lint scope:** `just jev-check` runs clippy without `--all-targets`, as the
  root does, so test-code `unwrap` is not denied.
- **The TooLong recogniser** is provisional through PHASE-08: one function over
  one named constant, exercised by the fake. PHASE-09/EX-1 replaces the
  constant with the body a real context-limit 422 returns.
- **Reproducibility (PHASE-10/VA-1)** is over gitignored runtime state, so the
  EVD body records the run id, `run.toml` values and a report hash for an audit
  to re-derive.
