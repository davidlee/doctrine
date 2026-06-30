# PreToolUse hooks fail OPEN — only exit 2 blocks

A `PreToolUse` hook command blocks the tool call **only on exit code 2**. Any
other non-zero exit — including `command-not-found` (127) from a missing/stale
binary — is a **non-blocking error**: the harness logs it and the **tool call
proceeds**. Source: `docs/claude/hooks.md:629-643` + the Warning ("only exit
code 2 blocks the action"; "exit code 1 … proceeds with the action"). The lone
exception is `WorktreeCreate`, where any non-zero aborts.

**Consequence for confinement/enforcement hooks:** a hook that enforces a jail
MUST invoke a **guaranteed-present, resolved** executable. A bare `doctrine` on
PATH that resolves to a stale binary lacking the subcommand — or is absent —
fails OPEN and lets the guarded tool run unconfined (exactly the RSK-014 hole
SL-182 closes). Register with an **absolute** resolved exec path, or wrap in a
shim that `exit 2`s on exec/`not-found`, so a missing binary **denies**.

- Verified against `docs/claude/hooks.md` (local official cache) + RSK-014
  probe-h1, 2026-07-01. Deny is normally expressed in JSON on exit 0; exit 2 is
  the hard fallback when the hook process itself cannot run.
- See [[mem.fact.claude.worktree-remove-auto-teardown]],
  [[mem.pattern.dispatch.claude-worktree-subagent-bwrap-confinement]]. Origin:
  SL-182 RV-200 F-2.
