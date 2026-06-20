# Implementation Plan SL-126: structural close-gate: refuse reconcile→done when dispatched code unintegrated

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Two phases, split by **module responsibility** (and so nearly file-disjoint),
each a clean TDD red/green/refactor unit:

- **PHASE-01 — `ledger` (leaf):** the whole mechanism (design §3.1) — the
  tree-read accessor + the three-state `trunk_integration` resolver + its DRY
  consumer migration in `dispatch::run_show_journal_trunk_oid`. Unit-tested
  exhaustively against a git fixture. Ships independently green; nothing observes
  it yet at the close seam.
- **PHASE-02 — `slice` (command):** the one gate block in `run_status` that calls
  PHASE-01's query, plus behavioural tests over the close transition.

## Sequencing & Rationale

**Why query-before-gate.** The query is the load-bearing, branch-heavy logic
(seven resolution outcomes); the gate is a five-line `match`. Landing and
exhaustively unit-testing the query first (PHASE-01) means PHASE-02 is a thin,
obviously-correct wiring with behavioural coverage — no logic hides in the gate.

**Why the DRY refactor rides PHASE-01.** `dispatch::run_show_journal_trunk_oid`
is the existing trunk-row tree-reader; introducing `read_journal_at_ref` and
migrating that verb in the same phase prevents a parallel journal-read from ever
existing (no-parallel-implementation). The behaviour-preservation gate applies —
the show verb's existing tests must stay green unchanged.

**Layering (ADR-001), already verified in design §4.** PHASE-01 adds `ledger→git`
(leaf→leaf, no cycle — `git ∌ ledger`). PHASE-02 adds `slice→ledger`
(command→leaf, downward). The `slice↔dispatch` cycle is avoided by siting the
query in `ledger`, not `dispatch` (which already imports `slice::read_plan`).
Neither phase touches the `[[accepted_violation]]` baseline or the leaf
`tangle_baseline` (stays 0); `just gate`'s `syn` fitness check passes untouched —
an EX criterion on both phases.

**Verification posture.** All criteria are VT (test-judged) — the gate's effects
(refusal tokens, transition success, edge-only firing, composition) are all
observable through the `slice status` command boundary and the `trunk_integration`
return, so nothing needs VA/VH. Tests use the existing git-repo fixture pattern
(cf. the dispatch journal tests) to stand up a `refs/heads/dispatch/<slice>` ref
with a committed `journal.toml`.

## Notes

- Non-goals (design §6): no trunk mutation, no auto-integrate, no `deliver_to`
  config (→ IMP-124), no `--force` bypass, `reconcile → done` only.
- Adversarial pass (pre-lock) cleared the load-bearing assumptions against real
  code: the dispatch coord ref survives to close (never GC'd; close 3a tree-reads
  it) and integrate (close 3a) precedes the `done` flip (close step 4).
