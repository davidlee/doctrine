# Installer dual-mode: marketplace source + exec-path bake axes, --dev sugar

## Context

`doctrine install` (claude arm) currently registers doctrine's CC plugin one
way: `claude plugin marketplace add <github-slug>` + `claude plugin install`,
loading the plugin **from github over the network** even though the files are
embedded in the binary. Two real workflows want two postures:

- **Dev** — iterating on doctrine's own plugin: load live from the local
  `plugins/` tree, no clone, no cache, no network.
- **Consumer** — installing into another project: github marketplace is the
  right distribution channel, but the enable must land in the project's
  **committed** `.claude/settings.json`, not silently at user scope.

Separately, the pi arm bakes a per-machine absolute exec path into the generated
`.pi/extensions/doctrine/mcp.ts` (`generate_mcp_extension`, `boot.rs:1793`, via
`--define BIN_PATH=…`). That literal home path is the same POL-002 non-portability
class as the `templates/mcp.ts` fallback already fixed (dd111479).

Empirical CC plugin-load mechanics for this slice were mapped and verified on
Claude Code 2.1.198 (see IMP-234 + `mem.system.claude.plugin-load-model`,
`mem.concept.claude.trust-layers`).

**Framing (settled in design; see design.md §7).** Analysis collapsed the two
axes to one:

- **Axis 1 — marketplace source (the only real axis):** `directory` (local
  abspath) | `github`. `--dev` ⇒ `source=directory`. Both modes shell out to
  `claude plugin marketplace add <source>` + `claude plugin install
  doctrine@doctrine --scope project`; only the source arg differs. Claude owns
  the trust/blacklist machinery — the installer never hand-writes
  `known_marketplaces.json` (reverses IMP-234's "two JSON writes, no bin").
- **Axis 2 — exec-path bake: dissolved, not a toggle.** On the committed
  `.mcp.json` a baked abspath is a POL-002 breach. Fix: MCP command
  `<abs-exec>` → `${DOCTRINE_BIN:-doctrine}` on the **committed** surface only.
  Gitignored baked surfaces (pi `mcp.ts`, Claude hooks) obey *baked ⟺ gitignored*
  and stay baked (design D2). `DOCTRINE_BIN` is the override.

## Scope & Objectives

1. **Single `--dev` boolean** on `doctrine install` (claude arm). No axis enum
   flags (github is the default source; YAGNI — design D1).
2. **Marketplace (both modes, shell out).** `claude plugin marketplace add
   <source>` then `claude plugin install doctrine@doctrine --scope project`.
   Source: `<github-slug>` (default) | `<detected-abs-project-root>` (`--dev`).
   `--scope project` commits **only** the portable `doctrine@doctrine` enable key
   to `.claude/settings.json` (verified: no `extraKnownMarketplaces`, no abspath).
   The per-machine source lives in user `known_marketplaces.json`, uncommitted.
3. **`--dev` precondition.** Detected project root must hold
   `.claude-plugin/marketplace.json`; absent ⇒ hard error, never a silent github
   fallback. Plugin + marketplace names read from that manifest (→ `doctrine@doctrine`).
4. **MCP command portability fix (POL-002) — committed surface only.**
   `desired_mcp_entry` (`.mcp.json`, committed) emits `${DOCTRINE_BIN:-doctrine}`;
   widen `is_doctrine_mcp_entry` to own the new form (migration). Per the
   invariant *baked ⟺ gitignored*, gitignored baked surfaces (pi `mcp.ts`, Claude
   hooks) are **left baked** — `generate_mcp_extension` untouched. `.mcp.json` is
   the sole committed POL-002 breach.
5. **Qualify + detect on `doctrine@doctrine`** (current code installs bare
   `doctrine`); update presence checks.
6. **Idempotent re-runs** for both modes.

## Non-Goals

- **IMP-249** (`flake.nix:95` jail RO-bind mutable→immutable store path). This
  slice makes IMP-249 *unnecessary as a correctness prop* for bake by choosing
  bake=none, but does not touch the jail bind — IMP-249 stands on its own merit
  (stale-bind hard-break after `cargo install`).
- The other absolute hardcodes IMP-249 censuses that the installer does **not**
  own (`.pi/extensions/doctrine/index.ts`, `.codex/hooks.json`) — manual edits,
  out of scope here.
- Codex MCP registration (IMP-111) — separate surface.

## Affected surface (design-target)

- `src/install.rs` — `--dev` flag on `InstallArgs`; source selection (abs root
  vs github slug); `--dev` precondition + name-read from marketplace.json;
  qualified `doctrine@doctrine` install + presence checks.
- `src/boot.rs` — `desired_mcp_entry` (env-expansion command),
  `is_doctrine_mcp_entry` (own the new form), `generate_mcp_extension` (drop the
  bake); ripples to `plan_mcp*` tests.
- `src/main.rs` — CLI `--dev` wiring.
- `templates/mcp.ts` — **no edit** (runtime `DOCTRINE_BIN || 'doctrine'` fallback
  already present); scope-relevant only.

## Risks / Assumptions / Open Questions

All design OQs resolved (design.md §6). Carried assumptions:

- **A:** CC plugin-load + scope mechanics verified live on CC 2.1.198 (design.md
  §5.4); not re-deriving.
- **A:** `InstallArgs` + forward-step orchestration is the seam — one new bool,
  no parallel installer.
- **R1 (impl):** `is_doctrine_mcp_entry` ownership must recognise the env-expansion
  command or a legacy abs entry reads as foreign / double-registers — migration
  test required.
- **OQ-4 (impl smoke, non-blocking):** confirm `${DOCTRINE_BIN:-doctrine}` in
  `.mcp.json` connects under `/mcp` (docs say yes, mcp.md:384).

## Verification / closure intent

- `doctrine install --dev` (on this repo) → live plugin load from the working
  tree, zero network; `known_marketplaces.json` carries the abs directory source
  (per-machine); `git status` clean of any abspath.
- `doctrine install` (default) → github marketplace + `doctrine@doctrine` enable
  committed to `.claude/settings.json`.
- `--dev` errors clearly when `.claude-plugin/marketplace.json` is absent.
- `.mcp.json` command is `${DOCTRINE_BIN:-doctrine}` (no absolute); pi `mcp.ts`
  unbaked. No absolute host path in any tracked file (INV-1).
- Re-run is a safe no-op for both modes (INV-2); legacy abs `.mcp.json` migrates.
