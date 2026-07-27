#!/usr/bin/env bash
# RV-314 round 4 disposal — F-22 probe, plus the residues probe 1 could not test.
#
# QUESTION 1: does `-c core.fsmonitor=false` restore the three legs on an index
# ALREADY PRIMED by an fsmonitor-enabled status? (Priming is the load-bearing
# half: if the FSMONITOR_VALID bits persist in the index, turning the config off
# at read time does not help and the repair must be something else.)
# FALSIFIER: tracked=0 under the neutralised column ⇒ config neutralisation is
# not the repair.
#
# QUESTION 2: does GIT_ATTR_NOSYSTEM actually suppress a system attributes file,
# and is a nonexistent core.attributesFile path as good as /dev/null?
set -u
export LC_ALL=C
G=$(command -v git)
NORM=(-c core.autocrlf=false -c core.eol=lf -c core.fileMode=true)

legs() { local d=$1; shift
  local t u r
  t=$( (cd "$d" && $G "${NORM[@]}" "$@" diff HEAD --binary --no-textconv --no-ext-diff 2>/dev/null | wc -c) )
  u=$( (cd "$d" && $G "${NORM[@]}" "$@" ls-files --others --exclude-standard 2>/dev/null | wc -l) )
  (cd "$d" && $G "${NORM[@]}" "$@" diff-index --quiet --cached HEAD 2>/dev/null); r=$?
  local s; s=$( (cd "$d" && $G "${NORM[@]}" "$@" status --porcelain 2>/dev/null | tr '\n' ';') )
  printf 'tracked=%-5s untracked=%-3s cached_rc=%-2s status=[%s]\n' "$t" "$u" "$r" "$s"
}

echo "git: $($G --version)"
echo
echo "=== Q1: core.fsmonitor blinding, and whether config neutralisation lifts it"
d=/tmp/fresh-fsm; rm -rf $d; mkdir -p $d
( cd $d && $G init -q . && $G config user.email a@b && $G config user.name a
  cat > hook.sh <<'EOF'
#!/bin/sh
printf 'stable-token\0'
EOF
  chmod +x hook.sh
  printf 'ELEVEN-BYTE\n' > f
  $G add f; $G commit -qm base
  $G config core.fsmonitor "$PWD/hook.sh"
  $G status --porcelain >/dev/null            # prime the index
  printf 'FORTY-FIVE-BYTES-OF-COMPLETELY-UNRELATED-STUFF\n' > f
  echo "  ls-files -v (non-H rows ⇒ a DEC-082 flag is set):"
  $G ls-files -v | sed 's/^/    /'
  echo "  index blob: $($G rev-parse :f)  worktree bytes: $(wc -c < f)" )
printf '  %-30s ' 'fsmonitor active';            legs $d
printf '  %-30s ' '-c core.fsmonitor=false';     legs $d -c core.fsmonitor=false
printf '  %-30s ' '-c core.fsmonitor= (empty)';  legs $d -c core.fsmonitor=

echo
echo "=== Q1b: does the priming survive? re-prime, then neutralise again"
( cd $d && $G status --porcelain >/dev/null )
printf '  %-30s ' 're-primed, then neutralised'; legs $d -c core.fsmonitor=false

echo
echo "=== Q1c: untracked cache — can it hide an untracked claim path?"
d=/tmp/fresh-uc; rm -rf $d; mkdir -p $d
( cd $d && $G init -q . && $G config user.email a@b && $G config user.name a
  printf 'x\n' > f; $G add f; $G commit -qm base
  $G config core.untrackedCache true
  $G config core.fsmonitor "$PWD/../fresh-fsm/hook.sh"
  $G status --porcelain >/dev/null
  mkdir -p d && printf 'new\n' > d/new.txt )
printf '  %-30s ' 'uc+fsmonitor active';         legs $d
printf '  %-30s ' 'neutralised';                 legs $d -c core.fsmonitor=false -c core.untrackedCache=false

echo
echo "=== Q2: GIT_ATTR_NOSYSTEM and the attributesFile spelling"
d=/tmp/fresh-attr; rm -rf $d; mkdir -p $d
( cd $d && $G init -q . && $G config user.email a@b && $G config user.name a
  $G config filter.canon.clean 'printf CANONICAL'
  printf 'ORIGINAL\n' > f; $G add f; $G commit -qm base
  printf 'ATTACKER\n' > f
  printf 'f filter=canon\n' > "$PWD/globalattrs"
  $G config core.attributesFile "$PWD/globalattrs" )
printf '  %-30s ' 'attributesFile set';              legs $d
printf '  %-30s ' '=/dev/null';                      legs $d -c core.attributesFile=/dev/null
printf '  %-30s ' '=<nonexistent path>';             legs $d -c core.attributesFile=/nonexistent/attrs
printf '  %-30s ' '= (empty value)';                 legs $d -c core.attributesFile=
echo "  system attributes file git would read: $($G var GIT_ATTR_SYSTEM 2>&1 | head -1)"

echo
echo "=== Q3: what does a conversion-bearing info/attributes look like to a cheap reader?"
echo "  (the only surface left; no flag reaches it — probe 1 measured that)"
d=/tmp/attrprobe-info
if [ -d $d ]; then
  echo "  \$GIT_COMMON_DIR/info/attributes contents:"
  sed 's/^/    /' "$d/.git/info/attributes"
fi
