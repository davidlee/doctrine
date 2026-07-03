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

## PHASE-01 — DONE (green, committed `b33bea5b`)

MCP command portability. `desired_mcp_entry` emits `MCP_COMMAND` const
(`${DOCTRINE_BIN:-doctrine}`); `is_doctrine_mcp_entry` owns env + legacy abs;
`plan_mcp` no-op comparator compares the const not `exec.display()` (F-1 blocker
fixed); `RefreshOutcome` carries the env command (F-7); dead `exec` param dropped
through 4 fns (DRY). `generate_mcp_extension` + pi `mcp.ts` + hooks untouched
(baked ⟺ gitignored, D2/F1). New tests: `plan_mcp_idempotent_when_current`
(env-form ⇒ None), `plan_mcp_migrates_legacy_abs_to_env_form`. `doctrine check
gate` PASS. EX-1..7 / VT-1..4 satisfied.

- **VH-1 (OQ-4) — VT-green; live env-expansion leg deferred to user reconnect.**
  `serve --mcp` launches; this session proves doctrine MCP connects under `/mcp`
  (old abspath file). The `${VAR:-default}` expansion is a CC contract
  (mcp.md:384), confirmable only by a `/mcp` reconnect against a regenerated
  env-form `.mcp.json`. No PHASE-02/03 code depends on it.
- **Footgun harvested:** `mem.pattern.doctrine.idempotency-comparator-tracks-emitted-value`
  — a refresh planner's no-op check must compare against what it now WRITES; a
  constant emit with an input-derived comparator thrashes. RV-241 F-1 root cause.

## PHASE-02 — DONE (green, gate PASS)

`--dev` flag + marketplace source selection + 4 inquisition fixes. src/install.rs
+ src/commands/cli.rs only (file-disjoint from PHASE-01 boot.rs).

- **`--dev` bool** (cli.rs Install → InstallArgs.dev, dry_run pattern). EX-1.
- **`select_marketplace_source(root, cwd, repo, dev)` → `MarketplaceSource`**
  (`Github(slug)` | `Directory(abs)`). dev=true canonicalizes the root ONCE
  (relative joined onto injected cwd, then `fs::canonicalize` — F-2/EX-5) and
  requires `.claude-plugin/marketplace.json` (hard error else — EX-3). cwd is a
  param ⇒ VT-4 relative-path test is deterministic (no process-CWD mutation).
- **Selection rule (F-3/EX-6):** `select_plugin` = manifest entry whose
  `name == manifest.name` (`doctrine`), never `[0]`. VT-5 reordered fixture.
- **Exact presence (F-4/EX-7):** `claude_list_has` = whitespace-token equality
  over `claude plugin[/marketplace] list` stdout — kills the `contains("doctrine")`
  substring that false-matched `doctrine-memory`/`-partner`. Replaced the 3 old
  substring helpers. VT-6.
- **Qualified install (EX-4):** `enable_key()` = `doctrine@doctrine` from the
  single const `DOCTRINE_MARKETPLACE` (STD-001). Both modes install the qualified
  key at `--scope project`.
- **Prompts/reminders (F-8/EX-8):** render the selected source + qualified key;
  skip flags now carry the display string (`Option<String>`) so the manual-install
  reminder matches what the run would have done.
- **VH-1:** mechanical legs confirmed (parses; manifest present; live marketplace
  already `Directory (/workspace/doctrine)`; no abspath in diff). Full interactive
  `install --dev` forward-run deferred to user confirm — no code dep.
- **CARRY → PHASE-03 (R-P2-1):** live-probe that `marketplace add <canonical-abs>`
  stores a path byte-equal to `fs::canonicalize` output before the R4 comparator
  trusts equality. select_marketplace_source already emits canonical abs.
- New tests (8, all green): `enable_key_is_qualified_doctrine`,
  `select_plugin_picks_by_name_not_first`, `plugin_presence_is_exact_not_substring`,
  `marketplace_presence_is_exact_token`, `source_default_is_github_slug`,
  `source_dev_is_directory_abs`, `source_dev_missing_manifest_errors`,
  `source_dev_relative_root_canonicalizes_absolute`.

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

## PHASE-03 — DONE (green, gate exit 0)

Reinstall marketplace source-refresh (R4). Closes the stale-source gap: the old
`if !claude_marketplace_has { add }` skipped on name-present, retaining a stale
source after a repo move / changed slug.

### R4 live probe — DECIDED (EN-1) · CC 2.1.198

`claude plugin marketplace add <src>`:
- src == registered → idempotent no-op (`✔ already on disk`, exit 0)
- src != registered (name collides) → **OVERWRITE** (`✔ Successfully added`,
  source flips, exit 0)

⇒ refresh verb = a single `add`. NO remove+add, no destructive window.
`marketplace update` only re-pulls content at the same path (not a relocation).
Recorded: `mem.fact.claude.marketplace-add-overwrites-source`.

**D-P3-1:** implement single-add refresh only; the destructive remove+add branch
is NOT written (probe proves it never runs on this CC; dead code fails the repo's
`-D dead-code` gate; YAGNI). EX-4's destructive-abort clause is vacuously met.
VT-2 (F-5) is honoured on the add/Refresh path: a failed *refresh* `return Err`s
(aborts forward steps) — never swallowed into `skipped_*` to report success with
a stale source live. A failed *fresh add* keeps the softer `skipped_*` reminder.

### Shapes (all src/install.rs)

- `enum RegisteredSource { Directory(String), Github(String) }` + pure
  `parse_registered_source(list, name)` — reads `❯ <name>` then its
  `Source: <Kind> (<inner>)` line; unrecognised/absent ⇒ None (⇒ safe Add).
- `enum MarketplaceAction { Skip, Add, Refresh }` + `marketplace_action` +
  `source_matches` (kind-tagged equality reusing `MarketplaceSource::as_arg()`).
- `refresh_failure_is_fatal(action)` = `matches!(Refresh)`.
- Claude arm rewired to the three-way action; `claude_marketplace_has` retired
  (dead after rewire).

### Verification

- VT-1 → parser + `marketplace_action` over `marketplace list` fixtures
  (stale⇒Refresh, match⇒Skip, absent⇒Add, sibling⇒None, kind-mismatch⇒Refresh).
- VT-2 → `refresh_failure_is_fatal` (pure; no shell-out flakiness).
- VH-1 (live repo-move reinstall) DEFERRED to a user confirm (mirrors PHASE-01/02
  VH-1); no code depends on it.

4 new unit tests; gate exit 0 (clippy 0-warn incl pedantic, test, fmt, build).
Diff confined to src/install.rs — file-disjoint from PHASE-01 (boot.rs), rides
PHASE-02 seams. **R-P2-1 CLOSED**: restore leg proved `list` echoes the canonical
abspath byte-equal to `fs::canonicalize` — the `as_arg()` comparator is sound.

Test names: parse_registered_source_reads_directory_and_github,
parse_registered_source_absent_or_sibling_is_none, marketplace_action_add_skip_refresh,
refresh_failure_is_fatal_only_on_refresh.
