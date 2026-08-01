#!/usr/bin/env bash
# pi-review.sh — confined, READ-ONLY pi reviewer. No worktree, no fork.
#
# Sibling of pi-spawn-confined.sh, stripped to a review posture. The dispatch
# worker spawn forks a worktree because a worker WRITES; a reviewer does not.
# Dropping the fork also drops the whole isolation-arm hazard class (ISS-034:
# `isolation: worktree` losing the git-lock race and silently falling back to a
# moving base) — there is no base to get wrong when nothing is written.
#
# Confinement: `--ro-bind / /` makes the entire filesystem read-only inside the
# namespace, including the tree under review. rw is re-granted to exactly two
# paths: `~/.pi` (pi writes session/auth state at runtime) and $OUT_DIR. The
# reviewer therefore CANNOT mutate the repo — that is an OS guarantee, not a
# prompt instruction.
#
# Model: no `--model` flag is passed, so pi resolves its configured default
# (`~/.pi/agent/settings.json` → deepseek/deepseek-v4-pro). This is deliberate
# and load-bearing: it makes the cheap tier the ONLY tier reachable through this
# script. Top-shelf review goes through the codex MCP, never through pi.
#
# Usage: pi-review.sh <LABEL> <PROMPT_FILE> <OUT_DIR> [BACKSTOP_SECS]
#
# Env:
#   REVIEW_ROOT   tree to review, ro-bound   (default: git toplevel of $PWD)
#   PI_THINKING   thinking level             (default: low)
#   PI_TOOLS      tool allowlist             (default: read,bash,grep,find,ls,write)
#
# Writes $OUT_DIR/<LABEL>.log (raw rpc stream) and expects the reviewer to write
# its own findings to $OUT_DIR/<LABEL>.md (the prompt must say so).
set -u

[ $# -ge 3 ] || {
  echo "usage: pi-review.sh <LABEL> <PROMPT_FILE> <OUT_DIR> [BACKSTOP_SECS]" >&2
  exit 2
}
LABEL="$1"
PF="$2"
OUT_DIR="$3"
BACKSTOP="${4:-1800}"

command -v bwrap >/dev/null || { echo "[review] bwrap not found" >&2; exit 1; }
command -v pi >/dev/null || { echo "[review] pi not found" >&2; exit 1; }
[ -f "$PF" ] || { echo "[review] prompt file not found: $PF" >&2; exit 1; }

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)
REVIEW_ROOT="${REVIEW_ROOT:-$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)}"
# bwrap --bind/--chdir require ABSOLUTE paths: under `--ro-bind / /` it cannot
# mkdir a relative mountpoint against the read-only new root.
case "$REVIEW_ROOT" in /*) ;; *) REVIEW_ROOT="$(cd "$REVIEW_ROOT" && pwd -P)" ;; esac
mkdir -p "$OUT_DIR" || { echo "[review] cannot create $OUT_DIR" >&2; exit 1; }
OUT_DIR="$(cd "$OUT_DIR" && pwd -P)"

[ -d "$REVIEW_ROOT" ] || { echo "[review] REVIEW_ROOT missing: $REVIEW_ROOT" >&2; exit 1; }

OUT="$OUT_DIR/$LABEL.log"
SESSION_DIR="$OUT_DIR/.pi-session-$LABEL"
mkdir -p "$SESSION_DIR"

# The prompt rides the rpc stream as a JSON string. The fifo stays open (the
# trailing sleep holds the write end) because pi self-exits on stdin EOF; we
# reap it ourselves on the typed completion event instead.
#
# `set -m` (job control) is enabled HERE, before the first background job, not
# just before the pi spawn: it puts each `&` job in its OWN process group so the
# reaps below can signal a whole group. It is load-bearing for BOTH jobs.
# For $KEEP: `kill -9 $KEEP` fells only the subshell and ORPHANS its `sleep
# $BACKSTOP` child, which inherits this script's stderr and so holds the pipe
# open for any caller that pipes or `wait`s on us — the script exits promptly and
# the CALLER hangs to the backstop anyway (ISS-293; the residual of IMP-024 §6's
# "orphans block a wait"). Observed: sleeps at ppid=1, 29 minutes after their
# raiser finished. `setsid` would also work but is ABSENT from this jail — do not
# "simplify" to it, and do not go back to a bare kill (CHR-051).
set -m
PI_FIFO=$(mktemp -u) && mkfifo "$PI_FIFO"
MSG=$(jq -Rs . <"$PF")
{
  printf '%s\n' '{"type":"set_auto_retry","enabled":false}'
  printf '{"type":"prompt","message":%s}\n' "$MSG"
  sleep "$BACKSTOP"
} >"$PI_FIFO" &
KEEP=$!

# $OUT and $PI_FIFO are host paths opened as fds by THIS shell before bwrap
# execs, so the inner `--tmpfs /tmp` cannot sever them. $OUT_DIR is bound AFTER
# the tmpfs so it survives as a real rw path even when it lives under /tmp —
# order matters, later mounts layer over earlier ones.
PREFIX=( bwrap
  --ro-bind / /
  --dev /dev --proc /proc --tmpfs /tmp
  --bind "$HOME/.pi" "$HOME/.pi"
  --bind "$OUT_DIR" "$OUT_DIR"
  --chdir "$REVIEW_ROOT"
  --die-with-parent )

echo "[review] $LABEL: ro=$REVIEW_ROOT rw=$OUT_DIR thinking=${PI_THINKING:-low}"

# --no-context-files matters for cost, not just hygiene: without it pi slurps
# the tree's AGENTS.md + CLAUDE.md, which `@`-import the ~28KB boot snapshot
# into every single reviewer. The bucket prompt carries what the reviewer needs.
# `set -m` (enabled above, at the first background job) puts this job in its OWN
# process group, pgid == $PI, so the reap below can signal the WHOLE group.
# Without it, `kill -9 $PI` fells only `timeout`; the real pi is a grandchild
# (timeout -> bwrap -> pi wrapper -> pi) and survives as an orphan holding its
# API session open and blocking any caller that `wait`s on this script. Observed:
# two such orphans still live 16 minutes after writing output and reporting
# agent_end.
timeout "$BACKSTOP" "${PREFIX[@]}" \
  pi --mode rpc --thinking "${PI_THINKING:-low}" \
  --session-dir "$SESSION_DIR" \
  --no-extensions --no-skills --no-themes --no-context-files \
  --offline --approve --tools "${PI_TOOLS:-read,bash,grep,find,ls,write}" \
  <"$PI_FIFO" >"$OUT" 2>&1 &
PI=$!

START=$(date +%s)
END=$((START + BACKSTOP))
REASON=timeout
# Poll the TAIL, not the whole file. pi's rpc stream re-serializes accumulated
# conversation state on every event, so $OUT grows super-linearly — 50-150MB for
# an ordinary turn is normal, not a runaway. `grep -q` over the whole file every
# 2s therefore costs more I/O than the model costs tokens, and it degrades as the
# turn goes on.
#
# The terminal event is `agent_settled`, NOT `agent_end` (ISS-293). `agent_end`
# carries the accumulated state with it, so it is pushed arbitrarily far back
# from EOF as the turn grows: measured 684,768 bytes from EOF on one census turn,
# 5.2x outside the old 128KiB window. The poll therefore NEVER fired on a real
# review, every raiser ran to BACKSTOP holding a live pi and an open API session,
# and nothing warned — the findings file lands on time, so it looks clean.
# `agent_settled` is a bare `{"type":"agent_settled"}` 17 bytes from EOF and is
# robust to any window; matching either keeps this working if the order changes
# again. 4MiB at a 2s cadence is still four orders of magnitude below the file.
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
# Negative pid = signal the whole process group (see the `set -m` note above).
# Fall back to the bare pid if the group is already gone. $KEEP is group-killed
# for the same reason $PI is: its `sleep $BACKSTOP` child is the orphan that
# outlives a bare kill and holds our stderr open.
kill -9 -"$PI" 2>/dev/null || kill -9 "$PI" 2>/dev/null
kill -9 -"$KEEP" 2>/dev/null || kill -9 "$KEEP" 2>/dev/null
# `kill -9` only QUEUES the signal. Without this `wait`, $PI and $KEEP — this
# shell's own children — linger as zombies that `ps` still lists, and the belt
# below reports a survivor that is already dead (ISS-294).
wait "$PI" "$KEEP" 2>/dev/null
rm -f "$PI_FIFO"

# Settle: the grandchildren (timeout -> bwrap -> wrapper -> pi) are NOT this
# shell's children and cannot be waited on, so give the group kill a bounded
# window to land before believing `ps`.
for _ in 1 2 3 4 5; do
  ps -eo pid,args 2>/dev/null | grep -q -- "[-]-session-dir $SESSION_DIR" || break
  sleep 0.2
done

# Belt: confirm nothing from this spawn outlived the reap. A surviving pi holds
# an API session open and silently blocks any caller that `wait`s on this script.
if ps -eo pid,args 2>/dev/null | grep -q -- "[-]-session-dir $SESSION_DIR"; then
  echo "[review] $LABEL WARNING: pi survived the reap for $SESSION_DIR" >&2
fi

# Elapsed-vs-backstop is the ONLY signal that distinguishes a clean early finish
# from a silent full-backstop burn — the findings file lands either way, so
# without this line the operator has to catch it in `ps` (ISS-293).
ELAPSED=$(($(date +%s) - START))
echo "[review] $LABEL terminated reason=$REASON after ${ELAPSED}s of ${BACKSTOP}s backstop"
if [ "$REASON" = timeout ]; then
  echo "[review] $LABEL WARNING: burned the full backstop — completion never detected" >&2
fi
if [ -s "$OUT_DIR/$LABEL.md" ]; then
  echo "[review] $LABEL findings: $OUT_DIR/$LABEL.md ($(wc -l <"$OUT_DIR/$LABEL.md") lines)"
else
  echo "[review] $LABEL WROTE NO FINDINGS FILE — inspect $OUT" >&2
fi
