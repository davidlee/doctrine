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

This slice is the **core Mode B** work. Mode A (the coordinator-exemption knob) is a
named follow-on, not this slice.

## Scope & Objectives

1. **IMP-253 keystone — gated `mcp__doctrine__worker_commit`.** A jailed worker calls
   it; the unconfined MCP server commits the worker's delta with the funnel belts:
   `doctrine check prove` gate; scope belt (reject `.doctrine/`/`.claude/` writes,
   enforce the slice's design-target selectors); **exactly one non-merge commit with
   `parent(tip) == B`** (ancestry "descended from B" is too weak — rejects a
   `B→C1→C2→C3` stack on a resumed worktree). Retires the "orchestrator imports the
   working-tree diff" dance **on the claude arm only** (see Non-Goals).

2. **Dispatch MCP surface for the mechanical happy path.** The mechanical funnel ops
   are CLI-only orchestrator verbs today; the MCP server currently exports only
   review/memory/onboard tools. Add MCP tools for the **file-disjoint happy path**
   (commit / one-shot import / reap sibling worktrees / record-boundary / lifecycle
   flips), each **riding the same belt/logic the CLI verb already runs** (no forked
   implementation — one seam, two invocation doors).

3. **Confined orchestrator drive-loop.** An orchestrator subagent type (lists `Agent`
   in `tools`) runs in the coordination worktree (→ `Jail(coord-cwd)`): spawns
   `dispatch-worker` subagents, drives the funnel cadence via the MCP tools, keeps raw
   `Edit`/`Write` confined to the coord tree. Conflict-judgement ops
   (`refresh-base`, `candidate create/admit`, `integrate`) stay **report-and-halt** to
   the human/main-thread — the existing "conflicts report-and-halt, never auto-merge"
   posture, unchanged.

4. **RSK-225 residual mitigation — worker tool-surface conformance lint.** A worker
   holds *only* the gated commit tool and nothing else writable. Because MCP tools
   bypass the wall, jail completeness now depends on the worker `tools:` list being
   pinned. Add a conformance lint on worker agent-defs that fails if a worker's
   `tools:` contains any writable MCP tool other than the gated commit tool (no such
   guard exists today; the doctrine MCP server already exports writable
   `memory_record`/`memory_edit`).

## Non-Goals

- **Mode A (coordinator exemption).** The `[dispatch] allow-coordinator-exemption`
  knob, the SubagentStart stamp / PreToolUse read, `validate_policy` role-store
  reservation, `(session_id, agent_id)` binding, GC lifecycle — all deferred to the
  follow-on slice. This slice un-jails nothing.
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

- `src/mcp_server/**` — new dispatch tool surface + `worker_commit`.
- `src/worktree/**` — the funnel belts the tools must reuse; jail interplay.
- `src/dispatch*` / dispatch command layer — the CLI verbs whose logic the MCP tools ride.
- `.agents/skills/dispatch*`, `.claude/agents/**` — orchestrator agent-def, worker
  tool-surface pin, the conformance lint target.
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

- A confined orchestrator subagent drives a real slice phase end-to-end on the claude
  arm: spawns a worker, the worker self-commits via `worker_commit`, the orchestrator
  records the boundary and flips lifecycle — all while its own raw Bash to the shared
  `.git` stays walled (proving no un-jailing).
- The `worker_commit` belts reject: a `.doctrine/`/`.claude/` write, an out-of-selector
  write, a multi-commit / merge / non-`parent==B` delta, a `prove`-red tree.
- The conformance lint fails a worker agent-def that grants a writable MCP tool beyond
  the gated commit tool; passes the pinned one.
- Subprocess-arm dispatch still green (import path untouched).

## Summary

Build Mode B: confined subagent orchestrator + gated MCP funnel surface (keystone
`worker_commit`, IMP-253) + worker tool-surface conformance lint. No exemption, no
un-jailing; additive MCP door over existing belts; subprocess arm and main-thread
fallback preserved.

## Follow-Ups

- **Mode A** coordinator-exemption knob slice (with codex-1 fixes).
- **Governance ratified at reconcile:** ADR-012 REV (confined-orchestrator actor
  class) + ADR-011 D6 amendment (network-exfil admitted) + SL-182 confinement note.
- **Mislabel hardening**, **exfil tightening for downstream projects** (named,
  not this work).
