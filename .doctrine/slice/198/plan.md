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

## Critical review & mitigations (plan step 7)

Grounding the plan against implementation reality before phases materialise:

- **Resolver mechanism (PHASE-01) — resolve by branch, not name-probe.** git guarantees
  at most one worktree per branch, so `worktree_for_ref("dispatch/<agent>")` over the
  primary-root porcelain (`git.rs:555/1335`) is unambiguous by construction — tighter and
  cheaper than enumerate-all-coords + probe-`jail/<agent>.toml`. `ambiguous-agent` is kept
  only as a defensive refusal (normally unreachable). The per-worktree record is then read
  from the resolved worktree's coord to supply `{base, coord}` and the consistency check.

- **F2 / scope-after-fmt (PHASE-02) — two lines of defence (owner steer).** `check commit`
  runs `fmt`, which mutates repo-wide and can normalise a *pre-existing, out-of-scope*
  file. (1) **Primary hygiene:** the orchestrator pre-fmts trunk/main **before arming**
  (PHASE-03 EX-5), so B is fmt-clean at fork and fmt only ever touches worker-changed files
  — F2 does not arise operationally. (2) **Invariant fallback (EX-5/VT-4):** the belt
  classifies the **pre-fmt** intended delta and the commit stages **exactly those
  classified paths** — never the post-fmt diff — so F2 is safe even if the pre-fmt ritual
  is skipped.

- **Scope belt is two-tier (PHASE-02, owner steer) — don't hard-fail a planner omission.**
  A hard reject on any out-of-selector write punishes the common case where the planner
  under-declared `design-target`. Split by zone: **escalation zones** (`.doctrine/**`,
  `.claude/**`, `.agents/**`, `install/agents/**`, build/gate config) HARD-refuse
  (`forbidden-zone`) — a worker there could rewrite its own scope/tool-grant/gate; **src
  paths outside the selectors** but in no forbidden zone COMMIT and return in `undeclared:
  [paths]` (soft warn) — the orchestrator blesses them into the selectors or rejects at
  import. The soft tier feeds the *existing* audit-time `slice conformance` delta rather
  than pre-empting it, and stays within the locked threat model (same audit-caught class as
  the X1 sibling-spoof residual). The one judgment call folded: build/gate config is
  hard-fenced (a worker must never edit its own gate).

- **DispatchRecord home + shape.** The record is a **sibling** file to the jail policy,
  not an overload of `<name>.toml` — confinement policy and dispatch-resolution are
  distinct concerns. The `DispatchRecord` type lives in `src/worktree/` (written by
  `create.rs`, deleted by `gc.rs`, read by `src/mcp_server/worker_commit.rs`) — command
  (mcp_server) depends on engine (worktree), respecting ADR-001 layering. TOML, matching
  the jail policy's format.

- **Behaviour-preservation on shared machinery.** PHASE-01 edits `create.rs`/`gc.rs` (the
  create-fork + reap paths — shared machinery). The existing create/fork/gc suites are the
  proof: they must stay green unmodified. Same gate for PHASE-03's `run_verify_worker`
  relaxation against the subprocess arm.

- **Lint host parser reuse (PHASE-04).** `doctrine doctor` is the host (in `just validate`
  → check/gate), but it does not parse agent-def frontmatter today. `src/install.rs`
  already handles agent-defs (it installs them) and is the likely home of a reusable
  frontmatter/`tools:` reader — ride it, do not fork a second YAML parser. Confirm at
  phase-plan (EN-2).

- **PHASE-02 sizing watch.** It is the heaviest phase (registration + resolver consumption
  + six belts + commit + pin). It is cohesive (one tool, one responsibility) and leans on
  reused seams, so it holds as one phase — but if it sprawls at phase-plan, the natural cut
  is belts/resolution vs the commit + pin.
