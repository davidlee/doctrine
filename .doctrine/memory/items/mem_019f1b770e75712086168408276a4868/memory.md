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

## ⚠️ CONTRADICTED on macOS (SL-183 PHASE-04, 2026-07-01) — trust downgraded

On macOS this session, `/reload-plugins` reported "3 hooks" but the PreToolUse
Bash wall did **NOT** fire for `isolation:worktree` subagents afterward: a
subagent's in-wt `echo` succeeded UNWRAPPED (no `<wt>/.tmp/jail.sb` materialized —
the T3b Seatbelt profile is only written on the wrap path, so its absence proves
the hook never wrapped), and a logging shim wired into the Bash matcher logged
nothing. Two spawns, zero interception. The higher-trust, VERIFIED
[[mem_019f18d2a9307cc38d5e4ba9749e6208]]
(`mem.pattern.dispatch.claude-worktree-subagent-bwrap-confinement`) states the
opposite for the settings.local.json path: **"Hook *registration* loads at
session start ONLY — no hot-reload."** The original evidence here was Linux +
the `settings.local.json` `hooks` block; the doctrine *skill/plugin* hooks.json
path on macOS did not reproduce it. So: `/reload-plugins` is NOT a reliable
substitute for a session-start restart for the plugin-hooks registration surface —
at least on macOS this build. Prefer a full restart + confirm the hook actually
fires (materialized `.sb` / a shim log), never trust the "N hooks" count as proof
of live interception.
