# CHR-172: Sample-based test quality review

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Intent

Audit test quality by depth, not breadth: choose a representative sample of
test modules across the codebase and review each one thoroughly.

## What to look for

- **Brittle** — asserts on incidental detail (exact prose, ordering, formatting,
  private structure) where the behaviour under test is narrower.
- **Vacuous** — passes regardless of the code under test: tautologies, asserts
  that can't fail, over-mocked seams, no assertion on the outcome.
- **Slow** — needless process spawns, real git/disk where a pure seam exists,
  repeated heavy fixtures.
- **Intent-opaque** — names and bodies that don't say what behaviour is pinned;
  setup noise drowning the assertion.
- **Duplication** — hand-rolled fixtures that an existing helper already covers.

## Sampling

Stratify by layer (ADR-001 leaf / engine / command), by test type (unit,
integration/CLI, golden), and by age/churn. State the sample and its rationale
in the review so coverage of the sample is auditable.

## Output

Findings on an RV (review) ledger via `/code-review`, severity-ranked. Follow-up
fixes become their own backlog items or a slice; systemic patterns become
test-helper improvements or a memory.

## Progress (2026-09-29)

- Review run as RV-411 (`doctrine show RV-411`): 21 findings raised (13 major,
  7 minor, 1 nit), raiser pass concluded; dispositions pending — routing of
  fixes (slice vs backlog items) is the user's call.
- Production defects surfaced and filed: ISS-500 (draft expiry sign), ISS-501
  (sha1-only ZERO_OID).
- ISS-008 appears stale (test renamed; 90 stressed runs clean) — recommend close.
