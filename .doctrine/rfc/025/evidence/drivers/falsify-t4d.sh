#!/usr/bin/env bash
# T4d's falsifiability round — NOT a rig artefact, and it never calls
# rows_write, so results.tsv is untouched.
#
#   usage: falsify-t4d.sh <case>
#
# Same claim as T4b's and T4c's rounds: does the row's OWN `Hnn_planted` detect
# the perturbation that would hollow it out? H8's two alternatives are each a
# claim about ONE ARM of conform leg 4 — a mode and a name — and the arms are
# adjacent enough that each leg can be made to observe the other's refusal. That
# is what M15 and M17 are for. M16 is the third appearance of the trap
# F-P05-14/18/26 name, made deliberate.
#
# Scaffolding and `rebind` live in `falsify-lib.sh`, shared with the other two.
# shellcheck source=/workspace/doctrine/.doctrine/state/slice/241/drivers/falsify-lib.sh
. "$(dirname -- "${BASH_SOURCE[0]}")/falsify-lib.sh"
# AFTER the source — sourcing pipeline.sh re-enables set -e.
set -euo pipefail
case_id=${1:?usage: falsify-t4d.sh <case>}

# ── the mutations ───────────────────────────────────────────────────────────

# M15 — the gitlink arrives as an ORDINARY FILE at the same declared path. The
# path still lands in the range, so every clause but the mode's survives — and
# the mode is the entire subject of leg 4's first arm.
mutate_m15() {
  rebind H8_mutate
  H8_mutate() {
    local run=$1 fixture=$2 alt=$4
    if [ "${alt}" = gitlink ]; then
      c3_plant_file "${run}" "$(c3_h8_path "${fixture}" "${alt}")"
      c3_commit "${run}" 'M15: an ordinary file where the gitlink should be' \
        "$(c3_h8_path "${fixture}" "${alt}")"
      c3_publish "${run}"
      return 0
    fi
    real_H8_mutate "$@"
  }
}

# M16 — THE TRAP, MADE DELIBERATE. The `.gitmodules` is planted at a path
# neither fixture declares, so conform refuses `undeclared-path` at leg 2 and
# leg 4 is never reached. `planted?` stays LIVE throughout — the payload really
# did land — which is exactly why this mutant is scored at the BOUNDARY rather
# than at the positive control: the trap is invisible to `planted?` by
# construction, and only the observed token says the row measured the wrong leg.
C3_M16_PATH=docs/h8-undeclared/.gitmodules
mutate_m16() {
  rebind c3_h8_path
  c3_h8_path() {
    if [ "$2" = gitmodules ]; then
      printf '%s' "${C3_M16_PATH}"
      return 0
    fi
    real_c3_h8_path "$@"
  }
}

# M17 — the `.gitmodules` arrives as a GITLINK at its own declared path. Leg 4
# then refuses on the MODE arm while the leg claims to observe the NAME arm, so
# a row without the `100644` clause would report this guard firing on evidence
# belonging to its sibling. The adjacent-observable failure, one arm over.
mutate_m17() {
  rebind H8_mutate
  H8_mutate() {
    local run=$1 fixture=$2 alt=$4 repo path
    if [ "${alt}" = gitmodules ]; then
      repo=$(c3_capsule_repo "${run}")
      path=$(c3_h8_path "${fixture}" "${alt}")
      git -C "${repo}" update-index --add \
        --cacheinfo "160000,$(c3_base "${run}"),${path}"
      git -C "${repo}" commit --quiet -m 'M17: a gitlink wearing the .gitmodules name'
      c3_publish "${run}"
      return 0
    fi
    real_H8_mutate "$@"
  }
}

# ── isolation controls ──────────────────────────────────────────────────────

isolate_m15() { # only the mode moved: the path is in the range, as a real file
  local run=$1 fixture=$2 path
  path=$(c3_h8_path "${fixture}" gitlink)
  rig_assert "m15 isolation: the path still lands in the range leg 2 folds" \
    c3_planted_paths "${run}" "${path}"
  rig_assert_eq 'm15 isolation: and it is an ORDINARY FILE — only the mode went' \
    100644 \
    "$(git -C "$(c3_capsule_repo "${run}")" ls-tree "$(c3_result "${run}")" -- "${path}" |
      awk '{print $1}')"
}

isolate_m17() { # only the mode moved: the NAME is still .gitmodules
  local run=$1 fixture=$2 path
  path=$(c3_h8_path "${fixture}" gitmodules)
  rig_assert_eq 'm17 isolation: the planted name is still .gitmodules' \
    "${C3_H8_GITMODULES}" "$(basename -- "${path}")"
  rig_assert_eq 'm17 isolation: and the entry really is a gitlink — the sibling arm would fire' \
    160000 \
    "$(git -C "$(c3_capsule_repo "${run}")" ls-tree "$(c3_result "${run}")" -- "${path}" |
      awk '{print $1}')"
}

# ── H11's LIGHT mutants: M18–M22 ────────────────────────────────────────────
#
# H11 is the first row whose payload has to EXECUTE, so its reds land in three
# different places and the round has to reach all three: two at `planted?`
# (the canary's arming), two inside `_assert` (the containment clauses), and one
# — the guard that refuses a plant nothing will run — at the plant itself, where
# it `rig_die`s rather than scoring. Heavy's two are M23–M24, below: since
# D-P05-15 that fixture scores at `conform`, so its clauses are different and it
# gets its own control.
#
# M22 is the row's vacuity mutant and the one worth reading twice: with an inert
# payload the CONTAINMENT CLAUSE STILL PASSES. The sentinel is absent because
# nothing ever tried to write it, which is the shape this phase has met at
# H13/M6, at H14/M14 and at H8/M16, and it is why the execution evidence is
# asserted before anything else in the row.

# M18 — the canary is never armed. Every other clause of `planted?` holds: the
# payload is planted, reachable, and in the range. Only the observation the row
# would later take is missing, and a row without this clause would report
# "unreached" about a canary that was never listening.
mutate_m18() { c3_h11_canary_start() { :; }; }

# M19 — the canary is armed and ALREADY DIRTY, as a leaked listener from an
# earlier run would leave it. The mark is there before the payload runs, so an
# arrival counted afterwards belongs to someone else. `planted?`'s quiet clause
# is the only thing between that and a measurement attributed to this cell.
mutate_m19() {
  rebind c3_h11_canary_start
  c3_h11_canary_start() {
    real_c3_h11_canary_start "$@"
    printf '%s\n' "${C3_H11_CANARY_MARK}" >>"$(c3_h11_canary_log "$1")"
  }
}

# M20 — THE ESCAPE IS NEVER ATTEMPTED. The payload still runs, still reports its
# cwd, still pings the canary; only the three lines that try the host write and
# record the outcome are cut. The sentinel is then absent for the wrong reason,
# and the `escape=denied` clause is the row's only defence against reporting
# containment of an attack that never happened.
mutate_m20() {
  rebind c3_h11_write_payload
  c3_h11_write_payload() {
    real_c3_h11_write_payload "$@"
    sed -i -e '/fs.writeFileSync(ESCAPE/d' -e '/escape=WROTE/d' -e '/escape=denied/d' -- "$1"
  }
}

# M21 — CONTAINMENT FAILS OPEN, simulated. The sentinel is placed on the host
# path by the rig, so the row faces exactly what it would face if the profile
# had let the payload through. It does not prove the profile can be broken; it
# proves the clause that would notice is not a tautology — the same thing M6
# proves for H13's absent bundle, and the reason both are written.
mutate_m21() {
  rebind H11_mutate
  H11_mutate() {
    real_H11_mutate "$@"
    : >"$(c3_h11_escape_target "$1")"
  }
}

# M22 — THE PAYLOAD IS INERT. An empty file at the same declared path: still
# committed, still in the range, still matched by `node --test src/*.test.ts`,
# and it does nothing. Four clauses red — and the isolation control is that the
# CONTAINMENT CLAUSE PASSES ANYWAY.
mutate_m22() {
  rebind c3_h11_write_payload
  c3_h11_write_payload() { : >"$1"; }
}

# ── H11's HEAVY mutants: M23–M24 (D-P05-15) ────────────────────────────────
#
# Heavy refuses at `conform/undeclared-path` rather than at the sandbox, so its
# clauses are all about the plant being a LIVE trigger that the selector gate
# stopped — not an inert file at a path nobody declared. Both mutants attack
# that liveness, from its two independent halves.

# M23 — THE PLANT MOVES TO THE PATH CONFORM WOULD ADMIT. D-P05-11's trap,
# deliberate, and this is the row where taking it is the damage: a `build.rs`
# under `scripts/spike-capsule/**` is admitted, is never run by cargo, and would
# score a refusal of nothing. The row must refuse to plant it at all.
mutate_m23() { c3_h11_path() { printf 'scripts/spike-capsule/build.rs'; }; }

# M24 — THE PACKAGE ROOT IS GONE. The payload still lands, still at the root,
# still in the range — but with no `Cargo.toml` beside it there is no crate for
# cargo to build and the build script is a text file with a suggestive name.
# Wraps the real writer rather than restating it, so the mutant cannot drift
# from the row it measures.
mutate_m24() {
  rebind c3_h11_write_payload_rs
  c3_h11_write_payload_rs() {
    real_c3_h11_write_payload_rs "$@"
    rm -f -- "$(dirname -- "$1")/Cargo.toml"
  }
}

# M24's isolation: the payload really did land, at the root, in the range. Only
# the crate around it is missing.
isolate_m24() {
  local run=$1 fixture=$2 path
  path=$(c3_h11_path "${fixture}")
  rig_assert 'm24 isolation: build.rs IS still planted, at the root' \
    test -f "$(c3_capsule_repo "${run}")/${path}"
  rig_assert 'm24 isolation: and still in the range the pipeline folds' \
    c3_planted_paths "${run}" "${path}"
}

# ── H11's isolation controls ────────────────────────────────────────────────

isolate_m18() { # only the arming went: the payload is planted and runnable
  local run=$1 fixture=$2 path
  path=$(c3_h11_path "${fixture}")
  rig_assert 'm18 isolation: the payload is still in the range the pipeline folds' \
    c3_planted_paths "${run}" "${path}"
  rig_assert 'm18 isolation: and it is still a path the verify command runs' \
    c3_h11_reachable "${fixture}" "${path}"
}

isolate_m19() { # only the quiet went: the canary really is listening
  local run=$1 fixture=$2
  rig_assert 'm19 isolation: the canary IS armed and listening — only its log is dirty' \
    kill -0 "${C3_H11_CANARY_PID}"
  rig_assert 'm19 isolation: and the payload is planted, as in the control' \
    c3_planted_paths "${run}" "$(c3_h11_path "${fixture}")"
  c3_h11_canary_stop
}

# `expect_planted` stops at the plant and never reaches `_assert`, so nothing
# else reaps the listener a case leaves running. Borrowing the isolation slot is
# the honest spelling of "this control has cleanup but nothing to isolate" —
# M18 needs none, having no-op'd the arming in the first place.
isolate_h11_control() { c3_h11_canary_stop; }

# `rig_assert_fails` runs its command in THIS shell, and the refusal under test
# is a `rig_die` — an `exit`. Without the subshell the assertion would kill the
# driver it is being made in, and the round would report nothing at all.
h11_mutate_subshell() { (H11_mutate "$@"); }

case "${case_id}" in
  # One control per alternative: an empty `planted?` means nothing unless the
  # same call on the same leg returns non-empty without the perturbation.
  control-gitlink) expect_planted H8 light fetch gitlink live ;;
  control-gitmodules) expect_planted H8 light fetch gitmodules live ;;

  m15)
    mutate_m15
    expect_planted H8 light fetch gitlink empty isolate_m15
    ;;
  m16)
    mutate_m16
    expect_refusal H8 light fetch gitmodules conform/undeclared-path
    ;;
  m17)
    mutate_m17
    expect_planted H8 light fetch gitmodules empty isolate_m17
    ;;

  # ── H11 ───────────────────────────────────────────────────────────────────
  control-h11) expect_planted H11 light fetch verify live isolate_h11_control ;;

  # Heavy's control. Its `planted?` has different clauses from light's — no
  # canary, a package root instead — so it needs its own unperturbed reading or
  # M24's empty proves nothing.
  control-heavy) expect_planted H11 heavy fetch undeclared-path live ;;

  # M23 is the GUARD, and it reds by refusing rather than by scoring: with the
  # plant moved to the path conform would admit, `_mutate` must `rig_die` rather
  # than plant a file cargo will never open. Asserted in a subshell because the
  # refusal is a `rig_die` — an `exit` — and without one it would kill the
  # driver making the assertion. The run dir is deliberately NONEXISTENT: the
  # guard is the first thing after two pure path computations, so a run that got
  # as far as provisioning would prove the guard fires late.
  m23)
    mutate_m23
    rig_assert_fails 'm23: H11 REFUSES a heavy plant at the admitted path — cargo never runs it' \
      h11_mutate_subshell /nonexistent heavy fetch undeclared-path
    ;;
  m24)
    mutate_m24
    expect_planted H11 heavy fetch undeclared-path empty isolate_m24
    ;;

  m18)
    mutate_m18
    expect_planted H11 light fetch verify empty isolate_m18
    ;;
  m19)
    mutate_m19
    expect_planted H11 light fetch verify empty isolate_m19
    ;;

  # The three that only `_assert` can see. `ASSERT_REDS` IS the isolation
  # control for the first two: every other clause of the row stood in the same
  # log and held.
  m20)
    mutate_m20
    expect_assert H11 light fetch verify
    rig_assert 'm20: the ATTEMPT clause reds — the escape was never tried' \
      assert_red 'the payload TRIED'
    rig_assert 'm20: and containment still reads clean — the vacuity M20 exists to show' \
      assert_held 'the escape sentinel is ABSENT'
    rig_assert_eq 'm20 isolation: exactly one clause red, and it is that one' 1 "${ASSERT_REDS}"
    expect_assert_done
    ;;
  m21)
    mutate_m21
    expect_assert H11 light fetch verify
    rig_assert 'm21: the CONTAINMENT clause reds — the sentinel is on the host path' \
      assert_red 'the escape sentinel is ABSENT'
    rig_assert 'm21: and the payload still reports being denied — only the observable moved' \
      assert_held 'the payload TRIED'
    rig_assert_eq 'm21 isolation: exactly one clause red, and it is that one' 1 "${ASSERT_REDS}"
    expect_assert_done
    ;;
  m22)
    mutate_m22
    expect_assert H11 light fetch verify
    rig_assert 'm22: the EXECUTION clause reds — nothing ran' \
      assert_red 'the payload EXECUTED'
    rig_assert 'm22: … and so does the cwd clause that would say WHERE it ran' \
      assert_red "ran inside the capsule's own clone"
    rig_assert 'm22 isolation: THE CONTAINMENT CLAUSE PASSES ANYWAY — the vacuity, exactly' \
      assert_held 'the escape sentinel is ABSENT'
    rig_assert 'm22 isolation: and the transport clauses are untouched — the payload DID travel' \
      assert_held 'is in the range the trusted side folded'
    expect_assert_done
    ;;
  *) rig_die "unknown case: ${case_id}" ;;
esac

rig_assert_done "falsify ${case_id}"
