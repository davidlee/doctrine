# Gather untracked leg respects host global gitignore

`doctrine worktree import --from-worktree <dir>` (the claude-arm live-import,
SL-182 PHASE-05) gathers the worker's untracked delta via
`git -C <wt> ls-files --others --exclude-standard`. `--exclude-standard` honours
**all** standard ignore sources, including the **host's global**
`core.excludesFile` (typically `~/.config/git/ignore`).

## Consequence (production — intended, safe)

An untracked working-tree file that the host globally ignores (e.g. `.claude/` is
commonly in a dev's global gitignore) is **silently omitted** from the import —
NOT gathered, NOT imported, and therefore NOT seen by the `classify_import` belt.
This is **safe by omission** (an ignored path is never laundered into the coord
tree) and correct (build junk / logs the worker dropped must not import). The belt
still rejects any **visible** governance touch: a tracked edit surfaces via
`diff HEAD`, and an untracked-but-non-ignored governance file surfaces via
`ls-files --others`. So `.doctrine/state/**` (gitignored runtime tier) and
`.claude/**` (gitignored by design) are safe-by-omission; authored
`.doctrine/slice/**` (tracked / not ignored) is belt-rejected.

## Consequence (tests — a real flake)

Any e2e fixture that models a governance-touch threat with an **untracked**
`.claude/` (or otherwise host-ignored) file is **host-fragile**: it passes where no
global ignore matches and fails (empty delta → the "no delta" halt, NOT the belt
refusal) where one does. Seen live: `import_from_worktree_refuses_claude_touch`
failed on a host whose global gitignore listed `.claude/`.

**Fix:** pin the test repo's excludes to empty so the fixture ignores ambient host
state — `git -C <repo> config core.excludesFile /dev/null` in `init_repo`. Linked
worktrees share the common config, so the pin reaches the gather's `-C <wt>`
invocation. (`tests/e2e_worktree_import.rs`, commit `f7f88a0f`.) A repo-**local**
`core.excludesFile` overrides the **global** one.

See [[mem_019eacd4b2af7a7099792cc3e1671cc5]] (skill source-of-truth) and the
sibling import gotchas [[mem_019ec470c29c79b393e7817f241c5fb7]] (glob `git add`
sweeps foreign untracked on shared main).
