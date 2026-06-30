#!/usr/bin/env bash
# Seatbelt jail builder — shell analog of the SL-183 `seatbelt_profile(policy)` +
# `sandbox_exec_argv(wt, policy)` Rust seam. DISPOSABLE PROBE apparatus, not the
# landed arm. Mirrors probe-h1/pretooluse-wrap.sh's idiom (opaque base64 body,
# fail-closed, realpath'd params) so the eventual Rust is a faithful port.
#
# Model (brief §1, INVERSE of bwrap): allow-default, deny file-write*, re-allow
# writes under the worktree (+ validated extra_rw). NOT SBPL default-deny.
#
# THE FOOTGUN (brief §4 / INV-5 twin): `subpath` matches the RESOLVED path and
# macOS aliases /tmp->/private/tmp etc. Feed REALPATHS into every -D param; never
# string-splice paths into the profile body.
set -u

# --- profile body (params, not interpolation) -------------------------------
# Emitted once; -D binds WT and RW0..RWn + TMP at invocation. network knob and
# /private/tmp deny are appended by the caller per policy (see build_profile).
emit_profile() {
  # args: <network:true|false> <deny_private_tmp:0|1> <n_extra_rw>
  local network="$1" deny_ptmp="$2" n_rw="$3" i
  # SBPL is LAST-MATCH-WINS (probe F-A). Order matters:
  #   floor deny -> coarse scratch deny -> SPECIFIC re-allows last (so they win).
  # If the WT lives UNDER /private/tmp (where macOS temp worktrees land), an
  # earlier `deny PTMP` would otherwise shadow the WT allow. Emit deny FIRST.
  echo '(version 1)'
  echo '(allow default)'                                   # nothing hidden; reads open (parity: reads OOS)
  echo '(deny file-write*)'                                # the floor
  [ "$deny_ptmp" = "1" ] && echo '(deny file-write* (subpath (param "PTMP")))'  # collapse global scratch (BEFORE allows)
  # device write surface MUST stay open or tooling breaks (probe F-B: /dev/null
  # denial broke python3/xcrun). Re-allow the standard device sinks.
  echo '(allow file-write* (literal "/dev/null"))'
  echo '(allow file-write* (literal "/dev/zero"))'
  echo '(allow file-write* (literal "/dev/dtracehelper"))'
  echo '(allow file-write* (subpath "/dev/fd"))'
  echo '(allow file-write* (regex #"^/dev/tty"))'          # /dev/tty, /dev/ttys00N
  echo '(allow file-write* (literal "/dev/stdout"))'
  echo '(allow file-write* (literal "/dev/stderr"))'
  # the worktree + scratch + extra_rw — SPECIFIC re-allows LAST so they win over
  # the floor deny AND any coarse PTMP deny above (WT-under-/private/tmp case).
  echo '(allow file-write* (subpath (param "WT")))'        # the worktree, rw
  echo '(allow file-write* (subpath (param "TMP")))'       # TMPDIR=<wt>/.tmp (D-mac3), realpath'd
  for ((i=0; i<n_rw; i++)); do
    echo "(allow file-write* (subpath (param \"RW$i\")))"  # one per validated extra_rw
  done
  [ "$network" = "false" ] && echo '(deny network*)'       # coarse: syscall-deny, not iface removal (M3/egress non-goal)
}

# --- argv builder: realpath every path param, opaque base64 body -------------
# Usage: seatbelt_run <wt> <network:true|false> <deny_private_tmp:0|1> \
#                     <profile_out> -- <command...>   [extra_rw paths via $EXTRA_RW array]
# Env in: EXTRA_RW (bash array of abs paths, optional)
seatbelt_run() {
  local wt="$1" network="$2" deny_ptmp="$3" prof="$4"; shift 4
  [ "$1" = "--" ] && shift
  local cmd="$*"

  # realpath the floor params (THE footgun mitigation)
  local rwt rtmp rptmp
  rwt="$(realpath "$wt")" || { echo "FAIL realpath wt=$wt" >&2; return 2; }
  mkdir -p "$rwt/.tmp"
  rtmp="$(realpath "$rwt/.tmp")"
  rptmp="$(realpath /tmp)"           # /private/tmp

  # validated extra_rw -> -D RW0.. (realpath each; footgun-validation is shared
  # Rust `validate_policy`, out of scope for the probe — we just realpath here)
  local -a dflags=(-D "WT=$rwt" -D "TMP=$rtmp" -D "PTMP=$rptmp")
  local n_rw=0 e
  if [ "${EXTRA_RW+set}" = set ]; then
    for e in "${EXTRA_RW[@]}"; do
      local re; re="$(realpath "$e")" || { echo "FAIL realpath extra_rw=$e" >&2; return 2; }
      dflags+=(-D "RW$n_rw=$re"); n_rw=$((n_rw+1))
    done
  fi

  emit_profile "$network" "$deny_ptmp" "$n_rw" > "$prof"

  # opaque wrap: base64 the original; decode+exec INSIDE the sandbox. children
  # inherit (probe M3). TMPDIR redirected into the rw worktree (D-mac3).
  local b64; b64="$(printf '%s' "$cmd" | base64)"
  sandbox-exec "${dflags[@]}" -f "$prof" -- \
    /bin/bash -c "export TMPDIR='$rtmp'; printf %s '$b64' | base64 -d | /bin/bash"
}

# Allow sourcing (for the runner) or direct CLI use for ad-hoc checks.
if [ "${BASH_SOURCE[0]}" = "$0" ]; then
  # direct: seatbelt-jail.sh <wt> <network> <deny_ptmp> <profile_out> -- <cmd...>
  seatbelt_run "$@"
fi
