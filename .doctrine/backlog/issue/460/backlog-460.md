# ISS-460: Render fit box overshoots the terminal per-side cap by graphviz rounding

`doctrine graph RFC-025 -X` refuses on a graph that would render:

```
Error: --render needs a smaller graph: 'dot' produced a 1294x10002 image and the
terminal takes at most 10000 px on a side and 80 Mpx in total; narrow it with a
focus id or --depth, or drop -X to emit DOT
```

## Mechanism

`fit_box` (`src/terminal_image.rs`) clamps the box it asks `dot` for to exactly
`MAX_IMAGE_DIMENSION` (10000 px, the measured kitty/ghostty per-side limit).
`raster_args` (`src/graphviz.rs`) states that box to graphviz as `-Gsize` in
inches. Graphviz scales the drawing to fit and rounds the resulting bitmap **up**
— it returns 1–2 px *over* the box it was handed. `check_image_size` then
compares `<= MAX_IMAGE_DIMENSION` and refuses.

So the backstop written for "a `dot` that ignores `-Gsize`" fires on the box
doctrine itself requested. The invariant asserted in `MAX_IMAGE_PIXELS`' doc
comment — "`fit_box` scales every drawing inside both by construction" — is off
by graphviz's rounding.

## Evidence

The real raster, reproduced by piping `doctrine graph RFC-025` DOT to
`dot -Tpng -Gdpi=<n> -Gsize=<w>,<10000/n to 4dp>`:

- dpi 241, 249, 251, 255, 256, 257, 268, 269, 310 → `1294 x 10002` (the reported size)
- ~50 further dpi values in 100..320 → `1294 x 10001`

`raster_dpi` is `72 * cell_height / 14`, so a cell height of roughly 47–60 px
(high-DPI window, large font) lands in the overshoot band. Same graph in a
smaller font renders.

## Fix

Keep the guard at the terminal's real limit; give the fit box headroom under it
so graphviz's round-up lands inside. Secondarily, floor the px→inch conversion
in `raster_args` rather than `{:.4}` round-half, so the stated box never encodes
*more* pixels than the caller asked for.

Follow-on to ISS-459 (size the `-X` raster by dpi rather than stretching it to
the window), which made natural drawing size — not the window — decide height,
and so made the cap reachable by rounding.
