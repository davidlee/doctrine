# SL-149 corpus migration — ambiguous-row dispositions (PHASE-05, VT-1 oracle evidence)

Committed evidence for the out-of-band corpus migration (design §2.9, §3 F2). Records the
hand-disposition of every **ambiguous** edge — chosen role + one-line rationale — that the
role-assignment oracle (VT-1) asserts the rewritten corpus against. Deterministic rows are
rule-driven (summarised below, not enumerated); ambiguous rows are judgment, gated by VH-1.

**Census provenance.** Live `doctrine relation list` taken at P05 execution time on
`dispatch/149` @ `ccc42c5` (post refresh-base, trunk +9 SL-150 merged). Counts: specs=67,
requirements=52, related=76 — 195 migrating edges total. The P1 artifact is stale reference only (AR-1).

**Totals.** 195 edges → implements 93 · concerns 76 · scoped_from 14 · related-kept 12.
Deterministic 136 · ambiguous (hand-dispositioned below) 59.

## Deterministic rules (rule-driven, not hand-judged)

| source→target shape | old label | → role | n | basis |
|---|---|---|---|---|
| SL→PRD | specs | references(implements) | 12 | slice realizes the product spec |
| backlog(IMP/RSK/IDE)→canon | specs | references(concerns) | 14 | implements is SL-only; target-gate forces concerns |
| SL→REQ | requirements | references(implements) | 52 | slice implements the requirement |
| SL→backlog(IMP/IDE/ISS/CHR) | related | references(scoped_from) | 14 | slice scoped from that item |
| RFC→non-RFC | related | references(concerns) | 44 | deliberation is *about* the target |

## Ambiguous dispositions (hand-judged — VH-1 gates these)

### A. specs SL→SPEC — implements vs concerns (41)
Default **implements** (the slice builds the capability the spec defines); exceptions flagged **concerns**.

**Flagged `concerns` (12) — the exceptions:**

| source | target | role | rationale |
|---|---|---|---|
| SL-048 | SPEC-005 | concerns | 3-spec structural-edges slice; adds edges touching ADR surface, doesn't build it |
| SL-048 | SPEC-006 | concerns | adds product-product edges touching spec-composition, doesn't build the machinery |
| SL-048 | SPEC-016 | concerns | governance-seam edges touch POL/STD surface, don't build it |
| SL-066 | SPEC-002 | concerns | revision entity is a reconciliation *vehicle*, not the reconciliation engine itself |
| SL-078 | SPEC-010 | concerns | chore sweep (spec-010 rename) touches skills distribution, doesn't build it |
| SL-096 | SPEC-004 | concerns | primary build is SPEC-019 (knowledge records); merely touches entity engine |
| SL-096 | SPEC-018 | concerns | primary build is SPEC-019; knowledge records gain relation edges but don't build the contract |
| SL-129 | SPEC-008 | concerns | primary build is SPEC-004 (entity engine id_path); only touches id lifecycle |
| SL-139 | SPEC-013 | concerns | primary build is SPEC-004 (uniform entity show); CLI surface is incidental |
| SL-139 | SPEC-014 | concerns | primary build is SPEC-004; slice surface is incidental |
| SL-143 | SPEC-007 | concerns | shipped-memory *content* overhaul, not memory-engine machinery |
| SL-148 | SPEC-022 | concerns | primary build is SPEC-008 (id reservation); uses git refs but doesn't define the git model |

**`implements` (29):** SL-036→SPEC-001, SL-038→SPEC-001, SL-039→SPEC-001, SL-043→SPEC-001, SL-046→SPEC-001, SL-047→SPEC-001, SL-050→SPEC-001, SL-057→SPEC-002, SL-058→SPEC-018, SL-059→SPEC-019, SL-060→SPEC-001, SL-065→SPEC-006, SL-095→SPEC-018, SL-096→SPEC-019, SL-097→SPEC-019, SL-101→SPEC-020, SL-102→SPEC-020, SL-103→SPEC-020, SL-104→SPEC-020, SL-107→SPEC-020, SL-118→SPEC-020, SL-124→SPEC-009, SL-129→SPEC-004, SL-131→SPEC-007, SL-139→SPEC-004, SL-145→SPEC-018, SL-146→SPEC-001, SL-148→SPEC-008, SL-149→SPEC-018

### B. related — peer-vs-role judgment (18)

| source | target | disposition | rationale |
|---|---|---|---|
| ADR-002 | ADR-001 | related (unchanged) | GOV<->GOV peer: ADRs compose/coexist, symmetric-neutral -> stays related |
| ADR-004 | ADR-002 | related (unchanged) | GOV<->GOV peer: ADRs compose/coexist, symmetric-neutral -> stays related |
| ADR-010 | ADR-004 | related (unchanged) | GOV<->GOV peer: ADRs compose/coexist, symmetric-neutral -> stays related |
| ADR-014 | ADR-013 | related (unchanged) | GOV<->GOV peer: ADRs compose/coexist, symmetric-neutral -> stays related |
| ADR-016 | ADR-004 | related (unchanged) | GOV<->GOV peer: ADRs compose/coexist, symmetric-neutral -> stays related |
| ADR-016 | ADR-010 | related (unchanged) | GOV<->GOV peer: ADRs compose/coexist, symmetric-neutral -> stays related |
| RFC-002 | RFC-001 | related (unchanged) | RFC<->RFC sibling deliberation, neutral cross-ref -> stays related (CONTESTABLE) |
| SL-107 | SL-101 | related (unchanged) | SL<->SL sibling/sequential peer -> stays related |
| SL-112 | SL-111 | related (unchanged) | SL<->SL sibling/sequential peer -> stays related |
| SL-142 | RFC-002 | references(concerns) | SL->RFC: target gate forbids scoped_from(backlog-only); slice is about the RFC |
| SL-143 | SL-144 | related (unchanged) | SL<->SL sibling/sequential peer -> stays related |
| SL-143 | SL-147 | related (unchanged) | SL<->SL sibling/sequential peer -> stays related |
| SL-144 | SL-143 | related (unchanged) | SL<->SL sibling/sequential peer -> stays related |
| SL-145 | RFC-003 | references(concerns) | SL->RFC: target gate forbids scoped_from(backlog-only); slice is about the RFC |
| SL-146 | RFC-002 | references(concerns) | SL->RFC: target gate forbids scoped_from(backlog-only); slice is about the RFC |
| SL-147 | ADR-007 | references(concerns) | SL->ADR: target gate -> concerns only; slice is about the ADR |
| SL-147 | RFC-004 | references(concerns) | SL->RFC: target gate forbids scoped_from(backlog-only); slice is about the RFC |
| SL-149 | RFC-003 | references(concerns) | SL->RFC: target gate forbids scoped_from(backlog-only); slice is about the RFC |

**Post-migration `related`** = 12 true peers (ADR↔ADR ×6, SL↔SL ×5, RFC↔RFC ×1); everything else migrated out (EX-4).

