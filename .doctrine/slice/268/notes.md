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

## Design review passes (2026-09-26)

- **RV-396** (codex-cli `gpt-6-sol`, high): 9 findings. 7 were verified on the
  first repair. F-3 and F-4 were contested twice and verified on the third
  round. The user ruled F-3 (catalog disclosure moves to ISS-492) and F-4
  (`raise`/`reopen` clear `concluded`). The review is concluded, with every
  finding terminal.
- **A further pass is not needed before lock.** The last two rounds changed
  only prose claims about existing code, and the raiser re-checked those
  against `doctor_checks.rs` and `admission.rs`. If a pass is wanted, it would
  probe:
  - (a) whether `review_ledger` really needs nothing from a command module,
    especially `read_review`'s `REVIEW_DIR` and `relation_edges`'
    `RelationLabel`;
  - (b) whether clearing `concluded` on raise interacts with any caller other
    than the design run's admission.
  Both are cheaper to settle in the split and D2 phases, against real code,
  than in prose.
- The pass is shown `STALE` because it is tied to the rev-23 section contents.
  That indicator is not a gate (`design_run/gate.rs:1140`).

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-26 · design (reviewing, pre-lock) · 8922d2e77

### Produced
- design.md drafted and revised for RV-396 (commits 810fef9dd..8922d2e77 on edge; run dr-01a0dc9c)
- minted: DEC-317..DEC-322 — inquiry answers; IMP-492 — doctor install-doc CLI check; IMP-493 — doctor status/last-turn check; ISS-492 — catalog callers drop warning diagnostics; RV-396 — design review
- design goes past the frontier: top-level `review_ledger` module; `raise`/`reopen` clear `concluded` (user ruling, RV-396 F-4)

### Learned
- mem_01a0062ebd7375f3a2f67ed833307283 — layering gate skips sub-classified out-edges; split engine units top-level
- mem_01a0dd0af5367eb39223bb694e390380 — design sections materialise in lexical id order

### Open
- DEC-317..DEC-322 — shape the slice; cite in plan
- RV-396 pass disposition, section review, design acceptance — awaiting user
- notes "Design review passes" — two probes deferred to the split and D2 phases
