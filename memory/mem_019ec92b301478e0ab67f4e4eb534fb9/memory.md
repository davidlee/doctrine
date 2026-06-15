# Doctrine revisions

Revisions (REV kind, ADR-013) are the change-axis for governance documents —
they track what changed, why, and what the before/after states are.

## When to use a revision

When changing a specification, policy, or standard that is already in force,
create a revision rather than editing in place. The revision records:
- What entity is being changed.
- The nature of the change (introduce, modify, retire requirements).
- The disposition (proposed, accepted, rejected).
- Links to the slices that implement the change.

## CLI

- `doctrine revision new --target <SPEC-ID> --title "..."` — scaffold a revision.
- `doctrine revision show <ID>` — full content.
- `doctrine revision list` — list revisions.
- `doctrine revision transition <ID> --to <status>` — advance the lifecycle.

Revisions live under `.doctrine/revision/nnn/`. They are the governance
counterpart to slices — slices change code; revisions change specs and
governance.

See [[signpost.doctrine.specs]] for the spec hierarchy,
[[signpost.doctrine.policies-standards]] for governance standing rules,
and [[signpost.doctrine.adrs]] for architectural decisions.
