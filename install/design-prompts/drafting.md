# Obligation: drafting

Turn dispositioned inquiry into sections.

## How the draft is built

Draft or revise **section by section**, interactively, rather than dumping a
whole design at once. Where a section shapes later sections, present it first
and treat what follows as provisional until the foundation is coherent. Do
targeted research where the design has to fit an implementation surface it has
not looked at.

Write for a cold human reader and the implementor who follows them. Open with a
plain-language account of what changes, why it matters, and where the system
boundary sits. Introduce names before abbreviations or coordinates; identifiers
support the prose, they do not replace it. A reader should not need the review
transcript to reconstruct the current design.

## Make the relationships visible

Use diagrams wherever prose would make a reader simulate structure or time in
their head. A substantial design normally needs at least one; omit them only
when the design is genuinely linear and say why. Prefer Mermaid in authored
Markdown. Choose the smallest view that answers the question:

- a context/container/component diagram for ownership and dependencies
- a sequence diagram for interaction across boundaries
- a state diagram for lifecycle and refusal paths
- a flowchart for branching control or data flow

Give each diagram a one-sentence purpose and explain the non-obvious edges in
prose. Keep names aligned with the surrounding text and implementation surface.
A decorative box inventory is not a model, and a diagram whose reader must
decode unexplained identifiers has not improved legibility.

## The content lens

One lens, not a checklist: hold it over each section as you write it and ask
what the section owes a reader who has to implement from it. Concrete detail
beats hand-wavy prose.

- current behaviour vs target behaviour
- module responsibility boundaries
- imports, coupling and cohesion
- structs, types, interfaces, function signatures
- data structures and algorithms
- example data shapes
- data-flow boundaries
- verification impact
- invariants and boundary conditions
- samples of critical code and protocols
- titles and descriptions of the key test cases
- diagrams of the load-bearing structure, flow, or state
- code-impact summary — paths plus intended changes
- verification alignment — what evidence must change or be added
- impact on the design decisions and the remaining open questions

## Standing lenses

- Do not present "the whole design" as settled before the foundational sections
  and decisions have been validated.
- Do not hide unresolved assumptions inside polished prose; name them.
- Do not confuse detailed design with implementation planning. Plan content
  leaking into a design section is easiest to catch as the section is written.
- A polished full-file rewrite is not progress while the hard design questions
  are still open.
- Governance is not optional background reading when the design makes
  architectural or workflow choices. Apply the ADRs, policies and standards you
  loaded to the choice in front of you, at the moment you make it.
- Keep only current governing meaning in the design. Review chronology,
  superseded wording, finding-by-finding responses, and revision narration live
  in the review ledger; durable rulings live in their normative records. Point
  to those sources when useful, but do not copy their history into the design.

## What the machine will reject

- Draft the required sections. Advancing to reviewing needs them to exist with
  materialisation current — a section that exists only in the conversation does
  not count.
- Declare section bodies through the run, so Doctrine digests the exact bytes. A
  section whose digest nobody computed is refused, not quietly accepted.
- One subject per declaration in a batch. A duplicate subject is refused, and
  that includes declarations merged in from an accepted proposal.
- Keep the authored design and the run in step. If they have diverged, stop:
  ordinary mutation is refused by the authored watermark and `adopt_authored` is
  the only lawful crossing.
- Write the design, not a summary of the process that produced it.
