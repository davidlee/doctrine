# IMP-169: Close-gate: recognize manual/external dispatch integration

## Problem

The slice close-gate refuses `reconcile → done` for a dispatched slice unless the
dispatch journal carries a **trunk row** written by `dispatch sync --integrate
--trunk`:

> `slice SL-147 → done: refused — dispatched code not integrated to trunk:
> dispatched but no trunk row — integrate --trunk never completed`

This assumes the dispatch-native integrate path is **always** the landing
mechanism. When a slice's code is integrated by another sanctioned route — e.g. a
**manual merge onto trunk** (the recovery path when the pre-dispatch `edge→main`
promotion was skipped and the bundle must replay over an advanced trunk) — the
code is correctly on trunk, green, and verified, but no journal trunk row exists,
so `done` is unreachable. There is no `--force`, no journal-record verb, and the
dispatch integrate path cannot be retro-fitted once trunk has advanced past the
prepared base (it would re-project code trunk already holds).

## Worked example (SL-147)

SL-147 forked from a stale `main` (33 commits behind `edge`). At close, the impl
bundle `review/147` was merged onto `edge` manually (one conflict resolved:
`review.rs` PrimeArgs domain_map burn), `just check` green, then `edge`→`main`
promoted. `edge == main == bfce657d`, code shipped. But `slice status 147 done`
refuses — no journal trunk row. Per user decision, SL-147 is left at `reconcile`
(shipped but lifecycle-incomplete) and this item captures the gate gap. See
RV-157 F-3.

## Possible directions

- A sanctioned verb to record an external/manual integration into the journal
  (write the trunk row from an observed trunk oid), gated on trunk actually
  containing the audited units.
- Or relax the gate to accept "trunk contains the admitted close_target OID"
  (content reachability) rather than requiring the journal row specifically.
- Either must stay fail-closed: never pass `done` when the code is *not* on trunk.
