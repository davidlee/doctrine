#!/usr/bin/env bash
# PROBE: PreToolUse(Bash) -> surface command-scoped doctrine memories.
# Admission by MEMORY METADATA, not by a CLI allowlist (POL-002: no host-command
# knowledge baked into the trigger; zero maintenance). Fires on every Bash;
# admits only elevated-severity memories (the footguns worth interrupting for).
# Shares the session seen-set with memory-probe.sh. Remove when done. IDE-032.
set -euo pipefail

DOCTRINE="/home/david/.cargo/bin/doctrine"
FETCH=8                      # pull a wider slate, then filter by severity
CAP=2                        # max surfaced per fire
SEV='["critical","major","high"]'   # footgun floor (tunable)
SEEN="${CLAUDE_PROJECT_DIR:-/workspace/doctrine}/.doctrine/state/mem-probe-seen.txt"
LOG="${CLAUDE_PROJECT_DIR:-/workspace/doctrine}/.doctrine/state/mem-probe.log"

input="$(cat)"
cmd="$(printf '%s' "$input" | jq -r '.tool_input.command // empty')"
[ -z "$cmd" ] && exit 0

rows="$("$DOCTRINE" memory retrieve --command "$cmd" --json --limit "$FETCH" 2>/dev/null || true)"
[ -z "$rows" ] && exit 0

touch "$SEEN"
fresh="$(printf '%s' "$rows" | jq -c \
  --slurpfile seen <(jq -R . "$SEEN" | jq -s .) \
  --argjson sev "$SEV" \
  '.rows
     | map(select(.severity as $s | $sev | index($s)))          # metadata admission
     | map(select(.uid as $u | ($seen[0] | index($u)) | not))'  # session dedup \
  2>/dev/null || printf '[]')"

fresh="$(printf '%s' "$fresh" | jq -c ".[:$CAP]")"
count="$(printf '%s' "$fresh" | jq 'length')"
[ "$count" -eq 0 ] && exit 0

printf '%s' "$fresh" | jq -r '.[].uid' >> "$SEEN"

block="$(printf '%s' "$fresh" | jq -r '
  "Doctrine footguns (severity-gated):\n" +
  ( map("- [\(.severity)\(if .staleness == "stale" then " ⚠stale" else "" end)] \(.title) — \(.uid)") | join("\n") )')"

printf '%s\n' "[cmd] surfaced $count for: $cmd" >> "$LOG"

jq -n --arg ctx "$block" '{hookSpecificOutput:{hookEventName:"PreToolUse",additionalContext:$ctx}}'
