# verify-vt reads UNATTRIBUTABLE for a file new to the current phase until it flips completed

`doctrine slice verify-vt` judges each VT by five ordered gates (`check_vt`,
`src/vtgate.rs`). Gate 4: if the VT's `test_file` is **not** in the slice's
`modified_files` set, the verdict is **UNATTRIBUTABLE** ("keyword present but
`<path>` not modified by this slice") — *before* keywords are even matched.

`modified_files` is built in the shell (`run_verify_vt`, `src/slice.rs`) from
`crate::state::read_source_deltas(root, id)` — the slice's **source-delta
registry**. That registry is populated per phase **only when the phase flips to
`completed`** (the conformance-boundary close: `slice phase --status completed`
captures `code_end_oid = HEAD` and records the `code_start..code_end` changed
files). Files touched by *earlier* completed phases are already in the set.

## The trap (SL-212 PHASE-04)

A file **first-touched in the current, still-`in_progress` phase** (e.g. PHASE-04
added `src/commands/guard.rs`, untouched by PHASE-01..03) is NOT yet in any
recorded source-delta. So its VT reads **UNATTRIBUTABLE mid-phase even after you
committed the code and the test passes** — the keyword IS present; the file just
isn't attributed to the slice yet. Do **not** chase this as a bug (wrong pointer,
missing keyword, uncommitted edit) — it is gate 4 working as designed.

## How to apply

- Expect UNATTRIBUTABLE for any VT whose `test_file` is new to the phase you're
  executing. It resolves to PASS automatically once you `slice phase <id>
  <PHASE-NN> --status completed` (the flip records the delta including that file).
- Sequence at phase close: commit all code → **flip completed** → *then* re-run
  `verify-vt`. Verifying before the flip under-reports newly-introduced files.
- A file touched by an earlier completed phase attributes immediately (its delta
  is already recorded) — this only bites files the current phase introduces.
- Contrast the two other UNATTRIBUTABLE-adjacent facts: gate 5 matches keywords
  **raw** over host source (POL-002, [[mem.pattern.gate.host-source-no-language-syntax]]);
  a genuinely wrong pointer (plumbing landed in a different seam) is corrected by
  editing the VT row's `test_file`, NOT by inlining code to satisfy the grep.
