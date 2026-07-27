#!/usr/bin/env bash
# ROUTE 1 — unresolved `..` alias: `missing/../link`
#
# CLAIM (F-37 route 1): resolution fails, yet the entry contributes AND the
# probe reads clean against a dirty target.
#
# FALSIFIERS (registered before the probe):
#   - if `realpath -e missing/../link` exits 0        -> route refuted (it DOES resolve)
#   - if `ls-files --error-unmatch` exits 1 or 128    -> route refuted (does not contribute / aborts)
#   - if `diff --quiet HEAD -- <alias>` exits 1       -> route refuted (probe is NOT blind)
#   - control: target path itself must exit 1 (dirty) -> else the setup is wrong, not git
set -u
R=$(mktemp -d /tmp/rv307-sl232-r1.XXXXXX)
cd "$R" || exit 1
git init -q .
git config user.email a@b.c; git config user.name t
mkdir real
printf 'original\n' > real/target.txt
ln -s real/target.txt link
git add -A
git commit -qm init
# make the target dirty
printf 'MODIFIED\n' > real/target.txt

echo "repo: $R"
echo "--- tracked entries ---"
git ls-files -s

echo
echo "--- resolution leg ---"
realpath -e missing/../link >/dev/null 2>&1; echo "realpath -e missing/../link            -> exit $?  (claim: 1)"
realpath -e link           >/dev/null 2>&1; echo "realpath -e link                       -> exit $?  (control)"

echo
echo "--- contribution leg (step 5) ---"
out=$(git ls-files --error-unmatch -- ':(literal)missing/../link' 2>&1); rc=$?
echo "ls-files --error-unmatch ':(literal)missing/../link' -> exit $rc  out=[$out]   (claim: 0, prints link)"

echo
echo "--- probe leg (is the claim dirty?) ---"
git diff --quiet HEAD -- ':(literal)missing/../link'; echo "diff --quiet HEAD -- ':(literal)missing/../link' -> exit $?  (claim: 0 = CLEAN, blind)"
git diff --quiet HEAD -- ':(literal)link';            echo "diff --quiet HEAD -- ':(literal)link'            -> exit $?  (control: 0 = clean, symlink blind)"
git diff --quiet HEAD -- ':(literal)real/target.txt'; echo "diff --quiet HEAD -- ':(literal)real/target.txt' -> exit $?  (control: 1 = DIRTY)"
git diff-index --quiet HEAD -- ':(literal)missing/../link'; echo "diff-index --quiet HEAD -- alias                -> exit $?  (claim: 0 = CLEAN)"
