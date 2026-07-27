#!/usr/bin/env bash
# RV-314 round 4 disposal — F-21 probe.
#
# QUESTION: which attribute sources survive `--attr-source=<empty tree>`, and
# what neutralises each?
#
# FALSIFIER: if every row below reads tracked=0 under the "all" column, the
# proposed neutralisation set is incomplete and the disposition must say so.
# If a row reads non-zero under a narrower column, that narrower flag set is
# sufficient and the wider one is unjustified.
set -u
export LC_ALL=C
G=$(command -v git); echo "git: $($G --version)"

NORM=(-c core.autocrlf=false -c core.eol=lf -c core.fileMode=true)

mk() {                       # mk <dir> <where-the-attribute-lives>
  rm -rf "$1"; mkdir -p "$1"; ( cd "$1" || exit
    $G init -q .
    $G config user.email a@b; $G config user.name a
    # a clean filter that rewrites content to a constant
    $G config filter.canon.clean 'printf CANONICAL'
    $G config filter.canon.smudge cat
    printf 'ORIGINAL-COMMITTED-BODY\n' > f
    case "$2" in
      tree)   printf 'f filter=canon\n' > .gitattributes; $G add .gitattributes ;;
      info)   mkdir -p "$($G rev-parse --git-dir)/info"
              printf 'f filter=canon\n' > "$($G rev-parse --git-dir)/info/attributes" ;;
      config) printf 'f filter=canon\n' > "$PWD/../attrfile-$$"
              $G config core.attributesFile "$PWD/../attrfile-$$" ;;
    esac
    $G add f; $G commit -qm base
    # now the worktree carries arbitrary content the filter will canonicalise away
    printf 'ATTACKER-CONTENT-THAT-MUST-BE-VISIBLE\n' > f
  )
}

legs() {                     # legs <dir> <flags...>
  local d=$1; shift
  local t u r
  t=$( (cd "$d" && $G "${NORM[@]}" "$@" diff HEAD --binary --no-textconv --no-ext-diff 2>/dev/null | wc -c) )
  u=$( (cd "$d" && $G "${NORM[@]}" "$@" ls-files --others --exclude-standard 2>/dev/null | wc -l) )
  (cd "$d" && $G "${NORM[@]}" "$@" diff-index --quiet --cached HEAD 2>/dev/null); r=$?
  local ca
  ca=$( (cd "$d" && $G "${NORM[@]}" "$@" check-attr filter -- f 2>&1) )
  printf 'tracked=%-6s untracked=%-3s cached_rc=%-2s  %s\n' "$t" "$u" "$r" "$ca"
}

for where in tree info config; do
  d=/tmp/attrprobe-$where
  mk "$d" "$where"
  E=$( (cd "$d" && $G hash-object -t tree /dev/null) )
  echo
  echo "=== attribute source: $where   (empty-tree oid $E)"
  printf '  %-26s ' 'NORMATIVE_FLAGS only';        legs "$d"
  printf '  %-26s ' '+ --attr-source';             legs "$d" --attr-source="$E"
  printf '  %-26s ' '+ attributesFile=/dev/null';  legs "$d" -c core.attributesFile=/dev/null
  printf '  %-26s ' '+ both';                      legs "$d" --attr-source="$E" -c core.attributesFile=/dev/null
  printf '  %-26s ' '+ both + ATTR_NOSYSTEM';      GIT_ATTR_NOSYSTEM=1 legs "$d" --attr-source="$E" -c core.attributesFile=/dev/null
done

echo
echo "=== is info/attributes in the common dir or the per-worktree dir?"
d=/tmp/attrprobe-wt; mk "$d" info
( cd "$d" && $G worktree add -q ../attrprobe-wt-linked -b wt 2>/dev/null
  echo "  main   git-dir        = $($G rev-parse --git-dir)"
  echo "  main   git-common-dir = $($G rev-parse --git-common-dir)" )
( cd /tmp/attrprobe-wt-linked 2>/dev/null && \
  echo "  linked git-dir        = $($G rev-parse --git-dir)" && \
  echo "  linked git-common-dir = $($G rev-parse --git-common-dir)" && \
  printf '  linked check-attr    = %s\n' "$($G "${NORM[@]}" check-attr filter -- f 2>&1)" )

echo
echo "=== F-23: does scoping the flag off check_attr_merge_z preserve the guard?"
d=/tmp/attrprobe-merge; rm -rf $d; mkdir -p $d
( cd $d && $G init -q . && $G config user.email a@b && $G config user.name a
  printf 'f merge=ours\n' > .gitattributes; printf 'x\n' > f
  $G add .gitattributes f; $G commit -qm base
  E=$($G hash-object -t tree /dev/null)
  printf '  without flag (guard sees): %s\n' "$(printf 'f\0' | $G "${NORM[@]}" check-attr --stdin -z merge | tr '\0' ' ')"
  printf '  with    flag (guard sees): %s\n' "$(printf 'f\0' | $G "${NORM[@]}" --attr-source="$E" check-attr --stdin -z merge | tr '\0' ' ')" )

echo
echo "=== F-24: bootstrap — does deriving the oid need the flag?"
d=/tmp/attrprobe-boot
printf '  inside repo, no flag : %s (rc=%s)\n' "$( (cd $d 2>/dev/null || cd /tmp/attrprobe-tree; $G "${NORM[@]}" hash-object -t tree /dev/null) )" "$?"
printf '  outside any repo     : %s\n' "$( cd /tmp && $G hash-object -t tree /dev/null )"
printf '  bad oid, right algo  : ' ; ( cd /tmp/attrprobe-tree && $G "${NORM[@]}" --attr-source=0000000000000000000000000000000000000000 rev-parse --git-dir >/dev/null 2>&1; echo "rc=$?" )
printf '  unsupported probe    : ' ; ( cd /tmp/attrprobe-tree && E=$($G hash-object -t tree /dev/null); $G --attr-source="$E" rev-parse --git-dir >/dev/null 2>&1; echo "rc=$?" )
