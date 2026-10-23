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

## Origin

Surfaced by SL-285 PHASE-03 review (EX-2 clean-porcelain fork test). SL-285
works around (1) in its test fixture (fixture `.gitignore` ignores `/.codex`).
(2) reported by the user 2026-10-23.
