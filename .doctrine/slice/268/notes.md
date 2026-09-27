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
fresh-as-of: 2026-09-27 · close · 13756e66b

### Produced
- design locked after RV-396 (run dr-01a0dc9c); DEC-317..DEC-322
- ten phases via capsule c gen 20, merged to edge at 39212685a; check gate green on edge at close (9 015 tests)
- SPEC-032 (active); REV-064 (done: ADR-007, SPEC-003); RV-397 (guidance review); RV-398 (audit)
- resolved: ISS-314, ISS-366, ISS-280, ISS-485, ISS-059, IMP-029, IMP-377, IMP-433, IMP-259, IMP-336, CHR-001; IMP-479 closed wont-do; IMP-068 partial, left open
- minted: IMP-492, IMP-493, ISS-492, ISS-493, ISS-494 (resolved), IMP-495 (secret scanning)

### Learned
- mem_01a0062ebd7375f3a2f67ed833307283 — layering gate skips sub-classified out-edges; split engine units top-level
- mem_01a0dd0af5367eb39223bb694e390380 — design sections materialise in lexical id order
- mem.pattern.review.done-requires-concluded, mem.pattern.review.reopen-to-revise-verified — v2 ledger rules
- mem_019f97fcab2e77a28902371f80743605, mem_019fdfe379b67e53857735b88c394b52 — updated for D1/D2
- capsule-landed phase state stays in the landing worktree (observation at close; scripts/oubliette.sh advice; SL-269 OQ-2) — no memory: SL-269 retires it

### Open
- SL-269 — review-capable worktree locus (RV-398 F-5, IMP-240)
- IMP-068 (partial), IMP-492, IMP-493, ISS-492, IMP-495
- RFC-032 slices 2–4 (read surface, design-run binding, identity & locus)

## PHASE-08 test-suite flips (D11)

D11 (design.md sec-5 "Prime") makes `review prime` degrade-and-disclose instead
of bailing on a non-slice target or a zero-selector slice, and ISS-059 makes it
filter (not error on) a literal selector naming a directory or symlink. These
existing tests changed shape to match — named here per the phase sheet's
"Expected test changes" so the audit can see the flips were intended, not
drift.

Unit (`src/review/tests.rs`, deleted; replaced in `src/review/prime.rs`'s new
`#[cfg(test)] mod tests`):

- `vt2_prime_bails_named_on_a_non_slice_target` → `prime_non_slice_degrades`
  (D11, IMP-259: was an `Err` assertion, now `Ok(Primed { degraded: Some(_),
  .. })`).
- `vt2_prime_bails_named_on_a_slice_with_no_selectors` →
  `prime_zero_selectors_degrades` (D11, same flip for the zero-selector case).

Golden (`tests/e2e_review_golden.rs`, "T7 — prime / status cache" block):

- `prime_refuses_non_slice_target` → renamed `prime_degrades_on_non_slice_target`
  (D11, IMP-259: exit ≠0 stderr error → exit 0, `RV-001 primed nothing: …`).
- `prime_refuses_zero_selector_slice` → renamed
  `prime_degrades_on_zero_selector_slice` (D11, same flip).
- `prime_refuses_directory_selector` → renamed `prime_skips_directory_selector`
  (D11, ISS-059: was `Is a directory (os error 21)`, now a successful prime with
  a `skipped non-file selector: src` line — same fixture, same selector).
- new: `prime_degraded_clears_previous_cache` (D11, RV-396 `F-6` — a degraded
  prime removes an earlier `cache.toml`, and `status` stops reporting it).

## Audit harvest (2026-09-27, RV-398)

Lifted from the runtime phase sheets before they are discarded.

- **Design divergences** (RV-398 `F-3`, reconciled into design.md):
  - PHASE-03 D1: `FindingState.status` is `Vocab<FindingStatus>`, not
    `Option<FindingStatus>` plus `raw`. Same information, one representation.
  - PHASE-03 D2: the JSON/MCP `warnings` field is omitted when empty
    (`skip_serializing_if`, IMP-114 precedent), so clean-ledger output is
    byte-identical.
  - PHASE-03 D3/D4: CLI `list` writes warnings to stderr in both formats; every
    warning line names the RV.
  - PHASE-08 P2/P4/P5: `Primed` gains `skipped`; an all-skipped slice is not
    degraded (empty cache, skip lines); fifo/socket/device literals are excluded.
  - PHASE-10: IMP-479 closed `wont-do`; it reopens on measurement.
- **Test flips.** Every changed pre-existing assertion is named in its phase
  sheet: PHASE-04 (`note_is_handoff_chatter…` replaced by `note_lands_in_turn`),
  PHASE-05 (D8 fixture inputs: disposition `fixed` → closed vocab; MCP tool
  count 29 → 31), PHASE-06 (U1–U14, G1–G5: the D2 `done`/`active` flips and
  `concluded = true` fixture lines), PHASE-07 (declared-label role refusal),
  PHASE-08 (above).
- **Accepted risks.** Legacy corpus RVs now read `active` until concluded
  (design R2; do not bulk-conclude them). The baton `awaiting` of an
  all-terminal unconcluded ledger is `raiser` (was `none`); only tests read it.
  A typo'd severity on an open finding now holds a design-run lock edge and a
  slice close (intended teeth).
- **Audit-locus friction.** Review verbs refuse an adopted capsule worktree;
  the capsule was merged into edge (`39212685a`) to run the audit. ISS-494.
