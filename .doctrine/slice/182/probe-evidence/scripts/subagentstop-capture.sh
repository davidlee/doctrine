#!/usr/bin/env bash
# SubagentStop CAPTURE PROBE — item 2 (the tallest, slice-gating: S1).
# Settles three load-bearing unknowns on the live harness:
#   (i)   SubagentStop is genuinely blocking/AWAITED (does the harness run this hook
#         to completion BEFORE `git worktree remove` tears the tree down?)
#   (ii)  TREE-INTACT — the worktree is still on disk when the hook runs.
#   (iii) CORRELATION — the payload (agent_id + agent_transcript_path, NO
#         worktree_path, RV-202) lets the hook resolve which worktree to diff.
# Captures `git -C <wt> diff` (+ untracked) to a patch OUTSIDE the worktree, the
# funnel-import source PHASE-05 builds on.
#
# Stop-loop hazard: a hook that always exit-2s blocks the subagent from EVER
# stopping. So the capture path ALWAYS exit 0; the exit-2 "blocking" assertion is a
# deliberate ONE-SHOT, gated by the BLOCK_ONCE sentinel (consumed on first fire).
set -u

ROOT=/workspace/doctrine
PD="$ROOT/.harness/probe"
LOG="$PD/subagentstop.log"
CAP="$PD/captured"
MAP="$PD/subagent-map"
mkdir -p "$CAP"
STDIN="$(cat)"
log() { printf '%s\t%s\n' "$(date -u +%s.%N)" "$*" >> "$LOG" 2>&1; }

# Full payload dump for field-presence audit (assert NO worktree_path).
printf '%s\n' "$STDIN" >> "$PD/subagentstop.payload.log" 2>&1

agent_id=$(printf '%s' "$STDIN" | jq -r '.agent_id // empty' 2>/dev/null)
atp=$(printf '%s' "$STDIN" | jq -r '.agent_transcript_path // .transcript_path // empty' 2>/dev/null)
wtp_field=$(printf '%s' "$STDIN" | jq -r '.worktree_path // empty' 2>/dev/null)
hook_pwd="$(pwd)"

log "STOP agent_id=${agent_id:-?} worktree_path_field=${wtp_field:-<absent>} hook_pwd=$hook_pwd atp=${atp:-?}"

# --- (iii) CORRELATION: try three candidates, record which resolve. -----------
cand_a="$ROOT/.worktrees/agent-${agent_id}"                       # reconstruct from agent_id
cand_c=""; [ -n "$agent_id" ] && [ -f "$MAP/$agent_id" ] && cand_c="$(cat "$MAP/$agent_id")"  # SubagentStart map
cand_b="$hook_pwd"                                                 # hook cwd (fallback)

wt=""; how=""
for pair in "a:$cand_a" "c:$cand_c" "b:$cand_b"; do
  tag="${pair%%:*}"; path="${pair#*:}"
  [ -n "$path" ] || continue
  if [ -d "$path" ] && git -C "$path" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    wt="$path"; how="$tag"; break
  fi
done

if [ -z "$wt" ]; then
  log "CORRELATE-FAIL a=$cand_a c=${cand_c:-<none>} b=$cand_b  => S1 candidate (abort to Path C)"
  exit 0   # never block on the capture path
fi
log "CORRELATE-OK via=$how wt=$wt"

# --- (ii) TREE-INTACT + CAPTURE before teardown. ------------------------------
if [ -d "$wt" ]; then
  patch="$CAP/${agent_id:-unknown}.patch"
  {
    git -C "$wt" diff
    git -C "$wt" diff --cached
  } > "$patch" 2>/dev/null
  # Untracked: list + tar contents (PHASE-05 funnel needs these too).
  git -C "$wt" ls-files --others --exclude-standard > "$CAP/${agent_id:-unknown}.untracked" 2>/dev/null
  if [ -s "$CAP/${agent_id:-unknown}.untracked" ]; then
    tar -C "$wt" -czf "$CAP/${agent_id:-unknown}.untracked.tgz" \
      -T "$CAP/${agent_id:-unknown}.untracked" 2>/dev/null
  fi
  pbytes=$(wc -c < "$patch" 2>/dev/null || echo 0)
  ucount=$(wc -l < "$CAP/${agent_id:-unknown}.untracked" 2>/dev/null || echo 0)
  log "CAPTURE-OK tree-intact wt=$wt patch_bytes=$pbytes untracked=$ucount"
else
  log "TREE-GONE wt=$wt  => S1 candidate (abort to Path C)"
fi

# --- (i) BLOCKING: one-shot exit-2 assertion, sentinel-consumed. --------------
if [ -f "$PD/BLOCK_ONCE" ]; then
  rm -f "$PD/BLOCK_ONCE"
  log "BLOCK-PROBE firing exit 2 (one-shot) — observe whether the stop was held/awaited"
  printf '%s\n' "worktree-jail probe: holding stop once (blocking assertion)" >&2
  exit 2
fi
exit 0
