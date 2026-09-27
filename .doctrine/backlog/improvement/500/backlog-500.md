# IMP-500: Library owners for skill-only rules

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Context

SL-273's library ownership audit (research.md, "Gaps") found rules taught only
inside skills, with no library doc to own them. SL-273 authors G7 (the
publication model and the `lib:` form — DEC-342); the rest are deferred here:

- G1 — PUSH/PULL tiering and boot anatomy
- G2 — memory mechanics
- G3 — backlog mechanics
- G4 — knowledge gating
- G5 — spec authoring
- G6 — worktree isolation
- G8 — `doctrine.toml` schema

## Why it matters

Until an owner exists, a skill restating one of these cannot be cut to a
citation (ADR-005 restate-line rule); SL-273's restate audit leaves such
concept tables in place and notes each one.

## Shape

Per gap: decide the owning library doc (new or existing), author it, cut the
skill restatements to `lib:` citations. Split into separate items if they are
taken up independently.
