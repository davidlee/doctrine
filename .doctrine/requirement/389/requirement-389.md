# REQ-389: The funnel is working-tree-free and every coord-tree operation is bounded by a no-pathless-commit / safe-commit guard, so the ISS-234 reverse-diff cannot commit mass reversions; read verbs keep funnel reads from observing the reverse-diff, the guard closes the write side, and ISS-234 is absorbed only when both hold.

## Statement

<!-- The sister TOML's `description` field is the primary, normative statement.
     Prose here may elaborate, expand upon, or disambiguate it — never
     duplicate it. -->

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->
