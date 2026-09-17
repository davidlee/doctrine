# IMP-453: Decouple -X raster resolution from placement width

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`terminal_image::fit_box` and `kitty::place` (`DEC-256`) currently share one
number: the raster is produced at the placement's width, and placed at its own
width. That welds a **cost** decision to a **display** decision.

It bites when the per-side cap binds. On a wide high-DPI window the fit box is
`6270 x 10000`, and every non-trivial graph binds on height — so the raster
comes out narrower than the window and the drawing does not fill it:

| graph | raster | of a 6270 px width |
|---|---|---|
| `SL-245` | 5491 x 10000 | 88% |
| `SL-245 --depth 2` | 5636 x 10000 | 90% |
| `SL-212 --depth 3` | 3484 x 10000 | 56% |
| whole corpus | 4214 x 10000 | 67% |

Splitting the two would let a capped-cost raster be *placed* across the full
window width. The kitty protocol scales the source into the `c=`/`r=` cell
rectangle, so this costs nothing extra on the wire — the image goes softer
rather than smaller.

## Why it is not obvious

`place` computing columns from the raster's own size is what makes it total and
distortion-free today. Forcing the placement to full width means deriving rows
from the aspect ratio instead, and a tall narrow graph would then become very
tall in rows — possibly worse than not filling the width. The rule probably
wants a bounded upscale factor rather than "always fill", and picking that bound
is the actual design question.

## Prerequisites

- `DEC-256` is the governing decision and would change; it was already recorded
  provisional, pending human acceptance on `SL-245` PHASE-05.
- The per-side cap and the silent-refusal behaviour are recorded in
  `mem.fact.kitty-graphics.image-size-limit` — read it before touching the
  bounds, because neither area nor byte count predicts what a terminal accepts.

## Origin

`SL-245` human acceptance. The render itself was accepted at 4K with a small
font at `--depth 3`; this is the residual, not a defect.
