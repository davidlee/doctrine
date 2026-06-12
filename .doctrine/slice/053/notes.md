# SL-053 — durable implementation notes

Harvested from the disposable phase sheets at audit (RV-011). Progress lived in
the runtime tree; this is what survives.

## What shipped

- **PHASE-01 (commits `b701de4` logic + `5b28112` goldens).** `render_table`
  reimplemented over comfy-table as the single layout/measurement authority;
  hand-rolled width/pad maths deleted. Minimalist ` │ ` interior separators, no
  outer frame/rules. Byte-stable piped output via `ContentArrangement::Disabled`
  + `Table::force_no_tty()` (D6). Exact line shape (D7) = pad-zero outer edges +
  per-line `trim_end()` (see gotcha below). RSK-1 two-commit split: logic
  separate from golden re-baseline.
- **PHASE-02 (commit `6e823c0`).** Colour seam behind `render_columns`. Capability
  resolved once in the impure shell (`src/tty.rs`), injected as a `bool` via
  `ListArgs` (D3). `ColumnPaint<R>` (`Fixed` id hue, `ByValue` status-by-row),
  shared `status_hue`, bold headers. 8 kinds painted; priority left monochrome.
  Goldens stayed green UNCHANGED (piped ⇒ plain).
- **PHASE-03 (commit `19edc20`).** Follow-ups captured: IMP-039 (deferred-colour
  surfaces), IMP-040 (`--color` flag, D5).

## Durable gotchas / decisions

- **comfy-table last-column fill needs `trim_end`** even with zero padding under
  `Disabled` — recorded as `mem.pattern.render.comfy-table-disabled-last-col-fill-needs-trim`.
  `force_no_tty()` is the load-bearing purity guard for piped determinism
  (`custom_styling`→`tty` reads isatty at format time).
- **`unsafe_code = "forbid"` blocks `set_var`-based tty tests.** Capability split
  into a pure `color_enabled(no_color, is_tty)` seam (tested directly) behind the
  thin `stdout_color_enabled` shell — the `git::trunk_tree_ish` injection pattern.
- **F-4 — `ByValue` reads the row's raw status, never the emitted cell.** slice
  decorates (`done ⚠`) and review composes (`open (await …)`); matching emitted
  text would silently drop colour there.
- **Uncoloured status tokens** (backlog `resolved`/`closed`, memory `superseded`
  etc., spec `deprecated`) fall to `status_hue`'s `None` default by design — no
  per-kind override, keeping the shared map singular.

## Standing risk (latent, non-blocking)

- Width measurement moved `chars().count()` → display-width (unicode-width). No
  current golden seeds a wide/CJK/`⚠`-decorated cell, so the re-baseline hides no
  present alignment shift — but the first golden that seeds a `done ⚠` slice row
  will surface one. Noted in design §6.

## Audit (RV-011)

Done, no blocker. 4 findings, all minor/nit, all verified. F-1/F-2 reconciled
`design.md` (D7 trim_end mechanism; §6 in-crate golden inventory); F-3/F-4 aligned
(purity-split + resolve-once both honor design intent).
