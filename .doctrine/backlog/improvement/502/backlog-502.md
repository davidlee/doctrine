# IMP-502: Path-included module test suites execute once per including binary

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

RV-411 F-5 (`doctrine show RV-411`). `#[path]`-including a module also includes
its `#[cfg(test)] mod tests`:
- `src/design_run/mod.rs` is included by 10 `tests/e2e_design_*.rs` crates, so
  its approximately 280 tests compile and run 11 times;
- `src/test_support.rs` carries 3 tests into all 106 integration binaries.

Measured 2026-09-29: 440 test names execute more than once, about 1,700
redundant executions. The e2e_design_state binary is 94MB against 26MB for a
comparable crate.

Root cause: `#[path]` is a workaround for the missing library boundary (see
IMP-404, deferred engine/leaf crate extraction). Cheap fixes:
- move test_support's tests into a file declared only from main.rs;
- include only the leaf modules that are needed, or put the included test
  module behind a cfg the includers don't set.

A structural option is to merge the design e2e crates into one binary.
