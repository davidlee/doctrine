# Implementation Plan SL-041: Resolve branch-point-check --base in the impure shell

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

One bugfix on one surface (`src/worktree.rs`), so one phase. The change lifts ref
resolution for both compare operands into the impure shell and leaves the pure
`matches` leaf untouched — the behaviour-preservation gate is the unchanged leaf
unit test plus the existing e2e suite staying green.

## Sequencing & Rationale

No phase split: the helper, the shell rewrite, and the new VT rows are one
indivisible unit — the tests can't go red-then-green against a half-applied
change, and splitting "add helper" from "use helper" would leave a dead-code
intermediate. TDD within PHASE-01: write VT-2/VT-3/VT-4 red against the current
verb, add `resolve_commit` + rewrite the shell to green, then refactor
(doc-comment, naming) with the leaf gate (VT-1) and existing e2e (VT-5) held
green throughout.

## Notes

- `/dispatch` already passes a resolved sha; `resolve_commit` is the identity on
  that input, so the shipped funnel contract is unchanged (design §3).
- Diagnostics now print resolved shas, not raw input — a deliberate clarity gain,
  not a regression; no golden pins the exact wording.
