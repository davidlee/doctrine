// SPDX-License-Identifier: GPL-3.0-only
//! RV blocker predicates and graph projections (SL-268 D4, engine tier): the
//! `reviews` relation edge, the derived-status string, and the unresolved-blocker
//! / pass-fact reads other kinds gate closure on.

use std::path::Path;

use anyhow::Context;

use super::schema::{
    ReviewDoc, canonical_id, parse_finding_status, parse_ref, read_review, read_reviews,
};
use super::vocab::{FindingStatus, ReviewStatus, Severity};
use crate::kinds::REVIEW_DIR;

/// A review's authored outbound relation (SL-046 §5.2/§5.3): the single
/// `[target].ref` subject edge `RV-N ──reviews──▶ target` →
/// [`RelationLabel::Reviews`]. Reads via the existing `read_review` reader (no new
/// TOML parse). Always exactly one edge (the target ref is required at `new`).
pub(crate) fn relation_edges(
    root: &Path,
    id: u32,
) -> anyhow::Result<Vec<crate::relation::RelationEdge>> {
    use crate::relation::{RelationEdge, RelationLabel};
    let doc = read_review(&root.join(REVIEW_DIR), id)?;
    Ok(vec![RelationEdge::new(
        RelationLabel::Reviews,
        doc.target.reference,
    )])
}

/// A review's DERIVED status string (`"active"`/`"done"`) for the cross-kind
/// priority scan (SL-047 §5.2). An RV authors no `status` field (D-C8); its status
/// is `derived_status` over the AUTHORED finding ledger — authored-tier, not a
/// runtime read. Reads via the existing `read_review` reader (no new TOML parse),
/// then runs the same pure `derived` the `show`/`list`/`status` surfaces use.
pub(crate) fn derived_status_string(root: &Path, id: u32) -> anyhow::Result<String> {
    let doc = read_review(&root.join(REVIEW_DIR), id)?;
    let (status, _await) = doc.derived();
    Ok(status.as_str().to_string())
}

/// One unresolved blocker holding a target's closure open (design §7, D8/D-C9b):
/// the canonical RV id (`RV-007`) and the offending finding id (`F-2`). Surfaced
/// by the close-gate to name *why* a closure-seam transition is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BlockerRef {
    pub(crate) rv: String,
    pub(crate) finding: String,
}

/// Pure check (design §7): the unresolved blocker findings *this* RV holds against
/// its target. A finding gates iff `severity == Blocker && status ∉ {verified,
/// withdrawn}` — but ONLY on an **Active** review (`derived_status == Active`,
/// D-C8): a `Done` ledger (every finding terminal, D-C9a) holds nothing, even if a
/// stray non-terminal status were hand-edited in (the derived gate already
/// excludes that by keeping it Active). No I/O — operates on already-read data so
/// the scan shell stays thin (the `integrity::scan_kind` shape).
pub(crate) fn doc_unresolved_blockers(doc: &ReviewDoc) -> Vec<BlockerRef> {
    if doc.derived().0 != ReviewStatus::Active {
        return Vec::new();
    }
    doc.finding
        .iter()
        .filter(|f| Severity::parse(&f.severity) == Ok(Severity::Blocker))
        .filter(|f| !parse_finding_status(&f.status).is_terminal())
        .map(|f| BlockerRef {
            rv: canonical_id(doc.id),
            finding: f.id.clone(),
        })
        .collect()
}

/// Pure check (SL-244 DEC-138): the findings on this RV that hold a design run's
/// `reviewing → locked` edge — `severity == Blocker` **and** `status ∈ {open,
/// contested}`, carried as the ledger's own `F-n` ids.
///
/// **Spelled separately from [`doc_unresolved_blockers`], and deliberately not by
/// copying it and restricting the state.** Two things follow from that, and the
/// second is the one worth writing down:
///
/// - The two predicates differ by the `answered` state, on purpose. D-C9b asks
///   *is this review finished* — a disposed-but-unverified blocker is not, so it
///   gates a slice's closure. This asks *has this pass been disposed of* — and an
///   answered blocker has been. One shared filter would silently pick a side.
/// - The review-level `derived_status` guard is **not** inherited. Doing so would
///   bind a per-finding fact to a display summary that ADR-007 D7 says is never a
///   gate. It happens to be unobservable here — any `open` or `contested` finding
///   forces `Active`, so the guard could not fire while this returns anything (the
///   `the_predicate_does_not_read_review_level_status` test pins that implication)
///   — which is precisely why the argument is about coupling and not about a wrong
///   answer today.
///
/// No I/O: operates on already-read data, like its neighbour.
pub(crate) fn undisposed_blockers(doc: &ReviewDoc) -> Vec<String> {
    doc.finding
        .iter()
        .filter(|f| Severity::parse(&f.severity) == Ok(Severity::Blocker))
        .filter(|f| {
            matches!(
                parse_finding_status(&f.status),
                FindingStatus::Open | FindingStatus::Contested
            )
        })
        .map(|f| f.id.clone())
        .collect()
}

/// The findings one `RV` still holds, counted by severity (SL-244 `EX-2`).
///
/// A **fixed record rather than a map**, because the ledger's severity vocabulary
/// is closed at four ([`Severity`]) — with a map, an absent key and a zero count
/// would be one fact with two spellings, and every reader would have to choose.
///
/// `review`'s tier-local record, paired into `design_run`'s `OutstandingBySeverity`
/// by the command tier — exactly as [`PassFacts`] is paired into `ObservedReview`,
/// and for the same reason (SL-244 `D4`): the *rendered* record belongs to the leaf
/// that renders it, and a general ledger query does not take a dependency on one
/// consumer's render vocabulary.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct OutstandingCounts {
    pub(crate) blocker: usize,
    pub(crate) major: usize,
    pub(crate) minor: usize,
    pub(crate) nit: usize,
}

/// Pure count (SL-244 `EX-2`): what this RV still holds, by severity — every
/// finding whose `status ∉ {verified, withdrawn}`, across all four severities.
///
/// **Spelled separately from both neighbours, and the divergence is the point.**
/// This is the third filter over the same ledger and it agrees with neither:
///
/// - [`undisposed_blockers`] asks *does this pass hold a design run's edge* —
///   blockers alone, and an `answered` blocker has been disposed of. This asks
///   *what does a reader still owe work on*, which an answered blocker plainly is.
///   Reusing that filter would take the lamp dark on exactly the findings it
///   exists to surface (`VT-4`).
/// - [`doc_unresolved_blockers`] asks *is this review finished* (D-C9b) — also
///   blockers alone, and it early-returns on a `Done` review. Neither restriction
///   belongs here: severity is what this counts *by*, not what it filters on, and
///   the review-level guard would bind a per-finding count to a display summary
///   ADR-007 D7 says is never a gate. That guard is unobservable for this
///   predicate — a `Done` review's findings are all terminal, so every count is
///   already zero — which is why the argument is about coupling, as its
///   neighbour's is.
///
/// A severity that will not parse is counted nowhere, matching
/// [`undisposed_blockers`]'s handling of the same malformed row.
///
/// No I/O: operates on already-read data, like both neighbours.
pub(crate) fn outstanding_by_severity(doc: &ReviewDoc) -> OutstandingCounts {
    let mut counts = OutstandingCounts::default();
    for finding in doc
        .finding
        .iter()
        .filter(|f| !parse_finding_status(&f.status).is_terminal())
    {
        let bucket = match Severity::parse(&finding.severity) {
            Ok(Severity::Blocker) => &mut counts.blocker,
            Ok(Severity::Major) => &mut counts.major,
            Ok(Severity::Minor) => &mut counts.minor,
            Ok(Severity::Nit) => &mut counts.nit,
            Err(_) => continue,
        };
        *bucket = bucket.saturating_add(1);
    }
    counts
}

/// What a design run's gate needs to know about one named `RV` (SL-244 `sec-3`).
///
/// `review`'s half of the answer, without the reference the caller already holds:
/// the command tier pairs the two into `design_run`'s `ObservedReview`, exactly as
/// it pairs a runbook key with its parsed book into `RunbookFacts`. Keeping the
/// reference out is what lets `review` answer this without importing `design_run`
/// (ADR-001) — the dependency runs one way, `design`-shell → `review`-query, as
/// [`unresolved_blockers_for`]'s does.
#[derive(Debug)]
pub(crate) struct PassFacts {
    /// Whether the ledger carries the concluded-pass marker
    /// ([`ReviewMeta::concluded`], set by [`run_conclude`]).
    ///
    /// Absence reads `false`, which is the honest answer for a pass nobody closed
    /// and the conservative one for a pass somebody did — every ledger minted
    /// before the marker existed reads unconcluded, and no migration says
    /// otherwise. This is what makes a design run's `Conducted` disposition
    /// admissible; `Waived` reads none of it.
    pub(crate) concluded: bool,
    /// The findings holding the run's `reviewing → locked` edge, by `F-n` id.
    pub(crate) undisposed_blockers: Vec<String>,
    /// What the ledger still holds, by severity — the warning lamp's input, wider
    /// than [`Self::undisposed_blockers`] on purpose (SL-244 `EX-2`).
    pub(crate) outstanding: OutstandingCounts,
}

/// Read a named `RV` for a design run — the single parse both consumers share.
///
/// `Err` where the reference will not parse or names no readable ledger, naming
/// the reference either way.
///
/// **One read, two postures** (SL-244 `D5`, the owner's 2026-08-05 ruling). The
/// failure is *returned* here rather than decided here, because the two callers
/// want opposite things from it and a second parse is how they would drift apart:
///
/// - the **gate** swallows it through [`observe_pass`] — `PHASE-04`'s `EX-5`
///   requires an unreadable pass to read as refusal, never as an error a caller
///   can dismiss;
/// - the **projection** path propagates it, because the render vocabulary is
///   two-valued (silence, or the counts) and silence means *nothing outstanding*.
///   A lamp that cannot read its own ledger has to say so rather than borrow the
///   spelling of good news.
pub(crate) fn read_pass_facts(root: &Path, reference: &str) -> anyhow::Result<PassFacts> {
    let id = parse_ref(reference)?;
    let doc = read_review(&root.join(REVIEW_DIR), id)
        .with_context(|| format!("read the review pass `{reference}`"))?;
    Ok(PassFacts {
        concluded: doc.review.concluded,
        undisposed_blockers: undisposed_blockers(&doc),
        outstanding: outstanding_by_severity(&doc),
    })
}

/// Read a named `RV` for a design run's gate — `None` if it cannot be read.
///
/// **Absence is refusal, not satisfaction** (SL-244 `sec-3`). An unparseable ref
/// and a ref naming no ledger both yield `None` rather than an empty observation,
/// because an empty observation reads as *no blockers* and would clear the very
/// edge an unreadable review must hold. The two failures are one answer on
/// purpose: the caller's question is *can Doctrine see this pass*, and it cannot,
/// either way.
///
/// The `.ok()` is the whole of this wrapper: [`read_pass_facts`] does the reading,
/// and this chooses the gate's posture over its failure.
pub(crate) fn observe_pass(root: &Path, reference: &str) -> Option<PassFacts> {
    read_pass_facts(root, reference).ok()
}

/// The reverse close-gate scan (design §7, D8/D-C9b) — a **standalone scoped scan**
/// over `.doctrine/review/*`, NOT the spec `Registry` (wrong cohesion) and NOT a
/// general reverse index (scope non-goal). Returns every unresolved blocker
/// (`severity == Blocker && status ∉ {verified, withdrawn}`) on an Active RV whose
/// `[target].ref` matches `subject_ref`. The thin shell: read every ledger, then
/// the pure [`doc_unresolved_blockers`] filters each. O(#RV)/close (R2 — fine at
/// scale, index later). Used by the slice-close command shell one-way
/// (`slice`-shell → `review`-query); `review` must not import `slice` (ADR-001).
pub(crate) fn unresolved_blockers_for(
    root: &Path,
    subject_ref: &str,
) -> anyhow::Result<Vec<BlockerRef>> {
    let review_root = root.join(REVIEW_DIR);
    if !review_root.is_dir() {
        return Ok(Vec::new());
    }
    let mut blockers = Vec::new();
    for doc in read_reviews(&review_root)? {
        if doc.target.reference == subject_ref {
            blockers.extend(doc_unresolved_blockers(&doc));
        }
    }
    Ok(blockers)
}
