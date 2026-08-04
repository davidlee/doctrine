`entity::claim_fresh_id` claims an id by **creating the directory**
(`LocalFs::claim` → `create_dir`), then runs the caller's `on_reserved` midpoint,
then builds and writes the fileset. A build failure — or an `Err` from the
midpoint — unwinds and the H2 cleanup removes the claimed dir.

**A hard exit does not.** `DOCTRINE_DESIGN_FAULT` simulates a crash with
`std::process::exit`, precisely so nothing unwinds. So a fault at `id-claim`,
`id-journal` or `record-materialise` leaves `NNN/` on disk containing **nothing**.
DEC-086 tolerates this deliberately — it is the "empty or partial reservation"
its ordering argument is written against — and the resumed submission then claims
the *next* id, because `scan_ids` counts any numeric dir.

Consequence for any test or tool asking "how many entities exist":

- a **directory** listing answers "how many ids were claimed", which after a
  crash is one more than the number of records;
- the **record file** (`review-NNN.toml`, `record-NNN.toml`, …) answers "how many
  entities exist", which is the question almost everyone means.

`tests/e2e_design_review.rs` keeps both, and the pair is deliberate:
`minted_reviews` lists numeric dirs, `authored_reviews` filters those to the ones
carrying a ledger. They differ **exactly** across DEC-086's tolerated window,
which is what lets one crash test assert "one RV authored" while another asserts
"one reservation consumed".

Distinct from [[mem.pattern.entity.slug-symlink-doubles-naive-walks]] — that miscount is
the `NNN-slug` alias symlink and `is_dir()` following it; this one is a real,
empty directory and no symlink filter catches it.
