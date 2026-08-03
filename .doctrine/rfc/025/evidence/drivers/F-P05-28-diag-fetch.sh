set -uo pipefail
RIG=/workspace/doctrine/scripts/spike-capsule
. "${RIG}/control/pipeline.sh"
. "${RIG}/lib/instantiations.sh"
set +e
rig_enter
pipeline_setup "diag-h15-heavy-fetch" \
  "${RIG_ROOT}/fixtures/heavy/repo" \
  "${RIG_ROOT}/fixtures/heavy/interpretation-surface.txt" \
  241 scripts/spike-capsule/capsule-stub.txt
run="${PIPELINE_RUN}"; q="${run}/quarantine"
PIPELINE_VERIFY_TIMEOUT=900
PIPELINE_VERIFY_DISK_CAP=$((8 * 1024 * 1024 * 1024))
pipeline_capsule "${run}"
printf 'c89b124a before kills: %s\n' "$(git -C "$q" cat-file -t c89b124a5b277d6bf182a44ad69d3efa723e53ba 2>&1)"
H15_mutate "${run}" heavy fetch ''
printf 'attempts: %s\n' "$(tr '\n' ' ' <"${run}/h15-attempts")"
printf 'c89b124a after kills:  %s\n' "$(git -C "$q" cat-file -t c89b124a5b277d6bf182a44ad69d3efa723e53ba 2>&1)"
git -C "$q" fsck --no-progress --connectivity-only >"${run}/f-fsck.out" 2>&1
printf 'fsck exit: %s   non-dangling: %s\n' "$?" "$(grep -cv '^dangling ' "${run}/f-fsck.out")"
grep -v '^dangling ' "${run}/f-fsck.out" | head -4
"${RIG}/control/harvest-fetch.sh" "${run}/capsule" "$q" >"${run}/f-oid" 2>"${run}/f-err"
printf 'harvest-fetch resume exit: %s  stdout: %s  stderr: %s\n' "$?" "$(cat "${run}/f-oid")" "$(cat "${run}/f-err")"
printf 'run dir: %s\n' "${run}"
