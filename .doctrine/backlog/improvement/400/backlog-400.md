# IMP-400: Retire the Claude plugin channel

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Intent

Stop distributing Doctrine's Claude integration through Claude Code's plugin
system. Return to direct-write install for the artefacts that matter — hooks and
skills — and add a doctor leg that can tell the user *why* an install is inert
when it is.

Three legs, as raised:

1. **Toss the plugin for Claude; keep it for other agents.** (See *Open
   questions* — "keep it for other agents" needs one line of clarification; the
   `plugins/` tree is the canonical skill source for every channel, so what is
   retired is the Claude *marketplace/plugin delivery*, not the tree.)
2. **Design a workable approach for hook activation** — direct-write into
   `.claude/settings*.json`, using the existing safe-merge code.
3. **Move skills back to `npx skills` (as other harnesses already use) or
   direct-write.**

## Why

The plugin system's cost is a trust and activation model that is only partly
documented and **fails silently at every layer**. A larger install footprint is
the cheaper problem: a file that is visibly there and visibly wrong beats a
plugin that is registered, enabled, and inert with no error anywhere.

The original move to the plugin (IMP-224, closed 2026-07-03) was motivated by
"Claude now has native plugin management — we should use it instead". Its
acceptance criteria were about idempotency and config plumbing; nothing in it
weighed silent-inert failure, because that failure mode had not yet been
decoded. It has been since.

## What we already know empirically

Do not re-derive these during design — they are verified against the 2.1.198
native binary and live state.

**Three independent trust layers, each failing silently**
(`mem.concept.claude.trust-layers`):

1. **Folder trust** — `hasTrustDialogAccepted` per project in `~/.claude.json`,
   not settings.json.
2. **Plugin enablement** — `enabledPlugins` in settings.json, *plus* marketplace
   registration in `~/.claude/plugins/known_marketplaces.json`. An
   `enabledPlugins` entry whose marketplace is unregistered is skipped with
   `Skipping orphaned enabledPlugins entry` — silent, no error, plugin absent.
3. **Restricted-mode hook gate** — `CLAUDE_CODE_SAFE_MODE` / `--safe-mode`, or
   managed `allowManagedHooksOnly` / `disableAllHooks`. When engaged, **plugin**
   hooks fire only if the plugin id is in *managed policy* `enabledPlugins`
   (`/etc/claude-code/managed-settings.json` and siblings) — not user or project
   settings.

**This partly inverts the premise for the move.** Layers 2 and 3 are *plugin-
specific* silent-inert modes: the orphaned-marketplace trap has no analogue for a
settings-file hook, and restricted mode's managed-only gate targets plugin hooks
by id. Direct-written hooks are not immune (`disableAllHooks`, folder trust), but
they are exposed to strictly fewer failure surfaces, and every one of those is
inspectable from the project itself rather than from per-user absolute-path state
that cannot be committed.

**The safe-merge code the intent assumes already exists and is live.**
`src/boot.rs` carries an owner-locked merge core — `HookSpec` / `plan_hook` /
`hook_array_mut` / `install_claude_hook` — generalised over event + matcher, with
`HookSpec::boot`, `::sync`, `::stamp_subagent`, `::create_fork` constructors. It
is poison-tolerant (SL-124 `normalize`: drop every owned entry, reinsert one
canonical entry at the first owned slot), never clobbers foreign content, and
writes atomically. `src/corpus.rs` uses it for the memory-sync hook. Adding a
hook means adding a `HookSpec` constructor, never a parallel merge path.

Non-hook top-level keys do **not** thread through `plan_hook` — they ride beside
it as a separate pure planner + shell, `plan_baseref` / `install_baseref` being
the worked example (`worktree.baseRef`). Ordering matters: the second write must
read after the first.

**Skills are already half-way there.** SPEC-010 (*Skills distribution*) already
specifies two channels off one canonical `plugins/<domain>/skills/<skill>/`
tree: Claude materialises a derived `.doctrine/skills/<id>` tree and reconciles a
relative agent symlink into it (ownership-proven, never-clobber, atomic
stage-then-rename); **every other agent already delegates to `npx skills add
davidlee/doctrine`**, described there as "the universal external installer
doctrine does not reimplement". So leg 3 is less "build it" than "make the
existing direct-write channel the only Claude channel" — subject to verifying
the state below.

## State of this repo (2026-08-05, for orientation only)

- `.claude/settings.json` carries `enabledPlugins: {"doctrine@doctrine": true}`
  and nothing else. No `hooks` key in either settings file.
- All Doctrine hooks currently ship in `plugins/doctrine/hooks/hooks.json`:
  `SessionStart`, `WorktreeCreate`, `SubagentStart`/`SubagentStop`
  (`dispatch-orchestrator`), `PreToolUse` (`Bash`, `Edit|Write`).
- `.claude/skills/` and `.doctrine/skills/` do not exist here — this repo is
  served by the directory-source plugin, so the SPEC-010 symlink channel is not
  materialised. **Verify before designing** whether that channel is exercised by
  any real install or is currently dead code.

## The dispatch exposure

`WorktreeCreate` is not a nicety. `isolation:worktree` teardown is *conditional*
on that hook firing (`mem.fact.claude.worktreecreate-hook-teardown`), and `-w`
bypasses it. A silently inert plugin hook here does not degrade dispatch — it
changes its semantics without saying so. This is the strongest single argument
for moving hook activation to a surface the project can inspect and doctor.

## Doctor leg

Add a check that walks the diagnosis order the trust memory already establishes,
and reports which layer is blocking rather than "hooks not working":

1. `~/.claude.json` project entry — trusted?
2. Marketplace registered in `known_marketplaces.json` + `enabledPlugins` true?
   (Only while a plugin channel survives.)
3. Safe-mode env / `--safe-mode`; managed `allowManagedHooksOnly` /
   `disableAllHooks` — probe `/etc/claude-code/managed-settings.json`,
   `~/.config/claude-code/managed-settings.json`, the macOS
   `/Library/Application Support/ClaudeCode/` path, `/run/claude-code/`.
4. `~/.claude/plugins/blocklist.json` — github/marketplace plugins only; a
   `directory`-source plugin never appears there, so it is never the cause of a
   silent-inert local plugin.
5. Doctrine's own hook entries present, canonical, and sole in the settings file
   it owns.

Note the layering tension: layers 1–4 are **per-user, outside the project**, and
POL-002 keeps host-project conventions out of the engine. A doctor that reads
`~/.claude*` is Claude-harness-specific by nature — decide where it lives before
writing it.

## Open questions

- **`OQ-1` — what does "keep it for other agents" mean?** Reading it as: retire
  the Claude *marketplace/plugin delivery path*, keep the `plugins/` tree as the
  canonical skill source (it already feeds `npx skills` and the symlink channel),
  and keep `.claude-plugin/marketplace.json` published for anyone who prefers the
  plugin. Confirm before scoping.
- **`OQ-2` — `settings.json` or `settings.local.json`?** The intent names
  `settings.json` (project, committable — so the activation is reviewable and
  travels with the repo). Doctrine's existing merge core targets
  `settings.local.json` (per-user, uncommitted). These are different products:
  committed activation is auditable and shareable; local activation avoids
  imposing hooks on every collaborator of a client project. Possibly both, by
  scope flag.
- **`OQ-3` — is `npx skills` acceptable as the Claude path too**, or does Claude
  keep the direct symlink channel while others keep `npx`? SPEC-010 currently
  splits them deliberately.
- **`OQ-4` — migration.** Existing installs carry `enabledPlugins` entries and a
  registered marketplace. Does retire mean *remove* those (touching per-user
  state doctrine did not solely author), or leave them and stop writing new ones?
- **`OQ-5` — what is lost.** `/reload-plugins` re-registers plugin hooks with no
  restart (`mem.fact.claude.reload-plugins-registers-hooks`). Is there an
  equivalent for settings-file hooks, or does direct-write mean a restart on
  every hook change? Answer before committing — this is a live developer-loop
  cost, not a theoretical one.

## Governance impact

SPEC-010 (*Skills distribution*) describes the two-channel model and names the
Claude plugin channel explicitly; retiring it is a spec change, not just an
install change. PRD-003 (*Skills*) sits above it. RFC-018 (*Claude harness field
notes*) is the existing home for the empirical findings this item leans on.
POL-002 bounds how much host-harness knowledge may enter the engine.

## Related

- IMP-224 (closed): the move *to* the plugin this item proposes reversing.
- CHR-045: bump plugin.json version when the skill set changes — moot if retired.
- IMP-234: installer `--dev` directory-link vs github-marketplace modes.
- CHR-037: SL-195 live acceptance legs still deferred (`--dev` install, repo-move
  refresh).
- IMP-245: Cursor as a doctrine harness — a second consumer of whatever
  activation model this settles on.
- RFC-018: Claude harness field notes.
- SPEC-010 / PRD-003: the governing spec pair.
