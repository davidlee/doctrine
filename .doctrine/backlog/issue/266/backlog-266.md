# ISS-266: pi wrapper scripts leave stdin open so pi never self-exits

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Observed

`scripts/pi-scout` and `scripts/pi-research` both end in:

```bash
exec pi --print --model "$model" --system-prompt "$sysprompt" \
  --tools "$tools" --thinking "$thinking" --no-skills --no-context-files "$query"
```

with **stdin inherited**. When the query arrives as an argument, the scripts'
own `[[ ! -t 0 ]]` stdin branch never fires, so nothing consumes or closes fd 0
and it passes straight into `pi`.

`pi` does not self-exit while stdin is held open. This is known and documented
in `scripts/pi-spawn-confined.sh:13-14`:

> Breaks on the pi `agent_end` event instead of waiting out the timeout (fifo
> holds stdin open so pi never self-exits; we kill it on completion).

That script handles it properly: `timeout "$BACKSTOP"` around the exec
(line 119), a poll loop for the typed `agent_end` event, then `kill -9` on the
pi pid (lines 126-141). The two research wrappers have none of that — no
`</dev/null`, no `timeout`, no completion detection, no kill.

## Impact

Thread termination becomes luck-dependent on whatever the caller does with fd 0.
Observed during SL-233 plan-grounding research (2026-07-27): six threads
launched identically from Claude Code background shells, four returned complete
artefacts, two produced zero bytes and vanished with no exit status — twice in a
row, across a re-fire. Time lost to diagnosis exceeded the research itself.

The failure is indistinguishable from a crash at the call site: empty output
file, no error, no exit code. Nothing in the wrapper's own output says "I am
waiting on stdin."

## Suggested fix

Mirror what `pi-spawn-confined.sh` already proved:

1. Close stdin for the argument-query path — `exec pi … "$query" < /dev/null`.
   This alone fixes the common case and is a one-token change.
2. Add a backstop `timeout` so a wedged run fails loudly with a non-zero status
   instead of hanging indefinitely.
3. Optionally lift the `agent_end` poll-and-kill from `pi-spawn-confined.sh` if
   the `--print` path ever proves to hang even with stdin closed.

Callers should not have to know this. Until fixed, invoke as
`timeout 900 ./scripts/pi-scout … < /dev/null`.

Sibling of ISS-265 — same two scripts, unrelated fault. Diagnosed by the user,
who recognised the symptom from `pi-spawn-confined.sh`. See
`.doctrine/rfc/011/case-notes.md` (`[preflight; sl233-plan-research-20260727]`).
