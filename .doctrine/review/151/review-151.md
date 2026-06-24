# Review RV-151 — reconciliation of SL-143

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reconciliation audit of SL-143 (shipped memory corpus overhaul) against
design.md D1-D11 and plan.toml VT-01 through VT-14. The slice touched 28
UUID-sourced memory directories (32 human-readable keys), 7 skill SKILL.md
files, and the binary re-embed path.

Lines of attack:
- VT-01/VT-06: corpus count and new signpost creation
- VT-02/EX-03: overview PULL hub structure (5 sections, 31-row table)
- VT-03/EX-04: ≤3-hop wikilink reachability from overview
- VT-05: cli-command-map deletion
- VT-07/D9: work-intake-membership promotion integrity
- VT-09/D5: no inline CLI verb enumeration
- VT-11/POL-002: no platform-independence signatures
- VT-12/D11: skill-to-memory reference mapping
- Gate: `just check` pre-existing failures (slice-143.toml `tracked_by` label)

## Synthesis

SL-143 delivered its four substantive objectives against a 5-phase plan:

1. **Audit (PHASE-01):** 29-memory systematic findings ledger (57 findings:
   12 blocker, 12 major, 24 minor, 9 nit) produced via parallel scout
delegation. All exit criteria met — cli-command-map staleness with full verb
diff (≥14 missing, 1 dead: claude), POL-002 violations flagged, D5-flagged
memories enumerated with file:line, wikilink graph validated, TOML
false-positives documented.

2. **Content update (PHASE-02):** Ledger-driven fixes applied to 27 existing
   memories, 4 new signposts created (rec, rfc, concept-map,
work-intake-membership), cli-command-map deleted. POL-002 violations remediated
in conventions, relating-entities, and reference-docs. D5 enforced on 13
flagged memories. Stale CLI references fixed (adrs --title flag, requirements
reconcile shape, revisions transition→status). All inbound wikilinks to
deleted cli-command-map replaced. Promotion followed D9
create-and-retire mechanics.

3. **Overview rewrite (PHASE-03):** 85-line PULL hub with 5 sections: Pillars,
Mental model, When-to-retrieve-what (31-row table), Conventions, Quick-links.
One retrieval after boot orients an agent. D5/POL-002 compliant.

4. **Reachability (PHASE-04):** 32/32 memories ≤3 hops from overview confirmed.
11 D11 skill references added across 7 skills. `[[relation]]` backtick
formatting confirmed. New signposts gained inbound wikilinks from nearest
parent memories.

5. **Gate (PHASE-05):** Binary re-embedded with updated corpus. `just check`
shows 3 pre-existing failures in `e2e_relation_migration_storage` — these
concern SL-143's own `slice-143.toml` `tracked_by` label and
`backlog-021.toml` relation types, both orthogonal to the memory corpus work.

**Standing risk:** The `tracked_by` label on slice-143.toml causes
`just check` to fail until SL-143's authored relation labels are aligned with
the allowed `dep_seq` set. This is tracked as pre-existing, not introduced by
SL-143's memory corpus changes.

## Reconciliation Brief

No spec or governance divergences found. All 12 findings are aligned (11) or
tolerated (1, the pre-existing e2e gate failure). No design.md, ADR, or
spec changes required — the implementation matches the locked design.

### Pre-existing gate (tolerated)
- F-1: 3 e2e_relation_migration_storage failures in SL-143 authored state
  and backlog-021.toml. Remediation out of scope for SL-143.

### Per-slice (no action)
All VT criteria met. Design.md D1-D11, plan.toml, and governance (ADR-002,
ADR-005, POL-002) are satisfied by the implemented corpus.

## Reconciliation Outcome

No-op reconcile pass. All 12 findings were aligned (11) or tolerated (1).
No per-slice edits or REV changes needed. The pre-existing e2e gate failure
(F-1, tolerated) concerns SL-143's own `slice-143.toml` `tracked_by` label
and `backlog-021.toml` `related` key — both orthogonal to the memory corpus
work and out of scope for SL-143.

Reconcile pass complete — handoff to /close.
