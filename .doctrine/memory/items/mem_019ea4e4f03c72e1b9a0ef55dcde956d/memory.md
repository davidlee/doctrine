# Edit-preserving authored-TOML status transition

The house seam for a verb that flips one authored entity's `status` (and kin)
in place without reserialising the file. Reuse it for the next entity `edit`
verb — do **not** re-roll a parse→serialise round-trip (that drops comments and
reorders keys).

## The recipe

`fn set_<entity>_status(root, id, status, …, today: &str) -> Result<…>`:

1. Read the `<entity>-NNN.toml`; `text.parse::<toml_edit::DocumentMut>()`.
   Mutating the `DocumentMut` preserves inert tables (`[relationships]`/`[facet]`),
   hand-added comments, and unknown keys verbatim — the file is never reserialised.
2. **No-op guard** (I5): if the current value(s) already equal the target, return
   without writing — content + mtime hold. Place it *before* the F-1 check.
3. **F-1 refuse**: the keys this verb sets are scaffold-seeded (e.g. `status`,
   `resolution`, `updated`). If any is absent the file is malformed (hand-edited)
   — `bail!`, never `table.insert`. A tail-`insert` appends the key *after* the
   trailing subtable header, landing it **inside** that subtable = silent
   corruption. Refuse → regenerate via `<entity> new`.
4. `table.insert(key, toml_edit::value(…))` for each managed key, then a single
   `fs::write(path, doc.to_string())`. The date is **injected by the shell**
   (`clock::today()`), never read in the pure/edit layer.

## Precedent

- `src/adr.rs::set_adr_status` — the original (sets `status` + `updated`).
- `src/backlog.rs::set_backlog_status` — mirrors it, adds a *coupled* `resolution`
  key (returns the written string so the shell prints the post-transition state;
  the coupling/D9 clear is the separate pure `validate_transition`).

Lints: this stays under the repo clippy denies — see [[mem.pattern.lint.clippy-denies]].
