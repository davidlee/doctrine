# IDE-031: Workflow-templated slice-driver: budget-metered, resumable phase-drive over the confined orchestrator

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

Drive a slice's phases to completion from a **dynamic Workflow** (the JS
orchestration harness), not ad-hoc `Agent`-tool spawns. The RFC-011 lever: main
context stays a thin cache-stable router; each phase's ~40–49k funnel ceremony
burns in a **disposable** orchestrator+worker context; the Workflow **`budget`
pool** meters the whole tree **live** — the one capability ad-hoc spawns lack (they
give receipts at the return boundary only, no live cross-agent pool).

Downstream of SL-199 (confined `dispatch-orchestrator` + the `dispatch_import` /
`dispatch_conclude_phase` / `dispatch_reap` MCP funnel). Gated by CHR-039 (spike):
the harness-behaviour assumptions must be verified before a design is sound.

## Why Workflow, not the Agent tool

| Axis | Agent tool (ad-hoc) | Workflow (JS harness) |
|---|---|---|
| Token metering | receipt at return boundary only | **live shared pool** (`budget.total/spent()/remaining()`, hard ceiling) |
| Control flow | model-driven, turn by turn | **deterministic JS** (loops/conditionals/fan-out) |
| Structured return | prose → parse (agents hallucinate) | `schema:` → validated object, retries on mismatch |
| Resume | none (SendMessage keeps context) | runId prefix-cache — **same session only** |

## Coarse design (baseline; fine variant gated by CHR-039)

Per SL-199 §2 the workflow script has **no shell** — it cannot `arm-spawn`. So
arming lives inside a spawned agent's turn:

- Workflow sequences ready phases; **one `dispatch-orchestrator` agent() per
  phase** (arm → nested worker → import → conclude → reap, all internal — SL-199
  as-shipped).
- Each phase agent returns a **typed phase-receipt schema**
  `{phase, status, coord_tip, worker_branch, verify:{green,failures}, halt_reason?}`
  — control flow becomes JS, not prose-parse. **Highest-leverage knob.**
- Budget-gated loop: `while (budget.total && budget.remaining() > estPhaseCost)
  driveNextPhase(coord_tip)`.
- **Report-and-halt** (red verify, hard import-scope refusal, integrate race) → a
  thrown stage, collected; never auto-merge (SL-199 §D boundary intact).
- Save via `/workflows` → `s` → `.claude/workflows/` → `/drive-slice`; slice-id via
  the `args` global.

## Knobs to expose

1. **Phase-receipt output schema** — the highest-leverage lever; moves control flow
   model→JS. `dispatch_import` already returns `{coord_tip}`; extend every funnel
   tool to typed receipts.
2. **Allowlist the funnel MCP tools pre-launch** — else a background run stalls on
   the permission gate (CHR-039 SQ4; `workflows.md:162-166`). Ship as a launch
   requirement, not just tool grants. (The SL-199 confinement **wall** survives —
   it's a `PreToolUse` Reject hook, a separate layer from `acceptEdits`.)
3. **`taskBudget` sub-orchestrator pacing** (Alpha) — inject remaining-token budget
   so the phase agent self-paces + wraps up. Upgrades the control story to three
   layers: prompt discipline + `taskBudget` signal + workflow `budget` ceiling.
4. **Effort/model tiering** — worker TDD loop = high; import/conclude/reap ceremony
   = low. Spend tokens where the reasoning is.
5. **Server-side `dispatch_arm` MCP** — **only if** CHR-039 forces the fine-grained
   variant (workflow drives each funnel step as its own agent()). Skip for coarse.

## Cross-session caveat

Workflow resume is **same-session only** (`workflows.md:297`) — exit CC mid-run and
the next session starts fresh. So the **committed dispatch boundary on
`dispatch/<slice>` (SL-199 atomic conclude) is the durable cross-session drive
oracle**, not the workflow journal. This strengthens SL-199's atomic conclude: not
just crash-safety but the cross-session resume anchor.

## Sub-orchestrator prompt contract

Bake in boundary-metering discipline: drive exactly ONE phase then STOP + return
the typed receipt (not prose); self-limit on tool-uses; read raw / write MCP
(SL-199 §C); report-and-halt on any judgement fork.

## Open fork (decided by CHR-039)

- **Coarse** — one orchestrator/phase; works now; fewer boundary crossings.
- **Fine** — script drives each funnel step as its own agent()/MCP; live per-step
  budget granularity; needs `dispatch_arm` (knob #5); fights the arming-cwd coupling.

## Dependencies

- **`needs SL-199`** — the confined orchestrator + MCP funnel this drives.
- **`needs CHR-039`** — the spike; a sound design can't be locked until SQ1–5 answer
  whether the harness permits the coarse approach.

## References

- `docs/claude/workflows.md` — user-facing Workflow doc (constraints, resume, cost).
- `docs/claude/agent-sdk/typescript.md` — exhaustive `agent()` / Workflow tool opts.
- SL-199 design §A (create-fork confined arm), §B (funnel), §D (drive-loop).
- RFC-011 — token-efficiency benchmark (the motivating lever).
- Neighbours: IDE-016 (MCP-read efficiency), IDE-017 (worker provisioning),
  IMP-104 (pi scout-spawn), IDE-008 (executable phase gates).
