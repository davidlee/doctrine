# IMP-207: Add spawned_from relation label for backlog→slice edges and retcon 19 mislabelled slices edges

## Context

`drn doctor` lifecycle check surfaces 19 backlog items with "all linked slices
terminal" — every slice they reference via `label = "slices"` is `done`/`closed`.
But the `slices` label means *addressed by* (the slice fulfills the item), and in
all 19 cases the causality is reversed: the item was **created from** the slice
(post-close review finding, deferred scope, split-out work, surfaced tech-debt).

SPEC-019 D6 already pins `spawns` for knowledge records → spawned work, but no
equivalent label exists for backlog ↔ slice edges.

> **Folded into RFC-003 (2026-06-29) — resolved.** This is `drift` redux, and the fix
> is part of **finishing Axis B**: `slices` retires into the universal grammar.
> Provenance becomes **`references(originates_from)`** (one neutral role) — `spawned_from`
> is **not** minted; the 19 rows relabel to `originates_from`. Fulfillment + completion
> becomes a new **`fulfils`** label carrying a `{full, partial}` degree facet. See
> RFC-003 § "Finish Axis B". The direction is decided; it lands via a ratifying ADR +
> the finish-Axis-B slice. **The 19-edge retcon (step 2) may proceed independently**,
> targeting `originates_from`.

## Problem

- `slices` implies "this item is implemented by SL-NNN" — semantically wrong for
  items spawned *from* a slice.
- The doctor lifecycle check correctly flags these because a backlog item
  "addressed by" terminal slices should itself be terminal. But these items aren't
  addressed-by the linked slice; they were *created because of* it.
- No `spawned_from` label exists in the relation model for backlog items.

## What changes

1. Admit `spawned_from` as a valid relation label on backlog items (target: slice).
2. Retcon the 19 mislabelled edges from `slices` → `spawned_from`:
   ISS-003, ISS-019, IMP-019, IMP-025, IMP-045, IMP-053, IMP-065, IMP-068,
   IMP-095, IMP-099, IMP-103, IMP-105, IMP-112, IMP-138, IMP-143, IMP-162,
   IMP-163, IDE-009, IDE-021.
3. The doctor warning resolves correctly — spawned_from implies origin, not
   implementation, so a terminal origin slice is expected.
