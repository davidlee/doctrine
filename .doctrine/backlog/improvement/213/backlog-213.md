# IMP-213: Auto-detect non-Claude agents during install and use configured repo

`doctrine install` currently only auto-detects Claude (`.claude/` dir). For
non-Claude agents you must pass `--agent pi` explicitly. If you don't, the
install silently skips skills installation — no hint, no prompt.

Additionally, the delegated `npx skills add <SOURCE>` uses a hardcoded
`DELEGATE_SOURCE = "davidlee/doctrine"` constant rather than the configured
`[install].repo` from `doctrine.toml`.

## Changes

1. **`detect_agents()`**: also check for `.codex/`, `.pi/`, `.agents/` dirs
   and auto-detect the corresponding non-Claude agents.

2. **`delegate_argv()`**: accept `repo: &str` parameter instead of hardcoded
   `DELEGATE_SOURCE`. Callers thread `[install].repo` through.

3. **Runner resolution**: new `resolve_runner()` — try `bunx` first, fall back
   to `npx`. Returns program name + Runner so the printed command matches what
   actually executes.
