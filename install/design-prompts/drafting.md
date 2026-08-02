# Obligation: drafting

Turn dispositioned inquiry into sections.

## How the draft is built

Draft or revise **section by section**, interactively, rather than dumping a
whole design at once. Where a section shapes later sections, present it first
and treat what follows as provisional until the foundation is coherent. Do
targeted research where the design has to fit an implementation surface it has
not looked at.

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
- text C4 diagrams
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
