# Implementation Plan SL-224: Honest dispatch refusals

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Two changes, each surfacing what the existing `conformance::undeclared_paths`
predicate already knows at a new moment (design §5.1). Neither edits the belt or
the classifier — the behaviour-preservation gate holds for `classify_import` and
`check_vt_shape` throughout.

- **PHASE-01 (obj 1)** — the import refusal names its offending paths *and* the
  runnable fix. A new pure leaf formatter, shared by both import arms.
- **PHASE-02 (obj 2)** — `check plan` catches, before dispatch, the plans the
  import belt would later refuse.

## Sequencing & Rationale

The two phases are **file-disjoint** — PHASE-01 touches `conformance.rs`,
`worktree/import.rs`, `mcp_server/dispatch.rs`; PHASE-02 touches `plan.rs`,
`commands/check.rs`. There is **no code dependency** between them: both consume
the pre-existing `conformance::undeclared_paths`, which neither phase edits. The
obj1→obj2 order is narrative (name-the-refusal, then prevent-the-refusal), not a
build constraint — they could dispatch in parallel.

**PHASE-01** leads with the pure formatter (`undeclared_detail`) because it is
the single source both consumers depend on: the CLI `report_undeclared_scope`
refactor and the MCP `import_compose` wiring both call it. Building and unit-
testing the formatter first (VT-1 golden) means the two wiring edits are trivial
and the format has one place to be right. The formatter takes the slice id
because the emitted `selector add <ID> <path>` remediation is otherwise
un-runnable (F1) — this also corrects the latent id-omission bug in the current
CLI hint. The MCP `reason` field already carries the `undeclared-scope` token, so
the formatter stays token-free and layering-clean (F2, leaf cannot see the
engine's `Refusal::token()`).

**PHASE-02** adds a *sibling* fn rather than extending `check_vt_shape` (D3): the
shape check needs only the plan, the coverage check also needs the slice's
selectors, and keeping them separate leaves `check_vt_shape`'s plan-only tests
byte-for-byte green. The empty-selector guard (A3) is the one subtlety — an empty
design-target set must yield an empty result, mirroring the import belt's own
no-op-when-empty scope leg; without the guard `undeclared_paths` would flag every
VT of every not-yet-selector'd slice. `run_check_plan` folds the coverage set
into the same non-zero-exit contract the shape findings already use (D1: hard).

## Notes

- **Verification split (PHASE-01).** VT-1 (pure `undeclared_detail` golden) is the
  load-bearing coverage of the detail format. VT-2 (the `import_compose` wiring)
  is integration, not a pure unit — the git rev-parse/merge-base gather precedes
  `classify_import`, so reaching the refusal needs the existing VT-1 git-fixture
  harness (`dispatch.rs:838`; F3).
- **Non-goals held.** No taxonomy redesign, no auto-widening of scope, no
  false-red/environmental funnel burners (those are SL-225). The refusal stays
  report-and-halt; we make the halt self-explanatory.
