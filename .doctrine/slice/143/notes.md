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

## 2026-06-24 — Dispatch & audit (RV-151)

### Dispatch outcome

5 phases completed in the dispatch funnel (coordination tree `.dispatch/SL-143`).

- **PHASE-01 (audit):** 57 findings across 29 memories via parallel scout
delegation (5 batches). Ledger at `.doctrine/state/sl-143-phase-01-ledger.md`.
- **PHASE-02 (content):** 41 files changed under `memory/`. +4 new signposts
(rec, rfc, concept-map, work-intake-membership), -1 deleted (cli-command-map).
D5/POL-002 remediated. 32 total memories.
- **PHASE-03 (overview):** 85-line PULL hub. 5 sections (Pillars, Mental model,
When-to-retrieve-what, Conventions, Quick-links). 31-row table.
- **PHASE-04 (reachability):** 32/32 ≤3 hops from overview. 11 D11 skill
references across 7 skills.
- **PHASE-05 (gate):** Binary re-embedded. Sync: 28 shipped. Pre-existing
e2e_relation_migration_storage failures (slice-143.toml `tracked_by` label,
backlog-021.toml `related` key) — orthogonal to memory corpus.

### Review outcome (RV-151)

12 findings raised, all verified (11 aligned, 1 tolerated). No spec/governance
divergences — all VT criteria met. Pre-existing gate failure tolerated.

### Standing concerns

- `slice-143.toml` uses `tracked_by` relation label not in allowed dep_seq set.
Causes `just check` to fail. Pre-existing — remediation out of scope for
SL-143.
- `backlog-021.toml` has `related` as typed key causing e2e failure. Pre-existing.

### Commit

Coordination branch `dispatch/143` at `eb57b52`. Review ref at `review/143`.
`prepare-review` completed.
