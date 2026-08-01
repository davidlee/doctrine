#!/usr/bin/env bash
# capsule/provision.sh — clone the contracted base INSIDE the sandbox (EX-8).
#
#   usage: provision.sh <base-oid>            (runs INSIDE the sandbox)
#
# Runs from /rig, ro-bound from outside the capsule's writable root (I4a). It
# is a CONTROL-PLANE script that happens to execute in the capsule, not a
# capsule script — the distinction is the mount posture, and it is the whole of
# why the verdict means anything.
#
# ── the provisioning mechanism, pinned against this environment (EX-8) ───────
#
# probe-specs § P-C1 step 2 provisions with `direnv allow`. **`nix` and
# `direnv` are ABSENT in this jail** — verified at planning time; `/nix/store`,
# `bwrap`, `node`, `npm` and `claude` are present. So the toolchain reaches the
# capsule by RO-BINDING `/nix/store` and setting PATH, which is what the P-C2
# profile already specifies, and there is no `direnv allow` step because there
# is no binary to run it.
#
# That divergence is RECORDED, not silently worked around: it is an environment
# fact, and P-C1a records the design's "nix env ready" step `n/a` WITH ITS
# REASON rather than dropping it from the step list (PHASE-04 EX-4). RFC-025
# prose is NOT edited — a slice non-goal.
set -euo pipefail

INNER_CAPSULE=/capsule
INNER_SOURCE=/source
CLONE="${INNER_CAPSULE}/repo"

die() {
  printf 'provision: %s\n' "$*" >&2
  exit 1
}

base=${1:?usage: provision.sh <base-oid>}

[ -d "${INNER_SOURCE}" ] || die "no contracted source at ${INNER_SOURCE} — pass --source"
[ -d "${INNER_CAPSULE}" ] || die "not running inside a capsule (${INNER_CAPSULE} absent)"
[ -e "${CLONE}" ] && die "already provisioned: ${CLONE}"

# `--no-hardlinks` is not an optimisation knob (fixtures.md F4): a local clone
# hardlinks object files by default, so the capsule and the source would SHARE
# them and a hostile capsule corrupting a shared object corrupts the source. It
# is the difference between a copy and an alias. The ro bind makes the write
# fail rather than corrupt — belt and braces, and the belt is the one that
# survives someone making /source writable.
git clone --no-hardlinks --quiet -- "${INNER_SOURCE}" "${CLONE}"
git -C "${CLONE}" switch --detach --quiet "${base}"

# The capsule's own identity. Never the host's — a capsule that adopted the
# control plane's git identity would author commits indistinguishable from it.
git -C "${CLONE}" config user.name 'capsule worker'
git -C "${CLONE}" config user.email 'worker@spike-capsule.invalid'

# No remotes. The capsule has nowhere to push; harvest is a control-plane pull
# from the capsule (§ 5.2), never a capsule-initiated write outward.
git -C "${CLONE}" remote remove origin 2>/dev/null || true

printf 'provision: %s at %s\n' "${CLONE}" "$(git -C "${CLONE}" rev-parse HEAD)"
