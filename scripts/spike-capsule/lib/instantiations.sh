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

# The two fixtures' declared surfaces, measured rather than recalled
# (2026-08-02) — H4 and H5 both key off them, and the difference is what
# F-P05-14/18/21 kept rediscovering the hard way. Only **design-target**
# satisfies `slice conformance --strict`; `scope-relevant` does not.
#
#   light  SL-001   src/**                          design-target
#                   .doctrine/**                    design-target   (D-P05-9)
#   heavy  SL-241   scripts/spike-capsule/**        design-target
#                   .doctrine/rfc/025/evidence/**   design-target   (EMPTY at B)
#                   .doctrine/rfc/025/**            scope-relevant
#                   .doctrine/knowledge/**          scope-relevant
#
# H4's undeclared path, undeclared under BOTH — `docs/` is named by neither, and
# deliberately not under `.doctrine/`/`.claude/`, which is conform leg 3's
# subject and H5's row.
C3_UNDECLARED_PATH=docs/h4-undeclared.md

# H5's forms. Both fixtures must be able to express a form for it to be common
# to the row (D-P05-10), and leg 2 runs BEFORE leg 3 — so every path here has to
# be design-target on the fixture that plants it, or the cell refuses
# `undeclared-path` and scores a defect of the MODEL (F-P05-18, R4's direction).
#
# `.doctrine/rfc/025/evidence/` is the ONLY prefix satisfying that on both:
# heavy declares nothing else as design-target under `.doctrine/`, and light's
# `.doctrine/**` covers it. The spec's literal `.doctrine/naïve.md` would refuse
# on heavy.
C3_H5_PLAIN='.doctrine/rfc/025/evidence/h5-plain.md'
C3_H5_NONASCII='.doctrine/rfc/025/evidence/h5-naïve.md'

# The rename-out, LIGHT ONLY (F-P05-21). Leg 3 reads a two-dot tree diff, so the
# source must exist AT B — a file created and renamed inside the range is in
# neither tree and never appears. Heavy's only design-target `.doctrine/` prefix
# holds zero files at B, so it has no source to rename; a guard is observed once
# rather than per fixture (D-P05-10), and T6's isolated probe is where that
# observation is made.
C3_H5_RENAME_SRC='.doctrine/project-orientation.md'
C3_H5_RENAME_DST='src/h5-renamed.md'

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
  # A failed `add` must not be survivable. It leaves the payload out of the
  # commit while the commit itself still succeeds on whatever else was staged,
  # so the cell runs on a range missing the very thing it was planting — and
  # `planted?` is the only thing standing between that and a scored result.
  # Met for real on H5's rename leg, where the fatal was tolerated in a GREEN
  # log (2026-08-02).
  git -C "${repo}" add -- "$@" ||
    rig_die "c3_commit: could not stage $* in ${repo}"
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

# H2 — the capsule REWRITES ITS PUBLISHED RESULT AFTER RINGING, and the pipeline
# passes anyway. A DISSOLUTION row (D-P05-8, F-P05-13): the hazard is not
# undefended, it is unreachable by construction.
#
# `harvest-fetch.sh:66-79` pins the ref ITSELF, then fetches, then compares — so
# its `oid-mismatch` guard covers exactly the window between its own `rev-parse`
# and its own `fetch`, inside one process. A mutate in the A-2 seam is upstream
# of BOTH reads, so pin and fetch agree and no mismatch is producible. The row
# therefore demonstrates the dissolution rather than asserting it: the rewrite is
# PERFORMED, and the observable is that the trusted side pinned the rewritten
# result and never consulted the OID the doorbell announced (I5).
#
# MESSAGE-ONLY amend, and that is a control rather than a shortcut — the same
# argument H1's `commit-tree` makes. The tree stays byte-identical to the
# worker's, so the row differs from the happy path in the OID and in nothing
# else; a tree-changing amend would have to clear conform and verify on its own
# merits, and a refusal could then be about the payload instead of about the
# rewrite. It also keeps the row off the fixtures' verify surfaces, where
# arbitrary bytes appended to a declared `.ts` or `.rs` path would refuse at
# stage 3 for a reason that is about the payload and not about H2.
#
# Descendant-preserving on purpose: `--amend` keeps the worker's parent, so S
# stays a non-merge descendant of B. An amend that moved the parent would be H1,
# and one that introduced a second parent would be H3 — `_planted` asserts
# against both, so this row cannot pass while wearing another row's payload.
H2_mutate() {
  local run=$1
  git -C "$(c3_capsule_repo "${run}")" commit --quiet --amend \
    -m 'H2: the result, rewritten after the doorbell rang'
  c3_publish "${run}"
}

# c3_doorbell_oid <run> — the OID the capsule ANNOUNCED when it rang.
#
# The one place in the rig that reads the doorbell's content, and it reads it as
# EVIDENCE, never as authority: this is the value the pipeline must be shown NOT
# to have used. `worker-stub.sh:72` writes `capsule=… oid=…` non-empty precisely
# so the "content is never read" claim has something to be tested against.
c3_doorbell_oid() {
  sed -n 's/.*oid=\([0-9a-f]\{7,\}\).*/\1/p' "$1/capsule/${RIG_DOORBELL}" | head -1
}

H2_planted() {
  local run=$1 repo base rung published
  repo=$(c3_capsule_repo "${run}")
  base=$(c3_base "${run}")
  rung=$(c3_doorbell_oid "${run}")
  published=$(c3_result "${run}")
  # A doorbell that never rang would make the comparison below vacuous — the
  # empty string differs from every OID — so its absence reds the cell.
  [ -n "${rung}" ] || return 1
  [ "${rung}" != "${published}" ] || return 1
  # Still H2 and not H1/H3: a rewrite that broke either relation would be scored
  # against H2's dissolution while carrying another row's hostility.
  git -C "${repo}" merge-base --is-ancestor "${base}" "${published}" 2>/dev/null || return 1
  [ -z "$(git -C "${repo}" rev-list --min-parents=2 "${base}..${published}" 2>/dev/null)" ] || return 1
  printf 'the ref moved %s → %s after the ring; S is still a non-merge descendant of B' \
    "${rung}" "${published}"
}

H2_assert() {
  local run=$1 at="H2/$2/$3" rung pinned
  rung=$(c3_doorbell_oid "${run}")
  pinned=$(cat "${run}/pinned-oid")

  # THE DISSOLUTION, stated as the two observations that constitute it.
  rig_assert_eq "${at}: the trusted side pinned the REWRITTEN result" \
    "$(c3_result "${run}")" "${pinned}"
  rig_assert "${at}: I5 — the pinned OID is NOT the one the doorbell carried" \
    test "${pinned}" != "${rung}"

  # And the rewrite cost the capsule nothing: every stage names that same OID
  # consistently, so all four pass. Asserted per stage rather than inferred from
  # the absence of a refusal — `cell_score` already reads the absence, and a row
  # that only re-read it would add nothing the scorer does not have.
  c3_assert_stage_passed "${at}" "${run}" harvest
  c3_assert_stage_passed "${at}" "${run}" conform
  c3_assert_stage_passed "${at}" "${run}" verify
  c3_assert_stage_passed "${at}" "${run}" advance
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

# H5 — a GOVERNANCE-PATH touch, refused by conform leg 3 (`forbidden-path`).
#
# The first row that needs to know which fixture it is running against. That is
# not a signature change: `probe-c3.sh:569` has always called
# `"${row}_mutate" <run> <fixture> <mechanism> <alt>`, and H1/H3/H4 take only
# `<run>` because they happened not to need more.
#
# THE ROW'S JOB IS THE BOUNDARY, not the belt's hardening (D-P05-10). Leg 3
# returns on the first matching path, so a range carrying several `.doctrine/`
# paths cannot show which hardening caught it — an unhardened leg 3 refuses on
# another form's path and the cell still scores `pass` (F-P05-22). The two
# hardening guards are T6's isolated probes; here the forms are simply the
# governance touches each fixture can express.

# c3_h5_paths <fixture> — PUBLISHES `C3_H5_PATHS`, the form set this fixture
# plants. One definition, because `_mutate`, `_planted` and `_assert` must agree
# about it and a `case` in each is three places to disagree in.
#
# Published rather than printed, the way `cell_pipeline_leg` publishes
# `CELL_OBSERVED` (`probe-c3.sh`): an array of paths cannot survive a `$( … )`
# intact, and reconstructing one by splitting a string is the bug the rig's `-z`
# discipline exists to avoid.
c3_h5_paths() {
  C3_H5_PATHS=("${C3_H5_PLAIN}" "${C3_H5_NONASCII}")
  # Both legs of the rename: `--no-renames` means leg 3 sees the SOURCE as its
  # own deletion, and the range carries the destination too.
  [ "$1" = light ] || return 0
  C3_H5_PATHS+=("${C3_H5_RENAME_SRC}" "${C3_H5_RENAME_DST}")
}

H5_mutate() {
  local run=$1 fixture=$2
  c3_plant_file "${run}" "${C3_H5_PLAIN}"
  c3_plant_file "${run}" "${C3_H5_NONASCII}"
  c3_commit "${run}" 'H5: a governance-path edit, and a non-ASCII governance path' \
    "${C3_H5_PLAIN}" "${C3_H5_NONASCII}"

  if [ "${fixture}" = light ]; then
    # `git mv` rather than a delete-plus-add, so the range carries a real rename
    # for `--no-renames` to decompose. A hand-rolled pair would exercise the
    # decomposition against something git never had to detect in the first place.
    git -C "$(c3_capsule_repo "${run}")" mv -- \
      "${C3_H5_RENAME_SRC}" "${C3_H5_RENAME_DST}"
    # DESTINATION ONLY. `git mv` has already staged both legs, so the source
    # exists in neither the worktree nor the index and re-adding it matches no
    # pathspec at all — `c3_commit` now dies on that rather than leaving a
    # `fatal:` inside a passing log. The deletion still lands: `c3_commit`'s
    # `git commit` carries no pathspec and takes the whole index.
    c3_commit "${run}" 'H5: a rename OUT of the governance surface' \
      "${C3_H5_RENAME_DST}"
  fi

  c3_publish "${run}"
}

H5_planted() {
  local run=$1
  c3_h5_paths "$2"
  c3_planted_paths "${run}" "${C3_H5_PATHS[@]}"
}

H5_assert() {
  local run=$1 at="H5/$2/$3"
  c3_h5_paths "$2"
  c3_assert_stage_passed "${at}" "${run}" harvest
  # Every form reached the trusted side. The token cannot say this: it names one
  # path at most, and a transport that dropped a form would refuse identically.
  c3_assert_ingested "${at}" "${run}" "${C3_H5_PATHS[@]}"
}
