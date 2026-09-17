Measured in both terminals with `q=0` (replies enabled), transmitting
solid-colour PNGs of known dimensions.

## The bound is per-side

Same area, opposite verdicts — identical in ghostty and kitty:

| raster | area | verdict |
|---|---|---|
| 8000 x 4000 | 32 Mpx | stored |
| 2000 x 16000 | 32 Mpx | refused |
| 32000 x 1000 | 32 Mpx | refused |
| 1000 x 32000 | 32 Mpx | refused |

Tallest accepted: 6298 x 7621 (ghostty), 1760 x 9090 (kitty). Shortest
refused: 6298 x 10161, 1760 x 13636. That brackets the cap in `[9090, 10161)`
— kitty's documented `MAX_IMAGE_DIMENSION` of 10000 px, which ghostty mirrors.

So **neither area nor byte count predicts acceptance.** Two traps follow:

- A PNG byte budget is the wrong unit twice over: terminals budget DECODED
  pixels, and PNG compresses a sparse line drawing by two orders of magnitude
  at a ratio that swings with density. A 3.09 MiB PNG passed an 8 MiB budget
  and drew nothing.
- An area budget inverts under window width. If a fit box takes width from the
  window and derives height as `budget / width`, a WIDER window yields a
  TALLER box — straight past the per-side cap. This shipped a blank screen on
  a 6270 px high-DPI window.

## ghostty does not answer

kitty replies `ENOMEM:PNG image is too large`. **ghostty sends nothing** — not
even with `q=0`. So there is no runtime signal to react to on the terminal
doctrine actually targets: an oversized image must be refused BEFORE the write,
never handled as an error after it. This is the standing justification for the
pre-send guard in `terminal_image::check_image_size`.

Note also that production runs `q=2` precisely so a terminal's refusal is not
typed into the user's shell — which means even kitty's reply is invisible in
the real pipeline.

## Reproducing

Transmit with `a=t,q=0,i=<id>`, read the reply in raw mode with a timeout,
delete with `a=d,d=I,i=<id>` between probes so nothing accumulates against the
storage quota. Treat "no reply within the timeout" as a refusal, not an error.
