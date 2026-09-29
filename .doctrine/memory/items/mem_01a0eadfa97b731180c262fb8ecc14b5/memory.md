`#[path = "../src/x/mod.rs"] mod x;` compiles the module's `#[cfg(test)] mod tests` too, because the integration crate is itself built under `cfg(test)`. Consequences, measured 2026-09-29 (RV-411 F-5):
- `design_run::tests` (~280 tests) run in 11 binaries;
- `common::test_support::tests` run in all 106 integration binaries;
- 440 test names execute more than once;
- a design e2e binary is 94MB against 26MB for a comparable crate.

A single unit failure is reported about 11 times.

How to apply:
- Don't add `#[cfg(test)]` tests to a module that integration tests `#[path]`-include. Put them in a file declared only from `main.rs`, or include only the leaf modules you need.
- To check: `cargo test --test <crate> -- --list | grep '<module>::'`.

Tracked: IMP-502. Root cause: no lib boundary (IMP-404).

Related: [[mem.fact.design-run.e2e-counts-embed-the-unit-suite]] (the derived-count consequence for phase baselines).
