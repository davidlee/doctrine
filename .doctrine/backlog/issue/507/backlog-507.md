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

## Resolution (2026-10-01)

Fixed without a slice (lite path, user-approved). The positional query moved into
`FindRetrieveArgs`, the argument struct `search` and `retrieve` share, guarded by
clap `conflicts_with = "flag_query"`; `FindRetrieveArgs::free_query()` resolves
either form. This replaces `search`'s hand-written exclusivity check. `--query`
now displays as `<QUERY>`, not `<FLAG_QUERY>`. Test:
`memory_search_and_retrieve_accept_the_query_either_way` (`src/main.rs`).

The two capsule agent defs needed no edit — the positional form they teach is now
valid. Fix sketch item 3 split to ISS-517 (silent empty with no git remote) and
IMP-512 (compact table output, `--path` wording).
