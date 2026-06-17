---
name: dispatch-subprocess
description: >-
  The pi / codex arms of `/dispatch` — `doctrine worktree fork --worker` then spawn the worker as a subprocess with its cwd bound to the fork via the per-harness spawn mechanism (pi subprocess via `subagent` tool, codex via `codex exec`). Carries the per-worktree env contract and the DOCTRINE_WORKER self-arm. Reached only from the `/dispatch` router on a pi/codex↔env-marker agreement; do not invoke directly.
---

# Dispatch — pi / codex arms (`fork` verb + subprocess spawn)

The harness-shaped **spawn half** for pi and codex. The harness-identical funnel
(capture `B` → import → verify → branch-point → one commit → record) and the drive
loop live in the [`/dispatch` router](../dispatch/SKILL.md) — **do not restate them
here.** This skill is only *how a pi or codex worker is created, identified, and
spawned* — the correct spawn mechanism depends on the detected harness (see the
[harness→spawn table](#harnessspawn-table) below).

**Reached from the router, never directly.** `/dispatch` routes here only when the
agent's harness self-belief (pi or codex) **agrees** with env-marker detection. A
mismatch/unknown refuses there, naming the cause — never a blind spawn.

## Harness→spawn table

The fork verb and worker identity infrastructure are shared; only the spawn
mechanism differs per harness. Select the row matching the harness the router
detected:

| Harness | Spawn mechanism | Identity | Notes |
|---|---|---|---|
| **pi** | `subagent(agent="dispatch-worker", task="<pre-distilled prompt>", cwd="$D")` | Disk marker (primary) + prompt self-arm (prefix `DOCTRINE_WORKER=1` per command) | ✅ Tested end-to-end. Requires `pi-subagents` extension; detected via `PI_HOME` env + self-belief. **Residual:** fork branch IS the phase ref — gc only after `dispatch sync --prepare-review`. |
| **codex** | `env -C "$D" DOCTRINE_WORKER=1 $fork_env codex exec "<pre-distilled prompt>"` | Disk marker (primary) + env `DOCTRINE_WORKER=1` (orchestrator-set) | ⚠️ Legacy placeholder — **untested end-to-end.** Detected by explicit self-belief ("I am codex"), no env marker known. Env-marker validation deferred to codex spike. **Residual:** same gc-after-sync ordering constraint as pi. |

### D3 — pi-subagents extension detection

Before spawning a pi worker, the orchestrator MUST verify the `subagent` tool
is available in its tool list (a self-check, not a `bash` command — the LLM's
available function/tool surface). If `subagent` is absent:

> "pi dispatch requires the `pi-subagents` extension. Install: `pi install pi-subagents`"

No silent fallback — the agent-def contract (source-only discipline, structured
report, no `.doctrine/` writes) is load-bearing and requires the extension.

### D3 — codex detection (provisional)

Codex detection uses explicit self-belief only ("I am codex") with no known env
marker. The dispatch router conveys self-belief via the pre-distilled prompt
when the harness check matches the codex branch. Env-marker characterization is
deferred to a separate codex spike — until then, unknown/non-pi/non-claude does
NOT imply codex; it refuses.

## Create — `doctrine worktree fork --worker`

One verb does create + provision + mark + emit (see the
[worktree skill](../worktree/SKILL.md) — do not restate its internals):

```sh
fork_env="$(doctrine worktree fork --base "$B" --branch "$BR" --dir "$D" --worker)" \
  || { echo "fork failed: $?" >&2; exit 1; }      # halt, do NOT spawn
```

- **Capture + check `$?`; never `eval "$(…)"`** — `eval` swallows the exit status,
  a fail-open trap. `$fork_env` is the stdout per-worktree env block (`KEY=value`
  per line); human status went to stderr. A non-zero `fork` means **no worktree** —
  halt, do not spawn.

## Spawn (codex) — bind the worker cwd to the fork

**The pi spawn variant is the [harness→spawn table](#harnessspawn-table) above — use
`subagent()` instead of `codex exec`.** The codex variants below are the legacy
`codex exec` spawn mechanism, preserved for codex workers only. Both rows share the
same fork/marker/cwd infrastructure.

The worker process **must** run with its cwd bound to the fork `$D`. Two variants:

**Unconfined (`env -C`):**

```sh
env -C "$D" DOCTRINE_WORKER=1 $fork_env codex exec "<pre-distilled prompt>"
#       ^ cwd→fork    ^ self-arm        ^ per-wt env    ^ harness-shaped line
```

**Confined (bwrap, PHASE-12 LAND — the D6 OS-floor profile):**

```sh
bwrap --ro-bind / / --bind "$D" "$D" --bind "$TARGET" "$TARGET" \
      --ro-bind "$MARKER" "$MARKER" --dev /dev --proc /proc \
      --unshare-user --chdir "$D" \
      env DOCTRINE_WORKER=1 $fork_env codex exec "<pre-distilled prompt>"
```

- **`env -C "$D"` / `--chdir "$D"` binds the worker cwd to the fork.** Without it
  `codex exec` inherits the orchestrator's cwd (the coordination root) and the
  worker's *source* edits land on the trusted branch — bypassing `import`, the
  belt, and branch-point discipline (`DOCTRINE_WORKER` catches only
  doctrine-mediated writes, never a raw editor write to cwd). This is a
  **spawn-shell mechanism**, not a prompt instruction. Portable fallback if `env
  -C` is absent: `( cd "$D" && exec env DOCTRINE_WORKER=1 $fork_env codex exec … )`.
- **The `--ro-bind "$MARKER"` overlays the rw worktree bind — order matters.** It
  comes **after** `--bind "$D"`: the worktree stays rw, but the marker file is
  ro, so the worker **cannot `rm`/overwrite its own marker** (DENIED) ⇒ cannot
  escape worker-mode at the OS floor (closes the §4c self-clear residual on this
  arm — codex only).
- **NEVER ro-bind `.claude/settings.local.json`.** It sits inside the rw worktree
  and is **not** overlaid — it must stay writable for the harness permission-grant
  persistence (§9). Only the marker is pinned ro.
- **No flake.nix change.** `bubblewrap` is pre-staged in `jailPkgs`; the profile is
  this inline confined spawn template, **not** a packaged wrapper — the confined
  equivalent of the `env -C "$D"` spawn.

`$TARGET` is the per-wt build target the env contract declares (doctrine-the-repo:
`CARGO_TARGET_DIR`); `$MARKER` is the worker marker path inside `$D`.

## Worker identity — the disk marker + `DOCTRINE_WORKER=1`

`fork --worker` stamps the marker before the spawn window. For the **codex** row the
spawn line also sets `DOCTRINE_WORKER=1` (env self-arm); for the **pi** row the
worker self-arms via prompt prefix per command (the `subagent` tool has no env
parameter). `worker_mode = (is_linked_worktree &&
marker_present) OR env DOCTRINE_WORKER`. On this arm both the marker (real env
channel) and bwrap are available, so the worker-mode floor is firmer than the
claude arm — the confined profile makes it an **OS floor**, not just a prompt
contract.

## Against `dispatch/<slice>` — the fork branch is the native phase unit (EX-4)

The orchestrator drives from the `dispatch/<slice>` coordination worktree
(SL-064 / ADR-012). On this arm the worker forks from the explicit base `B` — **a
ref on `dispatch/<slice>`**, never session HEAD (`fork --base "$B"` pins it). The
worker's single-commit fork branch **is** the native `phase/<slice>-NN` code unit
(ADR-012 D3): stage-2 `dispatch sync --integrate` consumes native and synthesized
phase branches uniformly.

**No funnel-time boundary recording on this arm.** `boundaries.toml` (and the
`dispatch record-boundary` verb) feed the **fork-less claude arm's** per-phase cut
(design §4.3) — where there is no fork branch to stand in. Here the fork branch
already is the deliverable, so the orchestrator **skips** record-boundary. Don't add
it; it is consumed only where the cut needs it.

## Red Flags

**Never:**
- `eval "$(doctrine worktree fork …)"` — it swallows `$?` (fail-open). Capture, then
  check `$?`, then spawn.
- Spawn without `env -C "$D"` / `--chdir "$D"` — the worker would edit the trusted
  cwd, bypassing the funnel.
- ro-bind `.claude/settings.local.json` (breaks permission persistence); only the
  marker is pinned ro.
- Add a flake.nix entry or a packaged bwrap wrapper — the profile is the inline
  template; `bubblewrap` is already staged.
- Author or edit this skill in `.doctrine/skills/` (the gitignored install copy);
  the source is here under `plugins/`.

**Always:**
- Halt on a non-zero `fork` — no worktree, no spawn.
- Bind the worker cwd to the fork. For pi: use `subagent(cwd="$D")`. For codex: use `env -C "$D"` or `--chdir "$D"`. Carry `$fork_env` and `DOCTRINE_WORKER=1` (codex env; pi prompt prefix).
- Prefer the confined bwrap profile in-jail (OS floor); fall back to `env -C` only
  where bwrap is unavailable.
- Return to the router for the funnel cadence — import, verify, branch-point, one
  commit, record.
