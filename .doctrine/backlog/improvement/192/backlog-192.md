# IMP-192: Solo slice reaches close with stale runtime phase-rollup + noisy binding warning

A slice driven **without dispatch** (solo or lean-serial) never moves its phases
`planned → in_progress → completed` in runtime state — that flip is the
`/execute`/dispatch path's job. So such a slice arrives at `/close`
substantively done (work implemented, RV verified, gate green) but with a stale
rollup (e.g. `0/2`, both sheets `planned`). The close pre-check "confirm X/X
complete" then mis-reads a finished slice as incomplete.

Compounding: when the rollup is then flipped manually
(`slice phase NNN PHASE-0N --status completed`), each flip emits
`phase-binding capture skipped … no code_start_oid stamped` — noise, because the
phase legitimately never entered `in_progress` under a binding on this path.

Fork-driven slices have a related variant: runtime phase state lives in the
fork's gitignored `.doctrine/state/` and never propagates to primary, so the
primary rollup is stale by design (close reads `2/5` for a done slice).

## Cost (RFC-011 instrumentation)

Recurred at SL-166 close, SL-163 close, SL-166 orientation. ~4 investigative
tool calls each time to reconstruct "where does phase state live / why N/M / how
to reconcile" before the picture is clear.

## Proposal

- `/close` (and/or `/audit`) recipe for "solo/fork slice, audited-done, runtime
  rollup stale → reconcile the rollup before transition", so the pre-check does
  not read an expected-stale rollup as dropped work.
- Suppress the `phase-binding capture skipped` warning on the legitimate
  no-binding flip path (or downgrade to info).

Surfaced by: RFC-011 case-notes. Platform issue — the lifecycle/runtime-state
split and the close skill ship with doctrine; affects every user driving a slice
outside dispatch.

## Capsule-driven variant (2026-09-17)

The fork paragraph above generalises further, and the capsule case is the sharp
end of it: a slice sent to an oubliette capsule (`just send <slice> <slot>`) has
its runtime tier in the *guest's* checkout for the whole time it is out.

Unlike a fork, that state **does** come home — it rides the separate state ref,
and `just back` runs `capsule adopt` plus an ignore-gated restore. But it comes
home into the **landing worktree** (`.worktrees/SL-NNN-<slot><gen>`), never into
the checkout the slice was sent *from*. So `~/dev/doctrine` on `edge` keeps the
sheets it had at send time, and `doctrine status` / `doctrine slice phases N`
there answers from them with nothing marking the answer as superseded.

Two moments, and the second is the one the fork paragraph does not cover:

1. **While the slice is out.** The origin's sheets are frozen at send. Every
   phase flip the capsule agent makes is invisible here, and there is no
   "this slice is on loan" state for anything to report.
2. **After it lands.** Two live copies — the worktree's (current) and the
   origin's (stale) — with no marker on either. `scripts/oubliette.sh`'s closing
   line is *"Audit here, not on edge"*, which is advice to a human reading
   terminal output, not a guard, and nothing downstream reads it.

The shared root is this item's: the runtime tier is per checkout and nothing
tracks which copy is authoritative. What the capsule case adds is that the
authoritative copy genuinely moves and comes back, so the answer is not only
"reconcile at close" — it is that a slice can know it is somewhere else.

### Proposal (this variant)

- A slice can record that its runtime tier is elsewhere — the unit is already
  recorded on the capsule side (`capsule <slot> unit <token>`, and the
  assignment record carries it), so the missing half is doctrine-side.
- `doctrine status` / `slice phases` say so rather than answering from a frozen
  copy: *"SL-NNN's runtime state is in capsule c since <date>"*, or the same for
  a landed worktree that has a newer copy.
- Failing both, the cheapest honest version: mark the origin's sheets at send
  and refuse to read them as current until a `back` has reconciled them.

Not reproduced on SL-245, and worth saying: that capsule was stopped by a stale
guest toolchain before any phase moved, so both copies still read `planned`. The
mechanism is read off the code paths, not off a divergence anyone has seen.
