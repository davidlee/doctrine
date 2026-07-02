# Empirically introspect Claude Code harness behavior

Don't infer CC hook/plugin/trust behavior from docs — probe the running binary.
Two complementary techniques, both used successfully this session.

## 1. Live hook-registration debug (authoritative for "did it register/fire?")

```sh
claude --debug hooks --debug-file /path/to.log -p 'print OK'
grep -iE 'Read hooks.json|Registered [0-9]+ hooks|orphaned|not cached|Skipping' /path/to.log
```

- Reveals: which plugins' `hooks.json` were read (`enabled=true/false` + the
  exact path — cache vs live repo), total hooks registered, and silent skips
  (`Skipping orphaned enabledPlugins entry …: marketplace not registered`).
- **Nested `claude` works inside the doctrine bwrap jail** (exit 0, real output)
  — a nested session is a valid, non-destructive probe.
- To detect a specific plugin hook FIRING, give it a `SessionStart` command that
  `echo`s a unique sentinel, then grep the log for the sentinel.
- `--include-hook-events` and subcommand `claude doctor` also exist.

## 2. Grep the compiled binary

The nix/native `claude` `bin/claude` is a tiny bash wrapper; the real program is
`bin/.claude-wrapped`, a ~245MB **bun-compiled ELF** with JS embedded. Grep it:

```sh
B=$(dirname $(readlink -f $(which claude)))/.claude-wrapped
grep -a -o '.\{80\}WorktreeCreate.\{80\}' "$B" | sort -u     # context windows
strings -n 8 "$B" | grep -oE '"hasTrustDialogAccepted"|…'    # config-key tokens
grep -a -o 'function <name>([^}]*}' "$B"                     # minified fn bodies
```

- Minified **symbol names are unstable across versions** (`rj` meant different
  things in 2.1.197 vs 2.1.198) — don't trust cross-version symbol tracing;
  re-derive per build. String literals and config-key names are stable.
- Distinguish enforcement from telemetry: a `Set([...])` of event names may feed
  an analytics/enumeration helper, not a gate (see
  [[mem.fact.claude.worktreecreate-hook-fires]] — the "plugin hook allowlist"
  turned out to be telemetry).

## 3. Trigger-path probes for worktree hooks

- `isolation: worktree` subagent → fires `WorktreeCreate`; lands at the
  hook-provided path.
- native `claude -w` → plain `git worktree add` at `.claude/worktrees/<name>`,
  **bypasses** the hook. Wrong probe for registration.
- Clean up probe worktrees: `git worktree remove --force <path>; git worktree prune`.

## Guardrails when probing

- Snapshot before mutating global state (`~/.claude/plugins/known_marketplaces.json`,
  `installed_plugins.json`), restore after — CC auto-writes `installed_plugins.json`
  on load, so it accrues test entries; scrub with `jq`.
- CC validates `known_marketplaces.json` entries: needs `lastUpdated` as a
  **string** or logs `Marketplace configuration file is corrupted`.

Reinforces the global rule: verify harness behavior empirically, don't infer.
See [[mem.system.claude.plugin-load-model]], [[mem.concept.claude.trust-layers]].
