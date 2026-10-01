# ISS-507: memory retrieve rejects a positional query and its error omits --query

## What

`doctrine memory retrieve "<topic>"` fails with `unexpected argument '<topic>' found`.
The verb takes `--query`; its sibling `memory search` takes a positional. The error
does not name `--query`.

The wrong form is **taught** by the shipped capsule agent definitions:

- `install/agents/claude/capsule-phase-planner.md:44` — `doctrine memory retrieve "<topic>"`
- `install/agents/claude/capsule-worker.md:52` — same

So every capsule planner pays one or more failed calls (six in one SL-268 phase).

Adjacent rough edges seen in the same records:

- `--format table` prints full framed bodies, not a compact candidate list.
- Retrieval is silently empty in a repo with no git remote.
- `--path` is the project root, not a scope probe; the retrieve-memory skill's
  "scoped to files" wording invites misuse.

## Fix sketch

1. Accept a positional query on `memory retrieve` (alias of `--query`), matching `memory search`.
2. Correct the two agent defs regardless.
3. Separately (may split): compact table output; disclose the no-remote empty result (STD-003).

## Evidence

24 friction observations, 2026-07-29 → 2026-09-28; 15 since 2026-09-17, nearly all
from capsule planners/orchestrators. E.g. `01a0dd66` (six failed calls, SL-268 PHASE-02),
`01a0ba9a`, `01a0e352`, `01a0e814` (no-remote empty), `01a0b9da` (table format).
