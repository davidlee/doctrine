# Review RV-032 — reconciliation of SL-068

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reconciliation audit of SL-068 (dispatch candidates for safe audit interaction)
against design.md §5.2/5.3/5.5, the ADR-006 D10 / ADR-012 candidate clauses, and
the plan.toml EX/VT criteria for PHASE-01..07.

Lines of attack:
- Does the candidate ledger model and the create/admit/status/integrate verbs
  match the locked design contract (admission-by-OID, no close-time merge,
  provenance validation, raw-ref/worker guards)?
- Do the EX/VT criteria hold on the audited bundle (review/068 = phase/068-06 code)?
- Independent gate re-run: clippy clean + tests green (not just the funnel's word).
- Is the discoverability minimum (OQ-1) actually shipped and reachable cold?
- Verification integrity: do the VT tests assert the property, or a proxy?

## Synthesis

**Verdict: SL-068 is audit-ready and reconciled.** The implementation matches the
locked design across all seven phases; the two findings are both non-blocking
coverage/scope debts captured as backlog follow-ups, not defects in the slice.

### Reviewed material

The slice is pre-close: the candidate code is not on `main` but on the dispatched
evidence bundle. `review/068` (impl bundle) and `phase/068-06` (code tip) carry an
**identical** source delta — `src/ledger.rs` (+364), `src/dispatch.rs` (+829),
`src/git.rs` (+81), `src/main.rs` (+163), `tests/e2e_dispatch_candidate.rs`
(+1763). The audit ran the gate independently on a throwaway worktree checked out
on `review/068` (dedicated CARGO_TARGET_DIR to defeat the shared-target false-RED).

### Conformance (design ↔ code)

- **PHASE-01 ledger model** — `Candidates`/`CandidateRow`/`CurrentAdmission`/
  `Admission` match design §5.3; `kind`/`role`/`payload`/`status` are enums (a
  malformed token is a parse refusal, VT-3); `set_candidate_status` is the only
  mutating helper (status write-once-else, EX-3); absent file → empty manifest.
- **PHASE-02/03 create** — provenance gate fires by ref name before any resolve or
  write (verified journal row; phase-chain hole refusal for code close targets);
  zero-OID CAS branch refuses an existing target; explicit no-ff 3-way merge
  records base+source as merge parents; conflict aborts with no durable row unless
  `--worktree`; raw-evidence + review_surface-needs-worktree guards refuse first.
- **PHASE-04 status** — evidence vs candidate refs are rendered in separate groups;
  drift (live tip ≠ admitted/recorded OID) is surfaced; output prints the safe next
  verb; read-only (warns, never refuses, from a raw-ref worktree).
- **PHASE-05 admit** — read → validate provenance (merge_oid parents == base+source
  AND merge_oid ancestor of admitted tip, I3/R7) → **re-read before record**
  (TOCTOU moved-ref refusal) → immutable `admitted_oid`; close_target supersession
  leaves exactly one current admission (I5); orchestrator-classed + guarded.
- **PHASE-06 integrate** — `plan_candidate_trunk_row`/`plan_candidate_edge_row`
  source the admitted OID, append an ff-only CAS journal row
  (`planned_new_oid = admitted_oid`), **never** a close-time merge (I6), refuse a
  moved target with a named reason (R4/D6), and refuse rather than fall back to a
  raw ref when the admission is absent. Existing stage-1/stage-2 sync suites stay
  green unchanged (behaviour-preservation gate held).
- **PHASE-07 discoverability** — dispatch/audit/close SKILL.md carry the candidate
  pointers; CLAUDE.md known-CLI-gaps documents the shipped workflow; the boot
  snapshot is current (not stale); the `cli-command-map` global master gains the
  candidate verbs (on the bundle, lands at close); project-local build-seam memory
  recorded; OQ-1 minimum shipped, full /audit + /code-review rewire deferred
  (IMP-042) and orientation masters deferred to SL-069 (CHR-009).

### Independent verification

- `cargo clippy` — clean, zero warnings.
- candidate suite `e2e_dispatch_candidate` — 23/23 green.
- full suite — green **except** one pre-existing date-coupled golden (F-2, below).

### Findings (both minor, both follow-up — no blockers)

- **F-1** — VT-2's *moved-during-admit* refusal is asserted by test name only. The
  TOCTOU guard is present and correct, but a single-process black-box test cannot
  move the ref between the two resolves, so the refusal branch is uncovered and the
  test name (`..._moved_ref_refuses`) overstates it. The test that exists actually
  proves the *post-admit* immutability (I4). → IMP-077.
- **F-2** — the gate surfaced `e2e_dep_seq_verbs::slice_needs_after_round_trip_*`
  failing on a hardcoded `2026-06-14` golden vs today's wall clock. Out of SL-068
  scope (not in the diff; fails identically on `main`). → ISS-017.

### Standing risks / consciously accepted

- The audited code is **not yet on `main`** — it lands only at `/close` via the
  admitted close_target OID (the design's whole point). Audit reviewed the bundle,
  which is the correct material per ADR-012.
- F-1 leaves a defensive concurrency guard without automated proof. Accepted: the
  guard is correct by inspection; the cost is test infrastructure, deferred to
  IMP-077, not a SL-068 code change.
