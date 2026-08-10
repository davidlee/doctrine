#!/usr/bin/env bash
# SL-248 — the two rows the off-jail audit left exposed: table A row 4
# (`BoundedFilesystemVisibility`) and row 9 (`ImmutableInputSet`).
#
# The cage the slice was measured in owns a MOUNT namespace with restricted
# binds (audit § E). A mount namespace can supply exactly the property these two
# rows attribute to the capsule, so an in-jail green is unattributable in the
# `F-48` sense: the row cannot say whether the capsule's confinement or the cage
# around it produced the reading. Neither row is exercised by `spike-deltas.sh`,
# `spike-teardown-2x2.sh` or `spike-credentials.sh`, so there was nothing to
# re-run — this is the missing instrument.
#
# What it measures, and it is NOT "does the row pass":
#   the row's CONTROL arm must FAIL. This asks whether the control arm can fail
#   *for its own reason* — whether the delta is observable at all in the
#   environment the suite runs in. A control that cannot fail is `EX-3`'s
#   vacuous pass, and that is the failure mode the cage would manufacture.
#
# Deltas are taken from production, not invented:
#   row 9  `Weakening::InputsWritable` flips FLAG_RO_BIND -> FLAG_BIND for the
#          source export and every readable entry (`bubblewrap.rs:1159`),
#          changing attachment alone: same paths, same inner destinations,
#          same count.
#   row 4  `Delta::Widened` mounts a decoy at a FIXED inner destination, because
#          a payload is a constant `Argv` built before any fixture exists and so
#          cannot be told a temporary host path (`conformance.rs:407`). The inner
#          destination here is `WIDENED_UNDECLARED`.
#
# ---------------------------------------------------------------------------
# Why this probe mostly does not write (`F-11`)
# ---------------------------------------------------------------------------
#
# The slice already paid for this lesson: the first row 9 spike literally wrote
# through each declared mount, and wrote OUTSIDE the fixture. The repair was
# `[ -w ]` — `access(2)` with `W_OK` — which reports `EROFS` for a read-only
# mount whatever the caller's identity. That separates the two attachments
# exactly and touches nothing, so it is what both rows use here. Exactly one
# real write survives, and it lands inside the fixture the way
# `the_source_export_written_through` does.
#
# Every real host path is attached `--ro-bind`. The only writable attachments
# are directories this script made with `mktemp -d`, and the floor below refuses
# to run if that ever stops being true. Nothing is signalled, nothing is killed.
set -uo pipefail

# ---------------------------------------------------------------------------
# The floor, written before the instrument that needs it.
#
# This is the first probe in this audit that writes at all, and the first to run
# UNCAGED. `notes_10-12.md:116` records the existing `va2-write` probe leaving
# files on `/nix` and `/bin`: inside the cage those are bind mounts; off-jail
# they are the real host paths. A writable attachment is therefore aimed at the
# operator's actual system unless something refuses. This refuses.
# ---------------------------------------------------------------------------
SCRATCH=$(mktemp -d) || exit 2
trap 'rm -rf "$SCRATCH"' EXIT

writable_bind() { # writable_bind <host path> -> echoes the path, or dies
  case "$1" in
    "$SCRATCH"/*) printf '%s' "$1" ;;
    *) echo "FLOOR: refused writable attachment outside the fixture: $1" >&2; exit 3 ;;
  esac
}

# `F-41`'s family, and this audit's own finding C1: an instrument that measures a
# sandbox must pin its PATH into a BOUND root, or every arm false-negatives
# through an absent binary while still exiting 0. `spike-deltas.sh` inherits the
# ambient PATH and returns a full table of `command not found` readings off-jail.
if [ -d /run/current-system/sw/bin ]; then
  SBPATH=$(dirname "$(readlink -f /run/current-system/sw/bin/id)")
else
  SBPATH="$PATH"
fi

BWRAP=$(command -v bwrap) || { echo "no bwrap"; exit 2; }
SH=/bin/sh

INNER_SOURCE=/source                     # backend.rs:84
INNER_READABLE=/readable                 # one declared readable entry
WIDENED_UNDECLARED=/widened/undeclared   # conformance.rs:417

binds=(); for d in /nix /usr /bin /lib /lib64 /etc; do
  [ -e "$d" ] && binds+=(--ro-bind "$d" "$d")
done
COMMON=("${binds[@]}" --proc /proc --dev /dev --tmpfs /tmp
        --unshare-all --setenv PATH "$SBPATH")

mkdir -p "$SCRATCH/source" "$SCRATCH/readable"
echo declared > "$SCRATCH/source/sentinel"
echo declared > "$SCRATCH/readable/sentinel"

# ---------------------------------------------------------------------------
# Provenance. This audit's finding D: an artefact that does not record the
# environment that produced it cannot later be told apart from one that did.
#
# `uid_map` is load-bearing for THIS probe specifically. The cage holds a single
# mapped uid (`1000 0 1`); off-jail the full uid range is real. Anything reading
# write permission through file OWNERSHIP therefore behaves differently in the
# two places — item 158's uid problem wearing row 9's hat. `[ -w ]` on a
# read-only mount answers `EROFS` regardless of identity, which is why it is the
# instrument; the ownership readings below are printed so the confound stays
# visible instead of silent.
# ---------------------------------------------------------------------------
echo "### provenance"
uname -srm
echo "bwrap: $BWRAP $("$BWRAP" --version 2>&1)"
for n in user pid net mnt; do printf '  ns %-5s %s\n' "$n" "$(readlink /proc/self/ns/$n)"; done
echo "  uid_map: $(tr -s ' ' < /proc/self/uid_map | tr '\n' '|')"
echo "  gid_map: $(tr -s ' ' < /proc/self/gid_map | tr '\n' '|')"
echo "  $(grep NoNewPrivs /proc/self/status)  uid=$(id -u) gid=$(id -g)"
echo "  proc entries: $(ls /proc | grep -c '^[0-9]')"
echo "  sandbox PATH: $SBPATH"
echo

# ---------------------------------------------------------------------------
# Row 9 — ImmutableInputSet.
#
# `access(2)` W_OK on the mount point, plus the identity the capsule sees, so a
# `denied` reading can be attributed to the attachment rather than to ownership.
# ---------------------------------------------------------------------------
echo "=== Row 9 — ImmutableInputSet (probe: --ro-bind; control: --bind) ==="

ACCESS_PROBE='
  echo "    seen as uid=$(id -u) gid=$(id -g)"
  for m in '"$INNER_SOURCE $INNER_READABLE"'; do
    [ -w "$m" ] && w=writable || w=EROFS-or-denied
    [ -r "$m" ] && r=readable || r=unreadable
    echo "    $m  W_OK=$w  R_OK=$r  owner=$(stat -c %u:%g "$m" 2>/dev/null || echo unknown)"
  done'

row9_arm() { # row9_arm <label> <bind flag>
  local label="$1" flag="$2" src rdb
  # The floor is consulted for the flag that actually grants writes. A ro-bind
  # of the fixture is harmless; a --bind of anything else never gets built.
  if [ "$flag" = "--bind" ]; then
    src=$(writable_bind "$SCRATCH/source") || exit 3
    rdb=$(writable_bind "$SCRATCH/readable") || exit 3
  else
    src="$SCRATCH/source"; rdb="$SCRATCH/readable"
  fi
  echo "  $label ($flag)"
  "$BWRAP" "$flag" "$src" "$INNER_SOURCE" \
           "$flag" "$rdb" "$INNER_READABLE" \
           "${COMMON[@]}" "$SH" -c "$ACCESS_PROBE" 2>&1
}

row9_arm "probe  " --ro-bind
row9_arm "control" --bind

# The one real write, and it is inside the fixture by construction — the floor
# has already refused any other target. `access(2)` says whether the kernel
# would permit; this says whether a byte actually traverses the bind, which is
# the property row 9 names. Confirmed from the HOST side, because the capsule's
# own view cannot tell a bind from a tmpfs.
echo "  the source export written through (control arm, one write, in-fixture):"
"$BWRAP" --bind "$(writable_bind "$SCRATCH/source")" "$INNER_SOURCE" "${COMMON[@]}" \
         "$SH" -c 'echo mutated > '"$INNER_SOURCE"'/written 2>/dev/null; echo "    capsule-side exit=$?"' 2>&1
if [ -f "$SCRATCH/source/written" ]; then
  echo "    host-side: the byte traversed the bind (fixture only)"
  rm -f "$SCRATCH/source/written"
else
  echo "    host-side: nothing arrived"
fi
echo "  REQUIRED: probe EROFS-or-denied, control writable. A control that is also"
echo "            denied cannot fail for its own reason — EX-3's vacuous pass here."
echo

# ---------------------------------------------------------------------------
# Row 4 — BoundedFilesystemVisibility.
#
# The control widens the mount set by one decoy. The question the cage puts is
# whether a host path OUTSIDE the cage's own binds can be widened onto at all:
# if bwrap cannot attach it, the control arm sees nothing, the row's control
# does not fail, and the row passes vacuously.
#
# Several candidate host sources, because one is a sample and the interesting
# ones are exactly those a cage is unlikely to bind. All attached READ-ONLY, and
# the payload only asks whether the path exists.
# ---------------------------------------------------------------------------
echo "=== Row 4 — BoundedFilesystemVisibility (probe: placement only; control: + decoy) ==="

VISIBILITY_PROBE='
  if [ -e '"$WIDENED_UNDECLARED"' ]; then
    echo "    '"$WIDENED_UNDECLARED"' VISIBLE  entries=$(ls -1 '"$WIDENED_UNDECLARED"' 2>/dev/null | wc -l)"
  else
    echo "    '"$WIDENED_UNDECLARED"' absent"
  fi'

echo "  probe   (no decoy)"
"$BWRAP" --ro-bind "$SCRATCH/source" "$INNER_SOURCE" "${COMMON[@]}" \
         "$SH" -c "$VISIBILITY_PROBE" 2>&1

for candidate in /etc "$SCRATCH/readable" /run/current-system "$HOME"; do
  echo "  control (decoy <- $candidate)"
  if [ ! -e "$candidate" ]; then
    echo "    host: $candidate DOES NOT EXIST here — candidate says nothing"
    continue
  fi
  out=$("$BWRAP" --ro-bind "$SCRATCH/source" "$INNER_SOURCE" \
            --ro-bind "$candidate" "$WIDENED_UNDECLARED" \
            "${COMMON[@]}" "$SH" -c "$VISIBILITY_PROBE" 2>&1)
  status=$?
  if [ $status -ne 0 ]; then
    # A bind bwrap REFUSES is the cage speaking, not the capsule. That is the
    # reading this instrument exists to catch, so it is named, never folded into
    # "absent".
    echo "    bwrap REFUSED the bind (exit=$status): ${out:-<no output>}"
  else
    echo "$out"
  fi
done
echo "  REQUIRED: probe absent, control VISIBLE. A control whose decoy cannot be"
echo "            attached, or is attached but invisible, cannot fail for its own"
echo "            reason — the cage supplied the boundedness, not the capsule."
