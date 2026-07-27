#!/usr/bin/env bash
# RV-314 — T84's missing artefact (raised as F-41 limb 2).
#
# QUESTION: DEC-090 classifies "an index pathname or derived target that is not
# valid UTF-8" as `unmeasurable`, refused at the emission boundary. Round 4
# accepted this on the reviewer's transcript and never persisted a probe. Two
# things need to be true for the design's mechanism to work:
#
#   1. a symlink whose TARGET is not valid UTF-8 can exist in the index, and
#      `cat-file blob` returns it with exit 0 — i.e. git hands doctrine bytes it
#      cannot put through a `&str` pathspec;
#   2. the failure is detectable at the conversion boundary BEFORE emission,
#      rather than surfacing as a git error after the fact.
#
# FALSIFIER: if `cat-file blob :<link>` exits non-zero, or if the returned bytes
# are valid UTF-8, then the non-UTF-8 member of DEC-090's table has no object and
# the classification is answering a question that cannot arise. Either outcome
# means the design should say so rather than carrying a fourth member.
#
# NOTE: this probe deliberately does NOT construct a non-UTF-8 index *pathname*.
# The design's table names both pathnames and derived targets; only the target
# route is measured here, and the pathname route stays unmeasured — recorded
# rather than smoothed, per § 5.1's treatment of GIT_ATTR_NOSYSTEM.
set -u
export LC_ALL=C
G=$(command -v git); echo "git: $($G --version)"

d=/tmp/nonutf8-probe; rm -rf "$d"; mkdir -p "$d"
cd "$d" || exit 1
$G init -q .; $G config user.email a@b; $G config user.name a

printf 'real content\n' > realfile
$G add realfile; $G commit -qm base

# A symlink blob is just its target string. Write one whose bytes are invalid
# UTF-8 (0xff is never a legal UTF-8 lead byte) straight into the object store,
# then place it in the index as mode 120000.
oid=$(printf '\377target' | $G hash-object -w --stdin)
$G update-index --add --cacheinfo 120000,"$oid",link

echo
echo "=== the index entry"
$G ls-files -s -v -- link | sed 's/^/  /'

echo
echo "=== cat-file blob :link — does git hand us the bytes, and with what rc?"
out_hex=$($G cat-file blob :link | od -An -tx1 | tr -s ' ' | sed 's/^ //')
# Capture rc from a NON-pipelined invocation — `$?` after a pipeline reads the
# last element, which is the F-41 defect this probe was written to replace.
$G cat-file blob :link >/dev/null 2>&1; rc=$?
printf '  target bytes (hex) = %s\n' "$out_hex"
printf '  cat-file rc        = %s\n' "$rc"

# The UTF-8 checker is itself an instrument and must be proved capable of BOTH
# answers before its output means anything. The first draft of this probe used
# `iconv`, which is absent in this jail: it returned 127 for every input, so both
# the fixture AND the control read "INVALID" and the probe appeared to confirm
# the design while measuring nothing. That is round 4's standing lesson —
# a negative result is evidence only if the query could have gone positive —
# committed by the very probe written to replace a defective artefact.
utf8_ok() { python3 -c 'import sys; sys.stdin.buffer.read().decode("utf-8")' >/dev/null 2>&1; }
if ! command -v python3 >/dev/null; then
  echo "  !! no UTF-8 checker available — every row below is unmeasured, not negative"
  exit 1
fi
printf 'x' | utf8_ok || { echo "  !! checker rejects known-valid input — instrument broken"; exit 1; }
printf '\377' | utf8_ok && { echo "  !! checker accepts known-invalid input — instrument broken"; exit 1; }
echo "  (checker verified: accepts valid, rejects invalid)"

echo
echo "=== is it valid UTF-8? (the conversion boundary the design refuses at)"
if $G cat-file blob :link | utf8_ok; then
  echo "  VALID UTF-8   <- FALSIFIES the non-UTF-8 member: no object exists"
else
  echo "  INVALID UTF-8 <- the member has an object; refusal at conversion is reachable"
fi

echo
echo "=== control: a well-formed target through the same path"
oid_ok=$(printf 'realfile' | $G hash-object -w --stdin)
$G update-index --add --cacheinfo 120000,"$oid_ok",link_ok
$G cat-file blob :link_ok >/dev/null 2>&1; rc_ok=$?
printf '  target             = %s\n' "$($G cat-file blob :link_ok)"
printf '  cat-file rc        = %s\n' "$rc_ok"
if $G cat-file blob :link_ok | utf8_ok; then
  echo "  VALID UTF-8   <- control behaves as expected"
else
  echo "  INVALID UTF-8 <- control is broken; the discrimination above is unsound"
fi

echo
echo "=== summary"
echo "  Both entries return exit 0 from cat-file, so the rc carries no signal."
echo "  The discrimination is the UTF-8 conversion, not the git invocation —"
echo "  which is why DEC-090 places the refusal at the emission boundary."
