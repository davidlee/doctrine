#!/usr/bin/env bash
# T4b's falsifiability round — NOT a rig artefact, and it never calls
# rows_write, so results.tsv is untouched (the mutant-append hazard).
#
#   usage: falsify-t4b.sh <case>
#
# THE CLAIM UNDER TEST is narrow and deliberately so: does each row's OWN
# `Hnn_planted` DETECT the perturbation that would hollow it out? The other
# half of the composition — that an empty `planted?` reds the cell — is
# already proven every run by the harness's own positive control
# (`probe-c3.sh:301`, 'a forced-false planted? REDS the cell'). So the two
# halves are established at both ends and nothing here needs a verify leg.
#
# Scaffolding, the mutation wrappers and the isolation controls live in
# `falsify-lib.sh`, shared with `falsify-t4c.sh`.
# shellcheck source=/workspace/doctrine/.doctrine/state/slice/241/drivers/falsify-lib.sh
. "$(dirname -- "${BASH_SOURCE[0]}")/falsify-lib.sh"
# AFTER the source — sourcing pipeline.sh re-enables set -e.
set -euo pipefail
case_id=${1:?usage: falsify-t4b.sh <case>}

# ── the mutations, each a wrapper over the real function (`rebind`, in the lib)

mutate_m1() { # H6's hooks are written NON-EXECUTABLE
  rebind c3_h6_write_hooks
  c3_h6_write_hooks() {
    real_c3_h6_write_hooks "$@"
    chmod -x -- "$2"/*
  }
}

mutate_m2() { # H6's core.hooksPath is made RELATIVE — the vacuity trap
  rebind H6_mutate
  H6_mutate() {
    real_H6_mutate "$@"
    git -C "$(c3_capsule_repo "$1")" config core.hooksPath .git/c3-h6-hooks
  }
}

mutate_m3() { # H9's escape symlink is DEREFERENCED to a regular file
  rebind H9_mutate
  H9_mutate() {
    local run=$1 fixture=$2 repo dir path target
    real_H9_mutate "$@"
    repo=$(c3_capsule_repo "${run}")
    dir=$(c3_design_target_dir "${fixture}")
    path="${repo}/${dir}/h9-escape-abs"
    target=$(readlink -- "${path}")
    rm -f -- "${path}"
    printf '%s\n' "${target}" >"${path}"
    c3_commit "${run}" 'mutant: dereference the escape symlink' "${dir}/h9-escape-abs"
    c3_publish "${run}"
  }
}

mutate_m4() { # H15's kill is a NO-OP, so every attempt runs to completion
  kill() { return 1; }
}

# M5 is the odd one out: it does not target a `planted?` clause at all. It
# restores the matrix's ORIGINAL root-level placement that D-P05-11 moved, and
# the claim is that the pipeline itself refuses it — `conform/undeclared-path`,
# which falsifies a dissolution. So the payload must still LAND (planted? stays
# live) and the red must come from conform.
#
# ONLY THE FLAKE. Root `.envrc` is unreachable for a SECOND and independent
# reason — it is gitignored in this repository, so `git add` refuses it before
# conform is ever reached (F-P05-26). Committing it here would kill the driver
# instead of producing a refusal, so that half is measured separately by `m5b`.
mutate_m5() {
  rebind c3_design_target_dir
  c3_design_target_dir() { printf '.'; }
  rebind c3_h12_paths
  c3_h12_paths() {
    real_c3_h12_paths "$@"
    # BARE, not `./flake.nix`. `c3_planted_paths` matches `grep -qxF` against
    # git's own changed-path list, which reports `flake.nix` — a `./` prefix
    # reds the landing guard and would have let the refusal below look like a
    # refusal of nothing.
    C3_H12_PATHS=("${C3_H12_FLAKE}")
  }
}

# ── one case: provision, mutate, ask the row's own planted? ─────────────────
case "${case_id}" in
  control-h6) expect_planted H6 light fetch dissolution live ;;
  m1)
    mutate_m1
    expect_planted H6 light fetch dissolution empty isolate_m1
    ;;
  m2)
    mutate_m2
    expect_planted H6 light fetch dissolution empty isolate_m2
    ;;
  control-h9) expect_planted H9 light fetch dissolution live ;;
  m3)
    mutate_m3
    expect_planted H9 light fetch dissolution empty isolate_m3
    ;;
  control-h15) expect_planted H15 light fetch dissolution live ;;
  m4)
    mutate_m4
    expect_planted H15 light fetch dissolution empty isolate_m4
    ;;
  m5)
    mutate_m5
    expect_refusal H12 heavy fetch dissolution conform/undeclared-path
    ;;
  m5b)
    # F-P05-26's SECOND and independent reason, measured rather than cited: at
    # the repository root `.envrc` is gitignored here, so the plant never
    # reaches conform — `git add` refuses it first. Cheap by construction; it
    # needs no pipeline leg because the refusal is git's, not the rig's.
    pipeline_setup "falsify-${case_id}-H12-heavy" \
      "${RIG_ROOT}/fixtures/heavy/repo" \
      "${RIG_ROOT}/fixtures/heavy/interpretation-surface.txt" \
      241 scripts/spike-capsule/capsule-stub.txt
    pipeline_capsule "${PIPELINE_RUN}"
    m5b_repo=$(c3_capsule_repo "${PIPELINE_RUN}")
    printf 'payload\n' >"${m5b_repo}/.envrc"
    rig_assert 'm5b: the root .envrc really is present in the worktree' \
      test -f "${m5b_repo}/.envrc"
    rig_assert_fails 'm5b: git add REFUSES the gitignored root .envrc' \
      git -C "${m5b_repo}" add -- .envrc
    pipeline_teardown "${PIPELINE_RUN}"
    ;;
  *) rig_die "unknown case: ${case_id}" ;;
esac

rig_assert_done "falsify ${case_id}"
