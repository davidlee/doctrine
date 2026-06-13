# force_no_tty suppresses only comfy-table's styling tty-consult, not Dynamic wrap

comfy-table 7.2.2 has **two independent axes** that both relate to the tty:

- **Styling axis** — `custom_styling` calls `stdout().is_terminal()` to decide
  whether to emit colour SGR. `force_no_tty()` suppresses *this* consult. This is
  the load-bearing SL-053 D6 purity guard (see the prior note,
  `mem.pattern.render.comfy-table-custom-styling-pulls-tty`).
- **Arrangement axis** — `ContentArrangement::Dynamic` + `set_width(w)` measures
  column widths and wraps over-long cells. This is driven by `table.width()`,
  which returns `Some(w)` directly when `set_width` was called — it does **not**
  consult the tty in that case.

The two are orthogonal. An SL-054 spike rendered `Dynamic + set_width(24)` with
and without `force_no_tty()` and got **byte-identical 4-line wrapped output** both
ways. So you can keep `force_no_tty()` **unconditional** (pure leaf stays tty-free)
AND wrap cells — no carve-out, no purity/readability trade.

Corollary (RSK-4): `Dynamic` still pads every wrapped line to its column width, so
per-line `trim_end` is needed under Dynamic too — keep it unconditional.

**Why:** the scope doc's ASM-1 assumed wrapping *required* dropping `force_no_tty`
(re-opening the tty read). False — they're different axes. Refuted before any code.

**How to apply:** when adding width-aware wrapping behind a `force_no_tty` render
seam, switch only `set_content_arrangement` + `set_width`; leave `force_no_tty`
alone. Always pass `set_width(w)` on the Dynamic arm so `table.width()` never falls
back to the live `crossterm::terminal::size()` tty probe.

Refines [[mem.pattern.render.comfy-table-custom-styling-pulls-tty]].
