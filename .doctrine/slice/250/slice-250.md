# Retire the Claude plugin delivery channel

## Context

Doctrine's Claude integration is currently activated through Claude Code's
plugin system: `.claude/settings.json` carries `enabledPlugins:
{"doctrine@doctrine": true}` and every hook ships in
`plugins/doctrine/hooks/hooks.json`. IMP-400 (*Retire the Claude plugin
channel*) argues that this trades a smaller install footprint for an activation
model that **fails silently at every layer**, and asks for direct-write
activation plus a doctor leg that can say *why* an install is inert.

This slice carries IMP-400's **activation** legs — direct-write hooks and the
Claude skills channel. Its **diagnostic** leg was carved out to IMP-407 (user,
2026-08-06): this slice ships activation, IMP-407 ships the diagnosis of
activation. IMP-400 therefore stays open on two counts, not one — migration
(`OQ-4`) and the doctor leg.

It is the second slice SL-247 (*Usable non-worktree subagents*) deferred to
under its Non-Goals — "touches SPEC-010 / PRD-003 and needs its own design run".

### Why the plugin channel is the wrong substrate

Three independent trust layers gate a plugin hook, each failing silently
(`mem_019f23713d8a7b53a2824b140b3005a9`, *Claude Code trust: three independent
layers*):

1. **Folder trust** — `hasTrustDialogAccepted` per project in `~/.claude.json`,
   not `settings.json`.
2. **Plugin enablement** — `enabledPlugins` in settings.json *plus* marketplace
   registration in `~/.claude/plugins/known_marketplaces.json`. An
   `enabledPlugins` entry whose marketplace is unregistered is skipped with
   `Skipping orphaned enabledPlugins entry` — no error, plugin simply absent.
3. **Restricted-mode hook gate** — `CLAUDE_CODE_SAFE_MODE` / `--safe-mode`, or
   managed `allowManagedHooksOnly` / `disableAllHooks`. When engaged, **plugin**
   hooks fire only if the plugin id appears in *managed policy* `enabledPlugins`
   — not user or project settings.

Layers 2 and 3 are **plugin-specific**. The orphaned-marketplace trap has no
analogue for a settings-file hook, and the managed-only gate targets plugin
hooks *by id*. Direct-written hooks are not immune (`disableAllHooks`, folder
trust) but are exposed to strictly fewer failure surfaces, and each surviving
one is inspectable from the project rather than from per-user absolute-path
state that cannot be committed.

### The dispatch exposure

`WorktreeCreate` is not a nicety: `isolation: worktree` teardown is *conditional*
on that hook firing (`mem_019f1a5ce1f472219da91d0724bb766b`), and `-w` bypasses
it. A silently inert hook here does not degrade dispatch — it changes dispatch's
semantics without saying so. This is the strongest single argument for moving
activation to a surface the project can inspect and doctor.

### What already exists — ride it, do not rebuild it

- **The safe-merge core is live.** `src/boot.rs` carries an owner-locked merge
  core (`HookSpec` / `plan_hook` / `hook_array_mut` / `install_claude_hook`)
  generalised over event + matcher (`mem_019ec3392f247d53a1a4c910be8306aa`),
  poison-tolerant, never clobbering foreign content, writing atomically. Adding
  a hook means adding a `HookSpec` constructor — **never** a parallel merge
  path. Non-hook top-level keys ride *beside* it as a separate pure planner +
  shell (`plan_baseref` / `install_baseref` is the worked example;
  `mem_019ec6374ef273719e9f38b99f1d96ac`), and ordering matters — the second
  write must read after the first.
- **Skills are half-way there.** SPEC-010 (*Skills distribution*) already
  specifies two channels off one canonical `plugins/<domain>/skills/<skill>/`
  tree: Claude materialises a derived `.doctrine/skills/<id>` tree with a
  relative agent symlink (proven-ownership, never-clobber, stage-then-rename),
  while every other agent delegates to `npx skills add davidlee/doctrine`. So
  the skills leg is closer to "make an existing direct-write channel the only
  Claude channel" than to new construction. Neither `.claude/skills/` nor
  `.doctrine/skills/` exists in *this* repo today — it is served by the
  directory-source plugin — but **this is how the repo used to work** (user,
  2026-08-06): the symlink channel served this repo before IMP-224 (closed
  2026-07-03) moved it onto the plugin. It is a channel being returned to, not
  speculative machinery.

### What "retire" means (settled)

IMP-400 `OQ-1` is settled by the user (2026-08-05): retire the Claude
**marketplace/plugin delivery path** only. The `plugins/` tree stays the
canonical skill source for every channel, and `.claude-plugin/marketplace.json`
stays published for anyone who prefers the plugin.

## Scope & Objectives

1. **Hook activation by direct write.** All **nine** hook entries currently in
   `plugins/doctrine/hooks/hooks.json`, across five events — `SessionStart`
   (`boot --emit`), `WorktreeCreate` (`worktree create-fork`), `SubagentStart` /
   `SubagentStop` (`dispatch-orchestrator` → `worktree nominate` /
   `denominate`), and six `PreToolUse` matchers (`Bash`, `Edit|Write`, `Agent`,
   `Workflow` → `worktree pretooluse`; `Read|Edit|Write`, `Bash` → `memory
   surface`) — activated through the existing `boot.rs` merge core into
   `.claude/settings*.json`, at **one scope, remembered as a `doctrine.toml`
   key, defaulting to project `settings.json`** (`OQ-1`, settled; `DEC-163`).

   **Design-run settlements (2026-08-06).** The nine entries collapse onto
   **six `HookSpec`s**, one of which (`create_fork`) already exists
   (`DEC-162`) — confirming `T1`. Ownership stays proven by `command` alone and
   `HookSpec` instead carries an **ordered matcher set** (`DEC-161`); the
   earlier "extend the predicate to `(command, matcher)`" reading is
   **withdrawn** — see `R5`. There is **no `--scope` flag**: the key is the only
   selector, and the installer announces its target and that key early
   (`DEC-163`). Writing one scope **evicts** this spec's owned entries from the
   other and reports it (`DEC-164`).

   The plugin's command strings are already `${DOCTRINE_BIN:-doctrine} <verb>`
   and none resolves through `${CLAUDE_PLUGIN_ROOT}` (verified 2026-08-06,
   `plugins/doctrine/hooks/hooks.json`); the `SessionStart` string is the
   **canonical current** form, not the plugin's stale copy (`DEC-162`, closing
   `T2`).

   **The command form is a scope-derived axis — added 2026-08-06 during the
   RV-348 remediation.** The earlier reading ("the strings port verbatim") was
   about the *plugin's* file and said nothing about doctrine's own writer, which
   bakes `current_exe()` — an absolute host path. `DEC-163` points the default at
   a **tracked** file, and SL-195 already closed that exact defect on `.mcp.json`
   and left the invariant **baked ⟺ gitignored** behind. So `HookSpec` holds its
   *args* rather than a rendered command and `CommandForm { Baked, Portable }`
   renders them: committed `Project` writes the portable literal (`MCP_COMMAND` →
   `PORTABLE_EXEC` — one constant, two surfaces), gitignored `Local` keeps the
   baked path, and the Codex arm answers `Baked` off its own file's gitignore
   status rather than a borrowed scope (RV-348 `F-13`). This is **in** scope: it
   is what the committed default costs.

   **The scope dial also reaches `install_baseref` and the routine sync path.**
   `install_baseref` takes the same scope — leaving it behind would have doctrine
   writing *both* settings files per install — and because eviction is spec-keyed
   while `worktree.baseRef` has no spec, the sibling is **read and a stranded
   override reported**, not swept: Local overrides Project for scalars, so an
   operator's stranded value keeps governing while doctrine's fresh one is inert
   (RV-348 `F-14`). Scope, sweep and that report all print through **one shared
   announcement writer**, which `run_sync_install` (`memory sync install`) also
   calls — the highest-frequency install path inherits the slice's one
   destructive write, so it must not inherit it in silence (RV-348 `F-2`).
2. **One Claude skills channel, sourced from the embed.** `OQ-2` settled as
   **`OQ-2b`** — rebuild `install_for_claude`: binary-sourced, derived canonical
   `.doctrine/skills/<id>` tree plus proven-ownership relative symlinks. The
   `npx` delegate was disfavoured on measured footprint (research `P5`) but
   stays as-is for non-Claude agents, so SPEC-010's dual-path `D2` survives.

   **Design-run settlement (`DEC-166`).** The byte-identical link-reconcile block
   in the agents and workflows legs is extracted to one helper, which the rebuilt
   skills channel consumes as its third caller; the multi-target capability is a
   **local loop** over link dirs, not a generic driver. The loop takes a list but
   this slice drives it with **one** entry — shipping `.agents/skills` is
   IMP-406's (`OQ-9`).
3. **Governance follow-through.** *Re-targeted by `DEC-171`.* The REV's **only**
   amendment target is **SPEC-011 / `REQ-186`**, which binds the hook write to a
   single `<exec> boot` entry in `.claude/settings.local.json` — invalidated here
   on three axes (six specs across five events, a scope-selected project default,
   and the abandoned-scope sweep). SPEC-010 governs *skills* distribution only —
   it never mentions hooks, `settings.json`, or `enabledPlugins` (research `X2`)
   — and **does not enter the REV**: `OQ-2b` restores exactly what its
   responsibilities 3–6 already describe, so its pre-existing divergence closes
   by **conformance**, verified at close, not by amendment. Whether the REV
   widens `REQ-186` or adds new requirements is deferred to reconciliation
   (`QUE-209`). RFC-018 (*Claude harness field notes*) remains the home for the
   empirical findings this leans on.

### Constraints

- **Design posture: less code over edge-case handling (user, 2026-08-06.)**
  Doctrine's user population is the author and a few people he knows personally.
  Where a gotcha can be *documented* instead of *engineered around*, document it.
  Do not build enterprise-grade migration, reconciliation, or recovery machinery
  for states that a short note and a one-line fix will cover. This governs `R8`
  and `R10` explicitly, and is the tie-breaker on `OQ-2`. It does **not** relax
  the never-clobber contract — refusing to destroy a user's own content is
  correctness, not edge-case handling.

- **POL-002** (platform independence from host-project conventions) bounds how
  much Claude-specific per-user knowledge may enter the engine. *Live again on a
  different facet (2026-08-06, RV-348 remediation.)* The per-user **reads** left
  with the doctor leg — **this slice reads no `~/.claude*` or managed-settings
  path at all**, so facet (3) is indeed dead here. But `DEC-163`'s committed
  default re-opens the facet SL-195 closed: **no absolute host path in a tracked
  file**. That is what makes the `CommandForm` axis a constraint-driven scope
  item rather than polish, and it is the one place POL-002 dictates code in this
  slice.
- **No parallel implementation.** Hooks go through `HookSpec`; non-hook keys go
  beside it in the `plan_baseref` shape. Skills go through the existing SPEC-010
  planner.
- **Never-clobber survives.** The proven-ownership contract on skill links and
  the owner-locked hook merge are both safety contracts, not conveniences.

## Non-Goals

- **Deleting the `plugins/` tree or unpublishing
  `.claude-plugin/marketplace.json`.** IMP-400 `OQ-1` settled the opposite: the
  tree stays canonical and the marketplace stays published.
- **Migrating existing installs.** Whether retire *removes* an existing
  `enabledPlugins` entry or marketplace registration is explicitly out of this
  slice (user, 2026-08-06). Retire here means **stop writing new plugin
  activation**; pre-existing per-user state is left where it is. Carried to
  Follow-Ups, and IMP-400 stays open on that leg.
- **Non-Claude harness install paths.** The `npx skills add davidlee/doctrine`
  delegate is unchanged in itself; `OQ-2` resolved to routing Claude *into* it,
  so the change is the removal of Claude's special-case, not a change to the
  delegate.
- **The doctor leg — carved out to IMP-407 (user, 2026-08-06).** The check that
  walks the trust-layer diagnosis order and names the *blocking layer*, and the
  conformance check keeping the published `hooks.json` honest against the
  `HookSpec` registry, both move to IMP-407 (sequenced `after` this slice).
  `R2`, `R7` and `OQ-5` travel with it and are no longer this slice's to settle.
  This slice ships activation; it does not ship the diagnosis of activation.
- **IMP-245 (Cursor as a doctrine harness).** A second consumer of whatever
  activation model this settles; not built here.
- **Dispatch or confinement semantics.** No change to the funnel, worker spawn,
  import belts, or the `worker_commit` gate. `WorktreeCreate` is in scope only
  as a hook to *activate differently*, not to redefine.
- **Redesigning the merge core.** *Re-drawn 2026-08-06 on research `X1`; amended
  by `DEC-161` / `DEC-164`, then widened again by the RV-348 remediation.* These
  extensions are explicitly **in** scope, because the hooks cannot be moved
  without them: new `HookSpec` constructors; `HookSpec` generalised from one
  matcher to an **ordered matcher set**; `HookSpec` holding its **args** instead
  of a rendered command, so the `CommandForm` axis can render per scope and reach
  the Codex arm; scope **resolved inside** `install_claude_hook` rather than
  passed to it (as a parameter, `run_sync_install` could omit it and re-create
  `R10` as a treadmill); a **drop-only sweep** of the scope being left, gated on
  the write landing and reported as a per-spec `EvictOutcome` folded into a
  per-file `SweepReport`; `RefreshReport.hook` becoming a collection; and the
  shared scope-announcement writer both install paths call.

  **The ownership predicate is NOT widened.** The earlier draft put "widening
  the ownership predicate from `command` to `(command, matcher)`" in scope;
  `DEC-161` rejects that and keeps ownership command-only, because widening
  orphans an entry whose matcher later moves into a permanent silent
  double-fire, where command-only ownership heals it.

  What stays out is replacing the plan/normalize/never-clobber architecture. The
  behaviour-preservation gate still binds the **Codex arm** — its emitted
  `.codex/hooks.json` stays byte-identical, which is what `CommandForm::Baked`
  buys it — and `DEC-161`'s shape satisfies it by construction, since the
  existing specs are the N=1 case. **`corpus.rs` is no longer on that gate**: it
  is a changed path (RV-348 `F-2`), not a regression surface.
- **Windows.** `${DOCTRINE_BIN:-doctrine}` is POSIX parameter expansion, and
  hooks are shell form — `sh -c` on macOS/Linux, Git Bash on Windows, else
  PowerShell, where `${…}` delimits a *name* and the token resolves as an unset
  variable so the command degrades to empty. Silently, since `PreToolUse` hooks
  fail open. Doctrine does not target Windows (user, 2026-08-06): this is a
  **stated boundary**, not a portability problem to engineer around (RV-348
  `F-16`).
- **SL-247's `OQ-2`/`OQ-3`** — whether a worktree-local `.claude/` binds for an
  in-session `isolation: worktree` subagent. SL-247 routed those to this slice's
  settings-scope question (`OQ-1`); with that now settled as *both scopes behind
  a sticky `[install]` key* (`DEC-163` — there is no flag), they are answered
  only to the extent that key's semantics require.

## Affected surface

- `src/boot.rs` — `HookSpec`'s ordered matcher set and its args-not-command shape
  (`DEC-161`, RV-348 `F-13`), six specs' constructors and one shared ownership
  predicate (`DEC-162`), `plan_hook` / `desired_entries` / `install_claude_hook`,
  scope resolution, the `CommandForm` axis threaded to both arms, the drop-only
  sweep of the abandoned scope with `EvictOutcome` / `SweepReport` (`DEC-164`,
  RV-348 `F-15`), `RefreshReport.hook` as a collection, and `install_baseref`
  following the scope and reporting a stranded sibling override (RV-348 `F-14`).
  New named constants for `.claude/settings.json`, the matcher tokens, and
  `PORTABLE_EXEC` (`MCP_COMMAND` renamed — STD-001).
- `src/install_config.rs` — the `ClaudeSettingsScope` enum and its sticky
  `[install] claude-settings-scope` key, read through the existing
  `dtoml::load_doctrine_toml` seam (`DEC-163`). *Corrected 2026-08-06: the key
  lives in the `[install]` table, not beside `[dispatch]` in `src/dtoml.rs`;
  `dtoml.rs` is touched only for the module doc that mirrors it.*
- `src/install.rs` — the rebuilt skills channel (`OQ-2b`), the extracted
  link-reconcile helper replacing the byte-identical blocks at `:2179-2191` and
  `:2279-2291` (`DEC-166`), the local target loop, the removal of the automated
  plugin steps and their orphans, and the shared scope-announcement writer on the
  existing `writeln!(stdout, …)` seam.
- `src/corpus.rs` — **changed, not merely a regression surface** (RV-348 `F-2`).
  `run_sync_install` matches the new `HookWrite` return and calls the shared
  announcement writer, so `memory sync install` reports the scope and the sweep
  it performs.
- `plugins/doctrine/hooks/hooks.json` — the hooks being relocated. **Not deleted**
  — it stays as the published plugin's payload (`R9`'s escape hatch), which is
  why `R6` dissolves and `src/doctor_checks.rs` leaves this slice's surface.
- `.claude-plugin/marketplace.json` — stays published; verify it still resolves
  after the skills channel settles.
- `.doctrine/spec/tech/011/` — **the REV's only amendment target**: `REQ-186`
  (`DEC-171`). Requirement granularity — one widened requirement or several new
  ones — is deferred to reconciliation (`QUE-209`).
- `.doctrine/spec/tech/010/` — **read, not amended.** `OQ-2b` restores what its
  responsibilities 3–6 already describe, so it is a conformance-verification
  target rather than a REV target (`DEC-171`).
- `install/` — user-facing guidance on activation and the cutover gotchas, plus
  the new `[install] claude-settings-scope` key (mirrored into
  `install/doctrine.toml.example` and `install/doctrine.toml`), the command-form
  note and its POSIX-shell boundary, and `R9`'s escape-hatch instructions.

## Risks / Assumptions / Open questions

- **`R1` — CLOSED, and it inverts (research, 2026-08-06).** Claude Code watches
  settings files and hot-reloads them, `hooks` explicitly included; only `model`
  and `outputStyle` are restart-gated. There is no `/reload-plugins` equivalent
  because none is needed. Direct-write is a **strict developer-loop improvement**
  over the plugin path, which needs an explicit `/reload-plugins` whose
  reliability the memory corpus already doubts
  (`mem_019f1b770e75712086168408276a4868`, flagged CONTRADICTED). Retained as an
  argument *for* the change, not a risk.
- **`R5` — the merge core cannot represent the hooks being moved. The slice's
  central design problem.** Ownership is proven by `command` alone
  (`src/boot.rs:1039-1056`) and the normalize collapses every owned entry to one
  canonical entry — by design, documented in the predicate's own comment. Four
  `PreToolUse` entries share the command `worktree pretooluse` (matchers `Bash`,
  `Edit|Write`, `Agent`, `Workflow`) and two share `memory surface`
  (`Read|Edit|Write`, `Bash`). One predicate per command would mark all four
  owned and silently drop three. This drove the Non-Goal re-draw.

  **SETTLED (design run `dr-019fd692`, 2026-08-06): `DEC-161` — the matcher set,
  not the widened predicate.** The fork was *widen ownership to
  `(command, matcher)`* versus *generalise `HookSpec` to an ordered matcher set
  while ownership stays command-only*. The second wins on what happens when a
  matcher set later changes: widening orphans the stale entry into a permanent
  silent double-fire, while command-only ownership still recognises and refreshes
  it. That is not hypothetical — `T2`'s live drift between the published
  `hooks.json` and the code is exactly that shape, and
  `is_doctrine_emit_command` already recognises both arg forms so it self-heals.
  Widening would also *narrow* the predicate for four already-shipping specs,
  putting the behaviour-preservation gate at risk; the matcher set is a strict
  generalisation with N=1 as the existing case.
- **`R6` — WITHDRAWN (design run `dr-019fd692`, 2026-08-06): its premise is
  false.** It read "retiring the plugin blinds `SpawnSeamSymmetry`", which parses
  `plugins/doctrine/hooks/hooks.json` (`src/doctor_checks.rs:622`) precisely
  because that is the authored shipped source. But the Non-Goals keep the
  `plugins/` tree and the published manifest, and `R9`'s mitigation actively
  *requires* the plugin to keep working as the managed-policy escape hatch —
  which requires its hooks. **The file survives, the check keeps its input, and
  nothing reds.** What retirement actually leaves behind is not a blinded check
  but un-policed drift between the manifest and the `HookSpec` registry — already
  live (the manifest ships the legacy `boot --emit` on matcher `*` while the code
  emits `prompt resolve --role orchestrator` on `startup|clear`). That is
  IMP-407's Leg 2, not this slice's.
- **`R7` — MOVED to IMP-407 (user, 2026-08-06).** The doctor can verify
  plausibility, not activation: `/hooks` is the only surface reporting which
  hooks are live and which file each came from, and it is interactive-only. The
  constraint binds IMP-407's acceptance criteria; with the doctor leg carved out,
  this slice's closure criteria no longer make any claim it could over-reach.
- **`R10` — cross-scope double-fire; the probe's one adverse finding.** Because
  scopes *merge* (`OQ-6`) and doctrine's merge core normalizes ownership only
  **within the single file it writes** (`install_hook_to_file(root, rel_path,
  …)`), an owned entry left in `settings.local.json` and the same hook newly
  written to `settings.json` both fire. This is not hypothetical: the live
  `memory sync install` path writes `SETTINGS_REL` = `.claude/settings.local.json`
  (`src/boot.rs:531`, `src/corpus.rs:506`), so every existing install already has
  an owned entry in local scope. Flipping the default to project scope without
  sweeping the other file double-fires the sync hook.

  ~~**Disposition (user, 2026-08-06): document, do not engineer.**~~
  ~~No file-spanning ownership, no automatic eviction, no migration pass.~~
  **SUPERSEDED (design run `dr-019fd692`, 2026-08-06) — see `DEC-164`.** The
  choice narrowed to a binary once the doctor leg left for IMP-407 (the middle
  rung, a read-only finding from a leg already walking both files, went with it),
  and design took the engineered side.

  **`R10` is ENGINEERED.** Writing one scope sweeps the sibling file for this
  spec's owned entries, drop-only, and reports the eviction in the installer's
  target line (`DEC-163`). The decisive fact: scopes *merge*, so the same
  doctrine hook in both files is never a configuration anyone chose — it is
  always the defect, and the merge core's one-canonical-entry invariant simply
  extends to the pair of files doctrine now writes. Documentation was rejected
  because the failure is silent (a hook firing twice reads as mild slowness), the
  leg that would have reported it left with IMP-407, and the default flip
  guarantees the defect for **every existing install, this repo included**. The
  sweep is reuse-only — `drop_owned_hooks` / `owned_positions` already exist and
  it is `plan_hook` minus the insert — and is gated by the same ownership
  predicate that protects foreign entries, so never-clobber is untouched.
- **`R8` — mid-migration double-fire.** The settings boot hook was originally
  removed because it double-fired against the plugin. Any state with the plugin
  still enabled *and* settings hooks written reproduces that. **Disposition
  (user, 2026-08-06): document, do not engineer.** The cutover note tells the
  handful of existing users to disable the plugin when they take the new
  activation; nothing detects or reconciles it for them.

  **This stays documentation, and cannot be otherwise (`DEC-167`).** Unlike
  `R10`, the plugin's entries load through `enabledPlugins` plus per-user
  marketplace registration — state this slice reads and writes nothing of by
  Non-Goal — so the ownership sweep cannot reach them. The asymmetry is the
  point: `R10` is engineered because both settings files are doctrine's to
  write; `R8` is documented because the plugin's activation state is not.

  **Prescribed order (`DEC-167`): install first, then disable the plugin.**
  Between the two acts everything double-fires, which is wasteful and harmless.
  The reverse order leaves the repo with *no* activation, and an inert
  `WorktreeCreate` hook does not degrade dispatch — `isolation: worktree`
  teardown is conditional on it firing, so absence changes dispatch's semantics
  without saying so. Order toward the degraded state, never the absent one.
- **`R2` — MOVED to IMP-407 (user, 2026-08-06).** The POL-002 boundary on
  reading per-user `~/.claude*` state was only ever a property of the doctor leg,
  which is no longer in this slice. Resolved in principle (research, 2026-08-06):
  POL-002 forbids depending on host state *silently*, not depending on it, and a
  feature-scoped capability declaration is the fit. **This slice now reads no
  per-user harness state at all** — it writes project-scoped settings files and
  nothing else — so the precedent it would have set is deferred with it.
- **`R9` — the silent-failure premise is narrower than claimed.** Direct-write
  sheds two plugin-only modes (orphaned marketplace registration, blocklist) but
  keeps `disableAllHooks`, `allowManagedHooksOnly`, and folder trust — and
  *acquires* one: `strictPluginOnlyCustomization` (managed settings only) blocks
  hooks from user and project sources so they may come only from plugins or
  managed settings. Under that gate the plugin channel is the safer one. Fewer
  failure modes, not a subset; the design must say so rather than overclaim.
  **Mitigation (user, 2026-08-06): this is a reason to keep the plugin manifest
  around and document it as the sanctioned workaround** for managed-policy-locked
  environments. `.claude-plugin/marketplace.json` already stays published under
  IMP-400 `OQ-1`; the slice's user-facing docs must name it as the escape hatch
  rather than leaving it as vestigial.
- **`R3` — bootstrap ordering.** *Downgraded (user, 2026-08-06): "I can manually
  fiddle it."* Retiring the channel that currently activates Doctrine in this
  repo can leave the working session (and dispatch workers) without hooks partway
  through. The human will hand-repair their own activation across the cutover, so
  this does not buy a both-channels-tolerated compatibility window. Design should
  still say plainly *when* activation flips, so the moment is expected rather than
  discovered.
- **`R4` — migration touches state Doctrine did not solely author.**
  *Downgraded (user, 2026-08-06): already the case — the current plugin form
  depends on exactly that per-user state.* Retire adds no new exposure, and with
  migration out of scope this slice does not write those files at all.
- **`A1`** — the empirical findings recorded on IMP-400 are verified against the
  2.1.198 native binary and are not re-derived during design.
- **`A2`** — *settled (user, 2026-08-06), then sharpened by research.* The
  symlink channel is how this repo worked before IMP-224 moved it onto the
  plugin — but its orchestration was **deleted**, not left dormant. Only the
  proven-ownership helpers survive. See `OQ-2`.
- **`A3` — CLOSED, and it bites (design run, 2026-08-06).** No gitignore change
  is needed: `.gitignore` already carries `!/.claude/settings.json` beside
  `/.claude/*`, so the project settings file is **tracked here today**. That is
  the fact that costs, not the one that saves — because the file is committed,
  the baked `current_exe()` command form becomes a POL-002 breach and had to go.
  `SETTINGS_REL`'s own rationale for choosing the gitignored file ("the absolute
  exec path belongs out of git") does not evaporate; it is **honoured** by
  emitting the portable `${DOCTRINE_BIN:-doctrine}` form in the committed scope
  (`OQ-7`, and see the `CommandForm` axis under Scope).
- **`OQ-1` — SETTLED (user, 2026-08-06): both scopes reachable, defaulting to
  project `settings.json`.** Committed activation is reviewable and travels with
  the repo; `settings.local.json` stays available for a collaborator who must not
  impose hooks on a client project's whole team. Doctrine's existing merge core
  targets `settings.local.json`, so the core gains a scope argument rather than a
  second write path. (IMP-400 `OQ-2`.)

  **Refined by `DEC-163` (design run, 2026-08-06): a sticky key, NOT a flag.**
  The selector is a `doctrine.toml` key read through the existing
  `load_doctrine_toml` seam (`[dispatch]` is the precedent table); there is **no
  `--scope` flag**. A per-invocation flag would let any routine flagless install
  revert a local-scope choice and re-create the entry in the other file, turning
  `R10` from a one-time cutover into a treadmill. In its place the installer
  announces early **where it will write and which key changes it** —
  discoverability was the flag's only remaining job. All six specs ride the one
  dial; there is no per-entry scope routing (`DEC-162`).
- **`OQ-2` — SETTLED (user, 2026-08-06): `OQ-2b`, rebuild `install_for_claude`.**
  Binary-sourced, canonical tree plus proven-ownership relative symlinks — the
  channel SPEC-010 already specifies. Recovered from `git show 347197e8 --
  src/skills.rs`. **Design it parameterised over the target directory** (see
  `OQ-9`), not hard-coded to `.claude/skills/`.

  The reasoning that got here is retained below.

  **The npx delegate was disfavoured on measured footprint.** The earlier provisional settlement optimised the wrong axis: it
  bought *less doctrine code* at the price of a heavier runtime dependency and a
  worse dev loop. Measured (research `P5`): the delegate lands **35 real skill
  directories (~320K, copies not symlinks)** in `.claude/skills/` plus a
  root-level `skills-lock.json`, and requires **a full GitHub clone on every
  install** — Node, `npx`, network and GitHub all become hard runtime
  requirements. It **discards the embed**, against SPEC-010's own premise that
  the binary carries every skill "with no network fetch and no sidecar bundle";
  and for doctrine dogfooding itself it would install *published `HEAD`* skills
  while ignoring the live local `plugins/` tree under edit. (It also surfaced
  that doctrine would pass the wrong agent token: `--agent claude` is rejected,
  the valid identifier is `claude-code`.)

  The live options are now the two that keep the binary self-contained:

  - **`OQ-2a` — plain copy from the embed** into `.claude/skills/<id>/`. Produces
    the same on-disk shape npx does, sourced from the binary instead of GitHub.
    No canonical tree, no symlink reconciliation, no gitignore self-enforcement.
    The least code of the three. Open question: how it meets never-clobber
    without reintroducing hash tracking.
  - **`OQ-2b` — rebuild `install_for_claude`** (derived `.doctrine/skills/<id>`
    canonical tree + relative symlinks, proven-ownership trichotomy). More code,
    but it is *specified* in SPEC-010 already, and every helper it needs is live
    and exercised today by the agents/workflows install paths (`classify_link`,
    `write_link`, `relative_target`, `install_base`). Gives the keep-foreign
    override hatch and single-source dedup for free.

  **A governance asymmetry favours `OQ-2b`.** SPEC-010 *still specifies* the
  canonical-tree-plus-symlink channel as current behaviour — its responsibilities
  read "Claude materialises a derived canonical `.doctrine/skills/<id>` tree and
  reconciles a relative agent symlink into it" — while `install_for_claude` has
  been deleted since `347197e8`. **The spec and the code are already divergent,
  independently of this slice.** `OQ-2b` closes that divergence and needs no
  SPEC-010 amendment for skills at all; `OQ-2a` would need one. Counting
  governance work, the "more code" option may be the cheaper total.

  The deleted orchestration is recoverable verbatim:
  `git show 347197e8 -- src/skills.rs`.

  (IMP-400 `OQ-3`.)
- **`OQ-9` — how far does the direct-write channel generalise?** `npx skills
  --agent universal` lands at **`.agents/skills/<id>/`** (research `P6`), the
  ecosystem's harness-neutral target; doctrine already maintains its own
  `.agents/skills/` tree here. So the same binary-sourced mechanism could serve
  most non-Claude harnesses from the embed, with `npx` kept only as the fallback
  for harnesses needing bespoke layouts (user, 2026-08-06).

  **In scope for SL-250: only that `OQ-2b`'s mechanism is parameterised over the
  target directory** — so `.agents/skills/` is a second *target*, not a second
  mechanism. Actually shipping the neutral target, and deciding which harnesses
  stop delegating, is follow-up work: it changes SPEC-010's `D2` for non-Claude
  agents, which this slice holds as a Non-Goal. Captured as IMP-406.

  **The deleted code is already most of the way there** (verified against
  `git show 347197e8^:src/skills.rs`, 2026-08-06). One canonical tree, N link
  sets — the content is stored and refreshed **once**, and each agent directory
  holds only relative symlinks into it:

  - `canonical_dir(root, global)` → `<base>/.doctrine/skills` — **agent-neutral
    already** (`:305`);
  - `claude_dir(root, global)` → `<base>/.claude/skills` (`:290`) — a one-line
    function, the *only* Claude-specific element;
  - `claude_links(skills, agent_dir, canon_dir)` (`:439`) — **already takes
    `agent_dir` as a parameter** and its body is agent-agnostic
    (`dest = agent_dir.join(id)`, `target = relative_target(agent_dir, canon_dir,
    id)`). It is misnamed, not Claude-bound.

  So parameterising means renaming `claude_links` → `agent_links`, hoisting
  `agent_dir` out of `install_for_claude` into a parameter, and looping the link
  phase over targets while the materialise phase still runs once. `install_base`
  keeps `--global` coherent (canonical and links both move to `$HOME`).
- **`OQ-3` — OUT OF SCOPE (user, 2026-08-06).** Whether retire *removes* existing
  `enabledPlugins` / marketplace registrations. Not settled, not carried: see
  Non-Goals and Follow-Ups. (IMP-400 `OQ-4`.)
- **`OQ-4` — SETTLED by research (2026-08-06): yes, and better.** Settings files
  are watched and hot-reloaded, `hooks` explicitly included. See `R1`.
  (IMP-400 `OQ-5`.)
- **`OQ-5` — MOVED to IMP-407 (user, 2026-08-06), carrying its settlement.** It
  settled by research as a feature-scoped POL-002 capability declaration: the
  doctor leg declares its dependency on Claude's per-user files, no default path
  acquires it, and absence yields a descriptive finding naming what was missing.
  That settlement stands and travels with the leg. See `R2`.
- **`OQ-6` — SETTLED BY PROBE (2026-08-06): hooks MERGE across settings scopes.**
  A project-scope hook and a local-scope hook on the same matcher both fired on
  one tool call. Writing project scope by default cannot clobber a user's own
  `settings.local.json` hooks. See `R10` for the consequence.
- **`OQ-7` — SETTLED BY PROBE (2026-08-06): yes.** `${PROBE_BIN:-sh}` expanded
  and ran inside a `settings.json` hook command, so every
  `${DOCTRINE_BIN:-doctrine}` string ports verbatim.
- **`OQ-8` — SETTLED BY PROBE (2026-08-06): the target shape is valid.** Two
  entries with a byte-identical command string on different matchers both fired,
  each receiving its own tool. `R5` is therefore purely doctrine's own
  limitation, not the harness's — the multi-entry file the fix must produce is
  one Claude Code honours. *(The probe framed this as validating a widened
  `(command, matcher)` ownership; `DEC-161` took the ordered-matcher-set route
  instead. The probe's finding is unaffected — it validates the on-disk shape,
  which both candidates emit identically, not the ownership rule behind it.)*

## Verification / closure intent

Done is judged by:

- **`R8` is written down; `R10` is demonstrated.** *Split 2026-08-06 by
  `DEC-164` / `DEC-167` — they are no longer one criterion.*
  - `R8` (plugin + settings double-fire during cutover) stays a **documentation**
    criterion: the note names the remedy *and* the prescribed order — install
    first, then disable the plugin. Nothing detects or reconciles it, and nothing
    can (`DEC-167`).
  - `R10` (owned entry left in the other settings scope) is now a **behavioural**
    criterion: switching scope evicts this spec's owned entries from the file
    being left and reports the eviction. Verified by test, not by prose.
- **The scope target is announced — on both install paths.** The installer states
  early where it will write and which `doctrine.toml` key changes it (`DEC-163`)
  — this is what replaces the `--scope` flag, so it is a criterion rather than a
  nicety. The same writer serves `memory sync install`, so the routine path
  reports its scope and any eviction too (RV-348 `F-2`).
- **No absolute host path in a tracked file.** SL-195's invariant survives the
  committed default: the `Project` scope emits `${DOCTRINE_BIN:-doctrine}` and
  the baked path appears only in the gitignored `Local` scope. Verified by test
  on both arms — including that `.codex/hooks.json` output stays byte-identical
  under `CommandForm::Baked`.
- **A stranded `worktree.baseRef` override is reported.** Seeding a non-`head`
  value in the abandoned scope and installing to the other yields a report naming
  the file, the value and the remedy — the key is never swept (no-clobber), so
  the report is the whole of the signal (RV-348 `F-14`).
- **A cold install activates.** In a scratch project with no plugin
  registration, `doctrine install` (or the settled verb) leaves hooks that
  actually fire — demonstrated live, not merely planned. This is the claim the
  whole slice rests on and should carry a `VH` leg; `/hooks` is the surface that
  shows both that they are live and which file they came from.
- **`SpawnSeamSymmetry` is untouched and still green.** Its input
  (`plugins/doctrine/hooks/hooks.json`) survives this slice, so the criterion is
  that the check needs **no** change and its live-config regression test passes
  unmodified (`R6`, withdrawn).
- **Safety contracts intact.** Foreign hook entries and pinned skill
  directories/links survive every path.
- **The behaviour-preservation gate is two classes, not one.** *Sharpened
  2026-08-06 — "the existing suites stay green unchanged" was true of the
  matcher-set design and false of the scope and `CommandForm` work.* Class (i)
  is preserved by construction and any red is a real narrowing: the Codex arm,
  `corpus.rs`'s memory-sync **hook semantics**, the `SpawnSeamSymmetry`
  regression test, and `plan_mcp` (whose only delta is the `MCP_COMMAND` →
  `PORTABLE_EXEC` rename). Class (ii) is knowingly rewritten — the settings-path
  and report-shape assertions in `boot.rs` — and the design enumerates every site
  so an expected red is distinguishable from a real one. Closure requires that
  enumeration to still match what actually changed.
- **Governance reconciled.** *Re-targeted 2026-08-06 (`DEC-171`).* The REV amends
  **SPEC-011 / `REQ-186`** alone — it binds the hook write to one `<exec> boot`
  entry in `settings.local.json`, and this slice invalidates it on three axes
  (six specs across five events, a scope-selected project default, and the
  abandoned-scope sweep). **SPEC-010 does not enter the REV**: `OQ-2b` restores
  exactly what its responsibilities 3–6 already describe, so the pre-existing
  divergence closes by conformance, not amendment. Its criterion is therefore a
  **verification**, not an edit — SPEC-010 responsibilities 3–6 confirmed true of
  the restored code. RFC-018 updated with anything new the slice learns.
- **Backlog dispositioned.** IMP-400 reduced to its migration leg and left open
  (not closed — migration is out of scope); IMP-407 (*doctor leg*) confirmed
  still open and still `after` this slice; CHR-045 (*bump plugin.json version
  when the skill set changes*) resolved or explicitly retained — note it is
  **no longer moot**, since the plugin manifest survives as `R9`'s escape hatch;
  IMP-234 and CHR-037 assessed for overlap.

## Summary

## Follow-Ups

- **Migration of existing installs** — whether to remove pre-existing
  `enabledPlugins` entries and marketplace registrations, and under what
  ownership discipline. Deferred out of this slice; IMP-400 stays open carrying
  it (IMP-400 `OQ-4`).
- **IMP-407 — the doctor leg** (user, 2026-08-06). Two checks: the trust-layer
  activation walk that names the blocking layer, and a conformance check keeping
  the published `hooks.json` honest against the `HookSpec` registry. Sequenced
  `after` SL-250. `R2`, `R7` and `OQ-5` travel with it.
- **IMP-406 — serve non-Claude harnesses from the embed** via `.agents/skills`.
  This slice ships only the *parameterisation* (`OQ-9`); shipping the neutral
  target and deciding which harnesses stop delegating is IMP-406's.
