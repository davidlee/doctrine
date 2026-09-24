# Codex `PreToolUse` wire fixtures (SL-263)

Verbatim captures of codex's `PreToolUse` hook stdin, taken live on 2026-09-24
with **codex 0.155.1** (`codex-cli 0.155.1`), model **`gpt-6-luna`**. The JSON is
the exact payload codex wrote to the hook subprocess's stdin, pretty-printed for
review; nothing is hand-authored. They are the fixtures the SL-263 codec VTs run
on, and they retire the design's `A-1`.

## How they were captured

A throwaway project hook teed stdin and echoed `{}` back:

```json
{ "hooks": { "PreToolUse": [
  { "matcher": "Bash|apply_patch",
    "hooks": [ { "type": "command", "command": "/tmp/codex-capture/capture-pre.sh" } ] } ] } }
```

run as, from the scratch cwd:

```
codex exec -m gpt-6-luna --dangerously-bypass-hook-trust --skip-git-repo-check "<prompt>"
```

`--dangerously-bypass-hook-trust` runs the untrusted throwaway hook for the one
invocation. Hooks are **subprocesses** — codex writes the payload to the hook's
stdin — so there is no port or IPC channel involved.

## The fixtures

| file | prompt | `tool_name` |
|---|---|---|
| `shell.json` | "Run the shell command: echo hi." | `Bash` |
| `shell_write.json` | "…create the file note.txt containing the text hello." | `Bash` |
| `apply_patch_tool.json` | "Use the apply_patch tool (do not use the shell) to create a file named patched.txt containing the line: hello patch" | `apply_patch` |

## The three unknowns, answered

1. **String vs argv `command`.** A **string**, on both wires:
   `"command": "echo hi"` and `"command": "*** Begin Patch\n…"`. The design's
   tolerant reader (join an argv vector with spaces) is kept as a defensive
   measure, but the observed shape is a string.
2. **Model command vs wrapper.** The **model's own command**. codex wraps it as
   `bash -lc 'echo hi'` only at *exec* time, after the hook fires; the payload
   carries the bare `echo hi`. So `command_admits`'s token-prefix match is not
   defeated by a wrapper.
3. **`apply_patch` through the shell.** **Not reachable in this version.** There
   is no `apply_patch` executable on `PATH`; codex exposes `apply_patch` as a
   distinct tool (fixture `apply_patch_tool.json` reports `tool_name:
   "apply_patch"`), and the model used the shell (`printf … > file`) for plain
   writes — which reports `tool_name: "Bash"` (`shell_write.json`). A shell-run
   patch would therefore reach the **command** surface, not the path surface;
   that is the codex delta §5.7 records, with the mechanism now observed. There
   is consequently no `apply_patch_via_shell.json`.

## Other observations

- **No `agent_id`.** The payload carries `session_id`, `turn_id`,
  `transcript_path`, `cwd`, `hook_event_name`, `model`, `permission_mode`,
  `tool_name`, `tool_input`, `tool_use_id` — and no agent identity, confirming
  §5.2/§5.7's main-thread-gate claim for codex.
- **`apply_patch` header paths may be absolute.** `apply_patch_tool.json` writes
  `*** Add File: /tmp/codex-capture/patched.txt` — absolute, not the
  `path/to/file.py` form the docs show. `probe_for` already anticipates both
  (rebase an absolute value under the reported cwd onto the canonical anchor,
  then strip the root).
