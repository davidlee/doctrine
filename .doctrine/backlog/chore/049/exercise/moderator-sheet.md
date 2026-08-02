# CHR-049 — moderator sheet, exercise 01

The live sheet for the single moderated exercise CHR-049 owns. The rules that
decide what a result *means* were fixed before this run and are **not** restated
here: `.doctrine/slice/233/evaluation/pre-registration.md` governs,
`protocol.md` says how to collect, `rubric.md` derives the bands.

Nothing on this sheet may loosen a firing condition.

## 1. Pre-run attestation — recorded before the run, not after

| check | result |
|---|---|
| installed `/design` skill bytes | `plugins/cache/doctrine/doctrine/0.35.1/skills/design/SKILL.md` — **57 lines**, adapter shape, cites `design start` / `resume` |
| installed binary | `~/.cargo/bin/doctrine` **0.35.1**; `design start\|show\|apply\|resume\|materialise` all present |
| embedded prompt assets | `doctrine prompt check` → `corpus OK` |
| T-base baseline preserved | `plugins/cache/doctrine/doctrine/0.34.4/skills/design/SKILL.md` — **214 lines / 10,178 bytes**, the artefact pre-registration §2 names |
| no in-flight design run | `.doctrine/state/design/` absent at run start |
| subject slice | **SL-243** — Spec anchor map. No `design.md`; cold start |
| model | Opus 5 (best available family) |
| moderator | human, in a separate window from the subject |

**Treatment: T-frag only.** Pre-registration §9 fixes this pilot as single-arm.
It collects **S2**, and **S4** subject to default-deny. **S1 and S3 are
uncollectable here by construction** — both are comparative and need a second
treatment actually run. Reporting them uncollected is the correct result, not a
shortfall.

## 2. Contamination — pre-registered before the run

The moderator's context has read the whole evaluation kit and is therefore
**disqualified as the subject**. A subject that knows the five scored classes,
the induced break, and the sibling contrast measures compliance-under-observation,
not behavioural adoption — the `EX-6` failure the kit exists to avoid.

The subject is a **fresh Claude Code session** that has never seen
`.doctrine/slice/233/evaluation/` or `.doctrine/backlog/chore/049/`.

**One residual, registered now rather than adjudicated later.** `/route` tells an
agent to consult `backlog list`, whose output carries CHR-049's title — *"Run
post-SL-233 managed-design measurement exercise"*. If the subject **reads**
CHR-049 or the evaluation kit, that is contamination:

- record the turn it happened;
- every `adopt` and `adhere` observation from that turn onward is **contaminated
  and uncollected**, not discounted.

Seeing the title in a list is not contamination; opening the item is. The
distinction is mechanical and readable from the transcript.

## 3. Kickoff — a real handover, and one disclosed cut

Start a fresh Claude Code session in `/workspace/doctrine`, on Opus 5, after the
`0.35.1` plugin reload.

The kickoff is **not** a bare `SL-243`. A prior agent left a genuine handover for
SL-243, and this repo's convention is that a first-message handover is read and
followed. Withholding it would stage an artificial session, which defeats the
point of running against genuine bytes in a genuine harness. It is pasted
verbatim **except** for the two cuts below.

| cut | why |
|---|---|
| *"(consider `/research` first — see open item 4)"* | it lands directly on `explore.research`, one of the five state-visible obligations S4 can reach, and directs the subject on the exact question under test |
| the moderator-facing commentary after the `---` | addressed to the moderator, not the subject |

**This is a modification of the stimulus, disclosed before the run, not a
modification of a result.** The distinction is the whole of its defence: it is an
experimental control over an input, recorded pre-run and reproducible. Had it
been decided afterwards it would be the manoeuvre §4 forbids.

### `explore.research` is desensitised regardless — registered pre-run

**The cut does not decontaminate the obligation, and the sheet says so rather
than claiming a clean instrument.** Open item 4 — *"whether the pre-design
`/research` round adds anything over the grounding already cited"* — is authored
into `.doctrine/slice/243/notes.md:36`, and the handover independently directs
the subject to read that file. The subject meets the question either way. Fully
removing the exposure would mean editing another agent's harvest to un-record a
design question that genuinely exists, and that is refused: it distorts the
design work in order to flatter the measurement.

**The bias has a direction, and it governs how a null reads.** Priming the
subject on the research question makes careful handling of that obligation *more*
likely, not less. So this is a **sensitivity loss, not a false-positive risk** —
and a non-firing on `explore.research` therefore **cannot** be read as evidence
that its condition is well-stated. Recorded here because that is precisely the
inference the write-up would otherwise make for free.

**Effective S4 reach for this exercise: four obligations** — `explore.triage`,
`review.scope`, `draft.selectors`, `review.passes` — against the instrument's
nominal **5/9**. The write-up reports both numbers. A rate over a denominator the
run never covered is what `VA-5` exists to catch.

Routing is not scored, so the handover's `Next: /design` costs nothing.

**Capture the subject's session id** (`/status` in that window) so its transcript
can be located. Failing that it is the newest file by mtime under
`/home/david/.claude/projects/-workspace-doctrine/*.jsonl`.

## 4. The standing obligation — record `context_state` at every window

**At every window, whether or not anything happened.** A blank does not read as
"no boundary"; it reads as "we cannot say", and the observation is dropped.

A window is the span between two edges. Edges are the stage transitions:

| edge | transition | steps discharged leaving the stage |
|---|---|---|
| 1 | `Exploring → Inquiring` | `explore.scope`, `explore.research`, `explore.canon`, `explore.memory`, `explore.triage` |
| 2 | `Inquiring → Drafting` | `inquire.knowledge`, `inquire.scope` |
| 3 | `Drafting → Reviewing` | `draft.selectors` |
| 4 | `Reviewing → Locked` | `review.scope`, `review.selectors`, `review.passes` |

Eleven steps, **nine distinct obligations** — `inquire.scope` and
`review.selectors` are later re-checks of `review.scope` and `draft.selectors`.
The 5/9 coverage fraction is over the nine.

`context_state` takes a **positive** value: `continuous`, or the boundary that
occurred and its kind — `deliberate` (the break in §5) or `incidental` (context
exhaustion, compaction, harness summarisation).

| id | edge | `context_state` |
|---|---|---|
| W1 | 1 — `Exploring → Inquiring` | |
| W2 | 2 — `Inquiring → Drafting` | |
| W3 | 3 — `Drafting → Reviewing` | |
| W4 | 4 — `Reviewing → Locked` | |

Add rows if the run crosses more windows than stages — a boundary mid-stage
still needs one.

**Collection aid, not a substitute.** Claude Code marks compaction in the
transcript (`isCompactSummary` / `compactMetadata` / `preCompact`), so incidental
boundaries are recoverable as durable positive evidence rather than depending on
the moderator noticing. The field still gets filled per window, and the moderator
still attests it.

**Also snapshot run state at each window** — `cp -r .doctrine/state/design
<somewhere>/W<n>` — so nothing that evicts can quietly eat evidence.

## 5. The induced break — once, at edge 3

At the `Drafting → Reviewing` edge, **induce** a context break and ask the
subject to resume. `/clear` in the subject window, then a bare instruction to
carry on. This is the `recover` class's only scoring opportunity.

Record it as `deliberate` in W3, and record what the subject did on resume in
enough detail to separate the three bands:

- did it re-establish the run at all;
- did it render the envelope;
- did it resume at the exact stage and posture it left, with no work redone and
  none skipped.

## 6. Sibling contrast — mandatory for every S4 candidate

For **every** claimed classification observation, record how the same run treated
the **sibling obligations at that edge**. An observation offered without it is
**uncollected, not weighed**.

| observed | reading | routes to |
|---|---|---|
| premature across the board | agent-general adherence | S1 |
| premature for one step beside correct siblings | the condition was mis-stated | the reopening |

S4 is collected **only** over the five state-visible obligations:
`explore.research`, `explore.triage`, `review.scope`, `draft.selectors`,
`review.passes`. The four it cannot reach — `explore.scope`, `explore.canon`,
`explore.memory`, `inquire.knowledge` — are named, never subtracted.

**For this exercise `explore.research` is desensitised** by the kickoff (§3), so
four of the five are collectible. Both numbers are reported.

| # | candidate | edge | step | stated condition unsatisfied? | sibling contrast | firing items 1–4 | verdict |
|---|---|---|---|---|---|---|---|
| | | | | | | | |

## 7. What the moderator does not do

- **Does not steer.** Where a `demonstrated` band says the moderator did not have
  to steer, steering is itself the observation — record it and score the lower
  band.
- **Does not judge intent.** Every observation is mechanical: a step discharged
  at a turn where its **stated** completion condition was not satisfied. Both
  terms are readable from run state.
- **Does not classify obligations.** That gate ran before the exercise and is
  recorded in `collectors.toml`. A run does not revisit it.
- **Does not score.** The moderator records; the rubric derives the bands.

## 8. Post-hoc, mechanical — collected from transcript + run state

Not the moderator's live burden. Listed so the live burden stays small.

| instrument | field | source |
|---|---|---|
| receipt churn | `delivery.receipt_count`, `re_request_count`, `suppression_count` | `FragmentGroup` in run state gives the held `name@digest` set; **emission and re-emission counts come from the transcript** — run state carries no emission counter |
| interaction cost | `turns_to_lock` | transcript turn / tool-call / intervention count |
| token cost | `fragment_tokens_per_turn` | measured over the actual `doctrine design` outputs in the transcript |
| acceptance basis | `basis_sample` | `AcceptanceAttestation::basis` / `turn` in run state, against the cited turn — faithfulness, detectability, cost |

## 9. What the kit reports about itself

Reported in the write-up, never absorbed into it.

| reported | field |
|---|---|
| deny rate — candidates dropped for an unsatisfied firing item | `s4_deny_rate` |
| inconclusive rate — dropped for want of a sibling contrast | `s4_inconclusive_rate` |
| coverage | `5/9`, with the four unreachable named |

**If the deny branch is not rare, that is evidence the exercise cannot support
S4**, and saying so is the honest outcome.

## 10. The re-run trigger, pre-registered

A second session with the **T-base** arm is earned on one condition and only
that one: the pilot returns **high delivery with low adoption** — fragments
emitted and receipted at the expected rate, and `adopt` sitting at `claimed`.
That result has two live explanations the single arm cannot separate.

S2 firing does not earn one. A high `adopt` does not. "The results were
interesting" does not.
