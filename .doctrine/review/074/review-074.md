# Review RV-074 — design of SL-097

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Lines of interrogation — the doctrine this Inquisition holds the accused to:

1. **ADR-010 D4** — the `superseded_by` reverse carve-out must be verb-written only, with a `validate` cross-check. Does seeding it in record templates compromise this?
2. **ADR-004 §5** — the three conditions for the reverse edge. Is the verb the sole writer?
3. **PRD-010 §6 matrix** — is the cross-kind gating correctly bounded? Does the RELATION_RULES row over-promise?
4. **`RecordKind` single source of truth** — the design says `is_terminal` delegates to `RecordKind` but no such method exists. Does the hide-set serve?
5. **SL-095 storage migration tension** — governance `supersedes` migrates to `[[relation]]`; records stay typed. Is there a tracked follow-up?

The areas under scrutiny:
- `src/supersede.rs` (new) — the extracted policy with record arms
- `src/main.rs` `run_supersede()` — the generalized cross-kind gating
- `src/relation.rs` RELATION_RULES — the new RECORD Supersedes LifecycleOnly row
- `src/knowledge.rs` RecordKind — the delegated `is_terminal` and `is_record_kind` predicates
- `.doctrine/templates/knowledge-*.toml` ×4 — the new `[relationships]` seed
