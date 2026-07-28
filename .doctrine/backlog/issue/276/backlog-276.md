# ISS-276: Research agent scripts hang on tool-using prompts

## Problem

`./scripts/pi-scout` and `./scripts/pi-research` hang indefinitely when the prompt
requires repo exploration. **3 of 3 threads failed** during the SL-237 pre-design
round (2026-07-29):

| thread | script | outcome |
|---|---|---|
| governance applicability | `pi-research` | 23 min, 0 bytes, killed manually |
| code map | `pi-scout` | 23 min, 0 bytes, killed manually |
| narrow spec sweep | `pi-research` | `timeout 900` → exit 124, 0 bytes |

The third was deliberately narrowed (two questions, 400-word ceiling) to test the
"prompt too heavy" hypothesis. It hung anyway.

## The tool itself is fine

```
$ timeout 120 ./scripts/pi-scout "Reply with exactly the word: ALIVE"
ALIVE     # exit 0, seconds
```

Model call, agent-profile parsing (`.pi/agents/scout.md`) and script plumbing all
work. **The hang is in the agentic tool-use loop** — it appears only when the
prompt asks the agent to read/grep the repo. Not yet isolated further: unknown
whether it is a `pi` tool-loop bug, a tool-permission prompt awaiting input that
never arrives (plausible — the process sleeps rather than spins), or a jail
interaction.

Next diagnostic step: run `pi` directly with `--tools` and a single trivial tool
call (e.g. "list files in ./scripts") to see whether ANY tool use completes, and
check whether it blocks on stdin.

## Why it matters

`CLAUDE.md` § *Research agents* makes these the **default** for the `/research`
pre-design round and explicitly says *"Do NOT use harness subagents for research
in this repo, use the above liberally to preserve interactive session context."*
With them broken, `/research` silently degrades to self-serve — which works, but
consumes exactly the interactive context the guidance exists to protect.

## Second-order defect: silent failure mode

`pi --print` buffers all output until completion, so a redirect target stays at 0
bytes throughout. **A working thread and a hung thread are indistinguishable**
from the orchestrator's side — no progress, no heartbeat, no stderr. Diagnosis
required `ps -eo pid,etime,comm | awk '$3=="pi"'` (an earlier `ps aux | grep`
missed the exec'd process).

Even once fixed, the scripts should either stream, emit a stderr heartbeat, or
have the guidance document the silence plus the `ps` check.

## Mitigations in the meantime

- Always wrap in `timeout <n>` — an unbounded thread will sit forever.
- Poll with `ps -eo pid,etime,comm | awk '$3=="pi"'`.
- Expect to self-serve; budget interactive context accordingly.

## Related

- obs `019fa932-8c8f` — first capture (buffering / indistinguishable-from-hung).
- obs `019fa953-d669` — refined capture (3/3 failure + ALIVE smoke test).
- SL-237 — the slice whose research round degraded; `research.md` header records it.
- RFC-011 — dispatch token efficiency; this is a direct hit on the context-preservation
  strategy that RFC's instrumentation exists to measure.
