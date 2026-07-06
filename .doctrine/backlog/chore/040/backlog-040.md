# CHR-040: Repoint pi dispatch-worker symlink to pi/, not codex/

## Context

`.doctrine/agents/codex/` is an historical misnomer — "codex" was the old name
for what is now "pi". The directory holds the pi dispatch-worker agent
definition, and `.doctrine/agents/pi/` contains an identical copy.

`.pi/agents/dispatch-worker.md` symlinks to `codex/dispatch-worker.md`.

## What to do

1. Repoint the symlink: `.pi/agents/dispatch-worker.md` → `../../.doctrine/agents/pi/dispatch-worker.md`
2. Verify both `pi/dispatch-worker.md` and `codex/dispatch-worker.md` remain intact
3. Commit as `chore(CHR-040): repoint pi dispatch-worker symlink to pi/, not codex/`
