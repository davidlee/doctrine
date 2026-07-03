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

**Framing (decided with user):** `--dev` is not one behaviour. It is **sugar**
over two orthogonal explicit axes, each set to its "live" pole:

- **Axis 1 — marketplace source:** `directory` (local link) | `github`.
- **Axis 2 — exec-path bake:** `none` (resolve `doctrine` via PATH /
  `DOCTRINE_BIN`) | `resolved` (absolute).

`--dev` ⇒ `source=directory` + `bake=none`. Default ⇒ `source=github` + a
consumer bake posture that is **not required to be an absolute host path** —
POL-002 forbids baking one machine's home into a shipped artifact.

## Scope & Objectives

1. **Two explicit flags** on `doctrine install` (claude arm) for the two axes,
   plus **`--dev`** as sugar expanding to `source=directory` + `bake=none`.
   Explicit flags win over the macro when both are given (or `--dev` +
   contradictory explicit flag is an error — decide in `/design`).
2. **Mode A (`source=directory`)** — no `claude` bin needed; two JSON writes:
   - `~/.claude/plugins/known_marketplaces.json` — a `directory`-source entry
     with an **absolute** `path`/`installLocation` and a **string** `lastUpdated`
     (non-string ⇒ CC drops the entry as corrupted).
   - `<project>/.claude/settings.json` — `enabledPlugins` with key
     `<plugin.name>@<marketplace.name>` read from the manifests.
   Result: live load from the working tree, no cache dir.
3. **Mode B (`source=github`, default)** — shell out
   `claude plugin marketplace add <slug>` + `claude plugin install
   doctrine@doctrine --scope project`, pinning the enable to **committed**
   project scope (both default to `--scope user` — the silent-orphan trap this
   closes).
4. **Axis 2 (`bake=none`)** — pi-arm `generate_mcp_extension` emits an exec that
   resolves `doctrine` via PATH / `DOCTRINE_BIN` rather than a baked absolute, so
   no host path is shipped and nix need not paper over a stale bake.
5. **Non-silent failure** — if the marketplace fails to register, surface it;
   do not inherit CC's silent-orphan behaviour.
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

## Affected surface

- `src/install.rs` — flag surface, claude-arm marketplace registration (Modes
  A/B), the two JSON writes, idempotency, failure surfacing.
- `src/boot.rs` — `generate_mcp_extension` / `plan_mcp_extension` bake axis
  (`bake=none` path).
- `templates/mcp.ts` — already bare-`doctrine` fallback; confirm the generated
  artifact honours bake=none.
- `src/main.rs` — CLI arg wiring for the new flags.

## Risks / Assumptions / Open Questions

- **A:** CC plugin-load mechanics in IMP-234 are ground truth (CC 2.1.198); not
  re-verifying.
- **A:** `InstallArgs` + forward-step orchestration is the seam — new bools, no
  parallel installer.
- **OQ-1 (design):** `--dev` directory `path` target — repo root
  (dev-on-doctrine) vs a dir the installer extracts the embedded payload into.
- **OQ-3 (design):** `--dev` enable scope — committed `settings.json` vs
  gitignored `settings.local.json` (avoid dirtying the committed file for a dev
  enable). May want a `--scope` passthrough.
- **OQ (design):** flag spelling/surface for the two axes; `--dev` + explicit
  contradiction handling (precedence vs error).
- **OQ (design):** default consumer bake posture — bake=none for everyone, or a
  non-home resolved path when hermeticity is wanted (IMP-234 left this TBD).
- **Risk:** `~/.claude/plugins/known_marketplaces.json` is per-machine + absolute
  — confirm no committed artifact carries it (OQ-2).

## Verification / closure intent

- `doctrine install --dev` → working live-from-`plugins/` load, zero network,
  zero cache dir; valid `known_marketplaces.json` (string `lastUpdated`).
- `doctrine install` (default) → committed project-scope github install;
  `enabledPlugins` in the committed `.claude/settings.json`.
- Neither mode silently no-ops on registration failure.
- Re-run is a safe no-op (idempotent) for both.
- pi-arm generated `mcp.ts` under bake=none carries no absolute host path.
