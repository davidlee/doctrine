# IMP-234: Installer: --dev directory-link mode + normal github-marketplace mode

Give the installer two ways to register doctrine's CC plugin, matching the two
real workflows. Empirical mechanics fully mapped this session — see
[[mem.system.claude.plugin-load-model]], [[mem.concept.claude.trust-layers]],
[[mem.pattern.claude.harness-introspection]] (verified on Claude Code 2.1.198).

## Motivation

Current install shells out to `claude plugin marketplace add <github-slug>`,
which loads the plugin **from github over the network** even though the files
are already present (embedded in the `doctrine` bin and/or vendored in the
repo). Two distinct needs:

- **Dev**: iterate on the plugin's own repo; want CC to load **live from local
  `plugins/`**, no clone, no cache, no network.
- **Normal (consumer)**: install into some other project; github marketplace is
  the right distribution channel — but the enable should land in the project's
  **committed `.claude/settings.json`**, not silently at user scope.

## Mode A — `--dev` (directory source, local link)

Point CC at the local files directly. **No `claude` bin needed** — two JSON
writes:

1. `~/.claude/plugins/known_marketplaces.json` (per-machine, path-absolute):
   ```jsonc
   "doctrine": { "source": { "source": "directory", "path": "<abs repo/dist dir>" },
                 "installLocation": "<abs repo/dist dir>",
                 "lastUpdated": "<ISO-8601 string>" }
   ```
   - `<dir>` = the dir holding `.claude-plugin/marketplace.json` + `plugins/`.
   - `lastUpdated` **must be a string** or CC logs `Marketplace configuration
     file is corrupted` and drops the entry.
2. `<project>/.claude/settings.json` (committed):
   ```json
   { "enabledPlugins": { "doctrine@doctrine": true } }
   ```
   - enable key = `<plugins[].name>@<marketplace.name>` from the manifests.

Result: directory source → **live load from repo, no cache dir**. No
`extraKnownMarketplaces`, no `installed_plugins.json` (CC auto-writes the latter
on load).

## Mode B — normal (github marketplace, default)

github distribution requires CC to clone+cache the payload (network), which is
fragile to hand-write. So **shell out to the bin** for registration, but pin the
enable to **project scope**:

```sh
claude plugin marketplace add <github-slug>            # e.g. davidlee/doctrine
claude plugin install doctrine@doctrine --scope project
```

- `--scope project` writes BOTH `extraKnownMarketplaces` and `enabledPlugins`
  into the committed `.claude/settings.json` (verified).
- **Trap this closes:** both subcommands default to `--scope user`. A bare
  `install` writes a user-scope enable and never touches the project — a
  teammate cloning the repo gets nothing, **no error** (silent). The `--scope
  project` flag is the whole point.
- Alternatively the installer could write `.claude/settings.json`
  (`extraKnownMarketplaces` + `enabledPlugins`) itself and only shell out for the
  clone+cache refresh — but `extraKnownMarketplaces` alone does NOT register a
  marketplace (CC skips it as "orphaned, marketplace not registered"), so the
  bin (or a `/plugins` refresh) must still populate `known_marketplaces.json` +
  cache. Simplest to let `marketplace add`+`install --scope project` do all of it.

## Acceptance sketch (for when this is sliced/planned)

- `doctrine install --dev` produces a working live-from-`plugins/` load with zero
  network and zero cache dir; `doctrine install` (default) produces a committed
  project-scope github install.
- Both leave `enabledPlugins` in the **committed** `.claude/settings.json`.
- `--dev` writes a valid `known_marketplaces.json` entry (string `lastUpdated`).
- Neither silently no-ops: if the marketplace fails to register, surface it
  (don't inherit CC's silent-orphan behaviour).
- Idempotent re-runs.

## `--dev` is a posture, not one behaviour — exec-path baking is the second axis

The marketplace source (github vs local dir) is only the first thing that differs
between a dev install and a consumer install. The second, surfaced 2026-07-03: the
**pi/codex MCP + hook exec paths** the installer bakes.

`doctrine claude install` (pi arm) resolves the current PATH doctrine and bakes it
via `--define BIN_PATH=…` into the generated
`.pi/extensions/doctrine/mcp.ts` (`const BIN_PATH = "/home/<user>/.cargo/bin/doctrine"`).
That is a per-machine absolute path shipped into a generated artifact — the same
class of non-portability as a literal home path in a source template (which was
also found and fixed to bare `'doctrine'` in `templates/mcp.ts`).

`--dev` should therefore **not bake an absolute exec path**: leave the consumer
resolving `doctrine` via PATH (or `DOCTRINE_BIN`), so a rebuilt/reinstalled binary
is picked up without a stale hardcode. Consumer (normal) mode may still bake a
resolved path if that's desired for hermeticity — TBD, same open decision tracked
in [[IMP-249]] (bake-nothing vs bake-resolved).

Related non-baked hardcodes the installer does NOT own (manual edits, out of scope
here but part of the same census): `.pi/extensions/doctrine/index.ts` and
`.codex/hooks.json` both carry literal `/home/<user>/.cargo/bin/doctrine`. See the
full census in [[IMP-249]].

Framing: `--dev` = *load and resolve everything live from this working tree, no
network, no per-machine bake*; default = *distributable, self-contained*. Both
marketplace-source and exec-path-baking are instances of that one axis.

## Open questions

- OQ-1: where does `--dev` point `path` — the repo root (dev-on-doctrine) or a
  dir the installer extracts the embedded payload into? Repo root is simplest
  when installing onto doctrine itself.
- OQ-2: absolute `known_marketplaces.json` path is per-machine — fine for a
  per-machine installer run; confirm no committed artifact carries it.
- OQ-3: should `--dev` also support `--scope local` (gitignored
  `settings.local.json`) so a dev enable doesn't dirty the committed file?
