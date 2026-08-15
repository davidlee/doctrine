# QUE-218: Does the inquiry map earn a semantic tier?

## The question

`RFC-026` (*Design review response effectiveness*) sets out, at its evidence
item **E8.5**, a capability table comparing the shipped managed design run
against `jkaloger/hydra` — an independent implementation of the same originating
idea. Four capabilities sit on hydra's side and not ours, and they are all one
kind of thing: **what the map means to someone using it**, as against what the
store guarantees under adversarial conditions.

| capability | our state | item |
|---|---|---|
| cascade — a changed answer reopens children and `blocked_by` dependents | nothing on the inquiry axis | `IMP-386` |
| `prior` — one superseded answer retained, so a reopened node is asked *"does this still hold?"* | `resolved.transition(Open)` drops the disposition | `ISS-303` → `IMP-386` |
| `cauterised_by` — B's answer kills A; A records which node killed it | `pruned` is a bare lifecycle token | `ISS-300`, `IMP-387` |
| `rejected: [{option, why_not}]` typed on the answer | prose only, at every tier | `IMP-388` |
| `ready` / `next` computed from graph state | the cursor is declared, never derived | `IMP-389` |

`RFC-026` **E8.6** states the asymmetry plainly: SL-233 answered the
store-guarantee class four times (`DEC-078`, `DEC-086`, `DEC-088`, `DEC-092`)
and the meaning class once (`DEC-062`) — and `ISS-300` shows that one answer was
not extended to the two adjacent moves.

## Why it matters

**`RFC-026` E8.3 is the counter-evidence, and it is strong.** SL-243's run
(`dr-019fc13a`) reached nine nodes with **zero edges of either kind** —
`grep -c parent` over run state returns 0, every frontier entry carries
`needs_in_degree=0` — across seventeen revisions, driven by a competent agent
who was never told not to use them. Both edge kinds shipped with full render
support: `is_blocked`, `blockers()` ranked by in-degree with a naming `reason`,
frontier exclusion, a `blocked` total. So `active_path` was empty by
construction, not by cursor state, and the "map" was a flat capped list.

E8.3 records the direction this cuts: **the render is not the missing part.**

That leaves three readings, and nothing in the corpus separates them:

1. **Under-featured** — the tier is genuinely missing and agents route around
   the gap. Build the four improvements.
2. **Under-surfaced** — the machinery exists and nothing obliges an agent to
   look at it (`ISS-299`: zero references to `frontier`, `map`, `design show` or
   a decision tree across every shipped design-prompt asset, including the
   `inquiry.md` fragment delivered every turn). Surface it first, then re-measure.
3. **Not needed at this altitude** — a slice-scale design run holds few enough
   questions that a flat list is the right shape, and the edge machinery is
   already over-built.

`ISS-299` is a **prerequisite for answering this**, and it cannot be recorded as
a `needs` edge because knowledge records carry no dep/seq axis. Read it as one:
reading (2) is untested until the map is put in front of an agent, and building
under reading (1) before that is answering `RFC-026`'s meaning-tier gap with more
machinery instead of more use.

**A second confound landed with the 2026-08-15 friction sweep.** `ISS-360` — batch
declare refuses an in-batch parent chain two levels deep — means the natural way to
author a tree (declare it top-down in one submission) is **mechanically refused**.
The workaround is one submission per level, costing a revision each, and it is not
discoverable. So E8.3's zero-edge measurement is not clean evidence of disuse: at
least part of it is an obstacle in front of the feature rather than indifference to
it. Reading (2) — *under-surfaced* — now has two distinct mechanisms behind it
(`ISS-299` nothing points at the map; `ISS-360` the map resists being built), and
both must clear before a re-measurement means anything. `CHR-065` carries that
re-measurement.

## What turns on it

`IMP-386`, `IMP-387`, `IMP-388` and `IMP-389` all carry `needs QUE-218`. They are
a coherent batch — one slice's worth — and they are blocked on warrant, not on
effort. `ISS-300` and `ISS-303` are deliberately **not** gated: they are
correctness defects in the moves that already exist (a lifecycle move that keeps
no reason; a reopen that discards a disposition `DEC-062` exists to preserve),
worth fixing whether or not the tier is deepened, and `ISS-303`'s fix is the
`prior` capability that `IMP-386` would build on.

## References

- `RFC-026` E8.3, E8.5, E8.6 — the measurement, the capability table, the asymmetry
- `ISS-299` — nothing surfaces the map (the prerequisite reading)
- `ISS-300`, `ISS-303` — the ungated correctness half
- `IMP-386`, `IMP-387`, `IMP-388`, `IMP-389` — the gated batch
- `DEC-062` — resolved inquiries require explicit semantic disposition
- `SPEC-029` — Design run engine
