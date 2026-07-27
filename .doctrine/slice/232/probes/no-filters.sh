#!/usr/bin/env bash
# Is `--no-filters` SUFFICIENT for untracked_fingerprint, or does it only skip
# clean filters while eol/text conversion still applies?
# FALSIFIER: under `text eol=crlf`, --no-filters still normalises CRLF -> LF
#            => --no-filters alone is insufficient and the attr flags are needed too.
set -u; export LC_ALL=C; G=$(command -v git)
NORM=(-c core.autocrlf=false -c core.eol=lf -c core.fileMode=true)
d=/tmp/sa-nf; rm -rf $d; mkdir -p $d
cd $d
$G init -q .; $G config user.email a@b; $G config user.name a
printf 'seed\n' > seed; $G add seed; $G commit -qm base
printf '*.txt text eol=crlf\n' > .gitattributes; $G add .gitattributes; $G commit -qm attrs
printf 'line-one\r\nline-two\r\n' > crlf.txt          # untracked, CRLF on disk
E=$($G hash-object -t tree /dev/null)
RAW=$(printf 'line-one\r\nline-two\r\n' | $G hash-object --stdin)
LF=$( printf 'line-one\nline-two\n'     | $G hash-object --stdin)
echo "  raw-bytes (CRLF) oid = $RAW"
echo "  lf-normalised    oid = $LF"
echo "  --- what each spelling of hash-object returns for the untracked file ---"
printf '  %-34s %s\n' 'plain'                  "$($G "${NORM[@]}" hash-object -- crlf.txt)"
printf '  %-34s %s\n' '--no-filters'           "$($G "${NORM[@]}" hash-object --no-filters -- crlf.txt)"
printf '  %-34s %s\n' '--attr-source=<empty>'  "$($G "${NORM[@]}" --attr-source="$E" hash-object -- crlf.txt)"
echo
echo "  VERDICT: plain==LF? $([ "$($G "${NORM[@]}" hash-object -- crlf.txt)" = "$LF" ] && echo YES-converted || echo no)"
echo "           --no-filters==RAW? $([ "$($G "${NORM[@]}" hash-object --no-filters -- crlf.txt)" = "$RAW" ] && echo YES-raw || echo NO-STILL-CONVERTED)"
echo "           --attr-source==RAW? $([ "$($G "${NORM[@]}" --attr-source="$E" hash-object -- crlf.txt)" = "$RAW" ] && echo YES-raw || echo NO)"
