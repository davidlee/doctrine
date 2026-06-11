# cordage denylist whole-word matches forbidden product vocab

`crates/cordage/tests/denylist.rs` enforces SPEC-001 Appendix B (forbidden-core
list): cordage must stay product-neutral. The check is a **whole-word** match over
every identifier and doc-comment in the crate — so a substring inside another word
is fine, but a standalone product noun fails the suite.

- Banned standalone words include `project`, `task`, `schedule`, `capacity` (read
  the current list from `tests/denylist.rs` — it is the source of truth).
- It bit SL-043 PHASE-02: `project_flag` / "projects" tripped it → renamed to
  `member_value` / "restricts". Use graph-neutral vocab: `cone`, `member`,
  `predecessor`, `overlay`, `value`, `witness`.
- **Do X:** before committing cordage source, grep your new identifiers + doc
  comments for the banned words, or just run `cargo test -p cordage` (the denylist
  suite is part of it) — clippy alone will NOT catch this.

Related: [[mem.pattern.cordage.opaque-ids-capture-from-builder]] (the other cordage
test footgun — opaque ids).
