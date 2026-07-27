#!/usr/bin/env bash
# The residue: what the candidate does NOT close, plus the remaining R-A shapes
# (core.quotePath, core.ignoreCase, control chars, the abort guard, glob field).
set -u
R=$(mktemp -d /tmp/rv307-sl232-res.XXXXXX)
cd "$R" || exit 1
git init -q .; git config user.email a@b.c; git config user.name t
mkdir -p real linkdirtarget
printf 'original\n' > real/target.txt
printf 'original\n' > linkdirtarget/inner.txt
ln -s linkdirtarget linkdir
ln -s real/target.txt link
printf 'sk\n' > skipfile.txt
printf 'au\n' > assumefile.txt
printf 'unicode\n' > 'ünï.txt'
printf 'case\n'    > 'CaseFile.txt'
git add -A >/dev/null; git commit -qm init >/dev/null
git update-index --skip-worktree skipfile.txt
git update-index --assume-unchanged assumefile.txt
printf 'MODIFIED\n' > real/target.txt
printf 'MODIFIED\n' > linkdirtarget/inner.txt
printf 'MODIFIED\n' > assumefile.txt
printf 'MODIFIED\n' > skipfile.txt

echo "repo: $R"; echo
echo "### (a) FAL-5 residue — are the suppressing index bits DETECTABLE?"
echo "  git ls-files -v (non-'H' rows are flagged entries):"
git ls-files -v | grep -v '^H ' | sed 's/^/    /'
echo "  -> 'S' = skip-worktree, 'h' = assume-unchanged. Both are visible on the row."
echo "  probe on each (content dirty on disk):"
for f in skipfile.txt assumefile.txt; do
  git diff-index --quiet HEAD -- ":(literal)$f"; echo "    diff-index $f -> exit $? (0 = suppressed/clean)"
done
echo

echo "### (b) FAL-4 — ancestor walk: can an index-only walk recover linkdir/inner.txt?"
entry='linkdir/inner.txt'
echo "  direct expansion:"
git ls-files -s -- ":(literal)$entry" | sed 's/^/    /'; echo "    (empty = matches nothing)"
echo "  ancestor walk over index entries:"
anc="$entry"
while [ "$anc" != "." ] && [ "$anc" != "/" ]; do
  anc=$(dirname "$anc")
  [ "$anc" = "." ] && break
  row=$(git ls-files -s -- ":(literal)$anc" 2>/dev/null | head -1)
  if [ -n "$row" ]; then
    mode=${row%% *}
    echo "    ancestor '$anc' IS an index entry, mode=$mode"
    if [ "$mode" = 120000 ]; then
      tgt=$(git cat-file blob ":$anc")
      rest=${entry#$anc/}
      echo "    -> link target [$tgt]; rewritten entry = $tgt/$rest"
      git diff-index --quiet HEAD -- ":(literal)$tgt/$rest"; echo "    -> probe on rewritten: exit $? (1 = DIRTY, recovered)"
    fi
  fi
done
echo

echo "### (c) abort guard — can we prevent exit 128 LEXICALLY, before git sees it?"
for e in '../gone' '/tmp/no-such-abs' 'nonexistent/inside.txt' "$R/real/target.txt"; do
  git ls-files --error-unmatch -- ":(literal)$e" >/dev/null 2>&1; rc=$?
  case "$e" in
    /*) if [ "${e#$R/}" != "$e" ]; then guard="INSIDE (rewrite to ${e#$R/})"; else guard="REJECT pre-emission (absolute outside)"; fi ;;
    ../*) guard="REJECT pre-emission (escapes root lexically)" ;;
    *) guard="emit" ;;
  esac
  printf '  %-46s git exit=%-4s lexical guard: %s\n' "$e" "$rc" "$guard"
done
echo

echo "### (d) core.quotePath — does it corrupt parsed output? does -z defeat it?"
git config core.quotePath true
echo -n "  ls-files (quotePath=true):  "; git ls-files -- ':(literal)ünï.txt'
echo -n "  ls-files -z (quotePath=true): "; git ls-files -z -- ':(literal)ünï.txt' | tr '\0' '\n'
echo -n "  ls-files -s -z:               "; git ls-files -s -z -- ':(literal)ünï.txt' | tr '\0' '\n'
git config core.quotePath false
echo

echo "### (e) core.ignoreCase — does pathspec matching become case-insensitive?"
for v in false true; do
  git config core.ignoreCase $v
  git ls-files --error-unmatch -- ':(literal)casefile.txt' >/dev/null 2>&1
  echo "  core.ignoreCase=$v : ls-files ':(literal)casefile.txt' (real name CaseFile.txt) -> exit $?"
done
git config core.ignoreCase false
echo

echo "### (f) control chars (F-38) at the argv boundary"
printf '  newline entry -> '; git ls-files --error-unmatch -- ":(literal)$(printf 'a\nb')" >/dev/null 2>&1; echo "git exit $? (reaches git)"
python3 - <<'PY'
import subprocess
try:
    subprocess.run(['git','ls-files','--','\x00'],capture_output=True)
    print('  NUL entry     -> git ran (unexpected)')
except ValueError as e:
    print(f'  NUL entry     -> {type(e).__name__}: {e} — NO git process exists, so NO exit code')
PY
echo
echo "### (g) glob-field entry rooted at a symlinked dir, under index-first"
git ls-files -s -- ':(glob)linkdir/**' | sed 's/^/    direct: /'; echo "    (empty = matches nothing)"
git ls-files -s -- ':(glob)linkdirtarget/**' | sed 's/^/    resolved-root: /'
