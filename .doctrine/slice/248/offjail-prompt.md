# Off-jail measurement prompt — SL-248 / VA-1

Hand the block below to a Claude instance running **outside** the bubblewrap
jail (auto mode is fine — the job is read-only). Paste back its whole stdout.

Everything it needs travels inline: the outside agent has no access to the
`sl-248` branch, and needs none.

---

You are running a **read-only** measurement for someone else's project. You are
not editing a repo, not committing, and not installing anything. Your entire job
is to run one script and report its exact output.

Do not modify it to "make it work". If something is missing, report that as the
finding — an absent binary is a real answer here, not an obstacle.

## Safety floor

This script only reads. Confirm before running that it contains no `kill`,
`rm`, `mv`, `>` redirect into a real path, `chmod`, or `sudo`. If your copy
does, stop and say so — you have the wrong text.

## Step 1 — three facts

```bash
command -v setsid || echo "setsid: ABSENT"
command -v bwrap  || echo "bwrap: ABSENT"
bwrap --version 2>&1 | head -1
grep NoNewPrivs /proc/self/status
uname -srm
```

If `bwrap` is absent, stop after this step and report it. The rest cannot run.

## Step 2 — write and run the spike

Write this to a scratch path (e.g. `/tmp/sl248-credentials.sh`), `chmod +x`, run
it, and capture stdout **and** stderr together.

```bash
#!/usr/bin/env bash
# SL-248 credential-confinement spike. Read-only: every payload reports.
set -uo pipefail

REPORT='echo "  uid=$(id -u) gid=$(id -g) groups=$(id -G)";
        grep -E "^(Groups|CapInh|CapPrm|CapEff|CapBnd|NoNewPrivs):" /proc/self/status | sed "s/^/  /";
        echo "  userns=$(readlink /proc/self/ns/user)";
        echo "  uid_map=$(tr -s " " < /proc/self/uid_map | tr "\n" "|")"'

# NixOS: the payload's tools live in /nix/store, but the ambient PATH points at
# /run/current-system/sw/bin, which is deliberately NOT bound into the sandbox.
# Pin PATH to the store bin dirs (reachable via the /nix bind) so no arm can
# return a false negative through an absent binary. Non-NixOS: keep ambient PATH.
if [ -d /run/current-system/sw/bin ]; then
  SYSPATH=$(dirname "$(readlink -f /run/current-system/sw/bin/id)")
  SYSPATH+=":$(dirname "$(readlink -f /run/current-system/sw/bin/grep)")"
  SYSPATH+=":$(dirname "$(readlink -f /run/current-system/sw/bin/sed)")"
else
  SYSPATH="$PATH"
fi

# Bind only what a shell needs, and only what exists on this host. All read-only.
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
```

## Step 3 — report

Paste the complete raw output of steps 1 and 2. Then answer these four
questions in one line each, from that output only — say "cannot tell from this
output" rather than reasoning past it:

1. Does `A1` (probe arm) read `CapBnd: 0000000000000000` while `A5`
   (`--cap-add ALL`) reads a non-zero `CapBnd`?
2. Do `A1` and `A2` sit in **different** `userns` inodes?
3. What are `uid` and `gid` on the `P` arm (the unsandboxed control)? Is `uid`
   1000? Is `gid` 100?
4. Did the parent shell already read `NoNewPrivs: 1` before any bwrap ran?

Then delete the scratch script. Do not commit anything anywhere.

---

## Why these four questions (context for me, not for the outside agent)

- **Q1** confirms or falsifies the planner's `create_user_ns()` reading — that a
  new user namespace gets `CAP_FULL_SET` in its bounding set regardless of the
  parent's stripped set. The jail already measured this; the host run is
  confirmation, which is exactly what `VA-1`/`T14` asks for.
- **Q2** is the `CredentialsConfined` row's discriminator: dropping
  `--unshare-user` must leave the arms in different user namespaces.
- **Q3** decides `F-4`. Row 13's shipped `CAPSULE_UID`/`CAPSULE_GID` are both
  1000. If the host operator is uid 1000, `uid` and `uid_map` read identically
  on both arms and two of `EX-9`'s three declared fields are vacuous **on the
  design host too** — not a jail artefact. A `gid` of 100 leaves gid/`gid_map`
  as the only live discriminator.
- **Q4** is `A6`: whether bwrap sets `no_new_privs` or merely inherits it. The
  jail cannot answer it — the parent already reads 1.

Step 1's `setsid` line settles `S8` separately: if `setsid(1)` exists on the
host, row 7's shipped payload is measurable there and the jail is the only
place it is byte-identical across arms. That tells the T9 worker whether to
change the payload or record row 7 as environment-limited.
