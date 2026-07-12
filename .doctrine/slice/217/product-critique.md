The maths is mostly deciding **what the product is allowed to claim, what it asks humans, and whose judgement gets to dominate**.

The core product-facing choices are these.

## 1. What does “stable” mean?

D5 defines stability only for `value_dim` ordering, not for the full delivery score. In product terms, doctrine can say:

> “Given current value evidence and costs, A’s value-for-effort robustly exceeds B’s.”

It cannot say:

> “A will definitely be recommended before B.”

Risk, leverage, burndown, dependency gating, and sequencing may still change the final recommendation. 

That is broadly consistent with doctrine’s separation-of-forces model. The danger is surface language: a command called `elicit` that reports “stable” will naturally be read as “priority is settled.” The design carefully qualifies this as `value_dim` stability, but that distinction is mathematically clean and product-hostile. Users care about “what should we do?”, not which component of the score is locally invariant.

**Product trade-off:** tractable and honest internals versus a weaker answer than the user probably thinks they are buying.

I would make the surface lead with:

> “Value-for-effort is settled among these items; final delivery order may still differ.”

## 2. How much of the backlog matters now?

The top-K design means doctrine deliberately ignores uncertainty outside the current pull horizon. K defaults to eight and is configurable. 

That shapes doctrine into a **rolling decision-support system**, not a master backlog-ranking system. It asks enough questions to make near-term work defensible and preserves ignorance elsewhere.

This is strongly with the grain of doctrine:

* partial information is legitimate;
* derive only what is currently actionable;
* avoid forcing premature total order;
* re-evaluate when inputs change.

The unresolved product detail is whether “top K” means the current membership is trustworthy. As designed, it mainly stabilises relations among the current top-K items; something just below the line may still plausibly displace one of them. That makes the feature better described as **local refinement of the current frontier** than “stabilise the next eight.”

## 3. What counts as a useful question?

The yield maths prefers questions that settle the largest number of currently relevant comparisons, weighted toward the top of the frontier. 

Product-wise, this means doctrine optimises for:

> “Which answer would most reduce ambiguity in the recommendations we are about to use?”

not:

> “Which question is easiest, most natural, most politically useful, or most likely to expose a mistaken assumption?”

That is why the curator layer exists. The engine chooses mathematically productive questions; an agent chooses humanly sensible ones.

This is consistent with doctrine’s usual split between deterministic mechanism and semantic agent judgement. It is one of the strongest choices in the design.

The caveat is that the ranking formula is product policy disguised as numeric tuning:

```text
yield × frontier impact × human-confirmation boost
```

Changing those constants changes whether doctrine behaves like:

* an information-maximiser;
* a near-term delivery optimiser;
* or a human-governance checker.

That deserves product-level names or presets, not merely implementation-owned numbers.

## 4. Is abstaining a valid answer?

Humans may answer `incomparable`, but the guaranteed-yield score excludes that outcome because including it would make every question’s guaranteed yield zero. 

This is mathematically practical, but the product claim changes subtly:

It is not really:

> “This question guarantees progress however you answer.”

It is:

> “This question guarantees progress if you provide an ordering or equality judgement.”

That matters because “these are not meaningfully comparable” is not an edge case; it may be a sign the product has offered a poor pair, the wrong audience, or incompatible granularity.

This runs slightly against doctrine’s “degrade, don’t falsify” posture. The evidence is not falsified—the zero-yield abstention is disclosed—but the headline metric systematically ignores one legitimate user outcome.

I would avoid the bare term **guaranteed yield** in user-facing output. “Order-bearing yield floor” is ugly but truthful; the human surface could simply say “expected structural usefulness” and expose the per-answer breakdown.

## 5. Are authored numbers more authoritative than accumulated evidence?

Yes. Authored values are hard anchors; comparisons are evidence interpreted around them. When they conflict, the anchor wins and comparison structure is quarantined. The queue then promotes anchor review because one stale anchor can sterilise a large amount of evidence. 

Product-wise, this preserves an explicit operator override:

> “I know the value is 5.0; treat that as policy.”

That is consistent with doctrine’s general distinction between authored truth and derived state, and with its preference for explicit correction rather than silent statistical reconciliation.

But it creates a conspicuous tension:

* dozens of stakeholder comparisons remain “evidence”;
* one manually entered float is “truth.”

That is probably the biggest place this subsystem runs against its own product pitch. The pitch says absolute scores are weak and relative judgements are more reliable; the implementation still gives an absolute score constitutional supremacy.

There are good governance reasons for that, but the product needs to frame `value set` as a **pin or policy override**, not merely another way to enter value. Otherwise users will not understand why one old number disabled half the elicitation ledger.

## 6. How much authority do agents have?

Agent and human comparisons currently constrain inference equally. The queue merely boosts questions in areas calibrated only by agents, nudging humans to inspect them. 

Product-wise, doctrine is saying:

> “Agent judgement is operationally valid unless and until a human chooses to review it.”

This fits the operator-and-agent product you are building: agents are instruments acting within a governed corpus, not untrusted external advisers.

It is nevertheless against the grain of doctrine’s broader evidence discipline in one respect. Doctrine normally cares about provenance, confidence, and what has actually been demonstrated. Here provenance is visible, but **epistemic authority is binary and equal**. Fourteen cheap agent comparisons can close the same questions as fourteen stakeholder decisions.

The design explicitly defers the demotion knob until before stakeholder-facing surfaces. That is probably acceptable for now, but product-wise it means Phase C is an **operator tool**, not yet a trustworthy stakeholder elicitation system.

## 7. What happens when evidence conflicts?

Contradictory comparison structures are quarantined rather than silently repaired. Phase C may recommend reviewing the anchor or retiring rows. 

The product choice is:

> Prefer visible loss of inferential power over an invisible guess about which testimony was wrong.

This is highly with doctrine’s grain:

* evidence remains recorded;
* derived state may degrade;
* contradictions become actionable findings;
* no hidden mutation;
* correction is explicit through supersession, tombstone, or anchor edit.

The price is operational friction. A user can answer more questions and make the system know less, because a new contradiction may quarantine previously useful evidence. The signed-yield model explicitly admits this. 

That is philosophically consistent with doctrine, but the UX must explain it as:

> “This answer exposed an unresolved disagreement,”

not:

> “Your answer reduced the model’s yield by 4.”

## 8. Is the queue a workflow or merely a report?

The design makes `compare elicit` read-only. It emits recommended questions and exact follow-up commands; it does not hold an interactive session or persistent queue state. 

This is very much with doctrine’s grain:

* derived state is not persisted;
* CLI is source of truth;
* commands compose with agents;
* no opaque stateful wizard;
* the ledger itself is the durable state.

But it runs somewhat against the product pitch of a “structured planning conversation.” In this phase, it is really a **question recommendation API**, not a conversation surface.

That is probably the right sequencing decision. Just do not evaluate Phase C as though it has already delivered the stakeholder experience.

## The conspicuous tensions

The ones I would elevate to explicit product decisions are:

1. **Absolute anchors outrank comparative evidence.**
   Defensible as operator policy, but contrary to the motivating claim that absolute values are the weakest input.

2. **“Stable” is component-local, not delivery-order stable.**
   Mathematically disciplined, but easy for users to overread.

3. **Agent evidence has full inferential authority.**
   Fits agent-operated doctrine, but not yet stakeholder-grade governance.

4. **Guaranteed yield excludes a legitimate answer.**
   Practical algorithmically, but the product terminology should not imply unconditional progress.

5. **Top-K stability is local refinement, not necessarily stable frontier membership.**
   With the grain of partial planning, but weaker than “the next K are settled.”

6. **A cardinal score is manufactured from ordinal evidence and then consumed everywhere.**
   Phase B labels provenance honestly, but gauge/projected magnitudes still affect `value_dim` and burndown as real numbers.  This is the deepest conceptual compromise in the whole model: doctrine refuses to invent structural facts, yet it necessarily invents cardinal spacing because the existing score engine demands it.

## Overall

Most decisions are strongly characteristic of doctrine: preserve evidence, derive rather than author, make uncertainty explicit, quarantine contradictions, keep pure deterministic machinery below semantic agent curation, and avoid persistent derived workflow state.

The two decisions that feel most against the grain are:

> **authored scalar anchors having greater authority than comparative evidence**, and
> **agent evidence being fully authoritative despite only provenance—not trust—being modelled.**

Both are conscious transitional policies rather than accidental design flaws. They should be presented that way in the product, because otherwise the maths makes them look inevitable when they are actually governance choices.
