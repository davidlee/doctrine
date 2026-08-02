# Review RV-324 — code-review of SL-233

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Subject.** The committed range `12e3fccfe..4b87ee89d` on `dispatch/233` —
PHASE-11 (legacy import reader), PHASE-12 (review choreography, reviewing→locked
gate), PHASE-10 (bounded delegation seam). 19 files, +4182/−76: `src/` 14 files
+2659/−74, `tests/` 5 files +1523/−2. Reviewed at coord head `95430fb14`, where
the review surface is byte-identical to `4b87ee89d`; `src/design_run/prompt.rs`
(added by PHASE-07 after the range) was out of scope.

**Lines of attack.** Eight seams, in expected-yield order: `run.rs` batch/ordering
semantics under the PHASE-10 same-batch acceptance merge; the `snapshot.rs`
watermark and DEC-092's admitted TOCTOU residual; the pure/shell digest split;
`legacy.rs` import conservatism (a prove-a-negative task); `delegation.rs`
staleness and bounding; the `ApplyRequest::writer_act()` prose-only enumeration
invariant; `refusal.rs` variant reachability; and test anti-theatre across the
five e2e files.

**Method.** Two tiers, split on one rule: *the cheap tier was given only work
whose output could be mechanically re-verified.*

- Five enumeration/census passes (deepseek-v4-pro, confined read-only) produced
  evidence tables, each carrying the command that reproduces it — prior-findings
  digest, refusal reachability, `writer_act()` correspondence, digest-derivation
  census, e2e assertion census. Negative results required a positive control.
- Two review buckets (deepseek-v4-pro) took the enumeration-shaped and
  rubric-shaped seams: the delegation seam, and test quality.
- One top-shelf adversarial pass (codex, GPT Sol) took the two seams that turn on
  subtle semantics — the core (`run.rs`/`snapshot.rs`/digests) and import
  conservatism — receiving the cheap tier's tables as *evidence to verify, not
  conclusions to trust*.

That adjudication step earned its keep: the cheap tier proposed
`ApplyRequest::writer_act()` itself as a defect, and the top-shelf pass rejected
it with reasoning (`writer_act()` is consulted only when `act.is_proposal()`, and
coordinator acts may lawfully coexist with coordinator declarations). Only the
test-coverage half of that finding survived, as F-6. The orchestrator then
re-verified both blockers against the source directly before raising them.

**Dedup.** All six findings were checked against the 32 findings already logged
during execution (PHASE-11 F-1…F-14, PHASE-12 F-1…F-9, PHASE-10 F-1…F-9) and
against the disclosed-and-adjudicated set (`ROW_FRAMING_BYTES`, the seven
`attestation.rs` per-item `expect`s, the eight-test pin on `tests.rs`, PHASE-10
`VA-3`/`VA-NC`). None is a duplicate. Three are *adjacent* to prior findings and
say so in their detail: F-1 to PHASE-10 F-8 and PHASE-12 F-5, F-5 to PHASE-11
F-11.

**Standing.** Findings are raised and open; dispositions are the owner's call.
Two blockers (F-1, F-2) contradict locked decisions — DEC-086 and DEC-092 — and
route to `/feedback` and possibly a revision rather than a quiet fix. These
phases are `completed` with `record-delta` recorded, so no landed phase was
edited by this review.
