# Implementation Plan SL-119: Wire boot snapshot to pi via APPEND_SYSTEM.md

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four phases, strictly sequenced. PHASE-01 lays the foundation (sentinel, guard,
rename) — everything else references the sentinel or the renamed harness variant.
PHASE-02 and PHASE-03 are independent of each other (symlink vs extension) but
both depend on PHASE-01 for `Harness::Pi`. PHASE-04 wires them together into
`install_refresh` and `wire()`, and updates the ~20 test call-sites.

The pure/imperative split is maintained throughout: each new mechanism has a
pure decision function (testable without disk) and a thin imperative apply.

## Sequencing & Rationale

### PHASE-01 — Foundation

Sentinel, guard, and harness rename are pure text changes with no new logic.
They establish the vocabulary the rest of the slice depends on:

- The sentinel must exist in boot.md before the AGENTS.md guard can reference it
- The guard expands the `@` reference block — the idempotency check stays on
  the `@` line, unchanged semantics, just larger payload
- `Harness::Codex` → `Harness::Pi` is a mechanical rename touching harness
  detection, label functions, import targets, test fixtures — everything in
  `boot.rs` that references the variant. Grep-driven, no logic changes.

Verification strategy: extend existing unit tests (render_boot sentinel
assertion, plan_boot_import guard-block tests), grep-assert no residual Codex
references, just check green.

### PHASE-02 — APPEND_SYSTEM.md symlink

First pi delivery mechanism. A pure `plan_append_system(root) -> SymlinkAction`
function encodes the four-state decision table; `install_append_system` applies
it (creates `.pi/` dir if absent, creates/replaces/no-ops the symlink).

Independent of PHASE-03 — the symlink and extension are separate concerns. Both
will be called by `install_refresh` in PHASE-04.

Verification strategy: unit tests for all four plan states, dry-run integration
test, VA check on relative path.

### PHASE-03 — Pi session_start extension

Second pi delivery mechanism. `plan_pi_extension(root, exec) -> ExtAction`
generates the candidate TS file content and byte-compares against disk for
idempotency. `install_pi_extension` writes the file, creates the directory as
needed, skips foreign (user-modified) files.

The byte-compare approach avoids parsing the baked exec path from the generated
file — simpler and more robust.

Verification strategy: unit tests for all four plan states, round-trip test
(generate → write → re-plan is NoOp), VA checks on generated file format.

### PHASE-04 — Wiring and integration

The integration phase. Extends `RefreshReport` with two new outcome fields,
wires the `Harness::Pi` arm in `install_refresh` to call both new install
functions, updates `wire()` to report outcomes, and updates all ~20 existing
test `RefreshReport` constructions with `NotApplicable` defaults.

The test call-site updates are mechanical but pervasive — the main risk is
missing one. The grep-driven approach from PHASE-01 (find all
`RefreshReport {` constructions) mitigates this.

Verification strategy: unit test for the pi arm return value, unit test for
Claude arm NotApplicable defaults, integration test for wire() reporting,
full just check pass, e2e `doctrine boot install --agent pi`.

## Notes

- The harness rename (`Codex` → `Pi`) is scoped to `src/boot.rs` only. Other
  files that reference "codex" in comments or CLI help text (e.g. `--agent
  codex` flag description) are out of scope — the CLI accepts both forms via
  case-insensitive matching, so `--agent pi` already works; only the internal
  enum variant name changes.
- PHASE-02 and PHASE-03 are file-disjoint (different functions) but not truly
  parallelizable — they share the `install_refresh` match arm which is wired in
  PHASE-04. Sequential execution avoids merge conflicts.
- All pure functions must remain free of clock/rng/disk/fs calls (the house
  rule). The `plan_append_system` function reads `fs::read_link` and
  `Path::exists` — these are impure. The pure/imperative boundary here is:
  `plan_*` functions take the current filesystem state as explicit inputs (bool
  flags or Option<PathBuf> for symlink target) rather than reaching disk
  internally. The imperative wrapper gathers state and passes it in.
