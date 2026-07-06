# Workflow-templated slice-driver

Promotes IDE-031. De-risked by CHR-039 (probe matrix resolved; findings at
`.doctrine/rfc/011/chr-039-findings.md`). Downstream of SL-199 (the
`dispatch_import` / `dispatch_conclude_phase` / `dispatch_reap` MCP funnel).

> **Design posture (2026-07-06): full-commit UNJAIL.** The orchestrator runs
> **nominated-unjailed** (`PassThrough`, RW `.git`), not confined — proven safe
> P1/P3/P4 (`unjail-direction.md`). This **supersedes** the confined-orchestrator
> framing this scope originally carried; see `design.md` §5 + the ADR-008 amendment
> in scope (below).

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

**Trust model (unjail).** The **workflow is the sole spawn authority** and a
durable serial loop; it **alternates** ephemeral **nominated-unjailed**
orchestrators and **jailed** workers. There are never two orchestrators
back-to-back: each interior orchestrator does **two jobs in one agent** — dispose
the previous worker's commit *and* prep the next worker's context. Safety no
longer rests on the MCP-unreachability of coord `.git` (the confined model); it
rests on the **nomination + spawn-gate** closed loop — a `SubagentStart` allowlist
flips the orchestrator to `PassThrough`, and a `PreToolUse` gate denies any
jailed→privileged spawn at *every* spawn seam (`Agent` **and** `Workflow`). The
worker stays jailed either arm; confinement is the worker's, never the
orchestrator's.

## Scope & Objectives

- A saved, parameterized **Workflow script** (`/drive-slice`, slice-id via the
  `args` global) that drives a slice's ready phases to completion through
  **alternating orchestrator + worker `agent()`s** — one nominated-unjailed
  orchestrator *between* two jailed workers (two jobs: dispose + prep), FRESH
  context per hop (no prompt-cache TTL bleed, no single context driving a
  7-phase slice).
- **Typed per-hop receipt schemas** (harness `schema:`) moving control flow from
  prose-parse to JS: `WorkReceipt {fork_tip, summary}` and the combined
  `HopReceipt {dispose · prep · next_ready}` (`fixup` XOR `prep`).
- **Three new read-only doctrine tools** (Rust) the driver consumes:
  `dispatch_phase_receipt` (durable, boundary-ledger-backed status),
  `dispatch_next_ready` (thin wrapper over `compute_next_phases`),
  `dispatch_authored_divergence` (IMP-174 advisory). **No new write surface.**
- **Worker delta handoff**: claude-arm worker **self-commits via `worker_commit`**
  (fork-durable — the orchestrator reads a committed tip) = target; the
  orchestrator-imports-uncommitted-diff path = both-arm in-a-pinch fallback (pi
  arm always imports — bwrap-confined worker cannot commit).
- **Revive-to-fixup**: a fixable worker delta revives the worker on its fork
  (bounded, `MAX_FIXUP`) rather than halting — fork-durable, not context-intact.
- **Budget-gated loop** degrading gracefully when `budget.total` is null.
- **Report-and-halt** on red verify / funnel refusal / coord refusal / anomaly —
  named closed halt vocabulary; never auto-merge (SL-199 §D boundary intact).

## Non-Goals

- **No auto-land** of the fork's authored+code deltas to edge/main — landing
  stays the existing `/audit → /reconcile → /close` path (IMP-174 unresolved).
  The driver drives + reports only.
- No cross-session resume guarantees beyond the harness's own runId
  prefix-cache (same-session only).
- No new escalation transport — return-value at the agent boundary is the
  contract; a live human-in-loop channel is out of scope (none exists).
- **Revive is fork-durable, not context-intact** — a fresh `agent()` on the fork,
  not `SendMessage` continuation (unavailable in-workflow). Context-intact revive
  is a later enhancement (OQ-5).
- **No batching-per-orchestrator** — one orchestrator per hop. The dumb-zone
  `SOFT_CEILING` ships advisory-only; batching is a deferred follow-up.
- **ISS-216 (install-reseat gap) is not fixed here** — SL-206 depends on a manual
  reseat of changed agent defs until it lands.

## Affected surface (design-target detail in `design.md` §9)

- **Three read-only emitter tools** — `src/dispatch.rs` (phase_projection +
  `ReceiptStatus`), `src/mcp_server/dispatch.rs`, `src/mcp_server/tools.rs`.
- **Conformance authority** — `src/doctor_checks.rs`: doctor check #9
  (`allowed_mcp_tokens` grows for the new tools) **plus** a new #9-sibling check
  mechanizing the I1 spawn-seam-symmetry invariant (nomination ⊆ gate).
- **Agent defs** — `install/agents/claude/dispatch-orchestrator.md` (gains the
  three read tokens) and a **new** `dispatch-probe.md` (read-only role; also serves
  the claude-arm bootstrap O₀).
- **The `/drive-slice` script** + its authored home (OQ-1) + hook config
  (nomination allowlist + `PreToolUse` gate, out of every jail — I2).
- **ADR-008 amendment** — the orchestrator-unjail exception + the standing
  seam-symmetry obligation (confined orchestrator kept as a reversible escape
  hatch).

## Risks / Assumptions / Open questions

- **Spawn-seam symmetry (OQ-4, safety-material).** The gate must cover *every*
  spawn seam. The `Workflow` tool is a second seam a jailed holder could launder
  identity through; unproven that a jailed `agent()`/`Workflow` call presents its
  caller id to the gate. Phase-0 must confirm, or deny `Workflow` to jailed callers
  wholesale.
- **(B) worker self-commit residual (OQ-3, P5).** `SubagentStart(dispatch-worker)`
  fires for a workflow leaf ⇒ `DispatchRecord` provisioned ⇒ `worker_commit`
  resolves; the one open gate is whether a workflow leaf **retains** the
  `worker_commit` MCP tool (runtime strips `Agent`/`Grep`/`Glob`). Fallback: (A)
  import the uncommitted diff.
- **Revive-base mechanism (OQ-6).** Can the `WorktreeCreate` arming hook redirect a
  revive worktree's fork base to an existing `fork_tip`, or only its cwd? Floor:
  worker-side `git reset --hard <fork_tip>`.
- **Worker fork-at-base from a workflow leaf** not empirically re-demoed under
  unjail — phase-0 de-risk (asserts the worker fork mints at armed base on
  `dispatch/<n>`); does NOT re-run CHR-039's settled probes.
- **ISS-216 install-reseat gap** — the live grant surface under `.claude/agents/`
  can lag the authored one; phase-0 carries a clean-install precondition + a
  runtime probe against the installed def.
- **Behaviour-preservation** — the `phase_projection` extract from `run_status`
  must leave existing dispatch suites green unchanged.
- **IMP-174 split-brain** (`related`) — the fork's authored `.doctrine/**` can
  diverge from edge; the driver inherits it at the drive→close handoff. Mitigated
  by coord-committed-truth reads + no auto-land + a seeded raw divergence advisory;
  full reconciliation is IMP-174's.

## Verification / closure intent

- One real `/drive-slice` drives a multi-phase scratch slice end-to-end in a
  dispatch coord tree — each hop returns a valid typed receipt, the worker fork
  mints at armed base on `dispatch/<n>`, an injected red verify halts the loop
  without auto-merge, and a forbidden-write runtime probe refuses.
- The three read-only tools are TDD'd; doctor check #9 + the new I1 symmetry check
  stay green on the updated agent defs; existing dispatch suites green unchanged.
- Budget loop demonstrably paces (or gracefully no-ops) under set / unset
  `budget.total`.
