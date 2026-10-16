The framework has **two** gitignore surfaces and they have already drifted.

**Surface 1 — the client contract.** `install/manifest.toml [gitignore].entries`
(`install/manifest.toml:36-58`), appended line-by-line by `ensure_gitignored`
(`src/install.rs:1152`). 11 entries, all `.doctrine/**`, **no `!` negations** — the
TOML format cannot express one. It declares `.doctrine/skills/*` but *not*
`.doctrine/agents/*`, and none of the harness dirs install writes
(`.claude/agents/`, `.claude/skills/`, `.pi/agents/`).

**Surface 2 — the dogfood `.gitignore`** in this repo. Richer than the contract:
`/.claude/*` + `!/.claude/settings.json`, `/.pi/*` + `!/.pi/agents`,
`.doctrine/agents/*` + `!.doctrine/agents/AGENTS.md`, plus retired
minimal-projection entries (`.doctrine/glossary.md`, `.doctrine/mod.just`, …).

**The gate's blind spot.** `every_runtime_gitignore_glob_is_classified`
(`src/worktree/mod.rs:734-753`) reads **this repo's** `.gitignore`, and only lines
that `starts_with(".doctrine/")` and are not `!`-prefixed. Each such line must
match `WITHHELD` or `DERIVED_RUNTIME` (`src/worktree/allowlist.rs:73,99`;
`DERIVED_RUNTIME = [".doctrine/skills/*", ".doctrine/agents/*"]`). Consequences:

- the *client contract* is not policed at all — only the repo file;
- harness-prefixed lines (`.claude/…`, `.pi/…`, `.agents/…`) are never checked, so a
  new framework-owned harness path can land unclassified;
- adding a new `.doctrine/`-prefixed line to the repo `.gitignore` **reds CI** until
  it is registered in one of those two lists.

**What it means for install work.** "Single-source the gitignore entries" (STD-001)
is a two-surface question, not a one-file edit: decide whether the manifest becomes
the generator of the repo `.gitignore` too, and whether the parity gate widens past
the `.doctrine/` prefix. "Ignore doctrine's files, not the client's" in a shared dir
like `.claude/agents/` has no expression in the current manifest format.

Verified 2026-10-15 during SL-281 research (`.doctrine/slice/281/research/research.md`,
cross-thread findings 4-5); the slice-side record is CHR-043.