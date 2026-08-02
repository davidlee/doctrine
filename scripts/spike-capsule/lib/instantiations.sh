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

# ── T4b — the dissolutions ──────────────────────────────────────────────────
#
# A dissolution is a hazard the model REMOVES rather than guards, and § 5.6
# calls it the design's best result rather than a gap. `cell_score` reads ANY
# refusal as falsifying one (R-D), so every row here must clear all four stages
# — which makes this the first group to pay heavy's verify: ~6 min a heavy cell
# against ~2s for T4a's, and most of it is `bun install` fetching at ~0% CPU
# (F-P05-19). That is a network-bound fetch, not a hang.
#
# H2 is the shape they copy: PERFORM the hostile thing and show the pipeline
# unbothered, which beats asserting that it could not be done.

# c3_execution_log <run> — the HOST file a planted payload appends to if it ever
# executes, and the row reads WHERE it ran, not merely whether.
#
# Under the run directory, never `/tmp`: the sandbox mounts a tmpfs over `/tmp`
# inside a capsule, so a sentinel written there is invisible from outside and
# its absence would attest to a containment that was never tested (F-P04-12).
#
# A log rather than a flag because the dissolution needs both halves. H6's
# positive control is the CAPSULE firing its own hook — without it, a payload
# with a bad shebang or a lost `+x` bit would leave the trusted side silent for
# a reason that has nothing to do with the model (F-7, one level in).
c3_execution_log() { printf '%s' "$1/executed-in"; }

# A predicate rather than an inline `test … -o …`: the log legitimately does not
# exist when nothing ran anywhere, and that is a PASS, not a missing file.
c3_no_execution_in() {
  local log=$1 where=$2
  [ -e "${log}" ] || return 0
  ! command grep -qF -- "${where}" "${log}"
}

# c3_assert_never_ran_in <at> <run> <dir…> — no planted payload executed in any
# of these directories. Read off the log's recorded `pwd`, so a payload that ran
# somewhere unexpected is still visible rather than silently matching nothing.
c3_assert_never_ran_in() {
  local at=$1 run=$2 log d
  log=$(c3_execution_log "${run}")
  shift 2
  for d in "$@"; do
    rig_assert "${at}: nothing the capsule planted executed in ${d##*/}" \
      c3_no_execution_in "${log}" "${d}"
  done
}

# `-e` is false for a DANGLING symlink, and every escape this group plants is
# dangling by design — so absence has to be asserted with `-L` beside it or a
# materialised symlink would read as "never written".
c3_path_absent() { [ ! -e "$1" ] && [ ! -L "$1" ]; }

# c3_range_matches <repo> <B> <S> <pattern> — does any CHANGED path match?
c3_range_matches() {
  c3_range "$1" "$2" "$3" | command grep -q -- "$4"
}

# H6 — A HOSTILE `.git/config` AND EXECUTABLE HOOKS in the capsule's own clone.
#
# Dissolved by construction: config and hooks are REPO-LOCAL and are never git
# objects, so neither the fetch nor the bundle carries them. The trusted side
# receives a commit and nothing else, and there is no ingestion step at which a
# capsule-authored script could become the thing that runs.
#
# The row performs the whole hostile setup rather than asserting the absence:
# the hooks are real, executable, and OBSERVED FIRING inside the capsule.

# Hook names chosen for what would fire TRUSTED-SIDE if the config travelled.
# `reference-transaction` is the sharpest — git runs it on every ref update,
# including the quarantine's own fetch, so a transferred hooksPath would execute
# during stage 1 itself.
C3_H6_HOOKS='reference-transaction post-checkout post-merge pre-push update post-update pre-receive post-receive'

# c3_h6_write_hooks <run> <dir> — a live payload under every hook name.
c3_h6_write_hooks() {
  local run=$1 dir=$2 hook
  mkdir -p -- "${dir}"
  for hook in ${C3_H6_HOOKS}; do
    # `$(pwd -P)` is escaped so the HOOK evaluates it at run time; everything
    # else is expanded now. The recorded directory is what tells the row apart
    # from its own positive control.
    cat >"${dir}/${hook}" <<EOF
#!/bin/sh
# H6's planted payload (${C3_PAYLOAD_MARK}).
printf 'h6/${hook} ran in %s\n' "\$(pwd -P)" >>"$(c3_execution_log "${run}")"
exit 0
EOF
    chmod +x -- "${dir}/${hook}"
  done
}

H6_mutate() {
  local run=$1 repo hooks
  repo=$(c3_capsule_repo "${run}")
  hooks="${repo}/.git/c3-h6-hooks"

  # BOTH locations, because there are two transfer stories and only one of them
  # involves the config: git's default `.git/hooks`, and a `core.hooksPath` the
  # capsule chose for itself.
  c3_h6_write_hooks "${run}" "${repo}/.git/hooks"
  c3_h6_write_hooks "${run}" "${hooks}"

  # ABSOLUTE, and that is what makes the observable non-vacuous. A repo-relative
  # `core.hooksPath` copied into the quarantine would find no hooks THERE, so
  # the trusted side would stay silent whether or not the config had travelled.
  # Pointed at the capsule's own directory, a transferred config would really
  # execute the capsule's scripts — which is the thing shown not to happen.
  git -C "${repo}" config core.hooksPath "${hooks}"
  # A config-borne execution trigger that is not a hook at all: `upload-pack`
  # runs this INSTEAD of `pack-objects` when serving a fetch, i.e. as a child of
  # the trusted side's own `git fetch` on M-A. Git defends it by honouring the
  # key only from protected (system/global) configuration; the row OBSERVES that
  # defence rather than trusting the documentation for it.
  git -C "${repo}" config uploadpack.packObjectsHook "${hooks}/pre-receive"
  git -C "${repo}" config core.fsmonitor "${hooks}/post-checkout"

  # THE POSITIVE CONTROL, and it belongs here rather than in `_planted`: a no-op
  # ref write fires the capsule's OWN `reference-transaction`, so the row knows
  # the payload is live before it claims nothing ran elsewhere.
  git -C "${repo}" update-ref refs/heads/c3-h6-control "$(c3_result "${run}")"

  # NO `c3_publish`, deliberately. H6 changes no objects, so the worker's ref
  # and the bundle it wrote are both already correct — and a republish now would
  # fire the hooks a second time, adding capsule-side noise to the one file this
  # row reads.
}

H6_planted() {
  local run=$1 repo log
  repo=$(c3_capsule_repo "${run}")
  log=$(c3_execution_log "${run}")

  [ -x "${repo}/.git/c3-h6-hooks/reference-transaction" ] || return 1
  [ "$(git -C "${repo}" config --get core.hooksPath)" = "${repo}/.git/c3-h6-hooks" ] || return 1
  # The payload is LIVE: git found it, executed it, and it wrote where it said
  # it would. Without this the row's whole claim would rest on a script nothing
  # had ever run.
  command grep -q "h6/reference-transaction ran in ${repo}" "${log}" 2>/dev/null || return 1
  printf 'core.hooksPath set and %s hooks live — reference-transaction OBSERVED firing in the capsule' \
    "$(printf '%s' "${C3_H6_HOOKS}" | wc -w)"
}

H6_assert() {
  local run=$1 at="H6/$2/$3"
  local q="${run}/quarantine" c="${run}/canonical"

  # THE DISSOLUTION, as the two structural facts that constitute it.
  #
  # 1. The config did not travel. Asserted on both trusted-side repositories,
  #    because the mechanisms differ in which one the capsule ever touches.
  rig_assert_fails "${at}: the quarantine has no core.hooksPath" \
    git -C "${q}" config --get core.hooksPath
  rig_assert_fails "${at}: the quarantine has no uploadpack.packObjectsHook" \
    git -C "${q}" config --get uploadpack.packObjectsHook
  rig_assert_fails "${at}: canonical has no core.hooksPath" \
    git -C "${c}" config --get core.hooksPath

  # 2. The hook FILES did not travel either — neither as config's target nor as
  #    objects. `ls-tree` is the object half: a hook that had been committed
  #    would be a path in the result and this would catch it.
  rig_assert "${at}: no hostile hook directory in the quarantine" \
    test ! -e "${q}/.git/c3-h6-hooks"
  rig_assert "${at}: no hostile hook directory in canonical" \
    test ! -e "${c}/.git/c3-h6-hooks"
  #    Read off the RANGE rather than the whole tree: `git grep` over a clone of
  #    this repository would match whatever the fixture's B happens to carry,
  #    which is a fact about the pin and not about the row.
  rig_assert_fails "${at}: hooks are not objects — no hook path is in the range B..S" \
    c3_range_matches "${q}" "$(c3_base "${run}")" "$(cat "${run}/pinned-oid")" 'c3-h6-hooks'

  # And therefore nothing executed on the trusted side. The positive control in
  # `_planted` is what makes this a result rather than a tautology.
  c3_assert_never_ran_in "${at}" "${run}" "${q}" "${c}"

  # The capsule gained nothing: all four stages name the same result and pass.
  c3_assert_stage_passed "${at}" "${run}" harvest
  c3_assert_stage_passed "${at}" "${run}" conform
  c3_assert_stage_passed "${at}" "${run}" verify
  c3_assert_stage_passed "${at}" "${run}" advance
}

# H9 — AN IN-TREE SYMLINK POINTING OUT OF THE TREE, and a `.gitattributes`
# naming a filter driver. Scored as TWO (D-P05-2, § 5.6 "scored as two"):
#
#   dissolution   inert at ingest — no candidate tree is materialised trusted
#                 side (I4), so there is no path for anything to follow and no
#                 checkout for a smudge filter to run in
#   verify        containment — the verify capsule DOES check the tree out, and
#                 the claim is that it is confined when it does
#
# Both legs run the same pipeline; what differs is what each OBSERVES, which is
# the point of one leg per alternative rather than one leg satisfying both.
#
# PLANTED UNDER A DESIGN-TARGET DIRECTORY, per fixture. A root `.gitattributes`
# is undeclared under both fixtures' selectors and would refuse `undeclared-path`
# at conform leg 2 — the F-P05-18 trap, and it would score as a defect of the
# model. Git honours a per-directory `.gitattributes` over that directory's
# files, so the hazard is unchanged by the move. Measured on both fixtures,
# dotfile included (2026-08-02).
C3_H9_FILTER=h9-hostile

# c3_design_target_dir <fixture> — a directory this fixture's slice declares
# design-target, and the answer to the question three rows now ask.
#
# Shared rather than per-row: H9 and H12 both need "somewhere leg 2 will admit",
# and a second copy is a second place to disagree with the fixtures' selectors.
# Measured on both fixtures, dotfiles included (2026-08-02) — light `src/**`
# from SL-001, heavy `scripts/spike-capsule/**` from SL-241.
c3_design_target_dir() {
  case "$1" in
    light) printf 'src' ;;
    heavy) printf 'scripts/spike-capsule' ;;
    *) rig_die "no design-target plant directory for fixture: $1" ;;
  esac
}

# c3_h9_paths <fixture> — PUBLISHES `C3_H9_PATHS`, for the same reason
# `c3_h5_paths` does: an array cannot survive a `$( … )`, and the three hooks
# must agree about the form set without a `case` in each.
c3_h9_paths() {
  local dir
  dir=$(c3_design_target_dir "$1")
  C3_H9_PATHS=(
    "${dir}/.gitattributes"
    "${dir}/h9-payload.txt"
    "${dir}/h9-escape-abs"
    "${dir}/h9-escape-rel"
  )
}

# The host path the ABSOLUTE symlink names — outside every capsule's writable
# root, and outside the sandbox's view entirely. Its continued absence is the
# containment observable.
c3_h9_escape_target() { printf '%s' "$1/h9-escaped"; }

H9_mutate() {
  local run=$1 fixture=$2 repo dir
  repo=$(c3_capsule_repo "${run}")
  dir=$(c3_design_target_dir "${fixture}")

  # A filter driver named in the attributes and DEFINED in the capsule's own
  # repo config — the complete hazard, and the half that cannot travel is the
  # same half H6 is about.
  printf 'h9-payload.txt filter=%s diff=%s\n' "${C3_H9_FILTER}" "${C3_H9_FILTER}" \
    >"${repo}/${dir}/.gitattributes"
  git -C "${repo}" config "filter.${C3_H9_FILTER}.smudge" \
    "sh -c 'printf \"h9/smudge ran in %s\\n\" \"\$(pwd -P)\" >>\"$(c3_execution_log "${run}")\"; cat'"
  c3_plant_file "${run}" "${dir}/h9-payload.txt"

  # Two escapes, because they fail differently. The absolute one names a host
  # path the sandbox has no view of at all; the relative one normalises above
  # the checkout root, so it is the traversal a `..`-stripping check would miss.
  ln -sfn -- "$(c3_h9_escape_target "${run}")" "${repo}/${dir}/h9-escape-abs"
  ln -sfn -- '../../../../../../../../../../h9-escaped-rel' "${repo}/${dir}/h9-escape-rel"

  c3_h9_paths "${fixture}"
  c3_commit "${run}" 'H9: an out-of-tree symlink and a filter-driver attribute' \
    "${C3_H9_PATHS[@]}"
  c3_publish "${run}"
}

H9_planted() {
  local run=$1 repo dir
  repo=$(c3_capsule_repo "${run}")
  dir=$(c3_design_target_dir "$2")
  # The symlink must be a SYMLINK in the tree, not a regular file git happened
  # to record: mode 120000 is what conform leg 4 leaves legal (SL-012), and a
  # dereferenced copy would make the row a different one entirely.
  git -C "${repo}" ls-tree "$(c3_result "${run}")" -- "${dir}/h9-escape-abs" |
    command grep -q '^120000 ' || return 1
  c3_h9_paths "$2"
  c3_planted_paths "${run}" "${C3_H9_PATHS[@]}"
}

H9_assert() {
  local run=$1 fixture=$2 at="H9/$2/$3/$4" alt=$4 dir
  local q="${run}/quarantine"
  dir=$(c3_design_target_dir "${fixture}")
  c3_h9_paths "${fixture}"

  c3_assert_stage_passed "${at}" "${run}" harvest
  # The objects came across. Stated first because every claim below is about
  # what did NOT happen, and each of them would hold vacuously against a
  # transport that had dropped the payload.
  c3_assert_ingested "${at}" "${run}" "${C3_H9_PATHS[@]}"

  case "${alt}" in
    dissolution)
      # I4, as the pairing that makes it non-vacuous: the payload is IN the
      # range the trusted side folded, and yet no candidate tree exists for
      # anything to follow. The quarantine's worktree sits at B by construction
      # (`pipeline.sh:28-34`); this observes it rather than citing it.
      rig_assert "${at}: I4 — no candidate tree trusted-side; the symlink was never written" \
        c3_path_absent "${q}/${dir}/h9-escape-abs"
      rig_assert "${at}: I4 — the filtered payload was never checked out either" \
        c3_path_absent "${q}/${dir}/h9-payload.txt"
      # Symlinks stay LEGAL (SL-012) and leg 4 matches `.gitmodules` by name, so
      # neither form is a conform refusal. Asserted because a leg that grew a
      # mode or name test would make this row score as H8.
      c3_assert_stage_passed "${at}" "${run}" conform
      ;;
    verify)
      # Containment: the verify capsule is the ONE place the tree is
      # materialised, and it ran to a verdict with both escapes present.
      c3_assert_stage_passed "${at}" "${run}" verify
      rig_assert "${at}: the absolute escape target is untouched on the host" \
        c3_path_absent "$(c3_h9_escape_target "${run}")"
      rig_assert "${at}: the relative escape normalises above the root and reached nothing" \
        c3_path_absent /h9-escaped-rel
      ;;
    *) rig_die "H9: unknown alternative: ${alt}" ;;
  esac

  # Neither the smudge filter nor anything else ran outside the capsule. The
  # driver is NAMED by an object and DEFINED by config, and only the first half
  # travels — which is H6's dissolution arriving as H9's.
  c3_assert_never_ran_in "${at}" "${run}" "${q}" "${run}/canonical"
}

# H12 — THE EVALUATION SURFACES `.envrc` AND `flake.nix`, MODIFIED AGAINST B.
#
# Dissolved by construction, and § 5.6 is precise about the shape: *an audit
# row, not a pipeline row*. `audit-dq4` proves no trusted-side step exists that
# could evaluate them; the pipeline cell exists to show the surfaces TRAVELLING
# — they are ordinary objects and conform admits them — while nothing on the
# trusted side reads what they say.
#
# PLANTED UNDER THE DESIGN-TARGET DIRECTORY (D-P05-11). The matrix's original
# root-level pair is unreachable on heavy for two independent reasons measured
# at F-P05-26: root `flake.nix` is undeclared under SL-241's selectors, so
# conform leg 2 refuses `undeclared-path` and any refusal falsifies a
# dissolution; and `/.envrc` is gitignored in this repository, so `git add`
# refuses it outright. The move costs the hazard nothing — `.envrc` is a
# PER-DIRECTORY direnv surface and nix evaluates a `flake.nix` wherever it is
# invoked, so the class-2/3 trigger is unchanged and only the directory differs.
C3_H12_ENVRC=.envrc
C3_H12_FLAKE=flake.nix

# PUBLISHES `C3_H12_PATHS`, for the reason `c3_h5_paths` and `c3_h9_paths` do.
c3_h12_paths() {
  local dir
  dir=$(c3_design_target_dir "$1")
  C3_H12_PATHS=("${dir}/${C3_H12_ENVRC}" "${dir}/${C3_H12_FLAKE}")
}

H12_mutate() {
  local run=$1 fixture=$2 repo dir log
  repo=$(c3_capsule_repo "${run}")
  dir=$(c3_design_target_dir "${fixture}")
  log=$(c3_execution_log "${run}")

  # `.envrc` is plain shell, so this is the class-3 trigger itself rather than a
  # stand-in for one: anything that evaluated the file would run the payload.
  cat >"${repo}/${dir}/${C3_H12_ENVRC}" <<EOF
# ${C3_PAYLOAD_MARK}: ${dir}/${C3_H12_ENVRC}
printf 'h12/envrc evaluated in %s\n' "\$(pwd -P)" >>"${log}"
EOF

  # And a flake whose devShell hook runs the same payload — class 2. Nix is
  # ABSENT from this jail, which is exactly why the row's claim rests on the
  # audit rather than on nix declining to run: an evaluator that is not
  # installed proves nothing about a trusted side that would have called it.
  cat >"${repo}/${dir}/${C3_H12_FLAKE}" <<EOF
# ${C3_PAYLOAD_MARK}: ${dir}/${C3_H12_FLAKE}
{
  description = "H12 planted payload";
  outputs = _: {
    devShells.x86_64-linux.default.shellHook = ''
      printf 'h12/flake evaluated in %s\n' "\$(pwd -P)" >>"${log}"
    '';
  };
}
EOF

  c3_h12_paths "${fixture}"
  c3_commit "${run}" 'H12: the evaluation surfaces, modified against B' \
    "${C3_H12_PATHS[@]}"
  c3_publish "${run}"
}

H12_planted() {
  c3_h12_paths "$2"
  c3_planted_paths "$1" "${C3_H12_PATHS[@]}"
}

H12_assert() {
  local run=$1 fixture=$2 at="H12/$2/$3" dir audit
  local q="${run}/quarantine"
  dir=$(c3_design_target_dir "${fixture}")
  c3_h12_paths "${fixture}"

  c3_assert_stage_passed "${at}" "${run}" harvest
  # The surfaces TRAVELLED. Stated first, because every claim below is about
  # something not happening and each would hold vacuously against a transport
  # that had dropped the payload.
  c3_assert_ingested "${at}" "${run}" "${C3_H12_PATHS[@]}"

  # I4, and here it is the mechanism rather than a restatement of H9's: no
  # trusted-side worktree is written at all, so there is no `.envrc` on disk for
  # direnv to find and no `flake.nix` for a `nix` invocation to pick up —
  # whatever a trusted-side step's working directory happened to be.
  rig_assert "${at}: no ${C3_H12_ENVRC} materialised trusted-side" \
    c3_path_absent "${q}/${dir}/${C3_H12_ENVRC}"
  rig_assert "${at}: no ${C3_H12_FLAKE} materialised trusted-side" \
    c3_path_absent "${q}/${dir}/${C3_H12_FLAKE}"
  c3_assert_never_ran_in "${at}" "${run}" "${q}" "${run}/canonical"

  # THE STRUCTURAL PROOF — § 5.6's actual evidence for this row: no trusted-side
  # step exists that COULD evaluate them. Run against THIS RUN'S OWN pinned
  # declaration, so the tokens under audit are the heavy fixture's
  # `cargo nix direnv just rustc` rather than the light default (F-P05-3).
  #
  # The EMITTED CLAIM is asserted, not the exit status, and that is VA-2's
  # discipline rather than a convenience: pointed at a non-default declaration
  # the audit ALSO reds its own exemption-staleness check, which is bookkeeping
  # about which token set was passed in and says nothing about DQ-4 (F-P05-27).
  # Redirected to a file, never piped — a pipe would report the reader's exit
  # status instead of the audit's, which this phase has met three times.
  audit="${run}/h12-audit-dq4"
  "${RIG_DIR}/control/audit-dq4.sh" \
    --declaration "${run}/interpretation-surface.txt" >"${audit}" 2>&1 || true
  rig_assert "${at}: DQ-4 — no unaccounted trusted-side invocation of the declared evaluators" \
    command grep -q 'clean — no unaccounted trusted-side invocation' "${audit}"

  # And the capsule gained nothing: all four stages pass on the modified
  # surfaces, because nothing trusted-side ever asked what they contained.
  c3_assert_stage_passed "${at}" "${run}" conform
  c3_assert_stage_passed "${at}" "${run}" verify
  c3_assert_stage_passed "${at}" "${run}" advance
}

# H15 — THE PIPELINE IS KILLED MID-RUN, AT EACH STAGE IN TURN (EX-9).
#
# Dissolved AND REPLACED (§ 5.6): there is no journal to replay, and the
# re-derived boundary is stronger than one would be — stages 1–3 touch nothing
# canonical and are idempotent, and stage 4 is a single atomic CAS. So a crash
# before stage 4 leaves canonical byte-identical and the re-run resumes from the
# SAME pinned OID; a crash after it has landed.
#
# The row plants no objects. Its hostile input is the interruption itself, so
# the worker's result and bundle are exactly what a crashed run would leave —
# and `c3_publish` is deliberately not called for the same reason H6 does not
# call it.
#
# ── the kill vehicle, and why pipeline.sh needs no seam for it ───────────────
#
# The pipeline already emits one `stage=…` line per stage (EX-9), and bash's
# `printf` builtin flushes per call — measured on a FIFO, 2026-08-02, the line
# arriving 5s before the writer's next one. So a watcher on the far side sees
# each stage AS IT COMPLETES and kills at a chosen boundary. The interruption is
# therefore DETERMINISTIC and synchronised on the subject's own emission; a
# timed kill would be a race, and D-P05-8 already refused a racer on the grounds
# that a nondeterministic probe is not a probe.
#
# SIGKILL to the PROCESS GROUP rather than the leader, which is why the attempt
# is launched under `set -m`: the verify stage's sandbox is a child, and killing
# only the leader would orphan a bwrap still growing a 4.4G capsule. That is a
# stronger kill than a bare parent crash — which would orphan them instead — and
# it is chosen so the rig leaks nothing. What CANONICAL holds afterwards, which
# is the whole claim, is the same either way.
#
# ── stage 4 has no kill of its own, and that is the RESULT, not a gap ────────
#
# There is no interruptible interior to crash inside. A kill during `advance` is
# either before the `update-ref` — in which case canonical is untouched and the
# attempt is indistinguishable from the during-verify one — or after it, in
# which case the run COMPLETED. Racing for the microseconds between them would
# reintroduce exactly the nondeterminism this vehicle exists to avoid, and would
# make the cell's own scored run refuse `stale-base` whenever the race was lost.
# The indivisibility IS § 5.6's claim, so the row observes its CONSEQUENCE
# instead (clause 4 of `_assert`): after the resume lands, a repeat advance
# refuses `stale-base` having transferred nothing — the CAS applied exactly once.

# Each entry is the stage whose PASS line triggers the kill; `start` kills
# inside stage 1, on the artefact the harvester's own invocation creates.
C3_H15_KILL_TRIGGERS='start harvest conform'

c3_h15_dies_during() {
  case "$1" in
    start) printf 'harvest' ;;
    harvest) printf 'conform' ;;
    conform) printf 'verify' ;;
    *) rig_die "H15: unknown kill trigger: $1" ;;
  esac
}

# c3_h15_kill_attempt <run> <mechanism> <trigger> <log>
#
# Returns 0 if the attempt was KILLED, 1 if it ran to completion instead. The
# distinction is not bookkeeping: an attempt that completed is not an
# interruption at all, and a row that accepted one would report crash-safety it
# had never tested — the absence-shaped result this phase keeps meeting.
c3_h15_kill_attempt() {
  local run=$1 mechanism=$2 trigger=$3 log=$4
  local fifo="${run}/h15-fifo" pid line killed=1 had_monitor=0 waited=0

  rm -f -- "${fifo}"
  mkfifo -- "${fifo}"
  : >"${log}"

  # Job control, so the attempt gets its OWN process group. Without it a
  # background job shares this shell's group and killing the group would kill
  # the rig itself. The pgid is fixed at fork, so restoring the flag afterwards
  # is safe.
  case "$-" in *m*) had_monitor=1 ;; esac
  set -m
  pipeline_run "${run}" "${mechanism}" >"${fifo}" 2>/dev/null &
  pid=$!
  [ "${had_monitor}" -eq 1 ] || set +m

  if [ "${trigger}" = start ]; then
    rm -f -- "${run}/harvest.err"
    # The writer blocks on the FIFO until a reader opens, so opening it is what
    # releases the run. Then poll for the file stage 1's own redirect creates as
    # it invokes the harvester — that lands the kill INSIDE the fetch rather
    # than before the pipeline has begun, which would prove nothing.
    exec 9<"${fifo}"
    while [ ! -e "${run}/harvest.err" ] && [ "${waited}" -lt 500 ]; do
      sleep 0.01
      waited=$((waited + 1))
    done
    kill -KILL -- -"${pid}" 2>/dev/null && killed=0
    exec 9<&-
  else
    while IFS= read -r line; do
      printf '%s\n' "${line}" >>"${log}"
      case "${line}" in
        "stage=${trigger} verdict=pass"*)
          kill -KILL -- -"${pid}" 2>/dev/null && killed=0
          break
          ;;
      esac
    done <"${fifo}"
  fi

  wait "${pid}" 2>/dev/null || true
  rm -f -- "${fifo}"
  return "${killed}"
}

H15_mutate() {
  local run=$1 mechanism=$3 trigger during

  : >"${run}/h15-attempts"
  for trigger in ${C3_H15_KILL_TRIGGERS}; do
    during=$(c3_h15_dies_during "${trigger}")
    if c3_h15_kill_attempt "${run}" "${mechanism}" "${trigger}" "${run}/h15-stages.${during}"; then
      printf '%s=killed\n' "${during}" >>"${run}/h15-attempts"
    else
      printf '%s=COMPLETED\n' "${during}" >>"${run}/h15-attempts"
    fi

    # Canonical as it stood the instant the attempt died. Captured per attempt
    # rather than asserted here — `_assert` owns the assertions, and a snapshot
    # taken at the end could not tell a crash that left canonical alone from one
    # that was tidied up by the run that followed it.
    canonical_refs "${run}/canonical" >"${run}/h15-refs.${during}"
    canonical_objects "${run}/canonical" >"${run}/h15-objects.${during}"
    # The OID stage 1 pinned, when the attempt got that far. Absent for the
    # harvest kill by construction, which is itself the evidence it died early.
    cp -- "${run}/pinned-oid" "${run}/h15-pinned.${during}" 2>/dev/null || true
  done
}

H15_planted() {
  local run=$1
  # Three attempts, all of them actually killed.
  [ "$(wc -l <"${run}/h15-attempts")" -eq 3 ] || return 1
  ! command grep -q '=COMPLETED' "${run}/h15-attempts" || return 1

  # And killed at three DISTINCT points, read off how far each got. Without this
  # a vehicle that killed everything at t=0 would satisfy the count while
  # interrupting only one stage — the control that makes "each stage in turn"
  # an observation rather than a label.
  [ ! -s "${run}/h15-stages.harvest" ] || return 1
  command grep -qx 'stage=harvest verdict=pass token=' "${run}/h15-stages.conform" || return 1
  command grep -qx 'stage=conform verdict=pass token=' "${run}/h15-stages.verify" || return 1

  printf 'killed mid-run at three distinct points: %s' \
    "$(tr '\n' ' ' <"${run}/h15-attempts")"
}

H15_assert() {
  local run=$1 at="H15/$2/$3" during resume base accepted probe

  # 1. EVERY PRE-STAGE-4 CRASH LEFT CANONICAL BYTE-IDENTICAL — refs and the
  #    object count both. The pairing is `assert_outcome`'s and it is the one a
  #    quarantine namespace inside canonical would have broken (I1).
  for during in harvest conform verify; do
    rig_assert_eq "${at}: killed during ${during} — canonical refs unchanged" \
      "$(cat "${run}/canonical-refs.before")" "$(cat "${run}/h15-refs.${during}")"
    rig_assert_eq "${at}: killed during ${during} — canonical OBJECT COUNT unchanged" \
      "$(cat "${run}/canonical-objects.before")" "$(cat "${run}/h15-objects.${during}")"
  done

  # 2. THE RESUME RE-PINNED THE SAME OID. Stages 1–3 are idempotent, so a resume
  #    is a repeat rather than a fresh negotiation; had the pin moved, every
  #    stage downstream would have gated one commit and landed another (RT-5).
  resume=$(cat "${run}/pinned-oid")
  for during in conform verify; do
    rig_assert_eq "${at}: the resume re-pinned the OID the ${during} attempt held" \
      "$(cat "${run}/h15-pinned.${during}")" "${resume}"
  done

  # 3. And the resume COMPLETED: the crashes cost latency, not the result.
  c3_assert_stage_passed "${at}" "${run}" harvest
  c3_assert_stage_passed "${at}" "${run}" conform
  c3_assert_stage_passed "${at}" "${run}" verify
  c3_assert_stage_passed "${at}" "${run}" advance

  # 4. STAGE 4 APPLIED EXACTLY ONCE — the atomicity claim, observed where a
  #    fourth kill cannot go. `advance_stage` is called directly rather than
  #    through a second pipeline: what is being observed is the CAS, and a full
  #    run would spend another six heavy minutes re-deriving a verify pass that
  #    is not part of the claim.
  #
  #    AGAINST A THROWAWAY COPY OF CANONICAL, and that is a correctness fix
  #    rather than fastidiousness. `advance_stage` MUTATES — precondition,
  #    transfer, CAS. Pointed at the real canonical it is only harmless while
  #    the resume actually landed; when the resume REFUSES, the precondition
  #    still holds, so the assertion itself transfers the objects and moves the
  #    ref — landing the result it was supposed to be observing, and destroying
  #    the state `assert_outcome` reads immediately afterwards. Observed doing
  #    exactly that on H15/heavy/bundle (F-P05-28), where it turned one honest
  #    refusal into seven reds and briefly looked like a canonical-safety
  #    failure. An assertion must not write to its own subject; this file's
  #    header already says rows touch the capsule clone and nothing else, and
  #    `_assert` is not exempt from it.
  base=$(c3_base "${run}")
  accepted=$(contract_field "${run}" accepted)
  probe="${run}/h15-advance-probe"
  rm -rf -- "${probe}"
  git clone --no-hardlinks --quiet -- "${run}/canonical" "${probe}"
  rig_assert_eq "${at}: a repeat advance refuses stale-base — the CAS applied once" \
    stale-base \
    "$(advance_stage "${probe}" "${run}/quarantine" "${accepted}" "${base}" "${resume}" || true)"
}
