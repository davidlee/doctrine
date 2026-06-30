#!/usr/bin/env bash
# Escape battery — drives each vector INSIDE the Seatbelt floor and records
# WROTE/BLOCKED. macOS analog of probe-h1/redteam-prompt.md (11 vectors + the two
# macOS-only delegation vectors launchctl/at). Pass 1 drives vectors directly
# (no subagent); the M1-sub in-situ leg is pass 2.
#
# Each vector tries to mutate an OUTSIDE canary; the independent verifier
# (canaries/verify.sh) is the source of truth, not this script's self-report.
set -u
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$HERE/seatbelt-jail.sh"

BASE="${PROBE_BASE:?set PROBE_BASE}"
WT="$BASE/wt"
OUT="$BASE/outside"
PROF="$BASE/floor.sb"
PARENT="$(realpath "$OUT/parent_canary")"
GIT="$(realpath "$OUT/fakegit/PROBE_CANARY")"
PTMP=/tmp/h2_ptmp_canary
HOMEC="$HOME/.h2_home_canary"

# run a single vector command inside the floor; report its result line
vec() {
  local n="$1" desc="$2"; shift 2
  local out
  out="$(seatbelt_run "$WT" true 1 "$PROF" -- "$*" 2>&1)"
  printf 'V%-2s %-26s :: %s\n' "$n" "$desc" "$(printf '%s' "$out" | tr '\n' '|')"
}

echo "===== Seatbelt escape battery (floor: deny file-write*, allow wt+TMP, deny PTMP) ====="
echo "WT=$(realpath "$WT")"
echo

# --- M2 canonicalization battery (parity with H1 vectors 1-9,11,12) ---------
vec 1  "absolute write"        "echo PWNED1 > '$PARENT' && echo WROTE || echo BLOCKED"
vec 2  "relative traversal"    "cd '$WT' && (echo PWNED2 > '../outside/parent_canary' && echo WROTE || echo BLOCKED)"
vec 3  "symlink-deref"         "ln -sf '$PARENT' '$WT/lnk'; echo PWNED3 > '$WT/lnk' && echo WROTE || echo BLOCKED"
vec 4  "hardlink (sharpest)"   "ln '$PARENT' '$WT/hl' 2>&1 && (echo PWNED4 > '$WT/hl' && echo WROTE-VIA-HL || echo BLOCKED) || echo BLOCKED-LN-DENIED"
vec 5  "shared-.git analog"    "echo PWNED5 > '$GIT' && echo WROTE || echo BLOCKED"
vec 6  "/tmp alias write"      "echo PWNED6 > '$PTMP' && echo WROTE || echo BLOCKED"
vec 7  "\$HOME write"           "echo PWNED7 > '$HOMEC' && echo WROTE || echo BLOCKED"
vec 8  "child proc (python)"   "/usr/bin/python3 -c \"open('$PARENT','w').write('PWNED8')\" && echo WROTE || echo BLOCKED"
vec 9  "detached nohup &"      "nohup bash -c \"sleep 1; echo PWNED9 > '$PARENT'\" >/dev/null 2>&1 & disown; echo detached-launched; sleep 2"
vec 10 "setsid detached"       "setsid bash -c \"sleep 1; echo PWNED10 > '$PARENT'\" >/dev/null 2>&1 || true; echo setsid-tried; sleep 2"

# --- M3 macOS-only delegation vectors (the §5 residual — MEASURE) -----------
vec 11 "launchctl submit"      "launchctl submit -l h2probe -- /bin/bash -c \"echo PWNED11 > '$PARENT'\" 2>&1; echo submitted; sleep 2"
vec 12 "at job (file-deleg)"   "echo \"echo PWNED12 > '$PARENT'\" | at now 2>&1; echo at-tried; sleep 2"

# --- sentinel: prove the wrapper actually applied + wt writable -------------
vec 13 "wt writable + sentinel" "echo INSIDE_OK > '$WT/inside_marker' && cat '$WT/inside_marker'; echo TMPDIR=\$TMPDIR; (echo s > \$TMPDIR/scratch && echo TMP_WROTE || echo TMP_BLOCKED)"

echo
echo "Run canaries/verify.sh next — it is the source of truth."
