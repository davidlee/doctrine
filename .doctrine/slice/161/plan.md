# Implementation Plan SL-161: DRY kind-registry seam

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML.

## Overview

Two phases. PHASE-01 covers the 7 straightforward conversions — everything where
a literal record-prefix cluster becomes `kinds::is_record()`, `kinds::RECORD`, or
`RecordKind::from_prefix`. PHASE-02 handles the two trickiest changes — the
`scan.rs` dispatch restructure and the KINDS count assertion — then runs the full
gate.

## Sequencing & Rationale

**PHASE-01 first, PHASE-02 second.** The split isolates the `scan.rs` restructure
from the mechanical conversions. If the restructure has issues (dispatch order,
fallthrough panic), PHASE-01 can land independently — it's 7 files of pure
mechanical edits with no control-flow changes.

**Why `scan.rs` is last.** The `outbound_for` dispatch is the panic-grade site —
`debug_assert!(false)` fallthrough — and the only site where control flow changes
(match arm → guard). Keeping it in its own phase means it gets focused attention
and a clean revert surface if needed.

**KINDS rides PHASE-02.** The count assertion is a one-line addition to an
existing test. Bundling it with `scan.rs` keeps PHASE-01 purely mechanical.

**No phase for `supersede.rs`/`superserde.rs`.** Already DRY — pre-existing
condition verified at plan time (design OQ-3), no code change needed.

## Notes

- The design follows the existing backlog convention (two-source pattern, no
  coherency test). No new architectural patterns introduced.
- Post-PHASE-02 grep for record-literal clusters is the drift canary — zero
  outside `kinds.rs` and `dep_seq.rs:285` (admissible vector — mixed superset,
  accepted as-is per design F2).
- SL-159 (EVD+HYP) benefits directly: adding kinds becomes a `kinds::RECORD`
  edit, not a 12-site grep.
- Pre-existing condition verified at plan time: `supersede.rs` and
  `superserde.rs` already use `RecordKind::from_prefix(prefix).is_some()` — no
  literal record-prefix clusters, no change needed (design OQ-3).
