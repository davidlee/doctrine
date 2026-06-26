# Review RV-168 — design of SL-161

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

This Inquisition examines SL-161's design and plan for heresy — specifically:

1. **Design→plan traceability.** Does every design decision (what to change, what
   to leave) have a corresponding EX criterion? Are the phase boundaries clean?
2. **VA criteria integrity.** Do the per-phase verification assertions hold under
   the actual site inventory? A VA that claims zero literal clusters when
   scan.rs hasn't been touched yet is bearing false witness.
3. **Design accuracy vs current code.** The design claims to describe "Before"
   states — do those match what `src/` actually contains?
4. **ADR-001 conformance.** `kinds.rs` is leaf; `integrity.rs`/`knowledge.rs` are
   engine. Does the plan respect the tier boundary?
5. **Behaviour-preservation gate specificity.** The design says "existing suites
   stay green" — but which suites are the canaries for each conversion?

Evidence gathered from the living code (all 14 selectors read), the governing
ADR-001, SPEC-008, SPEC-019, and the record-kind touch-site memory.

## Synthesis

### Overall: acceptable

The design proper is **sound** — no doctrinal heresy found in its decisions,
tier assignments, or technical claims. The design follows the existing backlog
two-source convention, respects ADR-001 layering, carries a thorough internal
adversarial pass (F1–F6), and maps every conversion to a named site with
before/after code. The inquisition's three findings all land on the **plan**
— the design's faithful but imperfectly-specified executor.

### Synopsis

**The design survived adversarial scrutiny intact.** All 14 selectors were read
against their design claims; the "Before" states match the living code; the
record-member census (12 sites → 1 source) is both necessary and sufficient.
The supersede/superserde "no-change" verification holds — those files already
route through `RecordKind::from_prefix`. ADR-001 layering is respected:
`kinds::is_record` sits in the leaf tier, all callers are engine or command.
The count-assertion approach for KINDS (vs a macro) is proportionate — a
one-line test addition for a 21-row table in a single file.

**The plan has three correctable blemishes, none blocking:**

1. **PHASE-01 VA-1** promises a grep-zero-literal-clusters state that the plan's
own PHASE-02 deferral (scan.rs) and accepted exception (dep_seq.rs:285)
contradict. Relocate and scope it.

2. **EX-8** says "pin test rewritten" without naming the function — the
implementer needs to know WHICH test dies and what the replacement is called.
Add the function name.

3. **PHASE-02 VA-3** verifies a pre-existing condition on files the slice never
touches. Move it to plan.md prose — it's a design-verification note, not a
phase exit criterion.

**Standing risks:** the design correctly acknowledges the hand-maintained
membership problem (Non-Goals §4) — Rust's lack of const reflection means the
KINDS registry can't be compile-time exhaustive. The count assertion is a
runtime drift canary, not a compiler gate. This is a known limitation, not a
flaw. SL-159 (EVD+HYP) will stress-test the DRY by adding two kinds — the
true measure of this slice's success.

**Tradeoffs consciously accepted:** dep_seq.rs:285 admissible vector stays
literal (mixed superset, more readable as-is); search.rs:25,38 stay literal
(full-corpus const arrays, composability limited by Rust const rules); no
coherency test between `kinds::RECORD` and `RecordKind::ALL` (follows backlog
convention). All three are reasoned, not overlooked.

### Haiku

*Four prefixes, twelve sites —*  
*now one const speaks for all.*  
*The plan's grep must wait.*

### Penance ordered

1. Relocate VA-1 from PHASE-01 to PHASE-02, scoped to exempt dep_seq.rs:285.
2. Add function name `is_record_is_exactly_knowledge_records` to EX-8.
3. Move VA-3 from PHASE-02 verification to plan.md §Notes prose.
