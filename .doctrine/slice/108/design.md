# Design: pi dispatch worker integration via RPC mode

## Current behavior

`dispatch-subprocess/SKILL.md` documents one spawn arm:

```sh
fork_env="$(doctrine worktree fork --base "$B" --branch "$BR" --dir "$D" --worker)" \
  || { echo "fork failed: $?" >&2; exit 1; }
env -C "$D" DOCTRINE_WORKER=1 $fork_env codex exec "<pre-distilled prompt>"
```

The `fork_env` contract (per-worktree env emission from `doctrine worktree fork
--worker`) is harness-identical — it emits `KEY=value` pairs on stdout that any
subprocess harness can inject. Only `codex exec` is documented as a consumer.

## Target behavior

Two documented spawn arms in the same skill, with pi RPC mode as the recommended
posture for subprocess dispatch workers.

### Spawn template (pi arm)

```sh
fork_env="$(doctrine worktree fork --base "$B" --branch "$BR" --dir "$D" --worker)" \
  || { echo "fork failed: $?" >&2; exit 1; }
env -C "$D" DOCTRINE_WORKER=1 PI_OFFLINE=1 $fork_env \
  pi --mode rpc --no-session --session-dir "$D/.pi-session" \
     --no-extensions --no-skills --no-themes \
     <<< '{"type":"prompt","message":"<pre-distilled prompt>"}'
```

**Flag rationale:**

| Flag | Reason |
|---|---|
| `--mode rpc` | Structured JSONL protocol; `agent_end` gives typed completion signal with full message array |
| `--no-session` | Ephemeral worker life; `--session-dir` colocated with fork for post-mortem debug |
| `--session-dir "$D/.pi-session"` | Session survives fork lifecycle; inspectable during the batch if worker fails; GC'd with fork |
| `--no-extensions` | No project-local extension surface — prevents `extension_ui_request` dialog hazards in RPC mode |
| `--no-skills` | No doctrine skill corpus in context; the boot sector references skills the worker won't have (pi handles missing skill references gracefully) |
| `--no-themes` | Unnecessary in headless mode |
| `PI_OFFLINE=1` | Suppress startup version check + package update checks — no network needed |

**Not passed:** `--no-context-files`. The fork worktree gets AGENTS.md on disk
(provisioned by the orchestrator, or by the `coordinate` verb during fork setup).
pi auto-discovers it from cwd; the boot sector (`@.doctrine/state/boot.md`)
dereferences governance and memory pointers. The worker gets project conventions
without the orchestrator spending tokens to inline them.

### Tool profile

Full built-in set: `read`, `bash`, `edit`, `write`, `grep`, `find`, `ls`.

`grep` and `find` are included because workers shouldn't burn turns doing
`bash "grep -r pattern src/"` and parsing raw output — structured tools are
faster and lower decision cost. `ls` is marginal but costs nothing to include.

### Print mode (documented alternative)

```sh
env -C "$D" DOCTRINE_WORKER=1 PI_OFFLINE=1 $fork_env \
  pi -p --no-extensions --no-skills --no-themes \
     --no-session --session-dir "$D/.pi-session" \
     "<pre-distilled prompt>" 2>&1
```

Fire-and-forget. No structured completion signal; the orchestrator gates on exit
code only. Documented as a simpler alternative when structured outcome extraction
isn't needed.

## Design decisions

### D1 — RPC mode for structured completion

The orchestrator reads JSONL from pi's stdout, discards all events except
`agent_end`. From `agent_end.messages` it extracts:

1. **Outcome:** last assistant message text → worker summary/findings
2. **Errors:** any `toolResult` message with `isError: true`
3. **Dirty check:** presence of tool calls → worker touched files (funnel delta-check confirms authoritatively)

No custom extension or filter. The `agent_end` event is the single integration
point. Intermediate events (`message_update`, `tool_execution_update`) are
discarded — the orchestrator doesn't steer workers mid-execution.

**Why RPC over print mode:** `-p` gives unstructured text output with no way to
separate the worker's final summary from streaming chatter. RPC's `agent_end`
event carries typed messages in machine-readable form.

### D2 — Filesystem-native context, not prompt-inlined

The fork worktree carries AGENTS.md on disk (provisioned during fork setup). pi
auto-discovers it at startup. The boot sector dereferences governance, routing
table, and memory signposts. The worker prompt is lean:

> Implement phase N of SL-NNN. Conventions are in AGENTS.md. Run `doctrine memory find <topic>` for subsystem gotchas.

**Why not inline conventions in the prompt:** burns orchestrator tokens every
batch. The orchestrator already provisions the fork — adding AGENTS.md to the
provision payload is zero marginal cost.

**Why `--no-skills`:** the boot text references skills the worker won't have (the
doctrine skill corpus is orchestrator-only). pi handles missing SKILL.md
references gracefully — they don't appear in context. If a worker genuinely needs
a specific skill, the orchestrator passes `--skill /path/to/skill` explicitly.

### D3 — Harness selection via doctrine.toml config + CLI override

A `[dispatch]` section in `doctrine.toml` carries the preferred subprocess
harness:

```toml
[dispatch]
preferred_subprocess_harness = "pi"  # default "codex" for backward compat
```

The skill reads this to choose the spawn arm. An absent preferred harness on PATH
is a clean error ("pi not found — install pi or set preferred_subprocess_harness
to codex"). A future `--harness` CLI flag on `worktree fork` can provide
per-invocation override.

**Deferred to IMP-101** (`dispatch: deliver_to config field in doctrine.toml
[dispatch] section`), which already scopes the `[dispatch]` config block. This
slice does not implement the config — it documents the expected key and makes the
pi arm a live target.

### D4 — Session colocation with fork lifecycle

`--session-dir "$D/.pi-session"` writes the session into the fork directory.
Colocated with the fork lifecycle — GC'd together when the fork is reaped.
Available for post-hoc inspection during the batch if the worker fails, without
adding an orchestrator funnel step.

### D5 — Trust posture

**Probe result:** pi RPC mode in the doctrine jail accepts prompts immediately
with `defaultProjectTrust: "ask"` — no trust hang. `PI_OFFLINE=1` suppresses
startup network checks. No `--approve`/`--no-approve` flag needed for the default
case.

The trust behavior is **project-config-dependent**. If a future project adds
project-local pi extensions and expects them in workers, the config posture may
need adjustment. For doctrine's own repo, no changes needed.

## Code impact

**Zero code changes.** This is a skill documentation change + an e2e exercise.

| File | Change |
|---|---|
| `.agents/skills/dispatch-subprocess/SKILL.md` | Add pi RPC spawn arm alongside `codex exec` |
| (exercise only) fork worktree | e2e validation of fork→marker→spawn→agent_end→import→commit |

No CLI verbs, no binary changes, no extension development. The existing
`doctrine worktree fork --worker` contract already serves pi identically.

## Verification

- **VA** — e2e exercise: fork a worktree, spawn pi RPC worker with a
  self-contained prompt, validate the worker completes and produces a clean delta,
  run the import funnel to completion.
- **VA** — the spawn template in the updated skill produces a running pi worker
  when copied into a shell.
- **VA** — pi worker completes with `agent_end` containing typed messages; no
  `extension_ui_request` dialogs block the run (confirmed by `--no-extensions`).

No VT tests — no code changes.

## Open questions (resolved in design)

- Session hygiene → D4: colocated with fork, `--no-session` + `--session-dir`
- Tool profile → full built-in set (read, bash, edit, write, grep, find, ls)
- Trust gate → D5: probe confirms no hang; project-config-dependent, ok for v1
- Harness selection → D3: `[dispatch] preferred_subprocess_harness` in doctrine.toml (deferred to IMP-101)
- Context delivery → D2: filesystem-native AGENTS.md, lean prompt
- Scout/fix-agent generalisation → out of scope for this slice; tracked IMP-104

## Follow-ups

- **IMP-101** — `[dispatch] preferred_subprocess_harness` config + CLI `--harness` flag
- **IMP-104** — General pi subagent spawn pattern for scout/fix-agent roles
- **IDE-012** — Read-only doctrine memory retrieval tool for agent harnesses
