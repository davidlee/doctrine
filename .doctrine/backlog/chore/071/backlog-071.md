# CHR-071: Collapse catalog::scan's private kref_for onto the shared test_support helper

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

SL-238 PHASE-04 `D-1` promoted the generic entity-seeding test helpers
(`seed_toml`, `seed_status_bearing`, `seed_status_less`, `kref_for`) out of
`src/authored_status.rs`'s private `mod tests` into a `pub(crate) mod
test_support` beside the reader whose input they author, and collapsed
`src/backlog.rs`'s slice-hardcoded `seed_slice_entity` into it.

One copy was left uncollapsed: `src/catalog/scan.rs`'s private `kref_for`.

## Why it was left

SL-238 PHASE-01 `EX-9` required the `search` / `map` / `catalog` suites green
**unmodified**, and re-pointing that helper's import is exactly the churn that
makes "unmodified" ambiguous at audit. The slice's notes proposed folding it in
during PHASE-08 "when that file is open for other reasons" — but PHASE-08 opens
`src/backlog.rs`, `src/commands/dep_seq.rs` and `src/commands/cli.rs` and
nothing else, so that moment never came. `EX-9` is discharged now, so the
constraint is gone.

## Disposition

Four duplicated lines with no correctness impact. Do it when `catalog/scan.rs`
is next open for another reason; it does not earn a visit of its own.

Raised at SL-238 audit (`RV-363` `F-5`) — the reasoning existed in
`notes.md ### Open` and only the capture was missing.
