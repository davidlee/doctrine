# IMP-039: Colour the deferred listing surfaces: ad-hoc writeln! + priority survey/next

Follow-up deferred by **SL-053** (terminal output polish). SL-053 shipped colour
strictly behind the `render_columns` seam, keeping the colour story to a single
mechanism. Two listing surfaces bypass that seam and so stayed monochrome:

- **`priority` (`survey_human` / `next_human`, src/priority/render.rs)** — call
  `listing::render_table` directly, bypassing the `render_columns` paint path.
  They gained the `│` separators from PHASE-01 but no colour.
- **ad-hoc `writeln!(io::stdout(), …)` listing surfaces** — render outside the
  shared seam entirely.

Scope: bring these under colour without forking a second colour mechanism —
either route them through `render_columns` (preferred — reuses the injected
`color` bool, `ColumnPaint`, and shared `status_hue`) or extend the seam to cover
them. Honour the same purity rule (capability resolved in the shell, injected as
a bool — no `if_supports_color` in the pure layer, D3) and the goldens-stay-plain
invariant (piped ⇒ byte-for-byte plain).

One item by design: both surfaces share the same deferred-colour story and the
same single-mechanism constraint.
