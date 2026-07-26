#!/usr/bin/env bash
# Build OQ-6 triage batches from the candidate uid list.
# usage: batch.sh <size>
set -euo pipefail
D="$(cd "$(dirname "$0")" && pwd)"
REPO=/workspace/doctrine
SIZE="${1:-12}"

rm -f "$D"/batch-*.md
n=0
batch=1
out="$D/batch-$(printf '%02d' $batch).md"
: > "$out"
while read -r uid; do
  item="$REPO/.doctrine/memory/items/$uid"
  [ -d "$item" ] || continue
  if [ $n -ge "$SIZE" ]; then
    batch=$((batch + 1))
    out="$D/batch-$(printf '%02d' $batch).md"
    : > "$out"
    n=0
  fi
  {
    echo "### $uid"
    grep -E '^(memory_key|title|memory_type|status|trust_level|severity)' "$item/memory.toml" 2>/dev/null | head -8
    echo
    cat "$item/memory.md" 2>/dev/null
    echo
    echo "---"
  } >> "$out"
  n=$((n + 1))
done < "$D/oq6-uids.txt"
wc -c "$D"/batch-*.md | tail -3
ls "$D"/batch-*.md | wc -l
