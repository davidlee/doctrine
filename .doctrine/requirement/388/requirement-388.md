# REQ-388: Every funnel git read is a first-class read verb over the object-db and ref primitives; no funnel read shells raw git. Existing seams are reused or relocated rather than reimplemented (e.g. is_linked_worktree for isolation detection); only genuinely absent reads are newly built.

## Statement

<!-- The sister TOML's `description` field is the primary, normative statement.
     Prose here may elaborate, expand upon, or disambiguate it — never
     duplicate it. -->

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->
