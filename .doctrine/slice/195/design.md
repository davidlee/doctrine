# Design SL-195: Installer dual-mode — `--dev` marketplace source + MCP-command portability

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

> **Scope note.** The slice title names two axes (marketplace source, exec-path
> bake). Design analysis collapsed the second: baking is *strictly dominated* by
> runtime env-resolution and, on the one committed MCP surface, is a POL-002
> violation. So "axis 2" is not a `--dev` toggle — it is a **portability fix**
> (bake absolute → `${DOCTRINE_BIN:-doctrine}`). The only real axis is the
> marketplace source. See §7 D2.

## 1. Design Problem

`doctrine install` (claude arm) registers doctrine's CC plugin exactly one way:
`claude plugin marketplace add <github-slug>` — loading from github over the
network even though the plugin files are embedded/vendored. Two workflows want
two postures:

- **Consumer (default):** github marketplace is the right distribution channel;
  the enable must land in the project's committed `.claude/settings.json`.
- **Dev (`--dev`):** iterating on doctrine's own plugin — load live from the
  local working tree, no clone, no network.

Separately, the installer bakes a per-machine absolute exec path into two MCP
registration surfaces (`.mcp.json` `command`, pi `mcp.ts` `BIN_PATH`). On the
**committed** `.mcp.json` this ships one machine's path to every teammate — a
POL-002 breach (the bug IMP-249 hand-patched and warned would re-bake).

## 2. Current State

- **Marketplace (`src/install.rs:410–520`).** For the `claude` agent: detect
  marketplace (`claude plugin marketplace list | grep doctrine`), if absent
  `claude plugin marketplace add <repo>`; detect plugin, if absent `claude plugin
  install doctrine --scope project`. `repo` = `doctrine.toml` `install.repo`
  (default `davidlee/doctrine`). Bare plugin name `doctrine` (unqualified).
- **`.mcp.json` (`src/boot.rs:1468` `desired_mcp_entry`).** Writes
  `{"command": "<abs-exec>", "args": ["serve","--mcp"]}` into the **committed**
  `.mcp.json`. Ownership recognised by `is_doctrine_mcp_entry` via
  `command.file_name() == "doctrine"` (boot.rs:1479).
- **pi `mcp.ts` (`src/boot.rs:1791` `generate_mcp_extension`).** Replaces the
  `declare const BIN_PATH: string;` marker with `const BIN_PATH = "<abs-exec>";`.
  The template already falls back to `DOCTRINE_BIN || 'doctrine'` at runtime when
  `BIN_PATH` is undefined (`templates/mcp.ts:338–341`).
- **Flags (`InstallArgs`, install.rs:155).** No `--dev`.

## 3. Forces & Constraints

- **POL-002** — no host absolute in a committed/shipped artifact. Binds
  `.mcp.json` (committed) and any committed `extraKnownMarketplaces`.
- **SPEC-009** — idempotent install; files written when absent, re-run is a safe
  no-op; foreign entries preserved.
- **Invariant (locked this design):** *baked/per-machine ⟺ uncommitted;
  committed ⟺ portable.* No absolute in a tracked file.
- **Empirical CC mechanics** verified live on CC 2.1.198 (see §5.4) — ground
  truth over recollection.

## 4. Guiding Principles

- Let the official `claude` tool own the marketplace **trust/blacklist**
  machinery; never hand-write `known_marketplaces.json`.
- One documented override across all harnesses: `DOCTRINE_BIN`. No per-file bake.
- `--dev` is a thin sugar: exactly one argument differs from default.

## 5. Proposed Design

### 5.1 System Model

`--dev` and default share **one** shell-out flow; only the `marketplace add`
source argument differs:

```
default:  claude plugin marketplace add <github-slug>          # install.repo
--dev:     claude plugin marketplace add <detected-abs-root>    # holds .claude-plugin/marketplace.json
both:      claude plugin install <plugin>@<marketplace> --scope project
```

`<plugin>` and `<marketplace>` are **read from `.claude-plugin/marketplace.json`**
(`plugins[].name` and top-level `name`), not hardcoded — both resolve to
`doctrine`, so the enable key `doctrine@doctrine` is **source-agnostic** and
identical across modes.

### 5.2 Interfaces & Contracts

- **Flag:** a single boolean `--dev` on `doctrine install` (§7 D1). Threaded via
  `InstallArgs { dev: bool, .. }`.
- **`--dev` precondition:** the detected project root must contain
  `.claude-plugin/marketplace.json`. Absent → hard error (clear message), never a
  silent fallback to github.
- **MCP command — committed surface only.** `.mcp.json` (committed):
  `{"command": "${DOCTRINE_BIN:-doctrine}", "args": ["serve","--mcp"]}`. pi
  `mcp.ts` is **gitignored → stays baked** (the invariant: baked ⟺ gitignored);
  `generate_mcp_extension` is untouched. Only the *committed* file, which is the
  sole POL-002 breach, loses the abspath. See §7 D2 and §10 (adversarial pass).

### 5.3 Data, State & Ownership

| Artifact | Tier | Content | Owner |
|---|---|---|---|
| `.claude/settings.json` `enabledPlugins` | committed | `doctrine@doctrine: true` (portable, both modes) | `claude plugin install --scope project` |
| `known_marketplaces.json` | per-machine (home) | source: github slug **or** directory abspath | `claude plugin marketplace add` |
| `.mcp.json` | committed | `command: ${DOCTRINE_BIN:-doctrine}` (portable) | `boot::desired_mcp_entry` (hand-write) |
| `.pi/extensions/doctrine/mcp.ts` | gitignored | **baked abspath — unchanged** (invariant: baked ⟺ gitignored) | `boot::generate_mcp_extension` (untouched) |
| `.claude/settings.local.json` hooks | gitignored | baked abspath — unchanged (same rationale) | `boot` hook wiring (untouched) |

**Verified (CC 2.1.198):** `install --scope project` writes **only** the
`enabledPlugins` key — **no `extraKnownMarketplaces`, no abspath** — so the
committed file is portable in both modes; the per-machine directory source stays
in `known_marketplaces.json`. This is what makes `--scope project` POL-002-safe
for `--dev`.

### 5.4 Lifecycle, Operations & Dynamics

Empirical facts the design rests on (live CC 2.1.198, §10 R-note carries the
transcript pointers):

1. `claude plugin marketplace add ./` (or abspath) registers a **Directory**
   source, **absolutized** (`list` shows `Source: Directory (/abs)`).
2. Marketplace **name** derives from `marketplace.json` `name`, **not** the
   dirpath. Enable key = `<plugin.name>@<marketplace.name>`.
3. `marketplace add` and `install` **default to user scope** (the IMP-234 silent
   trap); `install --scope project` writes the enable to committed
   `.claude/settings.json` and **nothing else**.
4. `.mcp.json` supports `${VAR:-default}` env-expansion in `command`/`args`
   (mcp.md:384). `.claude/settings.local.json` does **not** define MCP servers —
   MCP server defs live only in `.mcp.json` (project) or `~/.claude.json`
   (local/user) (mcp.md:301–310).
5. `claude mcp add` name-slot rejects `${...}` and double-quotes shell-expand it
   → **hand-write `.mcp.json`, do not shell out** for the MCP server.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1:** no absolute host path in any git-tracked file after install.
- **INV-2:** re-run converges to current — **refresh, not skip**. Baked gitignored
  surfaces (pi `mcp.ts`, hooks) already compare-and-regenerate on a changed exec
  path (boot.rs:1818). The marketplace step must likewise refresh a **stale
  source** (moved repo / changed slug), not skip-because-name-present — see R4.
- **INV-3 (dev refresh):** reinstalling `--dev` after a repo move / rebuild must
  update the registered directory source (and any baked path) to the current
  location. Directory sources are live-loaded, so *content* edits never need
  reinstall; only the registered *path* can go stale. INV-2 + R4 carry this.
- **EDGE:** `--dev` in a repo lacking `.claude-plugin/marketplace.json` → error.
- **EDGE:** an existing `.mcp.json` written by the *old* absolute-baking installer
  must be recognised as ours and refreshed to the env-expansion form — see §8 R1.

## 6. Open Questions & Unknowns

- **OQ-1 (path target) — RESOLVED (§7 D3):** `--dev` source = detected absolute
  project root.
- **OQ-2 (abspath vs literal) — RESOLVED:** claude absolutizes the dir; installer
  passes the resolved abs root.
- **OQ-3 (enable scope) — RESOLVED:** `--scope project` for both; verified it
  commits only the portable enable key, not the abspath.
- **OQ-4 (impl-time, not a blocker):** does hand-written `${DOCTRINE_BIN:-doctrine}`
  in `.mcp.json` `command` connect under `/mcp`? Docs say yes (mcp.md:384); smoke
  test at implementation.

## 7. Decisions, Rationale & Alternatives

- **D1 — Flag surface: single `--dev` boolean.** One axis, two values, github the
  obvious default. *Alt:* explicit `--marketplace-source directory|github` for
  future `url`/`git-subdir` sources — rejected as speculative (YAGNI); a source
  flag can grow later without breaking `--dev`.
- **D2 — Fix the committed surface only; obey the invariant *baked ⟺
  gitignored*.** The POL-002 breach is exclusively the **committed** `.mcp.json`
  baking a per-machine abspath. Fix = `${DOCTRINE_BIN:-doctrine}` (portable,
  overridable). **Gitignored** baked surfaces (pi `mcp.ts`, Claude hooks in
  `settings.local.json`) are per-machine and POL-002-safe → **left baked**;
  unbaking them selectively would contradict the hooks and is scope creep. Their
  install-time-snapshot staleness (cargo rename / jail RO-bind) is IMP-249's
  domain, not this slice's. *Alt A (uniform-unbake everything):* rejected —
  bigger, IMP-249-adjacent, and gains nothing POL-002 needs. *Alt B (per-machine
  Claude MCP in `~/.claude.json` via `claude mcp add --scope local`):* rejected —
  shell-out fragility (name-slot rejects `${}`), mutates global config;
  committed+portable `.mcp.json` is simpler. **This corrects an earlier draft that
  said "unbake pi mcp.ts" — see §10.**
- **D3 — `--dev` source = detected abs project root, gated on
  `.claude-plugin/marketplace.json` presence.** Live edits, no extraction, no
  cache — the point of dev mode. *Alt:* extract embedded payload to a dir —
  rejected: only serves a "`--dev` into a foreign repo" case that isn't real.
- **D4 — Let `claude` own marketplace trust; hand-write only `.mcp.json`.**
  Directory-source trust/blacklist handling is claude's; hand-writing
  `known_marketplaces.json` was painful and fragile in testing.

## 8. Risks & Mitigations

- **R1 — Ownership check breaks on the new command form.**
  `is_doctrine_mcp_entry` matches `command.file_name() == "doctrine"`;
  `${DOCTRINE_BIN:-doctrine}` file-names to `doctrine}` (or the literal),
  so an existing entry would read as *foreign* and never refresh — and a
  half-migrated `.mcp.json` could double-register. **Mitigation:** widen the
  ownership predicate to also match the env-expansion command literal; keep the
  `args == ["serve","--mcp"]` arm. Cover with a migration test (old abs entry →
  refreshed to env form).
- **R2 — Bare `doctrine` install unqualified.** Current `claude plugin install
  doctrine` may fail or mis-resolve; the enable key is `plugin@marketplace`.
  **Mitigation:** qualify to `doctrine@doctrine`; update presence checks to the
  qualified key.
- **R3 — `--dev` idempotency.** Re-running `marketplace add` on an already-added
  directory source. **Mitigation:** presence guard keyed on marketplace **name**
  (`doctrine`) — but see R4: name-presence alone is insufficient when the source
  is stale.
- **R4 — stale marketplace source on reinstall (INV-2/INV-3).** The current guard
  (`if !has_marketplace { add }`, install.rs:423) skips when a `doctrine`
  marketplace exists, retaining a **stale source** after a repo move or a changed
  `install.repo` slug. **Mitigation:** compare the *registered source* (from
  `marketplace list`, which prints `Source: Directory (/abs)` / the git slug)
  against the intended source; on mismatch, refresh it. **Impl-time empirical
  (for `/plan`):** determine the refresh verb — does `claude plugin marketplace
  add <newsrc>` overwrite an existing name's source, or is `remove`+`add`
  required (remove uninstalls plugins, plugin-marketplaces.md:988, so re-install
  after)? `marketplace update` refreshes *content at the same path*, not a
  relocation. Probe live before choosing.

## 9. Quality Engineering & Validation

- **Unit (pure):** `desired_mcp_entry` emits the env-expansion command;
  `is_doctrine_mcp_entry` recognises both the legacy abs form and the env form;
  `plan_mcp` refreshes a stale abs entry to the env form (migration). pi
  `generate_mcp_extension` is **untouched** — its existing bake tests stay green
  unchanged (behaviour-preservation).
- **Unit:** `--dev` source-selection picks abs root vs github slug; `--dev`
  precondition error when `.claude-plugin/marketplace.json` absent.
- **Behaviour-preservation:** existing `plan_mcp_*` / install tests stay green
  except the intentional command-form change (update fixtures).
- **Reinstall-refresh (INV-2/INV-3):** unit — source-mismatch detection selects
  refresh vs skip; a stale directory source (simulated `marketplace list` output
  with an old abspath) triggers refresh. Manual — `--dev`, move the repo, reinstall,
  confirm the registered source updates to the new abspath.
- **Manual smoke (OQ-4, R4 verb):** hand-write env-form `.mcp.json`, confirm `/mcp`
  connects; `doctrine install --dev` → live plugin load, no network; `git status`
  clean of any abspath; probe the marketplace-source refresh verb live.

## 10. Review Notes

### Adversarial self-pass (author)

- **F1 — scope coherence of the bake fix (accepted, design revised).** The draft
  fixed the committed `.mcp.json` *and* proposed unbaking the gitignored pi
  `mcp.ts`. Checking boot's hook targets showed Claude hooks bake the same
  abspath into gitignored `.claude/settings.local.json` and are left alone.
  Unbaking pi `mcp.ts` (also gitignored) but not the hooks is inconsistent, and
  contradicts the locked invariant *baked ⟺ gitignored*. **Resolution:** fix the
  committed `.mcp.json` only; `generate_mcp_extension` untouched; pi `mcp.ts` and
  hooks stay baked. Slice shrinks; boot.rs change = `desired_mcp_entry` +
  `is_doctrine_mcp_entry`. (D2 rewritten.)
- **F2 — ownership predicate migration (accepted → R1).** `is_doctrine_mcp_entry`
  keys on `command.file_name() == "doctrine"`; the env-expansion command
  file-names to `${DOCTRINE_BIN:-doctrine}` (has no `/`). Without widening, an
  env-form entry reads as foreign (never refreshed) while a legacy abs entry
  (file_name `doctrine`) still matches — risking a half-migrated double-register.
  Predicate must OR-in the env literal; migration test required.
- **F3 — flag-surface approval (open).** D1 picks a bare `--dev` boolean over an
  explicit `--marketplace-source`. Author's call, pending user sign-off.

### External / inquisition pass

<!-- lands here -->

