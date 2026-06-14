# Dispatch projection parents on a stale base → non-integrable deliverables

> Slug `wire-dispatch-stage-2-sync-integrate-into-close` is stale (original
> mis-scope); slug is non-authoritative. Title/scope below are current.

> **Repro done 2026-06-15 — see `notes.md` for the verified verdict.** It
> corrected this framing: the projection-base defect (live-tip parenting) is
> **already fixed by RV-030** (post-dated SL-067's dispatch); the live gap is
> *no re-anchor recovery when trunk advances past the fork-point* + the
> integration UX burying ~8 real deletions under ~2309 by-design `.doctrine`
> strips. The "stale base" language below is superseded by notes.md F1–F4.

## Context

The `/dispatch` SL-067 run produced `review/067` + `phase/067-*` deliverables
that **cannot be landed**:

```
diff main..phase/067-02   →  2317 deletions, 0 additions
src/revision.rs:  present on main,  ABSENT on phase/067-02
```

The whole post-fork main (the REV feature, the entire `.doctrine` corpus) reads
as deleted. Cause: the projected phase branches are parented on `91b05c4`
(18 behind main), **not** on the coordination base `26a3125` where the funnel
actually built the units.

Key distinction — the funnel was sound, the **projection** was not:

```
coordination base (dispatch/067):  26a3125   (PHASE-01 code_start; boundaries.toml clean)
funnel phase chain:                26a3125 → 777c76e → 6e89e12   (coherent)
projected phase/067-01 parent:     91b05c4   ← stale, ≠ coordination base
```

`dispatch sync --integrate --trunk main` is **fail-closed**: `is_ancestor`
(`plan_trunk_row`, dispatch.rs:359) trips because the stale-based phase tip does
not fast-forward main → *"trunk moved; re-anchor required, not auto-resolved."*
So trunk was never silently wiped. But there is **no recovery path**: the verb
refuses, or you force-merge the 2317 deletions by hand (the SL-067 salvage
nightmare). The run's deliverables are dead either way.

The **original SL-068 premise was wrong twice**: wiring `--integrate` into
`/close` would not have landed SL-067 (it refuses), and the missing wiring was
not the bug. The bug is upstream in projection-base resolution.

## Scope & Objectives

1. **Reproduce + pin the entry point (experimental, first).** The `26a3125` vs
   `91b05c4` discrepancy must be reproduced and the exact source isolated:
   `prepare_review` `trunk_base = merge_base(tip, trunk_tip)` resolution,
   `plan_phases` parent chaining, or claude-arm `record-boundary` base capture.
   Forensics point at projection-base, **not** the worker fork — but this is a
   hypothesis requiring experimental verification, not git archaeology alone.
2. **Make deliverables integrable when trunk advances mid-run.** Projection base
   must track the live mergeable base (3-way re-anchor), or deliverables rebase
   before projection, or a guarded recovery verb — approach decided in design.
3. **Preserve fail-closed.** Never silently wipe trunk; report + recover, never
   auto-resolve (ADR-006).

## Non-Goals

- Wiring `--integrate` into `/close` (the original mis-scope) — secondary, and
  only meaningful once projection is sound. Captured as a follow-up.
- Codex/pi worker-fork-base mechanics, unless the repro implicates them.
- Overturning ADR-012's pinned-fork-base decision without `/consult` — design
  surfaces the tradeoff (pinned base vs live re-anchor) explicitly.

## Summary

_(pending design)_

## Follow-Ups

- `/close` stage-2 `sync --integrate` wiring gap is real but secondary — restore
  as its own slice/backlog item once projection soundness lands.
