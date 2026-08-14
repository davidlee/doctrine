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

## For consideration — the owner's first proposal (2026-08-14)

Not a decision; recorded as the starting shape for design. **Superseded the same
day** by § *The likely home — obligations*, which keeps the discharge rule and
drops the new lifecycle state. Kept because the three positions weighed below
still bound the design space, and because the reasoning that moved off a new
terminal state is worth having in the record. As first put:

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

## The likely home — obligations (`RFC-027`), owner 2026-08-14

> They might best be represented as **obligations** — the outcomes a slice must
> discharge — and gated at **audit** (either done, deferred, or abandoned).
> Obligations don't yet exist, but once they do, that would feel appropriate.

This is the better shape than anything sketched above, and it also answers the
§ *parsing problem*: an obligation is a first-class thing with a status, so the
gate is a status query rather than a Markdown scrape. It does not need a new
lifecycle state either — the transition already refuses on unresolved RV
blockers, and this is the same seam.

**Audit is the right gate, not close.** Audit is where the reconciliation brief
is assembled — the existing act of *enumerating what this slice owes* — so the
sweep rides work that already happens. `IMP-418` independently reached the same
placement for the sibling ledger (*"run at audit or close while the sheets still
exist"*), which is corroboration rather than coincidence: both are accounting
over things that stop being readable once the slice ends.

### It does not reopen `H9`, and the distinction has to be stated

`RFC-027` retains *"`H9`'s obligation-graph rejection"* explicitly. `H9` tested
**obligations as the smallest actionability unit** — sub-phase nodes inside
phase envelopes carrying their own `needs`/`after` edges, to expose parallelism
— and the study killed it on two legs that survive:

* **fact ownership** — seven of eight candidate obligation fields restate facts
  already owned by `EX-N`, `EX-N.text`, the per-phase `requirements` array,
  `REQ-442`, `REQ-443`, and `compute_next_phases()`;
* **an unchanged frontier** — obligation granularity moved the actionable
  frontier on none of `SL-233`, `SL-057`, `SL-229`.

Neither leg reaches this proposal, because it is a different construct wearing
the same word. **`H9`'s obligations are a scheduling primitive; these are an
accounting one.** A follow-up is by definition the thing that *no* `EX-N` owns —
that is what makes it lose-able — so the fact-ownership objection inverts into
an argument for it. And this makes no claim about parallelism or the actionable
frontier at all, so the frontier measurement is silent on it.

Do not enter this against `H9`'s reopening condition (*"a named consumer for
which obligation granularity moves the actionable frontier"*). It is not that
consumer and should not be presented as one, or a tested disposition gets
relitigated on evidence that does not bear on it.

### Naming hazard — the word is taken twice already

`H9`'s third finding already flagged *"a naming collision with the shipped
runbook obligation"*. There is a second: `src/design_run/run.rs` uses
`obligation` throughout for a **delegation** obligation — an inquiry node
handed out and owed back, with `outstanding_for` tracking the debt. A third
sense needs either a distinct word or a mandatory qualifier, decided before any
schema, not after.

### Two things the three-valued disposition must pin down

* **`deferred` has to mean *deferred onto something durable*** — a backlog item,
  a slice, an RFC section — not "still prose, later". Otherwise it is today's
  failure mode with a label on it. `CHR-063` is the worked example of `deferred`
  done right; `SL-247`'s `inq-7` is the same value with no durable home, and it
  is why this issue exists.
* **`abandoned` slices skip audit too.** An audit-time gate does not cover the
  case that started this — `SL-247` reached a terminal status without ever
  reaching audit, and its obligation survived only because a later slice's
  author happened to read it. Abandonment needs its own rule: force the sweep,
  or require each obligation be explicitly dropped or rehomed as part of
  abandoning.
