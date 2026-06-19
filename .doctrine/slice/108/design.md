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

The orchestrator writes AGENTS.md into the fork before spawning (pi does not discover
it from a parent directory — the fork is not under the project root):

```sh
fork_env="$(doctrine worktree fork --base "$B" --branch "$BR" --dir "$D" --worker)" \
  || { echo "fork failed: $?" >&2; exit 1; }
cp AGENTS.md "$D/" \
  || { echo "AGENTS.md copy failed: $?" >&2; exit 1; }
timeout 300 env -C "$D" DOCTRINE_WORKER=1 $fork_env \
  pi --mode rpc --thinking off --session-dir "$D/.pi-session" \
     --no-extensions --no-skills --no-themes \
     --offline --approve --tools read,bash,edit,write,grep,find,ls \
  <<'PI_MSGS'
{"type":"request","method":"set_auto_retry","params":{"enabled":false}}
{"type":"prompt","message":"<pre-distilled prompt>"}
PI_MSGS
```

`<<'PI_MSGS'` is a bash heredoc with quoted delimiter (no variable expansion);
the dispatch jail uses bash. Non-bash orchestrators replace with
`printf '%s\n' '<json1>' '<json2>' | pi …`.

**Flag rationale:**

| Flag | Reason |
|---|---|
| `--mode rpc` | Structured JSONL protocol; `agent_end` gives typed completion signal with full message array |
| `--thinking off` | Workers implement code changes; extended reasoning is wasteful. Overridable per-phase for reasoning-heavy tasks |
| `--session-dir "$D/.pi-session"` | Session colocated with fork; inspectable during the batch if worker fails; GC'd with fork |
| `--no-extensions` | No project-local extension surface — prevents `extension_ui_request` dialog hazards in RPC mode |
| `--no-skills` | No doctrine skill corpus in context; the boot sector references skills the worker won't have (pi handles missing skill references gracefully) |
| `--no-themes` | Unnecessary in headless mode |
| `--offline` | Suppress startup version check + package update checks — no network needed |
| `--approve` | The orchestrator trusts its own worker; no trust hang. Projects with stricter trust may override |
| `--tools read,bash,edit,write,grep,find,ls` | pi's default is `read,write,edit,bash` — `grep`,`find`,`ls` are explicitly enabled. Structured tools reduce turn-count vs raw bash calls |
| `set_auto_retry` (RPC) | Disabled — the orchestrator owns retries, not the harness. Sent as the first RPC message before the prompt |

**Timeout:** `timeout 300` enforces a 300s wall-clock deadline inclusive of all
retry attempts. On expiry: `timeout` sends SIGTERM to pi (grace); if the process
hasn't exited within 10s, SIGKILL. The orchestrator treats a timeout exit (code
124) as a failed worker and proceeds to the next phase or aborts the batch per
existing dispatch error-handling rules.

**Not passed:** `--no-context-files`. The orchestrator `cp`s AGENTS.md into the
fork after creation. pi auto-discovers it from cwd; the boot sector
(`@.doctrine/state/boot.md`) dereferences governance and memory pointers. The
worker gets project conventions without the orchestrator spending tokens to
inline them. `doctrine worktree fork --worker` does not provision AGENTS.md
(it is not in `.worktreeinclude`).

### Tool profile

pi v0.79.6 default built-in set: `read`, `write`, `edit`, `bash`. The spawn
template explicitly enables `grep`, `find`, `ls` via `--tools`. Structured tools
are faster and lower decision-cost than raw `bash "grep -r pattern src/"` calls.
`ls` is marginal but costs nothing to include.

### Print mode (documented alternative)

```sh
cp AGENTS.md "$D/" \
  || { echo "AGENTS.md copy failed: $?" >&2; exit 1; }
timeout 300 env -C "$D" DOCTRINE_WORKER=1 $fork_env \
  pi -p --thinking off --no-extensions --no-skills --no-themes \
     --offline --approve --tools read,bash,edit,write,grep,find,ls \
     "<pre-distilled prompt>" 2>&1
```

Fire-and-forget. No structured completion signal; the orchestrator gates on exit
code only. Documented as a simpler alternative when structured outcome extraction
isn't needed.

## Design decisions

### D1 — RPC mode for structured completion

The orchestrator reads JSONL from pi's stdout, discards all events except
`agent_end`. From `agent_end.messages` it extracts:

1. **Outcome text** (fallback ladder):
   1. Last `content` block of `type: "text"` from the last `role: "assistant"` message
   2. First `content` block of any type from the last `role: "assistant"` message (tool-call-only turn)
   3. `status: "no_output"` — no assistant messages at all
2. **Errors:** any `toolResult` message with `isError: true` → worker hit a problem. Tally as `error_count`.
3. **Dirty check:** presence of tool calls → worker touched files (funnel delta-check confirms authoritatively)
4. **Structured outcome:** derive `status` = `"success"` (outcome text present, no errors) | `"partial"` (outcome text present with errors) | `"no_output"` (no extractable text)

No custom extension or filter. The `agent_end` event is the single integration
point. Intermediate events (`message_update`, `tool_execution_update`) are
discarded — the orchestrator doesn't steer workers mid-execution.

**Timeout:** 300s wall-clock enforced by `timeout` in the spawn template (see
Spawn template above). The deadline is inclusive of all auto-retry attempts — the
orchestrator does not implement its own timer; it relies on `timeout` exit code
124 to detect worker timeout and proceeds per existing dispatch error-handling
rules (abort phase, report, advance to next or halt batch).

**Auto-retry:** disabled via RPC `set_auto_retry {enabled: false}` sent as the
first message before the prompt. Rationale: the orchestrator owns retry
decisions (per-phase, per-batch policy) — the harness should not independently
retry and hide transient failures from the orchestrator.

**Why RPC over print mode:** `-p` gives unstructured text output with no way to
separate the worker's final summary from streaming chatter. RPC's `agent_end`
event carries typed messages in machine-readable form.

### D2 — Filesystem-native context, not prompt-inlined

The orchestrator writes (`cp`) AGENTS.md into the fork after creation. pi
auto-discovers it from cwd at startup (pi walks up from cwd; the fork is not a
child of the project root, so AGENTS.md won't be discovered from parent
directories — explicit copy is required). The boot sector (`@.doctrine/state/boot.md`)
loaded by AGENTS.md dereferences governance and memory signposts. The worker
prompt is lean:

> Implement phase N of SL-NNN. Conventions are in AGENTS.md. Run `doctrine memory find <topic>` for subsystem gotchas.

The project's `.worktreeinclude` does not include AGENTS.md, so
`doctrine worktree fork --worker` does not provision it automatically. The
orchestrator's `cp` step is the delivery mechanism. Future: adding AGENTS.md to
`.worktreeinclude` would make this automatic, but that's a project-local config
choice, not a framework requirement.

**Why not inline conventions in the prompt:** burns orchestrator tokens every
batch. The orchestrator already provisions the fork — copying AGENTS.md is zero
marginal cost.

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
with `defaultProjectTrust: "ask"` — no trust hang. `--offline` suppresses
startup network checks.

The spawn template passes `--approve`: the orchestrator trusts its own worker, so
no trust prompt gates the run. Projects with stricter trust (`always-ask` or
per-operation approval) may remove the flag or replace it with
`--no-approve` — the flag is explicit so the posture is visible and overridable,
not hidden in project config.

The trust behavior is **project-config-dependent**. If a future project adds
project-local pi extensions and expects them in workers, the config posture may
need adjustment. For doctrine's own repo, the `--approve` flag covers the
default case.

## Code impact

One test cap bump; one skill documentation change; one e2e exercise.

| File | Change |
|---|---|
| `.agents/skills/dispatch-subprocess/SKILL.md` | Add pi RPC spawn arm alongside `codex exec` |
| `tests/e2e_skills_dispatch_shrinkage.rs` | Bump `dispatch-subprocess` body-line cap from ≤25 to ≤40 to accommodate the pi arm (projection: ~24 existing + ~6 spawn template + ~6 flag table + ~4 print mode) |
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

- **VT** — `e2e_skills_dispatch_shrinkage` body-line cap ≤40: the updated
  skill file measured line count ≤40 (projection: ~24 existing + ~6 spawn
  template + ~6 flag table + ~4 print mode).

No other VT tests — no binary CLI changes.

## Open questions (resolved in design)

- Session hygiene → D4: colocated with fork, `--session-dir` only (dropped `--no-session` — it suppresses session-dir)
- Tool profile → full built-in set (read, bash, edit, write, grep, find, ls)
- Trust gate → D5: probe confirms no hang; project-config-dependent, ok for v1
- Harness selection → D3: `[dispatch] preferred_subprocess_harness` in doctrine.toml (deferred to IMP-101)
- Context delivery → D2: filesystem-native AGENTS.md, lean prompt
- Scout/fix-agent generalisation → out of scope for this slice; tracked IMP-104

## Follow-ups

- **IMP-101** — `[dispatch] preferred_subprocess_harness` config + CLI `--harness` flag
- **IMP-104** — General pi subagent spawn pattern for scout/fix-agent roles
- **IDE-012** — Read-only doctrine memory retrieval tool for agent harnesses
