#!/usr/bin/env bash
# Restore the skill hooks.json from arm.sh's backup (disarm the live-consumer rig).
# Restart Claude Code afterwards to re-register the shipped hook.
set -eu
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$HERE/../../../../../.." && pwd)"
HOOKS="$REPO/.claude/skills/doctrine/hooks/hooks.json"
if [ -f "$HERE/hooks.json.bak" ]; then
  cp "$HERE/hooks.json.bak" "$HOOKS"
  rm -f "$HERE/hooks.json.bak"
  echo "restored hooks.json from backup. RESTART Claude Code to re-register the shipped hook."
else
  echo "no backup found ($HERE/hooks.json.bak) — nothing to restore." >&2
  exit 1
fi
