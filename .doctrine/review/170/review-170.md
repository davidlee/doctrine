# Review RV-170 — reconciliation of SL-161

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Lines of attack:
1. **Conformance** — do the 9 changed files match the 9 design-target selectors?
   Are there undeclared or undelivered paths?
2. **Design fidelity** — does each edit match its design §5.2/§5.3 specification?
   Check kinds.rs predicate, dep_seq.rs delegation, partition.rs guard, search.rs
   alias, tag.rs test, relation.rs census, test_helpers.rs dispatch, scan.rs
   restructure, integrity.rs count.
3. **Behaviour preservation** — do all existing suites stay green? No unintended
   dispatch changes, no test narrowing, no new warnings.
4. **Verification criteria** — do VT-1/VT-2/VA-1/VA-2 pass against the review ref?
5. **Clean grep** — are remaining record-literal clusters only the sites
   designated to stay per design §5.4?

Invariants:
- `kinds::RECORD` is the sole grouping source of truth for record membership.
- `RecordKind::from_prefix` is the enum dispatch source (backlog convention).
- `scan.rs` fallthrough `debug_assert!` panics on unrouted KINDS prefixes.
- ADR-001 layering preserved: `kinds.rs` (leaf) → engine/command callers.

## Synthesis

All five lines of attack pass cleanly:

1. **Conformance:** 9 conformant paths, 0 undeclared, 0 undelivered — every
   changed file has a matching `design-target` selector. No scope creep.

2. **Design fidelity:** Every edit matches its design specification §5.2/§5.3.
   The `is_record()` predicate reads `kinds::RECORD` (leaf tier, zero imports).
   All ~12 literal-prefix sites are converted to read from the registry —
   dep_seq delegates, partition guards, search aliases, relation census reads
   from `RECORD`, test_helpers uses `RecordKind::from_prefix`, and scan.rs
   restructures to a guard-based branch in `other`. KINDS now carries a count
   assertion that catches missing/extra rows.

3. **Behaviour preservation:** Full test suite green (2607+ passed), clippy
   zero warnings, architecture layering gate passes. No regressions. The
   `is_record_predicate_matches_kinds_record` test is stronger than the old
   pin (set equality over KINDS vs RECORD).

4. **Verification criteria:** All VT-1/VT-2/VA-1/VA-2 satisfied. Grep shows
   remaining record-literal clusters only at the sites designated by design
   §5.4: kinds.rs (RECORD const), dep_seq.rs:283 (admissible vector, F2),
   integrity.rs:821 (KINDS prefix pin), relation.rs:1422,1427 (mixed
   supersets), search.rs:39 ("all" alias). scan.rs has no literal record
   arm — dispatch routes through `RecordKind::from_prefix` guard.

5. **Findings:** Two raised, both resolved. F-1 (minor): plan VA-1 imprecise
   — doesn't list all design-exempted sites; classified as aligned (code
   correct). F-2 (nit): scan.rs fallthrough comment removed in restructure;
   tolerated with rationale (old text misleading after restructure,
   `debug_assert!` message adequate).

**Verdict:** Implementation is faithful to design. No blockers. Ready for
reconciliation and close.

## Reconciliation Brief

### Per-slice (direct edit)

- **Plan VA-1** (F-1): Add integrity.rs:821, relation.rs:1422,1427, and
  search.rs:39 to the expected-hit list — the criterion currently says
  "zero outside kinds.rs and dep_seq.rs:285" but the design explicitly
  designates these additional sites as staying (§5.4).

## Reconciliation Outcome

### Direct edits applied
- Plan VA-1 (F-1): Updated to list all design-exempted sites — kinds.rs
  (RECORD const), dep_seq.rs admissible vector, integrity.rs KINDS prefix pin,
  relation.rs mixed supersets, search.rs "all" alias. Now matches design §5.4.

### Withdrawn / tolerated
- F-2: Tolerated — scan.rs fallthrough comment removed in restructure; old
  text misleading after restructuring to guard-based branch. Rationale in
  finding disposition.

All findings resolved. No REVs needed. Reconcile pass complete — handoff to
/close.
