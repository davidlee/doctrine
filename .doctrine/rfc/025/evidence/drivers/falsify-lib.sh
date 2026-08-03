#!/usr/bin/env bash
# Shared scaffolding for the falsifiability rounds — NOT a rig artefact.
#
# Sourced by `falsify-t4b.sh` and `falsify-t4c.sh`, which supply only their own
# mutations, isolation controls and case dispatch. One copy, because the two
# rounds make the SAME claim about different rows and a second copy of the
# vehicle is a second place for it to drift from the harness it stands in for.
#
# NOTE FOR CALLERS: set your shell options AFTER sourcing this file. Sourcing
# `pipeline.sh` re-enables `set -e`, which has silently killed throwaway
# drivers in this phase that turned it off beforehand.
#
# Callers must set `case_id` before invoking any function here; the labels read
# it at call time, so it may be assigned after the source.
RIG=/workspace/doctrine/scripts/spike-capsule
# Followed rather than stubbed to /dev/null, so shellcheck can see that the
# `C3_H*_PATHS` arrays this driver reads are really published by the library
# that defines the rows — the cross-file check is the point.
# shellcheck source=/workspace/doctrine/scripts/spike-capsule/control/pipeline.sh
. "${RIG}/control/pipeline.sh"
# shellcheck source=/workspace/doctrine/scripts/spike-capsule/lib/instantiations.sh
. "${RIG}/lib/instantiations.sh"

rig_enter

# The caller's case label. Declared here so the contract is visible rather than
# implicit; the caller assigns it, and every entry point below refuses an empty
# one instead of labelling its run dirs `falsify--…` and carrying on.
case_id=${case_id-}

# Rename <fn> to real_<fn> so a wrapper can call through to it — every
# mutation WRAPS the real function rather than restating it, so the driver
# cannot drift from the row it measures.
rebind() {
  eval "$(declare -f "$1" | sed "1s/^$1/real_$1/")"
}


# expect_planted <row> <fixture> <mechanism> <alt> <want:live|empty> [isolation]
#
# `isolation` names a function called with the run dir before teardown. It
# asserts the clauses of `Hnn_planted` the mutant did NOT target still HOLD, so
# an empty `planted?` is attributable to the one clause under test rather than
# to collateral damage. A mutant that reds by breaking everything proves
# nothing about the clause it was written for.
expect_planted() {
  local row=$1 fixture=$2 mechanism=$3 alt=$4 want=$5 isolation=${6:-}
  local slice stub run planted
  [ -n "${case_id}" ] || rig_die 'falsify-lib: caller must set case_id'

  case "${fixture}" in
    light) slice=001 stub=src/capsule-stub.ts ;;
    heavy) slice=241 stub=scripts/spike-capsule/capsule-stub.txt ;;
    *) rig_die "unknown fixture: ${fixture}" ;;
  esac

  pipeline_setup "falsify-${case_id}-${row}-${fixture}" \
    "${RIG_ROOT}/fixtures/${fixture}/repo" \
    "${RIG_ROOT}/fixtures/${fixture}/interpretation-surface.txt" \
    "${slice}" "${stub}"
  run="${PIPELINE_RUN}"
  # No verify bounds are set, and that is the design rather than an omission:
  # this driver never runs a pipeline leg. `planted?` is evaluated between
  # `pipeline_capsule` and the leg (probe-c3.sh:568-574), which is precisely
  # where the claim under test lives.

  pipeline_capsule "${run}"
  "${row}_mutate" "${run}" "${fixture}" "${mechanism}" "${alt}"

  planted=$("${row}_planted" "${run}" "${fixture}" "${mechanism}" "${alt}") || planted=''
  printf 'planted: %s\n' "${planted:-<EMPTY>}"

  case "${want}" in
    live) rig_assert "${case_id}: ${row}/${fixture} planted? is LIVE" test -n "${planted}" ;;
    empty) rig_assert "${case_id}: ${row}/${fixture} planted? REDS" test -z "${planted}" ;;
    *) rig_die "unknown expectation: ${want}" ;;
  esac

  [ -z "${isolation}" ] || "${isolation}" "${run}" "${fixture}"

  pipeline_teardown "${run}"
}

# ── the isolation controls, one per mutant ──────────────────────────────────

isolate_m1() { # only the x-bit moved: the hooksPath clause is untouched
  local run=$1 repo
  repo=$(c3_capsule_repo "${run}")
  rig_assert_eq 'm1 isolation: core.hooksPath is still the ABSOLUTE path' \
    "${repo}/.git/c3-h6-hooks" \
    "$(git -C "${repo}" config --get core.hooksPath)"
  rig_assert 'm1 isolation: and the hook file is present — only its x-bit went' \
    test -f "${repo}/.git/c3-h6-hooks/reference-transaction"
}

isolate_m2() { # only the config moved: the hooks are live and DID fire
  local run=$1 repo
  repo=$(c3_capsule_repo "${run}")
  rig_assert 'm2 isolation: the hooks are still EXECUTABLE' \
    test -x "${repo}/.git/c3-h6-hooks/reference-transaction"
  rig_assert 'm2 isolation: and one still FIRED — the payload was never deadened' \
    command grep -q "h6/reference-transaction ran in ${repo}" "$(c3_execution_log "${run}")"
}

isolate_m3() { # only the mode moved: the paths still landed in the range
  local run=$1 fixture=$2
  c3_h9_paths "${fixture}"
  rig_assert 'm3 isolation: all four H9 paths are still in the range' \
    c3_planted_paths "${run}" "${C3_H9_PATHS[@]}"
}

isolate_m4() { # only the kill moved: three attempts were still made
  local run=$1
  rig_assert_eq 'm4 isolation: three attempts were still made' \
    3 "$(wc -l <"${run}/h15-attempts")"
  rig_assert 'm4 isolation: and each is recorded COMPLETED, not killed' \
    test 3 -eq "$(command grep -c '=COMPLETED' "${run}/h15-attempts")"
}

# expect_refusal <row> <fixture> <mechanism> <alt> <want-refusal>
#
# The pipeline-level shape, for a mutant whose red is a REFUSAL rather than an
# empty `planted?`. `pipeline_run` is redirected, never piped and never
# substituted — a pipe reports the reader's status and a `$( … )` subshells the
# RIG DEFECT return out of reach (probe-c3.sh:521-524).
expect_refusal() {
  local row=$1 fixture=$2 mechanism=$3 alt=$4 want=$5
  local slice stub run planted observed rc=0
  [ -n "${case_id}" ] || rig_die 'falsify-lib: caller must set case_id'

  case "${fixture}" in
    light) slice=001 stub=src/capsule-stub.ts ;;
    heavy) slice=241 stub=scripts/spike-capsule/capsule-stub.txt ;;
    *) rig_die "unknown fixture: ${fixture}" ;;
  esac

  pipeline_setup "falsify-${case_id}-${row}-${fixture}" \
    "${RIG_ROOT}/fixtures/${fixture}/repo" \
    "${RIG_ROOT}/fixtures/${fixture}/interpretation-surface.txt" \
    "${slice}" "${stub}"
  run="${PIPELINE_RUN}"
  # Consumed by pipeline.sh as globals. Set even though conform is expected to
  # refuse first: if the mutant SURVIVES, verify runs, and a heavy leg under
  # light bounds would time out and confound the survivor with a bounds defect.
  # shellcheck disable=SC2034
  case "${fixture}" in
    heavy) PIPELINE_VERIFY_TIMEOUT=900 PIPELINE_VERIFY_DISK_CAP=$((8 * 1024 * 1024 * 1024)) ;;
  esac

  pipeline_capsule "${run}"
  "${row}_mutate" "${run}" "${fixture}" "${mechanism}" "${alt}"

  # The payload must LAND. Without this the refusal below could be a refusal of
  # nothing, which is the absence-shaped result this phase keeps meeting.
  planted=$("${row}_planted" "${run}" "${fixture}" "${mechanism}" "${alt}") || planted=''
  printf 'planted: %s\n' "${planted:-<EMPTY>}"
  rig_assert "${case_id}: the payload LANDED — the refusal is of something" \
    test -n "${planted}"

  pipeline_run "${run}" "${mechanism}" >"${run}/stages" || rc=$?
  [ "${rc}" -ne "${RIG_EXIT_DEFECT}" ] || rig_die 'RIG DEFECT from the pipeline — not a result'
  observed=$(pipeline_first_refusal "${run}/stages")
  printf 'observed: %s\n' "${observed:-<no refusal>}"

  rig_assert_eq "${case_id}: the pipeline REFUSES it — ${want}" \
    "${want}" "${observed}"

  pipeline_teardown "${run}"
}

# expect_assert <row> <fixture> <mechanism> <alt> [also]
#
# The third shape, for a row whose red lands in `Hnn_assert` rather than in
# `planted?` or in a refusal token. H11 is the first: its payload has to RUN
# before anything can be said, so the clause under test is only reachable after
# a full pipeline leg.
#
# `also` names a function called with <run> <observed> INSIDE THE CAPTURE, for a
# mutant whose red lands in `assert_outcome` rather than in the row at all —
# T4e's M24 and M25, where the dropped re-snapshot and stage 4's inverted
# ordering are both invisible to every clause H10/H16 make and visible only to
# the outcome assertion. Folded in here rather than given a fourth entry point
# of its own, which would be this function restated around one extra call.
# Empty by default, so the three rounds already scored are untouched.
#
# It runs the leg, then calls `_assert` with its verdicts CAPTURED and its
# failures ROLLED BACK out of this driver's own count — the row is supposed to
# red here, and a mutant that made the round itself red would report the
# expected result as a broken driver. Publishes `ASSERT_LOG`, `ASSERT_REDS` and
# `ASSERT_RUN`; the caller makes its own assertions against the log and calls
# `expect_assert_done` when finished with it.
#
# The isolation control for this shape is usually the RED COUNT rather than a
# separate function: "exactly one clause red, and it is this one" says in one
# line what an isolation control says in several, because every other clause of
# the row is standing in the same log.
expect_assert() {
  local row=$1 fixture=$2 mechanism=$3 alt=$4 also=${5:-}
  local slice stub run planted observed before rc=0
  [ -n "${case_id}" ] || rig_die 'falsify-lib: caller must set case_id'

  case "${fixture}" in
    light) slice=001 stub=src/capsule-stub.ts ;;
    heavy) slice=241 stub=scripts/spike-capsule/capsule-stub.txt ;;
    *) rig_die "unknown fixture: ${fixture}" ;;
  esac

  pipeline_setup "falsify-${case_id}-${row}-${fixture}" \
    "${RIG_ROOT}/fixtures/${fixture}/repo" \
    "${RIG_ROOT}/fixtures/${fixture}/interpretation-surface.txt" \
    "${slice}" "${stub}"
  run="${PIPELINE_RUN}"
  # shellcheck disable=SC2034
  case "${fixture}" in
    heavy) PIPELINE_VERIFY_TIMEOUT=900 PIPELINE_VERIFY_DISK_CAP=$((8 * 1024 * 1024 * 1024)) ;;
  esac

  pipeline_capsule "${run}"
  "${row}_mutate" "${run}" "${fixture}" "${mechanism}" "${alt}"

  planted=$("${row}_planted" "${run}" "${fixture}" "${mechanism}" "${alt}") || planted=''
  printf 'planted: %s\n' "${planted:-<EMPTY>}"

  pipeline_run "${run}" "${mechanism}" >"${run}/stages" || rc=$?
  [ "${rc}" -ne "${RIG_EXIT_DEFECT}" ] || rig_die 'RIG DEFECT from the pipeline — not a result'
  observed=$(pipeline_first_refusal "${run}/stages")
  printf 'observed: %s\n' "${observed:-<no refusal>}"

  before=${RIG_ASSERT_FAILURES}
  {
    "${row}_assert" "${run}" "${fixture}" "${mechanism}" "${alt}" "${observed}" || true
    [ -z "${also}" ] || "${also}" "${run}" "${observed}" || true
  } >"${run}/assert.log" 2>&1
  ASSERT_REDS=$((RIG_ASSERT_FAILURES - before))
  RIG_ASSERT_FAILURES=${before}
  ASSERT_LOG="${run}/assert.log"
  ASSERT_RUN="${run}"
  printf '%s reds from the row under mutation:\n' "${ASSERT_REDS}"
  sed 's/^/  | /' "${ASSERT_LOG}"
}

expect_assert_done() { pipeline_teardown "${ASSERT_RUN}"; }

# The two readers of a captured `_assert` log. Substring, not regex anchors: the
# clause text carries the cell label and an ellipsis, and a round that had to
# restate either would drift from the row it measures.
assert_red() { command grep -q "FAIL.*$1" "${ASSERT_LOG}"; }
assert_held() { command grep -q "ok .*$1" "${ASSERT_LOG}"; }
