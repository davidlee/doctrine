# IMP-444: Report the measured tangle count, don't make callers force a violation

`tests/architecture_layering.rs`'s per-tier ratchet raises `Violation::TangleGrew`
only when `actual > baseline`, so a green suite proves `actual <= baseline` — a
one-sided bound. Nothing reports the number itself: `dump_real_graph` (`#[ignore]`)
prints units and edges but no SCC/tangle count.

That is correct for a ratchet — a *drop* must never fail a build — but it leaves no
way to answer "is the count **unchanged** at N", which is what a slice doing a
routing repair actually needs. SL-238 PHASE-08's `VA-3` was worded exactly that way.

**The workaround it forces.** Set the tier's baseline to `0` in
`.doctrine/adr/001/layering.toml`, run the gate, read `TangleGrew { baseline: 0,
actual: N }` out of the failure, revert, and confirm with `git diff --stat` — i.e.
mutate an authored governance file in order to perform a read. It works and it is
recorded (`mem.fact.layering.ratchet-green-is-one-sided`), but it is a poor thing to
ask of anyone under time pressure: the file must be reverted, and nothing but the
author's care enforces that.

**The fix is small.** Have `dump_real_graph` — or a sibling `#[ignore]`d reporter —
print `count_tangle_edges` per tier beside the baseline, so the measurement is a
read rather than a mutation:

```
tier      measured  baseline
leaf             0         0
engine           0         0
command         76        76
```

Then a phase whose deliverable IS the number cites a printed measurement, and
`mem.fact.layering.ratchet-green-is-one-sided` shrinks to "run the reporter".

Raised at SL-238 PHASE-08 harvest, with the friction observation recorded alongside.
