#!/usr/bin/env bash
# DEC-080 — the symlink closure emits, the index only discovers.
#
# Re-authors the round-2 measurements behind RV-314 F-15, F-16 and F-20, which
# until now existed only as prose in design.md and the ledger. Their original
# scripts ran in a scratch directory that no longer exists; `probes/` exists so
# figures re-run rather than being re-derived (RV-313 F-1).
#
# FALSIFIERS, registered before the probe runs:
#
#   FAL-C1  (F-15, index-conditioned emission under-measures)
#     Surface A = the originating selector alone, as the index-conditioned
#     reading of step 4 produces when the target is absent from the index.
#     Surface B = A plus the joined target emitted literally.
#     REFUTED IF: A and B agree. The finding requires A clean on all three legs
#     while B is dirty. If both read dirty the index-conditioned reading was
#     never lossy and F-15 is wrong.
#
#   FAL-C2  (F-15 second route, never-tracked target)
#     Same discrimination for a target that was never tracked but is present
#     and non-ignored — inside DEC-070's evidence domain.
#     REFUTED IF: A already reports the target.
#
#   FAL-C3  (F-16, a derived string read as pathspec magic)
#     A tracked symlink whose blob text is `:(exclude)<uid>/**`, emitted raw
#     beside the mandatory `:(literal)<uid>` selector.
#     REFUTED IF: raw emission still reports the modified uid body dirty, i.e.
#     git does not honour magic in a derived position. Then the constant prefix
#     is not load-bearing for derived strings and I8's restatement is unearned.
#
#   FAL-C4  (F-20 / T68, unconditional emission is affordable)
#     A `:(literal)` pathspec matching nothing anywhere.
#     REFUTED IF: it returns non-zero on any leg, OR it masks a real signal
#     emitted alongside it. Either outcome makes unconditional emission cost
#     something and revives the coverage filter F-20 struck.
#
# NOTE ON ABSOLUTE FIGURES. design.md quotes byte counts (145, 152, 143) from
# the original unpersisted fixture. Byte counts of `git diff HEAD --binary` are
# a function of path names and content, so this probe's own numbers need not
# equal them and a difference is a fixture difference, not a contradiction.
# What is durable — and what the falsifiers above are written against — is the
# DISCRIMINATION: zero versus non-zero, exit 0 versus exit 1.
#
# Measured on git 2.54.0.
set -u

# The design's three legs (§ 5.1), under NORMATIVE_FLAGS (src/git.rs).
G=(git -c core.autocrlf=false -c core.eol=lf -c core.fileMode=true)

legs() {  # legs <label> <pathspec>...
  local label=$1; shift
  local tracked untracked rc
  tracked=$("${G[@]}" diff HEAD --binary --no-textconv --no-ext-diff -- "$@" | wc -c)
  untracked=$("${G[@]}" ls-files --others --exclude-standard -- "$@" | wc -l)
  "${G[@]}" diff-index --quiet --cached HEAD -- "$@"; rc=$?
  printf '    %-34s tracked=%-6s untracked=%-3s index_rc=%s\n' \
    "$label" "$tracked" "$untracked" "$rc"
}

echo "git $(git --version | awk '{print $3}')"
echo

# ---------------------------------------------------------------- FAL-C1 / C2
R=$(mktemp -d /tmp/rv314-closure-a.XXXXXX); cd "$R" || exit 1
git init -q .; git config user.email a@b.c; git config user.name t
printf 'evidence body, the content that is really being claimed\n' > target.txt
ln -s target.txt link
printf 'never tracked but present and non-ignored\n' > untracked-target.txt
ln -s untracked-target.txt link2
git add target.txt link link2 >/dev/null
git commit -qm init >/dev/null
# Detach the target from the index while leaving a modified copy on disk.
git rm -q --cached target.txt
printf 'MODIFIED AFTER THE ATTESTED COMMIT\n' >> target.txt

echo "### FAL-C1 — F-15: target detached from the index, modified on disk"
echo "  repo: $R"
echo "  index re-expansion of the blob target:"
"${G[@]}" ls-files -s -- ':(literal)target.txt' | sed 's/^/      /'
echo "      (empty above = the index-conditioned reading emits nothing)"
legs "A  index-conditioned  [link]" ':(literal)link'
legs "B  unconditional      [link,tgt]" ':(literal)link' ':(literal)target.txt'
echo "  -> FAL-C1 holds iff A is 0/0/0 and B is non-zero."
echo

echo "### FAL-C2 — F-15: target never tracked, present, non-ignored"
legs "A  index-conditioned  [link2]" ':(literal)link2'
legs "B  unconditional      [link2,tgt]" ':(literal)link2' ':(literal)untracked-target.txt'
echo "  -> the untracked leg is the one that must fire in B."
echo

# --------------------------------------------------------------------- FAL-C3
R2=$(mktemp -d /tmp/rv314-closure-b.XXXXXX); cd "$R2" || exit 1
git init -q .; git config user.email a@b.c; git config user.name t
UID_DIR=mem_0123456789abcdef0123456789abcdef
mkdir -p "$UID_DIR"
printf 'the attested prose\n' > "$UID_DIR/memory.md"
# A tracked symlink whose blob TEXT is pathspec magic. `ln -s` sets the blob;
# the link need not resolve to anything.
ln -s ":(exclude)$UID_DIR/**" evil
git add -A >/dev/null; git commit -qm init >/dev/null
printf 'MODIFIED AFTER THE ATTESTED COMMIT\n' >> "$UID_DIR/memory.md"

echo "### FAL-C3 — F-16: a derived string read as pathspec magic"
echo "  repo: $R2"
echo -n "  blob text of the tracked symlink 'evil': "; git cat-file blob :evil; echo
legs "control  uid dir alone" ":(literal)$UID_DIR"
legs "raw      uid dir + target" ":(literal)$UID_DIR" ":(exclude)$UID_DIR/**"
legs "prefixed uid dir + target" ":(literal)$UID_DIR" ":(literal):(exclude)$UID_DIR/**"
echo "  -> FAL-C3 holds iff 'raw' reads clean where 'control' and 'prefixed' read dirty."
echo "  The index leg reads 0 on all three rows above because the edit is not"
echo "  staged: --cached compares index against HEAD and is blind to a"
echo "  worktree-only change (which is F-19's premise). Staging it moves the"
echo "  same discrimination onto the index leg, which is design.md's figure:"
git add "$UID_DIR" >/dev/null
legs "staged control" ":(literal)$UID_DIR"
legs "staged raw" ":(literal)$UID_DIR" ":(exclude)$UID_DIR/**"
legs "staged prefixed" ":(literal)$UID_DIR" ":(literal):(exclude)$UID_DIR/**"
echo

# --------------------------------------------------------------------- FAL-C4
echo "### FAL-C4 — F-20 / T68: an unmatched literal pathspec is inert"
legs "unmatched alone" ':(literal)no/such/path/anywhere'
legs "real signal alone" ":(literal)$UID_DIR"
legs "real signal + unmatched" ":(literal)$UID_DIR" ':(literal)no/such/path/anywhere'
echo "  -> FAL-C4 holds iff the unmatched spec is 0/0/0 alone AND the third row"
echo "     equals the second. A masked signal refutes it."
