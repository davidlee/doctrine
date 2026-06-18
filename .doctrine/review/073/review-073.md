# Review RV-073 — reconciliation of SL-096

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Audit of SL-096 (Knowledge-record relation seam) against `review/096` — the
impl-bundle candidate branch. This audit interrogates five surfaces:
`src/relation.rs` (vocabulary + RELATION_RULES), `src/knowledge.rs` (tier1 read
path + show/inspect render), `src/catalog/scan.rs` (outbound_for dispatch),
`src/relation_graph.rs` (exact-coverage invariant), and
tests/e2e_knowledge_cli_golden.rs (show_json golden).

Lines of attack: D1-D6 design conformance; PHASE-01 VT 1-10 completion;
PHASE-02 VT 1-5 + VA-1 + VH-1 gating; scope boundary (no Supersedes leak, no
template changes, no Drift extension); existing-suite behaviour preservation.

The bodies most likely to hide issues: golden churn in relation.rs tests (5
goldens updated), exact-coverage extension (4 new per-prefix fixtures), and
the SL-059 VT-1 replacement (renamed test with new fixture). All three were
verified clean — no findings requiring remediation.

Evidence surface reviewed: `review/096` at 050063a5 (impl bundle on top of
main 03a5cf84). Tests run on review/096: 1664 passed, 0 failed core, 1
pre-existing ignored (sync_produces_all_shipped_dirs). just gate: zero clippy
warnings.

## Synthesis

SL-096 delivers SPEC-019 FR-005 cleanly across 5 files, 392 insertions, 19
deletions. Every design decision (D1-D6) is faithfully implemented; every VT
criterion in plan.toml is verified green; the behaviour-preservation gate
holds (all existing suites unchanged).

**Closure story.** Two new `RelationLabel` variants (`Shapes`, `Spawns`) enter
the cross-corpus relation contract. A single epistemic label `shapes` covers
all record→artefact influence, avoiding the rejected source-set extensions
that would have produced dishonest inbound labels. `Spawns` enables record→
backlog work creation. `GovernedBy` gains record sources. The tier1 read path
(`KnowledgeRecord.tier1`, `relation_edges`, `tier1_edges`) follows the
existing backlog precedent. `outbound_for` dispatch is live. `show`/`inspect`
render the three axes (`shapes`, `spawns`, `governed_by`).

**Standing risks.** One nit: the e2e link/unlink suite doesn't exercise
knowledge-record link/unlink end-to-end through the built binary. The link
verb is a shared mechanism validated through RELATION_RULES (single source of
truth), and unit-level relation.rs tests cover Shapes/Spawns lookup and
target-kind refusal. No dedicated e2e is needed before close given the
mechanical coverage — the risk is low and the audit downgrades this to nit.
The IMP-093 backlog item captures the FR-006/IMP-006 supersession follow-up.

**Tradeoffs consciously accepted.** The D4 decision to seed no `[[relation]]`
comment in templates is correct for toml_edit compatibility. The D2 explicit
16-kind target set for Shapes is verbose but semantically precise — each
addition is deliberate, and new numbered kinds must opt in. The RECORD
source-group const avoids drift between the two new RELATION_RULES rows.

## Reconciliation Brief

### Per-slice (direct edit)

None. All 5 audit findings are `aligned` — no design, code, or governance
drift was observed. The implementation is byte-for-byte conformant with
design.md.

### Governance/spec (REV)

None. No spec or governance changes surfaced by this audit.
