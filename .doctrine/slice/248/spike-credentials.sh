#!/usr/bin/env bash
# SL-248 credential-confinement spike. Read-only: every payload reports.
set -uo pipefail

REPORT='echo "  uid=$(id -u) gid=$(id -g) groups=$(id -G)";
        grep -E "^(Groups|CapInh|CapPrm|CapEff|CapBnd|NoNewPrivs):" /proc/self/status | sed "s/^/  /";
        echo "  userns=$(readlink /proc/self/ns/user)";
        echo "  uid_map=$(tr -s " " < /proc/self/uid_map | tr "\n" "|")"'

# NixOS: the payload's tools live in /nix/store, but the host PATH points at
# /run/current-system/sw/bin, which is deliberately NOT bound. Pin PATH to the
# store bin dirs (already reachable via the /nix bind) so no arm can return a
# false negative through an absent binary — the EVD-013 failure mode.
SYSPATH=$(dirname "$(readlink -f /run/current-system/sw/bin/id)")
SYSPATH+=":$(dirname "$(readlink -f /run/current-system/sw/bin/grep)")"
SYSPATH+=":$(dirname "$(readlink -f /run/current-system/sw/bin/sed)")"

# Bind only what a shell needs, and only what exists on this host.
binds=(); for d in /nix /usr /bin /lib /lib64 /etc; do
  [ -e "$d" ] && binds+=(--ro-bind "$d" "$d")
done
COMMON=("${binds[@]}" --proc /proc --dev /dev --setenv PATH "$SYSPATH")

arm() { # arm <label> <bwrap flags...>
  local label="$1"; shift
  echo "=== $label ==="
  if [ "$#" -eq 0 ]; then
    env PATH="$SYSPATH" /bin/sh -c "$REPORT" 2>&1
  else
    bwrap "$@" "${COMMON[@]}" /bin/sh -c "$REPORT" 2>&1
  fi
  echo "  exit=$?"
  echo
}

echo "### host"
uname -srm; echo "bwrap: $(command -v bwrap)"
bwrap --version
stat -Lc 'mode=%a owner=%U setuid=%A path=%n' "$(command -v bwrap)"
echo "max_user_namespaces=$(cat /proc/sys/user/max_user_namespaces 2>&1)"
echo "sandbox PATH=$SYSPATH"
echo

arm "P  — positive control, no bwrap at all"
arm "A1 — probe arm posture (--unshare-all)"                 --unshare-all
arm "A2 — CredentialsConfined as designed (--unshare-user dropped)" \
        --unshare-pid --unshare-ipc --unshare-uts --unshare-cgroup --unshare-net
arm "A3 — candidate: explicit identity"                      --unshare-all --uid 4242 --gid 4242
arm "A4 — candidate: identity delta removed"                 --unshare-all
arm "A5 — candidate: --cap-add ALL under unshare-all"        --unshare-all --cap-add ALL

echo "### A6 — does bwrap SET no_new_privs, or inherit it?"
echo "parent NoNewPrivs: $(grep NoNewPrivs /proc/self/status)"
echo "(If the parent already reads 1, this host cannot answer it either —"
echo " say so rather than reporting bwrap's 1 as evidence.)"
