# Invariant test must knock on the wall: drive run(), not a pure helper

A test that claims to protect a behavioural invariant must exercise the real
seam where the invariant could be violated — for doctrine that is the impure
shell entrypoint (`run()`), not a pure helper called in isolation.

**The trap (SL-044 / RV-004 F-1).** NF-001's "coverage never reaches authored
status through a function" was guarded by VT-5
`written_status_is_verdict_independent`, which built varied coverage, computed a
`Verdict`, then asserted `select_status(fixed_to, prior) == fixed_to`. That is
`id(x) == x` — a tautology. The actual laundering surface is the call site in
`run()` (`select_status(...) → set_status(...)`). A future edit to
`select_status(status_from_verdict(verdict), prior)` would leave the test green.
The wall had no test that knocked on it.

**The fix.** Drive `run()` end-to-end: hold the authored input (`--to`) fixed,
vary the on-disk state that feeds the dangerous derivation (coverage → verdict),
and assert the *persisted* result (requirement status on disk) tracks only the
authored input. Reuse an existing end-to-end harness (VT-6 supplied
`repo/mint_req/mint_slice/write_coverage`). Add a non-vacuity guard (the varied
inputs produced ≥N distinct intermediate values) so the test can't silently
regress to constant inputs.

**Smell test.** If an "independence" or "invariant" test never calls the impure
entrypoint, or asserts `f(const) == const` on the very function whose job is to
ignore an argument, it proves nothing about the call site. See
[[mem.pattern.review.interaction-bugs-hide-between-sound-parts]] — the bug lives
at the seam, not in the component.
