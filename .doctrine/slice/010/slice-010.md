# Symlink skills from a canonical .doctrine/skills tree (Claude-first)

## Context

`doctrine skills install` copies each embedded skill into `.claude/skills/<id>`
and *skips* when the dir already exists (`claude_steps`, `src/skills.rs:188`).
Copies drift: editing a `SKILL.md` under `plugins/` then re-running `install` is
a silent no-op, and the only refresh is a manual `rm -rf`.

The first cut at this (the slice's original `--force` scope) was a flag to opt
into overwriting the copies. A **symlink model removes the whole staleness class
instead, and needs no flag** — so it supersedes that idea rather than extending
it. Design forks settled in conversation:

- The `npx skills` delegate (vercel-labs/skills) has no force flag; it overwrites
  by default and symlinks into Claude Code as its recommended method — so agents
  honour symlinked skill dirs. (Confirmed.)
- Reach: **Claude-first** — doctrine owns the local canonical tree + the Claude
  links only; other agents keep delegating to `npx`. Chosen on complexity budget:
  no agent→dir registry, no Node-ownership, smallest diff.

## Scope & Objectives

- **Canonical tree.** `install` materialises `.doctrine/skills/<id>/` from the
  rust-embed. It is **derived** (regenerable), rewritten on every install (always
  overwrite — it owns no authored data). The `dirs.create` list already makes the
  dir; this slice adds `.doctrine/skills/*` to the manifest's `[gitignore].entries`
  so a downstream `doctrine install` ignores it too (today it's masked only by this
  repo's blanket `.doctrine/*` — the manifest writes additive entries, not the
  blanket, so without this a consumer would commit the derived tree). Mirrors the
  memory pattern: `items/` created-and-tracked, the derived subtrees ignored.
- **Claude path becomes symlinks.** `.claude/skills/<id>` → a *relative* symlink
  to `.doctrine/skills/<id>`, replacing the copy mechanism for that path.
- **Type-keyed, flag-free policy** per agent-dir target:
  - missing → create the symlink;
  - existing **symlink** → relink (overwrite) unconditionally;
  - existing **real dir/file** → refuse + warn (never clobber — not doctrine's).
- **Honest reporting.** Plan / `--dry-run` distinguishes *linked* (new),
  *relinked* (refreshed), and *kept* (a foreign real dir left untouched).
- **Delegation unchanged.** Non-Claude agents still resolve to `npx skills add …`
  (`delegate_argv` untouched); they pull from GitHub, not the local canonical
  tree. Claude-first means that split is accepted.
- **`--force` is dropped** — the premise (opt-in copy overwrite) no longer exists.

## Decision: `.doctrine/skills` is derived (gitignored)

Source of truth is the embed (`plugins/` in this repo, the binary downstream);
`.doctrine/skills/` is its regeneration, so it is **derived → gitignored**,
materialised by `install` (node-modules-style: a fresh clone runs `doctrine
install`). Not committed, not configurable — committing a copy invites drift
against the embed, and a commit/ignore config knob spends complexity budget for a
case the override hatch below already covers.

## Overrides (the escape hatch)

The type-keyed policy *is* the override mechanism — no flag needed:

- A managed skill is a **symlink** → `install` owns it, relinks freely.
- To pin / hand-edit a skill, make it a **real copy** in the agent dir; `install`
  sees a foreign real dir and **refuses + warns** (`kept .claude/skills/<id>
  (not a symlink)`). The copy is sacrosanct.

Ready answer to *"pin these skills, keep them in git, stop install clobbering
them"*:

```sh
rm .claude/skills/<id>                       # drop the managed symlink
cp -r .doctrine/skills/<id> .claude/skills/<id>   # your own real copy (then edit)
git add -f .claude/skills/<id>               # force-track (.claude is gitignored)
```

Reversible: `rm -rf .claude/skills/<id> && doctrine install` restores the managed
symlink. Two distinct concerns, do not conflate: the **real-copy** stops the
clobber (type policy); `git add -f` only versions it (`.claude` is blanket-ignored).
The `kept …` warning is a required affordance — it tells the user every install
that their override is respected.

## Non-Goals

- doctrine does **not** own the `.agents` matrix or other agents' dirs — no
  agent→dir registry; `npx` keeps the long tail.
- No `--copy` fallback for the Claude path (symlink only; nixos target).
- No Windows symlink handling.
- No migration verb: a pre-existing real `.claude/skills/<id>` (e.g. an old
  copy-install) is treated as foreign — kept + warned, not auto-converted.
- No change to discovery, selection, agent resolution, or the npx delegate argv.

## Summary

Replace the Claude-path copy+skip with a canonical `.doctrine/skills/` tree plus
relative symlinks, governed by a type-keyed policy: overwrite our own symlinks and
the derived tree, never touch foreign real dirs. Kills skill staleness, needs no
flag, and leaves `npx` delegation for other agents as-is. Pure plan layer (new
`Step` variants) so the trichotomy is disk-free testable; the imperative
link/relink/refuse lives behind the existing execution seam.

## Follow-Ups

- **Migration nicety** (separate slice): detect a foreign real `.claude/skills/<id>`
  that matches a doctrine skill and offer an opt-in convert-to-symlink.
- **Latent bug**: `DELEGATE_SOURCE = "doctrine/doctrine"` vs the real
  `davidlee/doctrine` — fix.
- **Split source**: Claude uses the local canonical tree, npx-managed agents pull
  from GitHub. Revisit only if frequent multi-agent local installs appear.
