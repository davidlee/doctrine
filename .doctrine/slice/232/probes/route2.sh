#!/usr/bin/env bash
# ROUTE 2 — sparse checkout: the tracked link is ABSENT from the working tree.
#
# CLAIM (F-37 route 2): resolution fails because the file is not on disk, yet the
# entry contributes (it is in the index) and the probe reads clean while the
# retained target is dirty.
#
# FALSIFIERS:
#   - if the link is present on disk after sparse-checkout   -> setup failed, not a route
#   - if `realpath -e link` exits 0                          -> route refuted
#   - if `ls-files --error-unmatch ':(literal)link'` != 0    -> route refuted
#   - if `diff-index --quiet HEAD -- ':(literal)link'` == 1  -> route refuted (not blind)
#   - control: retained dirty target must exit 1
set -u
R=$(mktemp -d /tmp/rv307-sl232-r2.XXXXXX)
cd "$R" || exit 1
git init -q .
git config user.email a@b.c; git config user.name t
mkdir -p real keep
printf 'original\n' > real/target.txt
printf 'kept\n'     > keep/kept.txt
ln -s ../real/target.txt keep/link          # tracked symlink INSIDE the kept cone
mkdir -p sparse_out
ln -s ../real/target.txt sparse_out/link    # tracked symlink in the EXCLUDED cone
git add -A
git commit -qm init

git sparse-checkout init --cone 2>/dev/null
git sparse-checkout set keep real 2>/dev/null

printf 'MODIFIED\n' > real/target.txt        # the retained target is dirty

echo "repo: $R"
echo "--- index vs worktree ---"
echo "index:"; git ls-files
echo "on disk:"; find . -path ./.git -prune -o -print | sort | sed 's/^/  /'
echo "skip-worktree bits:"; git ls-files -v | grep -v '^H ' || echo "  (none)"

echo
echo "--- resolution leg (sparse-excluded link) ---"
realpath -e sparse_out/link >/dev/null 2>&1; echo "realpath -e sparse_out/link                  -> exit $?  (claim: 1)"
test -e sparse_out/link;                     echo "test -e sparse_out/link                      -> exit $?  (claim: 1 = absent)"

echo
echo "--- contribution leg ---"
out=$(git ls-files --error-unmatch -- ':(literal)sparse_out/link' 2>&1); rc=$?
echo "ls-files --error-unmatch ':(literal)sparse_out/link' -> exit $rc out=[$out]  (claim: 0)"

echo
echo "--- probe leg ---"
git diff-index --quiet HEAD -- ':(literal)sparse_out/link'; echo "diff-index --quiet HEAD -- sparse_out/link   -> exit $?  (claim: 0 = CLEAN)"
git diff --quiet HEAD -- ':(literal)sparse_out/link';       echo "diff --quiet HEAD -- sparse_out/link         -> exit $?  (claim: 0 = CLEAN)"
git diff --quiet HEAD -- ':(literal)real/target.txt';       echo "diff --quiet HEAD -- real/target.txt         -> exit $?  (control: 1 = DIRTY)"
echo
echo "--- and the KEPT link (present on disk, resolves) for contrast ---"
realpath -e keep/link >/dev/null 2>&1;                echo "realpath -e keep/link                        -> exit $?  (control: 0)"
git diff --quiet HEAD -- ':(literal)keep/link';       echo "diff --quiet HEAD -- keep/link               -> exit $?  (control: 0 = blind anyway)"
