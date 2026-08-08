# ISS-331: cordage scale_cliffs asserts wall-clock and false-reds under load

`crates/cordage/tests/scale_cliffs.rs:279` —
`evaluate_scales_near_linearly_in_node_count` asserts that doubling the node
count costs roughly 2x wall time, and fails above a threshold.

## Observed

At `SL-249` `PHASE-06`'s close the gate returned exit 101:

```
evaluate ratio 3.2x for 2x nodes (near-linear; quadratic ≈ 4x)
evaluate doubled to 3.2x for 2x nodes — near-quadratic regression (expect ~2x)
```

The phase's diff touches only `src/commands/design.rs` and
`src/design_run/submission.rs` in the root package and cannot reach `cordage`.
Re-run three times alone on an idle machine: passed every time, ~1.03s. The full
gate re-run then returned exit 0 across 119 test binaries.

## Why it matters

This repo's conventions assume multiple agents work in one tree concurrently
(`AGENTS.md`: "assume multiple agents are working in the same repository"), and
dispatch runs workers in parallel worktrees each building into its own
`target/`. A wall-clock ratio assertion is therefore load-sensitive **by
construction**, and its failure mode is the expensive one: it reds a gate that
an agent must then prove innocent, costing a full re-run plus an isolation
experiment.

The second-order cost is worse than the first. `LOOP.md` teaches agents that a
truthful green report loses to a fresh gate — the right instinct. A gate that
reds for reasons unrelated to the diff teaches the inverse: that a red gate is
probably flaky. That is precisely the habit that lets a real regression through.

## Options

1. **Count operations rather than wall time.** The property under test is
   algorithmic complexity, which is countable without a clock. Removes the
   coupling rather than tolerating it.
2. **Keep the timing shape, `#[ignore]` by default**, and run it in a dedicated
   unloaded lane. Cheap interim.
3. **Widen the threshold.** Cheapest and weakest: it lowers sensitivity to the
   very regression the test exists to catch, and leaves the load coupling.

Recommendation: (1) on the merits, with (2) as the interim if (1) is not
immediately affordable.
