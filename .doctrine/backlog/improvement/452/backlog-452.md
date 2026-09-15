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
