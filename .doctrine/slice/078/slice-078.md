# Chore sweep: spec-010 rename + supersede recovery test

## Context

Two small backlog chores, each independent of its nominal `after` dependency
and file-disjoint from each other, bundled into one slice for efficiency.
CHR-005 (grid_min_width test) was assessed and closed as wont-do — the
existing VT-2 boundary test covers the real risk.

- **CHR-006** — Sweep SPEC-010 for the `skills install` → `claude install`
  rename. SL-056 PHASE-11 already renamed the CLI primary; the tech-spec
  (body, TOML responsibilities, and descendant requirement FR-005/REQ-177)
  still references the old names.
- **CHR-008** — E2e test for torn-state recovery in the supersede verb
  (`src/main.rs`). The existing `run_supersede` writes NEW then OLD in a
  specific order so a crash between the two writes leaves a detectable torn
  state (`NEW.supersedes∋OLD` without `OLD.superseded_by∋NEW`). Re-running
  naturally completes the write (no new code — `push_str_if_absent` is
  already idempotent). The test proves the recovery path.

## Scope & Objectives

1. **CHR-006**: Sweep `SPEC-010` (`.doctrine/spec/tech/010/`) — `.md` body,
   `spec-010.toml` responsibilities, and `FR-005` (REQ-177) title — for
   `skills install` → `claude install` / `skills list` → `claude list`.
   Text-only, no code changes.
2. **CHR-008**: Add an e2e test in `src/main.rs` that seeds ADR fixtures,
   simulates a torn supersede (NEW written, OLD not), re-runs `run_supersede`,
   and asserts full recovery (both files correct, OLD.status=superseded).

## Non-Goals

- CHR-001 (RV ledger robustness), CHR-007 (SL-057 VT backfill),
  CHR-009 (SL-068→SL-069 coordination) — remain in the backlog
- CHR-005 — closed wont-do
- IMP-025, RSK-008, IMP-051 — dependency chains assessed as non-blocking

## Summary

Two independent, file-disjoint changes: spec text sweep
(`.doctrine/spec/tech/010/`) + supersede recovery e2e test
(`src/main.rs`). Total effort ~1.5h.

## Follow-Ups

- CHR-001, CHR-007, CHR-009 remain in the backlog for future slices
