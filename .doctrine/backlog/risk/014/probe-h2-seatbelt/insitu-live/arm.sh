#!/usr/bin/env bash
# Arm the LIVE-CONSUMER in-situ rig (SL-183 PHASE-04 / EX-2).
#
# Repoints ONLY the skill hooks.json Bash matcher at the logging shim, which runs
# the REAL `doctrine worktree pretooluse` and tees each decision (WRAP/DENY/EMPTY +
# permission_mode) to $PROBE_BASE/consumer.log. Edit|Write / SessionStart /
# WorktreeCreate matchers are left byte-identical. disarm.sh restores from the
# backup this writes. Registration loads at SESSION START — restart after arming.
set -eu
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$HERE/../../../../../.." && pwd)"   # insitu-live/probe-h2-seatbelt/014/risk/backlog/.doctrine → repo root
HOOKS="$REPO/.claude/skills/doctrine/hooks/hooks.json"
SHIM="$HERE/pretooluse-log-shim.sh"
PROBE_BASE="$REPO/.harness/probe/h2-live"

mkdir -p "$PROBE_BASE"
cp "$HOOKS" "$HERE/hooks.json.bak"
echo "backed up hooks.json → $HERE/hooks.json.bak"

# Repoint the Bash matcher's command at the shim (PROBE_BASE inline, absolute paths).
python3 - "$HOOKS" "$SHIM" "$PROBE_BASE" <<'PY'
import json, sys
hooks_path, shim, probe_base = sys.argv[1], sys.argv[2], sys.argv[3]
with open(hooks_path) as f:
    cfg = json.load(f)
cmd = f'PROBE_BASE="{probe_base}" bash "{shim}" || exit 2'
for group in cfg["hooks"].get("PreToolUse", []):
    if group.get("matcher") == "Bash":
        for h in group["hooks"]:
            h["command"] = cmd
with open(hooks_path, "w") as f:
    json.dump(cfg, f, indent=2, sort_keys=True)
    f.write("\n")
print("Bash matcher repointed at:", cmd)
PY

echo
echo "ARMED. Next: RESTART Claude Code (hooks register at session start), then run.sh."
echo "PROBE_BASE=$PROBE_BASE"
