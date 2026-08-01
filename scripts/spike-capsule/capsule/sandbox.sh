#!/usr/bin/env bash
# capsule/sandbox.sh — the P-C2 v0 confinement profile (EX-1, EX-2, EX-4, VT-1).
#
#   usage: sandbox.sh --capsule DIR [--kind worker|verify] [--source DIR]
#                     [--timeout SECS] [--disk-cap BYTES] [--no-net]
#                     [--print-mounts] -- COMMAND…
#
#   env:   SPIKE_CAPSULE_ROOT   capsule / fixture root (default: ~/capsules)
#          SPIKE_SANDBOX_TIMEOUT / SPIKE_SANDBOX_DISK_CAP   bound defaults
#
# ONE PROFILE, BOTH KINDS (EX-2). The capsule kind selects the COMMAND and the
# BOUNDS; it never selects a profile. That is the whole of P-C2's uniform-
# confinement claim and RT-1's verify capsule being satisfied by one mechanism
# rather than two — so `--print-mounts` deliberately emits the mount posture
# ALONE, and the two kinds' output must compare equal.
#
# ── the floor is an ALLOWLIST, and that is load-bearing ──────────────────────
#
# `pi-spawn-confined.sh` (ADR-008 D-B3, this profile's seed) opens with
# `--ro-bind / /`: everything is READABLE and only writes are denied. That is a
# WRITABILITY floor, and EX-1 needs a VISIBILITY one — the canonical repo, the
# other capsules, and the git credentials must be ABSENT, not read-only. Absent
# is achieved by NOT BINDING, which only an allowlist can express. What
# transfers from the seed is the nesting mechanics and the fail-closed empty-
# argv guard, not the floor.
#
#   rw      the capsule dir, at /capsule — the ONLY --bind
#   ro      /nix/store (the whole toolchain), DNS + TLS trust, the agent home
#           carrying the API credential at /agent, and the runners at /rig
#   ABSENT  the canonical repo · other capsules · ~/.ssh · ~/.gitconfig ·
#           credential helpers · everything else
#
# ── I4a: the runners are ro-bound from OUTSIDE the writable root ─────────────
#
# The control-plane-authored runners mount at /rig, which is not under
# /capsule. So "no script the capsule can write is ever the process whose exit
# status is the verdict" is STRUCTURAL — there is no writable path on the
# runner's mount path at all. The ordinary reflex, `cp`-ing them into the
# capsule dir at provisioning, is all it takes to undo RT-1, and the rw bind
# would happily permit it. `control/audit-i4a.sh` is the standing check.
#
# ── the bounds are enforced TRUSTED-SIDE (EX-3) ──────────────────────────────
#
# `timeout` wraps the bwrap exec from outside; `ulimit -f` is set before it and
# inherited through the namespace; `du` runs after. Nothing the capsule can
# unset. They emit distinguishable STATUSES and no tokens: `verify-timeout` and
# `harvest/resource-cap` are trusted-side-computed in PHASE-03's pipeline (I5).
set -euo pipefail

RIG_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
# shellcheck source-path=SCRIPTDIR
# shellcheck source=../lib/common.sh
. "${RIG_DIR}/lib/common.sh"

# ── inner layout — named once (STD-001) ──────────────────────────────────────
#
# Fixed inner paths rather than the host's. `--bind "$D" "$D"` (the seed's
# form) would reproduce the capsule's host path inside, which reads the parent
# directory of every other capsule into existence as a mountpoint chain.
SANDBOX_CAPSULE=/capsule
SANDBOX_RIG=/rig
SANDBOX_HOME=/agent
SANDBOX_SOURCE=/source

SANDBOX_KILL_GRACE=5     # seconds between SIGTERM and SIGKILL at the bound
SANDBOX_DISK_BLOCK=1024  # `ulimit -f` counts 1024-byte blocks (probed, not assumed)

kind=worker
capsule=""
source_dir=""
timeout_secs="${SPIKE_SANDBOX_TIMEOUT:-300}"
disk_cap="${SPIKE_SANDBOX_DISK_CAP:-$((256 * 1024 * 1024))}"
net=1
print_mounts=0
cmd=()

while [ $# -gt 0 ]; do
  case "$1" in
    --capsule)
      capsule=${2:?--capsule needs a directory}
      shift
      ;;
    --kind)
      kind=${2:?--kind needs worker|verify}
      shift
      ;;
    --timeout)
      timeout_secs=${2:?--timeout needs seconds}
      shift
      ;;
    --disk-cap)
      disk_cap=${2:?--disk-cap needs bytes}
      shift
      ;;
    --source)
      source_dir=${2:?--source needs a directory}
      shift
      ;;
    --no-net) net=0 ;;
    --print-mounts) print_mounts=1 ;;
    --)
      shift
      cmd=("$@")
      break
      ;;
    -h | --help)
      sed -n '2,8p' "${BASH_SOURCE[0]}"
      exit 0
      ;;
    *) rig_die "unknown argument: $1" ;;
  esac
  shift
done

case "${kind}" in
  worker | verify) ;;
  *) rig_die "unknown capsule kind: ${kind} (worker|verify)" ;;
esac
[ -n "${capsule}" ] || rig_die "--capsule is required"

# I6 — FIRST, as a STATEMENT. Inside `$( … )` the refusal would end only the
# substitution's subshell and the sandbox would start anyway (F-P01-1).
rig_enter

capsule=$(rig_resolve "${capsule}")
guard_not_real_repo "${capsule}"
[ -d "${capsule}" ] || rig_die "capsule does not exist: ${capsule}"

runners="${RIG_DIR}/capsule"
[ -d "${runners}" ] || rig_die "missing runner directory: ${runners}"

# ── the toolchain allowlist ──────────────────────────────────────────────────
#
# PATH is the host's, FILTERED to entries under /nix/store — the one ro-bind
# that backs all of them. Everything else the host offers is dropped, and
# `~/.cargo/bin` being dropped is the point: the capsule gets the nix
# toolchain, not the control plane's binaries.
sandbox_path() {
  local dir out=""
  while IFS= read -r dir; do
    case "${dir}" in
      /nix/store/*) [ -d "${dir}" ] && out="${out}${out:+:}${dir}" ;;
      *) ;;
    esac
  done < <(printf '%s\n' "${PATH}" | tr ':' '\n')
  printf '%s' "${out}"
}

inner_path=$(sandbox_path)
[ -n "${inner_path}" ] || rig_die "no /nix/store entries on PATH — cannot build a toolchain"

# TLS trust and DNS. Egress is a capsule-model property EX-7 asserts, so the
# two things that make it work are named here rather than discovered later.
# The CA bundle already lives in the store, so it needs no bind of its own.
ca_bundle=$(rig_resolve /etc/ssl/certs/ca-bundle.crt)

# ── the profile ──────────────────────────────────────────────────────────────
#
# Mount posture ONLY. Bounds, the command, and the kind are deliberately not
# here: EX-2 is the claim that both kinds get the same posture, and this array
# is what makes that claim checkable rather than asserted.
mounts=(
  --unshare-all
  --proc /proc
  --dev /dev
  --tmpfs /tmp
  --ro-bind /nix/store /nix/store
  # `#!/usr/bin/env bash` is the rig's shebang everywhere, and the kernel
  # resolves it BEFORE PATH exists — so without this bind every runner fails
  # as `execvp: No such file or directory`, naming the SCRIPT while the file
  # that is actually missing is the interpreter (F-P02-2).
  --ro-bind /usr/bin/env /usr/bin/env
  --bind "${capsule}" "${SANDBOX_CAPSULE}"
  --ro-bind "${runners}" "${SANDBOX_RIG}"
  --chdir "${SANDBOX_CAPSULE}"
  --die-with-parent
  --new-session
  --clearenv
  --setenv PATH "${inner_path}"
  --setenv HOME "${SANDBOX_HOME}"
  --setenv SSL_CERT_FILE "${ca_bundle}"
  --setenv NODE_EXTRA_CA_CERTS "${ca_bundle}"
  --setenv TERM dumb
)

# The contracted source, READ-ONLY at /source (EX-8). The capsule clones from
# it INSIDE the sandbox, so no tree is materialised trusted-side (I4) and the
# toolchain reaches the work by ro-bind rather than by `nix`/`direnv`, neither
# of which exists in this jail. Read-only because a capsule that could write
# its own contracted base would make the base meaningless.
if [ -n "${source_dir}" ]; then
  source_dir=$(rig_resolve "${source_dir}")
  guard_not_real_repo "${source_dir}"
  [ -d "${source_dir}" ] || rig_die "source does not exist: ${source_dir}"
  mounts+=(--ro-bind "${source_dir}" "${SANDBOX_SOURCE}")
fi

# The agent home carrying the API credential (EX-1). Bound at /agent, so the
# host's home — with ~/.ssh and ~/.gitconfig in it — is never a mountpoint.
[ -d "${HOME}/.claude" ] &&
  mounts+=(--ro-bind "${HOME}/.claude" "${SANDBOX_HOME}/.claude")
[ -f "${HOME}/.claude.json" ] &&
  mounts+=(--ro-bind "${HOME}/.claude.json" "${SANDBOX_HOME}/.claude.json")

# DNS. `--share-net` retains the network namespace; without resolv.conf the
# capsule has egress but no names, which would read as "no network" in EX-7's
# reachability assertion and blame the wrong mechanism.
if [ "${net}" -eq 1 ]; then
  mounts+=(--share-net)
  [ -e /etc/resolv.conf ] && mounts+=(--ro-bind /etc/resolv.conf /etc/resolv.conf)
fi

if [ "${print_mounts}" -eq 1 ]; then
  printf '%s\n' "${mounts[@]}"
  exit 0
fi

[ "${#cmd[@]}" -gt 0 ] || rig_die "no command — pass it after \`--\`"

# Fail-closed: an empty profile must never fall through to an unconfined exec
# (the seed's EX-2 guard, `pi-spawn-confined.sh:115`).
[ "${#mounts[@]}" -gt 0 ] || rig_die "empty confinement profile — refusing to run unconfined"

# ── run, bounded ─────────────────────────────────────────────────────────────
#
# The subshell is where `ulimit -f` is set so it applies to bwrap and every
# descendant; `timeout` is outside bwrap so the capsule cannot signal its own
# reaper. `|| status=$?` rather than `set -e` propagation: a nonzero status
# from the capsule is DATA here (I4 — the verdict is this exit status as the
# parent observes it), not an error in this script.
status=0
(
  ulimit -f $((disk_cap / SANDBOX_DISK_BLOCK))
  exec timeout -k "${SANDBOX_KILL_GRACE}" "${timeout_secs}" \
    bwrap "${mounts[@]}" "${cmd[@]}"
) || status=$?

# The command could not be executed at all — a missing binary, or a shebang
# interpreter that is not in the allowlist (F-P02-2). A RIG fact, not a verdict
# on the capsule, so it gets its own status instead of being folded into the
# command's. Distinguishing the two is what stops "the runner refused" and "the
# runner never ran" reading identically in a matrix cell.
[ "${status}" -eq 127 ] && status="${RIG_EXIT_SANDBOX}"

# The whole-tree disk cap. `ulimit -f` is a PER-FILE limit and a capsule that
# writes many small files walks past it; this is the leg that catches that.
used=$(du -s -B1 -- "${capsule}" | cut -f1)
if [ "${used}" -gt "${disk_cap}" ]; then
  rig_warn "sandbox: disk cap exceeded — ${used}B used, cap ${disk_cap}B"
  status="${RIG_EXIT_DISK}"
fi

rig_warn "sandbox: kind=${kind} status=${status} disk=${used}/${disk_cap}B timeout=${timeout_secs}s"
exit "${status}"
