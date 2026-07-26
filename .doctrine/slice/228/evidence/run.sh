#!/usr/bin/env bash
# Run the OQ-6 triage over every batch, 4 at a time.
D="$(cd "$(dirname "$0")" && pwd)"
cd "$D"
run_one() {
  b="$1"
  n="${b##*batch-}"; n="${n%.md}"
  pi --print --model deepseek/deepseek-v4-pro \
     --system-prompt "$(cat "$D/oq6-sys.md")" \
     --no-tools --thinking low --no-session \
     "@$D/verb-surface.md" "@$D/needs-ledger.txt" "@$b" \
     "The first two files are your evidence: the verb surface as it now stands, and the memory-blind subject's full situations ledger. The third file is your batch of memories to triage. Produce one verdict block per memory in that batch file, in order, in exactly the format the system prompt specifies." \
     > "$D/out-$n.md" 2>"$D/err-$n.log"
  echo "done $n ($(grep -c '^verdict:' "$D/out-$n.md" 2>/dev/null) verdicts)"
}
export -f run_one
export D
ls "$D"/batch-*.md | xargs -I{} -P 4 bash -c 'run_one "$@"' _ {}
echo "ALL DONE"
grep -h '^verdict:' "$D"/out-*.md | sort | uniq -c
