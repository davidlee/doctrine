---
name: dispatch-subprocess
description: The codex/pi arm of `/dispatch` — `doctrine worktree fork --worker` then spawn the worker as a subprocess with its cwd bound to the fork. Reached only from the `/dispatch` router on a codex/pi↔env-marker agreement; do not invoke directly.
---

# Dispatch — codex/pi arm

Spawn a worker via `doctrine worktree fork --worker` + subprocess spawn. The
harness-identical funnel and drive loop live in the [`/dispatch`
router](../dispatch/SKILL.md) — this skill is only the spawn template.

## Spawn

```sh
fork_env="$(doctrine worktree fork --base "$B" --branch "$BR" --dir "$D" --worker)" \
  || { echo "fork failed: $?" >&2; exit 1; }
env -C "$D" DOCTRINE_WORKER=1 $fork_env codex exec "<pre-distilled prompt>"
```

Confined (bwrap) in-jail: bind `$D` rw, marker ro, `--chdir "$D"`.
**Never `eval`** — capture `$fork_env`, check `$?`, then spawn.

## Red Flags
**Never:** `eval "$(doctrine worktree fork …)"`; spawn without `env -C "$D"` /
`--chdir "$D"`; ro-bind `.claude/settings.local.json`; run `record-boundary` here
(the fork branch IS the native phase unit — skip it).
**Always:** halt on non-zero `fork`; bind worker cwd to the fork; carry `$fork_env`
and `DOCTRINE_WORKER=1`; return to the router for the funnel cadence.
