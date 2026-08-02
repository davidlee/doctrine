# The SL-233 evaluation kit

The instrument CHR-049 runs. SL-233 leaves behind the fixture, the moderator
protocol, the scoring rubric, and the mechanically collectible evidence surfaces;
that chore owns the live run, against genuinely installed bytes, after this slice
closes.

## Read this part first

**A successful outcome alone is not proof of behavioural adoption.** SL-233 can
end with a locked design, a green suite, and every criterion discharged, and none
of that is evidence that an agent in a real harness *noticed* the guidance,
*adopted* it, or *followed* it. A design run that reached a good result while
scrolling past every fragment produces exactly the same artefacts as one that
read them. Closure is not the measurement — **CHR-049's live exercise against
genuinely installed assets is the actual evidence**, and it has not been run.

That claim is `EX-6`, and it governs the whole kit rather than sitting in a
footnote. Three places it bites:

- **the rubric** scores `complete` — was the substantive result correct — as one
  class of five, with no composite that would let it carry the other four;
- **the bands** make `claimed` cheap to reach, so a run that receipted everything
  and adopted nothing reads as a row of ones rather than as success;
- **the pre-registration** fixes what each result means *before* the run, because
  after the fact any outcome narrates as vindication. DEC-104 admitted no test can
  fail against it. That admission is why pre-registration is necessary here and
  not merely tidy.

## What is in the kit

| artefact | what it is |
|---|---|
| [`pre-registration.md`](pre-registration.md) | **the operative rule set.** What is claimed, what would falsify it, what each result changes — written before any run |
| [`rubric.md`](rubric.md) | the scoring rubric, for a human. The artefact `VH-1` asks the owner to accept |
| [`rubric.toml`](rubric.toml) | the same rubric, in the form the tests score from |
| [`protocol.md`](protocol.md) | the moderator protocol: the standing `context_state` obligation, the induced break, the sibling contrast, and what the kit reports about itself |
| [`collectors.toml`](collectors.toml) | the four signals and their subject routing, the reopening's two arms, the nine-obligation gate record, and the cost / acceptance-basis instruments |
| [`fixtures/`](fixtures/) | a known-good and a known-bad transcript, and the concrete baseline artefact |
| `tests/e2e_design_evaluation.rs` | the assertions. Two, and the names are the contract |

## How to read it

**`pre-registration.md` governs.** `plan.toml`'s `EX-8` and `VA-5` each carry
eleven rounds of append-only amendment from RV-325, so read top-to-bottom they
present withdrawn requirements *before* their withdrawals — nine dead clauses.
The pre-registration is the settled state, and its §8 registers every withdrawal
with the round that killed it. Because `VA-5` is the criterion that *verifies*
this kit, building from the transcript would have failed a check that reads as
satisfied. Where the two differ, the pre-registration wins.

**The load-bearing property is subject routing.** Three of the four signals
concern *delivery* and one concerns *classification*, and no result may be routed
to whichever revision is cheapest. In `collectors.toml` that is expressed as set
disjointness — the delivery signals' admissible evidence keys and the
classification signal's share nothing — so the test checks it rather than a
reader believing it.

**The assertions are two, and they are not a summary of the kit.** The first
checks that the kit really covers five separate classes and that the routing
really holds; the second is the kit's own negative control — a rubric that cannot
score a deliberately bad transcript below a good one is not measuring anything.
Everything else in the kit is judged by reading it.

## What this kit cannot establish

Stated here so it is not quietly assumed away later. The long form is
`pre-registration.md` §9.

- **`N=1`.** One moderated exercise. The reopening's one-shot rule and cross-run
  corroboration are commitments, not controls this exercise collects.
- **The classification signal may not fire at all.** Its firing condition is
  four items, all required, default-deny — satisfied only on positive evidence.
  That is the correct design and it lowers the firing probability. The kit reports
  its **deny rate**; a high one is evidence the exercise cannot support the
  signal, which is a result rather than a gap.
- **Coverage is five of nine.** The classification signal reaches five of the nine
  2a obligations. The four it cannot reach are named in the gate record, and one
  of them — knowledge capture — is unreachable by a deliberate refusal to invent
  a new obligation for the sake of making an old one measurable.
- **Which deliverable is an obligation's real subject is a judgement.** The
  reword arm's truthfulness requirement reduces the room for a subject-redefining
  reword; it does not eliminate it.
- **Two of the four signals need a second treatment actually run.** S1 and S3 are
  comparative, and the first exercise is single-arm, so it collects S2 and, subject
  to default-deny, S4. That is scope, not shortfall.

## The first run is a pilot

The rubric was accepted as *plausibly* useful, to be judged by running it. So the
first exercise measures DEC-104's trade as far as one arm reaches, and measures
**the instrument** the rest of the way — which is what the deny rate, the
inconclusive rate and the coverage fraction are for. **What would earn a second
session with the baseline arm is pre-registered** (`pre-registration.md` §10),
for the same reason everything else here is: afterwards, any result narrates into
either *"clearly worth it"* or *"clearly not"*.
