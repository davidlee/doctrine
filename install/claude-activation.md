<!-- Shipped reference (ADR-005 PULL tier). Edit the source in
     `install/claude-activation.md`. Published, not projected (ADR-019): there is
     no copy on disk in an installed project — read it with `doctrine library show
     reference/claude-activation.md`. Explains how Claude Code activation works
     since SL-250 retired the plugin/marketplace install path — it never
     reproduces `doctrine --help`; ask the CLI for exact flags. -->

# Claude Code activation

Since SL-250, `doctrine install` activates Claude Code by writing hooks
straight into a Claude settings file — no marketplace registration, no plugin
install. This doc explains what gets written, where, in what command form, and
what changes if you are mid-cutover from the old plugin path or run under a
managed-settings policy that blocks direct writes.

## What doctrine writes

`doctrine install` (and `doctrine boot install` / `memory sync install`, which
share the same merge core) writes doctrine's hook entries directly into one
Claude settings JSON file under `.claude/`. The merge is narrow and
non-destructive: it edits only the `hooks.<event>` paths doctrine owns and
preserves every foreign hook and unrelated key already in the file.

**No restart, no reload command.** Claude Code hot-reloads its settings files
— hooks included — so a fresh install (or a re-run that changes an entry)
takes effect on the next matching event, not on the next session. This is the
direct-write path's main advantage over the retired plugin path, which needed
`/reload-plugins` to pick up a change.

## The scope key: `[install] claude-settings-scope`

Which of the two Claude settings files doctrine writes is controlled by one
`doctrine.toml` key:

```toml
[install]
claude-settings-scope = "project"   # or "local"
```

- **`project`** (the default) → `.claude/settings.json`, committed and shared
  by everyone who checks out the repo.
- **`local`** → `.claude/settings.local.json`, private to one checkout and
  conventionally gitignored.

Remedy for a client-project collaborator: if you're contributing to a project
you don't want your local hook wiring committed into, set
`claude-settings-scope = "local"` (or pass no override and use the local file
directly) so your activation stays out of the shared diff.

## Command form and `DOCTRINE_BIN`

The two scopes write different command forms, because only one of the two
files is safe to commit:

- **`project`** (committed) writes `${DOCTRINE_BIN:-doctrine}` — never a host
  absolute path in a tracked file (SL-195, POL-002). If `doctrine` is on every
  collaborator's `PATH`, the entries just work as `doctrine <args>`. If it
  isn't — a harness sandbox where the binary lives somewhere non-standard, for
  example — **set `DOCTRINE_BIN`** to the absolute path before the harness
  runs; that's the same override `.mcp.json` has taken since SL-195, and the
  hook commands honour it identically.
- **`local`** (gitignored) writes the baked absolute path directly, since the
  file never leaves the checkout that produced it.

**POSIX-shell boundary.** `${DOCTRINE_BIN:-doctrine}` is POSIX parameter
expansion, expanded by whatever shell runs the hook command (`sh -c` on
Linux/macOS, Git Bash on Windows). Doctrine does not target a non-POSIX
Windows shell for this form.

## The cutover: install first, then disable the plugin

If you were previously running doctrine's Claude plugin (`enabledPlugins` +
a marketplace registration), the prescribed order is:

1. **Run the install that writes the direct hooks** (`doctrine install` or
   `doctrine boot install`).
2. **Then** disable the doctrine plugin by hand.

Between the two acts every hook fires twice — that's the intended, bounded
cost. Do **not** reverse the order: disabling the plugin first leaves the repo
with no activation at all, which is worse — `isolation: worktree` teardown is
conditional on `WorktreeCreate` firing, so an inert hook silently changes
dispatch's semantics rather than just double-firing.

The plugin's own activation state (`enabledPlugins`, marketplace
registrations) is something doctrine reads and writes nothing of — disabling
it is a manual step Claude Code's plugin UI owns, not a `doctrine` command.
Skip it and every hook keeps firing twice indefinitely; that's a nuisance, not
a correctness problem, but there's no reason to leave it.

## The sweep: what it touches, and what it never touches

A re-run of `doctrine install` / `boot install` also sweeps the settings file
**doctrine did not choose** for entries doctrine itself owns — e.g. switching
`claude-settings-scope` from `local` to `project` evicts doctrine's stale
entries from `.claude/settings.local.json` once the project file carries them.
This is a description of existing behaviour, not an instruction to run
anything extra:

- **In scope:** doctrine-owned hook entries (recognised by their command
  signature), and only in the *other* of doctrine's two settings files.
- **Never touched:** anything doctrine did not write itself — a hand-authored
  hook, a third-party plugin's entry, any other key in either file.

If an install run reports evicting stale entries, that's this sweep at work,
not doctrine reclaiming a hook you placed by hand.

## Escape hatch: `strictPluginOnlyCustomization`

Some managed-settings policies block hooks from user- and project-scoped
files, allowing them only from plugins (Claude Code's
`strictPluginOnlyCustomization` setting). Direct-write cannot activate under
that policy — `doctrine install` will write the file, but Claude Code will not
load hooks from it. The remedy is the two commands the installer used to run
automatically, now run by hand:

```sh
claude plugin marketplace add davidlee/doctrine
claude plugin install doctrine@doctrine --scope project
```

**This is not a subset of what direct-write gives you — it's a different
trade.** Direct-write sheds two plugin-only failure modes (an orphaned
marketplace registration pointing at a stale source, and a plugin silently
blocklisted) but still exposes `disableAllHooks`, `allowManagedHooksOnly`, and
folder-trust gating exactly as before, and it *acquires* exposure to
`strictPluginOnlyCustomization` — a policy direct-write cannot satisfy at all.
So: fewer failure modes overall, not a strict improvement on every axis. If
your environment enforces `strictPluginOnlyCustomization`, the plugin path
above is not a fallback for the cautious — it's the only path that works.
