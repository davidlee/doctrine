# write_atomic renames over a read-only target file

`fsutil::write_atomic` writes a sibling temp then `rename`s over the target. Unix
`rename(2)` keys on the **directory's** write permission, not the target file's
mode — so `write_atomic` **succeeds over a `0o444` target file** where bare
`std::fs::write` failed with `EACCES`.

Consequences:

- **Test gotcha.** To induce a write failure in a test, `chmod 0o555` the target's
  **parent directory**, not `0o444` the file. The old file-mode trick silently
  passes now. (SL-113 hit this in
  `spec.rs::spec_req_add_orphan_on_append_failure_left_uncommitted`; restore perms
  before the tempdir is reaped.)
- **Semantic.** Benign for doctrine: authored files are git-tracked `0644` and are
  never chmod-ed read-only (design SL-113 §5.5 E3). But `write_atomic` is **not** a
  drop-in for code that relied on target-file mode to gate a write.

Related: [[mem.pattern.entity.unified-read-seam-does-not-deliver-a-unified-write-seam]].
