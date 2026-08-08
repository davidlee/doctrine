// SPDX-License-Identifier: GPL-3.0-only
//! `host` — the impure inputs provisioning is given, named as one contract
//! (SL-248 `sec-5` § `HostFacts`, `EX-1`/`EX-2`).
//!
//! Naming them once is what makes every pure step above this module testable
//! against a table rather than a filesystem: `config`'s capsule-root resolution
//! and `capacity`'s tiers are decided from these four answers and nothing else.
//!
//! **No clock** (`EX-2`, `VA-1`). Provisioning needs no wall-clock read — the
//! transaction id comes from a collision-resistant source, not a timestamp — and
//! `src/clock.rs` in the root package is already this project's single home for
//! wall-clock reads, under the rule that the *value* is passed in rather than a
//! clock handed down. A `now()` here would be a second one.
//!
//! Layering (`ADR-001`, `sec-6`'s unit table): `host` is `leaf` with **out=0**.
//! It imports nothing from this crate, which is what keeps `config → host` and
//! `capacity → host` acyclic. [`CapacityUnknown`] lives here rather than in
//! `capacity` because it is [`HostFacts::available_bytes`]'s error type; putting
//! it above `host` would close a cycle, and its placement here is precisely what
//! explains the `capacity → host` edge the table records (PHASE-03 `D2`).
//!
//! The figure is a raw `u64`, not `config::ByteCount` (PHASE-03 `D1`): `EX-3`
//! puts `ByteCount` in `config`, and `config → host` is forced by `EX-6`, so
//! returning the typed newtype here would make a two-node leaf-tier cycle that
//! the layering gate's zero tangle baseline rejects outright. The caller does the
//! one `ByteCount` conversion.
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "staged ahead of PHASE-06's provision consumer (PHASE-03 D5); \
                  PHASE-06 deletes this line when `provision` lands"
    )
)]

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

/// Why an available-space probe could not produce a usable figure.
///
/// A named outcome, not an error swallowed into "ample" (`POL-002` facet 3,
/// `DEC-158`): filesystems are diverse and not every `statvfs` field is
/// meaningful on every one. The two outcomes a silent implementation makes
/// identical — a probe that failed and a probe that returned plenty — are
/// opposite to an operator, so the reason travels with the verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CapacityUnknown {
    /// `statvfs` itself failed; the raw errno is carried so the operator can
    /// tell `ENOENT` from `EACCES` without re-running the probe.
    ProbeFailed { errno: i32 },
    /// The filesystem reported a zero fragment size, so `f_bavail` counts in a
    /// unit of zero bytes and the product is meaningless.
    UnusableFigure,
    /// `f_bavail × f_frsize` exceeds `u64`.
    FigureOverflows,
}

/// The impure inputs provisioning is given — exactly four, and no clock.
pub(crate) trait HostFacts {
    /// Available bytes at `path`, or why the probe could not answer.
    ///
    /// Raw `u64` rather than `config::ByteCount`; see the module docs (`D1`).
    fn available_bytes(&self, path: &Path) -> Result<u64, CapacityUnknown>;

    /// Fully resolve a path through its symlink components (`sec-2` rule 1).
    fn resolve(&self, path: &Path) -> io::Result<PathBuf>;

    /// Whether `path` exists.
    fn path_exists(&self, path: &Path) -> bool;

    /// An environment value, as an `OsString` — `var_os`, never `std::env::var`,
    /// which this repository bans through `disallowed_methods` (`clippy.toml`,
    /// precedent `src/tty.rs:41`).
    fn env_var(&self, name: &str) -> Option<OsString>;
}

/// The production [`HostFacts`]: real syscalls, no policy.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SystemHost;

impl HostFacts for SystemHost {
    /// `rustix::fs::statvfs`, available bytes as `f_bavail × f_frsize`
    /// (`EX-15`).
    ///
    /// `f_bavail` is the count available to an *unprivileged* process, which is
    /// the honest figure — `f_bfree` includes the reserved blocks a capsule
    /// cannot have. `f_frsize` is the fragment size and is the unit `f_bavail`
    /// counts in; `f_bsize` is the preferred I/O size and the wrong multiplier.
    /// The multiplication is **checked**: overflow is a named outcome, not a
    /// wrap into a plausible-looking small figure.
    fn available_bytes(&self, path: &Path) -> Result<u64, CapacityUnknown> {
        let stat = rustix::fs::statvfs(path).map_err(|errno| CapacityUnknown::ProbeFailed {
            errno: errno.raw_os_error(),
        })?;
        if stat.f_frsize == 0 {
            return Err(CapacityUnknown::UnusableFigure);
        }
        stat.f_bavail
            .checked_mul(stat.f_frsize)
            .ok_or(CapacityUnknown::FigureOverflows)
    }

    fn resolve(&self, path: &Path) -> io::Result<PathBuf> {
        std::fs::canonicalize(path)
    }

    fn path_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn env_var(&self, name: &str) -> Option<OsString> {
        std::env::var_os(name)
    }
}

/// The table-driven [`HostFacts`] every pure test in this crate runs against.
///
/// Test-only and deliberately so: it is what lets `config`'s root resolution and
/// `capacity`'s tiers be asserted without a disk of a particular size or a
/// mutated process environment (`std::env::set_var` is a disallowed method, and
/// it is test-hostile besides). `BTreeMap`/`BTreeSet`, never the hashed pair —
/// `clippy.toml` `disallowed-types`. PHASE-06 consumes this too.
#[cfg(test)]
pub(crate) mod fixture {
    use std::collections::{BTreeMap, BTreeSet};
    use std::ffi::OsString;
    use std::io;
    use std::path::{Path, PathBuf};

    use super::{CapacityUnknown, HostFacts};

    #[derive(Debug, Default, Clone)]
    pub(crate) struct FixtureHost {
        available: BTreeMap<PathBuf, Result<u64, CapacityUnknown>>,
        resolutions: BTreeMap<PathBuf, PathBuf>,
        existing: BTreeSet<PathBuf>,
        env: BTreeMap<String, OsString>,
    }

    impl FixtureHost {
        /// An empty host: no space answers, no resolutions, nothing exists, and
        /// **no environment at all** — which is the interesting default, since
        /// it is the case that must refuse rather than guess a capsule root.
        pub(crate) fn new() -> Self {
            Self::default()
        }

        pub(crate) fn with_env(mut self, name: &str, value: &str) -> Self {
            self.env.insert(name.to_owned(), OsString::from(value));
            self
        }

        pub(crate) fn with_available(
            mut self,
            path: &str,
            answer: Result<u64, CapacityUnknown>,
        ) -> Self {
            self.available.insert(PathBuf::from(path), answer);
            self
        }

        pub(crate) fn with_resolution(mut self, from: &str, to: &str) -> Self {
            self.resolutions.insert(PathBuf::from(from), PathBuf::from(to));
            self.existing.insert(PathBuf::from(from));
            self
        }
    }

    impl HostFacts for FixtureHost {
        fn available_bytes(&self, path: &Path) -> Result<u64, CapacityUnknown> {
            self.available
                .get(path)
                .copied()
                .unwrap_or(Err(CapacityUnknown::UnusableFigure))
        }

        fn resolve(&self, path: &Path) -> io::Result<PathBuf> {
            self.resolutions.get(path).cloned().ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, format!("{}", path.display()))
            })
        }

        fn path_exists(&self, path: &Path) -> bool {
            self.existing.contains(path)
        }

        fn env_var(&self, name: &str) -> Option<OsString> {
            self.env.get(name).cloned()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::fixture::FixtureHost;
    use super::{CapacityUnknown, HostFacts, SystemHost};

    const ENV_NAME: &str = "DOCTRINE_CAPSULE_FIXTURE";
    const ENV_VALUE: &str = "/somewhere/absolute";
    const A_PATH: &str = "/a/path";
    const A_TARGET: &str = "/a/resolved/path";

    /// `VT-8`: the four-method contract answered from a table, which is the
    /// fixture every pure test in this phase and in PHASE-06 depends on.
    #[test]
    fn the_fixture_answers_all_four_methods_from_its_tables() {
        let host = FixtureHost::new()
            .with_env(ENV_NAME, ENV_VALUE)
            .with_available(A_PATH, Ok(4096))
            .with_resolution(A_PATH, A_TARGET);

        assert_eq!(host.available_bytes(Path::new(A_PATH)), Ok(4096));
        assert_eq!(
            host.resolve(Path::new(A_PATH)).ok(),
            Some(Path::new(A_TARGET).to_path_buf())
        );
        assert!(host.path_exists(Path::new(A_PATH)));
        assert_eq!(host.env_var(ENV_NAME).as_deref(), Some(ENV_VALUE.as_ref()));

        // An empty host answers "no" to everything and refuses to invent a figure.
        let empty = FixtureHost::new();
        assert_eq!(
            empty.available_bytes(Path::new(A_PATH)),
            Err(CapacityUnknown::UnusableFigure)
        );
        assert!(empty.resolve(Path::new(A_PATH)).is_err());
        assert!(!empty.path_exists(Path::new(A_PATH)));
        assert_eq!(empty.env_var(ENV_NAME), None);
    }

    /// `VT-8`: `SystemHost` implements the same contract with real syscalls.
    ///
    /// A smoke test, deliberately weak: `sec-5`'s Table C rows — agreement with
    /// an independent `statvfs` within one allocation unit, and the
    /// cross-filesystem claim — are **PHASE-08's**, not this phase's. All this
    /// asserts is that the production implementation is reachable and answers.
    #[test]
    fn system_host_answers_the_contract_with_real_syscalls() {
        let host = SystemHost;
        let root = Path::new("/");

        assert!(host.available_bytes(root).is_ok());
        assert!(host.path_exists(root));
        assert_eq!(host.resolve(root).ok(), Some(root.to_path_buf()));
        // PATH is not guaranteed, so assert the seam rather than a value: an
        // absent variable is `None`, never a panic and never an empty string.
        let _ = host.env_var(ENV_NAME);
    }

    /// A probe that cannot answer says *which* of the three reasons, because a
    /// silent implementation makes a failed probe and an ample one identical.
    #[test]
    fn the_three_unknown_reasons_are_distinct() {
        let reasons = [
            CapacityUnknown::ProbeFailed { errno: 2 },
            CapacityUnknown::UnusableFigure,
            CapacityUnknown::FigureOverflows,
        ];
        for (i, left) in reasons.iter().enumerate() {
            for (j, right) in reasons.iter().enumerate() {
                assert_eq!(i == j, left == right);
            }
        }
    }
}
