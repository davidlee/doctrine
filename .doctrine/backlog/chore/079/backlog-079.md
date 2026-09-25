# CHR-079: Refresh install/review-ledger.md to the current verb surface; add a doc-to-help consistency check

`install/review-ledger.md` is the machine-copied protocol doc every client project
follows, and it has drifted from the verb surface (the substantive mismatch was
introduced by SL-147, 2026-06-24, and survived every later edit):

- §2 documents `review prime --seed`, `--from <file>` and a reviewer-authored
  `domain_map` — none exist; `PrimeArgs` is `{ reference }` and prime derives the
  path-set from the target slice's selectors (`src/review.rs:2971`).
- `review conclude`, `review unlock`, `review paths` are absent from the doc but
  present in `review --help`.
- §6's parent-tree caveat overstates the guard: `resolve_review_root` refuses only
  a `fork`, not every linked worktree (obs `01a0bc8d-e4af`, `019fb17f`).
- §4's disposition vocabulary is unenforced prose (RFC-026 E1).

Fix: refresh the doc, and add a cheap consistency check between the verbs/flags it
names and `review --help` so drift is caught rather than discovered.
See RFC-032 `research.md` F5/F7/F13.
