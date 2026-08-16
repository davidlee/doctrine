// SPDX-License-Identifier: GPL-3.0-only
//! The sole per-kind status reader (SL-238 §3, `DEC-233`): one engine-tier read
//! answering "what is this entity's authored status, as far as a reader at this
//! tier can say", with the title riding along so nobody pays for a second parse.
//!
//! Three arms, each decided **statically from the parsed kind** — never inferred
//! from what a read turned up:
//!
//! | kind set | status | title |
//! |---|---|---|
//! | [`DERIVED_STATUS`] | [`AuthoredStatus::Unavailable`] | lenient read |
//! | [`STATUS_LESS`] | [`AuthoredStatus::Absent`] | lenient read |
//! | otherwise | [`AuthoredStatus::Known`] from one `meta::read_meta` | same parse |
//!
//! Two readers, three arms. The common arm is ONE strict parse yielding both
//! fields (SL-050 `F-1`, preserved across the move); only the two special arms
//! take the lenient title reader, because `RV` and `REC` author no top-level
//! `status` and the strict read would fail for them.
//!
//! **Every arm reads.** A missing or unparseable toml is an `Err` on all three —
//! `STD-003`, so a corpus defect and a tooling limit can never share a signal.
//! What `Unavailable` means is that the *status* was never read, not that
//! nothing was: the title read always happens. An arm that returned before
//! reading would launder a corrupt `RV` into `Ok(Unavailable)`, which is the
//! failure the rule exists to forbid (`SL-238` PHASE-01, `notes.md` — §7's VT-2
//! bullet says otherwise and is corrected at reconcile).
//!
//! `RV`'s real status is derived at command tier from its finding ledger, above
//! where this module sits; `catalog::scan::status_and_title_for` is the overlay
//! that can reach it. This module names the gap, never guesses at it.
//!
//! Engine tier (`ADR-001`): imports `kinds`, `meta` and `entity` — the set
//! `integrity.rs` already carries, so this adds no new module edge in kind.
//! Deliberately not sited in `meta`, whose charter is zero per-kind knowledge
//! (`meta.rs:2-21`) and whose fifteen consumers rest on it.

use std::path::Path;

use crate::kinds::{self, AuthoredStatus, DERIVED_STATUS, STATUS_LESS};

/// One entity's authored status and title, from a single read.
pub(crate) struct Authored {
    pub(crate) status: AuthoredStatus,
    pub(crate) title: String,
}

/// Read an entity's authored status — as far as an engine-tier reader can say —
/// and its title. See the module doc for the three arms and why every one of
/// them reads.
pub(crate) fn read(root: &Path, kref: &kinds::KindRef, id: u32) -> anyhow::Result<Authored> {
    let prefix = kref.kind.prefix;
    // Static on the kind set, decided BEFORE any read outcome is known.
    let status = if DERIVED_STATUS.contains(&prefix) {
        AuthoredStatus::Unavailable
    } else if STATUS_LESS.contains(&prefix) {
        AuthoredStatus::Absent
    } else {
        // The common arm: one strict parse carries both fields.
        let tree_root = root.join(kref.kind.dir);
        let m = crate::meta::read_meta(&tree_root, kref.kind.stem, id, prefix)?;
        return Ok(Authored {
            status: AuthoredStatus::Known(m.status),
            title: m.title,
        });
    };
    Ok(Authored {
        status,
        title: title_for(root, kref, id)?,
    })
}

/// One entity's authored `title`, read leniently. Every kind authors a top-level
/// `title`, but the strict [`crate::meta::read_meta`] also demands `status`,
/// which `RV`/`REC` do not author — so a `title`-only deserialize is the one
/// reader that works across ALL kinds.
///
/// Moved here from `catalog::scan::title_for` (SL-238 §3): it belongs beside the
/// reader that uses it rather than standing as a second lenient reader outside
/// the confinement `meta.rs`'s `IdOnly` doc declares.
fn title_for(root: &Path, kref: &kinds::KindRef, id: u32) -> anyhow::Result<String> {
    #[derive(serde::Deserialize)]
    struct TitleOnly {
        title: String,
    }
    let path = crate::entity::id_path(root, kref.kind, id, crate::entity::Ext::Toml);
    let text = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("read {} for title: {e}", path.display()))?;
    let parsed: TitleOnly = toml::from_str(&text)
        .map_err(|e| anyhow::anyhow!("parse title from {}: {e}", path.display()))?;
    Ok(parsed.title)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    use crate::kinds::{self, AuthoredStatus, KindRef};

    /// Write `body` as the whole of `<dir>/<NNN>/<stem>-<NNN>.toml` for `kref`.
    /// The `KindRef` carries both halves of the path the reader resolves — the
    /// tree `dir` and the file `stem` — so one helper seeds any numbered kind and
    /// the tests below can loop over a kind SET rather than naming kinds by hand.
    fn seed_toml(root: &Path, kref: &KindRef, id: u32, body: &str) {
        let name = format!("{id:03}");
        let dir = root.join(kref.kind.dir).join(&name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{}-{name}.toml", kref.kind.stem)), body).unwrap();
    }

    /// Seed a status-BEARING toml: the four fields `meta::Meta` requires, and
    /// nothing else — a true unit fixture, independent of any kind's scaffold.
    fn seed_status_bearing(root: &Path, kref: &KindRef, id: u32, status: &str, title: &str) {
        seed_toml(
            root,
            kref,
            id,
            &format!("id = {id}\nslug = \"s{id}\"\ntitle = \"{title}\"\nstatus = \"{status}\"\n"),
        );
    }

    /// Seed a status-LESS toml — `id`/`slug`/`title` only, the shape RV and REC
    /// actually author. `meta::read_meta` hard-fails on this (pinned by
    /// `meta::tests::read_meta_still_hard_fails_on_a_missing_status`), so it is
    /// also the fixture that proves the lenient title read is in play.
    fn seed_status_less(root: &Path, kref: &KindRef, id: u32, title: &str) {
        seed_toml(
            root,
            kref,
            id,
            &format!("id = {id}\nslug = \"s{id}\"\ntitle = \"{title}\"\n"),
        );
    }

    fn kref_for(prefix: &str) -> &'static KindRef {
        kinds::kind_by_prefix(prefix).unwrap_or_else(|| panic!("no KindRef for `{prefix}`"))
    }

    /// SL-238 VT-1, the common arm: every kind admissible as a dep/seq target is
    /// status-bearing, so `read` yields `Known(status)` for each — asserted over
    /// the SET, so a kind added to `ADMISSIBLE_DEP_TARGETS` without a status to
    /// read fails here rather than at a `boundary:` line months later.
    ///
    /// The title is asserted from the SAME call: `Authored` carries both, which
    /// is what makes the common path one `meta::read_meta` rather than a status
    /// read plus a second title read (SL-050 `F-1`, preserved across the move).
    #[test]
    fn read_returns_a_known_status_for_each_admissible_target_kind() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        for (i, prefix) in kinds::ADMISSIBLE_DEP_TARGETS.iter().enumerate() {
            let kref = kref_for(prefix);
            let id = u32::try_from(i).unwrap() + 1;
            let title = format!("T {prefix}");
            seed_status_bearing(root, kref, id, "open", &title);

            let a = read(root, kref, id).unwrap_or_else(|e| panic!("{prefix}: {e}"));
            assert_eq!(
                a.status,
                AuthoredStatus::Known("open".to_string()),
                "{prefix} is an admissible dep/seq target and authors a status"
            );
            assert_eq!(
                a.title, title,
                "{prefix}: status and title come from one call"
            );
        }
    }

    /// SL-238 VT-1, the status-less arm: a kind in `STATUS_LESS` authors no
    /// top-level `status`, so `read` reports `Absent` — the honest "this kind has
    /// no status to state", which `authored_class` maps to `Terminal`. The title
    /// still arrives, so the lenient reader is doing its job.
    #[test]
    fn read_returns_absent_for_a_status_less_kind() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        for prefix in kinds::STATUS_LESS {
            let kref = kref_for(prefix);
            seed_status_less(root, kref, 7, "R 7");

            let a = read(root, kref, 7).unwrap_or_else(|e| panic!("{prefix}: {e}"));
            assert_eq!(
                a.status,
                AuthoredStatus::Absent,
                "{prefix} authors no top-level status"
            );
            assert_eq!(
                a.title, "R 7",
                "{prefix}: the lenient title read still yields a title where strict \
                 `meta::read_meta` would fail"
            );
        }
    }

    /// SL-238 VT-2: `Unavailable` is decided STATICALLY from the parsed kind, not
    /// from what a read turned up. The fixture's toml *does* carry
    /// `status = "open"` — the one a common-arm kind would report as
    /// `Known("open")` — and a derived-status kind reports `Unavailable` anyway.
    /// That is what "without reading" names: the STATUS read is never attempted.
    ///
    /// It does not name the title read, which always happens (§3's arm table), so
    /// there is deliberately no `no toml at all` fixture here: an absent file is
    /// an `Err` on every arm, not `Ok(Unavailable)`. VT-2 originally asked for
    /// that case; PHASE-01 T2 established it was unreachable and the criterion was
    /// amended. `read_never_launders_an_unparseable_toml_into_a_kind_set_verdict`
    /// below is where the absent/corrupt file's fate is actually pinned.
    #[test]
    fn read_returns_unavailable_for_a_derived_status_kind_without_reading() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        for prefix in kinds::DERIVED_STATUS {
            let kref = kref_for(prefix);
            seed_status_bearing(root, kref, 1, "open", "R 1");

            let a = read(root, kref, 1).unwrap_or_else(|e| panic!("{prefix}: {e}"));
            assert_eq!(
                a.status,
                AuthoredStatus::Unavailable,
                "{prefix}: the arm is static on the kind set, not on what the toml carries"
            );
        }
    }

    /// SL-238 VT-3 / STD-003: a corpus defect and a tooling limit never share a
    /// signal. A present-but-unparseable toml is an `Err` — not `Ok(Unavailable)`,
    /// not `Ok(Absent)` — so no caller can mistake a broken file for a kind whose
    /// status this tier cannot see.
    #[test]
    fn read_returns_err_on_a_present_but_unparseable_toml() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let kref = kref_for(kinds::SL);
        seed_toml(root, kref, 1, "id = notanumber\n");

        assert!(
            read(root, kref, 1).is_err(),
            "an unparseable toml is a corpus defect: `Err`, never `Ok(Unavailable)` \
             and never `Ok(Absent)`"
        );
    }

    /// SL-238 VT-3, reaching the two arms its own fixture cannot: §3 rule 3 binds
    /// EVERY arm, not just the common one. Each special arm has a token of its own
    /// to launder a broken file into — `Unavailable` for a derived-status kind,
    /// `Absent` for a status-less one — and an arm that decided its verdict and
    /// returned without reading would emit exactly that, silently converting a
    /// corpus defect into a tooling limit. Both arms take the lenient title read,
    /// so both `Err` instead, and a broken file and a tooling gap never share a
    /// signal (STD-003).
    #[test]
    fn read_never_launders_an_unparseable_toml_into_a_kind_set_verdict() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        for prefix in kinds::DERIVED_STATUS {
            let kref = kref_for(prefix);
            seed_toml(root, kref, 1, "id = notanumber\n");
            assert!(
                read(root, kref, 1).is_err(),
                "{prefix}: a corrupt toml is `Err`, never `Ok(Unavailable)`"
            );
        }

        for prefix in kinds::STATUS_LESS {
            let kref = kref_for(prefix);
            seed_toml(root, kref, 1, "id = notanumber\n");
            assert!(
                read(root, kref, 1).is_err(),
                "{prefix}: a corrupt toml is `Err`, never `Ok(Absent)`"
            );
        }
    }
}
