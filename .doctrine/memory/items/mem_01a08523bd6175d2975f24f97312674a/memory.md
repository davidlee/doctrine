Design runs, dispatch journals and phase sheets are **runtime state**: gitignored,
disposable, and continuously accruing rows. An authored artefact — `design.md`, a
spec, an ADR — is committed and diffable, and nothing re-derives it. Put a count
over the first into the second and it is wrong by the next apply, with no signal.

`SL-256`'s design measured the migration surface for a retired wire token:
*"7 rows over 6 live snapshots"*. Accurate on 2026-08-16. By audit it was 9 rows
over 7 runs — grown by the slice's **own** design run and `SL-254`'s. (`RV-364`
`F-7`.)

**The mitigation is not a fresher number.** It is:

1. **Write the predicate beside the figure** so a reader can re-run the census
   instead of trusting it — `event = "acceptance_attested"` across
   `.doctrine/state/slice/*/design.toml`.
2. **Say it is a floor**, not a constant. Live runs only accrue.
3. **Do not let the argument turn on the count.** `SL-256`'s did not — the compat
   pin is at the whole-file tier precisely because *one* unrecognised token costs
   the whole snapshot regardless of how many rows carry it — which is why a stale
   figure was a nit rather than a hole.

Where the number genuinely must be current, its home is a **code doc** next to the
constant it justifies (`change_log.rs:180`), which a reader reaches through the
same build that would break if the vocabulary moved.

Related: [[mem.fact.doctrine.storage-tiers]].
