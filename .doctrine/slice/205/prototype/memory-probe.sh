#!/usr/bin/env bash
# PROBE: PreToolUse(Read|Edit|Write) -> surface path-scoped doctrine memories.
# Emits additionalContext (factual block, data-not-instruction). Session dedup + cap.
# Remove from settings.local.json when done probing. Backlog: IDE-032.
set -euo pipefail

DOCTRINE="/home/david/.cargo/bin/doctrine"
LIMIT=3
SEEN="${CLAUDE_PROJECT_DIR:-/workspace/doctrine}/.doctrine/state/mem-probe-seen.txt"
LOG="${CLAUDE_PROJECT_DIR:-/workspace/doctrine}/.doctrine/state/mem-probe.log"

input="$(cat)"
path="$(printf '%s' "$input" | jq -r '.tool_input.file_path // empty')"
[ -z "$path" ] && exit 0

# repo-relative for scope probes
rel="${path#"${CLAUDE_PROJECT_DIR:-/workspace/doctrine}"/}"

rows="$("$DOCTRINE" memory retrieve --path-scope "$rel" --glob "$rel" --json --limit "$LIMIT" 2>/dev/null || true)"
[ -z "$rows" ] && exit 0

touch "$SEEN"
# select uids not yet surfaced this session
fresh="$(printf '%s' "$rows" | jq -c --slurpfile seen <(jq -R . "$SEEN" | jq -s .) \
  '.rows | map(select(.uid as $u | ($seen[0] | index($u)) | not))' 2>/dev/null || printf '%s' "$rows" | jq -c '.rows')"

count="$(printf '%s' "$fresh" | jq 'length')"
[ "$count" -eq 0 ] && exit 0

# record surfaced uids
printf '%s' "$fresh" | jq -r '.[].uid' >> "$SEEN"

block="$(printf '%s' "$fresh" | jq -r '
  "Doctrine memories for this file:\n" +
  ( map("- [\(.trust)\(if .staleness == "stale" then " ⚠stale" else "" end)] \(.title) — \(.uid)") | join("\n") )')"

printf '%s\n' "[$(basename "$path")] surfaced $count" >> "$LOG"

jq -n --arg ctx "$block" '{hookSpecificOutput:{hookEventName:"PreToolUse",additionalContext:$ctx}}'
