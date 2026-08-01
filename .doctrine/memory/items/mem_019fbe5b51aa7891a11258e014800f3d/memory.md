When an evidence harness has a legal recorded value for "nothing to measure
here" — `n/a`, `skipped`, `not applicable` — it is tempting to reuse it for
"this case is not written yet". Do not. Refuse instead, up front, naming the
case and the task that provides it.

**Why.** The legal value is not free: it is excluded from whatever the table
computes, so an unwritten case recorded that way silently *lowers the claim*.
In SL-241's P-C3 matrix an `n/a` cell is excluded from the altitude
computation, so an unwritten row would have read `unproven-beyond-rust` — a
finding about portability — for a reason that was really about the order the
rig was built in. That is the most damaging direction for the error to run: it
looks like the answer the exercise was built to find, so it is the finding
least likely to be re-examined.

Worse, the run stays GREEN throughout, so nothing prompts the re-examination.

**The rule.** A legal skip value names a STRUCTURAL absence — there is no
`.envrc` to plant, there is no artifact to read — and states it. "Not
attempted" is not a structural absence. Keep them distinguishable:

- structural absence  → record it, with the reason, in the results
- not implemented yet → refuse (usage exit), naming what is missing
- deliberately narrowed this run → run it, and record the NARROWING in the
  output itself, so a partial run cannot be mistaken for a complete one

That third case is the one people skip. If the scope of a run is a choice, the
choice belongs in the artifact, not only in the operator's memory: SL-241's
harness writes its leg selector into the results preamble for exactly that
reason.

Same family as [[mem.pattern.harness.grep-negative-needs-positive-control]] —
both are about an absence that reads as a result.
