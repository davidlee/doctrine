#!/usr/bin/env bash
# Throwaway: what does a SIGKILLed M-B harvest leave in the quarantine?
# Harvest only — no verify — so it costs ~1 min on heavy, not ~7.
set -euo pipefail
RIG=/workspace/doctrine/scripts/spike-capsule
# shellcheck source=/dev/null
. "${RIG}/control/pipeline.sh"
# shellcheck source=/dev/null
. "${RIG}/lib/instantiations.sh"

rig_enter
fixture=${1:-heavy}
mechanism=${2:-bundle}

case "${fixture}" in
  light) slice=001 stub=src/capsule-stub.ts ;;
  heavy) slice=241 stub=scripts/spike-capsule/capsule-stub.txt ;;
esac

pipeline_setup "repro-h15-${fixture}-${mechanism}" \
  "${RIG_ROOT}/fixtures/${fixture}/repo" \
  "${RIG_ROOT}/fixtures/${fixture}/interpretation-surface.txt" "${slice}" "${stub}"
run="${PIPELINE_RUN}"
pipeline_capsule "${run}"

printf '\n== bundle size ==\n'
ls -l "${run}/capsule/${RIG_BUNDLE}" 2>/dev/null || echo '  (no bundle)'

printf '\n== quarantine objects BEFORE any harvest ==\n'
find "${run}/quarantine/.git/objects" -type f | wc -l

printf '\n== attempt 1: killed during harvest ==\n'
if c3_h15_kill_attempt "${run}" "${mechanism}" start "${run}/repro-stages"; then
  echo '  killed'
else
  echo '  COMPLETED (kill missed the window)'
fi

printf '\n== quarantine AFTER the killed harvest ==\n'
find "${run}/quarantine/.git/objects" -type f | sed "s|${run}/quarantine/.git/objects/||" | head -20
printf '  total files: %s\n' "$(find "${run}/quarantine/.git/objects" -type f | wc -l)"
printf '  tmp/partial: %s\n' "$(find "${run}/quarantine/.git/objects" -name 'tmp_*' -o -name '*.pack' -o -name '*.idx' | wc -l)"

printf '\n== git fsck on the quarantine, standalone ==\n'
git -C "${run}/quarantine" fsck --no-progress --connectivity-only 2>&1 | head -10 || true
printf '  fsck exit: %s\n' "$?"

printf '\n== resume: harvest-bundle again, same quarantine ==\n'
set +e
"${RIG}/control/harvest-bundle.sh" "${run}/capsule" "${run}/quarantine" >"${run}/repro-oid" 2>"${run}/repro-err"
printf '  exit: %s\n' "$?"
set -e
printf '  stdout: %s\n' "$(cat "${run}/repro-oid")"
printf '  stderr: %s\n' "$(cat "${run}/repro-err")"

printf '\n== control: a FRESH quarantine, same capsule ==\n'
rm -rf -- "${run}/quarantine-fresh"
git clone --no-hardlinks --quiet -- "${run}/canonical" "${run}/quarantine-fresh"
set +e
"${RIG}/control/harvest-bundle.sh" "${run}/capsule" "${run}/quarantine-fresh" >"${run}/repro-oid2" 2>"${run}/repro-err2"
printf '  exit: %s\n' "$?"
set -e
printf '  stdout: %s\n' "$(cat "${run}/repro-oid2")"
printf '  stderr: %s\n' "$(cat "${run}/repro-err2")"

printf '\nrun dir: %s\n' "${run}"
