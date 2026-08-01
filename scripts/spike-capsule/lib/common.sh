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
