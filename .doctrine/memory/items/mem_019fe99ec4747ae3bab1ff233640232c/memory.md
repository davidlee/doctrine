Cost a full false-red round in SL-248 PHASE-09 `T9`.

A mutation battery runs build → mutate → run → restore cycles. The restore
leaves the **source newer than the last build**, and `doctrine check gate` builds
and tests its *own* artefact rather than the one an earlier `cargo test --no-run`
produced. A shell variable holding `target/debug/deps/<crate>-<hash>`, captured
once, therefore keeps pointing at the **last mutant's** binary.

The symptom is maximally misleading. Five consecutive "load tally" runs came back
red, identically, carrying the exact panic the mutant produces — which reads as a
genuine load-dependent failure, because a mutant's failure and a real failure are
the same assertion firing. The unloaded control runs failed identically, and that
is what finally broke the reading.

The tell: `ls -la --time-style=full-iso <binary> <source>`. A binary that
predates the source is conclusive.

Rules:

- rebuild immediately after every restore, **before** any run;
- re-resolve the executable path from `cargo test --no-run` output on each use,
  never cache it;
- treat "a whole batch failed identically" as a stale-artefact hypothesis before
  a load hypothesis.

This is the failing-open-selector hazard one level over: a pinned artefact path,
like an `--exact` name that selects zero tests, fails silently in the direction
that looks like a result.
