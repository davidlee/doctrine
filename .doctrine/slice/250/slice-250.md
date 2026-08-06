# Retire the Claude plugin delivery channel

## Context

Doctrine's Claude integration is currently activated through Claude Code's
plugin system: `.claude/settings.json` carries `enabledPlugins:
{"doctrine@doctrine": true}` and every hook ships in
`plugins/doctrine/hooks/hooks.json`. IMP-400 (*Retire the Claude plugin
channel*) argues that this trades a smaller install footprint for an activation
model that **fails silently at every layer**, and asks for direct-write
activation plus a doctor leg that can say *why* an install is inert.

This slice carries IMP-400 in full. It is the second slice SL-247 (*Usable
non-worktree subagents*) deferred to under its Non-Goals — "touches SPEC-010 /
PRD-003 and needs its own design run".

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
   `.claude/settings*.json`, at **both scopes behind a flag, defaulting to
   project `settings.json`** (`OQ-1`, settled).

   Every command is already `${DOCTRINE_BIN:-doctrine} <verb>`; none resolves
   through `${CLAUDE_PLUGIN_ROOT}`, so the command strings port into a settings
   file verbatim (verified 2026-08-06, `plugins/doctrine/hooks/hooks.json`).
   The merge core's ownership predicate must first be extended from `command`
   to `(command, matcher)` — see `R5`. Without that it cannot represent these
   hooks at all.
2. **One Claude skills channel.** Settle `OQ-2` (npx delegate vs the SPEC-010
   symlink channel for Claude) and make the survivor the only Claude path.
3. **The doctor leg.** A check that walks the diagnosis order the trust memory
   establishes and names the *blocking layer* rather than reporting "hooks not
   working": folder trust → (plugin registration, while any plugin channel
   survives) → safe-mode / managed policy → blocklist → Doctrine's own hook
   entries present, canonical, and sole in the file it owns.
4. **Governance follow-through.** The REV's primary target is **SPEC-011**
   (`REQ-186` and Responsibility 6), which binds the hook-write target to
   `.claude/settings.local.json`. SPEC-010 governs *skills* distribution only —
   it never mentions hooks, `settings.json`, or `enabledPlugins` — so it enters
   the REV only if `OQ-2` changes the skills channel (research `X2`). RFC-018
   (*Claude harness field notes*) remains the home for the empirical findings
   this leans on.

### Constraints

- **POL-002** (platform independence from host-project conventions) bounds how
  much Claude-specific per-user knowledge may enter the engine. Layers 1–4 of
  the doctor walk read `~/.claude*` and system managed-settings paths — outside
  the project entirely. Where that check lives is a design decision, not an
  implementation detail.
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
  delegate for other agents is unchanged — unless `OQ-2` resolves to making npx
  the Claude path too, in which case the change is on Claude's side only.
- **IMP-245 (Cursor as a doctrine harness).** A second consumer of whatever
  activation model this settles; not built here.
- **Dispatch or confinement semantics.** No change to the funnel, worker spawn,
  import belts, or the `worker_commit` gate. `WorktreeCreate` is in scope only
  as a hook to *activate differently*, not to redefine.
- **Redesigning the merge core.** *Re-drawn 2026-08-06 on research `X1`.* Three
  extensions are now explicitly **in** scope, because the hooks cannot be moved
  without them: new `HookSpec` constructors, a settings-scope argument, and
  widening the ownership predicate from `command` to `(command, matcher)`. What
  stays out is replacing the plan/normalize/never-clobber architecture. The
  behaviour-preservation gate applies — `corpus.rs`'s memory-sync hook and the
  Codex arm must stay green unchanged.
- **SL-247's `OQ-2`/`OQ-3`** — whether a worktree-local `.claude/` binds for an
  in-session `isolation: worktree` subagent. SL-247 routed those to this slice's
  settings-scope question (`OQ-1`); with that now settled as *both scopes behind
  a flag*, they are answered only to the extent the flag's semantics require.

## Affected surface

- `src/boot.rs` — `HookSpec` constructors, `plan_hook` / `install_claude_hook`;
  possibly a settings-file scope argument.
- `src/install.rs`, `src/install_config.rs` — the skills channel planner and the
  `claude install` surface.
- `src/doctor_checks.rs`, `src/commands/doctor.rs` — the new activation check.
- `src/corpus.rs` — existing `HookSpec` consumer (memory-sync hook); regression
  surface for any core change.
- `plugins/doctrine/hooks/hooks.json` — the hooks being relocated.
- `.claude-plugin/marketplace.json` — stays published; verify it still resolves
  after the skills channel settles.
- `.doctrine/spec/tech/010/` — SPEC-010 amendment (via REV).
- `install/` — user-facing guidance on activation and the doctor walk.

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
  owned and silently drop three. Ownership must widen to `(command, matcher)`,
  or a matcher-set must be planned as one unit. This drove the Non-Goal re-draw.
- **`R6` — the doctor's existing check reads the file being retired.**
  `SpawnSeamSymmetry` parses `plugins/doctrine/hooks/hooks.json`
  (`src/doctor_checks.rs:622`) *precisely because* it is the authored shipped
  source rather than a tamperable installed copy. Retiring the plugin blinds the
  check and reds its live-config regression test. Migrating its input is slice
  work, not fallout.
- **`R7` — the doctor can verify plausibility, not activation.** `/hooks` is the
  only surface that reports which hooks are live and which file each came from,
  and it is interactive-only; there is no programmatic query. Doctrine's doctor
  can confirm the settings file it wrote is present, canonical, and sole — it
  cannot confirm the harness loaded it. The closure criteria must not promise
  otherwise.
- **`R10` — cross-scope double-fire; the probe's one adverse finding.** Because
  scopes *merge* (`OQ-6`) and doctrine's merge core normalizes ownership only
  **within the single file it writes** (`install_hook_to_file(root, rel_path,
  …)`), an owned entry left in `settings.local.json` and the same hook newly
  written to `settings.json` both fire. This is not hypothetical: the live
  `memory sync install` path writes `SETTINGS_REL` = `.claude/settings.local.json`
  (`src/boot.rs:531`, `src/corpus.rs:506`), so every existing install already has
  an owned entry in local scope. Flipping the default to project scope without
  sweeping the other file double-fires the sync hook. Ownership must span both
  scope files, or the scope switch must evict the owned entry from the scope it
  leaves. **This lands directly on the `OQ-1` design and must be answered there.**
- **`R8` — mid-migration double-fire.** The settings boot hook was originally
  removed because it double-fired against the plugin. Any state with the plugin
  still enabled *and* settings hooks written reproduces that. Install ordering is
  load-bearing even though migrating pre-existing installs is out of scope.
- **`R2` — POL-002 boundary. Resolved in principle (research, 2026-08-06).**
  POL-002 forbids depending on host state *silently*, not depending on it: a
  feature-scoped capability is opt-in, acquired by no default path, and must fail
  with a message naming what was missing. The doctor verb is exactly that shape.
  Residual risk is only that no engine code has done this before — SL-250's leg
  would be the first to read `~/.claude*`, so it sets the precedent.
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
- **`A3` — `OQ-1`'s benefit needs a gitignore change to land in this repo.**
  `.gitignore:4` ignores `/.claude` wholly, so project `settings.json` is
  untracked *here*. **The user will adjust `.gitignore` as needed (2026-08-06)**,
  so this is a small in-scope edit, not a constraint. Note also that
  `SETTINGS_REL`'s own rationale for choosing the gitignored file — "the absolute
  exec path belongs out of git" — evaporates once commands are the portable
  `${DOCTRINE_BIN:-doctrine}` form, which the probe confirmed works (`OQ-7`).
- **`OQ-1` — SETTLED (user, 2026-08-06): both, by scope flag, defaulting to
  project `settings.json`.** Committed activation is reviewable and travels with
  the repo; `settings.local.json` stays available for a collaborator who must not
  impose hooks on a client project's whole team. Doctrine's existing merge core
  targets `settings.local.json`, so the core gains a scope argument rather than a
  second write path. (IMP-400 `OQ-2`.)
- **`OQ-2` — the only substantive question left open.** Is `npx skills`
  acceptable as the Claude path too, or does Claude keep the direct symlink
  channel while others keep `npx`? Research supplies the cost line the card
  lacked: the symlink orchestration (`install_for_claude`) was **deleted**, not
  merely dormant — removed at `347197e8`, with `skills.rs` deleted at `68d2107a`
  — so "keep the symlink channel" means resurrecting it from history. The
  proven-ownership helpers (`classify_link`, `write_link`, `relative_target`)
  survive and are exercised by the agents/workflows paths. Symlinks at
  `.claude/skills/<id>` are explicitly supported by the harness. The real trade
  is *rebuild machinery doctrine owns* against *one delegate path for every
  agent, depending on an external tool absent from the official docs and on Node*.
  Governance does not settle it: SPEC-010's own rationale supports both readings.
  (IMP-400 `OQ-3`.)
- **`OQ-3` — OUT OF SCOPE (user, 2026-08-06).** Whether retire *removes* existing
  `enabledPlugins` / marketplace registrations. Not settled, not carried: see
  Non-Goals and Follow-Ups. (IMP-400 `OQ-4`.)
- **`OQ-4` — SETTLED by research (2026-08-06): yes, and better.** Settings files
  are watched and hot-reloaded, `hooks` explicitly included. See `R1`.
  (IMP-400 `OQ-5`.)
- **`OQ-5` — SETTLED by research (2026-08-06): a feature-scoped POL-002
  capability declaration.** The doctor leg declares its dependency on Claude's
  per-user files, no default path acquires it, and absence yields a descriptive
  finding naming what was missing. See `R2`.
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
  limitation, not the harness's: widening ownership to `(command, matcher)`
  yields a file Claude Code honours.

## Verification / closure intent

Done is judged by:

- **No cross-scope double-fire.** `R10`: after a scope switch, exactly one owned
  entry per hook exists across `settings.json` *and* `settings.local.json`
  together — the merge core's sole-and-canonical guarantee must hold across the
  pair, not per file.
- **A cold install activates.** In a scratch project with no plugin
  registration, `doctrine install` (or the settled verb) leaves hooks that
  actually fire — demonstrated live, not merely planned. This is the claim the
  whole slice rests on and should carry a `VH` leg; `/hooks` is the surface that
  shows both that they are live and which file they came from.
- **The doctor names the layer.** Each blocking layer in the diagnosis order
  produces a distinct, accurate diagnosis against fixtures — the per-user layers
  are fixture-driven since they cannot be mutated in test. Scoped honestly per
  `R7`: the doctor reports that what doctrine wrote is present, canonical and
  sole, not that the harness loaded it.
- **`SpawnSeamSymmetry` still guards the spawn seams.** Its input migrates with
  the hooks and its live-config regression test passes against the new
  authoritative source (`R6`).
- **Safety contracts intact.** Foreign hook entries and pinned skill
  directories/links survive every path; the existing `boot.rs` and `install.rs`
  suites stay green unchanged (behaviour-preservation gate).
- **Governance reconciled.** SPEC-010 amended through a REV to describe the
  surviving channel set; RFC-018 updated with anything new the slice learns.
- **Backlog dispositioned.** IMP-400 reduced to its migration leg and left open
  (not closed — migration is out of scope); CHR-045 (*bump plugin.json version
  when the skill set changes*) resolved or explicitly retained; IMP-234 and
  CHR-037 assessed for overlap.

## Summary

## Follow-Ups

- **Migration of existing installs** — whether to remove pre-existing
  `enabledPlugins` entries and marketplace registrations, and under what
  ownership discipline. Deferred out of this slice; IMP-400 stays open carrying
  it (IMP-400 `OQ-4`).
