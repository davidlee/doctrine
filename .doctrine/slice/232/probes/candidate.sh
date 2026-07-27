#!/usr/bin/env bash
# CANDIDATE RULE — "index-first": never consult the filesystem at all.
#   1. emit the entry as declared, magic-prefixed by FIELD OF ORIGIN (no char sniffing)
#   2. expand it against the INDEX:  git ls-files -s -- <spec>
#      empty -> non-contributing (E7).  non-empty -> contributes.
#   3. for every matched entry of mode 120000, read the link target FROM THE INDEX
#      BLOB, join it lexically to the link's parent, and re-emit. Recurse, bounded.
#
# FALSIFIERS, registered before running:
#   FAL-1  `ls-files -s` must report 120000 for symlinks         -> else no discriminator
#   FAL-2  `cat-file blob :<path>` must work for a skip-worktree
#          entry whose file is ABSENT from the worktree          -> else route 2 unfixable
#   FAL-3  adding the resolved target must flip the probe to
#          DIRTY on all three F-37 routes                        -> else the candidate fails F-37
#   FAL-4  a path THROUGH a symlinked dir (linkdir/target.txt)
#          matches nothing -> does the ancestor walk recover it?  (if not: state as boundary)
#   FAL-5  assume-unchanged / skip-worktree REGULAR files read
#          clean under ANY pathspec -> candidate CANNOT close
#          these; must be declared, not claimed total.
set -u
R=$(mktemp -d /tmp/rv307-sl232-cand.XXXXXX)
cd "$R" || exit 1
git init -q .; git config user.email a@b.c; git config user.name t
mkdir -p real sparse_out
printf 'original\n' > real/target.txt
ln -s real/target.txt link
ln -s link            chain
ln -s real            linkdir
ln -s real/target.txt 'foo*'
ln -s ../real/target.txt sparse_out/slink
printf 'assume\n' > assumefile.txt
git add -A >/dev/null; git commit -qm init >/dev/null
git update-index --skip-worktree sparse_out/slink
rm -f sparse_out/slink
git update-index --assume-unchanged assumefile.txt
printf 'MODIFIED\n' > real/target.txt
printf 'MODIFIED\n' > assumefile.txt

echo "repo: $R"; echo

echo "### FAL-1 — does ls-files -s expose the symlink mode?"
git ls-files -s | sed 's/^/  /'
echo

echo "### FAL-2 — can we read a link target from the index when the file is ABSENT on disk?"
echo -n "  worktree present? "; test -e sparse_out/slink && echo yes || echo "NO (absent)"
echo -n "  cat-file blob :sparse_out/slink -> "
if t=$(git cat-file blob :sparse_out/slink 2>&1); then echo "OK  target=[$t]"; else echo "FAILED: $t"; fi
echo

# --- the candidate, implemented ---
# lexical normalise: collapse . and .., no filesystem access. Returns "" if it escapes root.
norm () {
  local p="$1"; local -a out=(); local c
  local IFS=/
  for c in $p; do
    case "$c" in
      ''|.) ;;
      ..) [ ${#out[@]} -eq 0 ] && { echo ""; return 1; }; unset 'out[${#out[@]}-1]'; out=("${out[@]}") ;;
      *) out+=("$c") ;;
    esac
  done
  echo "${out[*]}"
}

expand () {                       # $1 = entry text, $2 = field (paths|globs)
  local entry="$1" field="$2"
  local magic; [ "$field" = globs ] && magic=':(glob)' || magic=':(literal)'
  local -a surface=() queue=("$entry") seen=()
  local depth=0 cur spec line mode path tgt joined
  while [ ${#queue[@]} -gt 0 ] && [ $depth -lt 10 ]; do
    local -a next=()
    for cur in "${queue[@]}"; do
      # after the first hop we are resolving concrete index paths -> always literal
      if [ $depth -eq 0 ]; then spec="${magic}${cur}"; else spec=":(literal)${cur}"; fi
      while IFS= read -r line; do
        [ -z "$line" ] && continue
        mode=${line%% *}; path=${line#*$'\t'}
        surface+=("$path")
        if [ "$mode" = 120000 ]; then
          tgt=$(git cat-file blob ":$path" 2>/dev/null) || continue
          case "$tgt" in
            /*) continue ;;                                  # absolute target: outside our reach
          esac
          joined=$(norm "$(dirname "$path")/$tgt") || continue
          [ -z "$joined" ] && continue
          case " ${seen[*]-} " in *" $joined "*) continue ;; esac
          seen+=("$joined"); next+=("$joined")
        fi
      done < <(git ls-files -s -- "$spec" 2>/dev/null)
    done
    queue=("${next[@]-}"); depth=$((depth+1))
    [ ${#queue[@]} -eq 1 ] && [ -z "${queue[0]-}" ] && break
  done
  printf '%s\n' "${surface[@]-}" | grep -v '^$' | sort -u
}

verdict () {                      # $1 entry, $2 field, $3 expectation note
  local entry="$1" field="$2" note="$3"
  local -a specs=(); local p
  while IFS= read -r p; do [ -n "$p" ] && specs+=(":(literal)$p"); done < <(expand "$entry" "$field")
  local flat="${specs[*]-}"
  if [ ${#specs[@]} -eq 0 ]; then
    printf '  %-24s -> NON-CONTRIBUTING (report, E7)            %s\n' "$entry" "$note"; return
  fi
  git diff-index --quiet HEAD -- "${specs[@]}"; local pr=$?
  local st; case $pr in 0) st="CLEAN";; 1) st="DIRTY -> refuse";; *) st="ABORT($pr)";; esac
  printf '  %-24s -> surface={%s}  probe=%s   %s\n' "$entry" "$(expand "$entry" "$field" | tr '\n' ' ')" "$st" "$note"
}

echo "### FAL-3 — the three F-37 routes under the candidate"
verdict 'missing/../link' paths 'ROUTE 1 (must be DIRTY)'
verdict 'sparse_out/slink' paths 'ROUTE 2 (must be DIRTY)'
verdict 'foo*'            paths 'ROUTE 3 (must be DIRTY)'
echo
echo "### control — shapes that must keep working"
verdict 'link'            paths 'F-20 symlink (must be DIRTY)'
verdict 'chain'           paths 'symlink->symlink (must be DIRTY)'
verdict 'real/target.txt' paths 'plain dirty file'
echo
echo "### FAL-4 — path THROUGH a symlinked directory"
verdict 'linkdir/target.txt' paths 'does the candidate recover it?'
verdict 'linkdir'            paths 'the link itself'
echo
echo "### FAL-5 — index bits that suppress the measurement entirely"
verdict 'assumefile.txt'  paths 'assume-unchanged + dirty on disk'
