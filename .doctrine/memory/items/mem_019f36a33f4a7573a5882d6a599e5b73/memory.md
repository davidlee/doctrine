# Orchestrator-unjail nomination round-trips: allowlist flips PreToolUse posture deny→PassThrough, unjailed orchestrator commits

**Claim.** The nomination leg of SL-206 full-commit unjail works end-to-end: a
`SubagentStart(dispatch-orchestrator)`-written PassThrough allowlist entry flips a
spawned subagent's `PreToolUse` posture from **deny → PassThrough**, scoped to
**only** the listed `agent_id` (a non-listed control agent stays jailed). The
unjailed orchestrator then runs Bash **unwrapped** — RW `.git` — and commits.

**Evidence (SL-206 P1, 2026-07-06, Claude Code 2.1.198).**
- **P1a (mechanism):** allowlist membership is the sole discriminator — the listed
  `agent_id` gets PassThrough ×N; a control `agent_id` is denied. The grant is
  per-identity, not blanket.
- **P1b (E2E):** `SubagentStart` **fired** (matched `dispatch-orchestrator`,
  carried `agent_id`+`agent_type`), wrote the allowlist **before the
  orchestrator's first Bash** (sync-blocking). `PreToolUse` saw
  `agent_id ∈ allowlist` → PassThrough. The orchestrator's Bash ran unwrapped:
  `git -C .dispatch/SL-206 rev-parse` → `9ba5c0a6` (a jailed agent cannot run
  this), empty commit landed **`98835cc`**. Fired under `CLAUDE_CODE_CHILD_SESSION=1`.
  `settings.local.json` hook edit **hot-reloaded** — no restart.

**Composition.** This is the *nomination* half of the escalation-closed loop; the
*gate* half (deny a subagent-initiated privileged spawn) is
[[mem.fact.claude.pretooluse-agent-carries-spawner-id]] (P3 discriminator + P4
deny E2E). Together — `SubagentStart` nominates ∧ `PreToolUse(Agent)` gate blocks
jailed→unjailed escalation — full-commit unjail is provably safe, no arming token.
Invariants the design must hold: gate privileged-set ≡ nomination-eligible-set;
allowlist/hook config outside every jail (main-thread-write only).

See `.doctrine/slice/206/unjail-direction.md` §6 (P1 RESULT).
