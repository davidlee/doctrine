# dispatch: preferred worker harness for arm selection (subagent vs subprocess)

## Context

SL-108 / IMP-101 landed the `[dispatch] preferred-subprocess-harness` config,
which selects between subprocess harnesses (codex vs pi) once the dispatch
router has already chosen the subprocess arm. There is no config mechanism to
tell the dispatch router WHICH arm to use — subagent (Claude, via
dispatch-agent) or subprocess (codex/pi, via dispatch-subprocess).

Currently arm selection is inferred from the orchestrator's own harness: a Claude
orchestrator defaults to the subagent arm (it can use the `Agent` tool); a
codex/pi orchestrator must use the subprocess arm. A project may want Claude to
dispatch pi subprocess workers instead — e.g. for structured `agent_end`
outcomes, process isolation, or reproducibility. The config should express this
preference explicitly rather than relying on env-marker inference alone.

The existing `[dispatch]` section in `doctrine.toml` is the natural home for
this key.

## Scope & Objectives

1. Design and implement a `preferred-worker-harness` (or similarly named) config
   key under `[dispatch]` in `doctrine.toml` that selects between `claude`
   (subagent arm), `pi` (subprocess arm), and `codex` (subprocess arm).

2. Update the dispatch router skill (`dispatch/SKILL.md`) to consult this config
   when choosing the arm, with env-marker detection as the fallback.

3. The `dispatch_config` module (IMP-101) already owns the `[dispatch]` table —
   extend it with the new enum variant(s) and key.

## Non-Goals

- Changing the subprocess spawn template (codex/pi) — IMP-101 already covers that.
- Adding a CLI `--harness` override flag on `worktree fork` (separate concern).
- Generalising the Harness enum in `boot.rs` — dispatch_config's harness concept
  is narrower and independent.

## Summary

One new config key, one enum extension, one skill update. The existing
infrastructure (dtoml parser, dispatch_config module, dispatch router skill)
provides the seams.

## Follow-Ups

- IMP-104: General pi subagent spawn pattern (may interact with arm selection
  for subagent-classed pi roles like scout/fix-agent).
