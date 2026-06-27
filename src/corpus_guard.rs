// SPDX-License-Identifier: GPL-3.0-only
//! `corpus_guard` — the dispatch corpus-loss guards (SL-166, ISS-056).
//!
//! Three mechanism-level guards refuse the doctrine-verb-mediated paths by which
//! a `/dispatch` drive can silently delete or revert the authored `.doctrine`
//! corpus (design §5):
//!
//! - **g1** — refuse trunk-mutating verbs while HEAD sits on the integration
//!   buffer ([`REFUSE_ON_TRUNK`]).
//! - **g2** — refuse `coordinate()` setup when the fork base predates the corpus
//!   ([`BASE_CORPUS_STALE`]).
//! - **g3** — refuse a per-leg advance that clobbers authored [`DOCTRINE_PATHSPEC`]
//!   paths ([`CORPUS_CLOBBER`]).
//!
//! PHASE-01 lands the shared substrate: the named refusal tokens (STD-001) and
//! the corpus pathspec. The pure guard predicates grow here in PHASE-02/04; g2
//! extends `worktree::coordinate` in PHASE-03. Leaf tier (ADR-001): constants
//! only today, and any predicate stays pure over injected git readings — git I/O
//! lives in `git.rs`, which receives the pathspec as a parameter (never imports
//! this module, so no cycle).

/// Refusal prefix for g1 — a trunk-mutating verb invoked while HEAD is on the
/// integration buffer (`deliver_to`). The buffer ref and recovery are
/// interpolated at the call site (SL-166 design §5.2). Consumed in PHASE-04.
pub(crate) const REFUSE_ON_TRUNK: &str = "refused: HEAD is on the integration buffer";

/// The git local-branch ref prefix — the part `symbolic-ref --short HEAD` strips
/// when reporting a branch name. Single-source (STD-001).
const REFS_HEADS_PREFIX: &str = "refs/heads/";

/// Refusal prefix for g2 — a `dispatch setup` whose fork base predates the
/// authored `.doctrine` corpus the authoring branch holds (SL-166 design §5.2).
/// Consumed by `worktree::coordinate::ensure_base_corpus_fresh` (PHASE-03).
pub(crate) const BASE_CORPUS_STALE: &str =
    "refused: fork base predates the authored .doctrine corpus";

/// Refusal prefix for g3 — a funnel advance that would delete or revert authored
/// `.doctrine` paths the target ref holds, unallowed (SL-166 design §5.2).
pub(crate) const CORPUS_CLOBBER: &str = "refused: advance would clobber authored .doctrine paths";

/// The authored-corpus pathspec — the directory g2/g3 reason over and the
/// projection strips (`dispatch.rs` `filter_tree`). Single-source (STD-001).
pub(crate) const DOCTRINE_PATHSPEC: &str = ".doctrine";

/// Cap on the number of clobbered paths rendered into the g3 refusal message
/// (EX-5, R2): the catastrophe path is 4816 paths — show the first
/// [`CLOBBER_RENDER_CAP`] and summarise the rest as "(+N more)". Single-source
/// (STD-001).
pub(crate) const CLOBBER_RENDER_CAP: usize = 20;

use std::collections::BTreeSet;

/// One per-path 3-way reading for g3 — the blob oids of an authored `.doctrine`
/// path at the advance's `base` and `new` trees (`None` ⇒ the path is absent
/// from that tree). The path is *already* known to differ between `base` and the
/// live target `cur`: the shell computes the changed set via the diff seam, so
/// the predicate need only decide whether the advance authored that change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClobberReading {
    /// The `.doctrine/**` path under consideration.
    pub path: String,
    /// Its blob oid in the `base` tree (`merge-base(new, cur)`), `None` if absent.
    pub base_oid: Option<String>,
    /// Its blob oid in the `new` (planned) tree, `None` if absent.
    pub new_oid: Option<String>,
}

/// g3 predicate (design §5.2, EX-2) — of the `.doctrine` paths that changed
/// between the advance `base` and the live target `cur`, flag those the advance
/// would **revert or delete**: `new == base` (the advance did not itself author
/// the change `cur` holds) and the path is not operator-allowlisted. Pure over
/// injected readings — no git I/O. Returns the clobbered paths in input order.
///
/// `new == base` subsumes both loss shapes uniformly (design D2): both `None`
/// (cur added a path the advance drops — a deletion) and both `Some(x)` (cur
/// edited a path the advance reverts to `x` — a stale revert) are clobbers; a
/// genuinely authored delta (`new != base`) is not.
pub(crate) fn corpus_clobber_check<'a>(
    readings: &'a [ClobberReading],
    allow: &BTreeSet<String>,
) -> Vec<&'a str> {
    readings
        .iter()
        .filter(|r| r.new_oid == r.base_oid)
        .filter(|r| !allow.contains(&r.path))
        .map(|r| r.path.as_str())
        .collect()
}

/// Render a capped, human-facing clobber list for the [`CORPUS_CLOBBER`] refusal
/// (EX-5, R2): the first `cap` paths joined, plus a "(+N more)" tail when the set
/// exceeds `cap` — bounds the 4816-path catastrophe-case message. Pure.
pub(crate) fn render_clobbers(paths: &[&str], cap: usize) -> String {
    let shown = paths
        .iter()
        .take(cap)
        .copied()
        .collect::<Vec<_>>()
        .join(", ");
    if paths.len() > cap {
        format!("{shown} (+{} more)", paths.len() - cap)
    } else {
        shown
    }
}

/// The short branch name of a `refs/heads/<name>` ref — the form `symbolic-ref
/// --short HEAD` reports — or the input unchanged if it carries no such prefix.
/// Pure; single-source for the prefix strip (STD-001).
pub(crate) fn short_branch_name(refish: &str) -> &str {
    refish.strip_prefix(REFS_HEADS_PREFIX).unwrap_or(refish)
}

/// g1 predicate (design §5.2 g1, EX-3) — whether a trunk-mutating dispatch verb
/// must refuse because HEAD (`current`, the short branch name `symbolic-ref
/// --short HEAD` reports) sits on the integration buffer. Pure over the resolved
/// HEAD and the posture config.
///
/// Inert (false) when the buffered-trunk posture is off: `authoring` unset, or
/// `authoring == deliver_to` (a misconfiguration `validate_posture` rejects, but
/// g1 stays defensively inert — it may run on unvalidated config). Otherwise
/// refuses iff `current` is the short name of `deliver_to` (the buffer).
pub(crate) fn on_integration_buffer(
    current: Option<&str>,
    authoring: Option<&str>,
    deliver_to: &str,
) -> bool {
    let Some(authoring) = authoring else {
        return false; // posture off
    };
    if authoring == deliver_to {
        return false; // misconfigured posture — defensively inert
    }
    current == Some(short_branch_name(deliver_to))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading(path: &str, base: Option<&str>, new: Option<&str>) -> ClobberReading {
        ClobberReading {
            path: path.to_owned(),
            base_oid: base.map(str::to_owned),
            new_oid: new.map(str::to_owned),
        }
    }

    fn allow(paths: &[&str]) -> BTreeSet<String> {
        paths.iter().map(|p| (*p).to_owned()).collect()
    }

    #[test]
    fn phantom_deletion_is_clobber() {
        // VT-1 shape: cur added the path; base + new both absent ⇒ advance drops it.
        let r = [reading(".doctrine/x.toml", None, None)];
        assert_eq!(corpus_clobber_check(&r, &allow(&[])), [".doctrine/x.toml"]);
    }

    #[test]
    fn stale_revert_is_clobber() {
        // VT-2 shape: cur edited to a new blob; base==new==old ⇒ advance reverts it.
        let r = [reading(".doctrine/x.toml", Some("old"), Some("old"))];
        assert_eq!(corpus_clobber_check(&r, &allow(&[])), [".doctrine/x.toml"]);
    }

    #[test]
    fn authored_delta_is_not_clobber() {
        // VT-5 shape: the advance authored the change (new != base) ⇒ allowed.
        let r = [reading(".doctrine/x.toml", Some("old"), Some("new"))];
        assert!(corpus_clobber_check(&r, &allow(&[])).is_empty());
    }

    #[test]
    fn allowlist_lets_named_path_through() {
        // VT-5: an explicitly allowlisted clobber is permitted.
        let r = [reading(".doctrine/x.toml", Some("old"), Some("old"))];
        assert!(corpus_clobber_check(&r, &allow(&[".doctrine/x.toml"])).is_empty());
    }

    #[test]
    fn unnamed_path_still_refused_with_partial_allowlist() {
        // VT-5: a partial allowlist clears only its named path; the rest still bite.
        let r = [
            reading(".doctrine/x.toml", Some("old"), Some("old")),
            reading(".doctrine/y.toml", None, None),
        ];
        assert_eq!(
            corpus_clobber_check(&r, &allow(&[".doctrine/x.toml"])),
            [".doctrine/y.toml"]
        );
    }

    #[test]
    fn empty_changed_set_is_inert() {
        // VT-4 shape (caller side): an FF advance yields base==cur ⇒ no changed
        // paths reach the predicate ⇒ nothing to flag.
        assert!(corpus_clobber_check(&[], &allow(&[])).is_empty());
    }

    #[test]
    fn render_clobbers_lists_under_cap_verbatim() {
        assert_eq!(render_clobbers(&["a", "b"], 20), "a, b");
        assert_eq!(render_clobbers(&[], 20), "");
    }

    #[test]
    fn render_clobbers_caps_and_summarises_overflow() {
        let paths: Vec<&str> = ["p0", "p1", "p2", "p3", "p4"].into();
        assert_eq!(render_clobbers(&paths, 2), "p0, p1 (+3 more)");
    }

    // --- g1 predicate (PHASE-04) -------------------------------------------

    #[test]
    fn short_branch_name_strips_refs_heads() {
        assert_eq!(short_branch_name("refs/heads/main"), "main");
        // No prefix ⇒ unchanged (a bare branch name or a non-branch ref).
        assert_eq!(short_branch_name("main"), "main");
        assert_eq!(short_branch_name("refs/tags/v1"), "refs/tags/v1");
    }

    #[test]
    fn g1_refuses_when_head_on_buffer() {
        // Posture on, HEAD on the buffer short name ⇒ refuse (VT-1 unit side).
        assert!(on_integration_buffer(
            Some("main"),
            Some("refs/heads/edge"),
            "refs/heads/main"
        ));
    }

    #[test]
    fn g1_allows_on_authoring_branch() {
        // Posture on but HEAD on the authoring branch ⇒ inert (VT-2 safe leg).
        assert!(!on_integration_buffer(
            Some("edge"),
            Some("refs/heads/edge"),
            "refs/heads/main"
        ));
    }

    #[test]
    fn g1_inert_when_posture_unset() {
        // authoring-branch absent ⇒ inert even when HEAD is on the buffer
        // (single-branch parity, INV-2).
        assert!(!on_integration_buffer(
            Some("main"),
            None,
            "refs/heads/main"
        ));
    }

    #[test]
    fn g1_inert_when_authoring_equals_deliver_to() {
        // Misconfigured posture (authoring == buffer) ⇒ defensively inert (EX-3).
        assert!(!on_integration_buffer(
            Some("main"),
            Some("refs/heads/main"),
            "refs/heads/main"
        ));
    }

    #[test]
    fn g1_inert_on_detached_head() {
        // No symbolic branch (detached HEAD) ⇒ not on the buffer ⇒ inert.
        assert!(!on_integration_buffer(
            None,
            Some("refs/heads/edge"),
            "refs/heads/main"
        ));
    }
}
