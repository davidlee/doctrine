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
  Claude channel" than to new construction — **subject to verifying that channel
  is exercised**: neither `.claude/skills/` nor `.doctrine/skills/` exists in
  this repo, which is served by the directory-source plugin.

### What "retire" means (settled)

IMP-400 `OQ-1` is settled by the user (2026-08-05): retire the Claude
**marketplace/plugin delivery path** only. The `plugins/` tree stays the
canonical skill source for every channel, and `.claude-plugin/marketplace.json`
stays published for anyone who prefers the plugin.

## Scope & Objectives

1. **Hook activation by direct write.** Every hook currently in
   `plugins/doctrine/hooks/hooks.json` — `SessionStart`, `WorktreeCreate`,
   `SubagentStart` / `SubagentStop` (`dispatch-orchestrator`), `PreToolUse`
   (`Bash`, `Edit|Write`) — activated through the existing `boot.rs` merge core
   into `.claude/settings*.json`, with the scope question (`OQ-2`) settled at
   design.
2. **One Claude skills channel.** Settle `OQ-3` (npx delegate vs the SPEC-010
   symlink channel for Claude), verify the surviving channel is live rather than
   dead code, and make it the only Claude path.
3. **The doctor leg.** A check that walks the diagnosis order the trust memory
   establishes and names the *blocking layer* rather than reporting "hooks not
   working": folder trust → (plugin registration, while any plugin channel
   survives) → safe-mode / managed policy → blocklist → Doctrine's own hook
   entries present, canonical, and sole in the file it owns.
4. **Migration.** Settle and implement `OQ-4` — whether retire *removes* the
   `enabledPlugins` entry and marketplace registration (per-user state Doctrine
   did not solely author) or merely stops writing new ones.
5. **Governance follow-through.** SPEC-010 names the Claude plugin channel
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
- **Bootstrap hazard.** This repo activates Doctrine *through the very channel
  being retired*. Changing it alters the working session's own hook surface
  mid-slice; sequencing must account for that.

## Non-Goals

- **Deleting the `plugins/` tree or unpublishing
  `.claude-plugin/marketplace.json`.** `OQ-1` settled the opposite: the tree
  stays canonical and the marketplace stays published.
- **Non-Claude harness install paths.** The `npx skills add davidlee/doctrine`
  delegate for other agents is unchanged — unless `OQ-3` resolves to making npx
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
  `OQ-2`; they are *inputs* to settling the settings-file scope question, and are
  answered only to the extent that question requires.

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
- **`R3` — bootstrap ordering.** Retiring the channel that currently activates
  Doctrine in this repo can leave the working session (and dispatch workers)
  without hooks partway through. Needs an explicit cutover order, and possibly a
  period where both channels are tolerated.
- **`R4` — migration touches state Doctrine did not solely author.** Removing an
  `enabledPlugins` entry or a marketplace registration edits per-user files that
  may carry other consumers' entries. The owner-locked discipline that governs
  hook entries has no established analogue here.
- **`A1`** — the empirical findings recorded on IMP-400 are verified against the
  2.1.198 native binary and are not re-derived during design.
- **`A2`** — the SPEC-010 symlink channel is *specified*; whether any real
  install exercises it is unverified and is a research-round question, not an
  assumption to build on.
- **`OQ-1`** — `settings.json` (project, committable, reviewable, travels with
  the repo) vs `settings.local.json` (per-user, uncommitted, imposes nothing on
  collaborators)? Possibly both by scope flag. Doctrine's existing merge core
  targets `settings.local.json`; IMP-400's intent names `settings.json`. These
  are different products, not a naming detail. (IMP-400 `OQ-2`.)
- **`OQ-2`** — is `npx skills` acceptable as the Claude path too, or does Claude
  keep the direct symlink channel while others keep `npx`? SPEC-010 currently
  splits them deliberately. (IMP-400 `OQ-3`.)
- **`OQ-3`** — does retire *remove* existing `enabledPlugins` / marketplace
  registrations, or only stop writing new ones? (IMP-400 `OQ-4`.)
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
- **Backlog dispositioned.** IMP-400 closed; CHR-045 (*bump plugin.json version
  when the skill set changes*) resolved or explicitly retained; IMP-234 and
  CHR-037 assessed for overlap.

## Summary

## Follow-Ups
