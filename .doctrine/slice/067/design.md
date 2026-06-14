# SL-067 Design — Tags command surface (backlog beachhead)

## 1. Problem & frame

The backlog item model already carries a `tags: Vec<String>` axis
(`src/backlog.rs:365`), seeded `tags = []`, surfaced by `backlog show`
(`format_show`, `backlog.rs:1132`) and `show --json` (`show_json`, `backlog.rs:1271`).
It has **no command surface**: no verb mutates it, `backlog list` neither renders
nor colours it, and the `list --json` row (`json_rows`→`BacklogRow`,
`backlog.rs:846-1052`) **omits the axis entirely** — only `show` exposes it. Tags
are write-once-empty, read-only-by-`show` — effectively dead.

This slice lights up tags on the **backlog beachhead**, building on the shared
`listing.rs` machinery so the column + colour generalize to other kinds'
`list` surfaces later (the "everywhere eventually" intent). It is *not* a tag
registry, controlled vocabulary, rename, or cross-kind rollout — those are
follow-ups.

### Prior machinery this rides (no rebuild)

- **Filter-by-tag already shipped** — `backlog list -t/--tag <TAG>` (repeatable,
  OR logic) in `listing.rs::tags_admit`. This slice only adds *lenient input
  normalisation* so it round-trips with the new write verb; it is otherwise the
  read-back oracle for the verb's tests.
- **Colour** (SL-053) — `listing::ColumnPaint` + `RenderOpts.color` resolved once
  in the impure shell; `status_hue` is the by-value precedent; `paint_cell` is
  the central colour gate (no ANSI emitted under `color = false`).
- **Edit-preserving axis write** — `set_backlog_status` / `needs` / `after`:
  `toml_edit::DocumentMut` mutate-in-place (preserves comments / inert tables /
  unknown keys), no-op guard, F-1 refuse-on-missing-scaffold-key, shell-injected
  clock. See [[mem.pattern.entity.edit-preserving-status-transition]].
- **Column model extension** — pre-materialised typed row, `const` of
  non-capturing `fn(&R)->String` extractors, `select_columns` once,
  `render_columns` per block, JSON stays per-kind typed. See
  [[mem.pattern.listing.column-model-extension]].

### Locked decisions

| Ref | Decision |
|---|---|
| D1 | Colour = per-colon-segment chips: each tag split on `:`, segments hued by a stable hash, colons painted white. Needs a new `ColumnPaint::PerToken` variant. |
| D2 | Tags column is **default-visible only when ≥1 visible row is tagged** (dynamic); an explicit `--columns` request overrides (forces/omits verbatim). |
| D3 | Write verb = single `backlog tag <ID> <tags…> [--remove <tags…>]` — one atomic edit-preserving write. |
| D4 | Normalisation = trim → lowercase → dedupe → charset `[a-z0-9_:-]` (colon allowed for namespacing, e.g. `area:backlog`). Single chokepoint `normalize_tag`. |

## 2. Colour: `ColumnPaint::PerToken` + tag chips (`listing.rs`)

### 2.1 Current vs target

`paint_cell` (`src/listing.rs`) paints the whole cell one hue
(`ColumnPaint::{None,Fixed,ByValue}` → one `Option<AnsiColors>`). A tags cell is
`"a, b, c"` — to give each tag (and each colon-segment within it) its own colour,
the paint model must address *tokens*, not the whole cell.

### 2.2 New variant

```rust
pub(crate) enum ColumnPaint<R> {
    None,
    Fixed(owo_colors::AnsiColors),
    ByValue(fn(&R) -> Option<owo_colors::AnsiColors>),
    /// Multi-valued cell: `split` yields the row's tokens; `render` paints ONE
    /// token (ANSI). Invoked ONLY under `color = true`; tokens joined by `", "`.
    PerToken { split: fn(&R) -> Vec<String>, render: fn(&str) -> String },
}
```

`paint_cell` PerToken arm (reached only when `color == true`):

```rust
// Handle PerToken BEFORE the `let hue = match paint` block (its return type is
// String, not Option<AnsiColors> — folding it into the hue match reads as a type
// clash even though the diverging `return` compiles). `render(t.as_str())` is
// explicit; relying on &String→&str deref coercion through a fn pointer is obscure.
if let ColumnPaint::PerToken { split, render } = paint {
    return split(row).iter().map(|t| render(t.as_str())).collect::<Vec<_>>().join(", ");
}
```

**Byte-clean invariant (preserves SL-053 VT-2).** Under `color = false`,
`paint_cell` returns the column's `cell(r)` extractor output **unchanged** — it
never calls `render`. So the plain path emits zero ANSI. The colored render,
stripped of ANSI, MUST equal `cell(r)`; both are derived from the same `tags`:
- `cell  = |i| i.tags.join(", ")`
- `split = |i| i.tags.clone()`, joined by `", "`, each token's stripped form ==
  the token verbatim.

### 2.3 Tag chip renderer (shared, `listing.rs`)

Generic chip renderer + segment hue live beside `status_hue` (shared machinery,
reusable by any future tags column):

```rust
/// Render one tag as a colon-segment chip: segments hued by `segment_hue`,
/// colons painted white. ANSI unconditional — only ever called under colour.
pub(crate) fn paint_tag(tag: &str) -> String { /* split(':'), paint segs, white colons */ }

/// Stable, pure hue for a colon-segment: byte-fold hash → fixed palette index.
/// No RNG, no clock — deterministic across runs. Empty segment → None.
fn segment_hue(seg: &str) -> Option<owo_colors::AnsiColors> { /* hash % PALETTE.len() */ }
```

- `cli:command` → `segment_hue("cli")` + white `:` + `segment_hue("command")`
  (two distinct hues, white separator).
- `security` (no colon) → one segment, one hue.
- Empty segments (`:x`, `a::b`) → that segment contributes no text, the white
  colon still renders (a tolerated edge; charset permits colon anywhere).
- **Palette** = a fixed `const [AnsiColors; N]` of distinguishable hues
  **excluding Red/BrightRed** (Red is reserved for adverse status in
  `status_hue`) and Black/White (background / colon separator). Index =
  `stable_hash(seg) % N`. `stable_hash` is a pure byte fold (e.g. FNV-1a).

## 3. Tags column + dynamic visibility (`backlog.rs`)

- `BL_COLUMNS` gains a `tags` column (5 → 6):
  ```rust
  listing::Column {
      name: "tags", header: "tags",
      cell: |i| i.tags.join(", "),
      paint: listing::ColumnPaint::PerToken {
          split: |i| i.tags.clone(),
          render: listing::paint_tag,
      },
  }
  ```
- **Default order** when shown: `id, kind, status, tags, title` — tags before the
  wide, free-flow `title` column. NB the `BL_DEFAULT` const is **unchanged**
  (`["id","kind","status","title"]`, `backlog.rs:894`); `tags` is spliced into a
  locally-built `effective_default`, never into the const.
- **Dynamic default visibility** (D2). `any_tagged` is computed on the **final
  displayed set** — *after* both `listing::retain` and the `--kind` filter
  (`backlog.rs:994-995`), since D2 keys on a *visible* tagged row. Computing it on
  the pre-`--kind` retained corpus would reveal an all-empty column when the only
  tagged item was filtered out by `--kind`. Concretely, immediately before the
  `select_columns` call (`backlog.rs:1018`):
  `let any_tagged = items.iter().any(|i| !i.tags.is_empty());`
  - `--columns` **given** → honoured verbatim (tags shown iff requested, even when
    every cell is empty).
  - `--columns` **absent** → effective default = `BL_DEFAULT` with `"tags"`
    spliced before `"title"` **iff** `any_tagged`; otherwise `BL_DEFAULT`
    unchanged.
  The set is computed once and reused across `--by id` blocks, so column layout
  is uniform across blocks (a tagged item in one kind-block reveals the column
  for all blocks).

Implementation seam: today `run_list` calls
`select_columns(&BL_COLUMNS, BL_DEFAULT, columns.as_deref())`. Replace the
`default` argument with a locally-built `effective_default: Vec<&str>` derived as
above; `select_columns`'s `Some(columns)` path already ignores `default`, so the
dynamic logic only affects the `None` path.

### 3.1 `list --json` carries tags (scope add)

Today `json_rows`→`BacklogRow` (`backlog.rs:846-1052`) omits `tags`, so a JSON
survey consumer cannot read the axis it can now filter/colour — an inconsistency
once tags are live. Add a `tags: Vec<String>` field to `BacklogRow` and
`tags: i.tags.clone()` to the `json_rows` projection. This is **unconditional**
(unlike the dynamic *table* column — JSON rows are flat and stable, never
visibility-gated). `show --json` (`show_json`, `backlog.rs:1271`) already carries
tags and is unchanged.

## 4. Write verb `backlog tag` (`backlog.rs`)

### 4.1 Surface

```
doctrine backlog tag <ID> [TAGS]... [--remove/-d <TAGS>...]
```

clap subcommand alongside `needs` / `after`. Positional `TAGS` are adds;
`--remove`/`-d` (repeatable or comma-multi, matching `--tag`'s style) are
removes. **At least one** add or remove required (else hard error — no silent
no-op invocation). A tag appearing in both add and remove → reject (user error).

### 4.2 Normalisation chokepoint

```rust
fn normalize_tag(raw: &str) -> anyhow::Result<String>
```
trim → lowercase → validate every char in `[a-z0-9_:-]`, non-empty → else
`bail!` naming the offending token. Single chokepoint **for the write path** (cf.
`resolve_slug`, [[mem.pattern.input.slug-charset-wall-at-resolve-slug]]).

The filter path (§5) needs a **second, lenient** fold — name it `fold_filter_tag`
(trim + lowercase, **no charset reject**). It cannot route through `normalize_tag`:
a filter that matches nothing must succeed silently, never `bail!`. "Single
chokepoint" (D4) governs the write only; the two folds diverge by design (A3).

### 4.3 Edit-preserving write

On the `set_backlog_status` recipe:
1. `root::find` → `parse_ref` (kind + id) → `require_item` (exists, else `bail!`).
2. Read `<kind>-NNN.toml`; `parse::<toml_edit::DocumentMut>()`.
3. **F-1 refuse**: if the `tags` key is absent the file is malformed
   (hand-edited) — `bail!`, never `insert` (a tail-insert lands inside a trailing
   subtable = corruption). All items seed `tags = []`, so present in well-formed
   files.
4. Apply on a set: `new = (current ∪ normalize(adds)) ∖ normalize(removes)`,
   **stored sorted** (stable render + dedupe).
5. **No-op guard** (compare as **sets**, not ordered vecs): if the *set* `new`
   equals the *set* `current`, return without writing (content + mtime hold).
   Covers idempotent re-add of a present tag and remove of an absent tag.
   **Why set-compare:** §4.3-4 stores sorted, but a hand-authored `current` may be
   unsorted; an ordered-vec compare would treat an idempotent re-add of an unsorted
   store as a change and write+stamp spuriously. (Set-compare also means the first
   real write silently re-sorts a legacy unsorted store — acceptable self-heal.)
6. Write the `tags` array back via `toml_edit` — a **full sorted-array replace**
   (clear + push each sorted value, or assign a fresh `Array`), preserving the
   doc *outside* the array (comments, inert tables, unknown keys); stamp `updated`
   (clock injected by the shell, `clock::today()`); single
   `fs::write(path, doc.to_string())`.
7. Print post-state: `Tagged ISS-003: area:backlog, security`.

**Not reused: `dep_seq` write seam.** `dep_seq::append_string_array` /
`apply_string_append` (`dep_seq.rs:254-313`) is the nearest precedent but does not
fit: it navigates `relationships.<field>` (tags is **top-level**), it is
**append-only** (no remove), and it is currently `dead_code` staged for the slice
consumer. The set-replace here is genuinely new (the codebase has append +
status-key-set, no array-*replace*), so this is not a parallel implementation. The
F-1 refuse pattern (step 3) *is* modelled on `apply_string_append`'s
missing-array bail. **A5** (see §9): a rebuilt array does not preserve
*intra-array* comments (`tags = [ "a", # note ]`) — acceptable, tags are
machine-written from a seeded `[]`.

## 5. Filter round-trip (`-t`)

The existing `-t/--tag` filter (`tags_admit`, `listing.rs:262`) does exact match
on stored tags. Apply the lenient `fold_filter_tag` (§4.2 — trim + lowercase, **no
charset reject**, a filter matching nothing must not error) to the filter inputs
so `-t Security` matches stored `security`. Minimal touch: fold `args.tags` in
`list_rows` *before* `listing::build` (`backlog.rs:988`). `show` / `show --json`
emit tags verbatim and are unchanged; `list --json` **gains** the `tags` field
(§3.1, unconditional). The round-trip guarantee holds for verb-written tags;
a hand-authored non-normalised stored tag (e.g. uppercase) is a latent mismatch
the lenient filter does not bridge (A6).

## 6. Verification

- **Round-trip e2e**: `tag ISS add a b` → `list -t a` surfaces it → `tag ISS
  --remove a` → drops from `-t a`.
- **Normalisation**: `tag X Security` stores `security`; bad charset (`a b`, `a@b`)
  rejected naming the token; colon accepted (`area:backlog`).
- **Idempotency**: re-add present / remove absent → no write (assert mtime
  unchanged); add∩remove overlap rejected. The re-add-present fixture seeds an
  **unsorted** store (e.g. `["b","a"]`) to prove the set-compare no-op guard
  (§4.3-5) does not write+stamp on a logical no-op.
- **`list --json` tags** (§3.1): an untagged item emits `"tags": []`; a tagged
  item emits its tags array unconditionally (no visibility gate on the JSON path).
- **Dynamic column**: untagged corpus → no `tags` column; ≥1 tagged → column
  present; `--columns id,tags` forces it even when empty; `--columns` omitting
  tags hides it despite tagged rows; tagged item filtered out by `--kind` → no
  column (D2 keys on a *visible* row, §3).
- **Colour**: `color = true` → per-segment hues, white colons, stable across two
  runs; `color = false` → byte-clean (zero ANSI) AND chip-join stripped ==
  `cell(r)` plain join (SL-053 VT-2 plain-path holds). With `--columns id,tags`
  (tags **last**), assert the L505-515 trailing-fill `trim_end` strips only comfy
  -table padding, never the chip's trailing `\x1b[0m` (F4 multi-SGR alignment).
- **Edit-preserving**: a hand comment / inert `[relationships]` table / unknown
  key survives a `tag` write; `updated` stamped; unrelated keys untouched; an F-1
  missing-`tags` file is refused **byte-unchanged**.
- **Unit**: `normalize_tag` (case/charset/colon) + `fold_filter_tag` leniency
  (no charset reject); `segment_hue` determinism + palette excludes Red **and
  BrightRed** (and Black/White); `paint_tag` colon/white structure.
- `just gate` green; `cargo clippy` zero warnings.

## 7. Phase shape (for `/plan`)

- **P1** — `normalize_tag` + `backlog tag` verb (edit-preserving sorted-array
  replace, set-compare no-op guard, F-1) + `fold_filter_tag` filter normalisation
  (§5) + `BacklogRow.tags` / `json_rows` projection (§3.1, the producer's JSON
  read-back) + write/round-trip/idempotency/`list --json` tests.
- **P2** — `ColumnPaint::PerToken` + `paint_tag` + `segment_hue` + palette in
  `listing.rs`; `tags` column + dynamic visibility in `backlog.rs`; colour +
  column + plain-path tests.

(P1 producer before P2 reader/render — round-trip tests in P1 can seed tags via
the verb; P2 then renders them.)

## 8. ADR / governance alignment

- **ADR-001** (layering leaf ← engine ← command, no cycles): `PerToken`,
  `paint_tag`, `segment_hue`, palette are generic and live in the shared
  `listing.rs` (lower layer); `backlog.rs` (command) wires the column and owns
  the verb. No upward dependency, no cycle.
- **Pure/imperative split**: `normalize_tag`, `segment_hue`, `paint_tag`, the
  set-apply are pure; clock + disk live in the thin `tag` shell (date injected).
- **Storage rule**: `tags` is authored TOML structured data; the write is
  edit-preserving (no reserialise). No queried/derived data enters prose.

## 9. Open questions / assumptions

- **A1** Stored tags are sorted (set semantics); insertion order is not
  preserved. Accepted for stable render + simple dedupe.
- **A2** Empty colon-segments (`a::b`, `:x`) are tolerated, not rejected — charset
  permits colon positionally. Revisit only if it proves confusing.
- **A3** Lenient filter normalisation does not reject bad charset (unlike the
  write path) — a non-matching filter is valid, an erroring one is hostile.
- **A4** Palette size N and exact hues are an implementation choice at P2; the
  only hard constraints are determinism, Red/BrightRed-exclusion, and ≥ ~8
  distinguishable hues to keep collisions rare.
- **A5** The sorted-array replace (§4.3-6) preserves the doc outside the array but
  not *intra-array* comments. Accepted: tags are machine-written from a seeded `[]`.
- **A6** The verb normalises only adds/removes, never the existing `current`. A
  pre-existing hand-authored non-normalised tag (uppercase) is preserved as-is and
  will not round-trip with the lenient `-t` filter until rewritten through the
  verb. Accepted: seeds are empty and the verb is the sole writer going forward;
  the first set-replace write self-heals ordering (not casing).

## 10. Internal adversarial pass — integrated findings

- **F1 — implicit cell/split coupling (P2 guard test).** The byte-clean invariant
  (§2.2) is enforced only by `cell` and `split` both reading `tags`; nothing in
  the type system couples them. P2 MUST carry a guard test: for the tags column,
  `strip_ansi(paint_cell(color=true)) == paint_cell(color=false) == cell(r)` over
  a fixture with multi-tag, colon-namespaced, and empty-segment rows. This is the
  property, not a proxy ([[mem.pattern.review.guard-test-asserts-property-not-proxy]]).
- **F2 — dynamic logic is table-only.** The `any_tagged` probe and
  `effective_default` splice run ONLY on the table render branch. The `--json`
  branch is untouched by the *dynamic* logic, but it is NOT a no-op: `list --json`
  did not carry tags at all (only `show`/`show --json` did), so §3.1 adds a
  **flat, unconditional** `tags` field to `BacklogRow` — never visibility-gated.
  `any_tagged` is computed on the **final displayed set** — after both `retain`
  AND the `--kind` filter (§3), and once (so the column set is uniform across
  `--by id` blocks). (Corrects the original F2, which wrongly
  asserted `list --json` already had an unconditional tags field.)
- **F3 — overlap reject is post-normalisation.** `add ∩ remove` is checked after
  `normalize_tag` folds both sides (`tag X A -d a` collides as `a`), then rejected.
- **F4 — unconditional colour + multi-SGR alignment.** `paint_tag` uses owo's
  unconditional `.color()` gated solely on the injected `color` bool (D3, never
  `if_supports_color`) — see [[mem.pattern.render.force-no-tty-styling-axis-only]].
  A tags cell emits multiple SGR sequences (per segment + white colons); P2's
  render test MUST assert column alignment holds for multi-sequence cells.
  *De-risk:* comfy-table `custom_styling` measures width via `console`'s
  ANSI-stripping, which strips **all** CSI sequences (not one wrap), so multi-SGR
  is mechanically sound — the test is belt-and-braces, not a gamble. The one extra
  case to pin: `--columns id,tags` puts tags **last**, where `render_table`'s
  trailing-fill `trim_end` (`listing.rs:505-515`) meets a chip ending in `\x1b[0m`;
  verify the trim strips only comfy-table padding (fill lands after the reset).
- **F5 — `backlog show` stays plain (scope boundary).** `show` uses a separate
  `parts.push` renderer and keeps tags as a plain `tags: a, b` line. The coloured
  chip surface is `list` only; colouring `show` is out of scope (a follow-up if
  wanted).
