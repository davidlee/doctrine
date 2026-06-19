---
seq: 0018
scope: backlog
target: SL-121 (active) vs IMP-075 / IMP-102 / IMP-103 (integrate cluster stragglers)
confidence: med
reversible: yes (proposal only; no slice/backlog transition — fence holds)
---
## What
`dispatch sync --integrate` has a **six-item cluster**, and it is *half*-coordinated:
the active slice **SL-121** ("clean exit state and legible outcome", currently in
`design`) explicitly bundles **three** of them — ISS-022 (stale index), ISS-030
(stale worktree / close step-3a verify reads ref not tree), IMP-078 (silent
outcome) — all linked `slices: SL-121`. Good. But three more sit **outside** the
bundle and at least one collides with SL-121's own code:

- **IMP-075** — "Extract `with_journaled_projection` — collapse the duplicated
  journal-commit/apply/journal cycle shared by dispatch `prepare_review` **and
  integrate**." SL-121's design names its target as `src/dispatch.rs::integrate` /
  `run_integrate` (the integrate exit path). **IMP-075 refactors the same
  `integrate` body.** Done independently, one will re-touch / conflict with the
  other: SL-121 rewrites integrate's exit-state + reporting; IMP-075 restructures
  integrate's journal cycle. Sequencing them apart guarantees rework on a shared,
  delicate CAS/replay path.
- **IMP-102** — "close: structural gate — slice status `done` refuses when
  dispatched code not integrated to trunk." This is close-step adjacent; SL-121
  already touches **close step-3a** (the ISS-030 ref-vs-tree verify). Same close/
  integrate seam.
- **IMP-103** — "`dispatch sync --integrate --help`: clarify `--trunk` dry-run
  semantics." Doc/UX on the very command SL-121 is reworking; cheapest to fix while
  the surface is open.

So the risk is **uncoordinated re-touch of one delicate operation**: SL-121's own
design rationale is "fixing them piecemeal would re-touch the same exit path
repeatedly, so they bundle" — that exact logic argues for pulling IMP-075 (and
likely IMP-102/103) into the same bundle, yet they're currently outside it. The
slice is in `design` *now* — the cheapest moment to decide scope.

## Options
1. **Fold IMP-075 into SL-121; ride IMP-102/IMP-103 along.** One pass over the
   integrate/close seam: behavior fixes + the `with_journaled_projection` extraction
   + the close-gate + the help. Tradeoff: larger slice, but honours SL-121's own
   "don't re-touch piecemeal" rationale and eliminates the rework/conflict; refactor
   (IMP-075) lands *with* the behavior change it would otherwise fight.
2. **Sequence, don't fold: IMP-075 immediately after SL-121, on an explicit
   `after` edge.** Keep SL-121 tight to the three exit-state defects; do the
   extraction next, accepting one deliberate re-touch (now informed by SL-121's
   final shape). Tradeoff: smaller slices, clearer review; one knowing rework pass
   instead of an accidental collision. IMP-102/103 as separate close-adjacent items.
3. **Leave as-is (independent).** Tradeoff: zero coordination cost now; but IMP-075
   and SL-121 edit the same body with no ordering, the classic merge-collision /
   double-rework on a CAS path — the precise failure SL-121 was bundled to avoid.

## Recommendation
Option 1 for **IMP-075** specifically (fold into SL-121), Option-2-style sequencing
for IMP-102/IMP-103 (close-adjacent, can follow). Rationale: IMP-075 is a *refactor
of the same function* SL-121 rewrites — folding it means the extraction is shaped by
the new exit-state/reporting code in one coherent pass, which is strictly cheaper
than refactoring integrate twice. IMP-102 (close structural gate) and IMP-103 (help)
are real but separable; tag them to follow so they don't bloat SL-121, but record
the linkage so they aren't forgotten once the seam closes. Decide *now*, while
SL-121 is still in `design` — folding after the plan locks is far costlier.

Decisions deferred to YOU:
- (a) **fold IMP-075 into SL-121, or sequence after** — refactor-with vs
  refactor-after the behavior change on the shared `integrate` body.
- (b) **IMP-102 / IMP-103 disposition** — ride along, or explicit `after: SL-121`
  followups (and is IMP-102's close-gate in or out of scope given SL-121 already
  touches close step-3a?).
- (c) whether SL-121's scope text should be amended to name the coordination with
  IMP-075 explicitly (so the design review accounts for it).

## Next doctrine move
```
# read the overlap (read-only):
doctrine slice show SL-121                 # scope: integrate exit path + close step-3a
doctrine backlog show IMP-075              # same integrate body (with_journaled_projection)
doctrine backlog show IMP-102 IMP-103

# coordinate (NOT executed — fence forbids slice/backlog transition):
#  fold:  during SL-121 /design, add IMP-075 to the bundle (amend scope) then on
#         close transition IMP-075 as folded-in.
#  or sequence: doctrine backlog <after verb> IMP-075 --after SL-121   (verb per --help)
```
(Verbs described, NOT executed — fence forbids slice scope locks / backlog transitions.)

## Illustration (optional)
None — a sequencing/coordination call over the active slice, not a diff. The
evidence is SL-121's design naming `src/dispatch.rs::integrate` and IMP-075 naming
the same `integrate` journal cycle.
