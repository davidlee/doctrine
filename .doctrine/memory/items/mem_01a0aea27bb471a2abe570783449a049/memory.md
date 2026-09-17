`-Gsize=<w>,<h>` bounds a drawing: graphviz scales it **down** to fit and
leaves anything smaller alone.

`-Gsize=<w>,<h>!` — with the trailing bang — makes the box a **minimum as
well as a maximum**. Every drawing is stretched to meet it, however little it
contains. A one-node graph comes out the full width of the box, its text
several times the size of everything around it (ISS-459).

So the bang is not "fit to the window". It is "always exactly this size".

**Apparent size is `-Gdpi`'s job, not `-Gsize`'s.** The two are independent
and want to stay that way:

| concern | knob |
|---|---|
| how big the text comes out | `-Gdpi=<n>` |
| what it must not overflow | `-Gsize=<w>,<h>` (no bang) |

To render a graph at the size of the surrounding terminal text, pick the dpi
that draws a label one cell tall — `dpi = points_per_inch * cell_height /
label_point_size`, with graphviz's default label size being 14pt unless the
emitter sets `fontsize`. Then let `-Gsize` bound it.

Reaching for the bang to fix "my graph renders too small" swaps one wrong
apparent size for another; it is a sizing knob only by accident.

See [[mem.fact.kitty-graphics.image-size-limit]] for the terminal-side per-side cap
that bounds the box from the other direction.
