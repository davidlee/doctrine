# Force-reinstall skills (overwrite existing)

## Context

`doctrine skills install` is idempotent-by-skip: `claude_steps` (`src/skills.rs:188`)
emits `Step::Skip` whenever the destination skill dir already exists, so an
already-installed skill is never refreshed. The only way to pick up edited skill
sources today is to `rm -rf .claude/skills/<id>` by hand, then reinstall.

This bites the inner loop: editing a `SKILL.md` under `plugins/` and re-running
`install` is a silent no-op. The new `doctrine` process-skill stubs make this
routine — they will be edited and reinstalled repeatedly.

## Scope & Objectives

- Add a `--force` flag to `skills install` (`SkillsCommand::Install`, `src/main.rs`).
- Thread it through `run_install` → `build_plan` → `claude_steps`: when set, an
  existing destination installs (overwrites) instead of skipping.
- Overwrite is **clean** — the dest skill dir is replaced, not merged, so files
  deleted from source don't linger (stale-file correctness).
- Plan/report surface: a forced overwrite of an existing dir reports distinctly
  from a fresh install and from a skip (so `--dry-run` tells the truth).
- Delegate (non-Claude) path: pass the equivalent overwrite intent to
  `npx skills add` in `delegate_argv` **iff** that tool exposes one; otherwise
  document the gap. (Open question — settle in design.)
- Behaviour-preservation: without `--force`, the skip behaviour is unchanged; the
  existing skills suites stay green.

## Non-Goals

- No global "always overwrite" config/default — force is opt-in per invocation.
- No partial/selective file merge, diffing, or backup of the replaced dir.
- No change to discovery, selection, agent resolution, or the `npx` delegation
  shape beyond the single overwrite signal.
- No new uninstall verb.

## Summary

A one-flag change to the install planner: `--force` turns the `dest.exists()`
skip into a clean overwrite, kept pure in the plan layer (a new/extended `Step`
variant) so it's testable without disk, with the imperative replace behind the
existing execution seam.

## Follow-Ups

- If `npx skills add` lacks a force/overwrite flag, file the gap (and possibly a
  Claude-only `--force` until the delegate catches up).
- Consider whether `install` should warn when a skip *would have* refreshed a
  changed source (drift hint) — separate slice if wanted.
