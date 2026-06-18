# SPEC-020: Estimate hardening — NFR verification and polish

## Context

SL-101–103 implement the functional requirements. This slice hardens the
implementation: proves the NFR guarantees hold, adds defensive tests, and polishes
any rough edges found during integration.

**Depends on all three prior slices** — runs after the full feature is built.

## Scope & Objectives

- **NF-001 (REQ-275) verification** — Audit that no workflow predicate reads
  estimate presence. Prove structurally, not just by a passing run.
- **NF-002 (REQ-276) verification** — Confirm the facet attaches to a second entity
  kind mechanically (no re-modelling). Test the pure/impure boundary.
- **NF-003 (REQ-277) verification** — Verify forward-compatible parsing with
  additive unknown fields.
- **Dogfood** — Add `[estimate]` tables to existing entities (SL-055, others) and
  verify they parse, list, and validate correctly.
- **Edge cases** — Large bounds, zero-width estimates (`lower == upper`), TOML
  formatting edge cases.
- Any residual cleanup from the prior slices.

## Non-Goals

- New features — this slice hardens, it doesn't build.
- Write/edit CLI for estimates.

## Summary

Verification and hardening pass: audit trails, compatibility tests, and real-world
dogfooding. Should leave the estimate facet production-ready.
