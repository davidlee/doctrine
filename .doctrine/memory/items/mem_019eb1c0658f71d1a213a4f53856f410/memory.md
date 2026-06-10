# New workspace member trips the cargo lint group + pedantic doc lints

Adding a fresh crate under `[lints] workspace = true` lights up lints the root
`doctrine` crate already satisfies, so they look like "new" failures:

- **`clippy::cargo` → `cargo_common_metadata`**: requires `repository`, `readme`,
  `keywords`, `categories` in `[package]` **and** a `README.md` file. NOTE:
  `repository.workspace = true` does NOT work here — the root defines `repository`
  in `[package]`, not `[workspace.package]`, so inheritance errors. Set it
  literally in the member manifest.
- **`clippy::pedantic` doc lints**: `missing_errors_doc` (every `pub fn -> Result`
  needs a `# Errors` doc section), `doc_markdown` (backtick type-ish words like
  `BTreeMap`). The workspace pauses `missing_docs`/`missing_errors_doc` explicitly,
  but `pedantic = deny` re-implies `missing_errors_doc` — it fires anyway.
- **`trivially_copy_pass_by_ref`**: accessors on a ≤8-byte `Copy` type must take
  `self` by value, not `&self` (e.g. a 2-byte two-enum config). Larger Copy types
  (>8 bytes) keep `&self`.
- **`single_match`-style steer**: a two-arm `match` over `try_from`/`checked_add`
  whose `Err(_)`/`None` arm just sets a flag + returns reads as if-let-else →
  clippy wants `let … else`.

Checklist for the next `crates/<name>`: literal `repository`/`readme`/`keywords`/
`categories` + a README.md, `# Errors` on fallible pub fns, by-value accessors on
small Copy types. Lint as you go — plain `cargo clippy` (the gate; never
`--all-targets`, see [[mem.pattern.lint.clippy-denies]]).

Related: [[mem.pattern.lint.disallowed-types-collections]] (BTree not Hash),
[[mem.pattern.lint.as-conversions-ban]] (use `try_from`, not `as`),
[[mem.pattern.lint.expect-not-allow]].
