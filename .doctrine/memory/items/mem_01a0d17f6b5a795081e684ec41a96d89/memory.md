Observed live on 2026-09-24 (codex-cli 0.155.1, hook stdin teed) and
cross-checked against the published contract and binary. Captured payloads:
`tests/fixtures/codex/` in the doctrine repo.

- **`PreToolUse` payload**: `session_id`, `turn_id`, `transcript_path`, `cwd`,
  `hook_event_name`, `model`, `permission_mode`, `tool_name`, `tool_input`,
  `tool_use_id`. **No `agent_id`.**
- `tool_name` is `Bash` for any shell call and `apply_patch` for a patch; the
  patch body arrives in `tool_input.command`. `tool_input.command` is a
  **string**, and it is the **model's own command** (`echo hi`) — codex wraps it
  as `bash -lc '…'` only at exec time, after the hook — so a token-prefix
  command match is not defeated by a wrapper.
- `apply_patch` header paths can be **absolute** (`*** Add File: /abs/path`),
  not only the docs' relative form.
- There is **no read tool**: file reads go through the shell and match `Bash`; a
  shell file write (`printf … > f`) is also `Bash`, so a path-scoped memory
  cannot match it. `apply_patch` is the only path trigger.
- **No `apply_patch` executable on PATH**: the "patch via shell" form does not
  arise; a shell-run patch would report `Bash` and reach the command surface.
- `hooks.<Event>[].hooks[]` is the **handler**; `additionalContextLimit` and
  `timeout` are handler fields, beside `command`. Written on the matcher group
  they are ignored (at best). A canonicality check comparing only `command`
  misses a missing or stale limit and never heals it.
- Canonical hook `tool_name` values: `Bash` (shell AND unified exec are both
  canonicalised to `Bash`) and `apply_patch`. Matcher aliases `Edit`/`Write` are
  accepted for apply_patch, but the input still reports
  `tool_name: "apply_patch"`.
- Trust: non-managed hooks must be reviewed/trusted before they run, and trust is
  keyed to a **hash of the handler definition** (the binary exposes a per-hook
  `trusted_hash`). A baked absolute exec path changes on every upgrade, re-arming
  the review; codex then skips the hook silently. `codex exec
  --dangerously-bypass-hook-trust` runs enabled hooks without persisted trust.
- `PreToolUse` carries no `agent_id`; subagent hooks report the **parent**
  session id, so a seen-set keyed on session_id is shared across the
  parent/subagent boundary.
