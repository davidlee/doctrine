# Record-kind catalog: ~17 hardcoded prefix sites, not centralised

`src/knowledge.rs` §2 (and SL-159 design pass-1) claim the "add a record kind"
surface is centralised. **It is not.** `src/kinds.rs:37` defines the canonical
`RECORD: &[ASM, DEC, QUE, CON]` const, but ~17 sites **hardcode the prefix
literals** (`"ASM" | "DEC" | "QUE" | "CON"`) in match arms / array literals
instead of reading `RECORD`. Adding or renaming a kind means editing every one.

**Verified sites (2026-06-27, grep `"ASM".*"DEC".*"QUE"`):**

- `src/kinds.rs:37` — the canonical `RECORD` const (the one source of truth).
- `src/catalog/scan.rs:62` — `outbound_for` dispatch arm. **Panic-grade:** the
  fallthrough is `debug_assert!(false, "unrouted KINDS prefix")` (`:88`), so a
  `KINDS` row added in `integrity.rs` with **no matching scan arm panics every
  debug-build corpus scan.** Easiest site to miss; caught by no drift canary.
- `src/catalog/test_helpers.rs:119` — `seed_knowledge` prefix→dir map (test-only).
- `src/commands/dep_seq.rs` — `is_record` (`:29`), pin-test filters (`:267,:273`),
  admissible vector (`:285`), and a user-facing message string (`:83`).
- `src/priority/partition.rs:609` — record-row guard.
- `src/search.rs:33,:38` (+ `:25` full-corpus list) — searchable prefix groups.
- `src/tag.rs:17` — taggable prefix list.
- `src/integrity.rs:817` — prefix-collision check list.
- `src/relation.rs:1422,1427,1444,1445` (+ test pins `:1751,:1783`).

**How to work it:** treat "add/rename a record kind" as a **grep task, not a
checklist task**. Before close, grep the tree for `Constraint|"CON"|kinds::CON`
(rename) and every record literal cluster to zero. The drift canaries
(vocab/prefix-count/partition-cover) catch the *structured* sites; the literal
match-arm sites have **no canary** — only grep finds them. SL-159's site count
walked 13 → 18 → 20 across three review passes precisely because each pass found
another literal list.

**The fix** (deferred, see backlog): most of these should read `kinds::RECORD`
(or a membership predicate over it) instead of re-spelling the literals. DRY the
record-membership definition so adding a kind is one edit, not seventeen.

See [[mem_019eaf4518277951984cd6f48a392c4c]] (numbered-kind identity is scattered;
`integrity::KINDS` is the corpus-wide id table — the structural twin of this
prefix-literal scatter).
