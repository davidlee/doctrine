# CHR-005: Harden grid_min_width floor test to assert sliver-freedom against real comfy-table

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Origin: SL-054 reconciliation audit RV-012 F-2 (major, follow-up).

`grid_min_width(cols) = 4·cols-3` (src/listing.rs) is white-box coupled to
comfy-table 7.2.2's `available_content_width` accounting. Two test gaps:

1. `grid_min_width_is_the_derived_comfy_accounting` pins the *formula*
   (`grid_min_width(6)==21`), not comfy's *agreement* with it. A comfy-table
   version bump that changes the border/padding subtraction leaves the assert
   green while the real floor drifts.
2. `render_table_grid_floor_falls_back_below_and_wraps_at_or_above` asserts the
   at-floor render "wraps to >2 lines" — but a 1-char-per-column sliver IS >2
   lines, so the pathology the floor exists to prevent passes the test.

Fix: assert the floor's *purpose* against real comfy — at `grid_min_width(cols)`
every visible column seats ≥1 content char (no sliver), and below it the render
equals the Disabled byte-output. Then a comfy accounting change that breaks the
floor fails a test instead of silently serving slivers.

Low priority: comfy-table is workspace-pinned, so the coupling only breaks on a
deliberate bump; current behaviour is VH-verified (SL-054 audit).
