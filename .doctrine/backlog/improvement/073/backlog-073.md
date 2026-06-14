# IMP-073: SL-066 REV quality hardening: shared test harness, module decomposition, unit-test the row-build validation, settle_disposition trapdoor, doc nits

Source: SL-066 reconciliation review (RV-029, external code-review findings F2/F4/F5/
F6/F7/F8). The SL-066 bundle is correct, green, and ship-ready — these are non-blocking
quality improvements deferred so the verified-green dispatched bundle (`review/066`)
integrates sealed. None gates close; all are owned future work on `src/revision.rs`.

## Scope

- **F2 — shared test harness (DRY).** The five `tests/e2e_revision*.rs` files each
  copy-paste a ~40-line `Repo`/`RevRepo` harness (`git()`/`run()`/`ok()`/`BIN`/
  `seed_adr|req|spec`/`write`/`read`/`*_status`/`rec_count`). Extract one shared
  `tests/common_revision.rs` (or repurpose `RevRepo`) all five import. ~150 lines of
  duplication; goldens don't change. Highest ROI.
- **F5 — module decomposition.** `src/revision.rs` is 1478 lines with four clean
  seams. Extract `src/revision/change.rs` (`ChangeRow`/`ChangeAction`/`ChangeDoc`/
  `build_row`/`build_*_row`/`append_change_row`/`read_change_rows`) and
  `src/revision/apply.rs` (`run_apply`/`PlannedStatus`/`StaleFinding`/`compose_apply_rec`/
  `settle_disposition`/`parse_req_status`/`partition_change_rows`); the kind spine stays.
  Follows the `dispatch/` subdir precedent (SL-064); no public-API change. Consider
  applying the same to the REC module (rec.rs ~1178 lines — the precedent F5 cites).
- **F6 — unit-test the row-build validation.** `build_creation_row`/
  `build_existing_target_row` (~200 lines: E4 frozen `new_label`, shape routing, OQ-1
  dedup, `from` auto-capture, target-kind validation, `member_of`-must-be-SPEC) are
  covered only through e2e binary goldens. Add pure unit tests over `build_row` with
  mock roots — ms per case vs seconds. Compounds with F2.
- **F4 — `settle_disposition` trapdoor.** The `_ => &[]` arm silently no-ops if a
  future caller passes `Proposed`/`Abandoned` as target (caller then prints a lie).
  Today only `Started`/`Done` are passed, so no live bug — harden the `_` arm to
  `unreachable!`/`bail!`.
- **F7 — magic-`0` placeholder id.** `compose_apply_rec` sets `RecDoc.id = 0` (engine
  assigns at `materialise_populated`). Precedent-aligned with `rec::run_new`, but make
  the contract explicit: a `const REC_PLACEHOLDER_ID: u32 = 0` (or a doc-comment on
  `RecDoc.id`) in `rec.rs`.
- **F8 — TOCTOU doc-comment.** The gap between `run_apply`'s from-guard sweep and the
  per-row writes is by-design (single-operator, drift-surface posture, ADR-009 / SL-044
  precedent) — not a bug. Add a code comment recording it as a known, intended property.

Out of scope (dispositioned terminal on RV-029, no code change): F1 `allocated`
(design-intended operator-hand-fill anchor, design.md:228), F3 parse-coupling (REC
precedent), F9 branch-lag (review-base artifact), F10 dup-slug (all-kinds precedent).
