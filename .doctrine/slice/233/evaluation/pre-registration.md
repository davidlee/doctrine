# Pre-registration — the fragment-versus-step trade

**Written before any run.** This document discharges `SL-233` PHASE-09 `EX-8`.
It is the **operative** rule set for CHR-049's exercise: what is claimed, what
would falsify it, and what each result changes. Nothing here may be chosen after
the run.

## How to read this, and why it exists

`EX-8` and `VA-5` in `plan.toml` each carry **eleven rounds** of amendment from
`RV-325`, appended in place because criteria ids are immutable. Read
top-to-bottom they contain **withdrawn requirements before their withdrawals** —
at least four clauses that no longer hold still read as instructions.

This document is the settled state: the last word on each dimension. Where it
and the raw criterion text differ, **this document governs**, and §8 records
every withdrawal with its round so the difference is auditable rather than
merely asserted.

Authoring the kit from the criterion transcript instead of from this document
would ship requirements that do not exist — and because `VA-5` is the criterion
that *verifies* the kit, that error would be self-concealing. That is the whole
reason this section is first.

## 1. The claim under test

> Delivering craft as **every-turn fragment prose** puts it in front of an agent
> more reliably than delivering it as **discharged runbook steps at an exit**.

This is the ground DEC-104 chose 2b fragment delivery on: *fewer receipts,
better odds the content is actually in front of the agent when it matters.* The
nineteen-step shape would have produced nineteen attestations; fragment delivery
produces few. The trade buys attention with receipts.

DEC-104 admitted no test can fail against it. That admission is what makes a
pre-registration necessary rather than optional: after the fact, any outcome
narrates as vindication.

## 2. Treatments — a concrete baseline, not an unspecified "before"

`VA-4` requires the comparison to name a concrete baseline **artefact**.

| treatment | artefact | availability |
|---|---|---|
| **T-base** — pre-SL-233 skill | design `SKILL.md` at **214 lines / 10,178 bytes**, where the same content is skill prose an agent may scroll past. Recoverable from git before `59c5bdab` | **always** |
| **T-step** — the rejected step shape | the **nineteen-step edge-3 shape** D14 rejected — the treatment the trade was actually made against | **where a fixture permits**; disclosed as opportunistic |
| **T-frag** — as shipped | every-turn 2b fragment delivery | always |

Without T-base or T-step, an observation says only *"the fragment arrived"*,
which nobody disputes. An observation offered against neither is **uncollected**.

## 3. The four signals

Three concern **delivery**. One concerns **classification**. They have different
subjects and therefore different consequences, and keeping them apart is what
stops a result being narrated into whichever revision is cheapest (`VA-5`).

### 3.1 Delivery signals

| # | signal | fires when | 
|---|---|---|
| **S1** | **adherence** | fragment-delivered craft is followed **no more often** than the same content was as a step. The 2b content is doing no work a lens in the incumbent's prose was not already doing, and moving it bought nothing |
| **S2** | **receipt churn** | agents re-request or re-receipt fragments at a rate implying they are **not read on arrival** — delivery without attention, which is the whole thing the trade purchased |
| **S3** | **cost** | per-turn fragment tokens **exceed** the interaction cost of the step shape they replaced. The trade was *fewer receipts*; if it is not also cheaper, its only remaining claim is attention, and S1 must carry it alone |

### 3.2 The classification signal

**S4 — adherence against the stated condition.** The observation is
**mechanical** and contains no inference about a mind:

> the agent discharged a 2a step at a turn where **the step's stated completion
> condition was not satisfied**.

Every earlier formulation of S4 asked a moderator to read intent (*"could not
honestly have completed"*, *"treated a lens as gate-worthy"*) or to derive a
taxonomy from a single run. **All of those are withdrawn** — see §8.

**Collection scope.** S4 is collected **only** over conditions that are
**state-visible** — both terms readable from run state. See §7 for the covered
fraction, which the kit reports rather than assumes.

## 4. Consequences, routed by subject

**No result may be routed to whichever revision is cheapest.** Each signal's
evidence can support only its own consequence.

### 4.1 A delivery signal fires → mechanism-side revision. DEC-104 stands.

Deliver the same lenses **better**; do not reclassify them. None of S1–S3 is
evidence about classification.

**Two candidates, in escalating cost — two, not three:**

1. **Re-emission semantics.** Bound the suppression, or take it off the caller.
   DEC-078's suppression is *caller-declared*: an agent can suppress a fragment
   by claiming a receipt for bytes it never read, and Doctrine cannot tell that
   from a genuine hold. Re-emit unconditionally every *n* turns so a
   claimed-but-unread receipt cannot suppress indefinitely. Cheap, and it
   directly attacks what S2 detects.
2. **Fragment size / split.** Shorten or split the fragment so the lens is not
   buried in bytes an agent skims. Costlier — it reopens what belongs in each
   fragment — but it is the only lever that helps when the bytes genuinely
   arrive and are genuinely not absorbed.

> **A third candidate was withdrawn on inspection** (round 6): *"deliver a
> fragment digest in the envelope so non-attention is visible"*. The digest
> **already** rides every emission as the `name@digest` header, so the
> suggestion was either vacuous or a restatement of (1). Recorded rather than
> deleted, because a mechanism list nobody checked against the mechanism is the
> same failure as a classification rule nobody checked against the code.

**IMP-373 is not the fallback.** `set` mode stays deferred because its render is
unsketched and outside PHASE-08's design gate — not "on DEC-104's merits", which
was withdrawn as false at round 5.

### 4.2 S4 fires → route by the sibling contrast, then reopen

**The within-run sibling control is mandatory.** For every claimed S4
observation the kit records **how the same run treated the sibling obligations
at that edge**:

| contrast | reading | routes to |
|---|---|---|
| premature **across the board** | agent-general adherence | **S1**. Reaches the discriminator not at all |
| premature for **one step beside correct siblings** | step-specific — **the condition was mis-stated** | the reopening (§6) |

**An observation offered with no sibling contrast is rejected as uncollected,
not weighed.** A signal with more than one live explanation routes to none of
them.

Where more than one run is available the same contrast across runs strengthens
it. CHR-049 specifies a single moderated exercise, so the **within-run** control
is the collectible one and cross-run corroboration is **opportunistic**. Stated,
not assumed.

## 5. The firing condition for S4 — four items, all required, default-deny

The anomaly control sits **here**, pre-registered, not in a post-hoc judgement.
S4 fires only when **all** hold:

1. the **within-run sibling control** is satisfied (§4.2);
2. the run crossed **no context boundary** at that edge — neither the deliberate
   break and resume CHR-049's protocol induces, **nor an incidental one**
   (context exhaustion, compaction, or harness-initiated summarisation);
3. the run was **not aborted or restarted** mid-edge;
4. the **moderator protocol recorded the context state** across the window
   containing the discharge.

**Absent (4), condition (2) is UNDECIDED and the observation is UNCOLLECTED, not
weighed.** Default-deny is what makes the condition decidable: it is satisfied
only on **positive evidence**, never by the absence of a mark.

**Cost, disclosed.** Default-deny lowers the probability S4 fires at all, and at
`N=1` a falsifier that cannot fire is a formality. Recording context boundaries
is a **standing moderator obligation with a named field**, not a lucky
observation, so the deny branch should be rare — **and if it is not rare the kit
must report that.** A run whose context state went unrecorded is evidence the
exercise cannot support this signal, and saying so is the honest outcome rather
than absorbing it.

## 6. The reopening — two arms, pre-registered

**There is no third exit.** `stand` was admitted at round 5 and **removed at
round 6**: unconditioned, it let any firing be narrated after the fact as an
anomalous run — route-to-*no*-revision, the mirror of route-to-the-cheapest.
Anomaly now lives entirely in §5.

**Which arm a firing takes is itself pre-registered.** Leaving the selection to
judgement is the routing manoeuvre with one step renamed.

| | rule |
|---|---|
| **default** | **RECLASSIFY.** This is what a firing means unless reword earns its exit |
| **reword must produce an artefact** | the reopening records **candidate step text** naming a completion boundary that is **truthful** for that obligation **and observable in run state** at the moment the new text says the step completes. **No candidate text, no reword** |
| **one-shot per step** | a second firing on the same step after a reword **reclassifies with no further reopening**. The reworded boundary is a prediction; a second firing spends it |

**Disclosed:** at `N=1` CHR-049 cannot itself exercise the one-shot rule. It
binds the step's post-close life and is stated as a **commitment**, not as a
control this exercise collects.

**On the cost gradient.** Reclassification being the default inverts the earlier
asymmetry deliberately. A kit that offers the two arms as **equivalent** has
restored `stand` under the name `reword`.

## 7. Classification is an authoring gate, not a run inference

**Rounds 5–10 all failed the same way.** Classification is a property of an
obligation's **text**, not of a run: no single exercise can establish that no
truthful completion moment exists. Subject *absence* proves a step is not
complete; subject *presence* does **not** prove a truthful completion moment
exists. Building taxonomical truth from one observation was the category error
underneath all of it.

**The gate, and it runs before any exercise:**

1. every 2a step **states a truthful completion condition in its text** —
   satisfying it **discharges** the obligation rather than **starting** it;
2. an obligation whose completion condition **cannot be stated** is **not a
   step** — it is 2b.

This is decidable over the whole set **before any run** and re-runnable by
anyone, which is what makes DEC-104 falsifiable by a check that can actually
fail.

**State-visibility is NOT part of this gate.** It grades a condition's
**evidence**; it never decides classification. Conflating the two contradicts
the round-3 ruling that verification strength is an orthogonal axis — the ruling
that gives mechanically unverifiable but genuinely mandatory human acts
(semantic scope reconciliation, knowledge capture) somewhere honest to sit. A
runbook whose only admissible steps carry mechanical verifiers is a verifier
list, which is DEC-101 and DEC-102 gutted.

**The gate applied — all nine 2a obligations.** Source of truth:
`sketches/thin-adapter.md:1158-1168`. Every one states a condition, so every one
survives as 2a. **Five of nine are state-visible**, and S4 reaches only those.

| # | 2a obligation | state-visible? |
|---|---|---|
| 1 | `explore.scope` | **no** — nothing records a read |
| 2 | `explore.research` | **yes** — `doctrine verify research-current` |
| 3 | `explore.canon` | **no** |
| 4 | `explore.memory` | **no** |
| 5 | `explore.triage` | **yes** — text names its landing site |
| 6 | knowledge capture | **no** — left-hand term is in context, not run state |
| 7 | scope reconciliation | **yes at edge 4** |
| 8 | selector recording | **yes** — `slice selector list` against the section |
| 9 | repeat-pass judgement | **yes** — text names its landing site |

**Row 6 was left alone deliberately.** Making knowledge capture state-visible
needs a durable artefact enumerating *what this inquiry settled* — a new
obligation invented to make an old one measurable. That is the ceremony
objection arriving by the back door, and the trade is **refused**.

**The coverage number is a disclosure, not a footnote.** S4 reaches **five of
nine**. The kit **names the four it cannot reach** and reports the covered
fraction. A kit reporting an adherence rate over a denominator it never covered
fails `VA-5`.

## 8. Withdrawn — register of clauses that no longer hold

Every entry below still appears in `plan.toml`'s `EX-8` or `VA-5` because
criteria are append-only. **None is operative.**

| withdrawn | round | why |
|---|---|---|
| **`stand`** as a third reopening outcome | 6 (F-13) | unconditioned, it let any firing be narrated as anomaly — route-to-no-revision. Anomaly moved into the pre-registered firing condition (§5) |
| *"rewording IS a revision of its boundary contract, which is what makes the two-outcome set answer `VA-4`"* | 7 (F-15) | **false.** Rewording leaves the step 2a on the same edge — it revises the *contract*, not the *placement*, and `VA-4`'s noun is placement. Broadening the noun so the cheap arm qualifies is what `stand` was doing |
| `envelope-visible digest` as a mechanism-side candidate | 6 | **vacuous** — the digest already rides every emission. Candidates are two, not three |
| *"IMP-373 … stays deferred on DEC-104's own merits"* | 6 | the first half stands; **the second is false**. `set` is deferred because its render is unsketched and outside PHASE-08's gate |
| S4 as *"discharged a step it could not honestly have completed"* / *"treated a 2b lens as gate-worthy"* | 4 → 10 | **intent-reading.** An unadjudicable falsifier leaves DEC-104 unfalsifiable just as surely as a delivery proxy does |
| checks **(6)(7)(8)** — enumerate moments not wordings; scoped-and-defeasible reclassification; uninformative-window inconclusive | 10 | withdrawn **with the whole run-side inference**. Subject presence is a precondition, not a completion, so enumerating turns asks the wrong question at each one |
| *"text not producible therefore no such boundary exists"* | 9 (F-16) | **invalid** — non-production can equally be failure to invent |
| `checkable in run state` as a limb of the **classification** test | 11 (F-17) | contradicted the round-3 orthogonality ruling and condemned four of `exploring.toml`'s five shipped steps. **Relocated** to collection scope (§3.2, §7) |
| *"a negative delivery result returns the content to 2a"* / *"does not reopen DEC-104"* | 4 (F-10) | **incoherent** — DEC-104 classifies that content 2b *because* it has no truthful boundary completion, so moving it either falsifies the classification or is inadmissible under it. Exempting DEC-104 from its own falsifier was the original manoeuvre one level up |

**The reword arm deliberately retains `checkable in run state`** (§6), so that a
firing cannot be answered by rewording a condition **out of visibility**. That
retention is not an inconsistency with the row above it: the gate asks what an
obligation's text must state; the reword arm asks what a *repair* must prove.

## 9. What this exercise cannot establish

Stated here so it cannot be quietly assumed away later.

- **`N=1`.** One moderated exercise. The one-shot rule (§6) and cross-run
  corroboration (§4.2) are not exercisable within it.
- **S4 may not fire at all.** Default-deny (§5) is the correct design and it
  lowers the firing probability. The kit reports its **deny rate**; a high one is
  evidence the exercise cannot support S4, and that is a result, not a gap.
- **Coverage is five of nine** (§7). Four obligations are unreachable by S4 by
  construction, and one of those (knowledge capture) is unreachable *by a
  deliberate refusal to invent ceremony*.
- **Outcome alone is not proof.** A successful design run does not demonstrate
  behavioural adoption. That is `EX-6`'s claim and it governs the whole kit.
- **Which deliverable is an obligation's real subject is a judgement.** The
  truthfulness leg of §6 defeats a subject-redefining reword only if that
  judgement holds. Reduced by the artefact requirement, not eliminated.
- **S1 and S3 are not collectible in a single-arm run.** Both are comparative:
  S1 asks whether fragment-delivered craft is followed *no more often than the
  same content was as a step*, and S3 whether per-turn fragment tokens *exceed
  the interaction cost of the shape they replaced*. Each needs a second treatment
  actually **run**, not merely named. CHR-049 commits to one moderated exercise
  and its baseline entry check is conditional — *"preserve or identify the
  pre-SL-233 skill baseline **if** the protocol includes a paired comparison"*.
  So the first exercise collects **S2**, and **S4** subject to default-deny. S1
  and S3 are outside its reach **by construction, not by accident**, and
  reporting them uncollected is the correct result rather than a shortfall to be
  explained away.

## 10. The first run is a pilot, and what would earn a second

The owner accepted this rubric as *plausibly* useful and chose to run it to find
out. That is a complete answer to what `VH-1` asks, and it fixes what the first
exercise is: a **pilot**, which measures DEC-104's trade as far as a single arm
reaches and measures **the instrument** the rest of the way. The deny rate, the
inconclusive rate and the 5/9 coverage fraction are the kit's instrument-quality
outputs, not footnotes on its findings.

**The re-run decision is pre-registered here for the same reason everything else
is.** After the pilot, any result narrates into either *"clearly worth a paired
arm"* or *"clearly not"*. Deciding afterwards is the manoeuvre §4 forbids one
level up.

**The trigger — one condition, and it is a discriminator, not a threshold.**

> A second session with the **T-base** arm is earned when the pilot returns
> **high delivery with low adoption**: fragments emitted and receipted at the
> expected rate, and the rubric's `adopt` class sitting at `claimed` — content
> not reproduced in the agent's prose and not acted on.

That result has **two live explanations and the pilot cannot separate them**:

| explanation | what it would imply |
|---|---|
| fragments are the wrong vehicle for this craft | mechanism-side revision; DEC-104's attention claim is not doing its work |
| agents ignore delivered craft wherever it sits | agent-general — no mechanism revision is indicated, and the trade bought nothing because there was nothing to buy |

**This is §4.2's sibling control one level up.** Within a run, premature discharge
*across the board* reads agent-general and premature discharge of *one step
beside correct siblings* reads step-specific. Across treatments, low adoption
under **both** T-frag and T-base reads agent-general; low adoption under T-frag
**alone** indicts the vehicle. The baseline arm is the only instrument that makes
that cut, and that cut is the only thing it is being bought for.

**A threshold was deliberately not used.** A number — *"`adopt` below 2 on more
than half the fragments"* — would be arbitrary at `N=1` and would invite arguing
the number after the run. The condition above is instead *the pilot returned a
result with two live explanations it cannot separate*, which is the standard §4.2
already applies within a run. It inherits a settled argument rather than opening
a new one.

**What does not earn a second session:**

- **S2 firing.** Receipt churn already routes to re-emission semantics (§4.1
  candidate 1) on within-run evidence. Fix the mechanism; do not buy a session to
  re-confirm it.
- **A high `adopt` score.** DEC-104's attention claim is doing what it said. A
  baseline arm would quantify *how much* better — refinement, not a decision.
- **A mixed `adopt` result** — some fragments reproduced, others not. That is the
  fragment size/split question (§4.1 candidate 2), and it is answerable **within**
  the pilot by asking *which* fragments landed. It escalates to the baseline arm
  only if that within-run analysis is itself uninterpretable — stated as a
  conditional rather than a flat exclusion, because a pattern with no control can
  genuinely fail to resolve.
- **"The results were interesting."** Not a trigger. Stated because it is the one
  that will actually be reached for.

**A T-base arm collects S1 and only a PROXY for S3.** S3's stated comparator is
the *step shape* — the nineteen-step edge-3 shape D14 rejected, which was never
shipped. T-base is skill prose, not steps, so cost measured against it answers
*fragments versus a one-file read*, a different question from the one S3 states.
Collecting it is worth doing; calling it S3 is not. Recorded so the second run's
write-up cannot quietly upgrade it.
