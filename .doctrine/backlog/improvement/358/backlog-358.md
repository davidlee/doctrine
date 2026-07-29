# IMP-358: architecture_layering reports no tangle count when green

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`tests/architecture_layering.rs` surfaces the per-tier tangle count **only**
through `Violation::TangleGrew`, i.e. only when the measured value exceeds the
baseline in `.doctrine/adr/001/layering.toml`. A green run prints nothing, so
green proves only `actual <= baseline` — never the actual number.

Two consequences:

1. **A tangle *drop* is invisible.** Work that reduces coupling should prompt
   tightening the baseline; nothing reports that it happened, so baselines only
   ever ratchet loose.
2. **Proving a specific number requires forcing a failure.** SL-233 PHASE-14
   needed to show that moving `run_design` into the command tier added zero
   cyclic edges. The worker could not read the count from a green run, and
   `layering.toml` is a forbidden zone for a worker, so it ran the compiled test
   binary against a scratch tree — `src`/`Cargo.toml` symlinked, a *copy* of
   `layering.toml` with `command = 0` — purely to make the checker print:

   ```
   TangleGrew { tier: Command, baseline: 0, actual: 76 }
   ```

   76, exactly the untouched baseline, against the 135 measured for the
   rejected `slice -> commands` direction. Correct answer, absurd route.

One unconditional print of the per-tier counts would close both.

## Related

- IMP-198 (harden the gate against a pre-existing-red blind spot) and RSK-227
  (blind to intra-tier concentration) are adjacent observability gaps in the
  same checker; neither covers the green-run silence.
- Surfaced by SL-233 PHASE-14 (worker finding W2-F5).
