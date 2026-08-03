#!/usr/bin/env bash
# Falsifiability round for T4e — H10 and H16, the staleness rows.
# THROWAWAY. Never calls `rows_write`, so `results.tsv` is untouched.
#
#   usage: falsify-t4e.sh <case>      (one mutant per process, as T4b/T4c)
#
# Six mutants. The first four attack the rows' own clauses; the last two attack
# the machinery the rows are the SENTINEL for, and they are why this round
# matters more than its size suggests.
#
# WHAT MAKES THIS PAIR HARD TO FALSIFY, and why the set is shaped this way:
# every observable H10 and H16 make is an ABSENCE. Canonical unchanged, nothing
# merged, this result's version did not land — each of those holds just as well
# against a row that planted nothing, refused at stage 1, or never ran at all.
# So four of the six mutants exist to show the clauses are attached to
# something, and their isolation controls carry as much weight as the reds do.
#
# M25 IS THE ONE THIS ROUND EXISTS FOR. Design § 5.5's `assert_outcome` table
# gives `advance/stale-base` the STRICT clause — refs AND object count — while
# `cas-lost` gets refs only, and F-14 is why stage 4 must read its precondition
# BEFORE the transfer. T4e's task entry predicts that getting that ordering
# backwards "reds exactly here". M25 inverts it and measures whether that is so;
# its isolation control is the sharp half, because the REFS clause still holds
# under the inversion and a row carrying only that clause would score the defect
# green.
#
# Mutants WRAP the real function via `rebind` and never restate its body — the
# rule the earlier rounds set, and the reason M25 can invert stage 4's ordering
# without owning a copy of stage 4.
#
# One case per PROCESS, as T4b and T4c: it keeps each mutant's `rebind` out of
# the next one's shell, and it keeps every red inside the one
# `RIG_ASSERT_FAILURES` that `rig_assert_done` reads.

# shellcheck source=/workspace/doctrine/.doctrine/state/slice/241/drivers/falsify-lib.sh
. "$(dirname -- "${BASH_SOURCE[0]}")/falsify-lib.sh"
set -euo pipefail
case_id=${1:?usage: falsify-t4e.sh <case>}

# The fixture for the whole round. Light, and not as a shortcut: M24 and M25
# each need a FULL pipeline leg to reach `assert_outcome`, and on heavy that is
# a ~6-minute cargo build per mutant for a claim about stage 4's bookkeeping
# that has no fixture dimension at all.
FIXTURE=light

# Where a mutant sends a mover that is supposed to miss. Undeclared on both
# fixtures, as `C3_UNDECLARED_PATH` is, but deliberately not that constant: this
# path is written on CANONICAL and never enters a capsule, and a reader meeting
# H4's path on the trunk would go looking for a conform interaction that is not
# there.
M_ELSEWHERE='docs/m20-mover-elsewhere.md'

# ── the mutations ───────────────────────────────────────────────────────────

# M20 — H10's peer MISSES the contested path. The pair is then two commits from
# one base that do not conflict at all, which is H16's scenario wearing H10's
# name. `stale-base` is emitted either way, so only `planted?` can tell — which
# is why the conflict clause is a positive control and not a comment.
mutate_m20() {
  rebind c3_move_accepted
  c3_move_accepted() { real_c3_move_accepted "$1" "$2" "${M_ELSEWHERE}"; }
}

isolate_m20() { # only the mover's PATH moved: it still moved, still from B
  local run=$1
  rig_assert 'm20 isolation: the accepted ref still MOVED, and still to a child of B' \
    c3_stale_planted "${run}"
}

# M21 — the mover lands a GRANDCHILD of B. An intervening trunk commit, so the
# accepted ref has moved two commits rather than one. § 5.6 names H10 as a pair
# "from one base"; a mover further along is still staleness and still
# `stale-base`, and the row would have scored it green without the parentage
# clause.
mutate_m21() {
  rebind c3_move_accepted
  c3_move_accepted() {
    real_c3_move_accepted "$1" 'm21: an intervening trunk commit' "$3"
    real_c3_move_accepted "$@"
  }
}

isolate_m21() { # only the PARENTAGE moved: it moved, and it touched the path
  local run=$1 base moved
  base=$(c3_base "${run}")
  moved=$(c3_accepted_oid "${run}")
  rig_assert 'm21 isolation: the accepted ref still MOVED' \
    test "${moved}" != "${base}"
  rig_assert 'm21 isolation: and the mover still touches the contested path' \
    c3_lines_have "$(c3_range "${run}/canonical" "${base}" "${moved}")" \
    "$(contract_field "${run}" stub)"
}

# M22 — the accepted ref NEVER MOVES. The degenerate case, and the one every
# absence-shaped clause in both rows passes under: with nothing moved there is
# no staleness, stage 4 CASes cleanly, and "canonical was not changed by a
# refusal" is true because there was no refusal. `planted?` is the only thing
# standing between that and a green cell.
mutate_m22() {
  rebind c3_move_accepted
  c3_move_accepted() { :; }
}

isolate_m22() { # only the MOVE went: this capsule's own result still landed
  local run=$1
  rig_assert 'm22 isolation: the capsule result still carries the stub — only the trunk stood still' \
    c3_planted_paths "${run}" "$(contract_field "${run}" stub)"
}

# M23 — H16's mover TAKES the contested path, so H16 becomes H10. The mirror of
# M20, and the disjointness clause has to catch it: both rows refuse
# `advance/stale-base`, so a mover that collided with the stub would score H16
# green while instantiating H10 twice and leaving § 5.1's safety-versus-
# resolution contrast unmeasured.
mutate_m23() {
  rebind c3_move_accepted
  c3_move_accepted() { real_c3_move_accepted "$1" "$2" "$(contract_field "$1" stub)"; }
}

isolate_m23() { # only the mover's PATH moved: H16's other two clauses hold
  local run=$1 base moved
  base=$(c3_base "${run}")
  moved=$(c3_accepted_oid "${run}")
  rig_assert 'm23 isolation: the accepted ref still MOVED, and still to a child of B' \
    c3_stale_planted "${run}"
  rig_assert 'm23 isolation: and the mover still changed EXACTLY ONE path' \
    test 1 -eq "$(c3_range "${run}/canonical" "${base}" "${moved}" | wc -l)"
}

# M24 — the RE-SNAPSHOT is DROPPED. `c3_move_accepted`'s own comment calls the
# re-snapshot load-bearing; this is the measurement of that claim. Without it
# `assert_outcome` compares canonical against a `before` taken at B and reds on
# the row's OWN SETUP — a red that looks exactly like the assertion working,
# which is why it has to be produced deliberately once rather than met by
# accident later.
#
# Only the SECOND call is suppressed. Killing `pipeline_setup`'s snapshot too
# would leave no `before` state at all, and `assert_outcome` would error rather
# than red — a broken driver, not a falsification.
mutate_m24() {
  rebind pipeline_snapshot
  snapshot_calls=0
  pipeline_snapshot() {
    snapshot_calls=$((snapshot_calls + 1))
    [ "${snapshot_calls}" -gt 1 ] || real_pipeline_snapshot "$@"
  }
}

# M25 — stage 4's ordering INVERTED: transfer before precondition. The wrapper
# performs the transfer itself and then calls the real stage, so the real
# precondition runs with the objects already in canonical — the forbidden
# ordering, produced without restating a line of `advance_stage`.
mutate_m25() {
  rebind advance_stage
  advance_stage() {
    local canonical=$1 q=$2
    git -C "${canonical}" fetch --no-tags --quiet -- "${q}" "${RIG_QUARANTINE_REF}" 2>/dev/null || true
    real_advance_stage "$@"
  }
}

# M24 and M25 both red inside `assert_outcome`, which no shape reaches on its
# own. Passed as `expect_assert`'s fifth argument so the outcome's verdicts land
# in the SAME captured log as the row's — which is what lets the cases below say
# "the row held everywhere and only the outcome red" in one reading.
also_outcome() { assert_outcome "$1" "$2"; }

# ── the cases ───────────────────────────────────────────────────────────────

case "${case_id}" in
  m20)
    mutate_m20
    expect_planted H10 "${FIXTURE}" fetch '' empty isolate_m20
    ;;
  m21)
    mutate_m21
    expect_planted H10 "${FIXTURE}" fetch '' empty isolate_m21
    ;;
  m22)
    mutate_m22
    expect_planted H10 "${FIXTURE}" fetch '' empty isolate_m22
    ;;
  m23)
    mutate_m23
    expect_planted H16 "${FIXTURE}" fetch '' empty isolate_m23
    ;;
  m24)
    mutate_m24
    expect_assert H10 "${FIXTURE}" fetch '' also_outcome
    rig_assert 'M24: assert_outcome REDS on the un-re-snapshotted canonical' \
      assert_red 'assert_outcome'
    rig_assert 'M24 isolation: every clause of the ROW still held — the row cannot see this' \
      assert_held 'no merge was constructed'
    expect_assert_done
    ;;
  m25)
    mutate_m25
    expect_assert H10 "${FIXTURE}" fetch '' also_outcome
    rig_assert 'M25: the OBJECT-COUNT clause reds — objects landed before the precondition' \
      assert_red 'OBJECT COUNT'
    # THE FINDING, not bookkeeping: a design that had given `stale-base` the
    # refs-only clause `cas-lost` legitimately carries would score this defect
    # green, and these two rows are the only place in the matrix where it shows.
    rig_assert 'M25 isolation: the REFS clause still HOLDS — a refs-only assertion is blind to this' \
      assert_held 'canonical refs unchanged'
    rig_assert 'M25 isolation: and the row itself still held — this is invisible above the outcome' \
      assert_held 'no merge was constructed'
    rig_assert_eq 'M25 isolation: exactly one red, and it is the object count' \
      1 "${ASSERT_REDS}"
    expect_assert_done
    ;;
  *) rig_die "unknown case: ${case_id}" ;;
esac

rig_assert_done "falsify ${case_id}"
