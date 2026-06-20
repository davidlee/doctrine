# IMP-129: Separate main from edge — decouple dispatch landing zone from daily authoring churn

## Problem

`main` serves two competing roles: (1) the active working branch where every
design/plan/slice tweak lands, and (2) every dispatch's delivery target. These
are different traffic classes — high-churn authoring (typo fixes, status bumps,
`plan.toml` updates) vs. low-noise dispatch landing. Every intervening commit
widens the merge-base gap for every in-flight dispatch. When two dispatches run
concurrently, every `plan(SL-NNN)` or `notes(SL-NNN)` on main advances HEAD and
drifts both bases. The `.doctrine/` authored files almost never conflict (per-slice
directories, append-mostly), so the drift is metadata noise, not content collisions
— but it still forces terminal merge work at the worst moment (close/reconcile).

## Root cause

Single branch serving two traffic classes with no separation of concerns.

## Proposal

Bifurcate: **`edge`** (daily authoring, high-churn) and **`main`** (dispatch
landing zone, low-noise).

- `edge` is the normally-checked-out branch. All design/plan/slice/notes commits
  land here. Workflow unchanged for authoring.
- `main` only advances at explicit `edge → main` merge points (per-slice-close,
  or periodic). Dispatch forks from `main`.
- At defined points, merge `edge → main`. Since `.doctrine/` TOML/MD files are
  per-slice and append-mostly, these merges will almost always be fast-forward or
  trivial one-liners.

Net effect: a 5-phase drive that today accumulates dozens of intervening commits
would accumulate zero (edge merges into main at close time, not mid-drive).
When two dispatches overlap, neither sees the other's authoring noise.

## What exists today

The mechanism is already in the code:

- **SL-127's ancestor-dominant ladder** picks the most-advanced candidate ref
  regardless of naming — it doesn't care whether trunk is `main` or `edge` or
  `stable`.
- **IMP-124 / IMP-101** (`deliver_to` config in `[dispatch]`) provides the
  single-source-of-truth config surface for the trunk delivery ref.
- `dispatch refresh-base` (SL-127) handles incremental advance mid-drive
  regardless of which ref is trunk.

## What needs building

1. **`[dispatch] deliver_to` config** (IMP-124/IMP-101) — single config slot
   naming the trunk delivery ref. Dispatch setup reads it; ladder uses it.
   Already scoped, not yet implemented.
2. **`edge → main` merge workflow** — could be a thin CLI verb (`doctrine trunk
   promote`) or a manual convention. The merge itself is `git merge edge` from
   a clean `main` checkout. Likely run at slice-close time or periodically.
3. **Default branch switch** — local convention: `git checkout -b edge` and set
   as default. No code change needed beyond the config default.
4. **Dogfood** — run a few dispatches with `main` as the landing zone while
   authoring on `edge`, measure drift reduction.

## Open questions

- Does `edge → main` promotion happen per-slice-close, or periodically (N
  commits / time-based)?
- Should `doctrine close` gate on "edge merged into main" or leave it to the
  operator?
- Does this need a governance surface (POL/STD) or is it a local convention
  encoded in config defaults?
