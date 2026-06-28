# SL-171 — design: `doctrine next` read-surface improvements

Status: drafted (pre-lock). Governs design intent for SL-171; the plan is downstream.

## 1. Purpose & framing

`doctrine next` is the advisory "what should I work on?" worklist — actionable-only,
sorted by the SL-133 multi-dimensional score. Today its render is fixed: hardcoded
columns, no facet visibility, no pagination. This slice is a **read-surface**
upgrade only — **no change to scoring, partitioning, eligibility, ordering, or the
`--json` payload.** The graph, channels, and frontier order are untouched.

Four upgrades:

1. Wire `--columns` (the SL-037 `select_columns` projection `next` never adopted).
2. Surface authored `estimate` / `value` / `tags` facets as columns.
3. Retire the `unblocks` column (its signal is already in the score; see §6 D1).
4. Add `--limit` / `--offset` / `--page` pagination with a truncation footer.

## 2. Current behaviour vs target

| Aspect | Current | Target |
|---|---|---|
| Columns | fixed `[id, kind, status, score, unblocks, title]`; `next_human` collects **all** of `NEXT_COLS` | projected via `select_columns`; default `[id, status, score, estimate, value, (tags), title]` |
| `--columns` | absent | present; unknown name → clean `select_columns` error |
| `kind` column | in default | selectable-only (redundant with id prefix) |
| `unblocks` column | present (`blocking.len()`) | **removed from `NEXT_COLS` entirely** |
| `estimate`/`value`/`tags` | not surfaced | columns (compact, unitless); `tags` via `default_with_tags` |
| Row count | every actionable row dumped (~60) | default `--limit 20`; `--offset`/`--page`; footer when truncated |
| `--json` | full set, all fields incl. `blocking` | **unchanged** (no pagination, no column projection) |

## 3. Design decisions (locked)

See §6 for the full decision ledger. Headlines:

- **D1 — drop `unblocks` as a column.** `next` lists only actionable rows, already
  sorted by a score whose `leverage` + `optionality` dimensions encode downstream
  impact (weighted, transitive) better than a raw direct count. The precise set
  lives in `blockers <id>`, `explain <id>`, and `next --json` (`blocking`,
  unchanged). `NextRow.blocking` stays; only the column dies. `--columns unblocks`
  errors cleanly (no such column).
- **D2 — `NodeAttr` carries `facets: EntityFacets`.** The shared projection
  (estimate/value/risk/tags) per `mem.pattern.facets.shared-projection`. `NextRow`
  carries the render subset `estimate`/`value`/`tags`. A future `risk` column
  extends `NextRow` only — never `NodeAttr`.
- **D3 — pagination mirrors `memory`** (`--limit`/`--offset`/`--page`), and the
  truncation footer fn is **lifted** from `retrieve.rs` to shared `listing`.
- **D4 — compact unitless cells.** estimate `{lower}–{upper}` (en-dash), value
  `{value:.1}`, tags joined, absent → `·`. Not the unit+confidence `format_show`
  forms (wrong altitude; they need config + a captured closure — cells are bare
  `fn(&R)->String`).

## 4. Code impact (the declared touch-set → `design-target` selectors)

| File | Change |
|---|---|
| `src/commands/cli.rs` | `Next` variant: add `--columns` (`Vec<String>`, comma-delimited), `--limit` (default 20), `--offset` (default 0), `--page` (`Option<usize>`, `conflicts_with = "offset"`). Resolve `page→offset` at dispatch (mirror `memory`), thread into `run_next`. |
| `src/priority/mod.rs` | `run_next`: accept `columns: Option<Vec<String>>`, `limit: usize`, `offset: usize`; thread to `next_human`; `--json` path ignores all three. |
| `src/priority/render.rs` | `NEXT_COLS`: add `estimate`/`value`/`tags`, **remove `unblocks`**; `NEXT_DEFAULT = [id, status, score, estimate, value, title]`; `next_human`: `default_with_tags` + `select_columns` + `--limit`/`--offset` slice + footer via lifted helper. New `NEXT_LIMIT_DEFAULT` const. |
| `src/priority/view.rs` | `NextRow`: add `estimate: Option<EstimateFacet>`, `value: Option<ValueFacet>`, `tags: Vec<String>`. |
| `src/priority/surface.rs` | `next()`: project `g.attrs[k].facets` → the three `NextRow` fields. |
| `src/priority/graph.rs` | `NodeAttr`: add `facets: EntityFacets`; populate at 3c from `entity` (estimate/value/risk/tags already in scan; the 2b pre-pass already clones them). |
| `src/listing.rs` | **lift** `format_truncation_notice` here (pub(crate)); signature `(shown, total, offset, page_size) -> String`. |
| `src/retrieve.rs` | re-point its two call sites to `listing::format_truncation_notice`; delete the local copy. |

Selectors (literal paths the design names):
`src/commands/cli.rs`, `src/priority/mod.rs`, `src/priority/render.rs`,
`src/priority/view.rs`, `src/priority/surface.rs`, `src/priority/graph.rs`,
`src/listing.rs`, `src/retrieve.rs`.

## 5. Render contract detail

### 5.1 Columns (`NEXT_COLS`)

```
id      Fixed(Cyan)            r.id
kind    None                   r.kind                              (selectable-only)
status  ByValue(status_hue)    r.status
score   None                   format!("{:.1}", r.score)
estimate None                  r.estimate → "{lower}–{upper}" | "·"
value   None                   r.value    → format!("{:.1}")     | "·"
tags    PerToken{split,paint_tag}  r.tags.join(", ")             (via default_with_tags)
title   Alternate(TITLE_*)     r.title
```

`tags` column rides the house convention verbatim (backlog/knowledge/concept_map):
`cell` and `split` agree byte-for-byte stripped of ANSI; `default_with_tags(NEXT_DEFAULT, any_tagged)`
splices `tags` before `title` iff any surfaced row is tagged; `--columns` bypasses
the splice (explicit list wins). `any_tagged = rows.iter().any(|r| !r.tags.is_empty())`
is computed over the **full** result set, **before** the pagination slice (D7) — so
the `tags` column's presence is stable across pages, never flickering per-page.

Cell formatting is a pure non-capturing `fn(&NextRow)->String` — no config, no unit.
The `·` middle-dot is the listing absent-value convention. **No shared const for it
exists today** (adversarial F2) — introduce `listing::ABSENT_CELL = "·"` (STD-001;
no magic string) and use it for all three facet cells.

### 5.2 Pagination (`next_human`)

```
let effective = default_with_tags(NEXT_DEFAULT, any_tagged);
let sel = select_columns(&NEXT_COLS, &effective, columns.as_deref())?;
let total = rows.len();
let visible = if limit == 0 { &rows[offset.min(total)..] }
              else { &rows[offset.min(total)..(offset + limit).min(total)] };
let body = render_columns(visible, &sel, opts);
// footer: table only, only when shown < total
if visible.len() < total {
    body + &listing::format_truncation_notice(visible.len(), total, offset, page_size)
}
```

- **Footer guard is `limit != 0 && shown < total`** (NOT just `shown < total`).
  `format_truncation_notice` computes `offset / page_size`; with `--limit 0`,
  `page_size == 0` ⇒ **integer division-by-zero panic** (adversarial F1). The
  `limit != 0` gate forecloses it: uncapped ⇒ never paginated ⇒ never footed, even
  with `--offset N` (`--limit 0 --offset 5` shows `rows[5..]`, no footer).
- `page_size = limit` (the resolved value). `--page` resolves `offset = (page-1)*page_size`
  at the CLI layer (mirror `memory`): `--page 0` → bail; `--limit 0 --page N` → bail
  (`--page requires a positive --limit`).
- Footer wording is `format_truncation_notice`'s existing `--page`-flavoured string —
  now shared, identical across `next` and `memory retrieve`/`find`.

### 5.3 `--json` — untouched

`next_json` keeps the full row set, every field (incl. `blocking`), no pagination, no
column projection (the listing precedent: `--columns`/`--limit` are table-only).

## 6. Decision ledger

- **D1** drop `unblocks` column — see §3. Rationale: redundant with score's
  leverage/optionality dims; precise set in `blockers`/`explain`/`--json`.
- **D2** `NodeAttr.facets: EntityFacets`; `NextRow` render-subset. Future risk
  column extends `NextRow` only.
- **D3** mirror `memory` pagination triple; lift `format_truncation_notice` to
  `listing` (pure move — retrieve goldens stay green, behaviour-preservation gate).
- **D4** compact unitless cells (no `format_show` reuse — altitude + closure mismatch).
- **D5** `kind` demoted to selectable-only (redundant with id prefix).
- **D6** `--json` unchanged.
- **D7** `any_tagged` (the `default_with_tags` gate) is computed over the full
  result set, before the offset/limit slice — `tags` column presence is page-stable
  (surfaced at planning; no per-page flicker).

## 7. Verification alignment

Evidence that must change/add (all in-crate unit tests — `next` is pure over a seeded
fixture graph, the SL-133/SL-047 test style):

- **VT-A `--columns` projection** — `next --columns id,score` emits exactly those
  headers; unknown name → `select_columns` error (mirrors backlog's column tests).
- **VT-B default set** — default headers are `id status score estimate value title`
  when no row tagged; `kind`/`unblocks` absent; `unblocks` is not even selectable.
- **VT-C tags conditional** — `tags` column appears iff a surfaced row is tagged;
  hidden when none tagged; `--columns tags` forces it even all-empty.
- **VT-D facet cells** — estimate `3.2–4.8`, value `5.0`, absent `·`; faceted vs
  bare rows.
- **VT-E pagination** — `--limit 2` shows 2 + footer `2 of N; use --page 2 …`;
  `--offset`/`--page` equivalence; `--limit 0` → all rows, **no footer**;
  `--limit 0 --offset N>0` → `rows[N..]`, **no footer, no panic** (F1 guard);
  `--limit 0 --page 2` → error; `--page 0` → error.
- **VT-F `--json` invariance** — golden `next --json` byte-identical to pre-slice
  (no column/pagination leakage; `blocking` still present).
- **VT-G behaviour-preservation** — `retrieve`/`find`/`memory list` truncation
  goldens stay green unchanged after the `format_truncation_notice` lift.

## 8. Governance & constraints

- **STD-001** — named consts: `NEXT_LIMIT_DEFAULT = 20`, column names, the `·`
  absent marker (reuse if a shared const exists). No magic strings.
- **ADR-001 layering** — `facet`/`estimate`/`value` are leaf; `NodeAttr` (engine)
  carries data, never policy; render stays in the render layer. `format_truncation_notice`
  in `listing` is leaf-adjacent presentation — no upward dependency.
- **Behaviour-preservation gate** — the `listing` lift + the unchanged `--json`
  payload are proved by existing goldens staying green unchanged (VT-F, VT-G).
- **SL-037 column model** + `mem.pattern.listing.column-model-extension` — pre-materialise
  the row, non-capturing `fn` extractors, `select_columns` once. `--status`
  validation N/A (`next` has no `--status`).
- Governing spec: **PRD-011 / SPEC-001** (priority engine); precedent slices
  SL-037 (columns), SL-133 (facets/score), SL-053/SL-079 (render/colour).

## 9. Open risks / edges

- **R1** wide-glyph / emoji never enters the table (we dropped the `⛓` annotation
  with the `unblocks` column) — alignment risk eliminated.
- **R2** `EstimateFacet`/`ValueFacet` `f64` formatting via `{}` prints `1` for `1.0`,
  `3.2` for `3.2` — acceptable compact form. `{:.1}` for value pins one decimal.
- **R3** `offset > total` (with `limit > 0`) → empty body + footer's `offset >= total`
  branch (`reduce --offset or --page`) — mirror retrieve's guard exactly.

## 10. Adversarial review (self) — findings & disposition

- **F1 (must-fix, integrated §5.2)** — `--limit 0 --offset N>0` makes `shown < total`
  true while `page_size == 0`, so the lifted `format_truncation_notice`'s
  `offset / page_size` divides by zero (panic). retrieve never hits this (it always
  caps). **Fixed:** footer guard is `limit != 0 && shown < total`. Adds a test:
  VT-E covers `--limit 0 --offset N` → no footer, no panic.
- **F2 (must-fix, integrated §5.1/§8)** — the `·` absent marker has no shared const;
  rendering it inline would plant a magic string (STD-001). **Fixed:** introduce
  `listing::ABSENT_CELL`.
- **F3 (impl note)** — `NEXT_COLS` array size annotation changes `[…; 6]` → `[…; 8]`
  (drop `unblocks` −1, add `estimate`/`value`/`tags` +3). Mechanical.
- **F4 (verification scope clarification)** — the existing `next_human` golden tests
  (SL-047/SL-133) assert the *old* layout (incl. `unblocks`); **this slice updates
  them** — legitimate, `next` is this slice's own surface. This is distinct from
  VT-G: the `retrieve`/`find`/`memory list` truncation goldens must stay **unchanged**
  (the `format_truncation_notice` lift is a pure move — the behaviour-preservation
  gate, which binds shared machinery, not `next`'s surface).
- **F5 (accepted, non-blocking)** — `EntityFacets` is cloned twice (the 2b base-score
  map and the 3c `NodeAttr`). Acceptable; an optional refactor builds the per-entity
  `EntityFacets` once and has `base_score` read from it. Not in scope.
- **F6 (verify-at-impl)** — `--columns` rides the house arg form `Option<Vec<String>>`
  (backlog/memory precedent), `value_delimiter = ','` for `--columns id,score`; pass
  `columns.as_deref()` into `select_columns`. Confirm against `ListArgs` at impl.
