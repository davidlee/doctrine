The core idea is:

**Treat plans as authoritative current knowledge, not as perfect predictions of the future.** Then make Doctrine good at handling the inevitable discoveries that happen during implementation without collapsing into either “the worker violated the plan” or “the worker can silently rewrite the plan.” 

Today Doctrine is already strong at proving **what was supposed to happen** and **what actually happened**: selectors define scope, Git shows the delta, gates define completion, coverage records evidence, confinement limits authority. The weak spot is the semantic gap between those things. If implementation discovers an unexpected dependency, Doctrine can detect the extra file, but it usually cannot say *why it appeared, whether that was legitimate, who accepted it, and what else that discovery invalidates*. Likewise, a test criterion can evolve, but the system has historically had trouble distinguishing immutable history from what is still actively true. 

RFC-027's proposed model has two parallel flows:

```text
progressive concreteness:
requirement → design intent → obligation → proof → evidence

progressive discovery:
predicted → questioned → discovered → classified → accepted/rejected → reconciled
```

The important architectural rule is that **observation does not confer authority**. A confined worker can say, “the plan appears incomplete; I needed this additional thing, here is the evidence.” It cannot widen its own scope or rewrite the design. The trusted side decides whether the discovery is legitimate, rejects it, revises the plan, or sends the work back to design. 

The payoff is that implementation becomes less brittle without becoming less governed. You can have workers that adapt intelligently to reality while still preserving a deterministic audit trail:

> what we believed → what we discovered → what changed → who authorized it → what proved the result.

That should reduce several expensive failure modes at once: humans getting interrupted for harmless omissions, repeated adversarial plan reviews trying to predict every file touch, unexplained drift discovered only at audit, stale criteria remaining nominally active, and agents repeatedly researching context the project has already learned. 

There's also a deeper traceability payoff. The eventual aspiration is that Doctrine can answer questions such as:

```text
Why did this file change?
What requirement does this test prove?
Is that proof still valid?
What breaks if this design decision changes?
What did execution discover that design didn't know?
```

without pretending it has a perfect semantic model of the entire codebase. Git remains the source of actual change, selectors remain the scope contract, and language-specific testing remains project-owned. Doctrine mostly owns **identity, provenance, authority, lifecycle and evidence**. 

One important outcome of the RFC's investigation is that it **isn't proposing a giant new ontology**. Several candidate abstractions were tested and rejected because Doctrine already had the underlying fact elsewhere. The proposed per-phase “change claim” was killed. An obligation dependency graph was killed. Much of the proposed proof-binding model turned out already to exist in SPEC-002. The studies repeatedly found that the right move was usually to connect or strengthen existing mechanisms rather than invent another record type. 

So the practical pitch is:

**Doctrine is evolving from “prove that execution matched the plan” toward “prove that execution remained governed even when reality forced the plan to evolve.”**

If it works, the result is a system where plans can be strict without being brittle, agents can discover without self-authorizing, and every accepted deviation leaves enough evidence that a later human or agent can reconstruct not just *what happened*, but *why it was allowed to happen*.
