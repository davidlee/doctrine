# ISS-253: Dispatch arm routing marker is invisible from the coord worktree it mandates

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Symptom

The `/dispatch` router's step 4 routes by env marker: "`.claude/` present →
`/dispatch-agent`; otherwise → `/dispatch-subprocess`". Its own step 2 mandates
that the claude arm **park Bash cwd in the coordination worktree** for the full
drive loop. From there the marker test always answers **absent** — so a Claude
Code session that follows step 2 is routed by step 4 to the subprocess arm.

## Cause

`.claude/` is **gitignored** (`.gitignore:4: /.claude`). A linked worktree
materialises tracked content only, so the marker can never appear in one. The
two rules are evaluated in different trees: the marker belongs to the *session
root*, the cwd mandate puts the test in the *coord tree*.

Observed while driving SL-228 PHASE-09: `ls -d .claude` in
`/workspace/doctrine/.dispatch/SL-228` → absent; at `/workspace/doctrine` →
present. All eight prior phases of the same slice were driven on the claude arm,
so a literal reading of the router would have silently switched arms mid-slice.

## Relation to the PHASE-07 benchmark finding

SL-228's benchmark recorded the sibling case — a **clone** carries tracked
content only, so the absent `.claude/` silently changed which arm was measured
(`benchmark.md`, stated limit 3). This is the same root cause reaching a
different surface: not a clone, a linked worktree, and here the router's own
step-2 instruction is what defeats step-4's test.

## Fix

Switch from a filesystem marker (`.claude/` directory) to a process-level
env var (`CLAUDECODE=1`). An env var survives cwd changes into linked
worktrees, where a gitignored directory does not. One-line change in
`plugins/doctrine/skills/dispatch/SKILL.md` step 4; no engine change.
