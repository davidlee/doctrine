# REQ-387: One funnel state machine owns the shared transition semantics; each dispatch transport (main-thread, subprocess, confined-orchestrator) projects into that single authority, recording the same transitions whether committing directly or through mediation; per-transport altitude reconciles with REQ-291 and REQ-335.

## Statement

<!-- The sister TOML's `description` field is the primary, normative statement.
     Prose here may elaborate, expand upon, or disambiguate it — never
     duplicate it. -->

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->
