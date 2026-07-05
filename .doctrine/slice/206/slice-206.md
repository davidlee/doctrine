# Workflow-templated slice-driver

Promotes IDE-031. De-risked by CHR-039 (probe matrix resolved; findings at
`.doctrine/rfc/011/chr-039-findings.md`). Downstream of SL-199 (confined
`dispatch-orchestrator` + the `dispatch_import` / `dispatch_conclude_phase` /
`dispatch_reap` MCP funnel).

## Context

Today a slice's phases are driven by ad-hoc `Agent`-tool spawns from the main
context. Two costs: (1) the ~40–49k funnel ceremony per phase burns in the
main (cache-stable router) context instead of a disposable one; (2) there is
**no live token metering across the agent tree** — the Agent tool gives a
receipt only at the return boundary.

The RFC-011 lever: keep main a thin router; drive phases from a **dynamic
Workflow** (the JS orchestration harness) whose **`budget` pool** meters the
whole spawned tree **live**, with deterministic JS control flow and
schema-validated returns instead of prose-parse.

CHR-039 confirmed the load-bearing harness behaviour: a workflow
`agent(isolation:'worktree')` fires the `WorktreeCreate` hook with a
controllable cwd; isolated forks reach doctrine MCP with a clean binary; a
background worker has **no live escalation channel** (human-in-loop is
return-value only); `budget` plumbing is live. Safety posture (operator
conclusion): **the workflow spawns a confined orchestrator, never a bare
worker** — the orchestrator (ADR-006 sole-writer) is the trust boundary.

## Scope & Objectives

- A saved, parameterized **Workflow script** (`/drive-slice`, slice-id via the
  `args` global) that sequences a slice's ready phases and drives each to
  completion through **one `dispatch-orchestrator` `agent()` per phase**.
- **Typed phase-receipt schema** returned by each phase agent
  (`{phase, status, coord_tip, worker_branch, verify:{green,failures},
  halt_reason?}`) — moves control flow from prose-parse to JS. Highest-leverage
  knob; extend the funnel MCP tools (`dispatch_import` already returns
  `{coord_tip}`) toward typed receipts as needed.
- **Budget-gated loop**: `while (budget.total && budget.remaining() >
  estPhaseCost) driveNextPhase(coord_tip)`. Degrade gracefully when `total` is
  null (no `+Nk` directive).
- **Report-and-halt** on red verify / hard import-scope refusal / integrate
  race → a thrown, collected stage surfaced at the script boundary. Never
  auto-merge (SL-199 §D boundary intact).
- **Return-value escalation** — no automatic human-in-loop; a phase that needs
  a decision halts and returns a structured "needs decision X" for the main
  thread to relay.

## Non-Goals

- No change to SL-199's confined orchestrator internals (arm → nested worker →
  import → conclude → reap stay as shipped). This slice *drives* it, not
  reshapes it.
- No workflow-spawned **bare worker** — spawning a worker directly from the
  script is out of scope by safety posture (containment is by policy, not
  MCP-unreachability).
- No cross-session resume guarantees beyond the harness's own runId
  prefix-cache (same-session only).
- No new escalation transport — return-value at the script boundary is the
  contract; a live channel is explicitly out of scope (none exists).
- **No auto-land** of the fork's authored+code deltas to edge/main — landing
  stays the existing `/audit → /reconcile → /close` path (IMP-174 unresolved).
  The driver drives + reports only.
- **No batching-per-orchestrator** — one orchestrator per phase. The dumb-zone
  `SOFT_CEILING` ships advisory-only; batching (where it becomes control flow)
  is a deferred follow-up.

## Affected surface (coarse — `/design` fixes the touch-set)

- The funnel MCP tools' return shapes — `src/**` around
  `dispatch_import` / `dispatch_conclude_phase` / `dispatch_reap` (typed
  receipts).
- The saved workflow script + its registration (`.claude/workflows/`,
  harness-side; may be an authored template shipped by `install/`).
- Docs: `dispatch-mechanics.md` and/or a new driver reference; RFC-011
  case-notes.

## Risks / Assumptions / Open questions

- **SQ3 not empirically re-demoed** — the confined fork-at-base from a workflow
  path leans on SL-199 prior art, not a fresh demo. First design gate: decide
  whether to demand the demo before committing the receipt-schema work.
- **Un-allowlisted MCP writes run un-prompted** from a background worker
  (CHR-039 SQ4-permgate, reads only tested) — the driver must gate governance
  writes deliberately (cf. the server-side `worker_commit` gate), not inherit
  ambient trust. **Safety-material.**
- **Funnel receipt extension** touches shared machinery — behaviour-preservation
  gate: existing dispatch suites must stay green.
- **Budget target** must be set (`+Nk`) for live metering to bite; without it
  the loop can't pace. Assumption: acceptable to no-op the meter when unset.
- Where does the workflow script live as an **authored, shippable** artifact vs
  a harness-local file? `install/`-templated vs `.claude/workflows/` only.
- **IMP-174 split-brain** (`related`) — the fork's authored `.doctrine/**` tier
  can diverge from edge; the driver inherits it at the drive→close handoff.
  Mitigated by coord-committed-truth reads + no auto-land + a seeded raw
  divergence advisory; full reconciliation is IMP-174's, not this slice's.

## Verification / closure intent

- The `/drive-slice` workflow drives a real multi-phase slice end-to-end in a
  dispatch coord tree, each phase returning a valid typed receipt, halting
  correctly on an injected red verify, and never auto-merging.
- Funnel-tool receipt changes leave existing dispatch suites green.
- Budget loop demonstrably paces (or gracefully no-ops) under a set / unset
  `budget.total`.
