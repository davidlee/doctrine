`tests/e2e_design_state.rs`, `e2e_design_review.rs`, `e2e_design_runbook.rs`,
`e2e_design_checkpoint.rs` and `e2e_design_delegation.rs` each carry:

```rust
#[path = "../src/design_run/mod.rs"]
mod design_run;
```

That include pulls in `#[cfg(test)] mod tests` (and every other in-module test
in the `design_run` tree). So:

**binary count = its own `#[test]`s + 2 (`common::test_support`) + the whole
`design_run` unit suite.**

`e2e_design_evaluation.rs` does not embed `design_run`, so it moves only with
`common`.

## What this breaks

A phase sheet that records "the e2e baseline is `108/76/4/76/76`" is recording a
**derived** number. Add one unit test in `src/design_run/` and four of those
counts move by one each, with nothing wrong. SL-244 PHASE-01 and PHASE-02 each
recorded a tuple that did not reproduce at the next phase, and the mismatch was
carried in `notes.md` as unexplained until PHASE-03 traced it here.

The second half of the same trap: `cargo test --test a --test b --test c` does
**not** report results in flag order, so a tuple transcribed from that output can
also be scrambled across binaries. Measure one binary per invocation.

## What to do instead

- Compare **green to green**, not count to count. A count is a control only when
  you can say what it is a count *of*.
- If a count is worth recording, measure it per binary
  (`cargo test --test <one>`) and say what the arithmetic is.
- An exhaustive table test — `every_material_event_kind_persists_a_change_row`,
  `the_writer_act_table_covers_every_key_writer_act_checks` — is the real control
  for "did I wire the new member". Those fail loudly and name the gap; a count
  does not.

See [[mem.pattern.doctrine.tdd-loop]] and
[[mem.pattern.dispatch.verify-governance-freshness-before-distilling-worker]].
