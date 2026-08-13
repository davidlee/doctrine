#!/usr/bin/env bash
set +ue
set +o pipefail

BASE="https://code.claude.com/docs/"
MATCH='https://code\.claude\.com/docs/en/(.+/)?\K[^ )]+\.md'
INDEX="${BASE}llms.txt"
# Names must match index.txt — upstream renames turn into 404s here, not into
# silent staleness (`hooks-reference.md` -> `hooks.md` + `hooks-guide.md` and
# `settings-reference.md` -> `settings.md`, both caught 2026-08-14).
# Anything on disk but absent here is stale forever, so this list should cover
# the whole cache — see index.txt for what else is available.
DOWNLOADS="hooks.md hooks-guide.md subagents.md plugins.md plugins-reference.md settings.md workflows.md agent-sdk/typescript.md env-vars.md authentication.md mcp.md mcp-quickstart.md plugin-marketplaces.md skills.md claude-directory.md"
MAX_AGE_DAYS="${DOCS_MAX_AGE_DAYS:-7}" # refetch anything older; 0 refetches everything

echo -e "Fetching Claude Code docs index: llms.txt ..."
curl $INDEX -sL | grep -oP "$MATCH" | sort | uniq >index.txt

echo -e "Index of available docs written to index.txt\nDownloading (refetching anything older than ${MAX_AGE_DAYS}d) ..."
for file in $DOWNLOADS; do
  # `-mtime -N` matches "modified less than N days ago", so N=0 matches nothing
  # and every doc is refetched.
  if [ -s "$file" ] && [ -n "$(find "$file" -mtime "-${MAX_AGE_DAYS}" -print -quit 2>/dev/null)" ]; then
    echo -e "  -> $file ... skipping (fresh)"
    continue
  fi
  [ -s "$file" ] && why="stale" || why="new"
  echo -e "  -> $file ($why)"
  mkdir -p "$(dirname "$file")" # nested docs (e.g. agent-sdk/) need the dir first
  # Fetch to a temp first. A bare `>$file` truncates before curl runs, which was
  # harmless while existing files were never refetched — but now that a stale doc
  # IS refetched, a failed fetch would destroy the cached copy it was replacing.
  # `-f` makes curl fail the shell test on an HTTP error rather than saving the
  # error page as if it were the doc.
  if curl "${BASE}en/${file}" -sLf --retry 2 -o "$file.tmp" && [ -s "$file.tmp" ]; then
    mv "$file.tmp" "$file"
  else
    rm -f "$file.tmp"
    echo -e "     !! fetch failed — keeping the existing copy"
  fi
done

echo 'Done.'
