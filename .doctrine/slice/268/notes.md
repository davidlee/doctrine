# Notes SL-268: Review ledger v2

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design triage (2026-09-26, explore stage)

Design run `dr-01a0dc9c`. Evidence: `research/research.md` (tier 5) and RFC-032
`decision-frontier.md` (tier 5 until lock).

- **Open questions → run inquiries.** inq-1 (research T1, SPEC-003 inventory REV),
  inq-2 (T2, DEC-233 `Unavailable` arm), inq-3 (T3, unknown-severity disclosure),
  inq-4 (OQ-1, D13 doc check), inq-5 (OQ-2, spec timing), inq-6 (OQ-4, doctor
  checks; needs inq-3).
- **Constraining governance.** STD-003 (disclose on the caller's channel — a
  doctor-only finding does not reach `review show`/`status` callers); ADR-001
  (new modules in `layering.toml`, research X1 name collision with dispatch
  `ledger`); ADR-007 D-C5/C8/C10 (REV); DEC-138 (design-run gate reads
  `PassFacts`, not `derived_status`); DEC-233; SPEC-003 inventory rule (REV-035);
  STD-001 (closed vocab constants + canaries).
- **Shaping decisions already made** (RFC-032 §5, user-settled): D1, D2, D4,
  D8, D10, D11, D12, D15 as scoped.
- **Risks.** Scope R1–R4. Added: memory `mem_019fe112c6af7d908afe714d41ddb718`
  — a new doctor finding category touches six sites and one drops silently.
  Memory `mem_019f999f39fe7820a0831f4413a539b4` — spec `[[source]]` anchors
  rot silently (bears on inq-5).
- **Assumptions.** Scope's two, plus research T4: new authored fields ride
  `serde(default)`, absent-default, no migration.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: <yyyy-mm-dd> · <PHASE-NN | stage> · <head-commit>

### Produced

### Learned

### Open
