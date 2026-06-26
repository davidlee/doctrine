# Implementation Plan SL-157: Checkout-independent integrate

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-157 is a deletion: strip `advance_pure_ref`'s speculative post-CAS
re-probe/resync (`dispatch.rs:1842-1848`) and the machinery only it uses
(`resync_worktree_hard`, `Disposition::RacedDesync`). The not-checked-out (trunk)
advance becomes pure CAS-and-done. The checked-out (`edge`) leg and the M4 dirty
pre-gate stay; ADR-012 D4's CAS contract is preserved in full. The full deletion
map, governance impact, and verification posture are in `design.md` (§3-§6) — this
plan does not restate them.

## Sequencing & Rationale

**One phase, one commit — by necessity, not preference.** The three deletions are
mutually load-bearing: remove the resync call and `resync_worktree_hard` is an
unused fn; remove the `RacedDesync` producer and the variant is unused — either
intermediate state fails `cargo clippy` (`dead_code`), which the gate forbids. So
the resync strip, the fn retirement, and the disposition retirement land together
or not at all. There is no coherent partial state to checkpoint, hence no second
phase.

**TDD shape is "green stays green," not red→green.** This is behaviour-preserving
under the supported worktree posture (design §1, §3): the existing integrate-safety
e2e suite is the regression proof and must pass unchanged (EN-2 captures it as a
baseline; VT-1 as the exit gate). The only test removed is the unit test for the
deleted fn. No new test is added — the removed arms were unreachable under the
operating-model invariant, and pinning their behaviour would pin the hazard
(design §6). The "refactor" beat of red/green/refactor is the doc-comment cleanup
(EX-3) — trimming the three stale comments that still describe the resync.

**The risk this plan actively guards is over-deletion.** EX-2/EX-3/EX-4 are
written as keep/remove pairs precisely so execution does not delete the
load-bearing neighbours: `AdvancedResynced` (still the checked-out leg's
disposition), `ff_advance_in_worktree`, the M4 gate, the helper
`main_at_c1_with_descendant_c2` (3 surviving callers), and `report_integrate`'s
body (no structural change — `RacedDesync` rode its catch-all arm). VA-1 is the
human/agent check that the diff is exactly the resync machinery and nothing
adjacent.

## Notes

### Reconcile / close obligations (out of plan scope, do not lose)

- **SPEC-022 prose edit.** `spec-022.md:140-141` carries the resync parenthetical;
  strip it via a `modify` REV (`--target SPEC-022`) at **reconcile**, after the
  code lands — so the spec never leads the code (design §5). The `.toml`
  responsibility already conforms.
- **IMP-122 closure.** Its F-1/F-2 resync hardenings target the exact deleted code;
  close IMP-122 at slice close (design §7). Tracked here so close picks it up.
- No ADR-012 Revision (D4 preserved); SL-121 design §2.2 superseded at the slice
  level (design §5).

### Execution guidance (for /phase-plan → /execute)

- **Navigate by symbol, not line.** The line numbers in plan/design are accurate as
  of authoring (re-verified — no drift) but the repo is multi-agent; locate
  `advance_pure_ref` / `resync_worktree_hard` / `RacedDesync` by name. Import-clean:
  `advance_pure_ref` keeps only `git::update_ref_cas`; `tree_clean` /
  `worktree_for_ref` stay used (M4 gate + branch point), so no `use` edits.
- **Fork from a clean base.** The working tree carries cross-agent uncommitted
  files; EN-2's baseline gate must run on a fork from a **committed, green** base
  (main, promoted from edge per AGENTS.md), not dirty edge — else the baseline is
  not a true regression anchor.
- **`advance_pure_ref` inline/rename = NON-GOAL.** Post-deletion the fn is a thin
  ~6-line sibling of `advance_checked_out`; leave it as its own fn (leg symmetry).
  `AdvancedResynced` becomes checked-out-only but its report label
  `"advanced+resynced"` is test-pinned (e2e:1141) — do **not** rename the label
  (behaviour change). A variant rename without a label change is possible but is
  scope creep; if it seems worth it, raise via `/consult`, don't fold it in.
