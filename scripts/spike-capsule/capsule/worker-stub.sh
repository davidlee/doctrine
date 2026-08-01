#!/usr/bin/env bash
# capsule/worker-stub.sh — the scripted worker (DEC-103 default).
#
#   usage: worker-stub.sh [<message>]           (runs INSIDE the sandbox)
#
# Deterministic by design. `--stub` is the rig's default and `--agent` is
# explicit opt-in, because only P-C1 needs an LLM — everything in P-C2 and P-C3
# is scripted by DQ-2 mandate, and a probe "contained" by a worker politely
# declining is void.
#
# Inside the capsule the worker has TOTAL FREEDOM (I1a): commit, amend, rebase,
# iterate. The boundary is where the constraint lives, not the workspace. So
# this makes an ordinary commit and rings — nothing here is a permission check.
set -euo pipefail

INNER_CAPSULE=/capsule
CLONE="${INNER_CAPSULE}/repo"
RESULT_REF=refs/heads/capsule-result
DOORBELL="${INNER_CAPSULE}/result-ready"

die() {
  printf 'worker-stub: %s\n' "$*" >&2
  exit 1
}

message=${1:-'[add] capsule stub worker touch'}

[ -d "${CLONE}" ] || die "nothing provisioned at ${CLONE} — run provision.sh first"
cd -- "${CLONE}"

printf 'stub worker was here\n' >>capsule-stub.txt
git add -- capsule-stub.txt
git commit --quiet -m "${message}"

# The result the control plane will harvest. A REF the capsule owns, at a name
# the control plane chose — the capsule never names the harvest path, and the
# ref is read exactly once and pinned to an OID trusted-side (RT-5).
git update-ref "${RESULT_REF}" HEAD

# Ring. The doorbell carries NO AUTHORITY: content is never read, so what goes
# in the file is irrelevant by construction rather than by convention. Written
# non-empty on purpose — a rig that only ever rang with an empty file would not
# be exercising the "content is never read" claim at all.
printf 'capsule=%s oid=%s\n' "${INNER_CAPSULE}" "$(git rev-parse HEAD)" >"${DOORBELL}"

printf 'worker-stub: %s at %s\n' "${RESULT_REF}" "$(git rev-parse HEAD)"
