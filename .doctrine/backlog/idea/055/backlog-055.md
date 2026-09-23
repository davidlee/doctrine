# IDE-055: Fold /consult into the authority model

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Idea

Retire `/consult` as a skill and let ADR-023's authority model carry its
meaning: "ask the user, and defer to their answer". Because it's a skill,
agents sometimes insist on invoking it to settle something while already in
conversation with the user, when the escalation is simply asking.

ADR-023's boot section already says "ask the user and defer" instead of naming
the skill. What remains: the routing digest (`install/routing-process.md`,
"`/consult` (don't improvise past it)"), skill cross-references (`/design`,
`/canon`, `reviewing.md`, …), and whether the unsupervised case (a worker with
no user present) needs a residual escalation path. That path would be a
hand-back, not a skill.
