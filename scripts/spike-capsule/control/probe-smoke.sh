#!/usr/bin/env bash
# probe-smoke.sh — A2, as TWO assertions (EX-7, VA-1).
#
#   usage: probe-smoke.sh                       (dispatched by `rig smoke`)
#   env:   SPIKE_CAPSULE_ROOT   capsule / fixture root (default: ~/capsules)
#
# ── why two, and never one ───────────────────────────────────────────────────
#
# **Credential availability and network egress are distinct failure modes and a
# single test conflates them** (A8). `claude -p` failing tells you nothing on
# its own: the capsule may have no route out, or a route and no credential, and
# those have opposite fixes. So:
#
#   1. UNAUTHENTICATED reachability — is there a route out of the sandbox?
#   2. AUTHENTICATED `claude -p 'print OK'` — does the credential survive the
#      nested bwrap and the read-only agent home?
#
# Run separately, recorded separately. Run EARLY (§ 5.4 step 2) and near-free,
# because a failure here means the capsule model needs a CREDENTIAL-PROXY
# DESIGN, and that is worth learning on day one rather than after the pipeline
# is built on top of it (R3).
set -euo pipefail

RIG_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
# shellcheck source-path=SCRIPTDIR
# shellcheck source=../lib/common.sh
. "${RIG_DIR}/lib/common.sh"

SANDBOX="${RIG_DIR}/capsule/sandbox.sh"
SMOKE_TIMEOUT=120

case "${1:-}" in
  "") ;;
  -h | --help)
    sed -n '2,6p' "${BASH_SOURCE[0]}"
    exit 0
    ;;
  *) rig_die "unknown argument: $1" ;;
esac

# I6 — FIRST, as a STATEMENT (F-P01-1).
rig_enter

[ -x "${SANDBOX}" ] || rig_die "missing runner: ${SANDBOX}"

capsule="${RIG_ROOT}/capsules/smoke"
guard_not_real_repo "${capsule}"
rm -rf -- "${capsule}"
mkdir -p -- "${capsule}"

report="${RIG_ROOT}/probes/smoke/results.tsv"
mkdir -p -- "$(dirname -- "${report}")"
printf 'assertion\tmechanism\toutcome\tdetail\n' >"${report}"

record() {
  printf '%s\t%s\t%s\t%s\n' "$1" "$2" "$3" "$4" >>"${report}"
  printf '  %-6s %s — %s\n' "$3" "$1" "$4"
}

in_sandbox() {
  "${SANDBOX}" --capsule "${capsule}" --timeout "${SMOKE_TIMEOUT}" -- "$@"
}

# The sandbox narrates its own bound accounting on stderr, and these legs
# capture stderr to report WHY a leg failed. Without this the rig's own
# diagnostics land in the evidence file as though the capsule had said them —
# untrusted text and trusted text in one column (I5's whole point).
strip_rig_noise() {
  grep -v '^rig: ' || true
}

printf '\nA2 smoke — TWO assertions, recorded separately (EX-7, VA-1)\n'

# ── 1. unauthenticated reachability ─────────────────────────────────────────
#
# `npm ping` reaches the public registry with no credential of ours. It
# exercises DNS, TLS trust, and egress — the three things the profile has to
# get right for the second assertion to be about credentials at all.
status=0
out=$(in_sandbox npm ping 2>&1 | strip_rig_noise) || status=$?
if [ "${status}" -eq 0 ]; then
  record network unauthenticated pass 'npm ping reached the registry'
else
  record network unauthenticated FAIL "npm ping exit ${status}: $(printf '%s' "${out}" | tail -1)"
fi
network_status=${status}

# ── 2. authenticated claude -p ──────────────────────────────────────────────
#
# The credential arrives by the profile's read-only bind of the agent home. A
# failure HERE, with assertion 1 green, is the credential-proxy signal — the
# two are only separable because they were asserted separately.
status=0
out=$(in_sandbox claude -p 'print OK' 2>&1 | strip_rig_noise) || status=$?
if [ "${status}" -eq 0 ]; then
  record credential authenticated pass "claude -p answered: $(printf '%s' "${out}" | tr -d '\n' | cut -c1-40)"
else
  record credential authenticated FAIL "claude -p exit ${status}: $(printf '%s' "${out}" | tr -d '\n' | tail -c 120)"
fi
credential_status=${status}

printf '\nresults: %s\n' "${report}"

# Reported as a PAIR, because the interesting outcome is the combination. A
# green route with a red credential is the R3 finding; both red is a profile
# problem and says nothing about credentials yet.
if [ "${network_status}" -ne 0 ] && [ "${credential_status}" -ne 0 ]; then
  rig_warn 'A2: both legs failed — this is a PROFILE fault, not a credential result'
elif [ "${network_status}" -eq 0 ] && [ "${credential_status}" -ne 0 ]; then
  rig_warn 'A2: route out, no credential — the capsule model needs a credential proxy (R3)'
fi

[ "${network_status}" -eq 0 ] && [ "${credential_status}" -eq 0 ] || exit 1
printf 'A2: both assertions hold\n'
