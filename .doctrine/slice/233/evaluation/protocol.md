# The moderator protocol

How CHR-049's single moderated exercise is run, and what its moderator is
obliged to record. SL-233 owns this document; CHR-049 owns the live run and its
interpretation (that chore's Boundary says so).

The rules that decide what a result *means* are in
[`pre-registration.md`](pre-registration.md) and were fixed before any run. This
document is about collection, not interpretation, and nothing here may loosen a
firing condition.

## The standing obligation — `context_state`

**At every window, the moderator records the `context_state` field.** Not when
something notable happens: at every window, whether or not anything did. The
field name is single-sourced as `context_state_field` in
[`collectors.toml`](collectors.toml), and the kit's tests assert that this
document names it.

A window is the span between two edges of the design run. For each one, record:

| field | what goes in it |
|---|---|
| `id` | the window's label |
| `edge` | the stage transition it ends at |
| `context_state` | **positively**: `continuous`, or the boundary that occurred and its kind |

Boundary kinds worth naming distinctly, because the enumeration that omitted the
commonest one is what made this a standing obligation rather than an observation:

- **deliberate** — the break and resume the protocol induces (below);
- **incidental** — context exhaustion, compaction, or harness-initiated
  summarisation. *Nothing marks these.* They are the ones that get missed, and
  missing one is indistinguishable from continuity unless the moderator was
  watching for it.

**Why this is an obligation and not a nice-to-have.** The classification signal's
firing condition has four items and all four are required. Item (4) *is* this
record. Absent it, item (2) — "the run crossed no context boundary at that edge" —
is **UNDECIDED**, and the observation is **uncollected, not weighed**. The
condition is satisfied only on positive evidence and never by the absence of a
mark, so a blank `context_state` does not read as "no boundary". It reads as "we
cannot say", and the observation is dropped.

## The induced break

Once per exercise, at the `drafting -> reviewing` edge, the moderator **induces**
a context break and asks the agent to resume. This is the `recover` class's only
scoring opportunity, and it is deliberate so that a class which would otherwise
depend on an accident has an occasion.

Record the break as `deliberate` in `context_state`. Record what the agent did on
resume in enough detail to separate the three `recover` bands: did it re-establish
the run at all; did it render the envelope; did it resume at the exact stage and
posture it left.

## What the moderator does *not* do

- **Does not judge intent.** Every observation in this kit is mechanical. The
  classification signal asks whether a step was discharged at a turn where the
  step's **stated** completion condition was not satisfied — both terms readable
  from run state. Formulations asking the moderator whether the agent *could
  honestly have completed* the step, or whether it *treated a lens as
  gate-worthy*, were withdrawn (pre-registration §8) precisely because an
  unadjudicable falsifier leaves DEC-104 as unfalsifiable as no falsifier at all.
- **Does not classify obligations.** Classification is an authoring gate over an
  obligation's **text**, decidable before any run. It is already applied and
  recorded in `collectors.toml`'s gate record. A run does not revisit it.
- **Does not score.** The moderator records observations; the rubric derives
  bands. Keeping those apart is why no transcript fixture states its own score.
- **Does not steer.** Where a `demonstrated` band says "the moderator did not have
  to steer", steering is itself the observation — record it and score the lower
  band.

## The sibling contrast

For **every** claimed classification observation, record how the same run treated
the **sibling obligations at that edge**. Both branches are collectible results:

| observed | reading | routes to |
|---|---|---|
| premature across the board | agent-general adherence | the delivery signal S1 |
| premature for one step beside correct siblings | the condition was mis-stated | the reopening |

**An observation offered with no sibling contrast is uncollected, not weighed.** A
signal with more than one live explanation routes to none of them.

CHR-049 specifies a single exercise, so the **within-run** contrast is the
collectible one. Cross-run corroboration would strengthen it and is
**opportunistic**, stated rather than assumed.

## What the kit reports about itself

These are reported in the exercise's write-up, not absorbed into it. A thin
instrument that says so is more use than a thin instrument that does not.

| reported | field | why |
|---|---|---|
| **deny rate** | `s4_deny_rate` | the share of candidate classification observations dropped because a firing item was not positively satisfied. Default-deny lowers the firing probability by design; **if the deny branch is not rare, that is evidence the exercise cannot support the signal**, and saying so is the honest outcome |
| **inconclusive rate** | `s4_inconclusive_rate` | the share dropped for want of a sibling contrast |
| **coverage** | `5/9` | the classification signal reaches five of the nine 2a obligations. The four it cannot reach are **named** in the gate record. A rate reported over a denominator the instrument never covered is worse than no rate |

## What one exercise cannot establish

Restated here so a moderator does not quietly assume it away mid-run.

- **`N=1`.** The reopening's one-shot rule and cross-run corroboration are not
  exercisable within this exercise. The one-shot rule binds the step's post-close
  life and is a **commitment**, not a control this run collects.
- **The classification signal may not fire at all.** That is a result, reported as
  the deny rate — not a gap to be filled by relaxing an item.
- **Outcome alone is not proof.** See [`README.md`](README.md).
