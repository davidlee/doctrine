#!/usr/bin/env bash
# SL-248 PHASE-09 `T2` — re-run `EVD-013`'s 2x2 against the SESSION-ESCAPING
# payload (`RV-346` `F-27`).
#
# `EVD-013` measured pid-namespace x `--die-with-parent` against a payload that
# merely *outlives* its parent. `F-27` strengthened row 7's payload to one that
# also leaves its session (`setsid`). Reasoning about the new payload from the
# old measurement is the altitude error `R1` forbids, so it is measured again.
#
# The four cells report; the sweep at the end of each is the only thing here
# that signals, and it is floored before it is aimed (`F-36`, below).
set -uo pipefail

# ---------------------------------------------------------------------------
# The floor. Written before the instrument that needs it (`F-36`).
# ---------------------------------------------------------------------------
#
# Mirrors `conformance.rs`'s `LOWEST_SIGNALLABLE = 2`: sessions 0 and 1 belong
# to the machine, never to a capsule. In THIS jail `bwrap` is pid 1 in session
# 0 and the agent is pid 2, also session 0 — a sweep that ever recorded session
# 0 kills the sandbox and the agent with no error and no shutdown path. That
# has happened twice in this slice.
#
# The pid floor here is one higher than the Rust one (3, not 2) because this
# script runs *beside* the agent rather than under the suite: pid 2 is the
# process reading its output. Stricter than the type, deliberately.
LOWEST_SIGNALLABLE_SID=2
LOWEST_SIGNALLABLE_PID=3
OWN_SID=$(ps -o sid= -p $$ | tr -d ' ')

sweep_session() { # sweep_session <sid> <label>
  local sid="$1" label="$2" pid
  if [ -z "$sid" ]; then
    echo "  sweep refused: no session recorded for $label"; return
  fi
  if ! [ "$sid" -ge "$LOWEST_SIGNALLABLE_SID" ] 2>/dev/null; then
    echo "  sweep refused: sid=$sid below the floor ($label)"; return
  fi
  if [ "$sid" = "$OWN_SID" ]; then
    echo "  sweep refused: sid=$sid is our own session ($label)"; return
  fi
  for pid in $(ps -o pid= -s "$sid" 2>/dev/null | tr -d ' '); do
    if ! [ "$pid" -ge "$LOWEST_SIGNALLABLE_PID" ] 2>/dev/null; then
      echo "  sweep refused: pid=$pid below the floor ($label)"; continue
    fi
    kill -9 "$pid" 2>/dev/null && echo "  swept pid=$pid sid=$sid ($label)"
  done
}

# ---------------------------------------------------------------------------
# The payload's `setsid`, which this jail does not ship as a binary
# ---------------------------------------------------------------------------
#
# `EVD-013`'s own method note records the trap: an earlier attempt used
# `setsid`, which is ABSENT from this jail's PATH, so the payload never started
# and every arm returned the same false negative. It is still absent. `python3`
# is present and `os.setsid()` is the same syscall, so the escapee is a python
# that detaches and then sleeps.
PYTHON=$(command -v python3) || { echo "no python3: cannot build a setsid payload"; exit 2; }
SH=/bin/sh

# Short, because a cell whose arm waits the escapee out costs this many seconds
# (`F-30`), and the wall duration is recorded so that case is legible.
ESCAPE_SECONDS=8

# Each cell tags its escapee with a marker unique to that cell, so a survivor
# is attributed to the arm that spawned it and never to a neighbour's leak.
# The marker rides in the python source, which *is* the escapee's command line,
# so `pgrep -f` finds it.
#
# The escapee sleeps in python rather than `exec`ing `sleep`, because coreutils
# ships `sleep` as a multi-call symlink that dispatches on argv[0]: an `execv`
# putting the marker there makes it exit immediately with "unknown program",
# and every cell then reports a false negative — the `EVD-013` failure mode
# wearing a different hat. Measured, not guessed.
payload() { # payload <marker>
  local marker="$1"
  printf '( %s -c "import os, time; os.setsid(); marker = '\''%s'\''; time.sleep(%s)" </dev/null >/dev/null 2>&1 & ); echo STARTED' \
    "$PYTHON" "$marker" "$ESCAPE_SECONDS"
}

# A survivor is a process carrying the marker that is **not in our own
# session**. The own-session filter is what keeps this script from finding
# itself: the marker is a literal in this file, so any shell that read the file
# does not carry it on its command line, but a future edit that passes one
# through `bash -c` would. Filtering is cheaper than remembering not to.
survivors() { # survivors <marker>  -> "pid sid" lines
  local pid sid
  for pid in $(pgrep -f "$1" 2>/dev/null); do
    sid=$(ps -o sid= -p "$pid" 2>/dev/null | tr -d ' ')
    [ -n "$sid" ] && [ "$sid" != "$OWN_SID" ] && echo "$pid $sid"
  done
}

report_and_sweep() { # report_and_sweep <marker> <label>
  local found left
  found=$(survivors "$1")
  if [ -n "$found" ]; then
    echo "  DESCENDANT SURVIVES: yes"
    echo "    pid sid: ${found//$'\n'/$'\n'    pid sid: }"
  else
    echo "  DESCENDANT SURVIVES: no"
  fi
  echo "$found" | while read -r _pid sid; do
    [ -n "${sid:-}" ] && sweep_session "$sid" "$2"
  done
  sleep 1
  left=$(survivors "$1")
  [ -n "$left" ] && echo "  !! STILL ALIVE AFTER SWEEP: $left"
  return 0
}

# ---------------------------------------------------------------------------
# The binds. Only what a shell and a python need, and only what exists.
# ---------------------------------------------------------------------------
binds=(); for d in /nix /usr /bin /lib /lib64 /etc; do
  [ -e "$d" ] && binds+=(--ro-bind "$d" "$d")
done
COMMON=("${binds[@]}" --proc /proc --dev /dev --tmpfs /tmp --new-session)

# Bubblewrap has no `--share-pid` (`EVD-013`, adjacent fact), so "pid namespace
# absent" is the enumerated set the production `Weakening::ProcessVisibility`
# uses, verbatim from `bubblewrap.rs`'s `NON_PID_UNSHARE_SET`.
PID_NS_PRESENT=(--unshare-all)
PID_NS_ABSENT=(--unshare-user-try --unshare-ipc --unshare-net --unshare-uts --unshare-cgroup-try)

cell() { # cell <label> <marker> <die-with-parent: yes|no> <ns: present|absent>
  local label="$1" marker="$2" die="$3" ns="$4"
  local flags=() started began ended
  if [ "$ns" = present ]; then flags+=("${PID_NS_PRESENT[@]}"); else flags+=("${PID_NS_ABSENT[@]}"); fi
  [ "$die" = yes ] && flags+=(--die-with-parent)

  echo "=== $label (pid-ns=$ns, --die-with-parent=$die) ==="
  began=$SECONDS
  started=$(bwrap "${flags[@]}" "${COMMON[@]}" "$SH" -c "$(payload "$marker")" 2>&1)
  ended=$SECONDS
  # The duration disambiguates a reap from a wait: an arm that returns in ~0s
  # with no survivor reaped it; an arm that takes ESCAPE_SECONDS waited it out
  # and proves nothing about teardown (`F-30`).
  echo "  arm returned in $((ended - began))s: ${started:-<no output>}"
  report_and_sweep "$marker" "$label"
  echo
}

echo "### host"
uname -srm
echo "bwrap: $(command -v bwrap) $(bwrap --version 2>&1)"
echo "python3: $PYTHON"
echo "own sid: $OWN_SID (floor sid>=$LOWEST_SIGNALLABLE_SID, pid>=$LOWEST_SIGNALLABLE_PID)"
echo "setsid binary on PATH: $(command -v setsid || echo '<absent — the EVD-013 trap>')"
echo "escape seconds: $ESCAPE_SECONDS"
echo

# P — the positive control. Without it every negative result is unreadable:
# it shows the escapee starts, escapes its session, and is seen by the counter.
echo "=== P — positive control, no bwrap at all ==="
eval "$(payload SL248-P)"
sleep 1
report_and_sweep SL248-P "P"
echo

cell "C1" SL248-C1 no  present
cell "C2" SL248-C2 no  absent
cell "C3" SL248-C3 yes present
cell "C4" SL248-C4 yes absent

# C1b — C1 again with the arm's stdout on a FILE instead of a pipe.
#
# `$(bwrap …)` waits for two things at once: the bwrap process to exit, and the
# capture pipe to reach end-of-file. The namespace's init inherits that pipe, so
# a cell that takes ESCAPE_SECONDS does not say which of the two it waited on.
# The harness conflates them the same way (`wait_with_output`), so the answer
# decides whether row 7's control is observable at all or only observable by
# giving up the capture.
echo "=== C1b (pid-ns=present, --die-with-parent=no, arm stdout on a FILE) ==="
armlog=$(mktemp) || exit 2
began=$SECONDS
bwrap "${PID_NS_PRESENT[@]}" "${COMMON[@]}" "$SH" -c "$(payload SL248-C1b)" >"$armlog" 2>&1
ended=$SECONDS
echo "  bwrap exited in $((ended - began))s: $(cat "$armlog")"
rm -f "$armlog"
report_and_sweep SL248-C1b "C1b"
