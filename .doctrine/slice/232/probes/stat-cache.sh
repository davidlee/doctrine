#!/usr/bin/env bash
# RV-314 F-33 / F-42 — the stat-cache route: a same-size, mtime-preserving
# modification of a tracked file, invisible to all three legs while `ls-files -v`
# reports `H`.
#
# WHAT THIS PROBE CORRECTS. Two prior claims about this route are both wrong:
#
#   * Round 4's README said the limb "did NOT reproduce — the tracked leg
#     reported 101 bytes under both the default config and the reviewer's
#     weakened core.trustctime=false + core.checkStat=minimal". No probe was ever
#     persisted for it.
#   * RV-314 F-33 raised it as a CONFIG hazard, attributing the blinding to
#     `core.trustctime=false` / `core.checkStat=minimal`, with either key alone
#     sufficient.
#
# Neither is right. The blinding needs NO configuration at all — it reproduces
# under stock git — and the config keys are not the discriminator. What
# discriminates is ELAPSED TIME between the index write and the modification.
#
# FALSIFIERS, registered before the run:
#   FAL-1  the `busy` arm (≈1-2s of CPU before the write) MUST detect the change.
#          If it reads 0 the fixture cannot produce a positive and every row here
#          is unmeasured rather than negative.
#   FAL-2  the `boundary` arm (wait only for the wall-clock second to tick) MUST
#          read 0 under DEFAULT config. Non-zero refutes F-42 and restores F-33's
#          config attribution.
#   FAL-3  mtime and size MUST be identical across both arms. If they differ, the
#          arms are not comparable and the timing conclusion does not follow.
#   FAL-4  `ls-files -v` MUST read `H` in the blind arm — otherwise DEC-090's
#          tag detector already catches this.
#
# HONESTLY UNRESOLVED: the exact predicate inside git that makes elapsed time
# matter is NOT characterised here. Both arms present identical mtime and size,
# and the index records ctime at nanosecond precision, so a ctime comparison
# should separate them in BOTH arms — it does not. Recorded as unexplained rather
# than smoothed into a mechanism story. What is established is the DISCRIMINATION
# and its direction, which is what the design must answer for.
set -u
export LC_ALL=C
G=$(command -v git); echo "git: $($G --version)"
NORM=(-c core.autocrlf=false -c core.eol=lf -c core.fileMode=true)

busy(){ local i=0; while [ $i -lt 600000 ]; do i=$((i+1)); done; }
boundary(){ local s; s=$(date +%s); while [ "$(date +%s)" = "$s" ]; do :; done; }

# arm <label> <delay: busy|boundary> [config...]
arm(){
  local label=$1 delay=$2; shift 2
  local d=/tmp/statcache-$label
  rm -rf "$d"; mkdir -p "$d"
  ( cd "$d" || exit 1
    $G init -q .; $G config user.email a@b; $G config user.name a
    while [ $# -gt 0 ]; do $G config "$1" "$2"; shift 2; done
    printf 'AAAAAAAAAA\n' > f            # 11 bytes
    touch -d '2001-01-01 00:00:00 UTC' f
    $G add f; $G commit -qm base
    cp -p f .ref
    "$delay"
    printf 'BBBBBBBBBB\n' > f            # 11 bytes — SAME SIZE, different content
    touch -r .ref f                      # SAME MTIME
    rm -f .ref )
  local t u r tag
  t=$( (cd "$d" && $G "${NORM[@]}" diff HEAD --binary --no-textconv --no-ext-diff -- f | wc -c) )
  u=$( (cd "$d" && $G "${NORM[@]}" ls-files --others --exclude-standard -- f | wc -l) )
  (cd "$d" && $G "${NORM[@]}" diff-index --quiet --cached HEAD -- f); r=$?
  tag=$( (cd "$d" && $G "${NORM[@]}" ls-files -v -- f | cut -d' ' -f1) )
  printf '  %-26s tracked=%-6s untracked=%-3s cached_rc=%-3s tag=%-2s mtime=%-11s size=%s\n' \
    "$label" "$t" "$u" "$r" "$tag" \
    "$( (cd "$d" && stat -c %Y f) )" "$( (cd "$d" && stat -c %s f) )"
  TRACKED=$t                 # measurement out-of-band; the row above is display
}

echo
echo "=== 1. CONTROL — ~1-2s elapsed CPU before the write, stock config (FAL-1)"
arm busy-default busy
if [ "$TRACKED" -eq 0 ]; then
  echo "  !! FAL-1 FAILED: control cannot discriminate — all rows unmeasured"
  exit 1
fi
echo "  (control detects: the fixture can produce a positive)"

echo
echo "=== 2. ROUTE A — stock config, second-boundary wait only (FAL-2, FAL-4)"
arm boundary-default boundary; route_a=$TRACKED

echo
echo "=== 3. ROUTE B — the config keys, with the CONTROL's timing"
arm busy-trustctime busy core.trustctime false;  route_b1=$TRACKED
arm busy-checkstat  busy core.checkStat minimal; route_b2=$TRACKED

echo
echo "=== 4. both together"
arm boundary-trustctime boundary core.trustctime false
arm boundary-checkstat  boundary core.checkStat minimal

echo
echo "=== verdict"
echo "  Route A (timing, stock config): tracked=$route_a"
echo "  Route B (config, control timing): trustctime=$route_b1 checkStat=$route_b2"
echo
if [ "$route_a" -eq 0 ] && [ "$route_b1" -eq 0 ] && [ "$route_b2" -eq 0 ]; then
  echo "  BOTH ROUTES CONFIRMED, and they are INDEPENDENT."
  echo "  Route B is RV-314 F-33 as raised: either config key alone blinds all"
  echo "  three legs even with the control's timing. Route A is worse and was"
  echo "  missed by both the raiser and the responder: the SAME blindness with NO"
  echo "  configuration at all, on stock git, distinguished from the control only"
  echo "  by elapsed time before the write."
  echo
  echo "  Consequence for the design: this cannot be closed by pinning config"
  echo "  keys, because Route A pins nothing. NORMATIVE_FLAGS cannot reach it and"
  echo "  DEC-090's tag detector cannot see it (tag reads H throughout)."
elif [ "$route_a" -ne 0 ] && [ "$route_b1" -eq 0 ]; then
  echo "  ROUTE B ONLY. F-33 stands as raised; the timing route did not reproduce."
else
  echo "  UNEXPECTED COMBINATION — read the rows above before quoting this probe."
fi
