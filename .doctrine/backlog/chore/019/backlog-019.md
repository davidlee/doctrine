# CHR-019: Spike — toml_edit 0.22 root-tag strip+insert vs real worst-case corpus shapes (SL-136 F1 / RV-129)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Why

De-risks the load-bearing premise behind SL-136 D4, raised as **RV-129 F-1**
(blocker). The original proof was a throwaway `/tmp/tomlprobe` that was never
committed (`handover.md:69`) — unrepeatable, and tested only simplified synthetic
shapes. This spike pulls that proof **in-tree, against real corpus shapes**, before
the `apply_tags_set` leaf is written. Output feeds the design reopen's disposition
of F-1.

## Hypotheses under test

1. **Root insert is safe.** `doc.as_table_mut().insert("tags", …)` on a real
   governance entity lands the key **above** all trailing subtables / `[[relation]]`
   array-of-tables, and the file re-parses to the intended structure.
2. **Strip-then-insert relocation is safe.** The real op is not insert-into-clean —
   it is *remove the typed `tags` line from `[relationships]`, then insert root
   `tags`*. The interaction (strip + insert + value preservation) must round-trip.
3. **The contradiction resolves one way or the other.** Both `apply_status`
   (`dep_seq.rs:282-285`) and `apply_tags` (`backlog.rs:1921-1924`) currently
   *refuse* to insert a missing key, citing tail-insert corruption. Either that
   premise is false (then F-1's status/tags split is moot — explain why both can
   safely insert) or it is true (then D4 is unsafe and must be redesigned). The
   spike must produce evidence that settles this, not assert around it.

## Method

In-tree throwaway test (delete after evidence captured, or keep as VT-2 seed):
load each fixture below as a real on-disk `.toml`, run strip+insert, assert
post-state by **re-parsing** (not string match) — `root.tags` present with expected
values, every pre-existing trailing table/relation/comment intact and correctly
positioned. Run on the **pinned** `toml_edit 0.22.27` (Cargo.lock).

## Fixtures (real worst-case shapes — none were in the original probe)

- **SL-118** — `[[relation]]` followed by a *named* subtable `[estimate]`
  (AoT-then-named-subtable).
- **spec-tech (SPEC-016 / spec-005)** — root `tags` already present, followed by
  `[[source]]` AoT.
- **RFC-002** — 16× `[[relation]]` blocks **and** the only non-empty live tag set
  (`[program, consumption-surfaces, estimate, value, scoring]`) that must survive
  the strip+insert.
- **SL-048** — free comment block *after* the last `[[relation]]` (post-AoT
  comment).
- **ADR-014 / POL-001** — carry both root `status` and `[relationships].tags`
  (the same-file overlap that breaks the "disjoint seams" framing).

## Done when

- Each hypothesis is answered with re-parse evidence (pass/fail per fixture).
- The status/tags contradiction is explicitly resolved (premise true or false,
  with the reason).
- Result is recorded on **RV-129 F-1** (via `review dispose`) so the design reopen
  can disposition with evidence.
- If safe: the test graduates to SL-136 PHASE-01's first red (VT-2 seed). If
  unsafe: D4 returns to `/design`.

## Links

- RV-129 F-1 (blocker) — the finding this spike answers.
- SL-136 design.md §5.2 (`apply_tags_set`), §5.5 (probe claim), D4.
