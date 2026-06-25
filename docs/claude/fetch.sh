#!/usr/bin/env bash
set +ue
set +o pipefail

BASE="https://code.claude.com/docs/"
MATCH='https://code\.claude\.com/docs/en/(.+/)?\K[^ )]+\.md'
INDEX="${BASE}llms.txt"
DOWNLOADS="hooks.md hooks-reference.md 
subagents.md subagents-reference.md 
plugins.md plugins-reference.md" # see index.txt for more

echo -e "Fetching Claude Code docs index: llms.txt ..."
curl $INDEX -sL | grep -oP "$MATCH" | sort | uniq >index.txt

echo -e "Index of available docs written to index.txt\nDownloading ..."
for file in $DOWNLOADS; do
  echo -e "  -> $file"
  curl "${BASE}${file}" -sL >$file
done

echo 'Done.'
