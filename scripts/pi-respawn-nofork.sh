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
# `set -m` before the FIRST background job so each `&` job gets its OWN process
# group and the reaps below can signal a whole group — load-bearing for BOTH
# jobs, $KEEP included (its `sleep` orphans otherwise and holds our stderr open,
# ISS-293). `setsid` is ABSENT from this jail; do not "simplify" to it. CHR-051 §2.
set -m
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

# Poll the TAIL, not the whole file: pi's rpc stream re-serializes accumulated
# state on every event, so $OUT reaches 50-150MB on an ordinary turn and a whole-
# file `grep -q` every 2s costs more I/O than the model costs tokens.
#
# The terminal event is `agent_settled`, NOT `agent_end` (ISS-293). `agent_end`
# carries the accumulated state, so it sits arbitrarily far from EOF — measured
# 684,768 bytes — and a small window never fires, burning the full backstop on a
# live pi while looking clean. `agent_settled` is 17 bytes from EOF; match either.
START=$(date +%s)
END=$((START + BACKSTOP))
REASON=timeout
while [ "$(date +%s)" -lt "$END" ]; do
  if tail -c 4194304 "$OUT" 2>/dev/null | grep -qE '"(agent_settled|agent_end)"'; then
    REASON=agent_complete
    break
  fi
  if ! kill -0 "$PI" 2>/dev/null; then
    REASON=pi_exit
    break
  fi
  sleep 2
done
# Negative pid = signal the whole group. The chain is timeout -> bwrap -> wrapper
# -> pi, so $! is the TIMEOUT; a bare kill fells only that and orphans the real
# pi holding its API session — and a background `wait` on this script then blocks
# until the orphan dies. Observed: two orphans alive 16 minutes after writing out.
kill -9 -"$PI" 2>/dev/null || kill -9 "$PI" 2>/dev/null
kill -9 -"$KEEP" 2>/dev/null || kill -9 "$KEEP" 2>/dev/null
# `kill -9` only QUEUES; without the wait these stay zombies and the belt below
# reports an already-dead survivor (ISS-294).
wait "$PI" "$KEEP" 2>/dev/null
rm -f "$PI_FIFO"

# Grandchildren are not our children and cannot be waited on — give the group
# kill a bounded window to land before believing `ps`.
for _ in 1 2 3 4 5; do
  # pgrep landed in flake.nix (deb2cf44) but is not in an already-running
  # jail; switch this to `pgrep -f` only once it can be exercised, since a
  # silently-broken belt check reads exactly like a clean reap.
  # shellcheck disable=SC2009
  ps -eo pid,args 2>/dev/null | grep -q -- "[-]-session-dir $D/.pi-session" || break
  sleep 0.2
done
# pgrep landed in flake.nix (deb2cf44) but is not in an already-running
# jail; switch this to `pgrep -f` only once it can be exercised, since a
# silently-broken belt check reads exactly like a clean reap.
# shellcheck disable=SC2009
if ps -eo pid,args 2>/dev/null | grep -q -- "[-]-session-dir $D/.pi-session"; then
  echo "[respawn] WARNING: pi survived the reap for $D/.pi-session" >&2
fi

# Elapsed-vs-backstop is the only signal separating a clean early finish from a
# silent full-backstop burn — the output lands either way (ISS-293).
ELAPSED=$(($(date +%s) - START))
echo "[respawn] terminated reason=$REASON after ${ELAPSED}s of ${BACKSTOP}s backstop"
if [ "$REASON" = timeout ]; then
  echo "[respawn] WARNING: burned the full backstop — completion never detected" >&2
fi
echo "----- worker tail -----"
tail -25 "$OUT"
