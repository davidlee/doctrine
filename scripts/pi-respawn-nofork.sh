#!/usr/bin/env bash
# One-off: respawn a confined pi worker against an EXISTING fork dir (no re-fork,
# preserves working tree + warm target/). Mirrors pi-spawn-confined.sh minus the
# fork step. Usage: pi-respawn-nofork.sh <DIR> <PROMPT_FILE> [BACKSTOP_SECS]
set -u
D="$1"
PF="$2"
BACKSTOP="${3:-1800}"
ROOT=/workspace/doctrine
case "$D" in /*) ;; *) D="$ROOT/$D" ;; esac
[ -d "$D" ] || { echo "fork dir missing: $D"; exit 1; }
echo "[respawn] $D (HEAD $(git -C "$D" rev-parse --short HEAD))"

OUT=$(mktemp)
PI_FIFO=$(mktemp -u) && mkfifo "$PI_FIFO"
MSG=$(jq -Rs . <"$PF")
{
  printf '%s\n' '{"type":"set_auto_retry","enabled":false}'
  printf '{"type":"prompt","message":%s}\n' "$MSG"
  sleep "$BACKSTOP"
} >"$PI_FIFO" &
KEEP=$!

timeout "$BACKSTOP" \
  bwrap \
    --ro-bind / / \
    --dev /dev --proc /proc --tmpfs /tmp \
    --bind "$HOME/.pi" "$HOME/.pi" \
    --bind "$D" "$D" \
    --chdir "$D" \
    --die-with-parent \
    --setenv DOCTRINE_WORKER 1 \
    pi --mode rpc --thinking off --session-dir "$D/.pi-session" \
    --no-extensions --no-skills --no-themes \
    --offline --approve --tools read,bash,edit,write,grep,find,ls \
    <"$PI_FIFO" >"$OUT" 2>&1 &
PI=$!

END=$(($(date +%s) + BACKSTOP))
REASON=timeout
while [ "$(date +%s)" -lt "$END" ]; do
  if grep -qE '"(type|event)":"agent_end"|"agent_end"' "$OUT" 2>/dev/null; then
    REASON=agent_end
    break
  fi
  if ! kill -0 "$PI" 2>/dev/null; then
    REASON=pi_exit
    break
  fi
  sleep 2
done
kill -9 "$PI" 2>/dev/null
kill -9 "$KEEP" 2>/dev/null
rm -f "$PI_FIFO"
echo "[respawn] terminated reason=$REASON"
echo "----- worker tail -----"
tail -25 "$OUT"
