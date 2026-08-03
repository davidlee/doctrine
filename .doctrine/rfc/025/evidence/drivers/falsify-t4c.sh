#!/usr/bin/env bash
# T4c's falsifiability round — NOT a rig artefact, and it never calls
# rows_write, so results.tsv is untouched.
#
#   usage: falsify-t4c.sh <case>
#
# Same claim as T4b's round: does the row's OWN `Hnn_planted` detect the
# perturbation that would hollow it out? H13 is the row where this matters
# most — THREE of its four legs are absence-shaped ("the bundle is gone", "it
# is truncated", "nothing was done to it"), and an absence-shaped observable is
# the class that passes when its subject was never reachable.
#
# Scaffolding and `rebind` live in `falsify-lib.sh`, shared with
# `falsify-t4b.sh`.
# shellcheck source=/workspace/doctrine/.doctrine/state/slice/241/drivers/falsify-lib.sh
. "$(dirname -- "${BASH_SOURCE[0]}")/falsify-lib.sh"
# AFTER the source — sourcing pipeline.sh re-enables set -e.
set -euo pipefail
case_id=${1:?usage: falsify-t4c.sh <case>}

# ── the mutations ───────────────────────────────────────────────────────────

# M6 — THE VACUITY MUTANT, and the one this row exists to survive. The capsule
# writes a ZERO-BYTE bundle, so "the bundle is gone at ring time" is true of an
# artifact that was never there. The leg's OWN observable still passes; only
# the pre-mutation size control reds. Without that control H13 would report a
# boundary holding against an attack that never happened.
mutate_m6() {
  rebind H13_mutate
  H13_mutate() {
    : >"$(c3_h13_bundle "$1")"
    real_H13_mutate "$@"
  }
}

# M7 — the symlink is never planted. The mutation is a no-op, so the harvest
# path holds the honest bundle and `-L` must red.
mutate_m7() { bundle_symlink() { :; }; }

# M8 — the truncation is never performed, so the bundle stays byte-identical
# and VALID. Both of the invalid leg's clauses must red.
mutate_m8() { bundle_truncate() { :; }; }

# M9 — the cap is raised ABOVE the honest bundle, so the threshold cannot bite.
# This is `resource-cap`'s vacuity trap: the leg would otherwise claim a cap
# refused something while the cap was never in play.
mutate_m9() { C3_H13_CAP=$((64 * 1024 * 1024)); }

# ── H14's mutants: M10–M14 ──────────────────────────────────────────────────
#
# H14's three legs are one cell, so one control covers them, and each mutant
# targets exactly one leg's clause. Three of the five (M12–M14) simply no-op one
# of the row's three named hostile acts — the shape `bundle_symlink` made
# available to M7/M8, and the reason those acts have names at all.

# M10 — THE VACUITY MUTANT for the JOIN. The worker rings a CONTENT-FREE bell,
# so "the ring was lost / forged" would be true of a bell that never announced
# anything. The row's `rung` clause is what says the bell under attack is the
# one the WORKER rang about THIS result; without it the row measures a doorbell
# the rig planted for itself (`probe_doorbell`'s own vacuity, one level in).
mutate_m10() {
  rebind H14_mutate
  H14_mutate() {
    printf '\n' >"$(c3_h14_bell "$1")"
    real_H14_mutate "$@"
  }
}

# M11 — LOSS STOPS DEGRADING TO POLLING. The waiter gives up instantly when the
# bell is absent instead of polling to its deadline. It still ENDS and still
# reports the timeout, so every clause but the elapsed one survives — which is
# the point: fail-fast and poll-to-deadline are indistinguishable on status
# alone, and only the elapsed clause separates latency from correctness.
#
# Wrapped so the honest path is unaffected: legs 1 and 3 wait on a bell that
# EXISTS and call through to the real waiter.
mutate_m11() {
  rebind rig_wait_doorbell
  rig_wait_doorbell() {
    [ -e "$1/${RIG_DOORBELL}" ] || return "${RIG_EXIT_TIMEOUT}"
    real_rig_wait_doorbell "$@"
  }
}

# M12 — the forgery is never written, so leg 3's ring is the worker's honest one
# and the no-authority clauses have no forgery to be indifferent to.
mutate_m12() { c3_h14_forge() { :; }; }

# M13 — the ring is never lost. The wait then finds the bell and returns 0, so
# leg 2 would otherwise be reporting a polling fallback it never entered.
mutate_m13() { c3_h14_silence() { :; }; }

# M14 — THE DUPLICATE NEVER HAPPENS, and this is the mutant that changed the
# row. Every other clause of leg 1 holds against a single ring — a waiter that
# answers the same way twice answers the same way when asked about one ring —
# so before the `rings` / `rings-distinct` clauses existed this mutant SURVIVED,
# and leg 1 would have reported duplication surviving an experiment in which
# nothing was duplicated.
mutate_m14() { c3_h14_rering() { :; }; }

# ── isolation controls ──────────────────────────────────────────────────────

isolate_m6() { # the leg's own observable STILL HOLDS — only the control reds
  local run=$1
  rig_assert 'm6 isolation: the bundle really is absent — the leg would have passed' \
    c3_path_absent "$(c3_h13_bundle "${run}")"
  rig_assert_eq 'm6 isolation: and the recorded pre-size is 0 — nothing was ever there' \
    0 "$(cat "${run}/h13-size.before")"
}

isolate_m7() { # only the symlink went: the honest artifact is untouched
  local run=$1 bundle
  bundle=$(c3_h13_bundle "${run}")
  rig_assert 'm7 isolation: the harvest path is a REGULAR file, not a symlink' \
    test -f "${bundle}"
  rig_assert_eq 'm7 isolation: and it is still the honest bundle, byte for byte' \
    "$(cat "${run}/h13-size.before")" "$(stat -c %s -- "${bundle}")"
}

isolate_m8() { # the bundle is unchanged AND still valid — the attack never ran
  local run=$1 bundle
  bundle=$(c3_h13_bundle "${run}")
  rig_assert_eq 'm8 isolation: the bundle is unchanged in size' \
    "$(cat "${run}/h13-size.before")" "$(stat -c %s -- "${bundle}")"
  rig_assert 'm8 isolation: and git bundle verify still ACCEPTS it' \
    git bundle verify "${bundle}"
}

isolate_m9() { # the artifact is honest and present; only the threshold moved
  local run=$1
  rig_assert 'm9 isolation: the honest bundle is present and untouched' \
    test -f "$(c3_h13_bundle "${run}")"
  rig_assert "m9 isolation: and the cap was raised above it — ${C3_H13_CAP}" \
    test "${C3_H13_CAP}" -gt "$(cat "${run}/h13-size.before")"
}

isolate_m10() { # only the announcement went: leg 1's observations still hold
  local run=$1
  rig_assert_eq 'm10 isolation: the bell was still rung TWICE, one distinct line' \
    '2 1' "$(c3_h14_leg "${run}" rings) $(c3_h14_leg "${run}" rings-distinct)"
  rig_assert_eq 'm10 isolation: and the waiter still echoed the capsule it was asked about' \
    "${run}/capsule" "$(c3_h14_leg "${run}" echo-first)"
}

isolate_m11() { # only the polling went: the loss and the timeout both still hold
  local run=$1
  rig_assert_eq 'm11 isolation: the bell really was gone' \
    gone "$(c3_h14_leg "${run}" lost-bell)"
  rig_assert_eq 'm11 isolation: and the wait still reported the TIMEOUT — only sooner' \
    "${RIG_EXIT_TIMEOUT}" "$(c3_h14_leg "${run}" lost-status)"
  rig_assert_eq 'm11 isolation: it gave up in under a second, having polled nothing' \
    0 "$(c3_h14_leg "${run}" lost-elapsed)"
}

isolate_m12() { # only the forgery went: legs 1 and 2 read clean
  local run=$1
  # MEASURED, not assumed: leg 2 destroyed the bell and the no-op'd forgery
  # never rewrote it, so what leg 3 faced is an ABSENT bell rather than the
  # honest ring this control first claimed. The end state a no-op leaves is not
  # always the tidy one, and a control that asserts the tidy version reds for a
  # reason about itself.
  rig_assert 'm12 isolation: no ring at the bell at all — the forgery never happened' \
    c3_path_absent "$(c3_h14_bell "${run}")"
  rig_assert_eq 'm12 isolation: leg 1 still reads clean — two rings, one distinct' \
    '2 1' "$(c3_h14_leg "${run}" rings) $(c3_h14_leg "${run}" rings-distinct)"
  rig_assert_eq 'm12 isolation: and leg 2 still reads clean — gone, and timed out' \
    "gone ${RIG_EXIT_TIMEOUT}" \
    "$(c3_h14_leg "${run}" lost-bell) $(c3_h14_leg "${run}" lost-status)"
}

isolate_m13() { # only the loss went: the wait ran and found the bell present
  local run=$1
  rig_assert_eq 'm13 isolation: the bell was still there when leg 2 looked' \
    PRESENT "$(c3_h14_leg "${run}" lost-bell)"
  rig_assert_eq 'm13 isolation: so the wait SUCCEEDED rather than timing out' \
    0 "$(c3_h14_leg "${run}" lost-status)"
}

isolate_m14() { # only the second ring went: every other leg-1 clause still holds
  local run=$1
  rig_assert_eq 'm14 isolation: the waiter echoed the capsule on BOTH observations' \
    "${run}/capsule ${run}/capsule" \
    "$(c3_h14_leg "${run}" echo-first) $(c3_h14_leg "${run}" echo-second)"
  rig_assert_eq 'm14 isolation: and the published ref never moved — leg 1 reads clean' \
    "$(c3_h14_leg "${run}" published)" "$(c3_h14_leg "${run}" published-after)"
  rig_assert_eq 'm14 isolation: only the duplicate is missing — one ring, not two' \
    1 "$(c3_h14_leg "${run}" rings)"
}

case "${case_id}" in
  # One control per alternative: an empty `planted?` means nothing unless the
  # same call on the same leg returns non-empty without the perturbation.
  control-unsafe-path) expect_planted H13 light bundle bundle-unsafe-path live ;;
  control-absent) expect_planted H13 light bundle bundle-absent live ;;
  control-invalid) expect_planted H13 light bundle bundle-invalid live ;;
  control-cap) expect_planted H13 light bundle resource-cap live ;;

  m6)
    mutate_m6
    expect_planted H13 light bundle bundle-absent empty isolate_m6
    ;;
  m7)
    mutate_m7
    expect_planted H13 light bundle bundle-unsafe-path empty isolate_m7
    ;;
  m8)
    mutate_m8
    expect_planted H13 light bundle bundle-invalid empty isolate_m8
    ;;
  m9)
    mutate_m9
    expect_planted H13 light bundle resource-cap empty isolate_m9
    ;;

  # H14 — one alternative, so one control for all three legs.
  control-doorbell) expect_planted H14 light fetch harvest live ;;

  m10)
    mutate_m10
    expect_planted H14 light fetch harvest empty isolate_m10
    ;;
  m11)
    mutate_m11
    expect_planted H14 light fetch harvest empty isolate_m11
    ;;
  m12)
    mutate_m12
    expect_planted H14 light fetch harvest empty isolate_m12
    ;;
  m13)
    mutate_m13
    expect_planted H14 light fetch harvest empty isolate_m13
    ;;
  m14)
    mutate_m14
    expect_planted H14 light fetch harvest empty isolate_m14
    ;;
  *) rig_die "unknown case: ${case_id}" ;;
esac

rig_assert_done "falsify ${case_id}"
