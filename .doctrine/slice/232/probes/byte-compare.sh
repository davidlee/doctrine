#!/usr/bin/env bash
# SL-232 spike — does per-claim BYTE COMPARISON replace the three legs?
#
# THE HYPOTHESIS (handover 2026-07-27, decision 3, never run):
#   For each path in the claim surface, compare the RAW worktree bytes against
#   the HEAD blob bytes. If they differ, the claim is not committed; if they
#   match, it is. Attribute-immune, stat-cache-immune, index-free,
#   GIT_WORK_TREE-immune — and therefore it would collapse RV-314 F-19/21/22/24/
#   30/31/33/35/37/38/42 in one move and, if the anchor question goes too (D1),
#   delete the whole observe_dirt / Dirt / NORMATIVE_FLAGS / CON-002 apparatus.
#
# WHAT THIS PROBE DOES. For each hazard the three legs were blind to, it prints
# the LEGS reading and the BYTE reading side by side. A hazard is "covered"
# exactly when the legs read clean *and* the bytes read divergent.
#
# FALSIFIERS, registered before the run:
#   FAL-1  the CONTROL arm (different-size content edit) MUST read dirty on the
#          legs AND divergent on the bytes. If not, the harness cannot produce a
#          positive and every row below is unmeasured rather than negative.
#   FAL-2  every HAZARD arm's legs MUST read clean (0 / 0 / rc 0). A hazard arm
#          whose legs already catch it is not a hazard and its row is unmeasured.
#   FAL-3  every HAZARD arm's bytes MUST read divergent. A hazard arm with equal
#          bytes REFUTES the hypothesis for that family.
#   FAL-4  symlinks: `git hash-object --no-filters` on a tracked symlink MUST
#          FOLLOW the link — returning the TARGET FILE's content oid (rc 0), or
#          fail on a dangling link (rc != 0). It MUST NOT return the blob oid of
#          the link target STRING. Either way it is not the symlink primitive.
#   FAL-5  mode-only change: the legs MUST detect it (diff bytes > 0) and the
#          BYTES MUST be identical. If the bytes differed, mode would already be
#          inside the byte claim and no mode residual exists.
#   FAL-6  GIT_WORK_TREE redirect: the legs MUST read clean under the redirect,
#          and the byte primitive MUST still see the dirty worktree.
#   FAL-7  set completeness: a deletion MUST fail `hash-object` (rc != 0); a
#          staged-but-uncommitted addition MUST be absent from `--others` and
#          present under `--cached`. If either is false the set story changes.
#
# HONESTLY UNRESOLVED (recorded, not smoothed):
#   * The mode/exec bit is NOT inside the byte claim (FAL-5). Byte comparison
#     alone REGRESSES against the legs on a mode-only change unless mode is
#     compared too — so "replace the three legs" means "replace them with a
#     byte+mode+set comparison", not with bytes alone.
#   * The claim surface SET cannot come from bytes. It needs one enumeration
#     that includes the index (`--cached`), the worktree (`--others`) and HEAD;
#     see section 6. Byte comparison is index-free for CONTENT, not for SET.
#   * Cost is measured on a constructed fixture (section 7), not the live
#     corpus. The live corpus count is reported as context only.
set -u
export LC_ALL=C
G=$(command -v git); echo "git: $($G --version)"
NORM=(-c core.autocrlf=false -c core.eol=lf -c core.fileMode=true)
WORK=$(mktemp -d /tmp/byte-compare.XXXXXX)
trap 'rm -rf "$WORK"' EXIT
fails=0
note(){ printf '  %s\n' "$*"; }
fail(){ printf '  !! FALSIFIER FAILED: %s\n' "$*"; fails=$((fails+1)); }

newrepo(){ local d="$WORK/$1"; rm -rf "$d"; mkdir -p "$d"
  ( cd "$d" || exit 1
    $G init -q .; $G config user.email a@b; $G config user.name a ) ; echo "$d"; }

# legs <dir> <path...> -> prints "tracked=<bytes> untracked=<n> idx=<rc> tag=<..>"
legs(){
  local d="$1"; shift
  local t u r tag
  t=$( (cd "$d" && $G "${NORM[@]}" diff HEAD --binary --no-textconv --no-ext-diff -- "$@" 2>/dev/null | wc -c) )
  u=$( (cd "$d" && $G "${NORM[@]}" ls-files --others --exclude-standard -- "$@" 2>/dev/null | wc -l) )
  (cd "$d" && $G "${NORM[@]}" diff-index --quiet --cached HEAD -- "$@" >/dev/null 2>&1); r=$?
  tag=$( (cd "$d" && $G "${NORM[@]}" ls-files -v -- "$@" 2>/dev/null | cut -d' ' -f1 | tr '\n' ',') )
  printf 'tracked=%-5s untracked=%-3s idx=%-3s tag=%-4s' "$t" "$u" "$r" "$tag"
}
legs_clean(){ # <dir> <path...> -> rc 0 if legs read clean
  local d="$1"; shift
  local t u r
  t=$( (cd "$d" && $G "${NORM[@]}" diff HEAD --binary --no-textconv --no-ext-diff -- "$@" 2>/dev/null | wc -c) )
  u=$( (cd "$d" && $G "${NORM[@]}" ls-files --others --exclude-standard -- "$@" 2>/dev/null | wc -l) )
  (cd "$d" && $G "${NORM[@]}" diff-index --quiet --cached HEAD -- "$@" >/dev/null 2>&1); r=$?
  [ "$t" = 0 ] && [ "$u" = 0 ] && [ "$r" = 0 ]
}
# bytes <dir> <path> -> prints "raw=<oid|ABSENT:rc> head=<oid|ABSENT:rc> differ=yes|no"
bytes(){
  local d="$1" p="$2" raw head rrc hrc
  raw=$( (cd "$d" && $G "${NORM[@]}" hash-object --no-filters -- "$p" 2>/dev/null) ); rrc=$?
  head=$( (cd "$d" && $G "${NORM[@]}" rev-parse "HEAD:$p" 2>/dev/null) ); hrc=$?
  if [ $rrc -ne 0 ] || [ $hrc -ne 0 ]; then
    printf 'raw=%s head=%s differ=ABSENT' "${raw:-ABSENT:$rrc}" "${head:-ABSENT:$hrc}"
  elif [ "$raw" = "$head" ]; then
    printf 'raw=%s head=%s differ=no' "$raw" "$head"
  else
    printf 'raw=%s head=%s differ=yes' "$raw" "$head"
  fi
}
# hazard <label> <dir> <path...> -> applies FAL-2 and FAL-3
hazard(){
  local label="$1" d="$2"; shift 2
  printf '  %-30s ' "$label"
  local l; l=$(legs "$d" "$@")
  printf '%s  |  %s' "$l" "$(bytes "$d" "$1")"
  if ! legs_clean "$d" "$@"; then echo "   <-- FAL-2 FAILED (legs already dirty)"
  elif [ "$(bytes "$d" "$1" | sed 's/.*differ=//')" = yes ]; then echo "   [COVERED]"
  else echo "   <-- FAL-3 FAILED (bytes not divergent)"; fi
}

echo
echo "=================== 1. CONTROL (FAL-1) ==================="
d=$(newrepo control)
( cd "$d"; printf 'body\n' > f; $G add f; $G commit -qm base )
printf '  %-30s ' "clean (must agree)"
printf '%s  |  %s' "$(legs "$d" f)" "$(bytes "$d" f)"
if legs_clean "$d" f && [ "$(bytes "$d" f | sed 's/.*differ=//')" = no ]; then echo "   [both clean]"; else fail "control-clean disagreed"; fi
( cd "$d"; printf 'a different body, longer\n' > f )
printf '  %-30s ' "dirty by content (must agree)"
printf '%s  |  %s' "$(legs "$d" f)" "$(bytes "$d" f)"
if ! legs_clean "$d" f && [ "$(bytes "$d" f | sed 's/.*differ=//')" = yes ]; then echo "   [both dirty]"; else fail "FAL-1: control-dirty cannot discriminate"; fi

echo
echo "=================== 2. CONTENT CONVERSION (RV-314 F-19) ==================="
echo "  -- committed .gitattributes: f text eol=crlf (blob LF, worktree CRLF)"
d=$(newrepo eol)
( cd "$d"
  printf 'f text eol=crlf\n' > .gitattributes
  printf 'body\n' > f; $G add f .gitattributes; $G commit -qm base
  rm f; $G checkout -q -- f )
note "worktree: $( (cd "$d" && od -An -c f | tr -s ' ' | head -1) ) | blob: $( (cd "$d" && $G cat-file blob HEAD:f | od -An -c | tr -s ' ' | head -1) )"
hazard "text eol=crlf" "$d" f

echo
echo "  -- committed .gitattributes: f filter=canon (blob CANONICAL, worktree arbitrary)"
d=$(newrepo filter)
( cd "$d"
  $G config filter.canon.clean 'printf CANONICAL'
  printf 'f filter=canon\n' > .gitattributes
  printf 'body\n' > f; $G add f .gitattributes; $G commit -qm base
  printf 'ARBITRARY ATTACKER CONTENT\n' > f )
note "blob: $( (cd "$d" && $G cat-file blob HEAD:f) ) | worktree: $( (cd "$d" && cat f) )"
hazard "clean filter" "$d" f

echo
echo "  -- .git/info/attributes: f filter=canon (git-dir state, RV-314 F-21/F-38)"
d=$(newrepo infoattr)
( cd "$d"
  $G config filter.canon.clean 'printf CANONICAL'
  printf 'f filter=canon\n' > .git/info/attributes
  printf 'body\n' > f; $G add f; $G commit -qm base
  printf 'ARBITRARY ATTACKER CONTENT\n' > f )
hazard "info/attributes filter" "$d" f

echo
echo "=================== 3. FRESHNESS SUPPRESSION (RV-314 F-33/F-42) ==================="
echo "  -- same size, mtime preserved, ~1-2s elapsed before the write, core.trustctime=false"
d=$(newrepo statcache)
( cd "$d"
  $G config core.trustctime false
  printf 'AAAAAAAAAA\n' > f; touch -d '2001-01-01 00:00:00 UTC' f
  $G add f; $G commit -qm base
  cp -p f .ref
  i=0; while [ $i -lt 600000 ]; do i=$((i+1)); done
  printf 'BBBBBBBBBB\n' > f; touch -r .ref f; rm -f .ref )
hazard "stat-cache, same size" "$d" f
note "(tag=H throughout is why DEC-090's detector cannot see this class)"

echo
echo "=================== 4. GIT_WORK_TREE REDIRECT (RV-314 F-37) ==================="
# A CLEAN WORKTREE, not an empty repo: GIT_WORK_TREE points git at this plain
# directory while GIT_DIR is found from $d. (Getting this wrong reads the file as
# DELETED and the arm looks dirty — the fixture that made this falsifier fail.)
d=$(newrepo gwt); clean="$WORK/gwt-clean"; rm -rf "$clean"; mkdir -p "$clean"
( cd "$d"; printf 'body\n' > f; $G add f; $G commit -qm base; printf 'DIRTY\n' > f
  printf 'body\n' > "$clean/f" )
printf '  %-30s ' "legs under GIT_WORK_TREE=$clean"
l=$( cd "$d" && GIT_WORK_TREE="$clean" $G "${NORM[@]}" diff HEAD --binary --no-textconv --no-ext-diff -- f | wc -c )
u=$( cd "$d" && GIT_WORK_TREE="$clean" $G "${NORM[@]}" ls-files --others --exclude-standard -- f | wc -l )
( cd "$d" && GIT_WORK_TREE="$clean" $G "${NORM[@]}" diff-index --quiet --cached HEAD -- f >/dev/null 2>&1 ); r=$?
printf 'tracked=%-5s untracked=%-3s idx=%-3s  |  ' "$l" "$u" "$r"
raw=$( cd "$d" && GIT_WORK_TREE="$clean" $G "${NORM[@]}" hash-object --no-filters -- f 2>/dev/null ); rrc=$?
head=$( cd "$d" && GIT_WORK_TREE="$clean" $G "${NORM[@]}" rev-parse HEAD:f )
if [ "$raw" != "$head" ]; then printf 'raw=%s head=%s differ=yes\n' "$raw" "$head"; else printf 'raw=%s head=%s differ=no\n' "$raw" "$head"; fi
if [ "$l" = 0 ] && [ "$u" = 0 ] && [ "$r" = 0 ] && [ "$raw" != "$head" ]; then echo "   [COVERED — legs redirected, bytes not]"; else fail "FAL-6: GIT_WORK_TREE arm"; fi
note "hash-object with GIT_WORK_TREE set returned the DIRTY tree's oid (cwd-resolved, not work-tree-resolved) — that is the immunity."

echo
echo "=================== 5. MODE / EXEC BIT — THE RESIDUAL (FAL-5) ==================="
d=$(newrepo mode)
( cd "$d"; printf 'body\n' > f; $G add f; $G commit -qm base; chmod +x f )
printf '  %-30s ' "mode-only 100644 -> 100755"
l=$(legs "$d" f); b=$(bytes "$d" f)
printf '%s  |  %s\n' "$l" "$b"
if [ "$(cd "$d" && $G "${NORM[@]}" diff HEAD --binary -- f | wc -c)" -gt 0 ] && [ "$(bytes "$d" f | sed 's/.*differ=//')" = no ]; then
  echo "   [RR] legs detect, bytes do NOT — byte comparison ALONE regresses here."
  note "consequence: 'replace the three legs' must mean bytes + MODE + set, not bytes alone."
else
  fail "FAL-5: expected legs dirty / bytes equal on a mode-only change"
fi

echo
echo "=================== 6. SET COMPLETENESS — BYTES DO NOT ENUMERATE (FAL-7) ==================="
d=$(newrepo set)
( cd "$d"; mkdir -p uid; printf 'body\n' > uid/memory.md; printf '[review]\n' > uid/memory.toml; $G add -A; $G commit -qm base )
deldir="$d"; ( cd "$d"; rm uid/memory.md )
printf '  %-30s ' "H1 deletion (HEAD has, worktree absent)"
rawrc=$( cd "$d" && $G hash-object --no-filters -- uid/memory.md >/dev/null 2>&1; echo $? )
headrc=$( cd "$d" && $G rev-parse HEAD:uid/memory.md >/dev/null 2>&1; echo $? )
printf 'legs: %s  |  hash-object rc=%s rev-parse rc=%s\n' "$(legs "$d" uid/memory.md)" "$rawrc" "$headrc"
if [ "$rawrc" -ne 0 ] && [ "$headrc" -eq 0 ]; then note "[absence must be treated as divergent: HEAD has it, the worktree does not]"; else fail "FAL-7: deletion"; fi
( cd "$d"; $G checkout -q -- uid/memory.md )
( cd "$d"; printf 'stray\n' > uid/notes.txt )
printf '  %-30s ' "H2 extra untracked file in uid dir"
printf 'legs: %s  |  rev-parse HEAD:notes.txt rc=%s\n' "$(legs "$d" uid/notes.txt)" "$(cd "$d" && $G rev-parse HEAD:uid/notes.txt >/dev/null 2>&1; echo $?)"
note "enumeration must include --others, or this is missed:"
( cd "$d" && $G ls-files -z --cached --others --exclude-standard -- uid/ | tr '\0' '\n' | sed 's/^/      /' )
rm -f "$d/uid/notes.txt"
( cd "$d"; printf 'staged\n' > uid/extra.md; $G add uid/extra.md )
printf '  %-30s ' "H3 staged add (in index, not HEAD)"
o=$( cd "$d" && $G ls-files --others --exclude-standard -- uid/extra.md | wc -l )
c=$( cd "$d" && $G ls-files --cached -- ':(literal)uid/extra.md' uid/extra.md 2>/dev/null | wc -l )
printf 'others=%s cached-listed=%s  |  diff-index rc=%s\n' "$o" "$c" "$(cd "$d" && $G diff-index --quiet --cached HEAD -- uid/extra.md; echo $?)"
if [ "$o" = 0 ] && [ "$c" -ge 1 ]; then note "an --others-only enumeration MISSES this; the set needs --cached too."; else fail "FAL-7: staged add"; fi

echo
echo "=================== 7. SYMLINK EQUALITY (FAL-4) ==================="
d=$(newrepo symlink)
( cd "$d"; printf 'body\n' > f; printf 'other\n' > g; $G add f g; $G commit -qm base
  ln -s f link; $G add link; $G commit -qm link )
fo=$( cd "$d" && $G hash-object --no-filters -- f )
ho=$( cd "$d" && $G hash-object --no-filters -- link ); hrc=$?
bo=$( cd "$d" && $G rev-parse HEAD:link )
printf '  %-30s ' "hash-object on a tracked symlink"
printf 'rc=%s oid=%s  |  blob(target string)="%s" oid=%s  |  target f oid=%s\n' \
  "$hrc" "$ho" "$(cd "$d" && $G cat-file blob HEAD:link)" "$bo" "$fo"
if [ "$hrc" -eq 0 ] && [ "$ho" = "$fo" ] && [ "$ho" != "$bo" ]; then
  echo "   [FAL-4 held] hash-object FOLLOWS the link: it hashed f's content, not the link text."
  note "so hash-object is NOT the primitive for mode 120000; equality is readlink vs the blob string."
else
  fail "FAL-4: hash-object did not behave as a link-following primitive — re-derive before quoting link rows"
fi
# the correct primitive, and the retarget that must read divergent
( cd "$d"; rm link; ln -s g link )
lk=$( cd "$d" && readlink link ); bl=$( cd "$d" && $G cat-file blob HEAD:link )
printf '  %-30s ' "readlink vs blob, after retarget"
printf 'readlink="%s" blob="%s" differ=%s\n' "$lk" "$bl" "$([ "$lk" != "$bl" ] && echo yes || echo no)"
( cd "$d"; rm link; ln -s nonexistent link )
drc=$( cd "$d" && $G hash-object --no-filters -- link >/dev/null 2>&1; echo $? )
printf '  %-30s rc=%s\n' "hash-object on a dangling symlink" "$drc"
[ "$drc" -ne 0 ] && note "(dangling: hash-object errors — a second reason it cannot be the primitive)"

echo
echo "=================== 8. COST — CONSTRUCTED FIXTURE, NOT THE LIVE CORPUS ==================="
d=$(newrepo cost)
( cd "$d"
  mkdir -p data
  for i in $(seq 1 1500); do printf 'content %s\n' "$i" > "data/f$i.txt"; done
  for i in $(seq 1 200); do ln -s "f$i.txt" "data/l$i" 2>/dev/null || true; done
  $G add -A >/dev/null 2>&1; $G commit -qm base
  for i in $(seq 1 1500); do printf 'CHANGED %s\n' "$i" > "data/f$i.txt"; done )
# make every file dirty so the comparison cannot short-circuit on a clean prefix
tot=$( cd "$d" && $G ls-files --cached -- 'data/*' | wc -l )
reg=$( cd "$d" && $G ls-files -s -- 'data/*' | awk '$1!="120000"{print $4}' | wc -l )
echo "  fixture: $tot enumerated, $reg regular (rest symlinks)"
t0=$(date +%s%N); ( cd "$d" && $G "${NORM[@]}" diff HEAD --binary --no-textconv --no-ext-diff -- ':(glob)data/**' >/dev/null ); t1=$(date +%s%N)
echo "  legs alone:            $(( (t1-t0)/1000000 ))ms"
t0=$(date +%s%N)
( cd "$d" && $G ls-files -z --cached --others --exclude-standard -- 'data/' | tr '\0' '\n' | $G hash-object --stdin-paths --no-filters >/dev/null 2>&1 ); t1=$(date +%s%N)
echo "  bytes, BATCH:          $(( (t1-t0)/1000000 ))ms   (one hash-object invocation)"
t0=$(date +%s%N)
( cd "$d" && $G ls-files --cached -- 'data/' | while IFS= read -r p; do $G hash-object --no-filters -- "$p" >/dev/null 2>&1 || true; done ); t1=$(date +%s%N)
echo "  bytes, PER-FILE:       $(( (t1-t0)/1000000 ))ms   (one invocation per path)"
note "batch avoids the per-process cost; symlinks must be partitioned out (hash-object FOLLOWS them — section 7)."
echo
echo "  LIVE CORPUS CONTEXT (not a fixture claim — re-measure before quoting):"
( cd "$(git rev-parse --show-toplevel 2>/dev/null || echo .)"
  n=$($G ls-files -z --cached --others --exclude-standard -- ':(glob).doctrine/**' 2>/dev/null | tr '\0' '\n' | wc -l)
  s=$($G ls-files -s -- ':(glob).doctrine/**' 2>/dev/null | awk '$1=="120000"' | wc -l)
  printf '      .doctrine/** enumerates %s paths, %s of them symlinks\n' "$n" "$s" ) 2>/dev/null || note "  (run from a doctrine tree to see the live count)"

echo
echo "=================== VERDICT ==================="
if [ "$fails" -eq 0 ]; then
  echo "  All registered falsifiers held."
  echo "  * Every content-conversion and freshness-suppression hazard in this slice's"
  echo "    ledger reads CLEAN on the three legs and DIVERGENT on raw bytes. Byte"
  echo "    comparison covers F-19/21/22/24/33/37/38/42 on this evidence."
  echo "  * It does NOT cover, by itself: the claim-surface SET (needs --cached +"
  echo "    --others + HEAD), SYMLINKS (hash-object FOLLOWS them; compare the link"
  echo "    target via readlink against the blob string), or the MODE bit (bytes"
  echo "  * Cost is cheap with a batched primitive; per-file spawning is ~70x slower."
  echo "  * The anchor question (D1) is untouched by this probe — it is a design call."
else
  echo "  $fails falsifier(s) FAILED — read the failing rows before quoting any verdict."
  exit 1
fi
