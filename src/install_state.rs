// SPDX-License-Identifier: GPL-3.0-only
//! `install_state` — the install-currency model (SL-282 PHASE-01).
//!
//! Compares the two install stamps (the tracked `installed-version` in
//! `doctrine.toml`, the local `.doctrine/state/install.toml` marker) with the
//! running binary's version and words the result. A pure leaf (ADR-001): no IO,
//! no process, no clock. The shells that read the stamps live in `install`.
//!
//! This module is the sole home of the advisory text (STD-001): every line a
//! caller prints for a [`Currency`] finding is built here, so the wording has one
//! owner and one test.

#![cfg_attr(
    not(test),
    expect(dead_code, reason = "SL-282 PHASE-02/03 wire the call sites")
)]

use std::path::{Path, PathBuf};

use semver::Version;

/// What one stamp store yielded. `Unreadable` is never folded into `Absent`
/// (STD-003): a stamp that exists but cannot be understood is reported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StampRead {
    Absent,
    Unreadable { path: PathBuf, reason: String },
    Version(Version),
}

/// One way the install disagrees with the running binary. No `Current` variant:
/// an empty finding list means current.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Currency {
    /// The local marker is absent: doctrine was never installed in this checkout.
    CheckoutNotInstalled,
    CheckoutOlder {
        checkout: Version,
    },
    CheckoutNewer {
        checkout: Version,
    },
    /// The tracked stamp is absent (a pre-stamp install).
    RepoUnstamped,
    RepoOlder {
        repo: Version,
    },
    RepoNewer {
        repo: Version,
    },
    Unreadable {
        path: PathBuf,
        reason: String,
    },
}

/// Where the local marker was looked for, so the advisory can say which checkout
/// to install in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MarkerSource {
    /// This checkout's own `.doctrine/state/`.
    Checkout,
    /// The main worktree at this root (this checkout is a linked worktree).
    Main(PathBuf),
    /// A linked worktree whose main worktree could not be located.
    Unlocated,
}

/// Parse a stamp's raw text (PURE). Invalid semver is `Unreadable`, never `Absent`.
pub(crate) fn parse_stamp(path: &Path, raw: &str) -> StampRead {
    match Version::parse(raw.trim()) {
        Ok(v) => StampRead::Version(v),
        Err(e) => StampRead::Unreadable {
            path: path.to_path_buf(),
            reason: e.to_string(),
        },
    }
}

/// Every finding for the two stamps against `binary`; empty means current.
/// Order: unreadable (local, then tracked), checkout, repo.
pub(crate) fn classify(tracked: &StampRead, local: &StampRead, binary: &Version) -> Vec<Currency> {
    let unreadable = [local, tracked].into_iter().filter_map(unreadable_finding);
    let checkout = checkout_finding(local, binary);
    let repo = repo_finding(tracked, binary)
        // Installing writes the stamp and the commit hint carries it: when the
        // checkout needs installing, an unstamped repo adds nothing.
        .filter(|f| {
            !(*f == Currency::RepoUnstamped && checkout == Some(Currency::CheckoutNotInstalled))
        });
    unreadable.chain(checkout).chain(repo).collect()
}

fn unreadable_finding(read: &StampRead) -> Option<Currency> {
    match read {
        StampRead::Unreadable { path, reason } => Some(Currency::Unreadable {
            path: path.clone(),
            reason: reason.clone(),
        }),
        StampRead::Absent | StampRead::Version(_) => None,
    }
}

fn checkout_finding(local: &StampRead, binary: &Version) -> Option<Currency> {
    match local {
        StampRead::Absent => Some(Currency::CheckoutNotInstalled),
        StampRead::Version(v) if v < binary => Some(Currency::CheckoutOlder {
            checkout: v.clone(),
        }),
        StampRead::Version(v) if v > binary => Some(Currency::CheckoutNewer {
            checkout: v.clone(),
        }),
        StampRead::Unreadable { .. } | StampRead::Version(_) => None,
    }
}

fn repo_finding(tracked: &StampRead, binary: &Version) -> Option<Currency> {
    match tracked {
        StampRead::Absent => Some(Currency::RepoUnstamped),
        StampRead::Version(v) if v < binary => Some(Currency::RepoOlder { repo: v.clone() }),
        StampRead::Version(v) if v > binary => Some(Currency::RepoNewer { repo: v.clone() }),
        StampRead::Unreadable { .. } | StampRead::Version(_) => None,
    }
}

const INSTALL_CMD: &str = "`doctrine install`";
const NOT_INSTALLED: &str = "doctrine isn't installed in this checkout: run";
const MAIN_NOT_LOCATED: &str = "(main worktree not located)";

/// The one-line advisory for a finding. `binary` is the running version; `marker`
/// says which checkout the local marker was read from.
pub(crate) fn advisory(c: &Currency, binary: &Version, marker: &MarkerSource) -> String {
    match c {
        Currency::CheckoutNotInstalled => match marker {
            MarkerSource::Checkout => format!("{NOT_INSTALLED} {INSTALL_CMD}"),
            MarkerSource::Main(root) => {
                format!("{NOT_INSTALLED} {INSTALL_CMD} in {}", root.display())
            }
            MarkerSource::Unlocated => format!("{NOT_INSTALLED} {INSTALL_CMD} {MAIN_NOT_LOCATED}"),
        },
        Currency::CheckoutOlder { checkout } => format!(
            "this checkout was installed by doctrine {checkout} (you have {binary}): re-run {INSTALL_CMD}{}",
            in_main(marker)
        ),
        Currency::CheckoutNewer { checkout } => format!(
            "this checkout was installed by doctrine {checkout}, newer than yours ({binary}): upgrade doctrine or check which one is on PATH"
        ),
        Currency::RepoUnstamped => {
            format!("this repo has no recorded doctrine version: run {INSTALL_CMD} and commit")
        }
        Currency::RepoNewer { repo } => {
            format!("this repo is on doctrine {repo} and yours is {binary}: upgrade doctrine")
        }
        Currency::RepoOlder { repo } => format!(
            "your doctrine {binary} is newer than this repo's {repo}: run {INSTALL_CMD} and commit to move the repo up, or use {repo}"
        ),
        Currency::Unreadable { path, reason } => format!(
            "cannot read {} ({reason}): fix or remove it, then run {INSTALL_CMD}",
            path.display()
        ),
    }
}

/// The ` in <main root>` suffix for an install instruction, empty unless the
/// marker came from the main worktree.
fn in_main(marker: &MarkerSource) -> String {
    match marker {
        MarkerSource::Main(root) => format!(" in {}", root.display()),
        MarkerSource::Checkout | MarkerSource::Unlocated => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Version {
        Version::parse(s).unwrap()
    }

    fn binary() -> Version {
        v("0.47.0")
    }

    fn unreadable(name: &str) -> StampRead {
        StampRead::Unreadable {
            path: PathBuf::from(name),
            reason: "bad".into(),
        }
    }

    fn unreadable_finding_of(name: &str) -> Currency {
        Currency::Unreadable {
            path: PathBuf::from(name),
            reason: "bad".into(),
        }
    }

    /// The five states a store can be in, against binary 0.47.0.
    fn store(kind: &str, name: &str) -> StampRead {
        match kind {
            "absent" => StampRead::Absent,
            "unreadable" => unreadable(name),
            "older" => StampRead::Version(v("0.46.0")),
            "equal" => StampRead::Version(v("0.47.0")),
            "newer" => StampRead::Version(v("0.48.0")),
            other => panic!("unknown store kind {other}"),
        }
    }

    fn checkout_older() -> Currency {
        Currency::CheckoutOlder {
            checkout: v("0.46.0"),
        }
    }
    fn checkout_newer() -> Currency {
        Currency::CheckoutNewer {
            checkout: v("0.48.0"),
        }
    }
    fn repo_older() -> Currency {
        Currency::RepoOlder { repo: v("0.46.0") }
    }
    fn repo_newer() -> Currency {
        Currency::RepoNewer { repo: v("0.48.0") }
    }

    /// tracked x local, 25 rows. Tracked path is `t`, local path is `l`.
    #[test]
    fn classify_matrix() {
        use Currency::{CheckoutNotInstalled as NotInst, RepoUnstamped as Unstamped};
        let ut = || unreadable_finding_of("t");
        let ul = || unreadable_finding_of("l");
        let rows: Vec<(&str, &str, Vec<Currency>)> = vec![
            ("absent", "absent", vec![NotInst]),
            ("absent", "unreadable", vec![ul(), Unstamped]),
            ("absent", "older", vec![checkout_older(), Unstamped]),
            ("absent", "equal", vec![Unstamped]),
            ("absent", "newer", vec![checkout_newer(), Unstamped]),
            ("unreadable", "absent", vec![ut(), NotInst]),
            ("unreadable", "unreadable", vec![ul(), ut()]),
            ("unreadable", "older", vec![ut(), checkout_older()]),
            ("unreadable", "equal", vec![ut()]),
            ("unreadable", "newer", vec![ut(), checkout_newer()]),
            ("older", "absent", vec![NotInst, repo_older()]),
            ("older", "unreadable", vec![ul(), repo_older()]),
            ("older", "older", vec![checkout_older(), repo_older()]),
            ("older", "equal", vec![repo_older()]),
            ("older", "newer", vec![checkout_newer(), repo_older()]),
            ("equal", "absent", vec![NotInst]),
            ("equal", "unreadable", vec![ul()]),
            ("equal", "older", vec![checkout_older()]),
            ("equal", "equal", vec![]),
            ("equal", "newer", vec![checkout_newer()]),
            ("newer", "absent", vec![NotInst, repo_newer()]),
            ("newer", "unreadable", vec![ul(), repo_newer()]),
            ("newer", "older", vec![checkout_older(), repo_newer()]),
            ("newer", "equal", vec![repo_newer()]),
            ("newer", "newer", vec![checkout_newer(), repo_newer()]),
        ];
        for (tracked, local, want) in rows {
            let got = classify(&store(tracked, "t"), &store(local, "l"), &binary());
            assert_eq!(got, want, "tracked={tracked} local={local}");
        }
    }

    /// RV-421 F-3 control: a newer local marker with an unstamped repo reports
    /// both, so a classifier that returns nothing (or drops `RepoUnstamped`
    /// whenever local is non-current) fails.
    #[test]
    fn newer_local_with_unstamped_repo_reports_both() {
        let got = classify(&StampRead::Absent, &store("newer", "l"), &binary());
        assert_eq!(got, vec![checkout_newer(), Currency::RepoUnstamped]);
    }

    #[test]
    fn parse_stamp_reads_semver_and_never_folds_garbage_into_absent() {
        let p = Path::new("m.toml");
        assert_eq!(parse_stamp(p, " 0.47.0\n"), StampRead::Version(v("0.47.0")));
        assert!(matches!(
            parse_stamp(p, "not-a-version"),
            StampRead::Unreadable { path, .. } if path == p
        ));
        assert!(matches!(parse_stamp(p, ""), StampRead::Unreadable { .. }));
    }

    fn line(c: &Currency, marker: &MarkerSource) -> String {
        advisory(c, &binary(), marker)
    }

    #[test]
    fn advisory_lines_match_the_design_table() {
        let here = MarkerSource::Checkout;
        let want = [
            (
                Currency::CheckoutNotInstalled,
                "doctrine isn't installed in this checkout: run `doctrine install`",
            ),
            (
                checkout_older(),
                "this checkout was installed by doctrine 0.46.0 (you have 0.47.0): re-run `doctrine install`",
            ),
            (
                checkout_newer(),
                "this checkout was installed by doctrine 0.48.0, newer than yours (0.47.0): upgrade doctrine or check which one is on PATH",
            ),
            (
                Currency::RepoUnstamped,
                "this repo has no recorded doctrine version: run `doctrine install` and commit",
            ),
            (
                repo_newer(),
                "this repo is on doctrine 0.48.0 and yours is 0.47.0: upgrade doctrine",
            ),
            (
                repo_older(),
                "your doctrine 0.47.0 is newer than this repo's 0.46.0: run `doctrine install` and commit to move the repo up, or use 0.46.0",
            ),
            (
                unreadable_finding_of("a/b.toml"),
                "cannot read a/b.toml (bad): fix or remove it, then run `doctrine install`",
            ),
        ];
        for (finding, text) in want {
            assert_eq!(line(&finding, &here), text);
        }
    }

    #[test]
    fn main_source_names_the_root_on_install_instructions() {
        let main = MarkerSource::Main(PathBuf::from("/repo"));
        assert_eq!(
            line(&Currency::CheckoutNotInstalled, &main),
            "doctrine isn't installed in this checkout: run `doctrine install` in /repo"
        );
        assert_eq!(
            line(&checkout_older(), &main),
            "this checkout was installed by doctrine 0.46.0 (you have 0.47.0): re-run `doctrine install` in /repo"
        );
        // No install instruction to suffix.
        assert_eq!(
            line(&checkout_newer(), &main),
            line(&checkout_newer(), &MarkerSource::Checkout)
        );
    }

    #[test]
    fn unlocated_source_discloses_it_on_not_installed() {
        assert_eq!(
            line(&Currency::CheckoutNotInstalled, &MarkerSource::Unlocated),
            "doctrine isn't installed in this checkout: run `doctrine install` (main worktree not located)"
        );
    }

    #[test]
    fn repo_and_unreadable_lines_ignore_the_marker_source() {
        let main = MarkerSource::Main(PathBuf::from("/repo"));
        for f in [
            Currency::RepoUnstamped,
            repo_older(),
            repo_newer(),
            unreadable_finding_of("p"),
        ] {
            assert_eq!(line(&f, &main), line(&f, &MarkerSource::Checkout));
            assert_eq!(
                line(&f, &MarkerSource::Unlocated),
                line(&f, &MarkerSource::Checkout)
            );
        }
    }
}
