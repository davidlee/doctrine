# Implementation Plan SL-029: Dispatch worktree creation: detection and creation paths with guards

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Two phases, ordered Rust-before-prose. The load-bearing risk in this slice is the
coordination-tier exclusion guarantee — that a fork can never receive
`.doctrine/state/` and its siblings even under a careless `**` allowlist. That
guarantee lives entirely in `select_copies` and the copy-safety helper, both
testable Rust. PHASE-01 builds and proves it. PHASE-02 is the skill prose that
*drives* the proven verbs; it adds no new correctness-critical machinery, only
orchestration.

## Sequencing & Rationale

**Why CLI first (PHASE-01).** The design's OQ-3 resolution makes `provision` the
*sole* copier — the exclusion guarantee is a property of the copy seam, not the
skill. That seam is where the slice's correctness concentrates and where the
adversarial passes landed (B2 sole-copier, B5 copy-safety, M4 WITHHELD authority,
M6 allowlist subset, M7 candidate enumeration). It must exist and be green under
test before any prose can honestly claim "always provision". The skill in PHASE-02
is a consumer of this contract; building the consumer first would invert the
dependency and leave the guarantee unproven.

**Why a single Rust phase.** The pure core (`WITHHELD`, `parse_allowlist`,
`select_copies`, `allowlist_violations`), the impure `provision` shell, the
`src/fsutil.rs` copy helper, and the `src/main.rs` subcommand are one cohesive
unit: the verbs are not meaningfully testable without the pure core, and the pure
core has no standalone consumer. Splitting them would create a phase boundary with
nothing green to hand across it. TDD red/green/refactor runs naturally within the
phase (pure tests → impure e2e).

**Why the skill is a second phase, not folded in.** PHASE-02 is prose under
`plugins/`, verified by agent read-through and human acceptance rather than
`cargo test`. Different artifact, different verification mode, different failure
surface (skill-source-of-truth — author in `plugins/`, never the gitignored
`.doctrine/skills/`, per B1). It depends on PHASE-01's verbs existing, so it
follows. The `mode = solo | worker` contract is defined here even though only
`solo` is implemented: it is the OQ-1 split seam (§5/F8), so the follow-up funnel
slice reuses it without re-deciding.

**Boundary held against the funnel slice.** Worker-mode implementation, the
import→verify→commit→record funnel, `/dispatch`, and branch-point-under-concurrency
are out of scope (OQ-1 split; IMP-002-dependent). This plan ships standalone solo
value and the reusable seam — nothing here mints ids, so nothing pulls in IMP-002.

## Notes

- Pre-existing dirty working-tree files (`.gitignore`, `install/doctrine.just`,
  `install/manifest.toml`, `justfile`) are not part of this slice — leave them.
- Gate is `just check` (fmt+lint+test+build) before commits; clippy is *plain*
  `cargo clippy` (NOT `--all-targets`, which lights test-only denials).
- The native `WorktreeCreate` rung is unconfirmed-shipped (F1): design around rung
  3 (`git worktree add` + provision), never depend on rung 2.
