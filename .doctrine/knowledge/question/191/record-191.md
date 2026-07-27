# QUE-191: V1 delivery boundary for design prompts and skill adapters

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

SL-233 must state whether v1 actually delivers the guidance that adopts the
managed run, rather than merely building a CLI which the existing skill does
not reliably invoke.

Current seams:

- `design` is already a valid stage selector in the SPEC-023 prompt cascade.
- No framework-owned `install/hymns/stage/design.md` snippet currently ships.
- `plugins/doctrine/skills/design/SKILL.md` is the active distributed skill;
  its sibling `SKILL.compact.md` declares itself an inactive experiment.
- Hymns under `install/hymns/**` are embedded into the binary and composed with
  user-owned `.doctrine/hymns/**` snippets.

Options:

1. Deliver the full vertical adoption path: a framework stage/design hymn, a
   thin rewrite of the active plugin skill, prompt rendering that composes the
   stable hymn with the dynamic TurnEnvelope, and install/embed/check tests.
   Leave the explicitly inactive compact experiment outside the contract.
2. Update the skill but hard-code stable guidance inside `doctrine design
   show --format prompt`, bypassing the existing cascade.
3. Deliver only the run engine and defer prompt/skill adoption to a later
   slice.

Option 1 is recommended. It is the minimum boundary that can test adherence
while respecting DEC-064 and SPEC-023; option 2 creates a parallel composition
mechanism, and option 3 cannot demonstrate that agents adopt the new protocol.

## Answer

DEC-077 chooses option 1 with a deliberately small decomposition: one thin
active skill, one invariant stage hymn, and a few obligation-selected Markdown
fragments. V1 does not multiply content by role, model, or user override.
