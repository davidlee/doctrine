---
name: dispatch-subprocess
description: The codex/pi arm of `/dispatch` — `doctrine worktree fork --worker` then spawn the worker as a subprocess with its cwd bound to the fork. Reached only from the `/dispatch` router on a codex/pi↔env-marker agreement; do not invoke directly.
---

# Dispatch — codex/pi arm

Spawn a worker via `doctrine worktree fork --worker` + subprocess spawn.
Drive loop lives in the [`/dispatch` router](../dispatch/SKILL.md).

## Spawn — codex arm
```sh
doctrine worktree fork --base "$B" --branch "$BR" --dir "$D" --worker \
  || { echo "fork failed: $?" >&2; exit 1; }
env -C "$D" DOCTRINE_WORKER=1 codex exec "<pre-distilled prompt>"
```
Confined (bwrap); run the fork, check `$?`, then spawn. The worker inherits the
ambient env and defaults `CARGO_TARGET_DIR` to its own in-tree `$D/target`.
## Spawn — pi arm (RPC mode)
```sh
doctrine worktree fork --base "$B" --branch "$BR" --dir "$D" --worker \
  || { echo "fork failed: $?" >&2; exit 1; }
cp AGENTS.md "$D/" \
  || { echo "AGENTS.md copy failed: $?" >&2; exit 1; }
PI_FIFO=$(mktemp -u) && mkfifo "$PI_FIFO"
{ printf '%s\n' \
    '{"type":"set_auto_retry","enabled":false}' \
    '{"type":"prompt","message":"<pre-distilled prompt>"}'
  sleep 300
} > "$PI_FIFO" &
timeout 300 env -C "$D" DOCTRINE_WORKER=1 \
  pi --mode rpc --thinking off --session-dir "$D/.pi-session" \
     --no-extensions --no-skills --no-themes \
     --offline --approve --tools read,bash,edit,write,grep,find,ls \
  < "$PI_FIFO"
rm -f "$PI_FIFO"
```
Same confinement as codex arm; fifo keeps stdin open (pi RPC exits on EOF).
`sleep 300` keepalive, `agent_end` gives typed completion. Ignore
`extension_ui_request` widget events from installed packages.

## Boundary recording
At the funnel **Record** beat (router step 8), after the code commit:
`doctrine slice record-delta <SL> PHASE-NN --start <B> --end <B+1>` — writes the
per-phase boundary into the primary-tree conformance registry (F-5 resolves it
from the coord tree; F-6 guard; upsert). This arm has no `record-boundary`, so
this is its only conformance write; orchestrator-issued, every landed phase.

## Red Flags
**Never:** `eval`; spawn outside `env -C "$D"`; omit `timeout`; use a heredoc
for RPC mode (stdin EOF kills pi).
**Always:** halt on fork failure; carry `DOCTRINE_WORKER=1`; use a fifo for RPC
stdin; `rm -f` the fifo after exit; return to the router for the funnel cadence.
