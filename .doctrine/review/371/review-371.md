# Review RV-371 — design of SL-260

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Subject: `.doctrine/slice/260/design.md` (804 lines, nine sections) — the design
for a **design-review finding routing convention and trial**, descending from
`RFC-026` proposal `P10`. SL-260 stands the convention up and fixes the trial's
rules; it does not run the trial.

The design is **text-only**: its whole delta is prose in two shipped files
(§5.2 carries the proposed text verbatim). That shapes what a finding can be.
There is no implementation to predict wrongly — so the usual richest seam,
mechanism prediction, is mostly absent. What is left is the seam this design
*does* load-bear on: **normative claims about what the review protocol means**,
and whether the convention it proposes is self-consistent, operable by an agent
who has only the proposed text, and honest about what it does not solve.

### Lines of attack

1. **§3.2's tripwire framing.** Four standards are declared not-binding, all for
   one stated reason — they scope to `src/` or to shipped mechanism. One sentence
   carries all four. Attack the generalisation: is the shared reason actually
   shared, or does it fit three and get stretched over the fourth? A text-only
   delta that edits *shipped* files is exactly the case where "scopes to shipped
   mechanism" may bind after all.

2. **R5 in §8 — the twin-arm failure mode.** A repair discharges the arm the
   finding named and leaves its twin; observed four times in one review. The
   mitigation is explicitly partial: the owner-fix sweep clause covers the
   duplicate-accounts case and the other four routes do not. A `demonstrate` can
   still discharge one arm and read as complete. The design names this rather
   than claiming it solved. Probe whether the residual is as bounded as §8 says,
   and whether any *cheaper* mitigation was dismissed too quickly in §7.3.

3. **The closed route set.** Five routes (`review`, `demonstrate`, `probe`,
   `control`, `owner-fix`). Is it exhaustive over the finding population `E11`
   actually classified? Find a severe finding shape that fits none of the five,
   or that fits two with no tie-break — an author at disposition must pick
   exactly one, so an ambiguous pair is an operational defect, not a taxonomy
   nit.

4. **The gate seam (§5.1, §5.4).** The design gate may close with a routed
   obligation outstanding, carried to first-phase criteria. §5.1's flowchart
   calls out that only two edges have mechanism behind them. Attack the other
   edges: what actually prevents an obligation from evaporating between design
   close and phase planning? "Convention says so" is an answer, but the design
   should say that is what it is.

5. **Trial validity as stated, not as wished.** §1 concedes a combined
   intervention with `P2` and no causal separation, three ledgers as a practical
   minimum, no pass mark. Check §9.4's counting method against that concession —
   a counting method precise enough to look like measurement, sitting on a trial
   that disclaims inference, is a place where the design could mislead its own
   future reader.

### Invariants to hold it to

- Normative statements have **one record**, included by reference, never
  retyped (`RFC-026` `P2`). A design that proposes this convention while
  violating it is fair game.
- Doc-local ids (`R5`, `OQ-1`, `D1`) mean nothing outside the artefact; durable
  ids (`SL-`, `REQ-`, `ADR-`) are identity (STD-002).
- No silent skip — a degraded read is disclosed (STD-003).
- The design is not higher authority than `/canon`; the plan is not higher
  authority than the design.

### Out of scope for this review

Whether `RFC-026`'s `E11` classification is sound. Take `E11` as given; review
the design built on it.

## Responder corrections (post-disposition, pre-verify)

A disposition cannot be amended (no amend verb), so a defect found in one after
the fact is recorded here rather than silently left.

- **F-10's response mis-cites a risk id.** It says the `F-9` worked example is
  cited "in section 5.4 and in R8". The correct id is **`R4`** — *the adversary
  clause is silently eaten by the shell*. `R8` is the trial-population risk and
  is unrelated. The design itself is correct: `design.md` §8 `R4` carries the
  `F-9` citation and §5.4 carries the worked example. Only the ledger response's
  pointer is wrong.

Found by the responder's own read-back-and-self-attack pass, not by the raiser.
It is the second instance in this ledger of the failure mode `R4` describes —
the first being `F-9` itself — and both argue the same way: the recording site
for a review's durable reasoning has no correction path short of a whole round.


## Synthesis

**Closure story.** Ten findings raised, one withdrawn for shell damage and
re-raised, nine disposed. Every one was confirmed against the code or the cited
file; none was contested on its merits by the responder. The design grew from
804 to roughly 1130 lines under the pass — the largest single contributor being
`F-8`, which converted a five-column route extractor presented as *the counting
method* into §9.4's full collection and classification procedure.

Two findings were **contested by the raiser after disposition** and re-disposed:

- **`F-6`** — the first repair was itself defective. Its four-step precedence
  turned steps 2 and 3 on *instrument route*, a term this design defines as
  exactly `demonstrate`, `probe` and `control`, which left the pair
  `review` + `owner-fix` — `F-6`'s own second worked example — unresolved by all
  three steps while step 3 nonetheless enumerated `owner-fix` first. Step 2 now
  reads *prefer any route other than `review`*; steps 2 and 3 are together total
  over the five. `DEC-275` carries the amendment.
- **`F-2`** — the substance stood; the completeness claim did not. Its response
  said the transcription overclaim appeared in four places and all four were
  fixed. It appeared in five: `DEC-271`'s own `consequences` still named
  transcription as having teeth at the slice close, and `DEC-271`'s **title**
  still read *close-gate teeth are audit-grade* — the same withdrawn claim in the
  one field an append-style amendment cannot reach.

**What this ledger demonstrates about its own subject.** `SL-260` exists to route
severe findings to instruments because prose repair generates findings against
repair text. This pass is a live instance of the mechanism it is designing for.
`R5` — *a repair discharges the arm the finding named and leaves its twin* —
fired **three times inside the repair round**: on `F-6` (the twin arm of a
compound finding, in the repair that introduced the twin-arm mitigation), on
`F-2` (an incomplete class sweep, in a sweep whose clause `owner-fix` exists to
make explicit), and on `DEC-271`'s title. `R4` — *the adversary clause is
silently eaten by the shell* — fired twice, on `F-9` and on `F-10`'s own
response. None of the five was found by the raiser's original pass; all were
found by a fresh critical re-read afterwards. That is evidence for `P10`'s
premise and against any reading that a single adversarial pass converges.

**Standing risks.** `R5`'s residual is unchanged and is the one this ledger
keeps illustrating: the sibling-split rule fires only when somebody notices the
twin, and in all three instances here nobody did at the time. `R6` is materially
stronger after `F-4` — the plan-side pointer means the transcription instruction
now reaches a planner — but the raiser side stays unguarded, which `F-2`
established is not fixable without a `src/` change.

**Tradeoffs consciously accepted.** The no-`src/` tripwire was held throughout
and cost real capability: the `--response` quoting hazard takes a content rule
rather than a transport fix, the counting joins are manual, and the route set
stays prose rather than a code-backed closed vocabulary. Each is recorded with
its post-trial reopening path (`Q4`) rather than hidden. `DEC-271`'s slug still
carries the withdrawn claim; the id is identity and the slug is not
authoritative (`STD-002`), so it was left rather than renamed.

**Raised out of this pass, not repaired in it.** `ISS-472` (two shipped doc
comments deny that cargo tracks embedded assets — `R7`'s opposite claim is the
correct one) and `ISS-473` (`knowledge edit`'s list flags silently sever any item
containing a comma, which is how `CON-006` acquired three severed rows).
