# Replace CON (constraint) with INV (invariant): rename + waived→relaxed semantics

## Context

Carved out of **SL-159** (epistemic kind catalog), which originally bundled three
RFC-009 catalog changes: add EVD, add HYP, and replace CON→INV. The two additions
(EVD/HYP) are well-understood and proceed in SL-159. **CON→INV is split out here**
because the semantic shape is *not* settled — renaming "constraint (a boundary that
must not be crossed)" to "invariant (a property that must hold)" drags a chain of
vocabulary decisions (`waived → relaxed` status, `waiver_* → relaxation_*` facet,
`ConstraintSource → InvariantSource`) whose framing still reads as jank and warrants
its own design pass rather than blocking the additions.

This is a **destructive rename of a shipped kind** — tree dir, reservation namespace,
prefix, seed record, templates, and every literal `"CON"`/`Constraint` site move
together or integrity breaks. It rides the same ~17 hardcoded record-prefix touch
sites SL-159 edits (see `mem.pattern.doctrine.record-kind-touch-sites`), so it
**sequences after SL-159** (`after SL-159`) — serial, no parallel edits to the same
lines. EVD/HYP target the `RECORD` family which still includes CON in the interim;
when this slice lands, the rename carries through.

## Scope & Objectives

1. **CON → INV rename** — `RecordKind::Constraint → Invariant`; prefix `CON → INV`
   (CON prefix retired, not recycled); engine `CONSTRAINT_KIND → INVARIANT_KIND`
   (dir `…/invariant`); `integrity::KINDS` row rename + pin update.
2. **Status `waived → relaxed`** — INV vocab `active, relaxed, superseded, retired`
   (was `…waived…`). **Open design question** (the jank): is `relaxed` the right
   frame for "an invariant that no longer must hold," or does an invariant model
   violation/exception differently than a constraint waiver? Resolve in design.
3. **Facet rename** — `ConstraintFacet → InvariantFacet`: `waiver_reason/waived_by/
   waived_on → relaxation_reason/relaxed_by/relaxed_on`; `ConstraintSource →
   InvariantSource` (variants kept: canon, adr, external, technical, legal,
   compatibility, operator); `statement, source, applies_to` unchanged.
4. **Seed CON-001 → INV-001 (recreate, not migrate)** — delete the `constraint/`
   tree, re-mint INV-001 fresh from the new template; re-point two **live**
   citations (`adr-017.md`, `knowledge/question/001/record-001.md`). Historical /
   closed-context prose (`slice/097`, `rfc/003`, `rfc/008`, `rfc/009`) left as
   past-state narrative — no corpus-wide dangler gate fires (`scan_danglers` only
   on explicit `reseat`).
5. **Touch-site rename across the ~17 hardcoded prefix sites** — `kinds::RECORD`
   `CON → INV`; `catalog/scan.rs:62` dispatch arm literal (**panic-grade**:
   `debug_assert!(false)` fallthrough on an unrouted KINDS prefix);
   `catalog/test_helpers.rs`, `dep_seq.rs`, `priority/partition.rs`, `search.rs`,
   `tag.rs`, `integrity.rs:817`, `relation.rs` (rule lists + test pins),
   `relation_graph.rs`, `supersede.rs` + `commands/supersede.rs`; templates
   (`knowledge-constraint.toml → knowledge-invariant.toml`); docs (`using-doctrine.md`,
   `glossary.md`); shipped memory; e2e goldens (`e2e_knowledge_cli_golden.rs`,
   `e2e_memory_anchoring.rs`).
6. **Governance axis** — routes through a **Revision** (ADR-013), cut after design,
   settled in reconciliation (shared with / coordinated against SL-159's catalog
   Revision).

## Non-Goals

- EVD / HYP kinds and the `supports`/`disputes` edges (SL-159).
- Any invariant-native lifecycle redesign beyond the agreed rename + `waived →
  relaxed` (unless the design question in §2 forces it).
- Closing RFC-009 or its broader deliberation (D2/D3/D4/Tier 2).

## Summary

A focused, mostly-mechanical destructive rename — *except* the `waived → relaxed`
semantic question, which is the reason for the split. Behaviour-preservation gate:
existing record suites stay green (adjusted for the rename, never broken). Grep
`Constraint|CONSTRAINT|"CON"|kinds::CON|/constraint|waived` to zero before close.

## Follow-Ups

- IMP-184 (DRY record-kind membership) — this rename re-touches every hardcoded
  prefix site, reinforcing the case.
