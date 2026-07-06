# IMP-275: Workflow orchestrator-leaf lands/closes in-workflow (audit/reconcile/close)

Deferred from SL-206 §5.4 design conversation (2026-07-06). SL-206 scopes the
`/drive-slice` workflow to drive a slice's phases **to Completed + emit the
authored-divergence advisory only** — landing (`/audit → /reconcile → /close`,
which integrates onto `main`) stays a **main-thread** flow (reading (i), matches
IMP-174 no-auto-land). This item captures the deferred reading (ii): the
orchestrator role ultimately also lands the code.

**Why deferred, not dropped.** The operator's position: (ii) *needs to happen* —
the orchestrator's full remit is prep-context → check → adapt → **land/audit/
reconcile/close**, and landing itself "can require a fair bit of research /
thinking." But it is the **heaviest unproven surface**: a workflow leaf driving
`dispatch sync --integrate` onto `main`. `dispatch sync` is a CLI, and there is
no witnessed MCP path for the full integrate beat from a workflow-spawned leaf
(the funnel MCP tools cover import→verify→conclude, not integrate/land).

**Blocked on the SL-206 §5.4 outcome** — the three-role sequential shape
(script sequences · orchestrator-leaf judges · worker-leaf executes) must land
and the commit path be witnessed first. See [[mem.fact.claude.workflow-strips-agent-tool]].

**Open questions for when this is picked up:**
- Is there (or must there be) an MCP path for `dispatch sync --integrate` a
  workflow leaf can call? (`worker_commit` is the model — server-side unconfined,
  belt-gated — but integrate is a bigger, cross-branch write onto `main`.)
- Does landing's judgement (conflict resolution, research) exceed what a per-beat
  leaf reading doctrine state can do, forcing a richer/long-lived agent?
- Interaction with `/audit → /reconcile → /close` skill flows — reuse vs
  reimplement inside the workflow.
