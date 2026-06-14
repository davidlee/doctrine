---
name: dispatch-subprocess
description: The codex/pi arm of `/dispatch` — `doctrine worktree fork --worker` then spawn the worker as a subprocess with its cwd bound to the fork via `env -C`, or the confined bwrap `--chdir` profile. Carries the per-worktree env contract and the DOCTRINE_WORKER self-arm. Reached only from the `/dispatch` router on a codex/pi↔env-marker agreement; do not invoke directly.
---

# Dispatch — codex/pi arm (`fork` verb + subprocess spawn)

The harness-shaped **spawn half** for codex / pi. The harness-identical funnel
(capture `B` → import → verify → branch-point → one commit → record) and the drive
loop live in the [`/dispatch` router](../dispatch/SKILL.md) — **do not restate them
here.** This skill is only *how a codex/pi worker is created, identified, and
spawned*.

**Reached from the router, never directly.** `/dispatch` routes here only when the
agent's harness self-belief (codex/pi) **agrees** with env-marker detection. A
mismatch/unknown refuses there, naming the cause — never a blind spawn.

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

## Spawn — bind the worker cwd to the fork

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
  arm — codex/pi only).
- **NEVER ro-bind `.claude/settings.local.json`.** It sits inside the rw worktree
  and is **not** overlaid — it must stay writable for the harness permission-grant
  persistence (§9). Only the marker is pinned ro.
- **No flake.nix change.** `bubblewrap` is pre-staged in `jailPkgs`; the profile is
  this inline confined spawn template, **not** a packaged wrapper — the confined
  equivalent of the `env -C "$D"` spawn.

`$TARGET` is the per-wt build target the env contract declares (doctrine-the-repo:
`CARGO_TARGET_DIR`); `$MARKER` is the worker marker path inside `$D`.

## Worker identity — the disk marker + `DOCTRINE_WORKER=1`

`fork --worker` stamps the marker before the spawn window; the spawn line also sets
`DOCTRINE_WORKER=1` (self-arm). `worker_mode = (is_linked_worktree &&
marker_present) OR env DOCTRINE_WORKER`. On this arm both the marker (real env
channel) and bwrap are available, so the worker-mode floor is firmer than the
claude arm — the confined profile makes it an **OS floor**, not just a prompt
contract.

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
- Bind the worker cwd to the fork; carry `$fork_env` and `DOCTRINE_WORKER=1`.
- Prefer the confined bwrap profile in-jail (OS floor); fall back to `env -C` only
  where bwrap is unavailable.
- Return to the router for the funnel cadence — import, verify, branch-point, one
  commit, record.
