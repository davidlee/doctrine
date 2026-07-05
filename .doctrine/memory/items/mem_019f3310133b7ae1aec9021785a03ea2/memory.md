# Changed confined agent-def is made live by direct write to gitignored .claude/agents/<name>.md — doctrine install cannot reseat (no --force)

The ISS-216 reseat gap, characterised + worked around (SL-206 PHASE-01,
2026-07-06, empirically verified on this harness):

- `doctrine install` has **no `--force`/`--reseat`** flag and is **skip-if-exists**
  (`src/install.rs`, "skip … (exists)"). It cannot refresh a changed def that is
  already installed (ISS-216 fault #1). Fault #2: the `dispatch-worker` live symlink
  still points at the pre-subdir-split `.doctrine/agents/dispatch-worker.md`.
- `.claude/agents/*.md` (and the worker symlink's `.doctrine/agents/` target) are
  **gitignored** — the derived/runtime live-read tier the harness resolves.

**Procedure (demonstrated):** to make a new or changed confined agent-def live,
write the def bytes **directly** to `.claude/agents/<name>.md`. A scratch probe
def written v1 → rewritten v2 with an added
`mcp__doctrine__dispatch_phase_receipt` token was immediately readable at that
path with the v2 token present. For a NEW def (e.g. SL-206 PHASE-04
`dispatch-probe.md`) place a plain file; for a CHANGED existing plain-file def
(e.g. the orchestrator gaining MCP tokens) overwrite it in place. `install` only
helps for defs that do not yet exist.

Relates to ISS-216 and SL-199 PHASE-05 (which hit the same gap and hand-added a
frontmatter line to the live-read file).
