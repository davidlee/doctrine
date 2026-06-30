# IMP-215: Auto-execute Claude plugin install during `doctrine install`

`doctrine install` currently prints delegated `/plugin` chat-syntax commands.
That's useless in the terminal where `doctrine install` runs. Instead:

## Requirements

1. **Detect current state** (only when `claude` is on PATH):
   - Marketplace source: `claude plugin marketplace list` — substring-match for repo owner
   - Plugin installed: `claude plugin list --json` — parse for `"name": "doctrine"` entry
   - If both present → offer `claude plugin update doctrine@doctrine -s project`
   - If missing → offer the appropriate install command

2. **Auto-execute with prompt** (when `claude` on PATH):
   - Prompt `Run "claude plugin marketplace add {repo} -s project"? [y/n/A]`
   - Prompt `Run "claude plugin install doctrine@doctrine -s project"? [y/n/A]`
     (or `update` if already installed)
   - `y` → execute, check exit code
   - `A` → execute this + all remaining without further prompts
   - `n` → skip this one
   - If any command fails → print manual fallback for that step

3. **Fallback to print** (when `claude` NOT on PATH):
   - Print the `claude plugin ...` CLI form (not `/plugin` chat syntax)
   - Already threaded: `-s project` and `[install].repo` (items 1 & 2 done)

## Claude plugin CLI surface (confirmed from docs)

```
claude plugin marketplace add {repo} -s project     # add marketplace source
claude plugin install doctrine@doctrine -s project   # install plugin
claude plugin update doctrine@doctrine -s project    # update (no-op if latest)
claude plugin marketplace list                       # list marketplace sources
claude plugin list --json                            # structured plugin listing
```

## Version management (from Claude docs)

- `claude plugin update` SKIPS if installed version matches latest — it's already
  idempotent. We don't need our own version comparison.
- `claude plugin install` docs don't explicitly say it's idempotent, but the
  version-cache mechanism means re-running it fetches the same version. Use
  `update` for already-installed plugins; `install` for first-time.
- doctrine currently doesn't set `version` in `plugin.json` (uses git SHA), so
  every commit is a "new version" — `update` will always pull latest.

## Design decisions

| # | Decision |
|---|---|
| D1 | Use `claude plugin update` for installed plugins (handles version internally). No embedded minimum version needed |
| D2 | `claude plugin install` for first install; `claude plugin update` for already-installed |
| D3 | Parse `--json` for installed check (structured, reliable). Substring-match for marketplace (simpler) |
| D4 | New impure `try_auto_install_claude_plugin()` in `install.rs`; `post_install_instructions()` stays pure fallback |
| D5 | Separate PATH check (`which claude`). `.claude/` dir check says "project uses Claude", PATH says "can execute CLI" |
