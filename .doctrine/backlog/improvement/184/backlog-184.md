# IMP-184: DRY record-kind membership: ~17 sites hardcode prefix literals instead of reading kinds::RECORD

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

`src/kinds.rs:37` defines `RECORD: &[ASM, DEC, QUE, CON]` as the canonical
record-family membership, but ~17 sites across the tree **re-spell the prefix
literals** (`"ASM" | "DEC" | "QUE" | "CON"`) in match arms and array literals
instead of reading `RECORD`. Adding or renaming a record kind is therefore a
~17-site grep-and-edit with **no drift canary** on the literal sites — they are
invisible to the vocab/prefix-count/partition-cover canaries that guard the
structured surface.

Surfaced by SL-159 (EVD/HYP add + CON→INV rename): the touch-set count walked
13 → 18 → 20 across three review passes because each pass found another literal
list the "centralised checklist" had missed — including `src/catalog/scan.rs:62`,
whose `debug_assert!(false)` fallthrough makes an omission a **debug-build panic**,
not a silent gap.

See `mem.pattern.doctrine.record-kind-touch-sites` for the full verified site list.

## Scope to investigate

Whether the literal sites can read a single membership source. Candidates:

- a `kinds::is_record(prefix: &str) -> bool` predicate over `RECORD`, replacing the
  scattered `matches!(prefix, "ASM"|"DEC"|"QUE"|"CON")` arms.
- the dispatch sites (`catalog/scan.rs`, `dep_seq.rs`) and the
  search/tag/partition/integrity prefix-group lists.
- which sites genuinely need a *different* subset (e.g. search's full-corpus list
  vs the record subset) and so should NOT collapse to `RECORD`.

Likely overlaps the **numbered-kind identity scatter** (`integrity::KINDS` as the
corpus-wide id table) — the structural twin. Consider treating both together; a
single kind-registry seam could subsume both the prefix-literal scatter and the
identity-table scatter. Do not pre-build a grand registry without confirming the
real touch-sites collapse cleanly (`mem_019eaf93b6...` warns the dedup was
deliberately deferred once before).

## Acceptance intent

Adding a record kind edits **one** membership definition, not seventeen; the
literal match-arm sites gain canary coverage or are eliminated. Behaviour
preserved (existing suites green).
