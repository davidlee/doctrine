#!/usr/bin/env bash
# DEC-087 / CON-002 — committed .gitattributes separates "is git satisfied?"
# from "does the attested commit contain these bytes?" (I6), and the repair is
# one flag on NORMATIVE_FLAGS.
#
# Re-authors the round-3 measurements behind RV-314 F-19, which until now
# existed only as prose in design.md, DEC-087, CON-002 and the ledger. Their
# original scripts ran in a scratch directory that no longer exists.
#
# FALSIFIERS, registered before the probe runs:
#
#   FAL-A1  (F-19 limb (a), eol conversion)
#     `<path> text eol=crlf` committed, HEAD blob LF, worktree CRLF.
#     REFUTED IF: any of the three legs reports the divergence under
#     NORMATIVE_FLAGS. Those flags pin core.autocrlf/core.eol precisely to stop
#     machine-local config perturbing the frame; if they also stop COMMITTED
#     attributes then I6 was never false and DEC-087 is unnecessary.
#
#   FAL-A2  (F-19 limb (b), a clean filter, and the round-3 acquittal trap)
#     A clean filter mapping every line to CANONICAL, with worktree content
#     that diverges from the blob without limit.
#     REFUTED IF: `diff-index --quiet --cached HEAD` — the leg § 5.1 actually
#     specifies — reports non-zero. The round-3 reviewer probed the
#     worktree-inclusive form instead, got 1, and acquitted the limb; this
#     probe runs BOTH forms side by side so the two can never be confused
#     again. A difference between them is the finding, not an artefact.
#
#   FAL-A3  (DEC-087's repair)
#     `--attr-source=<empty tree>` added to the invocation.
#     REFUTED IF: the tracked leg still reports zero on either route. Then the
#     flag does not neutralise conversion at the probe and the repair fails.
#
#   FAL-A4  (STD-001: the empty-tree oid must be derived, not hardcoded)
#     REFUTED IF: `git hash-object -t tree /dev/null` returns the same oid
#     under sha1 and sha256. Then a literal would be safe and the derivation
#     is ceremony.
#
#   FAL-A5  (CON-002's detection is a capability probe, not version arithmetic)
#     REFUTED IF: `git --attr-source=<oid> rev-parse --git-dir` cannot
#     distinguish support from non-support by exit status alone.
#
# Measured on git 2.54.0. This repo has no .gitattributes, so every figure
# below comes from a constructed fixture; CON-002's unmet path cannot be
# exercised here at all and is probed only through the shape of the refusal
# git gives for an unrecognised main-command option.
set -u

G=(git -c core.autocrlf=false -c core.eol=lf -c core.fileMode=true)

legs() {  # legs <label> <extra-git-args-as-one-string> <pathspec>
  local label=$1 extra=$2 spec=$3
  local tracked untracked rc
  # shellcheck disable=SC2086
  tracked=$("${G[@]}" $extra diff HEAD --binary --no-textconv --no-ext-diff -- "$spec" | wc -c)
  # shellcheck disable=SC2086
  untracked=$("${G[@]}" $extra ls-files --others --exclude-standard -- "$spec" | wc -l)
  # shellcheck disable=SC2086
  "${G[@]}" $extra diff-index --quiet --cached HEAD -- "$spec"; rc=$?
  printf '    %-30s tracked=%-6s untracked=%-3s index_rc(--cached)=%s\n' \
    "$label" "$tracked" "$untracked" "$rc"
}

echo "git $(git --version | awk '{print $3}')"
echo

# ------------------------------------------------------------------- FAL-A1
R=$(mktemp -d /tmp/rv314-attr-eol.XXXXXX); cd "$R" || exit 1
git init -q .; git config user.email a@b.c; git config user.name t
printf 'body.txt text eol=crlf\n' > .gitattributes
printf 'attested body\n' > body.txt
git add .gitattributes body.txt >/dev/null; git commit -qm init >/dev/null
git rm -q --cached body.txt >/dev/null 2>&1; git checkout -q -- . 2>/dev/null
git reset -q --hard >/dev/null
# The checkout wrote CRLF into the worktree; HEAD's blob is LF.
EMPTY_TREE=$(git hash-object -t tree /dev/null)

echo "### FAL-A1 — eol conversion (repo: $R)"
echo -n "  HEAD blob bytes : "; git cat-file blob HEAD:body.txt | od -An -tx1 | tr -d ' \n'; echo
echo -n "  worktree bytes  : "; od -An -tx1 < body.txt | tr -d ' \n'; echo
git cat-file blob HEAD:body.txt > /tmp/rv314-blob.$$ 2>/dev/null
if cmp -s /tmp/rv314-blob.$$ body.txt; then echo "  cmp: identical"; else echo "  cmp: NOT identical"; fi
rm -f /tmp/rv314-blob.$$
legs "under NORMATIVE_FLAGS" "" ':(literal)body.txt'
legs "+ --attr-source=empty" "--attr-source=$EMPTY_TREE" ':(literal)body.txt'
echo "  -> FAL-A1 needs row 1 at 0/0/0; FAL-A3 needs row 2's tracked leg non-zero."
echo

# --------------------------------------------------------------- FAL-A2 / A3
R2=$(mktemp -d /tmp/rv314-attr-filter.XXXXXX); cd "$R2" || exit 1
git init -q .; git config user.email a@b.c; git config user.name t
git config filter.flatten.clean 'sed "s/.*/CANONICAL/"'
printf 'file.txt filter=flatten\n' > .gitattributes
printf 'anything at all\n' > file.txt
git add .gitattributes file.txt >/dev/null; git commit -qm init >/dev/null
printf 'CONTENT THAT DIVERGES FROM THE BLOB WITHOUT LIMIT\n' > file.txt
EMPTY_TREE2=$(git hash-object -t tree /dev/null)

echo "### FAL-A2 — clean filter (repo: $R2)"
echo -n "  HEAD blob : "; git cat-file blob HEAD:file.txt
echo -n "  worktree  : "; cat file.txt
legs "under NORMATIVE_FLAGS" "" ':(literal)file.txt'
echo -n "    the two diff-index forms, side by side: "
"${G[@]}" diff-index --quiet --cached HEAD -- ':(literal)file.txt'; a=$?
"${G[@]}" diff-index --quiet HEAD -- ':(literal)file.txt'; b=$?
echo "--cached -> $a   (worktree form) -> $b"
echo "    § 5.1 specifies the --cached form. The round-3 acquittal probed the"
echo "    other one. If these differ, that difference IS the finding."
legs "+ --attr-source=empty" "--attr-source=$EMPTY_TREE2" ':(literal)file.txt'
echo "  -> FAL-A2 needs the --cached leg at 0; FAL-A3 needs the last row non-zero."
echo

# ------------------------------------------------------------------- FAL-A4
echo "### FAL-A4 — the empty-tree oid is hash-algorithm dependent"
for algo in sha1 sha256; do
  RA=$(mktemp -d "/tmp/rv314-attr-$algo.XXXXXX")
  git init -q --object-format="$algo" "$RA" 2>/dev/null || { echo "  $algo: unsupported here"; continue; }
  printf '  %-7s empty-tree oid = %s\n' "$algo" "$(git -C "$RA" hash-object -t tree /dev/null)"
done
echo "  -> FAL-A4 holds iff these differ. Equal oids would make a literal safe."
echo

# ------------------------------------------------------------------- FAL-A5
echo "### FAL-A5 — CON-002's capability probe"
cd "$R2" || exit 1
git --attr-source="$EMPTY_TREE2" rev-parse --git-dir >/dev/null 2>&1
echo "  git --attr-source=<oid> rev-parse --git-dir       -> exit $? (0 = supported)"
git --no-such-main-command-option rev-parse --git-dir >/dev/null 2>&1
echo "  git --no-such-main-command-option rev-parse …     -> exit $? (unrecognised)"
echo "  -> non-zero is the sufficient test; the exit VALUE and the message are"
echo "     not load-bearing, which is why CON-002 does not parse either."
