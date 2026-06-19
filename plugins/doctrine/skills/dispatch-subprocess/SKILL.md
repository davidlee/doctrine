---
name: dispatch-subprocess
description: The codex/pi arm of `/dispatch` — `doctrine worktree fork --worker` then spawn the worker as a subprocess with its cwd bound to the fork. Reached only from the `/dispatch` router on a codex/pi↔env-marker agreement; do not invoke directly.
---

# Dispatch — codex/pi arm

Spawn a worker via `doctrine worktree fork --worker` + subprocess spawn.
Drive loop lives in the [`/dispatch` router](../dispatch/SKILL.md).

## Spawn — codex arm
```sh
fork_env="$(doctrine worktree fork --base "$B" --branch "$BR" --dir "$D" --worker)" \
  || { echo "fork failed: $?" >&2; exit 1; }
env -C "$D" DOCTRINE_WORKER=1 $fork_env codex exec "<pre-distilled prompt>"
```
Confined (bwrap); **Never `eval`** — capture `$fork_env`, check `$?`, then spawn.
## Spawn — pi arm (RPC mode)
```sh
fork_env="$(doctrine worktree fork --base "$B" --branch "$BR" --dir "$D" --worker)" \
  || { echo "fork failed: $?" >&2; exit 1; }
cp AGENTS.md "$D/" \
  || { echo "AGENTS.md copy failed: $?" >&2; exit 1; }
PI_FIFO=$(mktemp -u) && mkfifo "$PI_FIFO"
{ printf '%s\n' \
    '{"type":"set_auto_retry","enabled":false}' \
    '{"type":"prompt","message":"<pre-distilled prompt>"}'
  sleep 300
} > "$PI_FIFO" &
timeout 300 env -C "$D" DOCTRINE_WORKER=1 $fork_env \
  pi --mode rpc --thinking off --session-dir "$D/.pi-session" \
     --no-extensions --no-skills --no-themes \
     --offline --approve --tools read,bash,edit,write,grep,find,ls \
  < "$PI_FIFO"
rm -f "$PI_FIFO"
```
Same confinement as codex arm; fifo keeps stdin open (pi RPC exits on EOF).
`sleep 300` keepalive, `agent_end` gives typed completion. Ignore
`extension_ui_request` widget events from installed packages.
## Red Flags
**Never:** `eval`; spawn outside `env -C "$D"`; omit `timeout`; use a heredoc
for RPC mode (stdin EOF kills pi).
**Always:** halt on fork failure; carry `DOCTRINE_WORKER=1`; use a fifo for RPC
stdin; `rm -f` the fifo after exit; return to the router for the funnel cadence.
