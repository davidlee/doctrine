`dot` scales a drawing to fit `-Gsize` and then rounds the resulting bitmap
**up**. The raster it hands back is routinely 1–2 px LARGER on the binding side
than the box it was given.

Measured on `doctrine graph RFC-025` DOT, sweeping dpi 100..320 against a
10000-px box: +1 px at ~50 resolutions, +2 px at nine. That is what made
`doctrine graph RFC-025 -X` refuse with `'dot' produced a 1294x10002 image` when
`fit_box` asked for exactly 10000 (ISS-460).

**So: never fit to a hard cap exactly.** If a bound is enforced downstream —
kitty/ghostty's 10000-px per-side limit, cairo's 32767 — the box handed to `dot`
must sit a few pixels under it. `src/terminal_image.rs` reserves
`RASTER_ROUNDING_SLACK` (4 px) on each side, and subtracts the matching area
(`slack * (w + h + slack)`) from the pixel budget too, since an overshoot on one
side enlarges the area as well.

Rounding the px→inch conversion down instead does NOT fix this: the 4-dp
conversion error is a hundredth of a pixel, while the overshoot is graphviz's
own bitmap rounding.
