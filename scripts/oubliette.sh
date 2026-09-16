#!/usr/bin/env bash
# Round-trip a doctrine slice through an oubliette capsule.
#
#   oubliette.sh send <slice> <slot> [ref]   push the slice into a capsule
#   oubliette.sh back <slice> <slot>         bring it home, into a worktree
#
# Both halves of a slice have to travel, and only one of them does by default.
# The authored half rides the code ref. The gitignored half — `.doctrine/state/
# slice/<N>/**` (phase sheets, and therefore phase *status*) plus the ignored
# files under `.doctrine/slice/<N>/` (handover.md, research/) — rides the
# separate *state* ref: `--state-from-host --unit <N>` on the way in,
# `capsule adopt` on the way out. `capsule land` stops at code refs by design,
# so without the adopt half a landed slice's phase status is silently stale
# here, which is the failure this script exists to make impossible.
#
# Oubliette learns nothing about slices from this. Every verb below already
# exists; what is supplied is values — a ref, a unit token, a purpose, a branch
# name. docs/contract-doctrine.md Role 3 stays parked.
set -euo pipefail

# --- constants (STD-001: named once, here) ----------------------------------
CAPSULE=${CAPSULE_BIN:-$(command -v capsule || echo /run/current-system/sw/bin/capsule)}
DOCTRINE=${DOCTRINE_BIN:-./target/debug/doctrine}
GUEST_REPO=/work/doctrine                 # target.nix: volumePath/name
GUEST_STATE_REFS=refs/capsule/state       # host/state-snapshot.nix: refPrefix
EXHIBIT_DIR=.doctrine/state/capsule-exhibit   # under the landing worktree; ignored tier
WORKTREES=.worktrees

ROOT=$(git rev-parse --show-toplevel)
cd "$ROOT"

say()  { printf '\033[1m::\033[0m %s\n' "$*"; }
die()  { printf '\033[31m!!\033[0m %s\n' "$1" >&2; shift; [ $# -eq 0 ] || printf '   %s\n' "$@" >&2; exit 1; }

usage() {
  cat >&2 <<'USAGE'
usage: oubliette.sh send <slice> <slot> [ref]
       oubliette.sh back <slice> <slot>

  <slice>  245 or SL-245        <slot>  a single capsule, a..j
  [ref]    default: the current branch

  send  archives whatever the slot held, then provisions it with the slice's
        code AND its gitignored state half, and prints the agent's first line.
  back  lands the code on a fresh branch + worktree, adopts the state half
        into it, and tells you where to audit.
USAGE
  exit 2
}

# --- shared ------------------------------------------------------------------

resolve_slice() {                 # -> N, ID, TITLE
  local arg=${1:-}
  [ -n "$arg" ] || usage
  N=${arg#SL-}; N=${N#sl-}
  [[ $N =~ ^[0-9]+$ ]] || die "'$arg' is not a slice reference — 245 or SL-245."
  local json
  json=$("$DOCTRINE" slice show "$N" --json 2>/dev/null) \
    || die "no slice $arg in this repo." "doctrine slice list"
  ID=$(jq -r '.slice.id | "SL-\(.|tostring|("00"+.)[-3:])"' <<<"$json")
  TITLE=$(jq -r '.slice.title' <<<"$json")
  N=${ID#SL-}; N=$((10#$N))       # unit token: the bare number, as the record holds it
}

resolve_slot() {                  # -> SLOT, GEN
  SLOT=${1:-}
  [[ $SLOT =~ ^[a-j]$ ]] || die "'${SLOT:-}' is not a capsule slot (a..j)." "capsule all status"
  GEN=$("$CAPSULE" "$SLOT" record 2>/dev/null | jq -r '.generation // "-"') || GEN=-
  [ -n "$GEN" ] && [ "$GEN" != null ] || GEN=-
}

# The refs a slot holds in this repo, moved under its generation. Replicates
# `archiveRefs` (oubliette host/cli.nix): one `update-ref` transaction whose
# `create` refuses a name already taken, so a generation is superseded once.
archive_refs() {
  local slot=$1 g=$2 ref oid rest n=0
  local -a plan=()
  while read -r ref oid; do
    [ -n "$ref" ] || continue
    rest=${ref#refs/capsule/"$slot"/}
    plan+=("create refs/capsule/$slot/gen/$g/$rest $oid" "delete $ref $oid")
    n=$((n + 1))
  done < <(git for-each-ref --format='%(refname) %(objectname)' \
    "refs/capsule/$slot/heads/" "refs/capsule/$slot/state/")
  [ "$n" -gt 0 ] || return 0
  [ "$g" != - ] || die "'$slot' holds $n ref(s) here and has no generation to key an archive by." \
    "Move them by hand under refs/capsule/$slot/gen/<n>/."
  printf '%s\n' "${plan[@]}" | git update-ref --stdin
  say "archived $n ref(s) under refs/capsule/$slot/gen/$g/"
}

# Copy back only what git ignores. The authored half already arrived on the
# branch; the ignored half exists nowhere but the exhibit. `check-ignore` is the
# test, so a tracked file can never be overwritten by a guest-authored one —
# which is the judgement `capsule adopt` refuses to make for us, correctly.
restore_ignored() {
  local from=$1 prefix=$2 wt=$3 rel f n=0
  [ -d "$from" ] || return 0
  while IFS= read -r f; do
    rel="$prefix/$f"
    git -C "$wt" check-ignore -q "$rel" || continue
    mkdir -p "$wt/$(dirname "$rel")"
    cp -a "$from/$f" "$wt/$rel"
    n=$((n + 1))
  done < <(cd "$from" && find . -mindepth 1 \( -type f -o -type l \) -printf '%P\n')
  [ "$n" -eq 0 ] || say "restored $n ignored file(s) under $prefix/"
}

# --- send --------------------------------------------------------------------

cmd_send() {
  resolve_slice "${1:-}"
  resolve_slot "${2:-}"
  local ref=${3:-}

  # The host tree. Two silent half-failures live here: uncommitted *code* does
  # not travel (a provision pushes a ref) while uncommitted edits under the
  # slice dir DO (they are in the state half), so a dirty tree yields a checkout
  # nobody ever had. Oubliette refuses the same class in `handoff`; so do we.
  [ -z "$(git status --porcelain)" ] || {
    git status --short >&2
    die "the tree is dirty — commit first." \
      "Uncommitted code does not travel; uncommitted slice edits do."
  }
  if [ -z "$ref" ]; then
    ref=$(git branch --show-current) || true
    [ -n "$ref" ] || die "detached HEAD — name a ref: oubliette.sh send $N $SLOT <ref>"
  fi
  git rev-parse --verify --quiet "$ref^{commit}" >/dev/null \
    || die "no commit at '$ref' in this repo."

  # The state half is the whole point of a capsule trip; refuse to send a slice
  # whose runtime sheets were never materialised, rather than land an agent in a
  # checkout with a plan and no phase to run.
  local sheets=".doctrine/state/slice/$N/phases"
  compgen -G "$sheets/*.toml" >/dev/null \
    || die "no phase sheets at $sheets" "Run /phase-plan (doctrine slice phases $ID) first."

  say "$ID — $TITLE"
  say "$ref ($(git rev-parse --short "$ref")) -> capsule $SLOT, unit $N"

  "$CAPSULE" "$SLOT" start || true
  "$CAPSULE" "$SLOT" ssh true >/dev/null 2>&1 \
    || die "capsule $SLOT does not answer." "capsule $SLOT start   (or: just up $SLOT, in the oubliette checkout)"

  # Whatever the slot held, brought home and archived before anything is forced
  # — `handoff`'s decision 3, which is what makes the force below safe.
  local force=()
  if [ "$GEN" != - ]; then
    say "slot $SLOT holds generation $GEN — collecting it before it is overwritten"
    "$CAPSULE" "$SLOT" collect
    "$CAPSULE" "$SLOT" fetch
    archive_refs "$SLOT" "$GEN"
    # The guest's own outbound chain, which a provision does not touch and which
    # would refuse the incoming state half as a non-fast-forward, naming a cause
    # that is not the cause (NOTES item 50).
    local stage
    while read -r stage; do
      [ -n "$stage" ] || continue
      say "dropping the guest's state chain: $stage"
      "$CAPSULE" "$SLOT" ssh "git -C $GUEST_REPO update-ref -d $GUEST_STATE_REFS/$stage"
    done < <(git for-each-ref --format='%(refname:lstrip=-1)' "refs/capsule/$SLOT/gen/$GEN/state/")
    force=(--force)
  fi

  "$CAPSULE" "$SLOT" setup "$ref" \
    --unit "$N" --purpose "$ID — $TITLE" \
    --state-from-host "${force[@]}"

  cat <<EOF

$ID is in capsule $SLOT. Get in and hand it over:

  capsule $SLOT ssh
  tmux new -s $N
  cd $GUEST_REPO && claude

and the agent's first line — its handover is on the volume:

  Read .doctrine/slice/$N/handover.md first.

Bring it back with:  just back $N $SLOT
EOF
}

# --- back --------------------------------------------------------------------

cmd_back() {
  resolve_slice "${1:-}"
  resolve_slot "${2:-}"
  [ "$GEN" != - ] || die "nothing has been assigned to capsule $SLOT."

  local branch="capsule/$ID/$SLOT$GEN"
  local wt="$WORKTREES/$ID-$SLOT$GEN"
  git rev-parse --verify --quiet "refs/heads/$branch" >/dev/null \
    && die "branch '$branch' already exists — $ID generation $GEN has been landed." \
      "git worktree list | grep $ID"
  [ ! -e "$wt" ] || die "$wt already exists." "git worktree remove $wt"

  # collect + verify the exhibit against the live guest + fetch + branch +
  # report. The verify is the 2026-08-16 failure's remedy: a collect taken
  # before the agent's last commits leaves work behind and says nothing.
  "$CAPSULE" "$SLOT" land --branch "$branch"

  git worktree add "$wt" "$branch"
  say "worktree: $wt"

  # The state half. Laid out first where `adopt` insists — an empty directory,
  # checked before it is written — and then merged in by the ignore test above.
  "$CAPSULE" "$SLOT" adopt "$wt/$EXHIBIT_DIR"
  restore_ignored "$wt/$EXHIBIT_DIR/.doctrine/state/slice/$N" ".doctrine/state/slice/$N" "$wt"
  restore_ignored "$wt/$EXHIBIT_DIR/.doctrine/slice/$N"       ".doctrine/slice/$N"       "$wt"
  # `doctrine slice phases` mints this and it is ignored, so it never rides a ref.
  if [ -d "$wt/.doctrine/state/slice/$N/phases" ]; then
    ln -sfn "../../state/slice/$N/phases" "$wt/.doctrine/slice/$N/phases"
  fi

  cat <<EOF

$ID generation $GEN is home.

  branch    $branch
  worktree  $wt
  exhibit   $wt/$EXHIBIT_DIR   (whole state tree, as adopted)

  cd $wt && $DOCTRINE slice show $ID

Phase status, handover and research came back with it. The divergence and
conflict report above is against this repo's HEAD. Audit here, not on edge.
EOF
}

case "${1:-}" in
  send) shift; cmd_send "$@" ;;
  back) shift; cmd_back "$@" ;;
  *) usage ;;
esac
