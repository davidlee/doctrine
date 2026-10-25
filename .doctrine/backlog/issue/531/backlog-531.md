# ISS-531: Managed gitignore block omits .codex and slice research

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

`doctrine install` rewrites a managed `.gitignore` block from
`install/manifest.toml` `[gitignore] entries` (plus framework roots and
per-link lines; `src/install.rs` `gitignore_block`). In a newly installed
project two derived paths are left un-ignored:

1. **`.codex/`** — install writes `.codex/hooks.json` with a baked absolute
   exec path (`boot.rs` `CommandForm::Baked`), so it is machine-specific and
   must never be committed. A fresh client repo shows `?? .codex/` after
   install, and every provisioned worktree fork does too.
2. **`.doctrine/slice/*/research/`** — `/research` output (`raw/`,
   `baseline.toml`) is unignored.

This repo masks both with its own lines outside the managed block
(`.gitignore:10` `/.codex`, `.gitignore:42` `.doctrine/slice/*/research/`),
so dogfooding never sees it.

## Open points

- `.codex/` wholesale, or only `.codex/hooks.json`? Check what else codex keeps
  there (config.toml, agents per IMP-515) and whether any of it is portable.
- Research: whole dir, or only `raw/` + `baseline.toml`? Decide whether
  `research.md` is authored (committed) evidence.

## Resolution of open points (2026-10-26)

- **`.codex/hooks.json` only, not `.codex/`.** Install writes hooks with
  `CommandForm::Baked` (absolute exec path; `src/boot.rs` `CODE_HOOKS_REL`), so
  that file is machine-specific. `.codex/config.toml` is the project's own codex
  config: install only strips doctrine's old MCP wrapper from it
  (`src/install.rs` `codex_leg_outcome`) and leaves it, so it stays tracked.
  Future `.codex/agents` links (IMP-515) are ignored by the managed block's
  per-link lines, the same way as other harness links.
- **Research: the whole slice `research/` dir.** The research skill
  (`.agents/skills/research/SKILL.md`) and `install/glossary.md` both call the
  whole folder runtime-tier and gitignored, `research.md` included; durable
  findings are harvested at close (see IMP-314).
- **Slice-scoped, not `.doctrine/**/research/`.** RFC research is committed
  (`.doctrine/rfc/033/research/` is tracked), and only `doctrine slice
  research` mints the runtime kind. The worktree classifier already pins
  `.doctrine/slice/*/research/**` (`src/worktree/allowlist.rs`, `Tier::Research`).
- `install/glossary.md` said `research/*` in a slice, spec, or backlog dir
  is gitignored; reworded to scope that to the slice dir.

## Fix

Both entries added to `install/manifest.toml` `[gitignore] entries`; this
repo's out-of-block `.doctrine/slice/*/research/` line removed so dogfooding
exercises the managed block. Test:
`install::tests::managed_block_ignores_derived_paths_and_spares_authored_ones`
(real `git check-ignore` over the rendered block).

## Origin

Surfaced by SL-285 PHASE-03 review (EX-2 clean-porcelain fork test). SL-285
works around (1) in its test fixture (fixture `.gitignore` ignores `/.codex`).
(2) reported by the user 2026-10-23, and again 2026-10-26 (research skill says
`research/` is gitignored; the managed block had no entry).
