#!/usr/bin/env bash
# PreToolUse(Edit|Write|NotebookEdit) PATH-CHECK — second-wall stub.
# Edit/Write are NOT Bash and bypass the bwrap wrapper entirely. This denies any
# subagent file write whose realpath escapes its worktree (cwd). Orchestrator
# (no agent_id) passes through. Completeness only — full H1 needs BOTH walls.
set -u

ROOT=/workspace/doctrine
LOG="$ROOT/.harness/probe/pathcheck.log"
STDIN="$(cat)"
log() { printf '%s\t%s\n' "$(date -u +%s.%N)" "$*" >> "$LOG" 2>&1; }

deny() {
  log "DENY agent_id=${agent_id:-?} file=${fp:-?} reason=$1"
  jq -cn --arg r "$1" '{hookSpecificOutput:{hookEventName:"PreToolUse",permissionDecision:"deny",permissionDecisionReason:("worktree-pathwall: "+$r)}}'
  exit 0
}

agent_id=$(printf '%s' "$STDIN" | jq -r '.agent_id // empty' 2>/dev/null)
cwd=$(printf '%s' "$STDIN" | jq -r '.cwd // empty' 2>/dev/null)
fp=$(printf '%s' "$STDIN" | jq -r '.tool_input.file_path // .tool_input.notebook_path // empty' 2>/dev/null)

[ -z "$agent_id" ] && exit 0   # orchestrator/interactive — unjailed

case "$cwd" in "$ROOT"/.worktrees/agent-*) : ;; *) deny "cwd-not-a-worktree:$cwd" ;; esac
[ -n "$fp" ] || deny "no-file-path"

# Resolve target against cwd if relative; canonicalize parent (file may not exist).
case "$fp" in /*) abs="$fp" ;; *) abs="$cwd/$fp" ;; esac
real=$(realpath -m "$abs" 2>/dev/null) || deny "unresolvable:$fp"
wt=$(realpath -m "$cwd" 2>/dev/null)

case "$real/" in
  "$wt"/*) log "PASS agent_id=$agent_id file=$real"; exit 0 ;;
  *) deny "escapes-worktree:$real" ;;
esac
