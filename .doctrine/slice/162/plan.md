# Implementation Plan SL-162: Runtime-resolve test binary path

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Two phases: prove the resolver in isolation, then sweep the 59 call sites and
lock the result with the generalised guard. The split de-risks the mechanical
sweep — PHASE-01 proves `doctrine_bin()` resolves an existing executable before
any of the 59 files are touched, so a PHASE-02 regression points at the sweep,
not the resolver. Design canon: `design.md` §5 (resolver, wrapper, guard),
decisions D1–D5, RV-169 (inquisition clean).

## Sequencing & Rationale

**PHASE-01 before PHASE-02 — foundation before sweep.** The wrapper `bin()` in
every test file calls `common::doctrine_bin()`; that symbol and its
`#![allow(dead_code)]` re-export must exist and be proven first. PHASE-01's
constructor test (VT-1, = slice VT-4 / IMP-185) is the one resolver check that is
*in-namespace verifiable*: it asserts the returned path exists and is executable
in whatever namespace runs it — closing the gap left by VH-1, which only the
cross-namespace human run can prove.

**Guard generalisation lives in PHASE-02, not PHASE-01.** The generalised guard
bans `env!("CARGO_BIN_EXE…")`; until the 59 files are swept they still carry it,
so the guard is red. A phase must end green (AGENTS.md), so the guard change is
sequenced *with* the sweep that makes it pass — proper red→green inside PHASE-02:
generalise guard (red) → sweep 59 files → green. The guard is then the standing
regression lock (INV-1).

**Behaviour-preservation gate is the sweep's proof.** PHASE-02 changes only how
the bin path resolves, never CLI args or output. VT-1 holds every golden
byte-identical and every suite green under `just gate`; any transcription error
across 59 files surfaces as a spawn NotFound or a golden diff. This is the
ADR-/AGENTS-mandated proof for shared-machinery change.

## Notes

- **Call-site shape (D2):** per file — add `mod common;` where absent (55 of 59),
  add `fn bin() -> std::path::PathBuf { common::doctrine_bin() }`, delete the
  `const BIN` line, swap `Command::new(BIN)` → `Command::new(bin())`. Uniform and
  greppable so the sweep is auditable.
- **Lint (D5 / R4):** `#![allow(dead_code)]` on `tests/common/mod.rs` covers the
  subset-use case (55 files use only `doctrine_bin`, not `repo_root`). Confirm
  clippy-clean in PHASE-01; widen to `unused_imports` only if the re-export itself
  is flagged.
- **Guard comment-skip (INV-1 / F2):** the `doctrine_bin` doc-comment names the
  banned macro in prose; it must stay a doc-comment (`///`/`//!`), never code, so
  the guard — which scans `src/` too and skips `//`-prefixed lines — does not
  self-trip.
- **Follow-up:** IMP-185 (the constructor test) is delivered here as PHASE-01
  VT-1, not deferred — it strengthens in-namespace verification at no extra cost.
