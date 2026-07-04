# Confined dispatch orchestrator (Mode B)

## Context

Dispatch today runs orchestration on the **main thread**: it owns the `Agent`
spawn, imports worker working-tree diffs, and is the sole writer of `.doctrine/`
+ trunk. The SL-182 `PreToolUse` wall confines every subagent (`agent_id` present
→ `Jail(cwd)` or `Reject`), so a subagent cannot drive the funnel — it can't write
the shared `.git`/`.doctrine/` that live outside its cwd. Moving orchestration off
the main thread unlocks parallel/hierarchical dispatch and keeps the main context a
thin, cache-stable router (RFC-011 lever).

**Mode B** (RFC-005 design, `.doctrine/rfc/005/subagent-orchestrator-design.md`)
achieves this **without un-jailing anything**: a fully-confined orchestrator subagent
spawns workers (`Agent` passes the wall), reads freely (`Read` passes), and performs
every boundary-crossing privileged write through **doctrine MCP tools** that run in
the already-unconfined MCP server. The enabling harness fact — an `mcp__*` write tool
bypasses the `Bash|Edit|Write` wall — is **witnessed** (RSK-225 discharged 2026-07-04,
commit `7bd21f49`; `.doctrine/rfc/005/subagent-orchestrator-probe.md`,
[[mem.fact.dispatch.pretooluse-wall-mediates-write-tools-only]]).

This slice is the **Mode B foundation** — the `worker_commit` keystone + its
tool-surface guard, driven by the **existing (main-thread) orchestrator**. The
confined-*subagent* orchestrator drive-loop and the wider dispatch MCP funnel surface
it needs are the serial-dependent capstone (**SL-199**, `needs` this slice). Mode A
(the coordinator-exemption knob) is a separate named follow-on.

## Scope & Objectives

1. **IMP-253 keystone — gated `mcp__doctrine__worker_commit`.** A jailed worker calls it
   with an **opaque `agent` id (its worktree name), never a path**. The unconfined
   MCP server (root=primary) resolves the target entirely server-side: sanitise `agent`
   (one validator) → `git worktree list` enumerate live `dispatch/<NNN>` coord trees →
   probe each for the per-worktree record `jail/<agent>.toml` → **exactly one live hit**
   (0 ⇒ `unknown-agent`, >1 ⇒ `ambiguous-agent`) → validate `{dir,branch,base}` consistent
   (design §10 pass-2). Then commits the worker's delta with cheap-first belts: non-empty
   pre-fmt in-scope delta → scope belt (reject `.doctrine/`/`.claude/` + undeclared paths)
   → `HEAD==B` → **exactly one non-merge commit `parent(tip)==B`** (ancestry is too weak —
   would accept a `B→C1→C2→C3` stack) → then the mutating `doctrine check commit` gate
   (worker-side, forces fmt — owner-locked). B comes from the record, written by the
   trusted create-fork hook (not the racy arming slot). The worker's Bash `git commit` is
   walled (ro `.git`); this is its only self-commit route. Residual: a worker may spoof a
   *sibling's* live `agent` (attribution confusion, review-caught, **own work not
   promoted** — orchestrator imports the branch it armed) — accepted, tracked by RSK-226.

   The **per-worktree record** (`{name,dir,branch,base,coord}`, `jail/<name>.toml`) is
   net-new load-bearing state: create-fork writes it atomically pre-worker; **reap/gc
   deletes it** (net-new `gc.rs` step — fixes the stale-file oracle).

2. **Retire the import-the-working-tree-**diff** dance — claude arm only.** Today the
   orchestrator reads the Agent footer's `worktreePath`, runs `verify-worker --dir`,
   and imports the live *working-tree diff*
   ([[mem.fact.claude.worktree-remove-auto-teardown]]). With `worker_commit` the worker
   produces a *commit* on its worktree; the (still main-thread) orchestrator imports a
   **commit**, not a diff. Adjust the orchestrator-side import path + the CLAUDE.md
   `# orchestration` note accordingly. Linked worktrees stay (no clone switch). The
   subprocess arm keeps the existing import path (see Non-Goals).

3. **RSK-225 residual mitigation — worker tool-surface conformance lint.** A worker
   holds *only* the gated commit tool and nothing else writable. Because MCP tools
   bypass the wall, jail completeness now depends on the worker `tools:` list being
   pinned. Add a conformance lint on worker agent-defs that fails if a worker's
   `tools:` contains any writable MCP tool other than the gated commit tool (no such
   guard exists today; the doctrine MCP server already exports writable
   `memory_record`/`memory_edit`). Pin `dispatch-worker`'s `tools:` to add exactly
   `mcp__doctrine__worker_commit`.

## Non-Goals

- **Confined-subagent orchestrator drive-loop + wider dispatch MCP funnel surface →
  SL-199.** The orchestrator subagent type, the `Jail(coord-cwd)` drive-loop, and the
  MCP tools a *confined* orchestrator needs (import / reap / record-boundary /
  lifecycle-flip) are the serial-dependent capstone. Building those MCP tools here —
  before their only consumer (a confined orchestrator) exists — would be speculative
  parallel-implementation. `worker_commit` is the one MCP tool a *jailed worker* needs,
  so it lands here; the rest ride with SL-199.
- **Mode A (coordinator exemption).** The `[dispatch] allow-coordinator-exemption`
  knob, the SubagentStart stamp / PreToolUse read, `validate_policy` role-store
  reservation, `(session_id, agent_id)` binding, GC lifecycle — all deferred to a
  separate follow-on slice. This slice un-jails nothing.
- **Retiring the import dance globally.** IMP-253 retires it **for the claude arm
  only**. The subprocess arm (codex/pi) keeps the import path — a subprocess worker's
  stdio MCP inherits the jail (no bypass), and the import path is also the MCP-down
  fallback for solo users. Ripping it out wholesale breaks subprocess dispatch. Keep
  it intact; gate the retirement on arm.
- **Persistent (http/sse) MCP for arm unification.** The claude/subprocess unification
  option is noted, not built here.
- **Conflict/judgement automation.** `integrate`, `candidate admit`, `refresh-base`
  stay interactive report-and-halt; not MCP-ified.
- **Network-egress wall / exfil tightening** for downstream projects.

## Affected surface (coarse — `/design` sets the exact touch-set)

- `src/mcp_server/**` — the single `worker_commit` tool (registration + handler).
- `src/worktree/**`, `src/dispatch.rs` — the funnel belts (`prove`, scope classify,
  the `parent==B`/non-merge/one-commit invariant) `worker_commit` must **reuse**, not
  re-implement; the orchestrator-side commit-import path.
- `.claude/agents/**` (`dispatch-worker`) — pin `tools:` to add exactly the gated
  commit tool; the conformance-lint target.
- the conformance-lint host (a `doctrine check`/conformance verb — exact location
  `/design` decides).
- `CLAUDE.md` `# orchestration` note — retire the import-diff dance (claude arm).

## Risks / assumptions / open questions

- **Belt relocation (OQ-2).** Do the import belts survive relocation into a
  worker-invokable MCP tool? Which funnel ops lack a tool today and must gain one.
- **Enforcement mechanism (OQ-3, RSK-225).** Is there a harness/doctrine way to
  *enforce* (not just document) that a worker holds no un-gated writable tool?
- **MCP soft-dependency.** For the off-main-thread path, MCP-server health becomes a
  dispatch-stopper (manual fallback = main-thread import). Acceptable; document it.
- **`arm-spawn` under a subagent spawner + `Jail` (OQ-1).** Closed "yes mechanically"
  by codex-2; one live confirmation at execute-time, no design blocker.
- **Depth-5 ceiling (OQ-5)** vs hierarchical-orchestration ambitions.
- **Assumption:** the claude-arm MCP-write bypass holds across harness upgrades
  (version-sensitive — re-probe on upgrade).

## Verification / closure intent

- A **jailed worker** (cwd = linked worktree, shared `.git` ro) whose raw Bash
  `git commit` is walled calls `worker_commit` and its delta lands as one commit on
  the worker branch — witnessed end-to-end on the claude arm.
- The `worker_commit` belts **reject** (each a test): a `.doctrine/`/`.claude/` write,
  an out-of-selector write, a multi-commit / merge / non-`parent==B` delta, a
  `prove`-red tree. Belt logic is shared with the existing import path (behaviour-
  preservation: existing funnel suites stay green).
- The conformance lint **fails** a worker agent-def granting a writable MCP tool
  beyond the gated commit tool; **passes** the pinned `dispatch-worker`.
- The (main-thread) orchestrator imports the worker's **commit**; the working-tree
  **diff** import path is retired on the claude arm only. Subprocess-arm dispatch still
  green (its import path untouched).

## Summary

Mode B foundation: gated `worker_commit` MCP tool (keystone, IMP-253) so a jailed
worker self-commits through the trusted server, + a worker tool-surface conformance
lint (RSK-225 mitigation), + retire the claude-arm working-tree-diff import in favour
of commit-import. No exemption, no un-jailing; belts reused not re-implemented;
subprocess arm preserved. The confined-subagent orchestrator + wider MCP funnel
surface are the serial-dependent capstone SL-199.

## Follow-Ups

- **SL-199** (serial-dependent capstone) — confined-subagent orchestrator drive-loop +
  the dispatch MCP funnel surface (import / reap / record-boundary / lifecycle) a
  confined orchestrator needs. Shaped in parallel while this slice builds.
- **Mode A** coordinator-exemption knob slice (with codex-1 fixes).
- **Governance ratified at reconcile:** ADR-012 REV (confined-orchestrator actor
  class) + ADR-011 D6 amendment (network-exfil admitted) + SL-182 confinement note.
- **Mislabel hardening**, **exfil tightening for downstream projects** (named,
  not this work).
