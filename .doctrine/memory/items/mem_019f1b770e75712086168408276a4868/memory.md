# /reload-plugins registers plugin PreToolUse hooks, no restart

After `doctrine install` (re)writes `.claude/skills/doctrine/hooks/hooks.json`,
running `/reload-plugins` in the live session is enough to REGISTER the new
`PreToolUse` walls — a full session restart is NOT required (contra the earlier
assumption in SL-182 PHASE-01/03 notes, which said restart). Verified SL-182
PHASE-03 VA-1: pre-reload a worktree subagent's `/tmp` escape write succeeded
(hook not yet live); after `/reload-plugins` ("6 hooks") the same escape was
denied and in-worktree writes allowed.

Consequence: the installed-hook live battery is runnable WITHIN one session —
install → `/reload-plugins` → spawn an `isolation:worktree` subagent → observe.
No handoff-to-fresh-session needed purely to pick up freshly-installed hooks.

Pairs with [[mem_019f1b76ac487722bea2bf7d898a7dad]] (anchor present in hook env)
and `mem.fact.claude.pretooluse-hook-fail-open`.
