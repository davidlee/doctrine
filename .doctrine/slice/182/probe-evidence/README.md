# SL-182 PHASE-01 — live harness probe evidence

Harvested 2026-07-01 (post-restart live battery). Verdict: **all 3 items green →
Path L + plugin registration**. Narrative + verbatim verdict in the slice
`notes.md` ("PHASE-01 LIVE BATTERY VERDICT"). Durable fact recorded as
`mem.fact.claude.subagentstop-awaited-tree-intact-capture-seam`.

Environment: claude-code 2.1.x / NixOS bwrap 0.11.2. Two `isolation:worktree`
`general-purpose` subagents driven via the Agent tool:
- `a9fa4285c23fc26f7` — escape battery (T1) + uncommitted tracked+untracked change (T2 capture).
- `abdb78d35c75d448d` — one-shot `BLOCK_ONCE` exit-2 (T2 blocking assertion).

## Layout
- `logs/wrapper.log`      — PreToolUse(Bash) wrap: WRAP per subagent, PASS-THROUGH orchestrator, bwrap deny reasons.
- `logs/pathcheck.log`    — PreToolUse(Edit|Write) pathwall: DENY /tmp, PASS in-worktree.
- `logs/subagent.log`     — SubagentStart recorder (agent_id→cwd map writes).
- `logs/subagentstop.log` — SubagentStop: STOP / CORRELATE-OK via=a / CAPTURE-OK tree-intact / BLOCK-PROBE exit 2.
- `logs/subagentstop.payload.log`  — full stop payload (assert NO worktree_path — RV-202).
- `logs/subagentstart.payload.log` — full start payload (agent_id + cwd=worktree).
- `captured/<id>.patch|.untracked|.tgz` — diffs captured OUTSIDE the worktree before teardown (the Path-L funnel-import proof).
- `scripts/` — the probe apparatus (reproducible): pretooluse-wrap.sh, pretooluse-pathcheck.sh,
  subagentstart-record.sh, subagentstop-capture.sh, canaries/{setup,verify}.sh + manifest.

## Re-arm
Live apparatus remains at `.harness/probe/` (gitignored). To re-run: restore the
plugin `PreToolUse` entries into `.claude/skills/doctrine/hooks/hooks.json` and the
`SubagentStart`/`SubagentStop` entries into `.claude/settings.local.json` (baselines
kept at `.harness/probe/*.baseline` / `*.presnapshot`), then restart the session.
