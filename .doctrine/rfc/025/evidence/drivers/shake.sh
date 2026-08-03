#!/usr/bin/env bash
# Throwaway shakeout driver for one P-C3 cell — NOT a rig artefact.
# It never calls rows_write, so results.tsv is untouched (the mutant-append
# hazard the handover names).
#
#   usage: shake.sh <row> <fixture> <mechanism> [alt]
set -euo pipefail
RIG=/workspace/doctrine/scripts/spike-capsule
# shellcheck source=/dev/null
. "${RIG}/control/pipeline.sh"
# shellcheck source=/dev/null
. "${RIG}/lib/instantiations.sh"

rig_enter

row=$1
fixture=$2
mechanism=$3
alt=${4:-}

case "${fixture}" in
  light) slice=001 stub=src/capsule-stub.ts ;;
  heavy) slice=241 stub=scripts/spike-capsule/capsule-stub.txt ;;
  *) rig_die "unknown fixture: ${fixture}" ;;
esac

pipeline_setup "shake-${row}-${fixture}-${mechanism}-${alt}" \
  "${RIG_ROOT}/fixtures/${fixture}/repo" \
  "${RIG_ROOT}/fixtures/${fixture}/interpretation-surface.txt" \
  "${slice}" "${stub}"
run="${PIPELINE_RUN}"

case "${fixture}" in
  heavy) PIPELINE_VERIFY_TIMEOUT=900 PIPELINE_VERIFY_DISK_CAP=$((8 * 1024 * 1024 * 1024)) ;;
esac

pipeline_capsule "${run}"
"${row}_mutate" "${run}" "${fixture}" "${mechanism}" "${alt}"

planted=$("${row}_planted" "${run}" "${fixture}" "${mechanism}" "${alt}") || planted=''
rig_assert "planted? — this cell's own payload landed" test -n "${planted}"
printf 'planted: %s\n' "${planted:-<EMPTY>}"

rc=0
pipeline_run "${run}" "${mechanism}" >"${run}/stages" || rc=$?
cat "${run}/stages"
[ "${rc}" -ne "${RIG_EXIT_DEFECT}" ] || rig_die 'RIG DEFECT from the pipeline'

observed=$(pipeline_first_refusal "${run}/stages")
printf 'observed: %s\n' "${observed:-<no refusal>}"

"${row}_assert" "${run}" "${fixture}" "${mechanism}" "${alt}" "${observed}"
assert_outcome "${run}" "${observed}"

printf 'run dir (NOT torn down): %s\n' "${run}"
rig_assert_done "shake ${row}/${fixture}/${mechanism}/${alt}"
