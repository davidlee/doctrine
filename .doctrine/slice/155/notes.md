# SL-155 — Implementation & Audit Notes

## Audit summary (RV-162)

Clean audit. All 7 Cluster A one-liners + supersede edge match design exactly.
Revision list verb fully conformant — 5-column REV_COLUMNS (no slug, D5),
tags opt-in (D2), terminal hide (D1), REC-shaped (D3). 33 revision tests green,
clippy zero warnings.

Two findings, both terminal:
- **F-1** (aligned): PHASE-01 source-delta was missing — bootstrapped.
- **F-2** (tolerated): e2e_dispatch_candidate failures are pre-existing
  infrastructure debt, not SL-155 regression.

## Implementation notes

### PHASE-01
- 6 of 7 one-liners pre-applied by eager agent (4de1d4e2); verified, not authored.
- C3 (spec.rs doc comment) and G5b (supersede edge) were the remaining work.
- Supersede edge confirmed both ways: ADR-012.supersedes = ["ADR-004"],
  ADR-004.superseded_by = ["ADR-012"].

### PHASE-02
- Eager agent built ~80% against old design (tags default, slug column).
- Design alignment: removed slug from REV_COLUMNS, changed REV_DEFAULT to
  tags-opt-in, added round-trip test.
- CLI wiring: RevisionCommand::List + dispatch arm in run_revision.
- `revision show --json` includes tags in nested revision object (added in
  4fe7fd4d). Human-readable tag rendering in `show` deferred to IMP-170 G2.

### Conformance
- PHASE-01 delta recorded (a29b6cd4..9cf34868). PHASE-02 not recorded —
  commit range entangled with post-close fixes (4fe7fd4d, 6e86a30c) and SL-156.
- 39 undeclared paths in conformance report are from SL-154, SL-156, SL-138
  landing on same branch.
