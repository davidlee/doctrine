#!/usr/bin/env bash
# F-P05-28 diagnosis: run H15's REAL mutate (three kills), then decompose the
# harvest refusal instead of just observing it.
#
# No internal truncating pipes (that is what killed the previous driver), and
# no `set -e` around the measured commands — a refusal IS the datum here.
set -uo pipefail
RIG=/workspace/doctrine/scripts/spike-capsule
. "${RIG}/control/pipeline.sh"
. "${RIG}/lib/instantiations.sh"

rig_enter
fixture=heavy
mechanism=bundle

pipeline_setup "diag-h15-${fixture}-${mechanism}" \
  "${RIG_ROOT}/fixtures/${fixture}/repo" \
  "${RIG_ROOT}/fixtures/${fixture}/interpretation-surface.txt" \
  241 scripts/spike-capsule/capsule-stub.txt
run="${PIPELINE_RUN}"
PIPELINE_VERIFY_TIMEOUT=900
PIPELINE_VERIFY_DISK_CAP=$((8 * 1024 * 1024 * 1024))
pipeline_capsule "${run}"

q="${run}/quarantine"

snapshot() {
  local at=$1
  printf '\n-- snapshot: %s --\n' "${at}"
  printf '   object files:  %s\n' "$(find "${q}/.git/objects" -type f | wc -l)"
  printf '   *.lock:        %s  %s\n' \
    "$(find "${q}/.git" -name '*.lock' | wc -l)" \
    "$(find "${q}/.git" -name '*.lock' | tr '\n' ' ')"
  printf '   tmp_*/incoming: %s %s\n' \
    "$(find "${q}/.git/objects" \( -name 'tmp_*' -o -name 'incoming-*' \) | wc -l)" \
    "$(find "${q}/.git/objects" \( -name 'tmp_*' -o -name 'incoming-*' \) | tr '\n' ' ')"
  printf '   commit-graph:  %s\n' "$(ls "${q}/.git/objects/info/" 2>/dev/null | tr '\n' ' ')"
  printf '   midx:          %s\n' "$(ls "${q}/.git/objects/pack/" 2>/dev/null | grep -c 'multi-pack-index')"
  printf '   refs:          %s\n' "$(git -C "${q}" for-each-ref --format='%(refname)=%(objectname)' | tr '\n' ' ')"
  printf '   FETCH_HEAD:    %s\n' "$([ -e "${q}/.git/FETCH_HEAD" ] && echo present || echo absent)"
}

snapshot 'before any attempt'

printf '\n== H15_mutate: the three kills ==\n'
H15_mutate "${run}" "${fixture}" "${mechanism}" ''
printf 'attempts:\n'
sed 's/^/   /' "${run}/h15-attempts"
snapshot 'after all three kills'

printf '\n== standalone fsck (connectivity-only), exactly as the harvester runs it ==\n'
git -C "${q}" fsck --no-progress --connectivity-only >"${run}/d-fsck.out" 2>&1
printf '   exit: %s\n' "$?"
printf '   non-dangling lines: %s\n' "$(grep -cv '^dangling ' "${run}/d-fsck.out")"
grep -v '^dangling ' "${run}/d-fsck.out" | head -20 | sed 's/^/     /'

printf '\n== the resume: harvest-bundle.sh in place ==\n'
"${RIG}/control/harvest-bundle.sh" "${run}/capsule" "${q}" >"${run}/d-oid" 2>"${run}/d-err"
printf '   exit:   %s\n' "$?"
printf '   stdout: %s\n' "$(cat "${run}/d-oid")"
printf '   stderr: %s\n' "$(cat "${run}/d-err")"

printf '\n== DECOMPOSITION — the harvester swallows git stderr; recover it ==\n'
bundle="${run}/capsule/${RIG_BUNDLE}"
printf '   [1] bundle verify:\n'
git -C "${q}" bundle verify -- "${bundle}" >"${run}/d-bv.out" 2>&1
printf '       exit: %s\n' "$?"
tail -5 "${run}/d-bv.out" | sed 's/^/       /'
printf '   [2] the fetch, WITH stderr (this is the line that reports fsck-failed):\n'
git -C "${q}" config fetch.fsckObjects true
git -C "${q}" fetch --no-tags -- "${bundle}" \
  "+${RIG_RESULT_REF}:${RIG_QUARANTINE_REF}" >"${run}/d-fetch.out" 2>&1
printf '       exit: %s\n' "$?"
sed 's/^/       /' "${run}/d-fetch.out"
printf '   [3] post-fetch fsck:\n'
git -C "${q}" fsck --no-progress --connectivity-only >"${run}/d-fsck2.out" 2>&1
printf '       exit: %s\n' "$?"
printf '       non-dangling: %s\n' "$(grep -cv '^dangling ' "${run}/d-fsck2.out")"
grep -v '^dangling ' "${run}/d-fsck2.out" | head -20 | sed 's/^/         /'

printf '\n== control: FRESH quarantine, same capsule ==\n'
rm -rf -- "${run}/quarantine-fresh"
git clone --no-hardlinks --quiet -- "${run}/canonical" "${run}/quarantine-fresh"
"${RIG}/control/harvest-bundle.sh" "${run}/capsule" "${run}/quarantine-fresh" >"${run}/d-oid2" 2>"${run}/d-err2"
printf '   exit:   %s\n' "$?"
printf '   stdout: %s\n' "$(cat "${run}/d-oid2")"
printf '   stderr: %s\n' "$(cat "${run}/d-err2")"

printf '\nrun dir: %s\n' "${run}"
