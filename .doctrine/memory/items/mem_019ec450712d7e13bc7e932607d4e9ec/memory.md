# doctrine claude install self-enforces a too-broad .doctrine/agents/* gitignore that breaks the worktree classifier and swallows authored AGENTS.md

doctrine claude install appends an unclassified, too-broad .doctrine/agents/* gitignore — don't commit it

Running `doctrine claude install` (e.g. a re-embed/skill refresh) calls
`ensure_gitignored(".doctrine/agents/*")`. That glob is wrong twice over:

- It blanket-ignores the whole `.doctrine/agents/` dir, which holds the
  **authored, tracked** `AGENTS.md` — the never-blanket-ignore-an-authored-tree
  hazard (same lesson the install manifest documents for `memory/items/`). Only
  the *derived* installed agents (e.g. `dispatch-worker.md`) should be ignored.
- It is not registered in the worktree provision classifier
  (`WITHHELD`/`DERIVED_RUNTIME` in `src/worktree.rs`), so the invariant test
  `every_runtime_gitignore_glob_is_classified` (which enumerates every
  `.doctrine/`-prefixed gitignore glob) goes RED the moment the entry is committed.

Symptom: after a re-embed, `git status` shows `M .gitignore` (+`.doctrine/agents/*`)
and `just check` fails `every_runtime_gitignore_glob_is_classified`.

Do NOT commit that `.gitignore` line. The proper fix (narrow the install ignore to
derived agent outputs only, and add it to `DERIVED_RUNTIME`, in one change) is the
SL-056 agent-install surface's job — tracked as ISS-012. Surfaced auditing SL-061.
