#!/usr/bin/env bash
# ROUTE 3 — a `scope.paths` entry whose REAL FILENAME contains `*`.
#
# CLAIM (F-37 route 3): step 2 classifies by characters, sees the `*`, calls it a
# pattern, and skips whole-path resolution. The entry is a tracked symlink, so it
# contributes under `:(literal)` (literal magic makes `*` inert) while the probe
# is blind to the dirty target.
#
# FALSIFIERS:
#   - if `:(literal)foo*` matches nothing (exit 1)      -> route refuted
#   - if the probe on `:(literal)foo*` exits 1          -> route refuted (not blind)
#   - if step 3's whole-component-prefix rule DOES resolve it -> route refuted
#   - control: the target must be dirty (exit 1)
set -u
R=$(mktemp -d /tmp/rv307-sl232-r3.XXXXXX)
cd "$R" || exit 1
git init -q .
git config user.email a@b.c; git config user.name t
mkdir real
printf 'original\n' > real/target.txt
ln -s real/target.txt 'foo*'      # real filename IS  foo*
printf 'other\n' > foobar.txt     # a decoy that a WILDCARD reading would match
git add -A
git commit -qm init
printf 'MODIFIED\n' > real/target.txt

echo "repo: $R"
echo "--- tracked entries ---"; git ls-files -s

echo
echo "--- step 2 (shape classification by characters) ---"
echo "entry 'foo*' came from scope.paths (a CONCRETE PATH), but contains '*'"
echo "step 3 whole-component prefix of 'foo*' = ''  (no '/' before the wildcard) -> emitted unchanged, unresolved"

echo
echo "--- resolution leg, had it been attempted ---"
realpath -e 'foo*' >/dev/null 2>&1; echo "realpath -e 'foo*'                        -> exit $?  (it WOULD have resolved: 0)"
echo "resolved target: $(realpath -e 'foo*' 2>/dev/null || echo '(n/a)')"

echo
echo "--- contribution leg, as emitted (:(literal), per scope.paths) ---"
out=$(git ls-files --error-unmatch -- ':(literal)foo*' 2>&1); rc=$?
echo "ls-files --error-unmatch ':(literal)foo*'  -> exit $rc out=[$out]   (claim: 0, matches the real file)"

echo
echo "--- probe leg ---"
git diff --quiet HEAD -- ':(literal)foo*';            echo "diff --quiet HEAD -- ':(literal)foo*'      -> exit $?  (claim: 0 = CLEAN, blind)"
git diff-index --quiet HEAD -- ':(literal)foo*';      echo "diff-index --quiet HEAD -- ':(literal)foo*'-> exit $?  (claim: 0 = CLEAN)"
git diff --quiet HEAD -- ':(literal)real/target.txt'; echo "diff --quiet HEAD -- real/target.txt       -> exit $?  (control: 1 = DIRTY)"

echo
echo "--- the same string read as a GLOB, for contrast (what step 2 assumed) ---"
out=$(git ls-files -- ':(glob)foo*' 2>&1); echo "ls-files ':(glob)foo*'   -> [$(echo $out)]  (a wildcard reading matches DIFFERENT files)"
