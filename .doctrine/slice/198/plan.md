# Implementation Plan SL-198: Mode B foundation — gated `worker_commit` + tool-surface lint

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Four phases deliver the Mode B foundation: a jailed worker self-commits through
the unconfined doctrine MCP server, gated by belts the server enforces, driven by
the **existing main-thread** orchestrator. The confined-subagent orchestrator and
the wider MCP funnel surface are the serial-dependent capstone SL-199 — out of
scope here.

The design (locked, two external codex passes, LOCK-READY-WITH-THIS) reframed the
work: the "one import switch" is the *small* part. The load-bearing work is
**trusted plumbing** — a per-worktree dispatch record that lets the server resolve
an opaque worker identity to a target without trusting a worker-supplied path, and
a conformance lint that keeps the worker's writable tool-surface pinned now that
MCP writes bypass the SL-182 wall (RSK-225).

## Sequencing & Rationale

The phases form a hard serial spine 01 → 02 → 03, with 04 hanging off 02:

- **PHASE-01 (record + lifecycle) is first because it is the trust anchor.**
  Codex pass-2 established that the target-fence cannot be "a jail file exists"
  (a stale-file oracle — `gc.rs` reaps worktree+branch but never the record) and
  cannot be a worker-supplied path (the MCP server gets no caller identity, and a
  worker's `Read` passes the wall). The resolution is a per-worktree record
  `{name, dir, branch, base=B, coord}` written by the **create-fork hook** —
  trusted, pre-worker — and **deleted at reap**, resolved by enumerating coord
  trees through the *existing* `git worktree list --porcelain` seam. Building this
  first gives PHASE-02 a clean `resolve_agent(agent) → record | refusal` seam and
  closes the stale-oracle at the source. It also relocates base **B** off the
  racy single-slot arming slot (design D4 superseded) into immutable per-worktree
  state, which every later phase depends on.

- **PHASE-02 (`worker_commit`) consumes PHASE-01 and is the keystone (IMP-253).**
  The handler resolves the opaque, sanitised `agent` id, then runs belts
  **cheap-first** (design X4): non-empty pre-fmt in-scope delta → scope
  (`classify_import` vs the slice's `design-target` selectors) → `HEAD==B` →
  *then* the mutating `check commit` gate (`resolve_check(Commit)`) → exactly one
  non-merge commit `C^==B`. Order matters because `check commit` runs `fmt`, which
  mutates the tree and can widen the touched-path set — scope and base checks must
  precede it. Belts **reuse** `classify_import` and `resolve_check`, not fork them;
  the behaviour-preservation gate is that the existing funnel suites stay green.
  This phase also pins `dispatch-worker`'s `tools:` to grant exactly the new tool —
  the passing fixture PHASE-04 will lint against.

- **PHASE-03 (import switch) needs PHASE-02's commit and is claude-arm-only.**
  With the worker now producing a *commit*, the orchestrator imports a commit
  (`run_import` fork path) instead of the working-tree diff, with the `--branch
  dispatch/<agent>` coherence belt (`subagent.rs:316`). That `--branch` binding is
  what bounds the X1 residual: the orchestrator imports the branch **it armed**, so
  a poisoner who spoofed a sibling's `agent` (committing to the sibling's branch)
  leaves its own branch at B and promotes nothing. The subprocess arm keeps
  `--from-worktree` untouched — a subprocess worker's stdio MCP inherits the jail
  (no bypass) and the diff-import is also the MCP-down fallback for solo users.
  Isolating the switch in its own phase keeps the subprocess-arm behaviour-
  preservation gate (existing suites green, unmodified) the explicit proof.

- **PHASE-04 (conformance lint) is last: it guards the deployed surface.**
  It depends only on PHASE-02 (the pinned `dispatch-worker` fixture) and is
  independent static analysis, so it doesn't block the mechanism. It is the
  RSK-225 mitigation — jail completeness now rests on the worker holding no
  un-gated writable MCP tool. It rides `doctrine doctor` (DRY: `just validate`
  already runs it inside `check` and `gate`), scans the **authored** defs
  (`.doctrine/agents/**` + `install/agents/**`, not the installed `.claude/` copy —
  design X3), and treats the `doctrine-role` marker as **mandatory**
  (deny-by-default), so an unmarked worker def fails rather than passing as a "doc
  gap".

## Notes

- **Boundaries inherited from scope/design.** No confined-subagent orchestrator
  drive-loop, no wider MCP funnel surface (import/reap/record-boundary/lifecycle),
  no Mode A exemption — all SL-199 / follow-on. The import-dance retirement is
  claude-arm-only; the subprocess arm is preserved intact.
- **Governance (F4).** SL-198 un-jails nothing (it rides the witnessed passthrough),
  so it needs no ADR REV. The sanctioned worker MCP write, made safe by the lint, is
  a **note** on ADR-008 / SL-182 ratified at reconcile — not an amendment, not a
  blocker for this plan. ADR-012 REV + ADR-011 D6 amendment belong to SL-199.
- **Residual (RSK-226).** worker_commit cannot authenticate its caller (no harness
  `agent_id` channel to the server). The opaque-id + registry-validated resolution
  narrows this to sibling-name spoofing (attribution confusion, review-caught, own
  work unpromoted, no escalation), accepted per the locked threat model. A true
  caller-binding is the RSK-226 follow-on.
- **VT attribution.** The PHASE-01/03/04 VT mandates target files that already exist
  (existing suites live there); they read `UNATTRIBUTABLE` until the slice's commits
  touch them, then resolve. PHASE-02's `worker_commit.rs` is net-new (reads `FAIL`
  until created — the expected red). Distinctive keywords (`DispatchRecord`, the
  refusal tokens) are tightened at phase-plan so attribution is meaningful.
