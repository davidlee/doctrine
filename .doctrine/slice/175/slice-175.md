# Admit spawned_from relation label + retcon 19 mislabelled backlog→slice edges

## Context

`drn doctor` lifecycle check surfaces 19 backlog items with "all linked slices
terminal" — every slice they reference via `label = "slices"` is `done`/`closed`.
The `slices` label means *addressed by* (the slice fulfills the item), but in all
19 cases causality is reversed: the item was **created from** the slice (post-close
review finding, deferred scope, split-out work, surfaced tech-debt).

SPEC-019 D6 already pins `spawns` for knowledge records → spawned work, but no
equivalent label exists for backlog ↔ slice edges. The doctor lifecycle check is
correct to flag these — a backlog item "addressed by" terminal slices should itself
be terminal — but the edges are semantically wrong.

This slice admits the label and retcons the edges. It does **not** build a `spawns`
CLI verb or generalise the spawn-work pattern across kinds.

## Scope & Objectives

**What changes**

1. Admit `spawned_from` as a valid `[[relation]]` label on backlog items (target:
   `SL`). Register it in `RELATION_RULES` so `doctrine link` accepts it.
2. Retcon the 19 mislabelled edges from `label = "slices"` → `label = "spawned_from"`:
   ISS-003, ISS-019, IMP-019, IMP-025, IMP-045, IMP-053, IMP-065, IMP-068,
   IMP-095, IMP-099, IMP-103, IMP-105, IMP-112, IMP-138, IMP-143, IMP-162,
   IMP-163, IDE-009, IDE-021.
3. The doctor lifecycle check resolves correctly — `spawned_from` implies origin,
   not implementation, so a terminal origin slice is expected.

**Affected surface**

- `src/relation.rs` or wherever `RELATION_RULES` lives — admit the label.
- `.doctrine/backlog/*/NNN/backlog-NNN.toml` — retcon 19 `[[relation]]` rows
  (`s/"slices"/"spawned_from"`).
- Doctor lifecycle check — verify the warnings clear.

## Non-Goals

- No `spawns` CLI verb (doctrine link already covers authoring).
- No generalisation of spawn-work across kinds (IMP-053 covers record↔record).
- No change to the doctor check itself (the check is correct; the data was wrong).

## Summary

One label admission + 19 TOML retcons. The doctor lifecycle warnings clear because
edges that meant "created-from" no longer masquerade as "addressed-by."

## Follow-Ups

- IMP-053 (record↔record associative relation) may later want a general spawn-work
  pattern, but this slice is scoped to the backlog→slice label only.
