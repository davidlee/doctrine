#!/usr/bin/env bash
# Passive PreToolUse(Bash) observer — Exp 1 (H1a identification & cwd fidelity).
# Non-blocking, fail-open BY DESIGN (observation only; the real wrapper fails closed).
# Dumps full stdin JSON + the hook process's own cwd/realpath to an append-only
# log at a FIXED ABSOLUTE path outside every worktree.
set -u

LOG="/workspace/doctrine/.harness/probe/pretooluse.log"
STDIN="$(cat)"

{
  printf '===== PreToolUse(Bash) =====\n'
  printf 'epoch: %s\n' "$(date -u +%s.%N 2>/dev/null || echo NA)"
  printf 'hook_pwd: %s\n' "$(pwd)"
  printf 'hook_realpath: %s\n' "$(realpath . 2>/dev/null || echo NA)"
  printf 'env.CLAUDE_PROJECT_DIR: %s\n' "${CLAUDE_PROJECT_DIR:-<unset>}"
  printf 'stdin:\n'
  printf '%s\n' "$STDIN" | jq -S . 2>/dev/null || printf '%s\n' "$STDIN"
  printf '\n'
} >> "$LOG" 2>&1

exit 0
