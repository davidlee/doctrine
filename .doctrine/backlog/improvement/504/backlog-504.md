# IMP-504: Brittle goldens and prose-coupled test assertions

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

RV-411 F-14, F-16 (`doctrine show RV-411`).
- F-14: e2e_review_golden holds about 23 full copies of the ledger template.
  Its expected values were captured by pasting output ("bugs pinned as-is").
  It also pins clap usage lines, OS error text, `ParseIntError` text and
  pretty-printed JSON. Keep one full golden. Assert each step's change parsed
  as TOML, and compare JSON parsed.
- F-16: boot.rs `contains()` checks on phrases from the shipped docs. The file
  already compares sections for equality against the embedded asset; use that
  form everywhere.

Principle: a golden must say which behaviour it pins, and a copy edit to
shipped prose must not fail a unit test.
