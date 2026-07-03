# Notes SL-195: Installer dual-mode — `--dev` marketplace source + `.mcp.json` POL-002 fix

Durable per-slice scratchpad — tracked in git. Design-stage handoff notes for the
fresh agent running the GPT inquisition + `/plan`.

## Status

- Lifecycle: **design → plan** (locked this session).
- Design authored, self-adversarially reviewed, reconciled with scope. Commits:
  `3dae4594` (scope) · `4f624788` (design lock) · `b9935346` (reinstall-refresh).
- **No code written.** Pure design + governance. `doctrine check gate` N/A (no code
  touched).
- `.doctrine/` changes committed promptly (path-limited to `.doctrine/slice/195`).
- **Pending per user's plan:** fresh agent raises a **GPT (codex) inquisition** on
  the locked design, integrates findings using these notes, *then* executes.

## What the design landed on (the short version)

A long clarifying loop collapsed a two-axis slice to one real axis + a POL-002 fix.

1. **One axis — marketplace source.** Both modes shell out:
   `claude plugin marketplace add <SOURCE>` + `claude plugin install
   doctrine@doctrine --scope project`. Only SOURCE differs:
   - default: `<github-slug>` (`install.repo`, e.g. `davidlee/doctrine`)
   - `--dev`: `<detected-abs-project-root>` (must hold `.claude-plugin/marketplace.json`)
   Claude owns trust/blacklist — installer **never** hand-writes
   `known_marketplaces.json` (reverses IMP-234's "two JSON writes, no bin").
2. **`--dev` = single boolean** (design D1/F3 — see Open below).
3. **Bake axis dissolved → POL-002 fix on the committed surface only.**
   `.mcp.json` `command`: `<abs-exec>` → `${DOCTRINE_BIN:-doctrine}`.
   Gitignored baked surfaces (pi `mcp.ts`, hooks) **stay baked** per the invariant
   *baked ⟺ gitignored*; `generate_mcp_extension` untouched.

## Edit set (design-target selectors)

- `src/install.rs` — `--dev` on `InstallArgs`; SOURCE selection; `--dev`
  precondition + read plugin/marketplace names from `.claude-plugin/marketplace.json`;
  qualify install to `doctrine@doctrine`; **source-mismatch refresh** (R4); update
  presence checks.
- `src/boot.rs` — `desired_mcp_entry` (env-expansion command);
  `is_doctrine_mcp_entry` (own the new form + legacy abs — migration); ripples to
  `plan_mcp*` tests. **`generate_mcp_extension` NOT touched.**
- `src/commands/cli.rs` — CLI `--dev` wiring.
- `templates/mcp.ts` — **no edit** (runtime `DOCTRINE_BIN || 'doctrine'` already at
  line 341); scope-relevant only. Correct as-is.

## Verified CC mechanics (live, 2.1.198 — ground truth, don't re-derive)

- `claude plugin marketplace add ./|<abs>` → **Directory** source, **absolutized**
  (`list` → `Source: Directory (/abs)`).
- Marketplace **name** = `marketplace.json` `name` field (NOT dirpath). Enable key
  = `<plugin.name>@<marketplace.name>` = `doctrine@doctrine`, **source-agnostic**
  (same for github + directory).
- `marketplace add` + `install` default to **user scope** (IMP-234 silent trap).
- `install --scope project` writes **only** `enabledPlugins` to committed
  `.claude/settings.json` — **no `extraKnownMarketplaces`, no abspath**. This is
  why `--scope project` is POL-002-safe for `--dev`.
- `.mcp.json` supports `${VAR:-default}` env-expansion in `command`/`args`
  (mcp.md:384). `.claude/settings.local.json` does **NOT** define MCP servers —
  defs live only in `.mcp.json` (project) or `~/.claude.json` (local/user)
  (mcp.md:301–310). Corrected the user's initial settings.local.json assumption.
- `claude mcp add` first positional is the **name**, validated `[A-Za-z0-9_-]` →
  **rejects `${...}`**; double-quotes shell-expand it → **hand-write `.mcp.json`,
  don't shell out** for the MCP server.
- Docs were **not cached** — fetched `mcp.md`, `mcp-quickstart.md`, `settings.md`,
  `claude-directory.md`, `plugin-marketplaces.md` into `docs/claude/` via the
  base URL in `fetch.sh`. (These are now on disk for the fresh agent.)

## Key facts / gotchas the fresh agent must respect

- **POL-002 boundary:** the committed `.mcp.json` is the *sole* file boot bakes an
  abspath into (boot.rs:537 comment confirms committed; `SETTINGS_REL` =
  `.claude/settings.local.json` is gitignored). Do not "clean up" the baked exec in
  pi `mcp.ts` or hooks — that breaks the invariant and the F1 adversarial finding.
- **R1 / migration (boot.rs):** `is_doctrine_mcp_entry` keys on
  `command.file_name() == "doctrine"`. The env form `${DOCTRINE_BIN:-doctrine}`
  file-names to itself (no `/`) → reads as **foreign** unless the predicate is
  widened. A legacy abs entry (file_name `doctrine`) still matches → keep that arm
  too, so old→new migrates without double-register. **Write the migration test.**
- **R4 / reinstall refresh (INV-2/3):** baked gitignored files already
  compare-and-regenerate (boot.rs:1818). The **marketplace step is the gap** —
  `if !has_marketplace { add }` (install.rs:423) skips on a stale source (moved
  repo / changed slug). Must compare *registered* source (`marketplace list`
  output) vs intended and refresh on mismatch. Directory sources are live-loaded,
  so only the registered *path* goes stale, not content.
- **Behaviour-preservation:** existing `plan_mcp_extension` bake tests + hook tests
  must stay green **unchanged** (we don't touch those paths).

## Inquisition RV-241 (codex/GPT-5.5) — DONE, 8/8 terminal, all folded in-slice

Design-facet adversarial pass on the locked design + plan. **1 blocker + 4 majors
+ 3 minors confirmed; F3 flag-surface + committed-leak fear ACQUITTED.** Design
§5.1/§8-R1 corrected, R5/R6/R7 added, §10 external pass filled; plan criteria
appended. Verdict prose: `review-241.md` §Synthesis.

- **F-1 (blocker, SPEC-009):** `plan_mcp` no-op comparator compared existing cmd
  vs `exec.display()` (abspath) — env-form entry never hits no-op ⇒ thrash + breaks
  `plan_mcp_idempotent_when_current`. → PHASE-01 EX-5/EX-6/EX-7, VT-4. **The R1
  design named the predicate but MISSED the comparator — this is the PHASE-01 crux
  now, not just the predicate widening.**
- **F-2 (major):** abs-root assumed not enforced (`root::find` returns `--path`
  verbatim, root.rs:23) → relative path poisons R4 comparator. → canonicalize once;
  PHASE-02 EX-5, VT-4.
- **F-3 (major):** §5.1 misread the 3-plugin manifest. Selection rule pinned:
  plugin whose `name == marketplace name` (doctrine), never `[0]`. → PHASE-02 EX-6,
  VT-5 (reordered fixture).
- **F-4 (major):** substring presence greps false-match `doctrine-memory`/`-partner`
  (install.rs:527-533). → exact parsed match; PHASE-02 EX-7, VT-6.
- **F-5 (major):** PHASE-03 deferred verb × swallowed failures ⇒ remove+add failure
  reports success with doctrine uninstalled. → PHASE-03 EX-4, VT-2 (abort, no swallow).
- **F-6/F-7/F-8 (minor):** const for env literal (STD-001); outcome carries env-form
  cmd not abspath; `--dev` prompts/reminders render selected source + qualified key.

## Live probes still carried into execution (NOT blockers)

- **OQ-4 (PHASE-01 VH-1):** confirm `${DOCTRINE_BIN:-doctrine}` in `.mcp.json`
  `command` connects under `/mcp` (docs say yes, mcp.md:384). Live probe.
- **R4 verb (PHASE-03 EN-1/VH-1):** does `claude plugin marketplace add <newsrc>`
  overwrite an existing name's source, or is `remove`+`add` required (remove
  uninstalls plugins, plugin-marketplaces.md:988 → re-install after)?
  `marketplace update` only refreshes content at the same path, not a relocation.
  **Probe live before choosing the refresh mechanism** — now coupled to F-5's
  failure-handling requirement (destructive branch must abort, not swallow).
- **F3 — RESOLVED (acquitted):** bare `--dev` boolean stands; codex concurred with
  D1 (no extensibility trap; a source enum grafts later without breaking the bool).

## Cross-refs

- Backlog: fulfils **IMP-234**; sibling **IMP-249** (jail RO-bind staleness) owns
  the gitignored-surface *staleness* class — out of scope here, but the reason we
  can leave pi/hooks baked. **IMP-111** (codex MCP) separate surface.
- Governance: **POL-002** (governed_by), **SPEC-009** (references/concerns),
  **CHR-013** (the original `.mcp.json` abs-bake this fixes).
