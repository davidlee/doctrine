#!/usr/bin/env bash
# SELF-ATTACK 2 — two claims of MINE, each with a pre-registered falsifier.
set -u; export LC_ALL=C; G=$(command -v git)
NORM=(-c core.autocrlf=false -c core.eol=lf -c core.fileMode=true)

echo "############ A  untracked_fingerprint's hash-object is filter-sensitive"
echo "# MY CLAIM (DEC-089): flags on observe_dirt cover every attribute-sensitive"
echo "#                     read capture() makes."
echo "# FALSIFIER: two DIFFERENT untracked files hash IDENTICALLY under a clean"
echo "#            filter via plain hash-object => the claim is false."
d=/tmp/sa-uf; rm -rf $d; mkdir -p $d
( cd $d && $G init -q . && $G config user.email a@b && $G config user.name a
  $G config filter.canon.clean 'printf CANONICAL'
  printf 'seed\n' > seed; $G add seed; $G commit -qm base
  printf '*.dat filter=canon\n' > .gitattributes; $G add .gitattributes; $G commit -qm attrs
  printf 'UNTRACKED-CONTENT-ONE\n'          > a.dat
  printf 'COMPLETELY-DIFFERENT-CONTENT-TWO\n' > b.dat
  E=$($G hash-object -t tree /dev/null)
  echo "  plain hash-object          a.dat=$($G "${NORM[@]}" hash-object -- a.dat)"
  echo "                             b.dat=$($G "${NORM[@]}" hash-object -- b.dat)"
  echo "  with --attr-source=<empty> a.dat=$($G "${NORM[@]}" --attr-source="$E" hash-object -- a.dat)"
  echo "                             b.dat=$($G "${NORM[@]}" --attr-source="$E" hash-object -- b.dat)"
  echo "  with --no-filters          a.dat=$($G "${NORM[@]}" hash-object --no-filters -- a.dat)"
  echo "                             b.dat=$($G "${NORM[@]}" hash-object --no-filters -- b.dat)"
  echo "  ls-files --others sees:    $($G "${NORM[@]}" ls-files --others --exclude-standard | tr '\n' ' ')" )

echo
echo "############ B  the XDG/global attributes file (the testable machine-local route)"
echo "# MY CLAIM (DEC-089): -c core.attributesFile=/dev/null closes machine-local config."
echo "# FALSIFIER: the DEFAULT (unset core.attributesFile => \$XDG_CONFIG_HOME/git/attributes)"
echo "#            still converts under our flag set."
d=/tmp/sa-xdg; rm -rf $d; mkdir -p $d/xdg/git
printf 'f filter=canon\n' > $d/xdg/git/attributes
( cd $d && $G init -q . && $G config user.email a@b && $G config user.name a
  $G config filter.canon.clean 'printf CANONICAL'
  printf 'ORIGINAL\n' > f; $G add f; $G commit -qm base   # blob = CANONICAL (filter live via XDG)
  printf 'ARBITRARY-ATTACKER\n' > f
  E=$($G hash-object -t tree /dev/null)
  t() { printf '    %-38s tracked=%-5s check-attr=%s\n' "$1" \
        "$(XDG_CONFIG_HOME=$d/xdg $G "${NORM[@]}" "${@:2}" diff HEAD --binary --no-textconv --no-ext-diff 2>&1 | wc -c)" \
        "$(XDG_CONFIG_HOME=$d/xdg $G "${NORM[@]}" "${@:2}" check-attr filter -- f 2>&1)"; }
  echo "    committed blob: $(XDG_CONFIG_HOME=$d/xdg $G rev-parse :f | head -c12)"
  t 'XDG attributes active, no neutralisation'
  t 'with --attr-source only'            --attr-source="$E"
  t 'with -c core.attributesFile=/dev/null' -c core.attributesFile=/dev/null
  t 'with both'                          --attr-source="$E" -c core.attributesFile=/dev/null )
