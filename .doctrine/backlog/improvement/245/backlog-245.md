# IMP-245: Support Cursor as a doctrine harness

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Intent

Doctrine already treats agents through a harness seam (`src/boot.rs`
`enum Harness { Claude, Codex }`, per-harness `import_targets` /
`install_refresh` / `wire()`). Extend this to first-class Cursor support:
detection, session bootstrap, MCP registration, and (later) dispatch routing —
mirroring what Claude Code and codex/pi already get.

## Context — a surprising amount already works

Cursor already picks up doctrine mostly for free, no code changes required:

- **Skills** — `.agents/skills/` via `npx skills add` + `skills-lock.json`
  works today (this repo dogfoods it).
- **MCP** — `doctrine serve --mcp` tools (10 review + 8 memory) surface via
  project MCP config.
- **Governance** — `AGENTS.md` with the BOOT-SENTINEL guard is read as an
  always-applied workspace rule; `@.doctrine/state/boot.md` inlines correctly.

What's missing: Cursor is not a harness in `resolve_harnesses`; `doctrine
install` doesn't detect or wire `.cursor/*`; session refresh (regenerating
`boot.md`) is manual; no `install/hymns/harness/cursor.md` tool-vocabulary
hymn; dispatch routes only claude vs subprocess.

## Immediate blocker

A `.cursor/hooks.json` `sessionStart` hook calling `doctrine boot --emit`
fails because Cursor's hook runner expects **JSON on stdout**
(`{ "additional_context": "..." }` per Cursor's hooks.md), not raw markdown.

**First deliverable:** `doctrine boot --emit --json` — unchanged disk write
to `.doctrine/state/boot.md`; stdout wrapped as the Cursor `sessionStart` JSON
envelope. CLI: `--json` flag on `Boot`, `requires = "emit"`.

Known Cursor-side limitation (non-blocking): an open Cursor bug means
`sessionStart` `additional_context` may not always reach the agent's initial
system prompt in some builds. The `AGENTS.md` sentinel + on-disk `boot.md`
remain the reliable fallback path regardless; `--json` just stops the hook
from erroring, and is forward-compatible once Cursor fixes injection.

## Scope (phased)

1. **Phase 0** — `doctrine boot --emit --json` + tests (valid JSON, escaping,
   behaviour-preservation of plain `--emit`).
2. **Phase 1** — `Harness::Cursor` detection (`.cursor/` dir) in
   `resolve_harnesses` / `install.rs::detect_agents`; wire `.cursor/hooks.json`
   `sessionStart` → `<exec> boot --emit --json`.
3. **Phase 2 (optional)** — `install/hymns/harness/cursor.md` tool
   vocabulary hymn; dispatch router: `.cursor/` present, no `.claude/` →
   `/dispatch-subprocess`.
4. **Deferred** — Cursor Task-tool dispatch arm (native subagent spawn),
   contingent on a worktree-isolation spike; not pursued until proven useful.

## Acceptance criteria

- `doctrine boot --emit --json` emits valid JSON with `additional_context`
  equal to the rendered snapshot; disk write is byte-identical to plain
  `--emit`.
- Existing Claude/codex harness tests and behaviour are unchanged.
- A `.cursor/hooks.json` `sessionStart` hook using the new flag does not error
  in Cursor's Hooks output channel.
