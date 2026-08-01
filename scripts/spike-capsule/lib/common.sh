#!/usr/bin/env bash
# Shared library for the SL-241 capsule spike rig. SOURCED, never executed.
#
# Disposable scaffolding: the rig builds evidence for RFC-025's capsule
# authority model and nothing in it migrates into dispatch machinery
# (slice-241.md § Scope). Entry points set their own shell options; a library
# does not impose them on its caller.

# ── diagnostics ──────────────────────────────────────────────────────────────

# Exit codes are distinguishable on purpose: an I6 refusal must not read as a
# usage error in a matrix cell's recorded outcome.
RIG_EXIT_USAGE=2
RIG_EXIT_GUARD=3

rig_warn() { printf 'rig: %s\n' "$*" >&2; }
rig_die() { rig_warn "$*"; exit "${RIG_EXIT_USAGE}"; }

# ── path resolution (T3 / EX-8) ──────────────────────────────────────────────

# Canonicalise a path WITHOUT requiring it to exist (`-m`): the capsule root is
# created by the rig, so it does not exist the first time the guard inspects it.
# Resolution MUST precede every comparison — comparing an unresolved string is
# not a guard (mem.pattern.safety.resolve-every-ref-before-pure-compare).
rig_resolve() {
  realpath -m -- "${1:?rig_resolve: path required}"
}

# The capsule / fixture root. A rig PARAMETER, never a hardcoded path (EX-8).
# Default `~/capsules` — out of repo by operator ruling, which makes a
# mis-resolved root less likely and does NOT make the I6 guard optional.
rig_capsule_root() {
  rig_resolve "${SPIKE_CAPSULE_ROOT:-${HOME}/capsules}"
}

# The repository the rig itself lives in — I6's subject. Derived from this
# library's own location, so it is correct however the rig is invoked.
rig_repo_root() {
  local here top
  here=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P) || return 1
  top=$(git -C "${here}" rev-parse --show-toplevel 2>/dev/null) || return 1
  [ -n "${top}" ] || return 1
  rig_resolve "${top}"
}

# ── I6: the rig cannot touch the real repository ─────────────────────────────

# guard_not_real_repo <path>
#
# Refuses when <path> resolves to this repository's root, or to anything inside
# it. Runs at EVERY entry point BEFORE any provisioning — a guard that runs late
# is not a guard (design § 5.5 I6).
#
# EX-2 mandates equality; this refuses CONTAINMENT too (D-P01-2). Equality alone
# leaves `<repo>/scripts/spike-capsule/fixtures` open, and a mutator running
# there is the failure I6 exists to prevent — the criterion's own justification,
# not a widening of it.
#
# Fails CLOSED: a guard that cannot determine what it is protecting refuses.
guard_not_real_repo() {
  local candidate repo
  candidate=$(rig_resolve "${1:?guard_not_real_repo: path required}")
  repo=$(rig_repo_root) || {
    rig_warn "I6: cannot resolve this repository's root — refusing (fail closed)"
    exit "${RIG_EXIT_GUARD}"
  }
  # The `#` strip is a prefix test on RESOLVED paths with the separator
  # included, so a sibling (`<repo>-other`) is not swallowed. `"${repo}"` is
  # quoted inside the expansion so a glob character in the path is a literal.
  if [ "${candidate}" = "${repo}" ] || [ "${candidate#"${repo}"/}" != "${candidate}" ]; then
    rig_warn "I6: refusing — ${candidate} is the real repository (${repo}), or inside it"
    exit "${RIG_EXIT_GUARD}"
  fi
}

# The first action of EVERY entry point — `rig` and every control script alike.
# Resolves the capsule root, THEN guards it: resolution before comparison, or
# the guard compares an unresolved string.
#
# Publishes RIG_ROOT rather than printing it, and that is load-bearing. A guard
# that refuses by `exit` must run in the ENTRY SHELL: inside `$( … )` the exit
# ends only the subshell, and `set -e` does not propagate a failed substitution
# in argument position — so `dispatch … "$(rig_enter)"` printed the refusal and
# then dispatched anyway, with an empty root. Observed during T1 (F-P01-1).
rig_enter() {
  # Tripwire for exactly that. The guard probe subshells `guard_not_real_repo`
  # on purpose; `rig_enter` is the entry wrapper and must never be subshelled.
  if [ "${BASHPID}" != "$$" ]; then
    rig_warn "I6: rig_enter called from a subshell — a refusal could not propagate"
    exit "${RIG_EXIT_GUARD}"
  fi
  RIG_ROOT=$(rig_capsule_root)
  guard_not_real_repo "${RIG_ROOT}"
}

# ── assertions ───────────────────────────────────────────────────────────────
#
# Failures ACCUMULATE rather than exiting at the first, so one run reports every
# broken invariant instead of one per rebuild. `rig_assert_done` is the gate.

RIG_ASSERT_FAILURES=0

rig_assert() {
  local desc=$1
  shift
  if "$@" >/dev/null 2>&1; then
    printf '  ok    %s\n' "${desc}"
  else
    printf '  FAIL  %s\n' "${desc}" >&2
    RIG_ASSERT_FAILURES=$((RIG_ASSERT_FAILURES + 1))
  fi
}

rig_assert_eq() {
  local desc=$1 want=$2 got=$3
  if [ "${want}" = "${got}" ]; then
    printf '  ok    %s\n' "${desc}"
  else
    printf '  FAIL  %s — want %s, got %s\n' "${desc}" "${want}" "${got}" >&2
    RIG_ASSERT_FAILURES=$((RIG_ASSERT_FAILURES + 1))
  fi
}

rig_assert_done() {
  local what=$1
  if [ "${RIG_ASSERT_FAILURES}" -ne 0 ]; then
    rig_warn "${what}: ${RIG_ASSERT_FAILURES} assertion(s) failed"
    exit 1
  fi
  printf '%s: all assertions hold\n' "${what}"
}
