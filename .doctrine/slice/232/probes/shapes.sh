#!/usr/bin/env bash
# R-A obligation: enumerate the reachable entry shapes and PROBE each, before
# stating any rule. No rule is asserted by this script; it only measures.
#
# For each shape we record, for the entry as the CURRENT design would emit it:
#   RES   : does `realpath -e` on the (whole) entry succeed?   [the design's oracle]
#   LSF   : `git ls-files --error-unmatch` exit code           [step 5 contribution]
#   MATCH : what index entries the pathspec actually matched   [ground truth]
#   MODE  : index mode of the match (120000 = symlink)
#   PROBE : `git diff-index --quiet HEAD` exit (0 clean / 1 dirty / 128 abort)
#   TRUTH : is there dirty content this entry is morally about?
set -u
R=$(mktemp -d /tmp/rv307-sl232-shapes.XXXXXX)
cd "$R" || exit 1
git init -q . >/dev/null
git config user.email a@b.c; git config user.name t

mkdir -p real dir sub deep/nested
printf 'original\n' > real/target.txt
printf 'original\n' > real/second.txt
printf 'plain\n'    > dir/plain.txt
printf 'deep\n'     > deep/nested/file.txt

ln -s real/target.txt link                 # plain tracked symlink
ln -s link           chain                 # symlink -> symlink
ln -s real           linkdir               # symlink to a DIRECTORY
ln -s /etc/hostname  outlink               # symlink pointing OUTSIDE the repo
ln -s real/target.txt 'foo*'               # literal filename with a glob metachar
ln -s real/target.txt 'q?mark'             # literal filename with ?
ln -s real/target.txt 'br[a]cket'          # literal filename with [ ]
ln -s real/target.txt "$(printf 'tab\tchar')"   # literal filename with a TAB
ln -s real/target.txt 'Uniécode'      2>/dev/null || true
ln -s real/target.txt 'sparse_link'        # will get skip-worktree
printf 'skipme\n' > skipfile.txt           # regular file, will get skip-worktree
printf 'assume\n' > assumefile.txt         # regular file, will get assume-unchanged
mkdir -p ignoredroot
printf 'forced\n' > ignoredroot/forced.txt
printf 'ignoredroot/\n' > .gitignore

git add -A >/dev/null 2>&1
git add -f ignoredroot/forced.txt >/dev/null 2>&1
git commit -qm init >/dev/null

git update-index --skip-worktree sparse_link skipfile.txt
git update-index --assume-unchanged assumefile.txt
rm -f sparse_link skipfile.txt              # simulate the sparse/absent condition

# Make the CONTENT dirty. Everything a symlink points at, plus the skip/assume files.
printf 'MODIFIED\n' > real/target.txt
printf 'MODIFIED\n' > real/second.txt
printf 'MODIFIED\n' > deep/nested/file.txt
printf 'MODIFIED\n' > assumefile.txt
printf 'MODIFIED\n' > ignoredroot/forced.txt

echo "repo: $R"
echo "git: $(git --version)"
echo
printf '%-34s %-5s %-5s %-28s %-8s %-6s %s\n' "ENTRY (as declared)" "RES" "LSF" "MATCHED INDEX ENTRIES" "MODE" "PROBE" "NOTE"
printf '%s\n' "-------------------------------------------------------------------------------------------------------------------------"

probe () {
  local label="$1" spec="$2" note="${3:-}"
  local res lsf match mode pr
  # the design's resolution oracle, applied to the whole entry
  if realpath -e "${label}" >/dev/null 2>&1; then res=0; else res=1; fi
  git ls-files --error-unmatch -- "$spec" >/dev/null 2>&1; lsf=$?
  match=$(git ls-files -- "$spec" 2>/dev/null | head -3 | tr '\n' ',' | sed 's/,$//')
  mode=$(git ls-files -s -- "$spec" 2>/dev/null | head -1 | awk '{print $1}')
  git diff-index --quiet HEAD -- "$spec" >/dev/null 2>&1; pr=$?
  [ -z "$match" ] && match="(none)"
  [ -z "$mode" ]  && mode="-"
  printf '%-34s %-5s %-5s %-28s %-8s %-6s %s\n' "$label" "$res" "$lsf" "$match" "$mode" "$pr" "$note"
}

echo "== settled baseline =="
probe 'real/target.txt'      ':(literal)real/target.txt'      'plain tracked file, dirty'
probe 'link'                 ':(literal)link'                 'F-20: symlink blind'
probe 'dir/plain.txt'        ':(literal)dir/plain.txt'         'clean tracked file'

echo
echo "== F-37 routes =="
probe 'missing/../link'      ':(literal)missing/../link'      'ROUTE 1'
probe 'sparse_link'          ':(literal)sparse_link'          'ROUTE 2 (skip-worktree)'
probe 'foo*'                 ':(literal)foo*'                 'ROUTE 3 (literal *)'

echo
echo "== lexical / normalisation shapes =="
probe './link'               ':(literal)./link'               'dot component'
probe 'real/../link'         ':(literal)real/../link'         '.. with EXISTING prefix'
probe 'real//target.txt'     ':(literal)real//target.txt'     'double slash'
probe 'link/'                ':(literal)link/'                'trailing slash'
probe 'deep/nested/../nested/file.txt' ':(literal)deep/nested/../nested/file.txt' '.. round trip'

echo
echo "== index-state shapes =="
probe 'skipfile.txt'         ':(literal)skipfile.txt'         'skip-worktree REGULAR file'
probe 'assumefile.txt'       ':(literal)assumefile.txt'       'assume-unchanged, dirty'
probe 'ignoredroot/forced.txt' ':(literal)ignoredroot/forced.txt' 'E8 gitignored+tracked'

echo
echo "== glob-metachar literal filenames =="
probe 'q?mark'               ':(literal)q?mark'               'literal ?'
probe 'br[a]cket'            ':(literal)br[a]cket'            'literal [ ]'

echo
echo "== symlink topology =="
probe 'chain'                ':(literal)chain'                'symlink -> symlink'
probe 'linkdir'              ':(literal)linkdir'              'symlink -> directory'
probe 'linkdir/target.txt'   ':(literal)linkdir/target.txt'   'path THROUGH a symlinked dir'
probe 'linkdir/**'           ':(glob)linkdir/**'              'glob rooted at symlinked dir'
probe 'outlink'              ':(literal)outlink'              'symlink -> outside repo'

echo
echo "== abort / outside shapes (E13) =="
probe '../gone'              ':(literal)../gone'              'outside-shaped, non-resolving'
probe '/tmp/no-such-abs'     ':(literal)/tmp/no-such-abs'     'absolute outside'
probe "$R/real/target.txt"   ":(literal)$R/real/target.txt"   'absolute INSIDE'
probe 'nonexistent/inside.txt' ':(literal)nonexistent/inside.txt' 'non-resolving inside'
