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

## Ownerless restatements left in place (SL-273 restate audit)

Located by SL-273's sweep inventory (`.doctrine/slice/273/inventory.toml`);
each stays until its gap has an owner, then is cut to a `lib:` citation.

- G2 memory mechanics — `plugins/doctrine/skills/record-memory/SKILL.md:23-27` (SL-273 inventory R-510)
- G2 memory mechanics — `plugins/doctrine/skills/record-memory/SKILL.md:104-108` (SL-273 inventory R-513)
- G5 spec authoring — `plugins/doctrine/skills/spec-tech/SKILL.md:33-63` (SL-273 inventory R-521)
- Human-engagement default restated in
  `plugins/doctrine/skills/audit/SKILL.md:173` (R-006) and
  `plugins/doctrine/skills/reconcile/SKILL.md:68` (R-010), rejected under
  SL-273 OQ-3: the text stays. Its owner is project-local
  `governance.md`, which a client cannot reach through
  `doctrine library show`, so no library owner exists (QUE-228). Proposed
  gap G9.

### Noted, not rowed

- `plugins/doctrine/skills/plan/SKILL.md:67-97` — the VT-mandate TOML schema
  restates `install/templates/plan.toml`'s own comment, which no `--help` or
  reference doc owns.
- `plugins/doctrine/skills/elicit/SKILL.md:79-87` — the "Refresh and stop"
  footer vocabulary, which is skill-local and unowned.
