#!/usr/bin/env bash
# RV-314 — candidate closure for the index-state bound (F-17/F-22/DEC-082) and
# for F-30's unmerged entry: one `ls-files -v` over the claim surface.
#
# FALSIFIER: if `ls-files -v` does NOT report a suppressed or unmerged entry
# distinguishably, the fourth leg does not exist and the bound stays a bound.
set -u; export LC_ALL=C; G=$(command -v git)
NORM=(-c core.autocrlf=false -c core.eol=lf -c core.fileMode=true)
d=/tmp/fourth; rm -rf $d; mkdir -p $d
cd $d
$G init -q . ; $G config user.email a@b; $G config user.name a
printf 'a\n' > a; printf 'b\n' > b; printf 'c\n' > c; printf 'd\n' > d
$G add a b c d; $G commit -qm base
$G update-index --assume-unchanged a
$G update-index --skip-worktree b
echo "=== ls-files -v over the whole surface"
$G "${NORM[@]}" ls-files -v | sed 's/^/  /'
echo
echo "=== restricted to a pathspec set (the claim surface shape)"
$G "${NORM[@]}" ls-files -v -- ':(literal)a' ':(literal)c' | sed 's/^/  /'
echo
echo "=== non-H rows only (the refusal predicate)"
$G "${NORM[@]}" ls-files -v | command grep -v '^H ' | sed 's/^/  /'
echo
echo "=== F-30: an unmerged entry — what does ls-files -v show, and cat-file?"
rm -rf /tmp/fourth-um; mkdir -p /tmp/fourth-um; cd /tmp/fourth-um
$G init -q .; $G config user.email a@b; $G config user.name a
printf 'base\n' > f; $G add f; $G commit -qm base
$G checkout -q -b other; ln -sfn target-other link; $G add link; $G commit -qm other
$G checkout -q master 2>/dev/null || $G checkout -q main
ln -sfn target-main link; $G add link; $G commit -qm main
$G merge other >/dev/null 2>&1
echo "  ls-files -s -z -- link:"; $G ls-files -s -- link | sed 's/^/    /'
echo "  ls-files -v -- link:";    $G "${NORM[@]}" ls-files -v -- link | sed 's/^/    /'
# RV-314 F-41: `$?` after a pipeline is the LAST element's status, so the
# original form here reported `tr`'s rc (always 0) for a command that exits 128 —
# pinning DEC-090's load-bearing fact backwards. Capture the rc directly rather
# than reaching for `set -o pipefail`, which would change `$?` semantics for the
# `grep -v` above (it legitimately exits 1 when every row reads H).
um_out=$($G cat-file blob :link 2>&1); um_rc=$?
printf '  cat-file blob :link -> %s (rc=%s)\n' \
  "$(printf '%s' "$um_out" | head -2 | tr '\n' ' ')" "$um_rc"
# FALSIFIER: rc MUST be non-zero (128 on git 2.54.0). A zero rc here means either
# the entry is not actually unmerged or the rc is being read from the wrong
# command — in both cases DEC-090's "closed by ordering, not classification"
# claim is unsupported by this probe.
if [ "$um_rc" -eq 0 ]; then echo "  !! FALSIFIED: expected non-zero rc"; fi
echo
echo "=== does ls-files -v flag a sparse-checkout entry?"
rm -rf /tmp/fourth-sp; mkdir -p /tmp/fourth-sp; cd /tmp/fourth-sp
$G init -q .; $G config user.email a@b; $G config user.name a
mkdir -p in out; printf 'x\n' > in/f; printf 'y\n' > out/f
$G add in out; $G commit -qm base
$G sparse-checkout set in >/dev/null 2>&1
$G "${NORM[@]}" ls-files -v | sed 's/^/  /'
