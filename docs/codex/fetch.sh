#!/usr/bin/env bash
set -eu
set -o pipefail

# Keep the cache beside this script, regardless of the caller's working dir.
cd "$(dirname "$0")"

BASE="https://learn.chatgpt.com/docs/"
INDEX="https://developers.openai.com/codex/llms.txt"
MATCH='https://learn\.chatgpt\.com/docs/\K[^ )]+\.md'
# Keep this list aligned with the Codex-specific pages in index.txt. The index
# is the complete upstream catalog; this cache keeps the local agent workflow,
# CLI, configuration, security, and integration references close at hand.
DOWNLOADS="codex/cli.md codex/ide.md app-server.md codex-sdk.md developer-commands.md cli-customization.md config-file/config-basic.md config-file/config-advanced.md config-file/config-reference.md config-file/config-sample.md config-file/environment-variables.md agent-configuration/agents-md.md agent-configuration/rules.md agent-configuration/subagents.md customization/overview.md customization/memories.md extend/mcp.md hooks.md agent-approvals-security.md"
MAX_AGE_DAYS="${DOCS_MAX_AGE_DAYS:-7}" # refetch anything older; 0 refetches everything

echo "Fetching Codex docs index: llms.txt ..."
curl -fsSL "$INDEX" -o llms.txt.tmp
mv llms.txt.tmp llms.txt
grep -oP "$MATCH" llms.txt | sed 's/[?#].*$//' | sort -u >index.txt

echo "Index of available docs written to index.txt; downloading (refetching anything older than ${MAX_AGE_DAYS}d) ..."
for upstream_path in $DOWNLOADS; do
  # Store the selected Codex pages directly under this cache directory while
  # retaining their upstream path for the request.
  file="${upstream_path#codex/}"
  # `-mtime -N` matches "modified less than N days ago", so N=0 matches nothing.
  if [ -s "$file" ] && [ -n "$(find "$file" -mtime "-${MAX_AGE_DAYS}" -print -quit 2>/dev/null)" ]; then
    echo "  -> $file ... skipping (fresh)"
    continue
  fi
  [ -s "$file" ] && why="stale" || why="new"
  echo "  -> $file ($why)"
  mkdir -p "$(dirname "$file")"
  # Fetch to a temp file so a failed refresh never destroys a good cached copy.
  if curl "${BASE}${upstream_path}" -fsSL --retry 2 -o "$file.tmp" && [ -s "$file.tmp" ]; then
    mv "$file.tmp" "$file"
  else
    rm -f "$file.tmp"
    echo "     !! fetch failed — keeping the existing copy"
  fi
done

echo 'Done.'
