#!/usr/bin/env bash
# Resume the F-P05-28 measurement against the ALREADY-KILLED quarantine.
# The kill landed; only the driver's own head/SIGPIPE aborted the run.
set -uo pipefail
RIG=/workspace/doctrine/scripts/spike-capsule
run=/home/david/capsules/runs/repro-h15-heavy-bundle

printf '== quarantine object counts AFTER the killed harvest ==\n'
printf '  loose+pack files: %s\n' "$(find "${run}/quarantine/.git/objects" -type f | wc -l)"
printf '  tmp_*:            %s\n' "$(find "${run}/quarantine/.git/objects" -name 'tmp_*' | wc -l)"
printf '  *.pack:           %s\n' "$(find "${run}/quarantine/.git/objects" -name '*.pack' | wc -l)"
printf '  *.idx:            %s\n' "$(find "${run}/quarantine/.git/objects" -name '*.idx' | wc -l)"
printf '  incoming-*:       %s\n' "$(find "${run}/quarantine/.git/objects" -maxdepth 1 -name 'incoming-*' | wc -l)"
printf '\n  pack dir listing:\n'
ls -l "${run}/quarantine/.git/objects/pack" 2>&1 | sed 's/^/    /'
printf '\n  FETCH_HEAD / refs state:\n'
ls -l "${run}/quarantine/.git/FETCH_HEAD" 2>&1 | sed 's/^/    /'

printf '\n== git fsck on the quarantine, standalone ==\n'
git -C "${run}/quarantine" fsck --no-progress --connectivity-only >"${run}/fsck.out" 2>&1
printf '  fsck exit: %s\n' "$?"
sed 's/^/    /' "${run}/fsck.out"

printf '\n== resume: harvest-bundle again, SAME quarantine (in place) ==\n'
"${RIG}/control/harvest-bundle.sh" "${run}/capsule" "${run}/quarantine" >"${run}/repro-oid" 2>"${run}/repro-err"
printf '  exit: %s\n' "$?"
printf '  stdout: %s\n' "$(cat "${run}/repro-oid")"
printf '  stderr:\n'
sed 's/^/    /' "${run}/repro-err"

printf '\n== control: a FRESH quarantine, same capsule ==\n'
rm -rf -- "${run}/quarantine-fresh"
git clone --no-hardlinks --quiet -- "${run}/canonical" "${run}/quarantine-fresh"
"${RIG}/control/harvest-bundle.sh" "${run}/capsule" "${run}/quarantine-fresh" >"${run}/repro-oid2" 2>"${run}/repro-err2"
printf '  exit: %s\n' "$?"
printf '  stdout: %s\n' "$(cat "${run}/repro-oid2")"
printf '  stderr:\n'
sed 's/^/    /' "${run}/repro-err2"

printf '\nrun dir: %s\n' "${run}"
