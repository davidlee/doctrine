# Priority Ordering under Partial Information: a comparison-based approach to prioritisation

Most digital projects involve more possible work than can responsibly be done at once. Traditional prioritisation methods usually try to solve this by assigning scores, running planning poker, arranging sticky notes, or asking stakeholders to rank features from most to least important.


> MoSCoW is weak because it mistakes buckets for priorities. “Must” gets overused, “Should” gets fudged, and the model says little about effort, uncertainty, dependencies, sequencing, or trade-offs. It's situationally useful (e.g. for defining release boundaries or milestones), but poor for deciding what should happen next.

Those methods are useful, but they ask people to make abstract judgements too early.

We tend to find it easier to make relative judgements. It's hard to confidently decide what value to assign to a given feature on an abstract 10 point scale. It's usually much easier to decide: if one of two features must come first, which should it be? This prioritisation model is built around that question.

Rather than asking stakeholders to score every item in a backlog, the system presents carefully chosen comparisons between pieces of work. These may be features, requirements, slices of delivery, risks, research tasks, or technical improvements. Each answer becomes evidence about what matters most.

Engineering then add estimated effort ranges. The system combines stakeholder value, delivery effort, uncertainty, and known dependencies to produce a suggested order of work.

The result is a structured planning conversation, mediated by the software, that aims to provide a maximally useful view of sequencing and scope priority, with the minimum number of decisions.


## How it differs from a normal backlog

A conventional backlog is often a list. Items are placed above or below each other, sometimes with labels like “Must Have”, “Should Have”, or “Could Have”.
This model treats the backlog more like a map.

Some work cannot start until other work is done. Some work does not strictly block anything, but would make more sense earlier. Some work is valuable but expensive. Some work is cheap and unlocks learning. Some work is politically important, but not technically urgent. Some work only becomes important if a risk turns out to be real.

The model keeps those things separate:

* Dependencies describe what must happen before something else can happen.
* Recommended sequencing describes what would probably be sensible to do earlier.
* Stakeholder value describes what matters most to the business, users, or client.
* Engineering estimates describe the expected delivery burden.
* Priority is the combined judgement after all of those are considered.

The distinction is important. A client may care deeply about a large feature, while the team may still recommend doing a smaller enabling slice first. That is not a disagreement about value. It is a difference between value and delivery priority.

## What stakeholders might experience

Stakeholders would not need to learn the internal model. Their interaction can stay simple.

They might be shown two pieces of possible work and asked:

> Which of these would create more business value?

Or:

> If the effort were the same, which would you rather have first?

Or:

> Which of these would be more painful to lose from the next release?

Or:

> Which one better supports the strategic goal we agreed on?

The system should provide enough context to make the choice meaningful: a short description, affected users, known risks, dependencies, assumptions, and any relevant notes from prior discovery.

A comparison might look like this:

> Option A: Improve onboarding so new users can complete setup without support.
> Option B: Add dashboard exports for existing account managers.
>
> If delivery effort were roughly equal, which would matter more for the next release?

The answer does not force the final delivery order. It improves the value model.

Later, once engineering estimates are added, the system might surface a useful tension:

> Stakeholders preferred onboarding over dashboard exports.
>
> Engineering estimates suggest dashboard exports are much smaller and could be delivered first without delaying onboarding.
>
> Recommendation: consider delivering dashboard exports first, while keeping onboarding as the higher-value initiative.

The intended outcome is> better trade-off discussions, earlier.

## The kinds of conversations it surfaces

This model is especially useful because it separates different kinds of disagreement that are often blurred together in agile planning.

> “We value A more, but B should happen first”

A feature may be strategically more important, but another piece of work may be cheaper, lower-risk, or needed to unblock it. This helps avoid the false choice between “business priority” and “technical priority”.

> “This item looks important, but nobody can explain why”

If stakeholders consistently choose other items over it, the item may be stale, political, or poorly understood. That does not mean it should be removed, but it deserves clarification.

> “This low-profile task unlocks a lot of future work”

Some technical or design tasks look unimpressive in isolation. The graph can show that they enable multiple more visible outcomes.

> “The team and the client are using different definitions of priority”

A stakeholder may mean revenue, user impact, deadline risk, brand value, compliance, or executive visibility. Pairwise choices can expose which definition is actually driving decisions.

> “The estimate changed, so the order changed”

A feature that looked like an obvious priority may drop once effort or uncertainty increases. Conversely, a small slice may rise because it offers fast learning or meaningful value.

> “We need a decision, not another workshop”

Instead of asking a group to rank fifty backlog items, the system can ask a small number of high-impact questions. This can make planning sessions more focused and less performative.

## Likely benefits

The main advantage is that it makes prioritisation more concrete. People are often better at comparing two things than assigning abstract scores to many things.

It also creates a clearer audit trail. When priorities change, the team can point to why: a new estimate, a dependency, a stakeholder comparison, a changed assumption, or a resolved risk.

It can reduce loudest-voice planning. Instead of one person arguing for a favourite feature, the model accumulates many small judgements and makes the trade-offs visible.

It gives engineering estimates a healthier role. Estimates do not override stakeholder value, but they do influence delivery order. This helps prevent both extremes: business-only wishlists and engineering-only sequencing.

It also supports progressive refinement. Early in a project, comparisons can be rough. Later, as discovery and estimation improve, the ordering becomes more defensible.

## Likely drawbacks

This model is more structured than a simple backlog. If the team does not maintain the underlying information, the recommendations can become misleading.

It may feel unusual to stakeholders at first. Some clients expect to state priorities directly, not answer comparison prompts. The framing needs to be clear: the comparisons are a way to clarify judgement, not to remove authority.

The model can create a false sense of precision if presented badly. A ranked list is still a recommendation based on incomplete information. It should be treated as decision support, not an algorithmic truth.

It depends on good item descriptions. Pairwise comparison only works if both options are understandable. Poorly written backlog items will produce poor decisions.

There is also a facilitation risk. If every comparison becomes a debate, the process can become slower than traditional planning. The system should ask only the most useful questions and avoid trying to fully sort the entire backlog.

Finally, not all priorities are rational or stable. Sometimes a client has a board commitment, a political constraint, or a market deadline. The model should record those realities rather than pretending they are ordinary feature value.

## The honest promise

This approach will not remove the need for judgement, produce perfect estimates, prevent scope change, or make difficult trade-offs painless.

What it can do is make those trade-offs clearer, earlier, and easier to discuss.

Instead of treating priority as a manually ordered list, it treats priority as the result of several visible forces: value, effort, uncertainty, dependencies, and sequencing. Stakeholders contribute business judgement.

Engineers contribute delivery judgement. The system helps reveal where those judgements agree, where they conflict, and where a decision is needed.
That makes it a better fit for complex work than a flat backlog, while still remaining understandable to people used to agile delivery.

## The hidden benefits

The true value of this approach might be harder to demonstrate but ultimately more meaningful: it models the sequencing of work in a way that's more faithful to reality. It preserves partial information, and allows it to influence sequencing even after other things change. It reduces the number of decisions required to give useful structure to the work, and represents what's important about sequencing in a durable and intelligible way. And it survives the emergence of new requirements or dependencies, rather than deeming their integration into the original plan as ... out of scope.
