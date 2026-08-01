#!/usr/bin/env bash
# probe-capsule.sh — PHASE-02's observation harness (VA-3, VA-4, EX-1/2/4/5).
#
#   usage: probe-capsule.sh [posture|bounds|doorbell]
#   env:   SPIKE_CAPSULE_ROOT   capsule / fixture root (default: ~/capsules)
#
# Three sections, one file, because they observe one mechanism from three
# angles. PHASE-04's `probe-c2.sh` is the DISPATCHED confinement matrix and
# builds on this; this is the capsule side's own red/green.
#
# Every assertion here is on an OBSERVABLE — a path that does not resolve, a
# write that fails, a wall clock that elapsed — never on absence of error
# output (DQ-3, VA-3). Sections `posture` and `bounds` each carry a POSITIVE
# CONTROL, because a probe that observes nothing scores green against
# absence-shaped assertions alone.
set -euo pipefail

RIG_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
# shellcheck source-path=SCRIPTDIR
# shellcheck source=../lib/common.sh
. "${RIG_DIR}/lib/common.sh"

SANDBOX="${RIG_DIR}/capsule/sandbox.sh"

section=${1:-all}
case "${section}" in
  posture | bounds | doorbell | all) ;;
  -h | --help)
    sed -n '2,20p' "${BASH_SOURCE[0]}"
    exit 0
    ;;
  *) rig_die "unknown section: ${section}" ;;
esac

# I6 — FIRST, as a STATEMENT. Never inside `$( … )` (F-P01-1).
rig_enter

[ -x "${SANDBOX}" ] || rig_die "missing runner: ${SANDBOX} (PHASE-02 T2 provides it)"

capsule="${RIG_ROOT}/capsules/probe-p02"
guard_not_real_repo "${capsule}"

# The disk bound is CUMULATIVE over the capsule tree, so residue from one case
# is a real input to the next: the first run of the bounds section reddened its
# own positive control because the oversized case's file was still there. The
# reset is the fix; the cumulative behaviour is correct and stays.
reset_capsule() {
  rm -rf -- "${capsule}"
  mkdir -p -- "${capsule}"
}
reset_capsule

# Run a command inside the sandbox, capturing its status without tripping -e.
# The capsule's stdout is UNTRUSTED TEXT (I5) — read here only to assert on an
# observable the probe itself asked for, never parsed for a verdict.
in_sandbox() {
  local status=0
  "${SANDBOX}" --capsule "${capsule}" "$@" || status=$?
  return "${status}"
}

# The status itself, when an assertion needs to tell one nonzero from another —
# a script's own refusal (1) from an exec that never happened (RIG_EXIT_SANDBOX).
sandbox_status() {
  local status=0
  in_sandbox "$@" >/dev/null 2>&1 || status=$?
  printf '%s' "${status}"
}

# ── posture: EX-1 (rw / ro / ABSENT), EX-2 (one profile), EX-4 (I4a) ─────────

probe_posture() {
  printf '\nposture — EX-1 / EX-2 / EX-4, asserted on observables (VA-3)\n'

  # POSITIVE CONTROL. An absent-not-ro section made of "does not resolve"
  # assertions passes vacuously if NOTHING resolves — a sandbox that failed to
  # start scores green. This must resolve, or the rest proves nothing.
  rig_assert 'positive control: /nix/store RESOLVES inside the sandbox' \
    in_sandbox -- test -d /nix/store

  rig_assert 'rw: the capsule root is writable' \
    in_sandbox -- touch /capsule/probe-write

  # ABSENT, not ro (EX-1). Under an allowlist floor these are absent by
  # construction — asserted anyway, because "by construction" is exactly the
  # claim a later profile edit can silently break.
  local repo
  repo=$(rig_repo_root) || rig_die 'cannot resolve this repository root'
  absent_inside "ABSENT: the canonical repo (${repo}) does not resolve" "${repo}"
  absent_inside 'ABSENT: the capsule root (other capsules) does not resolve' "${RIG_ROOT}"
  # The GENERAL claim — nothing under the host home crosses except the two
  # allowlisted credential paths, which arrive at /agent. The home ROOT is the
  # subject that carries it, and unlike ~/.ssh it certainly exists.
  absent_inside 'ABSENT: the host home root does not resolve' "${HOME}"
  absent_inside 'ABSENT: ~/.ssh does not resolve' "${HOME}/.ssh"
  absent_inside 'ABSENT: ~/.gitconfig does not resolve' "${HOME}/.gitconfig"
  probe_credential_helper

  # EX-4 / I4a. The runners are reachable, read-only, and NOT under the
  # writable root — the third clause is the one a `cp` at provisioning breaks.
  rig_assert 'I4a: /rig/verify.sh resolves (ro-bound in)' \
    in_sandbox -- test -r /rig/verify.sh
  rig_assert 'I4a: /rig is READ-ONLY — the capsule cannot rewrite its runner' \
    runner_present_and_read_only /rig/verify.sh
  rig_assert 'I4a: no runner resolves under the writable root' \
    in_sandbox -- sh -c '! ls /capsule/verify.sh /capsule/sandbox.sh /capsule/provision.sh 2>/dev/null | grep -q .'

  # RESOLVES is not RUNS. The runners' shebang interpreter is a mount
  # dependency of its own, and `execvp` reports a missing interpreter as if the
  # SCRIPT were missing — so every readability assertion above passed while all
  # three runners were unexecutable (F-P02-2). This is the leg that notices.
  rig_assert_eq 'I4a: a ro-bound runner EXECUTES — its own refusal, not exec failure' \
    1 "$(sandbox_status -- /rig/verify.sh)"

  # EX-2 — ONE profile serves both kinds. Compare the emitted mount posture
  # with the kind swapped: identical mounts, or the uniform-confinement claim
  # is two mechanisms wearing one name.
  local worker verify
  worker=$("${SANDBOX}" --capsule "${capsule}" --kind worker --print-mounts)
  verify=$("${SANDBOX}" --capsule "${capsule}" --kind verify --print-mounts)
  rig_assert_eq 'EX-2: worker and verify kinds share ONE mount posture' \
    "${worker}" "${verify}"
}

# An ABSENT-not-ro assertion is only worth its green if its subject is VISIBLE
# TO THE PROBE. `test ! -e` against a path the probe cannot see either passes
# for the wrong reason and says nothing about the CAPSULE profile — F-P02-1's
# shape, and it was live on two legs of this section until F-P03-2. So: prove
# the subject reachable OUTSIDE the capsule, then assert it does not resolve
# INSIDE. A subject that is not reachable outside is recorded `n/a` WITH ITS
# REASON — a legal outcome; a silent green is not.
#
# ENVIRONMENT-CONDITIONAL, and deliberately so (PHASE-01 EX-9's discipline).
# `~/.ssh` exists on the host but is hidden by the OUTER bubblewrap jail, so
# in-jail this leg is `n/a` — the jail is doing the hiding, not the capsule,
# and the leg would pass with no capsule sandbox at all. On a HOST run the
# subject is visible and the same leg becomes load-bearing without an edit.
# That self-adaptation is the point: the gate is reachability from where the
# probe stands, never a hardcoded verdict about the path.
absent_inside() {
  local desc=$1 path=$2
  if [ ! -e "${path}" ]; then
    printf '  n/a   %s — subject not visible from here; the capsule is not what hides it\n' "${desc}"
    return 0
  fi
  rig_assert "${desc}" in_sandbox -- test '!' -e "${path}"
}

# The credential-helper leg needs two positive controls of its own, for two
# distinct vacuity routes (F-P03-2):
#
#   1. `git` might not be on the sandbox's allowlist at all, in which case the
#      substitution comes back empty and `[ -z … ]` is true for the wrong
#      reason. RESOLVES is not RUNS (F-P02-2), and neither is ABSENT.
#   2. NO helper is configured on this host, so `[ -z … ]` is true UNCONFINED.
#      Without a control the leg cannot tell "the profile hides the helper"
#      from "there was never a helper" — it is the DQ-3 trap wearing a
#      credential costume.
#
# Control 2 runs on the host against a config THIS PROBE OWNS; the operator's
# real ~/.gitconfig is never written. It exercises the global cascade, which is
# the route ~/.gitconfig itself takes.
probe_credential_helper() {
  rig_assert 'positive control: git EXECUTES inside the sandbox' \
    in_sandbox -- git --version

  local fake="${RIG_ROOT}/probes/p02-gitconfig"
  mkdir -p -- "$(dirname -- "${fake}")"
  rm -f -- "${fake}"
  git config --file "${fake}" credential.helper 'store --file=/dev/null'
  # The substitution runs under the probe's own GIT_CONFIG_GLOBAL, so it is
  # deliberately not expanded here either.
  # shellcheck disable=SC2016
  rig_assert 'positive control: the check DETECTS a helper when one is present' \
    env GIT_CONFIG_GLOBAL="${fake}" \
    sh -c '[ -n "$(git config --get credential.helper)" ]'

  # And none crosses. ~/.gitconfig is not bound, and `--clearenv` drops
  # GIT_CONFIG_GLOBAL — neither route reaches inside. The substitution must run
  # INSIDE, so it is deliberately not expanded here.
  # shellcheck disable=SC2016
  rig_assert 'ABSENT: no git credential helper on the effective cascade' \
    in_sandbox -- sh -c '[ -z "$(git config --get credential.helper || true)" ]'
}

# A write that must FAIL — and fail for the RIGHT REASON. The subject has to
# resolve first: appending to a path that does not exist also fails, so without
# this leg a deleted runner scores the same green as a read-only one. Observed
# passing vacuously when `/rig/verify.sh` did not yet exist (F-P02-1) — the
# same shape as F-P01-3, and precisely the DQ-3 trap VA-3 names.
runner_present_and_read_only() {
  in_sandbox -- test -e "$1" || return 1
  ! in_sandbox -- sh -c "printf x >> '$1'"
}

# ── bounds: EX-3 observed BITING, not merely coded (VA-4) ───────────────────

probe_bounds() {
  printf '\nbounds — EX-3 observed biting (VA-4)\n'

  # POSITIVE CONTROL. A wall-clock assertion made of "it was killed" passes
  # vacuously if the sandbox kills everything; a run well inside the bound must
  # survive it.
  rig_assert 'positive control: a short run SURVIVES the wall-clock bound' \
    in_sandbox --timeout 30 -- sh -c 'exit 0'

  # Wall clock. Asserted on TWO observables — the status, and the elapsed time
  # — because a command that failed instantly for an unrelated reason also
  # exits nonzero.
  local t0 t1 status=0 elapsed
  t0=$(date +%s)
  in_sandbox --timeout 3 -- sleep 600 || status=$?
  t1=$(date +%s)
  elapsed=$((t1 - t0))
  rig_assert_eq "wall clock: a hung run is KILLED (status ${status})" \
    "${RIG_EXIT_TIMEOUT}" "${status}"
  rig_assert "wall clock: killed at the bound, not at the 600s sleep (${elapsed}s)" \
    test "${elapsed}" -lt 30

  # Disk. 64 MiB written against an 8 MiB cap — a clean capsule per case,
  # because the bound is cumulative over the tree.
  local cap=$((8 * 1024 * 1024))
  reset_capsule
  status=0
  in_sandbox --disk-cap "${cap}" -- \
    sh -c 'dd if=/dev/zero of=/capsule/fat bs=1M count=64 2>/dev/null' || status=$?
  rig_assert_eq "disk: an oversized write is REFUSED (status ${status})" \
    "${RIG_EXIT_DISK}" "${status}"

  # POSITIVE CONTROL. Without it, a sandbox that refused every write would
  # score the assertion above green.
  reset_capsule
  rig_assert 'disk: positive control — a small write inside the cap succeeds' \
    in_sandbox --disk-cap "${cap}" -- \
    sh -c 'dd if=/dev/zero of=/capsule/thin bs=1K count=64 2>/dev/null'

  # The disk cap has TWO legs and both must REPORT, not merely bite. `du` is
  # cumulative and trusted-side; `ulimit -f` is per-file and fires as SIGXFSZ,
  # which reaches the parent as a raw status. The case above does not separate
  # them — it passes because `du` reports 8392704 against a cap of 8388608, one
  # 4096-byte block of accounting slop, so the 64 MiB overshoot is doing no
  # work (a 64 KiB overshoot is byte-identical). A SPARSE oversize separates
  # them: `ulimit -f` fires, the tree stays at 4096B, and `du` has nothing to
  # say. Observed reporting SUCCESS before this leg existed — the bound bit and
  # the sandbox called it a pass (F-P03-1).
  reset_capsule
  rig_assert_eq 'disk: a SPARSE oversize — the per-file leg REPORTS, not just du' \
    "${RIG_EXIT_DISK}" \
    "$(sandbox_status --disk-cap "${cap}" -- \
      sh -c 'dd if=/dev/zero of=/capsule/sparse bs=1 count=1 seek=16777215 2>/dev/null')"

  # EX-3 says BOTH capsule kinds, so both are observed. The kinds share one
  # code path, which is a reason to expect this to hold — not evidence that it
  # does. Inferring the verify kind from the worker kind is the same move as
  # scoring a matrix cell that never ran.
  reset_capsule
  rig_assert_eq 'verify kind: a hung run is KILLED at the same bound' \
    "${RIG_EXIT_TIMEOUT}" "$(sandbox_status --kind verify --timeout 3 -- sleep 600)"
  reset_capsule
  rig_assert_eq 'verify kind: an oversized write is REFUSED at the same cap' \
    "${RIG_EXIT_DISK}" \
    "$(sandbox_status --kind verify --disk-cap "${cap}" -- \
      sh -c 'dd if=/dev/zero of=/capsule/fat bs=1M count=64 2>/dev/null')"
}

# ── doorbell: EX-5, four properties ─────────────────────────────────────────

probe_doorbell() {
  printf '\ndoorbell — EX-5, four properties\n'

  # The CONSTANT, never a fourth literal. A probe carrying its own copy of the
  # name cannot catch the ringer drifting away from the waiter — it would just
  # keep observing the bell it planted itself.
  local bell="${capsule}/${RIG_DOORBELL}"
  reset_capsule

  # Loss degrades to polling with a deadline: no ring, and the wait RETURNS
  # (refusing) rather than hanging. Latency, not correctness.
  local status=0
  rig_wait_doorbell "${capsule}" 2 1 || status=$?
  rig_assert_eq 'no ring: the wait ends at its deadline, it does not hang' \
    "${RIG_EXIT_TIMEOUT}" "${status}"

  # Content is never read. A ring whose content names ANOTHER capsule is still
  # just a ring — identity comes from the argument, which the control plane
  # chose (I5).
  printf 'capsule=somebody-else oid=deadbeef\n' >"${bell}"
  rig_assert 'a ring naming another capsule is accepted as a bare signal' \
    rig_wait_doorbell "${capsule}" 5 1

  # Duplication is a no-op (I2): ringing again changes nothing observable.
  printf 'rung twice\n' >>"${bell}"
  rig_assert 'a duplicate ring is a no-op' \
    rig_wait_doorbell "${capsule}" 5 1

  # The waiter reports the capsule IT WAS ASKED ABOUT, never one the file names.
  rig_assert_eq 'identity comes from the control plane, not the file' \
    "${capsule}" "$(rig_wait_doorbell "${capsule}" 5 1)"

  # ── the JOIN ───────────────────────────────────────────────────────────────
  #
  # Everything above plants its own bell, so it proves the waiter works against
  # a bell the PROBE rang. That the waiter hears the bell the WORKER rings is a
  # different claim, and until this assertion existed it was inferred from the
  # two halves being green — the same adjacent-observable move as F-P02-1/2.
  # Both sides now take the name from RIG_DOORBELL, and this is what would red
  # if either drifted.
  local light="${RIG_ROOT}/fixtures/light/repo"
  rig_assert 'precondition: the light fixture is built (F1)' test -d "${light}"
  if [ -d "${light}" ]; then
    local base
    base=$(git -C "${light}" rev-parse HEAD)
    reset_capsule
    rig_assert 'live: no bell before the worker runs (negative control)' \
      test '!' -e "${capsule}/${RIG_DOORBELL}"
    "${SANDBOX}" --capsule "${capsule}" --source "${light}" -- \
      /rig/provision.sh "${base}" >/dev/null 2>&1
    "${SANDBOX}" --capsule "${capsule}" -- /rig/worker-stub.sh >/dev/null 2>&1
    rig_assert 'live: the waiter hears the bell the WORKER rang' \
      rig_wait_doorbell "${capsule}" 10 1
  fi
}

case "${section}" in
  posture) probe_posture ;;
  bounds) probe_bounds ;;
  doorbell) probe_doorbell ;;
  all)
    probe_posture
    probe_bounds
    probe_doorbell
    ;;
esac

rig_assert_done "probe-capsule (${section})"
