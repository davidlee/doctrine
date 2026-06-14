# Review RV-031 — reconciliation of SL-067

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reconciliation audit of SL-067 (Tags command surface: backlog beachhead) against
its design, plan, and the `review/067` implementation branch. Lines of attack:

1. **Design conformance** — Do the four locked decisions (D1 PerToken colour,
   D2 dynamic visibility, D3 verb shape, D4 single chokepoint normalisation)
   match the implementation byte-for-byte? Are the integrated adversarial
   findings (F1-F5) resolved?
2. **Phase criteria** — Does every EN/EX/VT in `plan.toml` hold? Are the
   behaviour-preservation gates (SL-053 VT-2, existing goldens) green?
3. **Code quality** — Is the implementation DRY? Does it ride existing seams
   (edit-preserving recipe, `BL_COLUMNS`/`select_columns`, `status_hue` palette
   precedent) or introduce parallel paths? Does the pure/imperative split hold?
4. **Branch hygiene** — `review/067` is a dispatch worktree amalgam; the
   integration to trunk at `/close` must filter out extraneous deletions
   (revision.rs, relation.rs Revises label, all pre-067 slices/specs/reviews).

Invariants this audit must hold the implementation to:
- F1: `strip_ansi(coloured) == plain == cell(r)` — PerToken byte-clean coupling
- Set-compare no-op guard (unsorted hand store must not write+stamp)
- `any_tagged` keys on visible rows (post-retain AND post-`--kind`)
- Palette excludes Red, BrightRed, Black, White
- Two divergent folds: `normalize_tag` (strict, bail) vs `fold_filter_tag` (lenient, silent)

## Synthesis

### Verdict

The SL-067 implementation on `review/067` is **design-conformant and ready for
integration**. All four locked decisions (D1-D4), all five adversarial findings
(F1-F5), and every phase EN/EX/VT criterion hold. The code quality is high —
it rides existing seams (edit-preserving recipe, `BL_COLUMNS`/`select_columns`,
`status_hue` palette precedent), respects the pure/imperative split, and passes
`cargo clippy` with zero warnings. The single blocker (F-1) concerns branch
hygiene, not implementation correctness — the review/067 branch carries
extraneous deletions from its dispatch worktree partial tree that must be
filtered at integration time.

### Design conformance walkthrough

- **D1 (PerToken colour).** `ColumnPaint::PerToken { split, render }` added to
  `listing.rs`. `paint_cell` handles it via an early return before the hue match,
  reached only under `color == true`. `paint_tag` renders colon-segment chips:
  segments hued by the deterministic `segment_hue` (FNV-1a byte fold → palette
  index), colons painted white. The 10-entry `TAG_PALETTE` excludes Red,
  BrightRed, Black, and White. `paint_tag` is unconditional ANSI — the colour
  gate is `paint_cell`'s `color` bool.
- **D2 (dynamic visibility).** `any_tagged` is computed on the final displayed
  set — after both `listing::retain` and the `--kind` filter — and once, so
  column layout is uniform across `--by id` blocks. `BL_DEFAULT` const is
  unchanged (4 columns); `effective_default` is built locally, splicing `"tags"`
  before `"title"` iff `any_tagged`. An explicit `--columns` override is
  honoured verbatim.
- **D3 (verb shape).** Single `backlog tag <ID> [TAGS]... [--remove/-d
  <TAGS>...]` — one atomic edit-preserving write. At least one add or remove
  required (shell-enforced). `-d` short flag is free within the BacklogCommand
  namespace (only `--domain` on `claude install` uses it elsewhere).
- **D4 (normalisation).** `normalize_tag` is the single write chokepoint: trim →
  lowercase → charset `[a-z0-9_:-]`, non-empty → bail naming the token.
  `fold_filter_tag` is the separate lenient filter fold: trim + lowercase, no
  charset reject. The two folds diverge by design.

### Adversarial findings (F1-F5) — all resolved

- **F1 (byte-clean coupling).** `pertoken_byte_clean_coupling_strip_equals_plain_equals_cell`
  asserts the property: `strip_ansi(coloured) == plain == cell(r)` over
  multi-tag, colon-namespaced, and empty-segment rows. `pertoken_color_false_emits_zero_ansi`
  asserts the plain path — SL-053 VT-2 holds.
- **F2 (dynamic table-only).** JSON path carries tags unconditionally
  (`BacklogRow.tags` is flat, never visibility-gated); `any_tagged` affects only
  the table column. `show`/`show --json` emit tags verbatim (unchanged from
  pre-SL-067).
- **F3 (overlap reject post-normalisation).** `add_set.intersection(&remove_set)`
  checked after both sides are normalised through `normalize_tag` → `BTreeSet`.
  Overlap rejected naming the first conflicting tag.
- **F4 (multi-SGR alignment).** `pertoken_multi_sgr_keeps_alignment_and_spares_the_reset`
  asserts column alignment with tags LAST, proving `render_table`'s
  trailing-fill `trim_end` strips only comfy-table padding, never the chip's
  trailing `\x1b[0m`.
- **F5 (show plain).** `format_show` keeps tags as `tags: a, b` — plain text,
  no chip colouring. Coloured chip surface is `list` only.

### Code quality

- **DRY.** The implementation rides existing seams: the edit-preserving recipe
  (F-1 refuse, `toml_edit::DocumentMut` in-place mutate, clock injection, single
  `fs::write`) modelled on `set_backlog_status`/`needs`/`after`. The
  `dep_seq::append_string_array` seam is correctly NOT reused — it is
  relationships-scoped and append-only. The sorted-array *replace* is genuinely
  new. `BL_COLUMNS` / `select_columns` / `render_columns` / `paint_cell` are
  extended, not forked.
- **Pure/imperative split.** `normalize_tag`, `fold_filter_tag`, `apply_tags`,
  `segment_hue`, `stable_hash`, `paint_tag` are pure. `run_tag` is the thin
  impure shell (disk + clock injection).
- **ADR-001 layering.** `PerToken`, `paint_tag`, `segment_hue`, `TAG_PALETTE`
  live in `listing.rs` (lower/shared); `backlog.rs` wires the column and owns
  the verb. No upward dependency.
- **No parallel implementation.** No new machinery duplicates existing surface.

### Standing risks

- **PerToken coupling is by convention only.** Nothing in the type system forces
  `split` and `cell` to agree. The F1 guard test mitigates this; a future
  refactor that changes one without the other would be caught by the test.
- **Intra-array comments not preserved.** The sorted-array replace rebuilds the
  `tags` array, dropping comments inside it. Accepted (A5) — tags are
  machine-written from a seeded `[]`.
- **Empty colon-segments tolerated.** `:x`, `a::b` produce coloured colons with
  no segment text. Charset permits colon positionally; revisit only if proven
  confusing.

### Corrigendum

F-2 ("VT-4 dynamic column visibility lacks a dedicated table-header assertion")
was raised in error. The code review revealed 5 dedicated VT-4 tests at
`src/backlog.rs:~2593-2668` covering: untagged corpus hides column, tagged
corpus shows it before title, `--columns` forces when empty, `--columns`
omitting hides despite tagged rows, and tagged row filtered by `--kind` hides
the column. F-2 is terminal (`verified`) per ledger rules and cannot be
withdrawn, but its premise was incorrect. IMP-076 should also be closed.

### Follow-up captured

- F-3: hoist `strip_ansi` from listing.rs tests to `pub(crate)`, remove
  backlog.rs copy — one-line refactor during integration.
- F-4: `paint_tag` capacity hint and `segment_hue` unwrap_or — tolerated
  (cosmetic, dead paths today).

### Phase criteria — all green

Every EN/EX/VT in `plan.toml` holds. The 1286 tests pass (1 extraneous failure
on the partial tree, not SL-067 related). `cargo clippy` zero warnings.
Behaviour-preservation gates: SL-053 VT-2 plain-path, existing `list`/`show`
goldens, and the two updated JSON goldens all pass.

### Integration path

At `/close`, integrate the SL-067 additions onto main as conventional commits,
filtering out the extraneous dispatch worktree deletions:

1. `feat(SL-067): add BacklogCommand::Tag CLI wiring` — `src/main.rs`
   (clap variant + write_class + dispatch)
2. `feat(SL-067): producer — tag verb, normalisation, JSON projection` —
   `src/backlog.rs` (normalize_tag, fold_filter_tag, apply_tags, run_tag,
   BacklogRow.tags, json_rows, filter input fold, any_tagged/effective_default,
   PHASE-01 tests)
3. `feat(SL-067): reader/render — PerToken paint, tag chips, dynamic column` —
   `src/listing.rs` (ColumnPaint::PerToken, TAG_PALETTE, stable_hash,
   segment_hue, paint_tag, paint_cell PerToken arm, PHASE-02 tests)
4. `test(SL-067): update golden tests for unconditional tags projection` —
   `tests/e2e_backlog_list_order_golden.rs`,
   `tests/e2e_list_columns_golden.rs`

Also apply during integration, per F-3:

5. `chore(SL-067): hoist strip_ansi to pub(crate) in listing.rs` — remove the
   backlog.rs copy, import from listing.rs in backlog.rs tests.

The integration is `/close`'s act; this audit hands off with all findings
terminal and the review done.
