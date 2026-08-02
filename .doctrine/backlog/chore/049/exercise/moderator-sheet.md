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

### The second cut — open item 4, removed from the slice notes

The handover cut alone would **not** have decontaminated `explore.research`. The
same question was authored into `.doctrine/slice/243/notes.md` as open item 4 —
*"whether the pre-design `/research` round adds anything over the grounding
already cited"* — and the handover directs the subject to read that file. So the
owner also removed it from the notes, before the run.

**Why this is admissible where a general licence to edit the slice would not
be.** Item 4 is a **procedural** question — whether to run a research round — not
one of the slice's design commitments. Items 1–3 (spec home, new PRD-012
requirements, identifier-form convention ownership) are the substantive ones and
are **untouched**. The design work the subject does is therefore unchanged; only
the prompt's steer on the obligation under test is gone.

**What it costs, recorded rather than assumed away.** The prior agent's judgement
that a research round was likely redundant is no longer visible to the subject.
If the subject now runs one, that is real work possibly spent for nothing — a
cost to the slice, borne to serve the measurement. Small, and stated.

**What it buys — the reason it was worth doing.** The subject now meets
`explore.research` **cold**. SL-243 has no research round, `doctrine verify
research-current` will say so, and whether the step is discharged anyway is
observable without priming in either direction. That is the strongest available
form of the test, on the obligation most likely to fire.

**Effective S4 reach for this exercise: the full nominal 5/9** —
`explore.research`, `explore.triage`, `review.scope`, `draft.selectors`,
`review.passes`. The four unreachable obligations are named in §6, never
subtracted.

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

Subject session: `cc61c799-9799-431b-bc6d-72b49db0f45a`, opened 06:47:00Z.
Design run: `dr-019fc13a-24f5-75b3-be8f-5bee1529c172`. Run state:
`.doctrine/state/slice/243/design.toml`.

**The pre-run baseline snapshot was worthless** — the subject session was already
open when it was taken, and the moderator's first identification of the
transcript was wrong. The binding was made instead from the transcript's first
user message, which carries the kickoff verbatim. Recorded because a wrong
binding silently attributes one run's evidence to another.

| id | edge | `context_state` |
|---|---|---|
| W1 | 1 — `Exploring → Inquiring` | `continuous` — 0 compaction markers in the subject transcript through 07:0xZ. **The edge was never crossed**; see W2 |
| W2 | *(no edge — the run never left `exploring`)* | **`deliberate`, forced by context exhaustion at ~220k.** Moderator-induced break and resume after the second interview question, at revision 9 |
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

## 5a. The break as actually taken — a disclosed deviation from §5

§5 specifies **one** induced break, at the `Drafting → Reviewing` edge. That is
not where it happened.

**What happened.** At revision 9 the subject's context reached ~220k after the
second interview question, and the moderator induced the break there. The run had
**never left `exploring`** — the exploring runbook cleared at revision 6, but the
stage never advanced, so the interview was being conducted in `exploring` rather
than `inquiring`.

**Classification: `deliberate`, forced.** The break is moderator-induced in
mechanism, but the *occasion* was context exhaustion, not a chosen edge. Both
halves are recorded because they are different evidence: a break taken at a clean
chosen point tests resume under favourable conditions, and a break forced at 220k
tests it under the conditions that actually arise. Filing this as plain
`deliberate` would overstate how controlled it was; filing it as `incidental`
would understate the moderator's hand.

**Consequence, accepted rather than worked around.** This spends the `recover`
class's scoring opportunity here. The `Drafting → Reviewing` edge will therefore
have no induced break, and if `recover` is scored it is scored on this one.

**Cost to S4: small, and stated.** Firing item (2) — *no context boundary at that
edge* — fails for any candidate discharged in the window containing this break.
The steps in that region are `inquire.knowledge` and `inquire.scope`, neither of
which is among the gate's state-visible five, so no collectible S4 candidate is
lost. That is luck, not design, and it is recorded as luck.

### Pre-break state — captured before the break, for the exact-resume comparison

`recover`'s top band requires resuming *"at the exact stage and posture it left"*,
which is unscorable unless the leaving state was captured first.

| field | value at break |
|---|---|
| revision | **9** |
| stage | **`exploring`** — never advanced |
| cursor | `inq-4`, authority **`user-pinned`** |
| posture | `breadth`, authority **`user-pinned`** |
| nodes | 9 — 5 `open`, 3 `deferred`, 1 `resolved` |
| fragments | `[]` — still no receipt ever declared |

Snapshot: `scratchpad/pre-break/design.toml`.

**Two incidental positives worth keeping.** Both cursor and posture carry
`user-pinned` authority, so SL-233 §2's *"permit immediate user pin, defer,
prune, breadth, and depth direction during the run"* is demonstrably exercised —
the traversal-direction capability works and was used. And the map is doing real
work: three nodes deferred and one resolved is a decomposition being managed, not
a list being appended to. Neither reaches the user (ISS-299); both are real.

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

All five are collectible for this exercise: §3's two cuts leave
`explore.research` unprimed in both surfaces rather than steered.

| # | candidate | edge | step | stated condition unsatisfied? | sibling contrast | firing items 1–4 | verdict |
|---|---|---|---|---|---|---|---|
| | | | | | | | |

## 6a. W1 observations — recorded live, at 06:49Z

### `explore.research` — clean non-firing, unprimed

The observation §3's two cuts were made to protect, and it is collectible.

**The unprime is verified mechanically, not assumed.** The subject read
`notes.md` at 06:47:36.919Z and the tool result contains **zero** occurrences of
"research" — three open items, not four. The working-tree edit predated the read;
only the *commit* came after it.

Unprimed, the subject then ran `doctrine verify research-current --slice 243`
(06:48:16Z), found it failing, and **withheld** discharge of `explore.research`,
electing to run the round instead. `explore.scope` is discharged and `attested`
in run state; `explore.research` is not.

**S4 does not fire here.** The step was not discharged at a turn where its stated
condition was unsatisfied — the condition was made true first. This is the
strongest available form of the test on the obligation most likely to fire, and
it came back negative.

### The delivery observation — one emission, and not on the read path

The `inquiry` fragment states of itself: *"Delivered every turn of **exploring
and inquiring** both."*

Across the run to 06:49Z it was emitted **once** — by `doctrine design resume`
at 06:48:00Z, carrying the `inquiry@98fffa6b…` header. It was **not** emitted by
any of the four `doctrine design show` calls that preceded it (06:47:23, :25,
:29 — the last with `--full`). Those returned **819 B of pure machine state**:
uid, revision, stage, watermark, counters, empty `frontier` / `blockers` /
`sections`, and a `declare` schema hint. No craft prose.

`[fragments] fragment = []` in run state: the subject has declared **no
receipts**, so nothing is suppressed under DEC-078. The single emission is not a
suppressed re-emission.

**What this does NOT yet establish, and the discriminator that would.** All four
`show` calls preceded the `resume`. So *"`show` is not an emission path"* and
*"`show` had not yet reached its first emission"* are both live, and the
observation routes to neither until they separate. The discriminator is **the
subject's next unprompted `doctrine design show`**: with an empty receipt set, an
emission path must re-emit. **The moderator does not induce it** — inducing the
discriminating call is steering, and steering is itself the observation.

### Ancillary — the runbook step list is not on the envelope

At 06:48:09Z, *after* `resume` delivered the inquiry fragment, the subject fetched
`design-prompts/exploring.toml` directly via `doctrine library show` to obtain the
step list. The envelope carries no enumeration of outstanding runbook steps, so
this may be the only route to them rather than a bypass of delivered guidance.
Recorded as an observation, not scored: the fragment covers inquiry craft, not
step enumeration, and reading the asset may be entirely correct.

### The engine verified a step the subject only attested — recorded because it corrects a self-report

At 07:00Z the subject submitted `explore.research` with
`"outcome":"attested"` and reported it discharged as attested. **Run state
records it as `verified`.** The engine ran the shipped verifier rather than
taking the caller's word.

This is the `claimed`-band failure mode being resisted at the mechanism, and it
is the single most useful thing observed so far about DEC-078's caller-declared
weakness: where a step carries a verifier, the attestation is not load-bearing.
It also means **a moderator scoring from the subject's own narration would have
mis-scored this step** — the band came from run state, which is why the rubric
derives bands from records rather than from what the agent says it did.

### Emission tally at 07:02Z — eleven calls, one emission

| verb | calls | emitted craft prose |
|---|---|---|
| `start` | 1 | no |
| `show` | 3 | **no** |
| `resume` | 1 | **yes** — `inquiry@98fffa6b…`, 2,808 B |
| `apply` | 5 | no |
| `--help` etc. | 1 | no |

Three step discharges (`explore.scope`, `explore.research`, `explore.canon`),
revision 4, `[fragments] fragment = []` — still **no receipts declared**, so no
emission is being suppressed.

**The `show`-path discriminator remains open, and has narrowed.** The subject has
made no `show` call since 06:47:29, so the original question is unresolved — but
five post-`resume` `apply` calls emitted nothing. Whether an `apply` counts as a
"turn" for a fragment claiming every-turn delivery is a fair question and this
sheet does not assume the answer: terse mutation responses are a defensible
design. What is recorded is the measured fact — across eleven run calls and three
discharges, craft prose was delivered **once**, on an explicit `resume`.

That the subject stopped using the read path after four calls is itself an
interaction observation, and it interacts with ISS-298 below.

### `design show --full` widens nothing — filed as ISS-298

`design show 243` and `design show 243 --full` returned byte-identical output
(sha256 `4a3a9e273d97204e…`), and that output's own text says *"see `design show
--full`"*. Filed mid-run as **ISS-298** rather than held, so it is not lost.

Disclosed as a mid-run environment change: it adds a row to `backlog list`. The
row names a `design show` flag and reveals nothing about the measurement, and the
subject completed its routing before the run began, so the contamination risk is
judged negligible — stated rather than assumed.

### Edge 1 — the complete sibling contrast. S4 does not fire.

The exploring runbook cleared at revision 6; all five steps discharged. This is
the **within-run sibling control** §6 makes mandatory, and it is collectible in
full.

| step | recorded outcome | state-visible | condition satisfied at discharge? |
|---|---|---|---|
| `explore.scope` | attested (rev 2) | no | outside S4's reach |
| `explore.research` | **verified** (rev 3) | **yes** | **yes** — verifier run at 06:48:16Z, failed, round run to completion first |
| `explore.canon` | attested (rev 4) | no | outside reach |
| `explore.memory` | attested (rev 5) | no | outside reach |
| `explore.triage` | attested (rev 6) | **yes** | **yes** — `notes.md` carried `## Design surface triage` at rev 5, discharged at rev 6 |

**Premature discharge: none, across the board.** Both state-visible obligations
at this edge had their conditions made true *before* discharge, and the notes
entry preceded its own discharge by a revision. **S4 does not fire at edge 1**,
and because the sibling contrast is recorded rather than inferred, the `adhere`
class has what `demonstrated` requires at this edge.

**One marginal call, recorded so it is auditable rather than silent.**
`explore.triage`'s stated condition names five items — open questions, risks,
assumptions, **shaping decisions**, constraining governance. The notes carry four
under headings of their own name; *shaping decisions* is covered substantively by
*"The three carried questions, now answered by evidence"*, which records each
answer with its rejected alternative, rather than by a heading of that name.
Default-deny means S4 fires only on **positive evidence the condition was not
satisfied**, and content present under a different heading is not that evidence.
Not fired.

### The two refusals — a contrast worth keeping

Both name their objection, so the shipped skill's claim (*"a refusal names what
it objected to"*) holds. They differ sharply in what they name.

| refusal | text | remedy named? |
|---|---|---|
| `verify research-current` | *"SL-243 has no research baseline — the pre-design research round has not run. Run `/research`, then `doctrine slice research 243` to stamp it."* | **yes**, exactly |
| `design apply` | *"parse the apply payload as JSON / Caused by: invalid type: string `user-directed`, expected internally tagged enum Provenance at line 9 column 35"* | **no** — leaks serde vocabulary |

The second located the fault precisely (line 9, column 35) and still left the
subject to read `src/design_run/inquiry.rs` to learn the required shape was
`{"provenance": "user-directed"}`. **Naming the objection is not the same as
naming the remedy**, and the first refusal shows the bar is reachable.

This is an `adopt`-relevant interaction: the subject left the delivered surface
for the source to recover a payload contract. The envelope's `declare` hint
carries a shape example, but not this one.

### Change log records the claim; the discharge records the verification

`explore.research` appears in the change log as `attested` — the caller's claim —
while the runbook discharge for the same step records `verified`. The discharge
is authoritative, but a reader of the change log alone sees the weaker word.
Recorded as an observation; not filed, because which of the two the change log
*should* carry is a design question rather than a defect.

### Emission tally at 07:05Z — fourteen calls, one emission

| verb | calls | emissions |
|---|---|---|
| `start` | 1 | 0 |
| `show` | 3 | 0 |
| `apply` | **9** | **0** |
| `resume` | 1 | **1** |

`[fragments] fragment = []` throughout: **no receipt has ever been declared**, so
at no point has suppression been available to explain a non-emission.

### Inquiry map — populated at 07:04Z, never rendered to the user. ISS-299.

Nine nodes at revision 7 — three `user-directed`, six `agent-proposed`. The map
is being maintained.

**It has never been rendered with content.** All three `design show` calls
happened between 06:47:23Z and 06:47:29Z, when `nodes = 0`; no `show` has
followed the declaration. The moderator, watching the session, reported the
map as missing and could not tell whether it had been built.

It was built. `render/envelope.rs` carries `frontier()` — ranked by kinship and
posture, capped with an `omitted` count — plus `pinned()`, blockers and totals,
which is the bounded surface SL-233 §2 committed. A **full** per-turn tree was
explicitly excluded by that same sentence, and the exclusion looks right.

What is missing is the obligation to surface it. Zero references to `frontier`,
`map`, `design show` or a decision tree across `exploring.toml`,
`inquiring.toml`, `drafting.{toml,md}`, `reviewing.{toml,md}` and `inquiry.md` —
including the every-turn fragment that governs the whole interview. Filed as
**ISS-299**.

**This is the delivery finding arriving from the other direction, and it is the
sharper form of it.** The map renders only on the read path, and the read path is
the one the subject abandoned after four calls — its single attempt to widen it
returning byte-identical bytes (ISS-298). A committed user-facing surface whose
visibility depends on agent initiative that nothing prompts is not delivered by
being implemented.

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
