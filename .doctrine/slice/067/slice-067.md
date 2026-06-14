# Tags command surface: backlog beachhead (add/remove, list column, colour)

## Context

The backlog data model already carries a per-item `tags: Vec<String>` axis
(`src/backlog.rs` — seeded `tags = []`, surfaced in `backlog show` and the JSON
projection). It has **no command surface**: no verb mutates it, and the `list`
survey neither renders nor colours it. Tags are write-once-empty, read-only-by-
`show`, effectively dead.

Prior shipped machinery this slice rides rather than rebuilds:
- **Filter-by-tag is already done** — `backlog list -t/--tag <TAG>` (repeatable,
  OR logic) lands in `listing.rs::tags_admit`. No filter work required; the slice
  *exercises* it as the read-back oracle for the new write verb.
- **Colour machinery exists** (SL-053) — `listing::ColumnPaint::{None,Fixed,ByValue}`
  + `RenderOpts.color` resolved once in the impure shell, `status_hue` as the
  by-value precedent. The list column model (`BL_COLUMNS`, `select_columns`,
  `render_columns`) is the shared, cross-kind surface.
- **Edit-preserving axis-append verbs exist** — `backlog needs` / `backlog after`
  append to a list axis; `run_edit` / `set_backlog_status` is the in-place,
  clock-injected, edit-preserving write precedent.

Backlog is the **beachhead**: tags are wanted on other kinds' list surfaces
eventually, so the column + colour approach must live on the shared `listing.rs`
machinery (not bespoke to backlog) where it already generalizes.

## Scope & Objectives

1. **Tags column in `backlog list`** — add a `tags` column to `BL_COLUMNS`.
   Decision for design: in the default visible set, or opt-in via `--columns`
   (cf. `slug` which is opt-in, SL-037 D4).
2. **Colour the tags** — paint tag values when colour is enabled. The current
   `ColumnPaint::ByValue` returns one hue for the whole cell; a tags cell is a
   join of N values. Design must resolve: one fixed tag-column hue, a stable
   per-tag hue (hash→colour) requiring a richer paint than per-cell-single-colour,
   or paint-the-cell-uniformly. Reuse `status_hue`'s stable-mapping shape.
3. **add/remove CLI** — a verb to mutate an item's `tags` axis (working name
   `backlog tag add|remove <ID> <tag…>`). Edit-preserving in-place write on the
   `needs`/`after` precedent: validate the ref exists, append/remove, dedupe,
   clock-stamp `updated`. Design: idempotency (re-add a present tag), removing an
   absent tag (no-op vs error), tag charset/normalisation.

## Non-Goals

- **Filter-by-tag** — already shipped; not re-implemented (only read-back used).
- **Tags on non-backlog kinds** (slice/spec/adr list surfaces) — beachhead only.
  The shared-machinery design keeps the door open; wiring other kinds is a
  follow-up, not this slice.
- A tag *registry* / controlled vocabulary / rename / colour-config — free-text
  tags only. (`--color=auto|always|never`, IMP-040, is a separate listing-wide
  concern.)
- Tag-axis relations or semantics beyond a flat string set.

## Affected surface

- `src/backlog.rs` — new `tag` verb + handler, `BL_COLUMNS` tags column, CLI
  arg wiring, tests.
- `src/listing.rs` — only if multi-value cell painting needs a richer
  `ColumnPaint` (design-dependent).
- `src/main.rs` / CLI command enum — the new subcommand.

## Open Questions — RESOLVED (design.md §D1–D4)

- **OQ-1** Column visibility → **default-visible when ≥1 visible row tagged**
  (dynamic); explicit `--columns` overrides. (D2)
- **OQ-2** Colour → **per-colon-segment chips**: each tag split on `:`, segments
  hued by a stable hash, colons white. New `ColumnPaint::PerToken` variant. (D1)
- **OQ-3** Verb shape → single `backlog tag <ID> <tags…> [--remove <tags…>]`,
  one atomic edit-preserving write. (D3)
- **OQ-4** Normalisation → trim + lowercase + dedupe + charset `[a-z0-9_:-]`
  (colon allowed for namespacing), single `normalize_tag` chokepoint. (D4)

## Assumptions

- **ASM-1** Filter `-t` semantics stay as-is (exact OR match); only *lenient*
  input normalisation added so it round-trips with the write verb.
- **ASM-2** Stored tags are sorted (set semantics); insertion order not preserved.

## Verification / Closure intent

- Round-trip: `tag add` → `list -t <tag>` surfaces the item; `tag remove` →
  it drops out. Idempotent re-add / absent-remove behave per the design ruling.
- `list` renders the tags column; colour present under `color=true`, absent
  (byte-clean, no ANSI) under `color=false` — the SL-053 VT-2 plain-path
  invariant holds.
- Edit-preserving write: unrelated TOML fields/formatting untouched; `updated`
  stamped.
- `just gate` green; clippy zero-warning.
