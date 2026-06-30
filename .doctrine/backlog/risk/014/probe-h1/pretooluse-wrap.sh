#!/usr/bin/env bash
# PreToolUse(Bash) WRAPPER — Exp 2 (H1b/c/d).
# Opaquely rewrites a worktree-subagent's Bash command via `updatedInput` to run
# inside a nested bwrap jail: rw ONLY the subagent's worktree (its cwd), ro-bind
# everything else. Reuses scripts/pi-spawn-confined.sh's proven flag set.
#
# FAIL-CLOSED: any ambiguity (unresolved worktree, missing bwrap, parse failure)
# → permissionDecision "deny". Never falls through to unwrapped execution.
#
# Orchestrator/interactive (no agent_id) → pass through unjailed (exit 0).
# Decision binding proven in Exp 1: payload.cwd == hook pwd == the worktree, and
# agent_id present iff subagent.
set -u

ROOT=/workspace/doctrine
LOG="$ROOT/.harness/probe/wrapper.log"
STDIN="$(cat)"

log() { printf '%s\t%s\n' "$(date -u +%s.%N)" "$*" >> "$LOG" 2>&1; }

deny() {
  log "DENY agent_id=${agent_id:-?} cwd=${cwd:-?} reason=$1 cmd=${cmd:-?}"
  jq -cn --arg r "$1" '{hookSpecificOutput:{hookEventName:"PreToolUse",permissionDecision:"deny",permissionDecisionReason:("worktree-jail: "+$r)}}'
  exit 0
}

agent_id=$(printf '%s' "$STDIN" | jq -r '.agent_id // empty' 2>/dev/null)
cwd=$(printf '%s' "$STDIN" | jq -r '.cwd // empty' 2>/dev/null)
cmd=$(printf '%s' "$STDIN" | jq -r '.tool_input.command // empty' 2>/dev/null)
desc=$(printf '%s' "$STDIN" | jq -r '.tool_input.description // empty' 2>/dev/null)

# Orchestrator / interactive: no agent_id → leave unjailed (full-repo access).
if [ -z "$agent_id" ]; then
  log "PASS-THROUGH orchestrator cwd=$cwd"
  exit 0
fi

# Subagent path — fail closed on every ambiguity.
[ -n "$cmd" ] || deny "empty-command"
case "$cwd" in
  "$ROOT"/.worktrees/agent-*) : ;;
  *) deny "cwd-not-a-worktree:$cwd" ;;
esac
[ -d "$cwd" ] || deny "worktree-dir-missing:$cwd"
command -v bwrap >/dev/null 2>&1 || deny "bwrap-unavailable"

# Opaque wrap: base64 the original (charset [A-Za-z0-9+/=] — safe inside the
# single-quoted `bash -c` arg). Decode + exec INSIDE the jail; the harness's
# outer shell never re-parses the original command. Flags mirror
# scripts/pi-spawn-confined.sh (known-good in live pi dispatch).
b64=$(printf '%s' "$cmd" | base64 -w0)
wrapped="bwrap --ro-bind / / --dev /dev --proc /proc --tmpfs /tmp --bind '$cwd' '$cwd' --chdir '$cwd' --die-with-parent -- bash -c 'printf %s $b64 | base64 -d | bash'"

log "WRAP agent_id=$agent_id cwd=$cwd cmd=$cmd"
jq -cn --arg c "$wrapped" --arg d "$desc" \
  '{hookSpecificOutput:{hookEventName:"PreToolUse",permissionDecision:"allow",permissionDecisionReason:"worktree-jail: wrapped",updatedInput:{command:$c,description:$d}}}'
exit 0
