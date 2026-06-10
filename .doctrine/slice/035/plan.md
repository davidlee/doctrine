# Implementation Plan SL-035: Record-time stderr nudge for hidden thread memories

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-035 closes a discoverability gap, not a behaviour bug. `thread_expiry`
(SL-008 D6) correctly hides an unverified `thread` from `find`/`retrieve`;
`record` always scaffolds `unverified`, so a freshly-recorded thread is silently
invisible until `verify`d. An agent recording via the raw CLI (no
`/record-memory` skill) gets no signal and may read it as a retrieval bug. The
fix is one non-blocking stderr line at record time.

## Sequencing & Rationale

One phase. The change is a single pure helper plus a few lines of shell wiring
in one file (`src/memory.rs`); it rides an existing seam (the linked-worktree
stderr warning in `run_record`) so there is no scaffolding, no read-path edit,
and no cross-file fan-out to sequence. Splitting it would add ceremony without
isolating any independent risk.

PHASE-01 carries the whole change: author the pure `thread_hidden_notice`
decision/text fn TDD-first (the unit tests are the spec — Thread→Some,
non-thread→None, reference key-vs-uid splice), then wire it into `run_record`
after the success line. Behaviour preservation is by construction: the read path
and `thread_expiry` are untouched, so the SL-008 retrieval suites stay green
unchanged, and stdout's machine-readable success line is preserved (stderr-only
nudge).

## Notes

- Pure/imperative split (ADR-001): the decision and wording live in the pure
  helper; the shell does only IO. Keeps the text testable without a subprocess.
- Lint: emit via `writeln!(io::stderr(), …)` — repo clippy denies
  `print_stderr`/`eprintln!`. `io` is already imported in the module.
- Fires for `--global` threads too — intended; `thread_expiry` keys on kind.
- On close: reconcile slice status to `done` and close backlog IMP-011.
