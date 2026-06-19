# pi RPC mode stdin lifecycle: heredoc closes stdin before model responds

pi v0.79.6 in RPC mode (`--mode rpc`) exits immediately when stdin closes (EOF),
even if a model call is in-flight. This means the heredoc approach used in the
dispatch-subprocess skill template silently fails:

```sh
pi --mode rpc ... <<'PI_MSGS'
{"type":"prompt","message":"..."}
PI_MSGS
```

What happens: pi accepts the prompt (success response), echoes the user message
as a `message_start`/`message_end` pair, then stdin EOF triggers process exit
before the model response arrives. No `agent_end` event is emitted. The session
file confirms the model DID respond — the events simply never reached stdout.

**Workaround 1** — named pipe (fifo) with delayed close:
```sh
mkfifo /tmp/pi-in
{ printf '...jsonl...\n'; sleep 300; } > /tmp/pi-in &
pi --mode rpc ... < /tmp/pi-in
```
The sleep keeps the write end open; pi stays alive and emits all events.

**Workaround 2** — use `-p` (print mode) instead of `--mode rpc`:
```sh
pi -p "prompt text"
```
Fire-and-forget, no stdin lifecycle issue, but no structured `agent_end` event.

**Design impact**: design.md D1 and the spawn template in both design.md and
dispatch-subprocess/SKILL.md assume the heredoc works. They need amendment to
either document the fifo pattern or fall back to `-p` with a deferred follow-up
for structured extraction (IMP-104).
