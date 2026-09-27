# RFC-026 E12 — The P10 routing convention's first trial window

Added **2026-09-27**. Evidence for `RFC-026`, split out of the main document
because it carries apparatus as well as findings. Preserved in this directory:
`rater-prompt.txt` (the classification instrument both raters received),
`classification-rater-a.md` and `classification-rater-b.md` (the two independent
classifications, verbatim). The main document's Evidence section carries the
summary and points here.

**The title is the finding, stated plainly:** the notation failed before the
mechanism was exercised. Read the two together — neither is the whole story.

*Gathered 2026-09-27. Instrument: per-finding and per-ledger counts over the three
eligible design-review ledgers — `RV-374`, `RV-382`, `RV-377` — read from
`doctrine review show --json`, plus git-side counts, plus a two-rater
classification. The procedure was fixed on 2026-09-20 by `DEC-276` / `SL-260`
design §9.4, before any eligible slice opened, and is not adjusted here. The window
closed 2026-09-26; nothing below was re-run.*

**Eligibility.** `SL-260` scope §5 took the next three code-changing slices by id
from its landing commit `70bd7d1ff` (2026-09-23). That set is `SL-261` (*Design
adopt verb*), `SL-262` (*Envelope names the next move*) and `SL-263` (*Ambient
memory surfacing for pi and codex*) — all three code-changing, none parked, all
three `done`. **3 of 3.**

**Per severe finding.** 18 severe findings across the three design ledgers: 2
blocker, 16 major.

| ledger | slice | severe | routed in the specified field | routed anywhere | transcribed to a plan criterion |
|---|---|---|---|---|---|
| `RV-374` | SL-261 | 2 | 0 | 2 | 2 |
| `RV-382` | SL-262 | 5 | 0 | 0 | 0 |
| `RV-377` | SL-263 | 11 | 0 | 0 | 0 |
| **total** | | **18** | **0** | **2** | **2** |

"Specified field" is what the convention said at the time and what `DEC-276`
fixed as the extractor: the first token of `finding[].disposition`, i.e. a
`route:<route>` prefix. **The result is 0 of 18.** The two routes that exist were
recovered after the fact, in a prose amendment on `RV-374.md` — F-1 `owner-fix`,
F-2 `review` — added once it was noticed that the dispositions had been written
bare. `RV-382` and `RV-377` recorded no route on any severe finding, in any
location; their `.md` companions have empty briefs.

**Per ledger.** Counts, not shares, and the denominator stated — slice scope
differs, so these are not comparable defect rates against `E11`.

| ledger | rounds | contests | design.md growth during review | later audit ledger | audit findings | slice |
|---|---|---|---|---|---|---|
| `RV-374` | 21 | 1 | +117 / −36 | `RV-379` | 6 (4 minor, 2 nit) | done |
| `RV-382` | 18 | 0 | +210 / −63 | `RV-383` | 5 (4 minor, 1 nit) | done |
| `RV-377` | 68 | 0 | unavailable | `RV-381` | 6 (3 minor, 1 nit, **2 major**) | done |

**The cause is documented contemporaneously, and it is not a routing judgement.**
A friction observation recorded during `SL-261` on 2026-09-24 states the mechanism
plainly:

> The `route:<route>` rule for blocker/major dispositions lives only in
> `reviewing.md`'s tail, below "Recording the lock"; the agent disposed F-1/F-2
> before reading that far, and no verb checks the form. Repair was a prose
> amendment on `RV-374.md` since verified findings cannot be re-disposed.

That is the failure `DEC-103` (*instruction is delivered at the point of effect*)
exists to prevent, in the slice that invoked `DEC-103` to justify hanging the
convention at three surfaces. The obligation was hung at three surfaces and at
none of them *at the moment the responder composes a disposition*. It is also the
first reading on `P10`'s would-kill condition *"the routes need the owner to
interpret"* — not because the routes were ambiguous, but because they were not in
front of the agent that had to choose one. One of `P10`'s three trial questions
is whether agents can choose and execute the right route without repeated human
steering; on this window the answer is not *no*, it is **the question was never
put**.

**No instrument-route obligation was ever exercised.** Both routes that were
recovered are `owner-fix` and `review` — neither is an instrument route, and
neither owes a phase criterion under `P10`. The 2 of 2 transcribed into a plan
criterion are those two. So there is no instance in the window of the mechanism
`P10` is actually aimed at — a `demonstrate`/`probe`/`control` finding carrying
its adversary or named fault into a phase criterion instead of into repair prose —
and no instance either of the repair-text finding it was aimed at reducing. The
window measures the notation and nothing else.

**Confounds, stated rather than left to the reader.**

1. **The recording surface was replaced mid-window.** `SL-268` (*Review ledger
   v2*, closed 2026-09-27) promoted the route to a first-class validated enum —
   `--route review|demonstrate|probe|control|owner-fix` — and made the CLI refuse
   a `route:` prefix in `--disposition`. That is the code-backing `RFC-026`'s
   `P10` follow-up reserved for after a validating trial, on the stated ground
   that it *changes the intervention*; it landed while the trial was live. The
   three ledgers above are all pre-v2 and were collected against a surface the
   binary no longer accepts. Note what this does **not** invalidate: the 0 of 18
   is a fact about dispositions as written and read back verbatim in v2, so the
   count survives the move. What does not survive is the *generality* — the
   convention now ships with a validation the trial's window never had.
2. **`SL-268` edited the convention's own normative owner and left no trace of the
   trial.** `install/design-prompts/reviewing.md`'s heading went from
   *"Routing a severe finding (provisional — RFC-026 P10 trial)"* to
   *"(provisional)"*; `install/review-ledger.md`'s scope line lost its
   *"under the RFC-026 P10 trial"* clause (the design-review-only scope itself
   was kept, so `P10`'s open question about extending the axis is not
   pre-empted); and `SL-268`'s scope, design and notes contain no occurrence of
   the word "trial". The tree already contains the fix for the defect this entry
   reports — `--route` closes exactly the "nothing checks the form" and "not in
   front of the agent" gaps — and nothing joins that fix to the experiment it
   answers.
3. **The at-conclusion capture was missed, and the miss is disclosed, not
   estimated.** `DEC-276` requires the trial owner to record `doctrine review
   status RV-NNN` verbatim into the chore at each eligible pass's conclusion,
   because rounds and contests live in a gitignored runtime baton. No capture was
   made. The values in the table above were read late from batons that happened to
   survive on this machine (`.doctrine/state/review/{374,382,377}/baton.toml`) and
   are reported as late-read, not as the specified measure. Had the batons been
   lost, `DEC-276` directs "unavailable", and that is what the ledger state
   genuinely is. Rater B independently confirms the batons are now the only
   source: no authored ledger carries `[[finding.turn]]` rows.
4. **Shell truncation is excluded as the cause of the missing routes.**
   `DEC-270`'s confound guard asks whether a high miss rate is an eaten-clause
   artefact. All 18 severe responses are present and substantive (238–872
   characters); none is empty or truncated. The routes are absent because they
   were never written.
5. **Design line growth for `RV-377` is unavailable.** `SL-263`'s `design.md`
   first lands in the review's own commit (`ddccae191`), so git bounds the
   document's creation rather than the review window; the pre-review baseline is
   not in the tree. `SL-261` and `SL-262` are measurable and shown above.

**The rater classifications, by two independent raters.** `DEC-276` names two
classification rows and the *drew a related finding* judgement, with `E11`'s
categories reused unchanged and a blind second rater. Two read-only research
agents classified the same 18 findings from the same ledger JSON and design
history, neither shown the other's labels. Per `DEC-276` the disagreements are
reported as disagreements, not reconciled.

| class | rater A | rater B | `E11` (137 findings) |
|---|---|---|---|
| costly surface (A) | 7 (38.9%) | 3 (16.7%) | 19.0% |
| mechanism prediction (B) | 11 (61.1%) | 15 (83.3%) | 59.8% |
| artefact prose (C) | 0 | 0 | 20.4% |
| missing intent (D) | 0 | 0 | 0 |
| other (E) | 0 | 0 | 0 |
| target written to repair an earlier finding (R) | 3 (16.7%) | 3 (16.7%) | 31 of 137 in the non-convergent group |
| repair drew a later finding in the same ledger (L) | 9 (50.0%) | 11 (61.1%) | not measured |
| of the B rows, B2 (only a hostile or edge input exposes it) | 10 of 11 | 14 of 15 | 57% of B |

Disagreement is concentrated exactly where `E11` says its classes mix axes — the
costly-surface / mechanism-prediction boundary. Four of the 18 (22%) differ:
`RV-382` F-3 and F-5, `RV-377` F-5 and F-9. Both raters record the same fork and
split on the rule for it: rater A keyed on the finding's *primary subject* (a
generated, persisted or agent-facing surface → A); rater B keyed on whether the
diagnosis cites a code-level mismatch (mechanism → B). Rater B notes its own
three A rows each cite a code-level mismatch *as well*, and says a written rule
defending B would move them. **Both raters agree on every finding's C/D/E status
— zero artefact prose, zero missing intent, no other.** On R they agree in count
and differ in composition (A: `RV-382` F-5; B: `RV-377` F-9; both: `RV-377`
F-16 and F-22). On L they agree on nine rows and rater B adds `RV-382` F-1 and
F-2.

**The artefact-prose row is the one `P10` was aimed at, and it is zero.** `E11`
put artefact prose at 20.4% of severe findings, and the whole distinguishing
feature of its non-convergent group was 22.8% artefact prose against 0% in the
comparison group. Across three ledgers and 18 severe findings, **both raters
independently found none**, and neither found a missing-intent finding. That is
the direction `P10` wants. On this window it is also **unattributable**: with
zero routed findings there was no instrument obligation to keep out of the
design, so nothing here separates `P10`'s mechanism from the three designs' own
delegation posture, which `E11` already identifies as the variable that carries
the artefact-prose cost. Read as consistent with `P10`, not as evidence for it —
three ledgers, 18 findings, and the counterfactual unobserved.

**Against `P10`'s trial questions.** *Can agents choose and execute the right
route without repeated human steering?* Not answerable from this window — routing
was not performed, so it was neither chosen nor steered. *Does the instrument
actually expose the named defect and support checking its repair?* Not exercised;
no instrument route was taken. *Is there enough reduction in repeated argument to
justify a larger comparison?* The artefact-prose row is 0 against `E11`'s 20.4%,
which is the shape of the answer `P10` wants and is not attributable to `P10`.

**Against the would-kill list.** *A routed finding is admitted unresolved* — none
was routed. *A protected boundary is crossed while one is open* — not applicable.
*A negative control is reported as rejected when it was not* — no control was run.
*The routes need the owner to interpret* — **this window is a positive hit, in its
delivery-placement form.** *The saved rounds reappear as implementation rework or
audit findings* — indeterminate: `RV-381` carries 2 major audit findings against
`SL-263` and the other two audits carry none above minor, but with no routed
findings there is no "saved rounds" against which to read them.

**What this entry does not claim.** It is not a defect rate and not a verdict on
the convention. One ledger in three (the `SL-261` one) shows a responder who
eventually produced the two routes correctly, unprompted, writing them into
prose — which is evidence *for* the mechanism and *against* the delivery, as
`P10` predicted the failure would look. Nothing here separates that from `E11`'s
own confounds, and no causal claim about routing's effect on repair quality is
available from three ledgers with 18 findings and two post-hoc routes.

**Procedure status.** Every row of the fixed procedure is now collected, with two
of them qualified as above: `rounds` and `contests` are late-read because the
at-conclusion capture was missed; `RV-377`'s design line growth is unavailable
because the pre-review baseline is not in git. Nothing in the procedure was
altered after the outcomes were visible.

**Still owed, and owned elsewhere.** `QUE-224` (does the route axis extend beyond
design-review ledgers?) and `QUE-225` (does the route set become a code-backed
closed vocabulary?) are `CHR-077`'s to settle at trial conclusion, and this entry
does not settle them. On `QUE-225` the tree has already moved: `SL-268` shipped
the validated `Route` enum, so the question as written in `SL-260` is now
historical — what remains open is whether that was the right call made at the
right time, which is confound 1 and not a question this entry can answer.
