// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine-control` — the capsule control binary (SL-248, `DEC-153`).
//!
//! Skeleton: this crate exists from PHASE-01 so that every later phase's tests
//! are inside the checked set from their first commit. The two verbs land with
//! the code behind them — `provision` in PHASE-06, `backend verify` in PHASE-10 —
//! and nothing is wired here until then.
//!
//! **Bin-only, permanently** (`sec-6`). A `tests/` file cannot link a bin-only
//! package (`E0433`), and adding a lib target to rescue one would force the
//! conformance suite's weakening vocabulary public (`E0603`). Every test in this
//! crate is a `#[cfg(test)]` module inside the unit it tests.

mod backend;
mod capacity;
mod config;
mod host;

fn main() {}
