# Review RV-077 — reconciliation of SL-097

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Audit of the SL-097 implementation (candidate `cand-097-review-001` at
`c8535ad8`, merged from `review/097` onto `main`). Three phases generalized
`run_supersede()` for record cross-kind supersession: template seeding +
terminal predicate (PHASE-01), policy extraction + rule row (PHASE-02),
cross-kind gating + conditional flip + idempotency (PHASE-03).

**Lines of attack:**

1. **Behaviour preservation** — do all existing ADR supersede tests pass
   unchanged through the refactor?
2. **§6 matrix fidelity** — does `validate_matrix()` match PRD-010 §6 exactly?
3. **Terminal flip correctness** — are already-terminal records preserved?
4. **Cross-family gating** — does ADR↔record refusal work in both directions?
5. **E2E compatibility** — do pre-existing e2e tests survive the error message
   change from "cross-kind" to "cross-family"?
6. **Gate cleanliness** — clippy zero, tests green, `just check` passes.

**Bodies likely buried:** the e2e supersede test at
`tests/e2e_supersede.rs:224` asserts on the old "cross-kind" substring;
the new cross-family gate now emits "cross-family", so this assertion is
stale. Also: the plan's EX-3 parenthetical says QUE→CON "(not in matrix)"
but the §6 matrix allows it — the code is correct, the plan annotation is
misleading.

## Synthesis

SL-097 implemented three phases cleanly. PHASE-01 (template seeding + terminal
predicate) and PHASE-02 (policy extraction + RECORD Supersedes rule row) are
mechanically sound — no findings. PHASE-03 (cross-kind gating) had two
issues, both now repaired on the candidate interaction branch:

**F-1 (blocker, fixed):** The pre-existing e2e test
`supersede_refuses_non_adr_cross_kind_and_self` asserted the old "cross-kind"
error substring. The SL-097 cross-family gate now emits "cross-family", so
the assertion was stale. Fixed by updating the assertion string at
`tests/e2e_supersede.rs:224`.

**F-1 repair exposed a deeper defect:** the same-family gate in `run_supersede()`
treated all `(false, false)` pairs (non-ADR) as records, causing non-record,
non-ADR pairs like SL→SL to enter `RecordKind::from_prefix("SL")` and fail
with "not a valid record kind" instead of the expected "not yet supported
for SL" message. Refactored the gate into a `same_family` boolean computed
by: ADR family, record family (with §6 matrix), same-prefix non-record
(fall-through to policy), and cross-family refusal for everything else.
All 1681 unit + 7 e2e tests pass, clippy zero warnings.

**F-2 (minor, aligned):** Plan EX-3 parenthetical "(not in matrix)" for QUE→CON
is incorrect — the §6 matrix allows question→constraint. The code is correct;
the plan annotation is cosmetic.

**F-3 (major, tolerated):** `just gate` fails on a pre-existing cordage
denylist test (`crates/cordage/tests/denylist.rs`) unrelated to SL-097.
The SL-097-specific gate (doctrine unit tests + clippy) is clean.

### Standing risks

- The `RELATION_RULES` RECORD Supersedes row has `TargetSpec::Kinds(RECORD)`
  which structurally permits all 4×4 directed pairs. The §6 matrix enforcement
  lives only in the verb. If another code path ever writes `Supersedes` edges
  for records (e.g. a future import/apply verb), it must independently
  re-validate the matrix — D4 explicitly accepts this split.
- The terminal-preservation guard (`is_terminal()`) is conservative: an
  out-of-vocab status is treated as terminal (decline to flip unknown status).
  This is safer than flipping unknown statuses, but a corrupted status value
  would silently suppress the flip.

### Tradeoffs consciously accepted

- Cross-kind supersession for records is gated at the verb, not in
  `RELATION_RULES`. This keeps the rule row simple while the verb owns
  the enforcement logic. D4 documents this tradeoff explicitly.
- The `check_already_superseded` helper is extracted but remains in `main.rs`
  rather than `supersede.rs` because it depends on IO-free borrows of parsed
  TOML docs — it's an impure-layer concern.

## Reconciliation Brief

### Per-slice (direct edit)

- **F-1 repair (candidate/097/review-001 @ c8a07d44):** Two files changed:
  - `tests/e2e_supersede.rs:224` — assertion updated from "cross-kind" to "cross-family"
  - `src/main.rs` — same-family gate refactored to distinguish record/record from
    non-record/non-record pairs; non-ADR, non-record pairs now fall through to
    `supersede_policy()` correctly
  The candidate needs to be re-admitted (`dispatch candidate admit --slice 97 --id
  cand-097-review-001 --review RV-077`) and then integrated.

- **F-2 (plan.toml):** EX-3 criterion text should remove "(not in matrix)" from the
  QUE→CON entry. Current text:
  `"§6 reopening directions refused decisively: DEC→ASM, DEC→QUE, QUE→CON (not in matrix), CON→ASM, CON→QUE."`
  Fix: remove `(not in matrix)` or replace with note that the code correctly allows it.

### Governance/spec (REV)

- None. All findings are per-slice.

## Reconciliation Outcome

### Direct edits applied
- `plan.toml` EX-3: removed `QUE→CON (not in matrix), ` from reopening list.
  QUE→CON is allowed by §6; the parenthetical was incorrect (RV-077 F-2,
  commit `df946119`).

### Candidates admitted & integrated
- `cand-097-review-001` admitted at `8a3c030f` (review surface, RV-077 F-1 repair)
- `cand-097-close-001` admitted at `fd317597` (close target)
- Integrated onto `main` at `fd317597` (post-audit repair included via
  `review/097` force-update to include the repair commit `c8a07d44`)

### Tolerated
- RV-077 F-3: cordage denylist test pre-existing failure — tolerated with
  rationale (SL-097-specific gate is clean; cordage issue is environmental).

### Withdrawn
- None.

Reconcile pass complete — handoff to /close.
