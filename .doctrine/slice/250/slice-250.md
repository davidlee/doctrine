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
2. **One Claude skills channel.** Settle `OQ-2` (npx delegate vs the SPEC-010
   symlink channel for Claude) and make the survivor the only Claude path.
3. **The doctor leg.** A check that walks the diagnosis order the trust memory
   establishes and names the *blocking layer* rather than reporting "hooks not
   working": folder trust → (plugin registration, while any plugin channel
   survives) → safe-mode / managed policy → blocklist → Doctrine's own hook
   entries present, canonical, and sole in the file it owns.
4. **Governance follow-through.** SPEC-010 names the Claude plugin channel
   explicitly, so this is a spec change, not just an install change: amend it
   through a REV. RFC-018 (*Claude harness field notes*) remains the home for
   the empirical findings this leans on.

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
- **Reworking the merge core.** `HookSpec` / `plan_hook` are ridden as-is;
  extending them with a constructor or a scope argument is in scope, redesigning
  them is not.
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

- **`R1` — developer-loop cost of direct-write hooks.** `/reload-plugins`
  re-registers plugin hooks with no restart
  (`mem_019f1b770e75712086168408276a4868`). Whether a settings-file hook has an
  equivalent is unknown; if not, every hook change costs a restart. This is
  IMP-400 `OQ-5` and must be answered before committing, not after.
- **`R2` — POL-002 boundary.** A doctor that reads `~/.claude.json`,
  `known_marketplaces.json`, and system managed-settings is harness-specific by
  nature. Placing it wrongly imports host-harness knowledge into the engine.
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
- **`A2`** — *settled (user, 2026-08-06).* The SPEC-010 symlink channel is not
  speculative: it is how this repo worked before IMP-224 moved it onto the
  plugin. Research confirms the mechanism's current shape, not its viability.
- **`OQ-1` — SETTLED (user, 2026-08-06): both, by scope flag, defaulting to
  project `settings.json`.** Committed activation is reviewable and travels with
  the repo; `settings.local.json` stays available for a collaborator who must not
  impose hooks on a client project's whole team. Doctrine's existing merge core
  targets `settings.local.json`, so the core gains a scope argument rather than a
  second write path. (IMP-400 `OQ-2`.)
- **`OQ-2`** — is `npx skills` acceptable as the Claude path too, or does Claude
  keep the direct symlink channel while others keep `npx`? SPEC-010 currently
  splits them deliberately. (IMP-400 `OQ-3`.)
- **`OQ-3` — OUT OF SCOPE (user, 2026-08-06).** Whether retire *removes* existing
  `enabledPlugins` / marketplace registrations. Not settled, not carried: see
  Non-Goals and Follow-Ups. (IMP-400 `OQ-4`.)
- **`OQ-4`** — is there a reload equivalent for settings-file hooks, or does
  direct-write mean a restart on every hook change? (IMP-400 `OQ-5`; the
  evidence side of `R1`.)
- **`OQ-5`** — where does the doctor check live, given POL-002? (The evidence
  side of `R2`.)

## Verification / closure intent

Done is judged by:

- **A cold install activates.** In a scratch project with no plugin
  registration, `doctrine install` (or the settled verb) leaves hooks that
  actually fire — demonstrated live, not merely planned. This is the claim the
  whole slice rests on and should carry a `VH` leg.
- **The doctor names the layer.** Each blocking layer in the diagnosis order
  produces a distinct, accurate diagnosis against fixtures — the per-user layers
  are fixture-driven since they cannot be mutated in test.
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
