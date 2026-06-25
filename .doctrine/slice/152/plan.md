# Implementation Plan SL-152: Claude-arm WorktreeCreate worker creation

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

The slice tests one hypothesis: with doctrine as the worktree creator on the
claude arm, both `/dispatch` arms collapse onto one byte-identical
`worktree fork --worker` core (design §1, §4). The plan delivers that mechanism
bottom-up — pure core first, then the imperative shell, then the orchestrator
handshake, then install wiring, then the skill contract — and isolates the
secondary plugin idiom behind a droppable final phase.

All three design pre-plan checks are discharged (notes.md, design §10): F3 proved
`fork --worker` is live and provisions from the coord tree (the D1 byte-identical
thesis holds), `dispatch setup` already surfaces base B on stdout, and
`.worktrees/` is gitignored. So the plan leans on a confirmed foundation, not a
hoped-for one.

## Sequencing & Rationale

**Bottom-up, pure-before-impure (CLAUDE.md split).** PHASE-01 isolates the
decision logic — `classify_create` + the `name` shape-sanitiser — as pure
functions with golden refusal tokens, mirroring the existing `classify_stamp`
(subagent.rs:84). Pure code is the cheapest place to nail the discrimination
matrix (I1's mechanical floor) and the I4 sanitiser, and it carries no
environment risk. Live-ref collision detection is deliberately NOT here — it
needs git, so it belongs to the shell (PHASE-02).

**PHASE-02 is the heart.** The `worktree create-fork` shell wires gather → classify
→ act over the PHASE-01 core, reusing `run_fork` and `run_provision` unchanged
(DRY; the behaviour-preservation gate). Three design locks land here as tests:
I5 root-forcing (root always resolved from *payload* cwd via `--show-toplevel`,
passed explicitly — never process cwd), I2 benign provisioning parity (the
pass-through provisions through the *same* copier so repo-global benign subagents
don't lose `.worktreeinclude` files), and fail-closed exit. VT-2 freezes the F3
e2e (gitignored sentinel absent-from-B landing in the fork) as a permanent
regression guard against the ISS-011 Defect C trap. This phase also removes the
`#![expect(unused)]` extraction lids once their functions become live consumers.

**PHASE-03 (arm-spawn) is small and could precede PHASE-02 mechanically** — it
only writes the `base` file create-fork reads — but it is sequenced after so the
file contract is fixed by the consumer first. Base B is already available from
`dispatch setup`'s stdout; arm-spawn just persists it where the hook can read it.
Disarm is positional (cd-back), so no load-bearing teardown verb is required.

**PHASE-04 closes the primary hypothesis.** Install emits the WorktreeCreate hook
(D7 primary, settings-block form — zero plugin work) and retires the now-redundant
SubagentStart stamp (D2): create-fork already provisions+marks atomically, and the
stamp fires too late to feed base selection. F7's verification guards the marker
invariant migrating to the new seam. The headline H1-under-churn scenario is
exercised end-to-end at the CLI here (VT-4); the true live-harness confirmation is
a you-run-it agent check (VA-1), since the harness owns the spawn.

**PHASE-05 updates the dispatch-agent skill** to the new post-spawn contract (I3):
arm/cd-in/cd-back bracket replaces the cwd-placement hack, and the orchestrator
derives `branch` from the footer's `worktreePath` (normative; P2) instead of the
unproven `worktreeBranch`. Prose work, verified by review + a live dry-run.

**PHASE-06 is secondary and droppable (RSK-2).** Gated on probe P1 (plugin-hook
parity), it migrates the hook into a plugin's `hooks/hooks.json` and removes the
settings block in the same step (mutual exclusion — double-wiring would double
creation). The primary is complete without it; drop it if it threatens the slice.

## Notes

- **Behaviour-preservation gate** rides every phase touching shared machinery
  (`fork --worker`, `run_provision`, install): the existing dispatch/subprocess
  suites are the proof and must stay green unchanged (VT-6, VT-3 in PHASE-04).
- **Probe carry-ins:** P1 (plugin parity) gates only PHASE-06; a cheap confirming
  probe on whether `worktreeBranch` populates for a *named*-branch hook fork is
  nice-to-have, not gating (D8 derives branch from `worktreePath`).
- **Follow-ups (out of scope, design §slice Follow-Ups):** WorktreeRemove hook /
  branch GC for re-dispatch leaks (F5/D10); reassessing the SL-123 belt scope.
- If any phase surfaces a substantive design problem, STOP and re-enter `/design`
  (reconcile design first) — the plan is not higher authority than the design.
