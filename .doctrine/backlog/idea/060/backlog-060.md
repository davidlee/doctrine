# IDE-060: Deterministic inlining of lib: references into skills

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

SL-273 cuts restated guidance out of skills to `lib:` citations (DEC-344,
DEC-345). That trades duplication for a retrieval an agent must choose to make,
and agents do not do anything 100% reliably: some fraction of the time a cited
rule will not be read. SL-273 mitigates with a boot-resident rule that a
specified retrieval is mandatory; it cannot make it certain.

## Idea

Move the body of skills (and similar prompt surfaces) into the reference
library, and have doctrine itself deterministically inline the `lib:`
references a skill names when it delivers the skill — so the guidance arrives
without depending on agent compliance. How skills, reference docs, and hymns
compose under that model is part of the exercise.

## Relations

A consequence of SL-273; likely a descent of RFC-033 (learning surface: corpus
unification and format). Later, not now.
