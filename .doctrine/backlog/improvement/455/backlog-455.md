# IMP-455: graph label flags: clamp raster size, titles off

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

IMP-454 put titles on every `doctrine graph` node unconditionally. That
inflates the raster — for `SL-245 --depth 1`, monospace titles took it from
781×1423 to 1417×2696 — and `-X` fits the image to the window, so a larger
source raster is downscaled harder and can land *smaller* on screen.

Wanted, once there is evidence about which bound actually bites:

- a flag to suppress titles (back to id-only labels);
- a clamp on label height — cap a wrapped title at N lines with an ellipsis,
  which also bounds node height on long titles (`DEC-256`'s wraps to five
  lines, `RV-369`'s likewise);
- possibly a wrap-column override, since `LABEL_WRAP_COLS` is currently a
  fixed 22.

Deferred deliberately at IMP-454 D1 — default-on first, bound it once the
real complaint is known.
