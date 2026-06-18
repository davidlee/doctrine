# Fold knowledge::set_record_status onto set_authored_status seam and reword F-1 refuse hints

> Scopes **IMP-061** and **IMP-066** — the two outstanding RV-024 findings from the
> SL-062 code review. Both are small, quality-focused, and touch the same status-setter
> code path.

## Context

**SL-062** shipped a unified lifecycle-transition write-core:
`dep_seq::set_authored_status` — one read→parse→core→write-once seam that
replaced byte-duplicated status-setter bodies in slice, backlog, governance, and
requirement. The code review (**RV-024**) found the unification was 4/5, not 5/5:

- **F-1 (IMP-061):** `knowledge::set_record_status` (`src/knowledge.rs:1346–1383`)
  remains a byte-duplicate of the very write body SL-062 exists to eliminate. It
  independently reads, parses, writes, and bakes its own malformed-refuse logic —
  all of which `set_authored_status` already does.

- **F-2 (IMP-066):** Three per-kind status setters still refuse with destructive
  "regenerate via `<kind> new`" guidance (`src/slice.rs:526`,
  `src/backlog.rs:1390`, `src/knowledge.rs:1375`), contradicting the
  non-destructive "restore the seeded keys — the file is left untouched"
  philosophy the shared `dep_seq` seam already applies. The SL-060 lesson flagged
  this explicitly; the divergence is a half-finished thought.

These are the last two loose ends from SL-062's code review. Both are small,
zero-risk, and confined to lines already identified in RV-024.

## Scope & Objectives

1. **Fold `knowledge::set_record_status` onto `dep_seq::set_authored_status`.**
   Replace the independent read/parse/write body in `src/knowledge.rs` (lines
   1346–1383) with a call to `dep_seq::set_authored_status`. The knowledge
   setter writes `[status, updated]` — the same `managed` shape as slice
   (already proven). The no-op guard (unchanged status → hold mtime) and the
   strict malformed-refuse must be behaviour-preserving. `run_status` in
   `knowledge.rs` is the sole caller; it already resolves the path — pipe it
   through.

2. **Reword the three "regenerate via" refuse hints (IMP-066).** Replace the
   destructive guidance in:
   - `src/knowledge.rs:1375` — `"regenerate via \`knowledge new\`"`
   - `src/slice.rs:526` — `"regenerate via \`slice new\`"`
   - `src/backlog.rs:1390` — `"regenerate via \`backlog new\`"`
   
   with the non-destructive pattern already used in `dep_seq.rs`: "restore the
   seeded keys — the file is left untouched" (or kind-appropriate equivalent
   that never suggests regenerating an authored entity).

3. **Behaviour preservation.** `knowledge::set_record_status` goes away; every
   test that exercises the knowledge status transition path must stay green
   unchanged. The three reworded messages must still carry the same information
   (which keys are missing, which file) but never suggest regeneration.

## Non-Goals

- **No new status setter unification beyond knowledge.** Slice, backlog,
  governance, and requirement already use `set_authored_status`. Only knowledge
  remains; no other kind has a bespoke setter.
- **No message reword beyond the three "regenerate via" sites.** The dep_seq
  message is already correct. Other malformed-refuse messages (revision,
  memory, requirement) are not in the "regenerate via" pattern and are out of
  scope.
- **No test-suite restructure.** Existing tests for `set_record_status` and
  the per-kind status transitions stay green with zero assertion changes (the
  code under test changes, but the behaviour must be identical).

## Affected Surface

- `src/knowledge.rs` — delete `set_record_status` (lines 1346–1383); route
  `run_status` through `dep_seq::set_authored_status`; reword the refuse
  message at line 1375 (will move/change when the fn is deleted — the reword
  applies to the dep_seq hint instead)
- `src/slice.rs` — reword the refuse message at line 526
- `src/backlog.rs` — reword the refuse message at line 1390
- `src/dep_seq.rs` — unchanged (the seam itself); may need to ensure the
  knowledge hint wording is passed through correctly

## Risks & Assumptions

- **R1 — behaviour preservation.** `set_record_status` has its own malformed
  check (seeded `status`/`updated` keys); `set_authored_status` uses
  `apply_status` which also checks for the managed keys. The refusal
  behaviour must be identical. Verified by existing knowledge status tests
  staying green.
- **R2 — no-op guard compatibility.** `set_record_status` guards on
  `status` string equality before the write; `set_authored_status` does the
  same via `apply_status`. Equivalent.
- **A1:** `run_status` in `knowledge.rs` resolves the full TOML path before
  calling `set_record_status`; the same path feeds `set_authored_status`.
- **A2:** No knowledge records exist in `.doctrine/knowledge/` (confirmed
  empty per SL-097 and SL-096); the change is test-proven only with tempdir
  fixtures.

## Verification / Closure Intent

- `cargo test knowledge::` — all existing knowledge status tests green
- `cargo test -- slice` — slice status tests green (message change only)
- `cargo test -- backlog` — backlog status tests green (message change only)
- `src/knowledge.rs` no longer contains a `set_record_status` fn (grep for it)
- All three "regenerate via" strings absent from the codebase
- `just gate` clean
- IMP-061 → resolved; IMP-066 → resolved

## Summary

Two RV-024 findings, one slice: delete the last byte-duplicate status-setter
(`knowledge::set_record_status` → `dep_seq::set_authored_status`) and finish
the F-1 refuse reword the code review flagged. Single-digit line changes,
behaviour-preserving, zero risk.

## Follow-Ups

- None — this closes the RV-024 F-1/F-2 tail. Remaining RV-024 findings
  (F-3 speculative IO wrapper, F-4 path-resolution copy, F-5 torn-state
  recovery test) are tracked separately (CHR-008 for F-5).
