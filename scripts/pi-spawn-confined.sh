#!/usr/bin/env bash
# Reusable dispatch pi worker spawn (subprocess arm) — CONFINED variant.
#
# IMP-004 D-B3 spike: nested bwrap confinement. Discharges ADR-006 D2b at the OS
# level (governing ADR-008). The worker's `pi` exec is wrapped in a nested bwrap
# that rw-binds ONLY its own worktree ($D) and ro-binds everything else (main
# tree, .doctrine/* authored+runtime state, sibling worktrees). The fork itself
# is orchestrator-classed, runs in $ROOT before confinement, and is untouched.
#
# Identical to pi-spawn.sh except for the bwrap wrapper around the pi exec.
# Kept as a separate file while pi-spawn.sh is in live dispatch use.
#
# Breaks on the pi `agent_end` event instead of waiting out the timeout
# (fifo holds stdin open so pi never self-exits; we kill it on completion).
# Usage: pi-spawn-confined.sh <B> <BRANCH> <DIR> <PROMPT_FILE> [BACKSTOP_SECS]
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
cd "$ROOT" || { echo "cd ROOT failed"; exit 1; }
# bwrap --bind/--chdir require an ABSOLUTE path: under `--ro-bind / /` it cannot
# mkdir a relative mountpoint against the read-only new root. Absolutize $D
# (relative dirs are resolved against $ROOT) so callers keep the pi-spawn.sh
# relative-dir convention.
case "$D" in /*) ;; *) D="$ROOT/$D" ;; esac
rm -rf "$D"

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

# Confinement boundary. --ro-bind / / makes the whole fs read-only inside the
# namespace, then --bind "$D" "$D" re-grants rw to just the worker's worktree
# (its in-tree target/ rides along, so cargo builds stay writable). OUT and
# PI_FIFO are host-/tmp paths opened as fds by THIS shell before bwrap execs, so
# the inner --tmpfs /tmp does not sever them. --die-with-parent lets the kill -9
# below reap pi through bwrap.
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
