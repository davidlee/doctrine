# IMP-364: Extract a shared design-run e2e fixture

Six integration tests each carry their own private `Fixture` over the same
protocol — `design start` into a tempdir, learn the run uid and snapshot path
from the command's own stdout, `payload()` / `apply()` / `refuse()`, and a
`#[path]`-included pure model:

- `tests/e2e_design_state.rs`
- `tests/e2e_design_checkpoint.rs`
- `tests/e2e_design_import.rs`
- `tests/e2e_design_materialise.rs`
- `tests/e2e_design_projection.rs`
- `tests/e2e_design_review.rs`  (SL-233 PHASE-12, the sixth)

Each has helpers the others do not, so the duplication is not total — but the
envelope construction, the uid/snapshot discovery, and the run/fail spawn pair are
the same code six times. A shared `tests/common/design.rs` (the `tests/common/mod.rs`
idiom already in use) would own those three.

**Why PHASE-12 did not do it:** the extraction touches four completed phases'
committed test files, and the behaviour-preservation gate makes those suites the
proof of the machinery they cover. Worth doing as its own change, with the six
suites green before and after, rather than as a passenger on a phase that needed a
seventh fixture.
