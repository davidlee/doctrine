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

### Integration path — LANDED

The SL-067 additions were integrated onto main as 4 clean conventional commits,
filtering OUT the extraneous dispatch-worktree deletions (the `review/067` branch
forked a pre-revision base, so a whole-branch diff/merge would have reverted the
REV kind, `is_work_like`'s SL-066 REV widening, and all pre-067 entities). main.rs
took only its 3 SL-067 hunks by hand (Tag variant / write_class / dispatch),
NEVER the 190 revision-command deletions; backlog.rs / listing.rs / the goldens
were clean (`git checkout review/067 -- <file>` = main + SL-067) and taken whole.

- `31b3fc0 feat(SL-067): reader/render — PerToken paint, tag chips, dynamic column`
- `83dad91 feat(SL-067): producer — tag verb, normalisation, JSON projection`
- `7843921 feat(SL-067): add BacklogCommand::Tag CLI wiring`
- `93ef823 test(SL-067): update golden tests for unconditional tags projection`

Prior `61a843e` (F-3) made listing's `strip_ansi` pub(crate). At integration it
was moved OUT of `mod tests` to a `#[cfg(test)] pub(crate)` module-level helper so
the backlog tags-colour proof reaches it as `crate::listing::strip_ansi` (a
private `mod tests` blocks the nested path); backlog's local copy removed.

Gate at close: `cargo build` + `cargo clippy` clean; 1310 bin tests pass
(SL-067 tag/paint_tag/strip_ansi-coupling suites green); both e2e goldens green.

## Risks carried forward

1. **PerToken coupling is by convention only.** `split` and `cell` both read
   `tags` but nothing in the type system couples them. The F1 guard test
   (`strip_ansi(coloured) == plain == cell(r)`) pins it.
2. **Intra-array comments not preserved.** The sorted-array replace rebuilds the
   `tags` array, dropping inline comments. Accepted (A5) — tags are machine-written
   from a seeded `[]`.
3. **Empty colon-segments tolerated.** `:x`, `a::b` render coloured colons with
   no segment text. Charset permits colon positionally.
