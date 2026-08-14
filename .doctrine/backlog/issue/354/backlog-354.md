# ISS-354: Slice follow-ups have no mechanical discharge gate

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

A slice's `## Follow-Ups` section is prose in `slice-NNN.md`. Nothing reads it.
Nothing checks it at reconcile, nothing checks it at close, and the lifecycle
transition to a terminal status does not consult it.

So a follow-up is discharged only if a human or an agent remembers to look. When
that memory fails, the item does not become overdue or blocked or visible — it
becomes **invisible**, because the artefact holding it is now closed and nobody
re-reads a closed slice.

`/close`'s pre-check does gate on the adjacent things: every finding terminal,
the `## Reconciliation Outcome` recorded, harvest done *or consciously rejected*.
That last clause is the shape this is missing — a conscious-rejection gate for
follow-ups, where today there is no gate at all.

## The evidence, and it is this issue's own provenance

One obligation, three slices, two near-misses:

1. **`SL-247`** carried *"At reconcile — contribute the post-capsule finding to
   `RFC-025`"* (`inq-7`, owner 2026-08-06), deliberately deferred to reconcile
   because *"before the implementation lands and the live probe runs, the finding
   is a prediction"*. Sound reasoning.
2. **`SL-247` was abandoned** — terminal, and it never reconciles. The obligation
   had no discharge path left. It survived only because `SL-254`'s scoping
   author happened to read the abandoned slice and wrote *"that obligation is
   homeless. It transfers here"* into `slice-254.md`. A manual rescue.
3. **`SL-254` closed** with it still undischarged, and the slice went `done`
   without a murmur. It was caught in a post-close conversation, by accident,
   while unpacking a *different* follow-up.

Two of the three steps were luck. The third — the abandonment — exposes a
structural hole rather than a discipline one: **`abandoned` is terminal and
skips reconcile entirely**, so a slice can reach a terminal state having never
visited the stage where its follow-ups were meant to be discharged.

`SL-254`'s own Follow-Ups had three items. One was nearly lost twice; one
(`OQ-2`, the backlog dispositions) was discharged only because the closing agent
volunteered it; one (the memory sweep) became `CHR-063`. That last is the
pattern that works — and it works because someone chose to mint a card, not
because anything required it.

## For consideration — the owner's proposal (2026-08-14)

Not a decision; recorded as the starting shape for design:

> Maybe we need a terminal state like `done` which is *after* `closed` — and
> maybe `closed` should be demoted to non-terminal — with a condition that all
> follow-ups are swept and either done or assigned durable, executable work
> tracking.

The substance is the **two-valued discharge rule**: a follow-up leaves a slice
only by being *done* or by being *rehomed onto something with its own lifecycle*
(a backlog item, a slice, a knowledge record, an RFC section). Prose in a closed
slice is neither, and that is the state this issue exists to make unreachable.

Whether that needs a new lifecycle state is the open design question. Three
positions worth weighing before adding one:

- **A new terminal state.** Faithful to the proposal. `ADR-009` §1 owns the
  state machine, so this is a governance change, and every consumer of the
  terminal set (`{done, abandoned}` — the close gate, the rollup, priority
  scoring's actionability) has to move with it. Real cost; buys a state that
  *cannot be reached* with follow-ups outstanding.
- **A gate on the existing transition.** Refuse `slice status <id> done` while a
  follow-up is neither discharged nor rehomed — the same shape as the existing
  refusal on an unresolved RV blocker, which already works. Much cheaper, and it
  reuses the seam `/close` already leans on. Does not address `abandoned`.
- **Advisory report only.** A `doctrine slice follow-ups <id>` leg that lists
  undischarged items, run at reconcile and close. Cheapest, catches the honest
  case, catches nothing when nobody runs it — which is the exact failure above.

`abandoned` needs its own answer under any of them: either abandonment forces a
follow-up sweep too, or abandoning a slice must explicitly rehome or explicitly
drop each one.

## The parsing problem, and why it is the crux

`## Follow-Ups` is free prose under the storage rule — *never queried/derived
data in prose*. A gate that regex-scrapes a Markdown section is exactly the
thing the storage model exists to prevent, and it fails closed on formatting.

So the real design question is not *what state* but **where a follow-up lives**.
Options: a `[[follow_up]]` table in `slice-NNN.toml` with a discharge field and
an optional rehome ref; or no new storage at all — require every follow-up to be
minted as a backlog item at the moment it is written, making the prose a
narrative index over durable ids and the gate a relation query. The second is
more in keeping with how the rest of the corpus works, and it is what `CHR-063`
did by hand.

## Sibling — do not solve these separately

`IMP-418` (*Reconcile slice owed items against phase sheet findings*) is the
same defect one ledger over: phase-sheet findings are gitignored runtime state,
`notes.md` § *Owed* is the tracked record that must survive them, and keeping
the two in step is *"currently a manual orchestrator sweep with no mechanical
backstop"*. Its evidence is `SL-248`, whose § *Owed* ledger drifted three times
and was rescued three times by a sweep that happened to run.

Same class, same argument, same objection (prose parsing), and plausibly the
same verb. Design them together.

## References

`SL-247` § Follow-Ups · `SL-254` § Follow-Ups · `IMP-418` · `ADR-009` §1 (the
state machine) · `ADR-003` §7 (the audit → reconcile → close seam) · `/close`
skill, step 1 (the harvest conscious-rejection gate — the precedent) ·
`CHR-063` (a follow-up that was rehomed correctly, by choice).
