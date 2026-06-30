# Probe H1 apparatus + evidence (RSK-014 candidate path #1)

Question → `../probe-brief-h1-pretooluse-bwrap.md`. Answer → `results.md`.

Verdict: **H1 SUPPORTED.** A `PreToolUse(Bash)` hook that opaquely rewrites the
command via `updatedInput` into a nested `bwrap` jail (rw only the subagent's
worktree) contains an `isolation: worktree` claude subagent's Bash writes,
including parent-tree and shared-`.git`. Edit/Write turned out **already natively
confined** by the harness. Probed on claude-code 2.1.181, NixOS bwrap 0.11.2.

## Apparatus
- `pretooluse-log.sh` — passive stdin/cwd logger (Exp 1, H1a identification).
- `pretooluse-wrap.sh` — the wrapper; opaque base64 → nested bwrap, fail-closed,
  orchestrator pass-through. Flags mirror `scripts/pi-spawn-confined.sh`.
- `pretooluse-pathcheck.sh` — Edit/Write second-wall stub (redundant vs native).
- `canaries/setup.sh`, `canaries/verify.sh` — plant/verify checksummed canaries.
- `redteam-prompt.md` — the 12-vector adversarial subagent prompt.

## Raw evidence (logs)
- `wrapper.log` — per-call decision (PASS-THROUGH orchestrator / WRAP subagent).
- `pretooluse.log` — full PreToolUse stdin dumps (shows `agent_id`, `cwd`).
- `pathcheck.log` — Edit/Write hook records (only the inside-worktree PASS fired;
  the outside Write was denied NATIVELY before the hook ran).

NOTE: scripts use absolute paths under the original live location
`/workspace/doctrine/.harness/probe/`. These copies are evidence/reference; to
re-run, restore them there and wire into `.claude/settings.local.json` (hooks
load at session start — restart required).
