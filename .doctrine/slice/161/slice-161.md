# DRY kind-registry seam: record-membership predicate + numbered-kind identity table

## Context

Doctrine's kind-identity surface has two scattered-identity problems that share a
root cause — membership is hand-maintained at every consumption site instead of
reading from a single source:

**Problem A — record-kind prefix literals (~14 sites).**
`kinds::RECORD = &[ASM, DEC, QUE, CON]` exists as the canonical membership, but
~14 sites hardcode `"ASM" | "DEC" | "QUE" | "CON"` in match arms and array
literals instead of reading `RECORD`. Adding or renaming a record kind is a
~14-site grep-and-edit with **no drift canary** on the literal match-arm sites.
The highest-severity site is `src/catalog/scan.rs:62` — the `outbound_for`
dispatch arm; its `debug_assert!(false)` fallthrough panics every debug-build
corpus scan when a KINDS row has no matching scan arm. See
`mem.pattern.doctrine.record-kind-touch-sites` for the full verified site list.

**Problem B — numbered-kind identity table (advisory membership).**
`integrity::KINDS` is the corpus-wide id table (23 rows, `KindRef` referencing
view over per-module `entity::Kind` consts). Membership is hand-maintained: add a
Kind const in a module → no compile-time gate forces adding its `KindRef` row.
The `kinds_table_*` unit test is a literal prefix pin (asserts the current set),
not a set-equality guard. SL-031 audit F-7 confirmed Rust's lack of const
reflection means this can't be closed with a test alone — it needs a
macro/inventory pattern. See `mem.pattern.entity.numbered-kind-identity-table`.

These are structural twins: both scatter identity across literal sites instead of
reading from a single registry. A shared kind-registry seam subsumes both.

## Scope & Objectives

1. **Predicate over `RECORD`.** Add `kinds::is_record(prefix: &str) -> bool` (or
   `is_record(kind: &Kind) -> bool`) that reads `kinds::RECORD`. Replace all ~14
   literal `"ASM"|"DEC"|"QUE"|"CON"` sites with this predicate or direct
   iteration over `RECORD`.

2. **Restructure `scan.rs:62` dispatch.** The `outbound_for` match arm can't use
   a runtime predicate directly (Rust match arms need literal patterns).
   Restructure to a guard-based branch — `if kinds::RECORD.contains(&prefix)` —
   preserving the fallthrough panic for truly unknown kinds.

3. **Macro-generated KINDS registry.** Replace the hand-maintained `integrity::KINDS`
   array with a `numbered_kinds_registry!` macro invocation that takes
   `(kind_const, stem, state_dir?)` tuples and generates both the `KINDS` array
   and its test pin from one input list. Adding a numbered kind becomes: define
   the Kind const in its module, add one line to the macro invocation.
   **Membership is still hand-maintained** (Rust const-reflection constraint),
   but **centralized** — the test pin is derived, not duplicated.

4. **Behaviour-preservation gate.** Existing test suites stay green. Pin tests
   that hardcode record/kind lists are updated to derive from the registry (not
   deleted — their coverage remains, just the source of truth moves).

## Non-Goals

- **Adding, renaming, or removing kinds.** That's SL-159 (EVD+HYP additions) and
  the separate CON→INV rename. This slice only DRYs the membership definitions
  that those slices will edit.

- **Other prefix groups** (GOV, BACKLOG). Those are smaller (3 + 5 sites) and
  mechanically simpler — `GOV` and `BACKLOG` are already const arrays consumed
  directly. Out of scope to keep this slice focused.

- **Macro-based Kind const generation.** The `entity::Kind` consts stay defined
  in their owning modules (`slice.rs`, `adr.rs`, etc.). The KINDS registry macro
  generates only the KINDS array + its test, not the Kind consts themselves.

- **Compile-time exhaustive KINDS membership.** Rust can't reflect over all Kind
  consts. The macro centralizes membership, but a missing row is still a runtime
  gap (caught by the test pin, not the compiler). A compile-time guard would
  require an inventory macro that *also* generates the Kind consts — a larger
  refactor, out of scope here.

- **CON→INV rename.** Pulled from SL-159. This slice works with the current
  4-kind RECORD (`ASM, DEC, QUE, CON`).

## Affected Surface

Selectors in `slice-161.toml` (15 scope-relevant). Key sites:

- `src/kinds.rs` — add `is_record()` predicate; RECORD const (exists, unchanged)
- `src/integrity.rs` — KINDS macro + test pin rewrite
- `src/commands/dep_seq.rs` — `is_record()` switches from literal match to
  `kinds::RECORD`; pin test updated
- `src/priority/partition.rs` — `:609` record-row guard reads `is_record()`
- `src/search.rs` — `:33` knowledge group reads `kinds::RECORD`
- `src/tag.rs` — `:17` taggable list derives record subset from `kinds::RECORD`
- `src/relation.rs` — `:1422,1427,1444,1445` hardcoded vectors iterate `RECORD`
- `src/catalog/scan.rs` — `:62` dispatch restructured (guard-based branch)
- `src/catalog/test_helpers.rs` — `:119` prefix→dir map iterates `RECORD`
- `src/supersede.rs`, `src/commands/supersede.rs`, `src/commands/superserde.rs` —
  `is_record` calls already route through `RecordKind::from_prefix`; verify they
  don't need change
- `src/knowledge.rs`, `src/relation_graph.rs` — verify no latent literal lists

## Risks / Assumptions / Open Questions

- **R1 — `scan.rs` restructuring changes dispatch order.**
  *Mitigation:* the guard-based branch fires only when no non-record arm matched;
  behaviour is identical for the record family. Existing scan tests are the gate.

- **R2 — KINDS macro breaks `integrity.rs` compilation for other modules.**
  *Mitigation:* the macro lives in `integrity.rs` (or a small `src/registry.rs`
  leaf module). The public API (`KINDS`, `kind_by_prefix`) is unchanged — only
  the internal construction changes. `cargo check --workspace` gate.

- **R3 — the `is_record` predicate is added to `kinds.rs` (leaf tier).**
  `kinds.rs` already carries `RECORD`, `GOV`, `BACKLOG` — a predicate over
  `RECORD` is a leaf-tier addition, respecting ADR-001.

- **OQ-1 — KINDS macro location.** **Resolved:** no macro. Count assertion in
  existing test instead. Two edits in one file.

- **OQ-2 — `RecordKind::from_prefix` dual-source.** **Resolved:** follow backlog
  convention — two sources (`kinds::RECORD` for grouping, `RecordKind::ALL` for
  enum dispatch), no coherency test. Both are "the definition."

- **OQ-3 — supersede/superserde `is_record` calls.** **Resolved:** already use
  `RecordKind::from_prefix(prefix).is_some()`. No change. Verified.

## Verification / Closure Intent

- `just gate` green (zero warnings)
- Existing record suites green (behaviour-preservation gate)
- `cargo test` — all pin tests updated to derive from registry, not hand-copied
- Grep for `"ASM".*"DEC".*"QUE".*"CON"` literal clusters in `src/` → zero
  (except `kinds.rs` RECORD definition + test)
- `integrity::KINDS` constructed via macro; test pin auto-derived from same
  macro input
- `scan.rs` dispatch: record family still routes to `knowledge::relation_edges`;
  fallthrough panic preserved for unknown kinds

## Follow-Ups

- **SL-159** — after this lands, SL-159's design gets a pass to use the DRY
  membership: adding EVD+HYP edits `kinds::RECORD` + `numbered_kinds_registry!`
  (2 edits, not 17).
- **CON→INV rename** (pulled from SL-159) — edits the same two registry sites.
- **GOV/BACKLOG groups** — smaller mechanical DRY, trivial after this pattern is
  established.
