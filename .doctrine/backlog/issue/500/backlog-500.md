# ISS-500: Draft memory expiry check fires on future review_by, never on past

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Found by RV-411 (CHR-172 test-quality review).

## Defect

`memory_health_findings` check 3 (`src/memory.rs`, "Draft expiry") computes
`days_between(&memory.review_by, today)`, which is `today - review_by`
(`src/retrieve.rs` `days_between`: "`b - a`"). It then fires on `days < 0` —
i.e. when `review_by` is in the **future**. The sign is inverted.

Observed (reproduced with `target/debug/doctrine` in a scratch project):

- draft, `review_by = 2020-01-01` → no finding;
- draft, `review_by = 2099-01-01` → `expired: draft past review_by 2099-01-01 (26392 days ago)`.

## Why tests missed it

The only test named for the check, `draft_expiry_validation_detects_past_review_by`,
exercises `retrieve::days_between` alone — never `memory_health_findings`.

## Fix

Flip the predicate (`days > 0`, message uses `days`), and table-test
`memory_health_findings` per check with a fixed `today` (past / today / future
`review_by`; draft vs active). Check 2 (stale verification on scoped paths)
has no test either — cover it in the same table.
