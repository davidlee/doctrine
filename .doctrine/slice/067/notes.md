# SL-067 implementation notes

## Audit (RV-031)

Reconciliation + code-review audit complete. The implementation is
design-conformant — all four locked decisions (D1-D4), all five adversarial
findings (F1-F5), and every EN/EX/VT criterion hold. `cargo clippy` zero
warnings; 1286 tests pass.

### Findings (4, all terminal)

- **F-1 (blocker → verified):** `review/067` is a dispatch worktree amalgam —
  extraneous deletions of revision.rs, relation.rs Revises label, main.rs
  Revision command, all pre-067 slices/specs/reviews, 5 revision e2e tests.
  SL-067 code additions are correct; the branch must NOT be merged whole.
  Integrate only the SL-067 additions as conventional commits (see Integration
  path below).
- **F-2 (minor → retracted):** Premise was wrong — VT-4 has 5 dedicated
  table-header assertions covering all D2 cases + --kind filter. IMP-076 closed
  (resolved/wont-do).
- **F-3 (minor → verified):** `strip_ansi` was copy-pasted between backlog.rs
  and listing.rs (code review). Fix applied on main (6352b01):
  `listing::strip_ansi` is now `pub(crate)`. During integration, the backlog.rs
  SL-067 additions must remove the local `strip_ansi` copy and use
  `crate::listing::strip_ansi` instead. The single call site is in
  `backlog_list_tags_column_colour_strips_to_plain`.
- **F-4 (nit → tolerated):** `paint_tag` capacity hint (`with_capacity(tag.len())`)
  underestimates by ~3x; `segment_hue` unwrap_or silently swallows overflow on
  pathological palette sizes. Cosmetic — dead paths on the current 10-entry
  const palette.

Pattern recorded: `mem.pattern.dispatch.review-branch-extraneous-deletions`.

### Integration path (for /close)

Apply the SL-067 additions from `review/067` onto main as 5 conventional commits,
FILTERING OUT the extraneous dispatch worktree deletions. Do NOT merge the
review/067 branch whole.

1. `feat(SL-067): add BacklogCommand::Tag CLI wiring` — `src/main.rs`
   (clap variant + write_class + dispatch)
2. `feat(SL-067): producer — tag verb, normalisation, JSON projection` —
   `src/backlog.rs` (normalize_tag, fold_filter_tag, apply_tags, run_tag,
   BacklogRow.tags, json_rows, filter input fold, any_tagged/effective_default,
   PHASE-01 tests). **NB: remove the local `strip_ansi` copy; use
   `crate::listing::strip_ansi` (made pub(crate) in 6352b01).**
3. `feat(SL-067): reader/render — PerToken paint, tag chips, dynamic column` —
   `src/listing.rs` (ColumnPaint::PerToken, TAG_PALETTE, stable_hash,
   segment_hue, paint_tag, paint_cell PerToken arm, PHASE-02 tests)
4. `test(SL-067): update golden tests for unconditional tags projection` —
   `tests/e2e_backlog_list_order_golden.rs`,
   `tests/e2e_list_columns_golden.rs`
5. `chore(SL-067): hoist strip_ansi to pub(crate) in listing.rs` — already
   done (6352b01 on main); step 2 must import it instead of copying.

## Risks carried forward

1. **PerToken coupling is by convention only.** `split` and `cell` both read
   `tags` but nothing in the type system couples them. The F1 guard test
   (`strip_ansi(coloured) == plain == cell(r)`) pins it.
2. **Intra-array comments not preserved.** The sorted-array replace rebuilds the
   `tags` array, dropping inline comments. Accepted (A5) — tags are machine-written
   from a seeded `[]`.
3. **Empty colon-segments tolerated.** `:x`, `a::b` render coloured colons with
   no segment text. Charset permits colon positionally.
