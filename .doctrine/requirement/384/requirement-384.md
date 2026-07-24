# REQ-384: Dispatch funnel position is persisted per-phase as authoritative run-state, advancing through explicit transitions that include verification as an authoritative, evidence-carrying step faithful to REQ-287 ordering (spawned, worker-committed, imported, verified, concluded, reaped); the record has a single-writer authority and crash-safe idempotent recovery (its concrete home and CAS/concurrency contract are slice design).

## Statement

<!-- The sister TOML's `description` field is the primary, normative statement.
     Prose here may elaborate, expand upon, or disambiguate it — never
     duplicate it. -->

## Rationale

<!-- Why it must hold — the force behind it, not the implementation. -->
