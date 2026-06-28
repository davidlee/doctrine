#!/usr/bin/env bash
# Reusable dispatch pi worker spawn (subprocess arm).
# Breaks on the pi `agent_end` event instead of waiting out the timeout
# (fifo holds stdin open so pi never self-exits; we kill it on completion).
# Usage: pi-spawn.sh <B> <BRANCH> <DIR> <PROMPT_FILE> [BACKSTOP_SECS]
set -u
B="$1"
BR="$2"
D="$3"
PF="$4"
BACKSTOP="${5:-1800}"
ROOT=/workspace/doctrine
DOCTRINE=~/.cargo/bin/doctrine

# Fork is orchestrator-classed: run it from the orchestrator root, never from a
# worker-stamped worktree (else `worktree fork` resolves to worker-mode + refuses).
cd "$ROOT" || { echo "cd ROOT failed"; exit 1; }m -rf "$D"

"$DOCTRINE" worktree fork --base "$B" --branch "$BR" --dir "$D" --worker ||
  {
    echo "FORK FAILED $?"
    exit 1
  }
cp "$ROOT/AGENTS.md" "$D/" || {
  echo "AGENTS copy failed"
  exit 1
}
echo "[spawn] fork $BR @ $B -> $D (HEAD $(git -C "$D" rev-parse --short HEAD))"

OUT=$(mktemp)
PI_FIFO=$(mktemp -u) && mkfifo "$PI_FIFO"
MSG=$(jq -Rs . <"$PF")
{
  printf '%s\n' '{"type":"set_auto_retry","enabled":false}'
  printf '{"type":"prompt","message":%s}\n' "$MSG"
  sleep "$BACKSTOP"
} >"$PI_FIFO" &
KEEP=$!

timeout "$BACKSTOP" env -C "$D" DOCTRINE_WORKER=1 \
  pi --mode rpc --thinking off --session-dir "$D/.pi-session" \
  --no-extensions --no-skills --no-themes \
  --offline --approve --tools read,bash,edit,write,grep,find,ls \
  <"$PI_FIFO" >"$OUT" 2>&1 &
PI=$!

# Poll for the typed completion event; kill pi when the worker's turn ends.
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
echo "[spawn] terminated reason=$REASON"
echo "----- worker tail -----"
tail -40 "$OUT"
echo "----- worker commit -----"
git -C "$D" log --oneline -1 2>&1
git -C "$D" rev-parse HEAD 2>&1
