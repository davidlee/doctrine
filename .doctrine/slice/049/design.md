# SL-049 Design — CLI list-surface & input-validation hygiene

Canonical technical design for SL-049. Two independent, file-disjoint fixes
(IMP-017, ISS-004) bundled as one hygiene slice. ISS-005 was triaged out and
closed `wont-do` (see §0).

## 0. Triaged out — ISS-005 (record)

ISS-005 claimed `rec list` prints no header on an empty/missing corpus,
"inconsistent with `adr`/`slice list`". The premise is false. Every spine kind
suppresses the header on an empty result **by design** — `listing::render_columns`
returns `""` when `rows.is_empty()` (SL-025 §5.5), and `rec` already rides the
spine (`rec.rs:25,571`). Proof: `doctrine adr list --status superseded` → zero
rows → no header, byte-identical to an empty `rec list`. The report compared an
empty `rec` corpus against a populated `adr` corpus. Closed `wont-do`; no code
change. Reversing §5.5 (header-on-empty spine-wide) was offered and declined —
it is an SL-025 contract change touching every kind, out of scope here.

## 1. Fix A — IMP-017: `memory list` adopts the shared column model

### Current behaviour

`memory list` rides most of the shared spine already — `validate_statuses`,
`build`, `retain`, `json_envelope`, and `render_table` — but **not** the column
model (`Column<R>` / `select_columns` / `render_columns`). Two consequences:

- `list_rows` (`memory.rs:1287`) **bails** when `--columns` is set
  (`memory.rs:1294`), a guard explicitly tagged "deferred to IMP-017 … reject it
  loudly rather than silently no-op". The shared `--columns` flag reaches the
  verb via `CommonListArgs` but is refused.
- `format_rows` (`memory.rs:1070`) hand-builds a fixed 6-column grid
  (`uid type status trust key title`) and calls `render_table` directly.

### Target behaviour

`memory list` renders through the column model and accepts `--columns` with the
same subset/order/duplicate semantics as every other kind. Default (no
`--columns`) output is **byte-identical** to today.

### Code impact

`src/memory.rs`:

- Add a column table beside the other per-kind constants:
  ```rust
  const MEMORY_COLUMNS: [Column<Memory>; 6] = [
      Column { name: "uid",    header: "uid",    cell: |m| m.uid.clone() },
      Column { name: "type",   header: "type",   cell: |m| m.kind.as_str().to_string() },
      Column { name: "status", header: "status", cell: |m| m.status.as_str().to_string() },
      Column { name: "trust",  header: "trust",  cell: |m| scrub_line(&m.trust_level) },
      Column { name: "key",    header: "key",    cell: |m| scrub_line(m.key.as_deref().unwrap_or("-")) },
      Column { name: "title",  header: "title",  cell: |m| scrub_line(&m.title) },
  ];
  const MEMORY_DEFAULT: [&str; 6] = ["uid", "type", "status", "trust", "key", "title"];
  ```
  Every `cell` is a non-capturing `fn(&Memory) -> String` (the `Column.cell` fn-pointer
  contract; `scrub_line` is a free fn, so the closures coerce). This preserves the
  full-uid lead (F-A11), the security `scrub_line` on free-text cells (F-A10), and
  keyless→`-`.
- `list_rows`: delete the `args.columns.is_some()` bail; take columns before
  `build`, mirroring `rec::list_rows`:
  ```rust
  listing::validate_statuses(&args.status, MEMORY_STATUSES)?;
  let columns = args.columns.take();
  let (filter, format) = listing::build(args)?;
  let mut rows = listing::retain(collect_all(root)?, &filter, is_hidden, key);
  rows.retain(|m| type_f.is_none_or(|t| m.kind == t));
  sort_default(&mut rows);
  match format {
      Format::Table => {
          let sel = listing::select_columns(&MEMORY_COLUMNS, &MEMORY_DEFAULT, columns.as_deref())?;
          Ok(listing::render_columns(&rows, &sel))
      }
      Format::Json => listing::json_envelope("memory", &json_rows(&rows)),
  }
  ```
  `args` must become `mut`. `format_rows` is retired (its empty-suppression and
  scrub responsibilities move into the column cells + `render_columns`).

`src/main.rs`:

- `CommonListArgs.columns` doc (`main.rs:103`): drop the "rejected on `memory
  list`" clause — the rejection is gone.

### Invariants preserved

- **Empty list → no header.** `render_columns` returns `""` on empty rows (§5.5),
  identical to `render_table` before. Unchanged.
- **Security scrub.** Free-text cells (`key`, `title`, `trust`) stay `scrub_line`d;
  a newline cannot forge a row. `uid`/`type`/`status` are closed vocabularies,
  unscrubbed as before.
- **JSON path untouched** — `json_rows` + `json_envelope`, no column projection
  (SL-037 D7).
- **Ordering** stays per-kind (`sort_default`, `created`-desc + uid), applied
  after `retain`, never inside it (§5.3).

## 2. Fix B — ISS-004: `spec req add` overlong-slug abort + `--slug`

### Current behaviour

`run_req_add` (`spec.rs:782`) derives the slug unconditionally:
`resolve_slug(&title, None)` (`spec.rs:810`). `resolve_slug` (`input.rs:43`) only
guards the empty case; `derive_slug` (`entity.rs:171`) emits a slug as long as the
title. The requirement fileset writes an `NNN-slug` symlink
(`requirement.rs:212`, `format!("{name}-{}", ctx.slug)`); a long slug overflows the
255-byte filesystem name limit → `std::io` ENAMETOOLONG (OS error 36), aborting
the whole command. Two gaps: no `--slug` escape (unlike `spec new`), and no length
bound — the abort is latent in **every** kind's slug path, `req add` is just the
reported trigger.

### Target behaviour

1. `spec req add` accepts `--slug <S>` (parity with `spec new`).
2. The shared slug path is length-bounded so a long title never aborts, with
   provenance-differentiated handling:
   - **explicit `--slug` over cap → error** (`bail!`, naming the cap): the user
     chose it; fail loud, never silently truncate their input.
   - **derived slug over cap → truncate** best-effort to the cap on a `char`
     boundary, preferring a cut at the last `-` within the cap (avoids a trailing
     partial word). The user did not choose it.

### Code impact

`src/spec.rs`:

- `run_req_add`: add a `slug: Option<String>` parameter; replace
  `resolve_slug(&title, None)` with `resolve_slug(&title, slug)`.

`src/main.rs`:

- The `spec req add` clap command (the `run_req_add` call site): add
  `#[arg(long)] slug: Option<String>` and thread it into `run_req_add`.

`src/input.rs` — bound `resolve_slug`:
```rust
/// Symlink filenames are `NNN-slug` / `requirement-NNN-slug`; the filesystem caps
/// a single name at 255 bytes. Cap the slug well under that. Slug chars are
/// ASCII (1 byte each), so the char cap is the byte cap.
const SLUG_MAX: usize = 100;

pub(crate) fn resolve_slug(title: &str, slug: Option<String>) -> anyhow::Result<String> {
    match slug {
        Some(s) => {
            if s.is_empty() {
                bail!("--slug must not be empty");
            }
            if s.chars().count() > SLUG_MAX {
                bail!("--slug too long ({} chars; max {SLUG_MAX})", s.chars().count());
            }
            Ok(s)
        }
        None => {
            let derived = entity::derive_slug(title);
            if derived.is_empty() {
                bail!("Could not derive a slug from the title; pass --slug");
            }
            Ok(truncate_slug(&derived, SLUG_MAX))
        }
    }
}
```
`truncate_slug` (new, `input.rs`, pure): if the slug is within `SLUG_MAX`, return
it unchanged; else take the longest `char`-boundary prefix ≤ `SLUG_MAX`, and if a
`-` occurs within that prefix (and not at position 0), cut at the last `-` instead
(trim the trailing `-`). Never empties a non-empty slug (if no usable `-`, the
hard char-prefix stands).

### Invariants / boundary conditions

- **Collision-safe.** The numeric `NNN` is identity (`scan_ids`, allocator);
  the slug only labels the symlink. Two truncated slugs colliding still land in
  distinct `NNN` dirs — the symlink alias is cosmetic, never the key.
- **Behaviour-preserving for existing callers.** Truncation only fires on slugs
  longer than `SLUG_MAX` — which today *abort*. Every previously-successful slug
  (≤100 chars) is returned unchanged. `adr new` / `slice new` / `spec new` see no
  change except that a formerly-aborting long title now succeeds.
- **Explicit-empty** `--slug ""` errors (it did via the old empty guard; keep it).

## 3. Verification

Both fixes carry behaviour tests; nothing here is trivial-implementation.

### IMP-017
- `memory list` default output unchanged (re-pin the existing golden; it must
  stay byte-identical — proves no regression).
- `memory list --columns key,title` projects the subset in order.
- `memory list --columns nope` errors with the available-set message
  (`select_columns` parity with `validate_statuses`).
- Empty memory corpus → no header (unchanged; guards §5.5 didn't regress).
- `scrub_line` still applied — a memory with a newline in `title` cannot forge a
  second row (unit on the cell or black-box).

### ISS-004
- `spec req add <spec> "<title>" --slug short` → succeeds, writes slug `short`.
- A title long enough to previously abort → succeeds, slug bounded to `SLUG_MAX`,
  no ENAMETOOLONG, files + `NNN-slug` symlink created.
- `--slug <101-char>` → errors naming the cap; nothing written.
- `truncate_slug` unit: within-cap unchanged; over-cap cuts at last `-`; over-cap
  with no `-` hard-cuts on a char boundary; never empties.
- Two long titles sharing a 100-char prefix → distinct `REQ-NNN` dirs (collision
  safety).

### Gate
- `just check` green; `cargo clippy` zero warnings (watch the `bail!`/`format!`
  string-assembly denies and the fn-pointer `Column` cells).

## 4. Phasing intent (for /plan)

Two file-disjoint phases, independently shippable:
- **PHASE-01 — IMP-017** (`memory.rs`, `main.rs` doc): column model + `--columns`.
- **PHASE-02 — ISS-004** (`spec.rs`, `main.rs`, `input.rs`): `--slug` + shared cap.

Order is free (no shared file beyond read-only `listing.rs`). `main.rs` is touched
by both but in disjoint regions (memory columns doc vs spec-req-add command) — a
trivial merge if parallelised; serial avoids it entirely.

## 5. Open questions

- **OQ-1 (resolved, confirm at execute):** `SLUG_MAX = 100` chars. Rationale: the
  255-byte name limit minus the longest prefix (`requirement-NNN-`, ~16 bytes)
  leaves >200; 100 is a safe, round, generous bound for any real slug. Adjustable
  at execute if a convention surfaces.
