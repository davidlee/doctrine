# SL-143 Notes

## 2026-06-24 — RV-150 design/plan review & revision

### Review outcome

17 findings raised via RV-150, all resolved by revising the design and plan.
Key changes:

- **Phase re-sequenced**: PHASE-01 (audit) → PHASE-02 (content) → PHASE-03 (overview) → PHASE-04 (reachability) → PHASE-05 (gate). The overview must be written against the complete post-content-update corpus.
- **Self-correction (objective 3) removed from scope**, deferred to IMP-163 (:after SL-147). Tracked via `tracked_by` relation in slice-143.toml.
- **Dependency updated**: `after=SL-147` (not SL-144). SL-147 provides the domain-map mechanism IMP-163 depends on.
- **New design decisions**: D9 (promotion mechanics: move-and-rewrite), D10 (agent delegation strategy), D11 (ADR-005 skill-to-memory pre-assignment table).
- **POL-002 operationalised**: concrete signature checklist in PHASE-01 audit.
- **D5 extended**: corpus-wide verb-enumeration audit + cleanup of surviving memories.
- **Overview line budget**: ~60 → ~100 lines.
- **Corpus count**: "30" → "29" (preflight found 29, not 30).
- **Plan verification IDs unified** to VA-NN-N format.

### Dispatch notes

The phase sheets under `.doctrine/state/slice/143/phases/` still reflect the
OLD plan (PHASE-02=overview, PHASE-03=content). Before dispatching, the phase
sheets must be regenerated from the revised plan.toml. The plan is now the
authoritative source.

Agent delegation candidates (D10):
- PHASE-01: parallel scout delegation (29 memories, read-only, independent)
- PHASE-02: librarian agent for CLI surface verification
- PHASE-04: parallel researcher agents for subgraph validation; parallel skill edits

### Commit

`e7be9b6b plan(SL-143): revise design & plan per RV-150 critical review (17 findings)`
