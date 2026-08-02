# DEC-103: Instruction is delivered at the point of effect

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## The rule

> Instruction — and guardrails governing invariants the system does not itself
> enforce — is delivered at the moment it must take effect.

Prose is not a destination. It is a **failure to locate a delivery moment**, and
where it remains it is labelled as such.

## Why DRY is the wrong model here

DRY earns its keep in code because a reference is resolved perfectly, every
time, by a machine that cannot forget. Agent instruction has neither property.
An obligation stated once in a boot snapshot, a sealed hymn or an early runbook
step is not thereby *in force* three stages later; it is merely *on file*.

So "something with more authority already says this" is not a warrant for
deleting an obligation from where it fires. The question is not *is this stated
somewhere* but **does it arrive when it bites**.

## Three failures, one shape

This rule was not derived in the abstract. [[DEC-101]]'s obligation-runbook
machinery gave PHASE-08 a way to hang obligations on the four forward edges of
the design run, and the triage that followed produced three instances of the same
error within a single sitting:

1. **`SKILL.md:203`** — *governance is not optional when the design makes
   architectural or workflow choices* — was deleted against the shipped runbook
   step `explore.canon`. That step guards **edge 1** and discharges once, at the
   start of the run. The obligation bites while drafting and reviewing, at edges
   3 and 4. Correct under DRY; the obligation is gone at the moment it matters.

2. **`SKILL.md:198`** — *do not hide unresolved assumptions inside polished
   prose* — was deleted against the stage hymn's *"show provenance, unresolved
   branches and blockers rather than a tidy surface"*. That bullet's subject is
   **the inquiry map**, under the heading *Provisional is not evidence*.
   `:198`'s subject is the **design document's prose**. Same injunction, different
   object, and the object is where the work happens.

3. **`SKILL.md:111-113`** — *qualify doc-local acronyms by their containing
   artifact; assume nothing is memorised* — is the one obligation in the triage
   with genuinely no locatable moment, and the agent conducting the triage
   violated it while presenting that triage to the owner, in unexpanded
   `D`-numbers. The item that could only be ambient is the item that did not
   fire. Observed, not hypothesised.

Two of these are near-misses of authority; the third is the null case. Together
they are why this is a rule and not a preference.

## Corollaries

**Multi-hook is a reason to deliver repeatedly, not to stop delivering.** An
obligation that fires at several moments is hung at every one of them. The
genuine objection to a single representative hook — [[DEC-101]]'s false
completeness, where a discharge asserts a process is finished when it is
iterative — is real, but its remedy is *step text that does not overclaim*. A
per-edge discharge that claims only what is true of that edge has no false
completeness to answer for.

**General vigilance is the weakest available form.** Diagram A's own vocabulary
names it without drawing the conclusion: `vigilance — a standing "be on the
lookout" with no trigger point`. "Be alert to X" is worth little. "Apply this
lens to this artefact, now" is worth a great deal. Where an obligation truly has
no moment, it stays prose **and is recorded as unenforced-by-construction** — so
that "we decided to keep this" never reads the same as "we found no mechanism".

## What this does not settle

It does not settle **which** assets may be overridden — that is [[DEC-102]], and
this record narrows rather than contradicts it. An asset can be overridable craft
and still owe a delivery moment; the two rules run on different axes.

It does not deliver the third clause into the shipped authoring rule
(`install/design-prompts/exploring.toml:8-13`, which today has two branches and
no prose branch). That header edit, and its propagation to every runbook authored
after it, is outside PHASE-08's scope and carries a backlog item.
