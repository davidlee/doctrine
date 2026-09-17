# IMP-452: Bounded subprocess helper and render timeout

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

Cut from SL-245 on 2026-09-15. SL-245's CLI render (`graphviz::rasterise_png`)
runs `dot -Tpng` with no deadline: `-X` is interactive, so Ctrl-C is the
timeout. Two things were deferred with that cut:

1. **A CLI render deadline** (`RENDER_TIMEOUT`, a `TimedOut` outcome and its
   message) — only worth it if a hung `dot` shows up in practice.
2. **The bounded-subprocess extraction.** `coverage_verify::run_argv` owns a
   bounded synchronous spawn whose timeout cleanup calls a blocking
   `child.wait()` after an unchecked `kill`: a child that survives `SIGKILL`
   (uninterruptible I/O) holds it indefinitely. That latent defect stands on its
   own, independent of rendering.

## Already designed

SL-245 design sec-5 *One bounded-subprocess helper, not a second copy*: a leaf
`subprocess::run_bounded(command, stdin, timeout) -> io::Result<Bounded>` with
an unjoined stdin writer thread, per-pipe drain threads, a 50 ms poll interval
and a bounded reap (kill, then `try_wait` for a 1 s grace, never a blocking
wait). `coverage_verify` delegates to it with its `RunResult` mapping
unchanged; stdin is piped only when bytes are supplied, so `coverage_verify`
keeps inheriting stdin. If (1) is wanted, `rasterise_png` then rides the same
helper rather than growing its own timeout.

## Related

ISS-455 (descendant-held pipes extend the drain joins) is the other known gap in
the same mechanism; fix them together if convenient.

## Annotation — the calculus changed (SL-245 reconcile, 2026-09-17)

`RV-369` `F-6`. Deferral item (1), the CLI render deadline, was weighed on the
assumption that the slow path ends in an **image**: wait 36 s for the whole
corpus, get your picture, and Ctrl-C is there if you lose patience.

It can now end in a **refusal**. `F-6` added an 8 MiB budget on the rasterised
PNG (the whole corpus rasterises to 62 MiB, which ghostty silently declines),
so the most obvious thing a user types — `doctrine graph -X`, no focus — spends
~36 s in graphviz and then prints a refusal. Waiting that long to be told no is
worse than waiting that long for a picture, which is the trade this deferral
actually made.

That does not reverse the deferral, and it is deliberately not a byte-budget
problem to solve here: a pre-spawn bound on DOT size would refuse in 2 s instead
of 36, but DOT size is only a proxy and could refuse a graph that renders
perfectly well — accuracy beat latency (`RV-369` synthesis). Re-read this item's
priority knowing the outcome can be a refusal, not just a slow success.

Deferral item (2), the bounded-subprocess extraction, is unaffected: its
`coverage_verify` blocking-`wait` defect stands exactly as described above.
