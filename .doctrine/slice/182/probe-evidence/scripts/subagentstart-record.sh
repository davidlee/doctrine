#!/usr/bin/env bash
# SubagentStart RECORDER — item-2 correlator candidate (c).
# Fires when an isolation:worktree subagent starts. Payload (probed SL-056):
#   { session_id, transcript_path, cwd:<worktree>, agent_id, agent_type, hook_event_name }
# Records agent_id -> cwd so the SubagentStop hook (which gets agent_id but NO
# worktree_path, RV-202) can recover the worktree from a map it controls.
# Observe-only; exit 0 (non-blocking). No jail effect.
set -u

ROOT=/workspace/doctrine
PD="$ROOT/.harness/probe"
LOG="$PD/subagent.log"
MAP="$PD/subagent-map"
mkdir -p "$MAP"
STDIN="$(cat)"
log() { printf '%s\t%s\n' "$(date -u +%s.%N)" "$*" >> "$LOG" 2>&1; }

agent_id=$(printf '%s' "$STDIN" | jq -r '.agent_id // empty' 2>/dev/null)
cwd=$(printf '%s' "$STDIN" | jq -r '.cwd // empty' 2>/dev/null)
atype=$(printf '%s' "$STDIN" | jq -r '.agent_type // empty' 2>/dev/null)

log "START agent_id=${agent_id:-?} agent_type=${atype:-?} cwd=${cwd:-?}"
# Full payload dump (one line) for field-presence audit.
printf '%s\n' "$STDIN" >> "$PD/subagentstart.payload.log" 2>&1

if [ -n "$agent_id" ] && [ -n "$cwd" ]; then
  printf '%s\n' "$cwd" > "$MAP/$agent_id"
  log "MAP-WRITE $agent_id -> $cwd"
fi
exit 0
