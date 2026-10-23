# DEC-412: codex_mcp plan: implementation deltas from the SL-286 design

Deltas adopted while implementing SL-286 PHASE-02 (`src/codex_mcp.rs`,
commits 23a963447..3112de99c). Reconcile folds each into `design.md`.

1. **FileRead input** — `plan` takes `FileRead { Absent, Unreadable, Text }`
   for both files, not `Option<&str>`: the design's own
   "unreadable → Degraded(UserUnreadable)" needs a third state.
2. **destination split** — pure `destination(&CodexEnv)`; the shell calls it
   before reading the user file, `plan` reuses it. `codex_home_exists` is a
   supplied fact.
3. **Wrapper recognition** is table-level `is_wrapper_form` (command + args),
   shared with boot's `classify_codex_entry`; `classify_codex_entry` itself
   stays in boot until PHASE-04 (plan wins over the design's "moves").
4. **`UserEntry::Add` carries no `dropped`** (Add means no entry); `dropped`
   is on `Replace` only.
5. **`Current`** exactly per design: command == bake, serve args, `enabled`
   not false; other keys survive.
6. **Degraded causes** each get a `codex: user MCP registration skipped —
   <cause>` line; `run:` only when a bake is known;
   `ExecNotOnPath { exec, on_path }` carries both paths.
7. **`removes` consent line** appends ` (with <extra keys>)`; `env_vars` is an
   extra key unless exactly `[DOCTRINE_BIN]`.
8. **Removal-only consent** (user Current + project OwnedWrapper):
   `Codex: remove doctrine's old project MCP wrapper?` / `removes …` /
   `Remove it? [y/N]`. Wording flagged to the user.
9. **`proposes_change` is false when the project entry blocks**
   (PathShadow / Malformed / Unreadable) — design § Execute writes nothing
   there, so no prompt. Contradicts PHASE-02 EX-4's "exactly when"; user
   ruling "agreed" (2026-10-24). EX-4 amended at reconcile.

Also: `find_on_path(name, path_value)` (EX-6) over the design's
`find_on_path(name)`; the design's layering note "→ boot" is void (EX-1).
