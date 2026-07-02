# Claude Code project-local plugin load model

How Claude Code (native build, verified **2.1.198**) resolves a project-local
plugin at session start. All claims below are **empirically tested** this
session via nested `claude --debug hooks --debug-file` + observing load — not
inferred from docs. See [[mem.pattern.claude.harness-introspection]].

## Three artifacts, three roles

| Artifact | Scope | Role | Committable |
|---|---|---|---|
| `~/.claude/plugins/known_marketplaces.json` | per-user | **the registry — the gate** | no (absolute paths) |
| `enabledPlugins` in `.claude/settings.json` | settings (user/**project**/local) | per-plugin ON switch | **yes** at project scope |
| plugin files on disk | — | the payload | yes if vendored |

## Load sequence (from the debug log)

1. Read `enabledPlugins`. For each `"<plugin>@<mkt>"`, check `<mkt>` is present
   in **`known_marketplaces.json`**. If not → `Skipping orphaned enabledPlugins
   entry <p>@<mkt>: marketplace not registered` — **silent, no error, plugin
   absent.** This is the "silent blackmark" failure mode.
2. Registered → resolve `<plugin>` in that marketplace's
   `.claude-plugin/marketplace.json` `plugins[]`.
3. **directory** source → read files **live** from `installLocation/plugins/<name>/`;
   **no cache dir is created**. **github/url** source → payload must be
   **cached**; miss → `Plugin not cached … run /plugins to refresh`.
4. Read the plugin's `hooks/hooks.json` / skills / agents → register.

## Findings that overturn the obvious guesses

- **`extraKnownMarketplaces` in settings does NOT register a marketplace.**
  Writing it alone (even project scope) → "orphaned, not registered". The
  operative registry is `known_marketplaces.json`. `claude plugin marketplace
  add` writes *both*, but only the latter is consulted at load.
- **`known_marketplaces.json` entries need a `lastUpdated` STRING** or CC logs
  `Marketplace configuration file is corrupted: <mkt>.lastUpdated: Invalid
  input: expected string, received undefined` and refuses the entry.
- **`installed_plugins.json` is CC bookkeeping, auto-written** — never a
  required input. CC logged `Added <p>@<mkt> with scope project` itself once the
  marketplace resolved; it also auto-populates on load.
- **directory source ⇒ no cache, live load.** A `cache/<mkt>/` dir, if present,
  is leftover from a prior github install; the live path is the repo.
- **Both `claude plugin install` and `marketplace add` default to `--scope
  user`.** A bare `claude plugin install` writes a *user*-scope enable and never
  touches the project — teammates cloning get nothing, no error. Use
  `--scope project` to land in the committed `.claude/settings.json`.

## Direct-write recipe (no bin), directory source — dev

Two files, nothing else:

```jsonc
// ~/.claude/plugins/known_marketplaces.json  (per-machine — installer writes it)
"doctrine": { "source": { "source": "directory", "path": "<abs-dir>" },
              "installLocation": "<abs-dir>", "lastUpdated": "2026-07-03T00:00:00.000Z" }
```
```jsonc
// <project>/.claude/settings.json  (committed)
{ "enabledPlugins": { "doctrine@doctrine": true } }
```

- `<abs-dir>` = dir holding `.claude-plugin/marketplace.json` + `plugins/`.
- enable key = `<plugins[].name>@<marketplace.name>` from the manifests, **not**
  the repo slug.
- No `extraKnownMarketplaces`, no `installed_plugins.json`, no cache needed.

## Commit vs per-machine split (for an installer)

- **Commit to repo:** `enabledPlugins` (project `.claude/settings.json`),
  vendored plugin files, `.claude-plugin/marketplace.json`.
- **Write per-machine (installer, not committed, path-absolute):** the
  `known_marketplaces.json` registry entry. The registry can't be committed
  (absolute paths) and `enabledPlugins` without it fails silently — that
  asymmetry is the whole trap.
- **Distribution (github source):** CC must clone
  (`~/.claude/plugins/marketplaces/<mkt>/`) + cache
  (`cache/<mkt>/<plugin>/<ver>/`) over the network. Hand-writing that is
  fragile; let the bin (`marketplace add <repo> && install --scope project`) or
  a `/plugins` refresh do it.

Related: [[mem.concept.claude.trust-layers]] (enablement is one of three trust
layers), [[mem.fact.claude.worktreecreate-hook-fires]].
