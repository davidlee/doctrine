#!/usr/bin/env bash
# lib/instantiations.sh — the P-C3 rows' INSTANTIATIONS (SL-241 PHASE-05 T4).
# SOURCED by control/probe-c3.sh, never executed.
#
# One trio per § 5.6 row, named mechanically off the `row` column so the harness
# needs no translation table:
#
#   H<n>_mutate  <run> <fixture> <mechanism> <alt>
#   H<n>_planted <run> <fixture> <mechanism> <alt>            → PRINTS the observable
#   H<n>_assert  <run> <fixture> <mechanism> <alt> <observed>
#
# ── the seam these run in (A-2) ─────────────────────────────────────────────
#
# `_mutate` runs BETWEEN `pipeline_capsule` and `pipeline_run`, which is the
# only reason those are two calls (`pipeline.sh:196-200`). The worker has
# already committed, published `RIG_RESULT_REF` and rung; the rig now plays the
# adversary against the capsule's own clone, trusted-side. That is a rig
# convenience, not a claim about who could do it: everything mutated here is
# inside the capsule's writable root, so a hostile worker could author all of it.
#
# ── what these files may and may not touch ──────────────────────────────────
#
# The capsule clone, and nothing else. Not `canonical` (that is the subject of
# `assert_outcome`), not `quarantine` (the trusted side's own workspace). A row
# that reached into either would be testing the rig's bookkeeping.
#
# ── scoring is not here ─────────────────────────────────────────────────────
#
# `_assert` observes what the refusal TOKEN cannot say — that the payload was
# ingested before it was refused, that the naive predicate would have passed it.
# The boundary comparison itself is `cell_score`'s, already proven falsifiable
# (F-P05-8); re-asserting it here would let the two disagree about what the
# matrix says.

# The mark every planted file carries. Named once: `_planted` greps the range
# for the PATH, and a human reading a refused range needs the file to say what
# put it there (STD-001).
C3_PAYLOAD_MARK='p-c3 planted payload'

# H4's undeclared path. Undeclared under BOTH fixtures' slices — light's SL-001
# declares `src/**` alone, heavy's SL-241 declares `scripts/spike-capsule/**`,
# `.doctrine/rfc/025/**` and `.gitignore` — and deliberately NOT under
# `.doctrine/`/`.claude/`, which is conform leg 3's subject and H5's row.
C3_UNDECLARED_PATH=docs/h4-undeclared.md

# ── the shared vehicle ──────────────────────────────────────────────────────

# The capsule's own clone — the one tree these rows mutate.
c3_capsule_repo() { printf '%s' "$1/capsule/repo"; }

c3_base() { contract_field "$1" base; }

# The OID the capsule is currently publishing.
c3_result() {
  git -C "$(c3_capsule_repo "$1")" rev-parse --verify "${RIG_RESULT_REF}"
}

# c3_plant_file <run> <path> — write a marked payload file into the capsule clone.
c3_plant_file() {
  local run=$1 path=$2 repo
  repo=$(c3_capsule_repo "${run}")
  mkdir -p -- "$(dirname -- "${repo}/${path}")"
  printf '%s: %s\n' "${C3_PAYLOAD_MARK}" "${path}" >>"${repo}/${path}"
}

# c3_commit <run> <message> <path…> — stage the named paths (deletions included)
# and commit them in the capsule clone.
c3_commit() {
  local run=$1 message=$2 repo
  shift 2
  repo=$(c3_capsule_repo "${run}")
  git -C "${repo}" add -- "$@"
  git -C "${repo}" commit --quiet -m "${message}"
}

# c3_publish <run> [oid]
#
# Re-point `RIG_RESULT_REF` at <oid> (default HEAD) and REWRITE THE BUNDLE.
#
# Both, always, and the bundle is the half that is easy to forget: the worker
# wrote it inside `pipeline_capsule`, BEFORE this mutate ran. A row that moved
# only the ref would leave every M-B cell harvesting the UNMUTATED result and
# passing vacuously — half the matrix scoring a payload that never shipped,
# which is F-7's hazard at the mechanism level and invisible in the results
# table. Rewritten unconditionally rather than only under `mechanism=bundle`:
# a conditional here is one a reader would have to verify per row, and the cost
# is ~0.5s on the heavy fixture.
c3_publish() {
  local run=$1 oid repo
  repo=$(c3_capsule_repo "${run}")
  oid=$(git -C "${repo}" rev-parse --verify "${2:-HEAD}")
  git -C "${repo}" update-ref "${RIG_RESULT_REF}" "${oid}"
  git -C "${repo}" bundle create --quiet \
    "${run}/capsule/${RIG_BUNDLE}" "${RIG_RESULT_REF}" 2>/dev/null ||
    rig_die "c3_publish: could not rewrite the bundle at ${run}/capsule/${RIG_BUNDLE}"
}

# c3_range <repo> <base> <oid> — the paths B..S, in the BELT'S OWN invocation
# form (F-4): `core.quotePath=false` so a non-ASCII path emits verbatim rather
# than C-quoted, `--no-renames` so a rename's source leg cannot hide behind its
# destination, `-z` because a path may contain a newline. Copied from conform
# leg 3 rather than re-derived — a reader spelling it for itself would drop one
# of the three and score its own blind spot as the model's.
c3_range() {
  git -C "$1" -c core.quotePath=false diff --name-only --no-renames -z "$2..$3" |
    tr '\0' '\n'
}

# c3_planted_paths <run> <path…>
#
# The positive control most rows want: every named path is in the range the
# pipeline will fold. Prints them; returns 1 if ANY is missing, which reds the
# cell (F-7). Read off the capsule's PUBLISHED result, not its worktree — a file
# written but never committed is exactly the plant that would not have fired.
c3_planted_paths() {
  local run=$1 changed p
  shift
  changed=$(c3_range "$(c3_capsule_repo "${run}")" "$(c3_base "${run}")" "$(c3_result "${run}")")
  for p in "$@"; do
    printf '%s\n' "${changed}" | command grep -qxF -- "${p}" || return 1
  done
  printf '%s' "$*"
}

# c3_assert_stage_passed <at> <run> <stage> — the payload got PAST <stage>.
# `grep -qx` against the emitted line, never an exit code (VA-2).
c3_assert_stage_passed() {
  rig_assert "$1: ${3} is recorded PASSED before the refusal" \
    command grep -qx "stage=${3} verdict=pass token=" "$2/stages"
}

# c3_assert_ingested <at> <run> <path…>
#
# The planted paths are in the range the TRUSTED SIDE folded, read from the
# QUARANTINE. The distinction from `_planted` is the whole value of this
# assertion: the capsule's copy proves the rig planted something, the
# quarantine's proves the pipeline had it in front of it when it refused. A row
# that only checked the capsule could not tell a real refusal from a transport
# that dropped the payload and refused for some other reason.
c3_assert_ingested() {
  local at=$1 run=$2 changed p
  shift 2
  changed=$(c3_range "${run}/quarantine" "$(c3_base "${run}")" "$(cat "${run}/pinned-oid")")
  for p in "$@"; do
    rig_assert "${at}: '${p}' is in the range the trusted side folded" \
      command grep -qxF -- "${p}" <<<"${changed}"
  done
}

# ── T4a — the result-tree rows ──────────────────────────────────────────────
#
# Ordinary git mutations on the capsule repo. Everything here refuses at
# `harvest` or `conform`, upstream of the verify capsule, which is why the group
# is unaffected by the heavy fixture's stage-3 disk-cap exposure (F-P05-11).

# H1 — the result is committed on a history REBASED OFF B, so the pinned OID is
# not a descendant of the contracted base. conform leg 1's first clause.
#
# Built with `commit-tree` rather than a real rebase, and that is a control not
# a shortcut: the tree is the worker's own, byte for byte, so the row differs
# from the happy path in the PARENT and in nothing else. A rebase would also
# have re-authored the tree, and a refusal could then be about either.
H1_mutate() {
  local run=$1 repo base tree oid
  repo=$(c3_capsule_repo "${run}")
  base=$(c3_base "${run}")
  tree=$(git -C "${repo}" rev-parse 'HEAD^{tree}')
  oid=$(git -C "${repo}" commit-tree "${tree}" -p "${base}^" \
    -m 'H1: the result, committed on a history rebased off B')
  c3_publish "${run}" "${oid}"
}

# The payload IS the ancestry relation, so the control is its ABSENCE: this
# reds the cell whenever S is still a descendant, which is the state a mutate
# that silently did nothing would leave behind.
H1_planted() {
  local run=$1 base oid
  base=$(c3_base "${run}")
  oid=$(c3_result "${run}")
  git -C "$(c3_capsule_repo "${run}")" merge-base --is-ancestor "${base}" "${oid}" 2>/dev/null &&
    return 1
  printf 'S=%s is NOT a descendant of B=%s' "${oid}" "${base}"
}

H1_assert() {
  local run=$1 at="H1/$2/$3"
  # Ancestry is a CONFORM-stage decision over objects already in quarantine,
  # not a transport failure. Without this the row would score identically if
  # harvest had refused the result for some unrelated reason.
  c3_assert_stage_passed "${at}" "${run}" harvest
}

# H3 — a MERGE COMMIT inside the result range (I3).
#
# The side parent is built off B with B's own tree, so the merge introduces no
# path of its own: the row's only hostile property is its shape.
H3_mutate() {
  local run=$1 repo base side merge
  repo=$(c3_capsule_repo "${run}")
  base=$(c3_base "${run}")
  side=$(git -C "${repo}" commit-tree "${base}^{tree}" -p "${base}" \
    -m 'H3: a side branch off B')
  merge=$(git -C "${repo}" commit-tree "$(git -C "${repo}" rev-parse 'HEAD^{tree}')" \
    -p HEAD -p "${side}" -m 'H3: a merge commit inside the result range')
  c3_publish "${run}" "${merge}"
}

H3_planted() {
  local run=$1 base oid merges
  base=$(c3_base "${run}")
  oid=$(c3_result "${run}")
  merges=$(git -C "$(c3_capsule_repo "${run}")" rev-list --min-parents=2 "${base}..${oid}")
  [ -n "${merges}" ] || return 1
  printf 'merge commit(s) in B..S: %s' "$(printf '%s' "${merges}" | tr '\n' ' ')"
}

# I3's claim, made observable: a naive single-commit ancestry predicate PASSES
# this row. `--is-ancestor B S` is true for a merge whose first parent chain
# reaches B, so only the no-merge leg kills it. Asserted rather than commented,
# because it is the entire reason leg 1 has two clauses instead of one.
H3_assert() {
  local run=$1 at="H3/$2/$3" base
  base=$(c3_base "${run}")
  rig_assert "${at}: I3 — the NAIVE ancestry predicate passes this result" \
    git -C "${run}/quarantine" merge-base --is-ancestor "${base}" "$(cat "${run}/pinned-oid")"
  c3_assert_stage_passed "${at}" "${run}" harvest
}

# H4 — an edit OUTSIDE the contracted slice's design-target selectors.
#
# The worker's own commit stays in the range and is DECLARED under both
# fixtures, so the refusal is attributable to this path rather than to a range
# in which nothing was declared at all.
H4_mutate() {
  local run=$1
  c3_plant_file "${run}" "${C3_UNDECLARED_PATH}"
  c3_commit "${run}" 'H4: an edit outside the slice design-target selectors' \
    "${C3_UNDECLARED_PATH}"
  c3_publish "${run}"
}

H4_planted() {
  c3_planted_paths "$1" "${C3_UNDECLARED_PATH}"
}

H4_assert() {
  local run=$1 at="H4/$2/$3"
  c3_assert_stage_passed "${at}" "${run}" harvest
  c3_assert_ingested "${at}" "${run}" "${C3_UNDECLARED_PATH}"
}
