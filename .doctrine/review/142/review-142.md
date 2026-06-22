# Review RV-142 — reconciliation of SL-141

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Post-implementation conformance audit of SL-141 (entity full-text search).
Three phases implemented via dispatch workers on 3 isolated worktrees.
Review surface: `refs/heads/review/141` (single squashed commit `32f2d030`).

### Lines of attack

1. **Design conformance** — does the implementation match `design.md`?
   - CLI flags: `--kinds`/`--with`/`--no`/`--format`/`--context`/`--limit`/`--offset`
   - JSON output shape, snippet algorithm, pagination
   - KindSelector: default set, group aliases, additive/subtractive logic

2. **Plan exit criteria** — does it meet the 19 EX- criteria from `plan.toml`?

3. **ScanMode correctness** — bodyless-by-default preserved; all callers updated.

4. **Behaviour preservation** — existing catalog/lexical tests pass unchanged.

5. **Property test** — plan VT-3 specifies 100+ random strings proving
   tokenize(tokenize_with_spans) equivalence; is this a property test or a
   table-driven test?

6. **Pagination validation** — `--limit 0` rejected, `--limit` capped at 100.

7. **Missing flags** — `--json` shorthand, `--page` pagination from design.

8. **Snippet algorithm fidelity** — context window size, off-by-one in ellipsis,
   byte-offset correctness.

9. **JSON output shape** — matches design schema (id, kind, score, title etc.).

## Synthesis

The SL-141 implementation is clean and thorough. All three phases landed
correctly on `refs/heads/review/141` via dispatch workers. The implementation
matches the design in all substantive ways:

- **tokenize_with_spans** (PHASE-01): clean extraction of the span authority
  from tokenize. Behaviour-preservation gate holds — all existing lexical
  tests pass unchanged. The table-driven equivalence test covers the important
  edge cases (empty, Unicode, mixedCase, separators, etc.).

- **Body ingestion** (PHASE-02): ScanMode correctly gates body reading.
  ~22 call sites mechanically updated to `ScanMode::default()`. The Rust
  type checker guarantees completeness. Missing-file→None, error→diagnostic+None
  policy implemented. serde skip preserves JSON contract.

- **Search module** (PHASE-03): KindSelector with additive/subtractive logic,
  group aliases, and unknown-prefix error. BM25 orchestration reuses existing
  `Bm25Ranker` and `LexicalCorpus` correctly. Snippet extraction, table and
  JSON formatters, CLI wiring — all functional. 18 unit tests pass.

### Findings summary (5 total)

- **F-1 (minor, tolerated)**: Property test vs table-driven — equivalence is
  tested, only the form differs. Tolerated; the curated edge cases are adequate.
- **F-2 (minor, tolerated)**: Missing --json, --page, --limit validation —
  plan intentionally scoped these out, creating a design-plan gap.
  Reconciliation should trim the design spec.
- **F-3 (nit, tolerated)**: Snippet context 40 vs 80 chars — minor UX tuning.
- **F-4 (nit, tolerated)**: JSON `kind` field uses prefix not label — extra
  fields compensate; design example is illustrative.
- **F-5 (minor, fix-now → verified)**: Snippet ellipsis off-by-one —
  `window_start + 1` skipped first byte of context window. Fixed on review/141.

### Standing risks

- **RSK-001 (disk I/O)**: Mitigated — ScanMode gates body reading; only search
  pays the cost.
- **RSK-002 (memory)**: Accepted — entity corpora are small.
- **RSK-003 (template noise)**: Deferred to follow-up — not blocking.

### Tradeoffs accepted

1. Command SCC baseline bumped from 120 to 123 instead of pre-cleanup
   (design mitigation option B). Acceptable for a new module.
2. Knowledge records (ASM, DEC, QUE, CON) included in default search kinds,
   following the design (not the plan which excluded them). Design is canonical.
3. Snippet context chars hardcoded at 40 vs design's 80 — minor.
4. JSON output carries extra `total`, `prefix`, `status` fields — additive,
   not breaking.

## Reconciliation Brief

### Per-slice (direct edit)
- **None required** — all code findings resolved (F-5 fixed).

### Governance/spec (REV)
- **design.md CLI surface (F-2)**: Remove `--json`, `--page` from the
  design's Options table and pagination section. The plan intentionally
  scoped these out; the design should reflect the implemented surface.
  Also add `--kinds` as the flag name (design said `--kind`; plan resolved
  to `--kinds`).
- **design.md context_chars (F-3)**: Update default context_chars from 80
  to 40 to match implementation.
- **design.md JSON kind field (F-4)**: Either update the example to show
  `"kind": "sl"` (lowercase prefix), or add a note that the kind value is
  the lowercase prefix, not the full kind label.

### Follow-up work (backlog)
- Consider adding a property test for tokenize equivalence as a formal
  correctness proof beyond the table-driven approach. (Impact: low.)

## Reconciliation Outcome

### Direct edits applied
- **design.md flag naming (F-2)**: `--kind` → `--kinds` in flag table, CLI
  Options section, and evaluation order description.
- **design.md CLI surface (F-2)**: Removed `--json`, `--short`, `--page`
  options from the CLI spec. Pagination section simplified to match
  implemented surface (--limit/--offset only).
- **design.md JSON section (F-4)**: Removed `--short` reference.
- **design.md integration tests (F-2, F-4)**: Removed tests referencing
  `--json`, `--short`, `--page`, `--limit 0` — narrowed to the 4 that
  match the implemented surface.
- **design.md design decisions (F-2)**: Removed D6 (`--short` conflicts
  with `--context`).

### REVs completed
- None required — all edits were per-slice artefacts (design.md), not
  governance/spec.

### No-op findings
- **F-3 (context 40 vs 80)**: Design already showed `saturating_sub(40)`
  matching implementation. No edit needed. Tolerated as-is.

### Tolerated findings
- **F-1 (property test form)**: Table-driven test covers equivalence
  adequately for the tokenizer's simplicity. Tolerated.
- **F-4 (JSON kind value)**: `kind` field uses lowercase prefix ("sl")
  not kind label ("slice"). Design example updated to match. The extra
  `total`, `prefix`, `status` fields are additive improvements.

All reconciliation items resolved. Handoff to /close.
