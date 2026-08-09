#!/usr/bin/env bash
# SL-248 PHASE-09 `T3` — `VA-3`: measure every delta rows 1–8 and table B rely
# on that was adopted by reasoning rather than by measurement.
#
# The hit list after `EN-3`'s correction: `WorkingDirectory`, `FileSizeBound`,
# `WallBound`, and a **row-shaped** confirmation for `ProcessVisibility` —
# `F-7` measured that one's mechanism (how many `/proc` entries appear), not the
# row's control failing. `Teardown` is `T2`'s and is not repeated here.
#
# For each axis: run the confining form and the weakened form, and show the
# observable difference. A delta that does not produce its row's control failure
# means the ROW is wrong (`S4`), not the measurement inconvenient —
# `CredentialsConfined` is the worked example: reasoned, adopted, and wrong.
#
# Nothing here signals anything it did not start, and nothing is deleted outside
# a `mktemp -d` this script made. The only kill is `timeout`'s, on its own child.
set -uo pipefail

SH=/bin/sh
BWRAP=$(command -v bwrap) || { echo "no bwrap"; exit 2; }

# Same binds as the production profile's shape: read-only system, a private
# /proc, /dev and /tmp. Placement binds are the parts that vary per axis.
binds=(); for d in /nix /usr /bin /lib /lib64 /etc; do
  [ -e "$d" ] && binds+=(--ro-bind "$d" "$d")
done
COMMON=("${binds[@]}" --proc /proc --dev /dev --tmpfs /tmp --new-session --die-with-parent)

CONFINING_NS=(--unshare-all)
# Bubblewrap has no `--share-pid`; `Weakening::ProcessVisibility` names the
# namespaces that remain. Verbatim from `bubblewrap.rs`'s `NON_PID_UNSHARE_SET`.
WEAKENED_NS=(--unshare-user-try --unshare-ipc --unshare-net --unshare-uts --unshare-cgroup-try)

SCRATCH=$(mktemp -d) || exit 2
trap 'rm -rf "$SCRATCH"' EXIT
mkdir -p "$SCRATCH/capsule"

run_arm() { # run_arm <label> <payload> <bwrap flags...>
  local label="$1" payload="$2"; shift 2
  local out status
  out=$("$BWRAP" "$@" "${COMMON[@]}" "$SH" -c "$payload" 2>&1)
  status=$?
  printf '  %-34s exit=%-4s %s\n' "$label" "$status" "${out:-<no output>}"
}

echo "### host"
uname -srm
echo "bwrap: $BWRAP $("$BWRAP" --version 2>&1)"
echo "own pid: $$"
echo

# ---------------------------------------------------------------------------
# 1. WorkingDirectory — production omits `--chdir <placement working dir>`
# ---------------------------------------------------------------------------
echo "=== WorkingDirectory (confining: --chdir /capsule; weakened: omitted) ==="
run_arm "confining"  'pwd' "${CONFINING_NS[@]}" --bind "$SCRATCH/capsule" /capsule --chdir /capsule
run_arm "weakened (no --chdir)" 'pwd' "${CONFINING_NS[@]}" --bind "$SCRATCH/capsule" /capsule
echo "  (run from cwd=$PWD, which does not exist inside the sandbox)"
echo

# ---------------------------------------------------------------------------
# 2. FileSizeBound — production sets RLIMIT_FSIZE on the child in `pre_exec`,
#    OUTSIDE the namespace. `ulimit -f` is the same rlimit, set on the same
#    side of the same exec.
# ---------------------------------------------------------------------------
echo "=== FileSizeBound (confining: RLIMIT_FSIZE 1MiB; weakened: unset) ==="
# Two readings, because the row and the mechanism want different ones. Bash's
# `ulimit -f` is in 1024-byte units, so 1024 is 1 MiB.
#
# (a) how much of a 2 MiB write survives — the truncation the cap causes;
# shellcheck disable=SC2016  # the payload's `$(…)` is the capsule's to expand.
WRITE_2MIB='dd if=/dev/zero of=/tmp/big bs=1024 count=2048 2>/dev/null; echo "wrote $(wc -c </tmp/big 2>/dev/null || echo 0) bytes"'
( ulimit -f 1024 && run_arm "confining" "$WRITE_2MIB" "${CONFINING_NS[@]}" )
run_arm "weakened (no cap)" "$WRITE_2MIB" "${CONFINING_NS[@]}"
# (b) what the capsule's TOP-LEVEL process exits with when the write is the
#     payload itself — the reading row 8 turns into `Termination::FileSizeExceeded`
#     via `classify_termination`'s 128+SIGXFSZ.
EXEC_WRITE='exec dd if=/dev/zero of=/tmp/big bs=1024 count=2048 2>/dev/null'
( ulimit -f 1024 && run_arm "confining, write is the payload" "$EXEC_WRITE" "${CONFINING_NS[@]}" )
run_arm "weakened, write is the payload" "$EXEC_WRITE" "${CONFINING_NS[@]}"
echo "  (a write past RLIMIT_FSIZE raises SIGXFSZ; 128+25=153)"
echo

# ---------------------------------------------------------------------------
# 3. WallBound — production wraps the exec in `timeout -k <grace> <secs>` from
#    OUTSIDE, where the capsule cannot reach it (`EX-15`).
# ---------------------------------------------------------------------------
echo "=== WallBound (confining: timeout -k 5 2; weakened: no wrapper) ==="
SLEEP_6='sleep 6; echo LIVE'
began=$SECONDS
out=$(timeout -k 5 2 "$BWRAP" "${CONFINING_NS[@]}" "${COMMON[@]}" "$SH" -c "$SLEEP_6" 2>&1); status=$?
printf '  %-34s exit=%-4s %ss %s\n' "confining" "$status" "$((SECONDS - began))" "${out:-<no output>}"
began=$SECONDS
out=$("$BWRAP" "${CONFINING_NS[@]}" "${COMMON[@]}" "$SH" -c "$SLEEP_6" 2>&1); status=$?
printf '  %-34s exit=%-4s %ss %s\n' "weakened (no timeout)" "$status" "$((SECONDS - began))" "${out:-<no output>}"
echo "  (GNU timeout reports 124 when it fires)"
echo

# ---------------------------------------------------------------------------
# 4. ProcessVisibility — row-shaped, not mechanism-shaped.
#    The row's claim is that the capsule cannot see host processes. So the
#    payload asks for a NAMED host process — this script — by pid, which is the
#    question the row asks. The entry count is kept alongside as the `F-7`
#    mechanism reading, so the two are not confused again.
# ---------------------------------------------------------------------------
echo "=== ProcessVisibility (confining: --unshare-all; weakened: enumerated set) ==="
PROBE="printf 'own-pid=%s host-pid-$$-visible=%s numeric-proc-entries=%s\\n' \
  \"\$(cat /proc/self/stat | cut -d' ' -f1)\" \
  \"\$( [ -e /proc/$$ ] && echo yes || echo no )\" \
  \"\$(ls /proc | grep -c '^[0-9]')\""
run_arm "confining"  "$PROBE" "${CONFINING_NS[@]}"
run_arm "weakened (pid ns shared)" "$PROBE" "${WEAKENED_NS[@]}"
echo "  (the row fails when the capsule can see host pid $$)"
