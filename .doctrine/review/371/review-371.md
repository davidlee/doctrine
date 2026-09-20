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
