Yes. I think this is closer to the missing centre of Doctrine than any individual gap around planning, verification, or orchestration.

Doctrine already has good representations of **things**—requirements, decisions, phases, selectors, evidence, review findings—and increasingly good machinery for their lifecycle. What it does not yet have is a satisfactory representation of **a normative commitment travelling through the development process**.

That is the thing currently smeared across design prose, plan prose, criteria, selectors, implementation notes, review responses, and finally evidence.

### The useful decomposition

I would distinguish three concepts, without immediately making them three entity types:

**Claim** — “I assert/propose that X should/must/is true.”
Has provenance and basis. May be agent-proposed, user-directed, inferred from evidence, contested, superseded, rejected.

**Commitment** — a claim which has become governing intent.
This is what implementation is *not allowed to accidentally change*.

**Obligation** — an execution-scoped responsibility to make or keep some commitments true.
This is what belongs in phases, becomes blocked/actionable, and is eventually discharged by evidence.

So the overall pipeline becomes something like:

```text
prose / research / evidence
        ↓ extraction
claims / proposals
        ↓ adjudication
governing commitments
        ↓ planning
phase obligations
        ↓ execution
realisation + discoveries + challenges
        ↓ verification
evidence
        ↓
discharged / revised / contested commitments
```

The important part is that **planning does not copy the design**. It projects governing commitments into executable obligations.

That is almost exactly the gap current `SPEC-031` exposes. It now explicitly says that phase `EX` criteria are the obligations, `VT`/`VA`/`VH` are how they are proved, and that the verification→exit binding is still a missing authored fact.  But immediately upstream, there is still no correspondingly crisp design-intent→`EX` seam. The current runtime model remains basically `{id,text}` for an exit criterion.

So I would not invent `OBL` or replace `EX`. **`EX` is already the execution obligation. The missing object is what it owes.**

### There are really two graphs

Your “graph of graph edges” phrasing is significant.

There is a **semantic graph**:

```text
REQ
 └─ interpreted/constrained by → DEC
                              └─ governing commitment C17
```

and independently an **actionability graph**:

```text
EX-3 ─needs→ EX-7
  │
  └─ discharges → C17
```

Those are different relations answering different questions.

The semantic graph says **what must remain true**.

The actionability graph says **what work can happen when**.

Trying to use one graph for both is where I think this gets ugly very quickly. A change in implementation sequencing should not rewrite design semantics; a revised design commitment should be able to invalidate or reshape obligations without pretending that this is merely rescheduling.

Phases then become largely **collections/projections over obligations**, not the identity of those obligations. Moving work from PHASE-03 to PHASE-05 need not change what is owed. SPEC-031's planned criterion relocation semantics are already moving in exactly this direction.

### The trick is to type the governance, not all the semantics

This is where I would resist the tempting three-year ontology project.

I would **not** start with:

```text
subject
predicate
object
modality
strength
confidence
scope
precondition
postcondition
...
```

and try to turn design prose into some universal executable predicate language.

RFC-027's Stage 0 result is a very useful warning here: its apparently sensible five-field semantic change claim was tested and **failed to own a sufficiently distinct fact to justify itself**.  That is precisely the immune response Doctrine should retain.

Instead, make the semantic payload initially quite weak:

```text
stable identity
statement
canonical subjects/references where known
provenance
basis/evidence
```

and put the strong typing around the things Doctrine actually needs to govern:

```text
who proposed it
who may admit it
whether it currently governs
what it supersedes/refines/splits
what obligations discharge it
whether it is contested
what evidence bears on it
what contract revision it belongs to
```

Then add typed payloads only when a consumer earns them.

RFC-020 is a very good existence proof for this architecture. Value and estimate claims gained enormous leverage not because every judgement became elaborately typed, but because the generic mechanics became solid: provenance, derived authority, append-only supersession, conflict rather than silent winning, evidence tiers, and a derived operative view. 

I would steal those **ledger semantics**, not try to generalise `comparison::claims` itself.

### And do not make one gigantic lifecycle enum

There are several independent axes:

```text
epistemic:     proposed / accepted / rejected / contested
lineage:       governing / superseded / withdrawn
execution:     unplanned / assigned / blocked / actionable / complete
proof:         unbound / bound / evidenced / verified / stale
```

Keep them orthogonal.

The managed design run has already learned this lesson. `InquiryNode` separately carries provenance, lifecycle, disposition, parentage and `needs`; blocking is derived rather than stored, and resolution requires an explicit semantic disposition.  That is a much healthier pattern than `status = "accepted-planned-blocked-contested"`.

### The adaptation envelope then becomes surprisingly simple

I think RFC-026's G15 is central: **intent and implementation approach are currently conflated**, leaving execution no explicit boundary between “adapt this freely” and “changing this changes the design.” 

A governing commitment needs some trusted-side statement of its adaptation posture. Conceptually:

```text
reserved
    implementation may challenge it, never silently change it

bounded
    implementation may refine/reify it inside stated constraints

delegated
    implementation owns the realisation choice

observational
    implementation may append discoveries/evidence freely
```

I would be careful about storing anything called `authority` on a worker-writable record—the RFC-020 lesson applies. Authority should come from **who admitted the governing record and through which operation**, not from the claimant saying “trust me”.

That gives the worker a useful freedom:

> I may change the realisation until I cross a governing commitment boundary.

When it does cross one, that is not automatically a violation. It emits a **challenge/revision proposal**, with evidence.

This is the other half of RFC-027's strongest idea: predictions are authoritative but provisional; observations do not self-promote into authored truth. 

### I think the implementation unit should be a contract revision

At dispatch, a phase receives something like:

```text
governing contract revision R17

commitments:
  C12
  C17
  C31

obligations:
  EX-2 → [C12]
  EX-5 → [C17, C31]

dependencies:
  EX-5 needs EX-2

adaptation envelope:
  C12 reserved
  C17 bounded
  C31 delegated
```

Execution can produce two very different diffs:

```text
implementation diff
    code/tests/files changed

contract diff
    C17 challenged
    C42 proposed
    EX-5 split
    dependency EX-7 needs EX-3 discovered
```

The first is ordinary implementation.

The second is **semantic drift made explicit**.

The control plane can admit some of that into R18, reject some, or escalate some. Already-completed work is then evaluated against the new revision rather than magically rewritten. That is essentially RFC-027 P11/H11, but with a concrete thing being revised: the governing commitment/obligation contract. 

And it makes audit dramatically more interesting:

> “The implementation differed from the original plan in nine files”
> becomes
> “Two implementation discoveries caused one accepted commitment refinement and one new obligation; all other governing commitments survived unchanged.”

That is a much more Doctrine-shaped account of what happened.

### Where I would start

Not by designing the generic schema.

I would build one deliberately narrow vertical experiment:

1. During managed design, allow the agent to **extract candidate commitments** from accepted prose/decisions. Extraction has provenance and is never self-authorising.
2. Have an explicit trusted admission step produce the governing set.
3. Give each `EX` criterion a link to the commitment(s) it discharges.
4. Implement the already-planned `VT → EX` binding from SPEC-031.
5. Give an implementer exactly two semantic operations: **challenge commitment** and **propose refinement**, both append-only and evidence-bearing.
6. At reconciliation, render a semantic diff between the dispatched and final governing sets.

No generic predicate system. No new actionability engine. No universal change record. No attempt to structure every sentence.

Then measure whether it catches the expensive failures you actually care about:

* design intent that never reaches a plan;
* plan obligations with no governing reason;
* implementation silently changing intent;
* reviewers rediscovering why some constraint exists;
* corrections being smeared across several prose copies;
* needless human escalations for adaptations that were actually within the envelope.

RFC-023 and RFC-026 together suggest that the current cost is heavily concentrated exactly there: executable structure stops short of semantics, while prose review repeatedly pays to recover and re-establish those semantics.  

So my current model would be:

> **Doctrine's fundamental governed object is not a document, plan, phase, or change. It is a revisable commitment, justified by claims and evidence, projected into executable obligations, and ultimately discharged or revised by observed reality.**

Documents are authoring surfaces. Plans are projections. Phases are scheduling envelopes. Tests are proof mechanisms. Git is evidence.

The commitment is the thread that ought to survive all of them.
