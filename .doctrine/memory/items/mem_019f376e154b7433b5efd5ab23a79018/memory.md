# Workflow spawn seam is closed by absence — no subagent holds the Workflow tool

The `Workflow` tool is **main-thread-only**. No subagent — by either spawn path —
receives it, so the "second spawn seam" the SL-206 §5.6 security model worried
about (a jailed caller laundering identity through the runtime to spawn a
privileged leaf, I1(b)) has **no reachable seam at runtime**: there is nothing for
a jailed actor to invoke. This settles design SL-206 **OQ-4** more strongly than
the design assumed (which planned a blanket `PreToolUse(Workflow)` subagent-deny in
case the tool WERE reachable).

## Proof (SL-206 PHASE-08, 2026-07-06, Claude Code / Opus 4.8)

- **Agent-tool spawn path** (live probe this session): a `general-purpose`
  (`Tools: *`) subagent asked to invoke `Workflow` reported **NO-TOOL** — not in
  its toolset, and `ToolSearch select:Workflow` found no deferred match. A
  throwaway `PreToolUse(Workflow)` deny matcher (blanket subagent-deny) **never
  fired** (empty log) — consistent: the seam was never reached.
- **Workflow-leaf spawn path**: [[mem_019f36028bca7411b33fde4981aaba85]] row 3 —
  a `Tools: *` def spawned as a workflow `agent()` leaf returns
  `Artifact, Bash, Edit, Read, ReportFindings, SendUserFile, Skill, ToolSearch,
  Write, StructuredOutput` — **no `Agent`, no `Workflow`**.

Both jailed spawn paths lack `Workflow`. Only the main thread (`agent_id=<NONE>`)
holds it, which is the legitimate `/drive-slice` launch surface.

## Defense-in-depth still ships

Closed-by-absence is a property of the CURRENT harness, not a guarantee. SL-206
PHASE-12's `check_spawn_seam_symmetry` doctor check enumerates
`SEAM_REGISTRY = {Agent, Workflow}` and reds if any known seam lacks a
`PreToolUse` matcher — so IF a future harness release exposes `Workflow` to a
subagent, conformance catches the ungated seam before it becomes an escalation
hole. The blanket subagent-deny matcher remains the correct authored posture.

Relates: [[mem_019f36028bca7411b33fde4981aaba85]] (workflow strips Agent — the
Agent-seam half), [[mem.fact.claude.pretooluse-agent-carries-spawner-id]]
(the Agent-seam active-deny gate, P3/P4).
