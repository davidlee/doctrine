# PreToolUse(Agent) carries the spawner's agent_id — the parentage discriminator SubagentStart lacks

**Claim.** A `PreToolUse` command hook on matcher `Agent` fires for every
`Agent`-tool spawn, and its payload carries the **caller's** `agent_id` **iff a
subagent initiated the spawn** (absent — main-thread call — when the unconfined
principal spawns), plus `tool_input.subagent_type` naming the target. This is the
spawner-identity discriminator that `SubagentStart`'s payload **lacks**
(`SubagentStart` attributes `session_id`/`prompt_id`/`transcript_path` to the
top-level session regardless of nesting depth —
[[mem.fact.claude.subagentstart-fires-nested-no-parent-discriminator]]). ⇒
escalation can be closed at the **spawn seam** without an arming token.

**Evidence (SL-206 P3, 2026-07-06, Claude Code 2.1.198).** Observer matcher
`Agent` on `PreToolUse`; nested spawn `main → spawner(general-purpose) → leaf`.
Two `PreToolUse(Agent)` log entries under one `session_id`:
- **outer** (main-thread call): `agent_id="<NONE>"`, `target_subagent_type="general-purpose"`.
- **inner** (spawner-initiated): `agent_id="a1904cf1da408e13e"` (= the spawner's
  own agent id), `agent_type="general-purpose"`, `target_subagent_type="general-purpose"`.

Preserved: `scratchpad/p3-pretooluse-agent.log`. Docs corroborate
(`docs/claude/hooks.md:595`: `agent_id` "Present only when the hook fires inside a
subagent call. Use this to distinguish subagent hook calls from main-thread
calls"; `:1412`: `tool_input.subagent_type` for the Agent tool).

**Crux — the child-session env does NOT contaminate it.** The main-thread call
read `agent_id="<NONE>"` **even though this session carries
`CLAUDE_CODE_CHILD_SESSION=1`** (the P1b surprise). So that env var — unreliable
as an "am I nested" signal — does not leak into the PreToolUse discriminator. The
new `Agent` matcher **hot-reloaded** (no restart), matching the P1b
settings.local.json finding.

**Consequence — SL-206 §5 "Option A" spawn-gate (no arming token).** Add matcher
`Agent` to the jail's `PreToolUse` hook and **deny the spawn** iff
`agent_id present ∧ caller ∉ allowlist ∧ subagent_type ∈ privileged-set`
(`dispatch-orchestrator`, `fork`, …). Escalation closes at the source, so
`SubagentStart` type-match becomes trustworthy again and the unjail-nomination
leg needs **no** single-slot arming token — the token's only job was
compensating for `SubagentStart`'s missing parentage
([[mem.fact.dispatch.single-slot-arming-rendezvous]] not needed here). Nominated
(unjailed) actors pass by allowlist membership.

**Deny proven E2E (SL-206 P4, 2026-07-06).** An *active* deny gate (emits
`{"hookSpecificOutput":{"permissionDecision":"deny",…}}` on stdout; silent exit 0
= allow — `docs/claude/hooks.md:140`) was run live. Main thread spawned a
`general-purpose` "attacker"; it attempted two nested spawns. Log:
`{agent_id:<NONE>, target:general-purpose, allow}` (my spawn of the attacker),
`{agent_id:a551f6df03874e447, target:dispatch-orchestrator, DENY}` (escalation
attempt — **the orchestrator never spawned**, attacker reported `BLOCKED`),
`{agent_id:a551f6df03874e447, target:general-purpose, allow}` (benign nested spawn
succeeded, returned `BENIGN-OK`). ⇒ (1) the deny **actually prevents** the spawn,
not just warns; (2) the gate is **scoped** — a subagent's benign nested spawns
still pass; (3) the legit main-thread nomination path (`agent_id=<NONE>`) is
**not over-blocked**. Preserved: `scratchpad/p4-spawn-gate.{log,sh}`. Full-commit
unjail's escalation-closed safety bar is met (`PreToolUse(Agent)` deny ∧
`SubagentStart` nomination — the latter round-tripped in P1).

**Generalizes — standing property of the SubagentStart seam.** Every
`SubagentStart` type-match grant is invocable by any jailed `Agent`-holder (the
hook can't see who asked). The shipped `dispatch-worker` provisioning matcher is
therefore escalatable today — bounded (worker commits land on an ephemeral
`dispatch/<name>` fork behind the commit gate; nothing reaches trunk without an
orchestrator import), but it is an ADR-008 threat-model line, and the PreToolUse
spawn-gate is the fix pattern. Rule: grant power must scale with intent evidence —
type-match alone buys bounded/gated capabilities (worker tier); anything unbounded
(PassThrough) requires the spawn-gate.

**Also — IMP-269 /fork.** Subagent-initiated `/fork` goes through the Agent tool
⇒ gated by the same rule; user-typed `/fork` either fires `PreToolUse(Agent)` with
no `agent_id` (passes) or bypasses tools entirely (never gated) — both land on
"user forks pass, subagent forks denied", the intended trust model. (User-typed
`/fork` PreToolUse behavior itself is **un-probed** — informational, not
load-bearing; Option A holds either way.)

See `.doctrine/slice/206/unjail-direction.md` §5, and the memory it refines:
[[mem.fact.claude.subagentstart-fires-nested-no-parent-discriminator]].
