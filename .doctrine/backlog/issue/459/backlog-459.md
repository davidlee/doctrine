# ISS-459: -X stretches a small graph to fill the terminal

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`doctrine graph IMP-104 -X` — a single-node graph — drew one box filling the
whole terminal width, its label several times the size of the surrounding
text.

## Cause

`graphviz::raster_args` emitted `-Gsize=<w>,<h>!`. The trailing `!` makes the
size box a *minimum as well as a maximum*, so graphviz stretches any smaller
drawing up to meet it. Since the box is the window's usable width, every
graph filled the window regardless of how little it contained.

The `!` was deliberate (SL-245): without it, `size` only ever scales down, and
a small graph rendered native-and-tiny — the symptom the fit box was added to
fix. Both symptoms are real; the mistake was making one knob serve two jobs.
Apparent size and overflow are independent concerns:

- **apparent size** is the resolution the drawing is rasterised at;
- **overflow** is the box it must not exceed.

## Fix

`FitBox` carries a `dpi` alongside its pixel box, and `terminal_image`
computes it from the terminal's cell height so a 14pt label — graphviz's
default, which the emitter never overrides — lands about one terminal row
tall. Graph text then reads at the size of the text around it, at any window
DPI. The box loses its `!` and becomes a pure maximum: it shrinks a graph too
big for the window and leaves a small one alone.

`RASTER_DPI` (a fixed 384, documented as "not a knob for apparent size") is
replaced by `MIN_RASTER_DPI`, a floor guarding against a terminal reporting an
implausibly small cell.

## Revises

DEC-256's fill rule, as implemented. The decision's own wording — "native if
it fits, else scale to columns minus one" — is what this restores; the `!`
went past it.
