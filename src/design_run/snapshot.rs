// SPDX-License-Identifier: GPL-3.0-only
//! The canonical run snapshot — **storage** (SL-233 PHASE-03 EX-1/EX-10/EX-13).
//!
//! Schema-versioned, with parse-time rejection of an unknown version and a
//! remedy-naming message. The precedent is `src/comparison/wire.rs`'s
//! `COMPARISON_SCHEMA` / `COMPARISON_VERSION` / `SUPPORTED_VERSIONS` triple and
//! its `parse()` refusal — ridden rather than re-invented, so the corpus has one
//! versioning scheme and not two (EX-1).
//!
//! The snapshot **groups rather than flattens** design §5.3: a flat table would
//! make "which group owns this key?" a question every reader has to answer from
//! the key's spelling. Each group below is one §5.3 bullet, plus the change log
//! as its own group (EX-13).
//!
//! Storage keeps **full fidelity**. No `ENVELOPE_*` constant applies here, and by
//! construction none can be named from this module — they are private to
//! [`super::render`].

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::Stage;
use super::attestation::{
    ActorClass, Attestation, IntegratedReview, LockAcceptance, RecoveryIntent, ReviewPolicy,
};
use super::bounds::CHANGE_LOG_REVISIONS;
use super::change_log::ChangeLog;
use super::delegation::{DelegationGroup, DelegationState};
use super::facts::DerivedDesignFacts;
use super::gate::ReviewStanding;
use super::ids::{DesignId, Fingerprint};
use super::inquiry::InquiryMap;
use super::runbook::RunbookGroup;
use super::traversal::{Cursor, TraversalPosture};

/// The `schema` discriminator every design-run snapshot carries; checked on
/// parse. Riding the `comparison::wire` precedent: a dotted, kind-naming string,
/// not a bare name that could collide with another runtime file.
pub(crate) const DESIGN_SNAPSHOT_SCHEMA: &str = "doctrine.design-run";

/// The wire version this model WRITES.
pub(crate) const DESIGN_SNAPSHOT_VERSION: u32 = 1;

/// The versions parse accepts. A one-member set today; it widens the way
/// `comparison::wire` widened, by adding a member rather than by loosening the
/// check.
pub(crate) const SUPPORTED_VERSIONS: &[u32] = &[DESIGN_SNAPSHOT_VERSION];

/// Run identity and position — design §5.3's first group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RunHeader {
    /// The run UID, for stale-context detection. One active run per slice in v1.
    pub(crate) uid: String,
    /// The slice this run designs.
    pub(crate) slice: u32,
    /// The monotonic revision every mutation compare-and-swaps against
    /// (DEC-059).
    pub(crate) revision: u64,
    pub(crate) stage: Stage,
    /// The reviewer lanes this run requires of each section (DEC-073, ISS-310).
    ///
    /// Run data rather than a constant of the gate: *which* lanes a section needs
    /// is the user's declaration, which is why the condition's required actor
    /// cannot be fixed in the table. `default` is what makes the migration a
    /// no-op by construction — the same argument [`Section::seq`] makes.
    #[serde(default)]
    pub(crate) review_policy: ReviewPolicy,
    /// The next closed obligation, when the run knows one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) next_obligation: Option<String>,
}

/// One submission receipt — the idempotency key and what it produced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Receipt {
    pub(crate) submission: String,
    /// The revision this submission produced.
    pub(crate) revision: u64,
    /// The digest of the accepted payload: a replay carrying different bytes
    /// under the same id is a different submission wearing a used name.
    pub(crate) digest: String,
    /// The delegated obligation this receipt is referenced by, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) delegation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) delegation_state: Option<DelegationState>,
}

impl Receipt {
    /// Whether an outstanding delegation still references this receipt — the
    /// pin that eviction may never break.
    pub(crate) fn is_pinned(&self) -> bool {
        self.delegation.is_some()
            && matches!(
                self.delegation_state,
                Some(DelegationState::Outstanding | DelegationState::Proposed) | None
            )
    }
}

/// Bounded submission receipts, plus the floor that makes "expired" a fact
/// rather than a guess.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ReceiptGroup {
    /// The oldest revision for which receipt history is **complete**. Below it
    /// an unknown submission id cannot be told apart from one already applied
    /// and evicted, so it is refused as expired rather than treated as new.
    pub(crate) floor: u64,
    #[serde(default, rename = "receipt")]
    pub(crate) receipts: Vec<Receipt>,
}

impl ReceiptGroup {
    /// The receipt for `submission`, if history still holds it.
    pub(crate) fn find(&self, submission: &str) -> Option<&Receipt> {
        self.receipts
            .iter()
            .find(|receipt| receipt.submission == submission)
    }

    /// Re-state the delegation standing every receipt of `delegation` carries.
    ///
    /// The pin ([`Receipt::is_pinned`]) reads the receipt's own stored state, so
    /// without this a delegation's receipt would stay pinned for the life of the
    /// run: eviction could never reclaim the receipt of an assignment that has
    /// long since been accepted. Retention is a storage bound and it has to be
    /// able to make progress.
    pub(crate) fn restate_delegation(&mut self, delegation: &str, state: DelegationState) {
        for receipt in &mut self.receipts {
            if receipt.delegation.as_deref() == Some(delegation) {
                receipt.delegation_state = Some(state);
            }
        }
    }

    /// Record a receipt and evict past the retention window.
    pub(crate) fn record(&mut self, receipt: Receipt) {
        let current = receipt.revision;
        self.receipts.push(receipt);
        self.evict(current);
    }

    /// Evict receipts below the window — never the latest, and never one an
    /// outstanding delegation still references (EX-4).
    fn evict(&mut self, current_revision: u64) {
        let floor = current_revision.saturating_sub(CHANGE_LOG_REVISIONS) + 1;
        if floor > self.floor {
            self.floor = floor;
        }
        let keep = self.floor;
        let latest = self.receipts.iter().map(|r| r.revision).max();
        self.receipts
            .retain(|r| r.revision >= keep || Some(r.revision) == latest || r.is_pinned());
    }
}

/// The inquiry map, cursor, traversal posture and pin — one group, four separate
/// facts (DEC-060, DEC-061).
///
/// The pin is its own field rather than a flag on a node, and that separation is
/// what keeps design R4 (authority laundering) a type-level property: a
/// user-pinned direction cannot be confused with agent-proposed structure,
/// because the two are different fields carrying different [`Authority`] values.
/// At most one pin exists — an invariant the *type* holds, so projection never
/// has to resolve a conflict `apply` should have refused.
///
/// [`Authority`]: super::traversal::Authority
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct MapGroup {
    pub(crate) inquiry: InquiryMap,
    pub(crate) cursor: Cursor,
    pub(crate) posture: TraversalPosture,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) pin: Option<Pin>,
    /// The creation-order counter every node's `seq` is claimed from. Monotonic
    /// within the run and never reused, so ordering survives a node's removal
    /// from the frontier.
    #[serde(default)]
    pub(crate) next_seq: u64,
}

impl MapGroup {
    /// Claim the next creation-order sequence number.
    pub(crate) const fn claim_seq(&mut self) -> u64 {
        let claimed = self.next_seq;
        self.next_seq = claimed.saturating_add(1);
        claimed
    }
}

/// A node the caller asked to be told about every turn, and on whose authority.
///
/// A pin is **not** a frontier entry: a pinned node renders with its *current*
/// lifecycle and blocked state whatever those are, because the agent asked to be
/// told about this node and being told it is now blocked is the answer, not a
/// reason to hide it (projection-bounds sketch §(c)).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Pin {
    pub(crate) at: DesignId,
    pub(crate) authority: super::traversal::Authority,
}

/// One fingerprinted draft section. The body is stored **unbounded** — the
/// obvious case of prose the snapshot already keeps whole.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Section {
    pub(crate) id: DesignId,
    #[serde(default)]
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) body: String,
    pub(crate) fingerprint: Fingerprint,
    /// Where this section sits in the **document**, claimed once at creation
    /// from the run's existing [`MapGroup::claim_seq`] counter and carried
    /// unchanged through every later edit. Re-claiming on edit would move
    /// existing prose to the end of the document, which is the defect document
    /// order exists to close.
    ///
    /// `default` is what makes the migration a no-op *by construction*: a
    /// snapshot written before this field deserialises with every `seq` = 0, and
    /// a STABLE sort by `seq` leaves such a run in the id order it already had.
    /// There is no migration pass to get wrong.
    #[serde(default)]
    pub(crate) seq: u64,
    /// The 1-based line of the authored document this section was **imported**
    /// from (`EX-2`), and `None` when it was not imported — which is the
    /// truthful reading for every section declared through the run, and for
    /// every run that predates the field.
    ///
    /// Additive and optional on purpose (SL-233 PHASE-11 D3): a v1 snapshot
    /// still reads, `SUPPORTED_VERSIONS` stays a one-member set, and no wire
    /// break is bought for a provenance note. `skip_serializing_if` keeps a
    /// declared section's TOML byte-identical to what it was before the field
    /// existed, so the behaviour-preservation gate compares like with like.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) source_line: Option<usize>,
}

/// Fingerprinted draft sections, ordered by id so serialisation is
/// deterministic.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SectionGroup {
    #[serde(default, rename = "section")]
    pub(crate) sections: Vec<Section>,
}

impl SectionGroup {
    /// The section with `id`, if the run holds one.
    pub(crate) fn find(&self, id: &DesignId) -> Option<&Section> {
        self.sections.iter().find(|section| &section.id == id)
    }

    /// Insert or replace a section, keeping the group id-ordered.
    ///
    /// The id ordering is a SERIALISATION rule and stays exactly as it is.
    /// Document order is a different rule with a different owner
    /// ([`SectionGroup::document_order`]); conflating the two is what made the
    /// incumbent materialise `sec-11` before `sec-2`.
    pub(crate) fn upsert(&mut self, section: Section) {
        self.sections.retain(|held| held.id != section.id);
        self.sections.push(section);
        self.sections.sort_by(|a, b| a.id.cmp(&b.id));
    }

    /// The sections in **document** order — `seq`, sorted **stably**, so a run
    /// whose sections all carry the default `seq` = 0 keeps the id order it is
    /// already stored in.
    pub(crate) fn document_order(&self) -> Vec<&Section> {
        let mut ordered: Vec<&Section> = self.sections.iter().collect();
        ordered.sort_by_key(|section| section.seq);
        ordered
    }

    /// The set of sections the run holds — the thing both the document's marker
    /// set and a re-adoption's declared map are compared against. One owner, so
    /// the two comparisons cannot disagree about what "held" means.
    pub(crate) fn ids(&self) -> BTreeSet<DesignId> {
        self.sections
            .iter()
            .map(|section| section.id.clone())
            .collect()
    }

    /// Every section's current fingerprint — the observation the DEC-066
    /// liveness rule is evaluated against.
    pub(crate) fn fingerprints(&self) -> BTreeMap<DesignId, Fingerprint> {
        self.sections
            .iter()
            .map(|section| (section.id.clone(), section.fingerprint.clone()))
            .collect()
    }
}

/// One runtime review finding (design §5.3). Thin by intent: the review *record*
/// is a separate kind, and this is only what the run needs to project.
///
/// **Runtime data, and it stays that way** (design R3): a finding is never
/// promoted into the knowledge graph or the authored review ledger by anything
/// here. Promotion is a deliberate, separate act; silence would make it a side
/// effect of raising a concern.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Finding {
    pub(crate) id: DesignId,
    pub(crate) subject: DesignId,
    pub(crate) summary: String,
    /// Whether the lock gate must wait for this one. Defaults to `false` — a
    /// finding blocks only when it is *said* to, the same conservative reading
    /// provenance and authority take (design R2/R12).
    #[serde(default)]
    pub(crate) blocking: bool,
    /// How it was disposed. `None` is undisposed; a blank string is not a
    /// disposition either, which is why the predicate below reads the content
    /// rather than the presence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) resolution: Option<String>,
}

impl Finding {
    /// Whether this finding still holds the lock gate open.
    pub(crate) fn is_outstanding(&self) -> bool {
        self.blocking
            && self
                .resolution
                .as_ref()
                .is_none_or(|resolution| resolution.trim().is_empty())
    }
}

/// Content-bound review attestations, the integrated pass, runtime findings, and
/// the user's acceptance of the whole.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ReviewGroup {
    #[serde(default, rename = "attestation")]
    pub(crate) attestations: Vec<Attestation>,
    #[serde(default, rename = "finding")]
    pub(crate) findings: Vec<Finding>,
    /// The integrated adversarial pass — at most one, over the whole document.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) integrated: Option<IntegratedReview>,
    /// The user's acceptance of the design as locked (DEC-088).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) acceptance: Option<LockAcceptance>,
}

/// One prompt-fragment receipt: what the caller has already been shown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FragmentReceipt {
    pub(crate) name: String,
    pub(crate) digest: String,
}

/// Prompt-fragment receipts.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct FragmentGroup {
    #[serde(default, rename = "fragment")]
    pub(crate) fragments: Vec<FragmentReceipt>,
}

/// Recoverable checkpoint intent — journalled *before* the authored effect
/// (DEC-083, DEC-086), so recovery always has the exact canonical target.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CheckpointGroup {
    #[serde(default, rename = "intent")]
    pub(crate) intents: Vec<RecoveryIntent>,
}

/// The authored watermark: the fingerprint of `design.md` as Doctrine last left
/// it, plus whether Doctrine has ever left it at all.
///
/// `materialised` is what makes "absent" answerable: an absent `design.md`
/// before first materialisation is **cold**, and absent after it is
/// **divergent**. Without the flag those two are the same observation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct AuthoredGroup {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) watermark: Option<Fingerprint>,
    #[serde(default)]
    pub(crate) materialised: bool,
}

/// The canonical snapshot. Serialises 1:1 to
/// `design.toml` under the slice's runtime state tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DesignSnapshot {
    /// [`DESIGN_SNAPSHOT_SCHEMA`], checked on parse.
    pub(crate) schema: String,
    pub(crate) version: u32,
    pub(crate) run: RunHeader,
    #[serde(default)]
    pub(crate) receipts: ReceiptGroup,
    #[serde(default)]
    pub(crate) map: MapGroup,
    /// Gate evidence, each clearance bound to the subject fingerprint it was
    /// recorded against (DEC-066). Clearance itself is **derived, never stored**.
    #[serde(default)]
    pub(crate) gate: DerivedDesignFacts,
    #[serde(default)]
    pub(crate) sections: SectionGroup,
    #[serde(default)]
    pub(crate) review: ReviewGroup,
    /// Exported assignments and the proposals they are waiting for (DEC-068).
    /// Its own group, because delegation is its own state model.
    #[serde(default)]
    pub(crate) delegation: DelegationGroup,
    #[serde(default)]
    pub(crate) fragments: FragmentGroup,
    /// Runbook discharge records (SL-233 PHASE-16, DEC-101). Its own group,
    /// because an obligation discharge is its own state model — the reason
    /// `delegation` has one.
    #[serde(default)]
    pub(crate) runbook: RunbookGroup,
    #[serde(default)]
    pub(crate) checkpoint: CheckpointGroup,
    #[serde(default)]
    pub(crate) authored: AuthoredGroup,
    #[serde(default)]
    pub(crate) change_log: ChangeLog,
}

impl DesignSnapshot {
    /// What this run's review state says about the four derived lock conditions
    /// (design §5.4).
    ///
    /// Derived on every evaluation, never stored — the DEC-066/DEC-067 rule that
    /// governs every other clearance. Its home is here rather than in
    /// [`super::gate`] because it reads two of the snapshot's groups; `gate` owns
    /// what the answers *mean*, this owns where they come from.
    pub(crate) fn review_standing(&self) -> ReviewStanding {
        let current = self.sections.fingerprints();
        // Coverage, not presence: a run with three sections and two live
        // attestations has not been reviewed. `is_empty` is the degenerate case —
        // a run holding no sections cannot have current attestations for them,
        // and reporting `true` would let an empty draft lock. It CANNOT be folded
        // into the emptiness of the unreviewed list: a run with no sections owes
        // no lanes, so that list is empty and would read as satisfied.
        let sections_attested = !current.is_empty() && self.sections_unreviewed().is_empty();
        ReviewStanding {
            sections_attested,
            integrated_current: self
                .review
                .integrated
                .as_ref()
                .is_some_and(|review| review.is_current(&current)),
            findings_disposed: !self
                .review
                .findings
                .iter()
                .any(super::snapshot::Finding::is_outstanding),
            acceptance_current: self
                .review
                .acceptance
                .as_ref()
                .is_some_and(|accepted| accepted.is_current(&current)),
        }
    }

    /// The lanes `subject` still owes at `fingerprint` — the lanes this run's
    /// policy requires with no live attestation naming them (ISS-310).
    ///
    /// Content-bound through `fingerprint`: an attestation given over other bytes
    /// is not an attestation of these, which is DEC-066's rule and not a second
    /// mechanism.
    pub(crate) fn missing_lanes(
        &self,
        subject: &DesignId,
        fingerprint: &Fingerprint,
    ) -> Vec<ActorClass> {
        self.run
            .review_policy
            .lanes()
            .iter()
            .copied()
            .filter(|lane| {
                !self.review.attestations.iter().any(|held| {
                    held.subject() == subject
                        && held.fingerprint() == fingerprint
                        && ActorClass::from(held.reviewer()) == *lane
                })
            })
            .collect()
    }

    /// Every (section, lane) pair the run still owes over current content —
    /// id-ordered, so a refusal naming them renders deterministically.
    ///
    /// The nested quantification `section-attestations-current` means, in one
    /// home. [`DesignSnapshot::review_standing`]'s `sections_attested` is defined
    /// *through* it, exactly as [`ContentCoverage::is_current`] is defined through
    /// [`ContentCoverage::diff`]: the verdict and the explanation cannot disagree
    /// because there is only one of them.
    ///
    /// [`ContentCoverage`]: super::attestation::ContentCoverage
    /// [`ContentCoverage::diff`]: super::attestation::ContentCoverage::diff
    /// [`ContentCoverage::is_current`]: super::attestation::ContentCoverage::is_current
    pub(crate) fn sections_unreviewed(&self) -> Vec<(DesignId, ActorClass)> {
        self.sections
            .fingerprints()
            .into_iter()
            .flat_map(|(subject, fingerprint)| {
                self.missing_lanes(&subject, &fingerprint)
                    .into_iter()
                    .map(move |lane| (subject.clone(), lane))
            })
            .collect()
    }

    /// A fresh run at revision 1, stage `exploring`.
    ///
    /// Both floors start at 1: the log and the receipt history cover the run
    /// from its first revision, and neither is inferred from what happens to be
    /// stored.
    pub(crate) fn new(uid: impl Into<String>, slice: u32, watermark: Option<Fingerprint>) -> Self {
        DesignSnapshot {
            schema: DESIGN_SNAPSHOT_SCHEMA.to_owned(),
            version: DESIGN_SNAPSHOT_VERSION,
            run: RunHeader {
                uid: uid.into(),
                slice,
                revision: 1,
                stage: Stage::Exploring,
                review_policy: ReviewPolicy::default(),
                next_obligation: None,
            },
            receipts: ReceiptGroup {
                floor: 1,
                receipts: Vec::new(),
            },
            map: MapGroup::default(),
            gate: DerivedDesignFacts::default(),
            sections: SectionGroup::default(),
            review: ReviewGroup::default(),
            delegation: DelegationGroup::default(),
            fragments: FragmentGroup::default(),
            runbook: RunbookGroup::default(),
            checkpoint: CheckpointGroup::default(),
            authored: AuthoredGroup {
                watermark,
                materialised: false,
            },
            change_log: ChangeLog {
                floor: 1,
                rows: Vec::new(),
            },
        }
    }
}

/// Parse a snapshot body. Rejects a wrong `schema` discriminator or a version
/// outside [`SUPPORTED_VERSIONS`] with a remedy-naming message — the
/// `comparison::wire::parse` contract, applied to this file.
pub(crate) fn parse(text: &str) -> anyhow::Result<DesignSnapshot> {
    let snapshot: DesignSnapshot = toml::from_str(text)?;
    if snapshot.schema != DESIGN_SNAPSHOT_SCHEMA {
        anyhow::bail!(
            "unrecognized design-run schema `{}` (expected `{DESIGN_SNAPSHOT_SCHEMA}`)",
            snapshot.schema
        );
    }
    if !SUPPORTED_VERSIONS.contains(&snapshot.version) {
        let supported: Vec<String> = SUPPORTED_VERSIONS.iter().map(u32::to_string).collect();
        anyhow::bail!(
            "unsupported design-run snapshot version {} (expected one of: {}) — \
             this run was written by a different build of doctrine; upgrade doctrine, \
             or delete the snapshot and re-run `doctrine design start` \
             (the authored design document is untouched either way)",
            snapshot.version,
            supported.join(", ")
        );
    }
    Ok(snapshot)
}

/// Serialize to a snapshot body (serde-escaped — no raw splicing).
pub(crate) fn to_toml(snapshot: &DesignSnapshot) -> anyhow::Result<String> {
    Ok(toml::to_string(snapshot)?)
}

#[cfg(test)]
mod tests {
    use super::super::attestation::{
        AcceptanceAttestation, ContentCoverage, ReviewPolicy, Reviewer,
    };
    use super::super::fixture::{attest, id, run_holding, section};
    use super::*;

    /// Raise one finding against `sec-a`.
    fn raise(snapshot: &mut DesignSnapshot, raw: &str, blocking: bool, resolution: Option<&str>) {
        snapshot.review.findings.push(Finding {
            id: id(raw),
            subject: id("sec-a"),
            summary: "something is wrong".to_owned(),
            blocking,
            resolution: resolution.map(str::to_owned),
        });
    }

    /// Attest every section the run currently holds, in the default lane.
    fn attest_all(snapshot: &mut DesignSnapshot) {
        let subjects: Vec<String> = snapshot
            .sections
            .fingerprints()
            .keys()
            .map(|subject| subject.as_str().to_owned())
            .collect();
        for subject in subjects {
            let attestation = format!("att-{}", subject.replace("sec-", ""));
            attest(snapshot, &attestation, &subject, Reviewer::Human);
        }
    }

    #[test]
    fn section_attestations_are_current_only_when_every_section_is_covered() {
        let mut snapshot = run_holding(&[("sec-a", "sha256:a"), ("sec-b", "sha256:b")]);
        assert!(
            !snapshot.review_standing().sections_attested,
            "an unreviewed run is not attested"
        );

        // One of two: coverage, not presence, is what the condition means.
        snapshot.review.attestations.push(Attestation::bind(
            id("att-a"),
            id("sec-a"),
            Fingerprint::new("sha256:a"),
            Reviewer::Human,
        ));
        assert!(
            !snapshot.review_standing().sections_attested,
            "a partially reviewed run is not attested"
        );

        snapshot.review.attestations.push(Attestation::bind(
            id("att-b"),
            id("sec-b"),
            Fingerprint::new("sha256:b"),
            Reviewer::Human,
        ));
        assert!(snapshot.review_standing().sections_attested);

        // And an edit to one section drops the standing, because that
        // section is no longer covered at the fingerprint it was attested at.
        snapshot
            .sections
            .upsert(section("sec-a", "sha256:a-revised"));
        assert!(!snapshot.review_standing().sections_attested);
    }

    /// The review policy's migration is a no-op *by construction*, the same
    /// argument [`Section::seq`] makes: a snapshot written before the field
    /// deserialises to the default, and the default is the behaviour that run
    /// already had. There is no migration pass to get wrong.
    #[test]
    fn a_snapshot_written_before_the_policy_reads_as_human_only() {
        let pre_policy = format!(
            "schema = \"{DESIGN_SNAPSHOT_SCHEMA}\"\n\
             version = {DESIGN_SNAPSHOT_VERSION}\n\
             \n\
             [run]\n\
             uid = \"dr-test\"\n\
             slice = 233\n\
             revision = 4\n\
             stage = \"reviewing\"\n"
        );

        let parsed = parse(&pre_policy).expect("a pre-policy snapshot still parses");
        assert_eq!(parsed.run.review_policy, ReviewPolicy::HumanOnly);
        assert_eq!(parsed.run.revision, 4, "the rest of the header is unmoved");
    }

    #[test]
    fn a_run_holding_no_sections_is_never_attested() {
        // The degenerate case `all()` gets wrong: vacuous truth over an empty
        // set would let an empty draft clear the gate.
        let snapshot = run_holding(&[]);
        assert!(!snapshot.review_standing().sections_attested);
    }

    #[test]
    fn whole_run_evidence_stops_being_current_when_any_section_moves() {
        let mut snapshot = run_holding(&[("sec-a", "sha256:a"), ("sec-b", "sha256:b")]);
        attest_all(&mut snapshot);
        let covered = ContentCoverage::of(snapshot.sections.fingerprints());
        snapshot.review.integrated = Some(IntegratedReview::over(id("int-1"), covered.clone()));
        snapshot.review.acceptance = Some(LockAcceptance::over(
            AcceptanceAttestation::bind("the user said so", None, Fingerprint::new("sha256:pay")),
            covered,
        ));

        let standing = snapshot.review_standing();
        assert!(standing.integrated_current && standing.acceptance_current);

        // A section neither of them named individually still moves both, because
        // both were given over the document.
        snapshot
            .sections
            .upsert(section("sec-b", "sha256:b-revised"));
        let standing = snapshot.review_standing();
        assert!(!standing.integrated_current, "the integrated pass is stale");
        assert!(!standing.acceptance_current, "the acceptance is stale");

        // A section *added* after the fact is the other half: coverage is the
        // whole set, so new content is uncovered content.
        let mut widened = run_holding(&[("sec-a", "sha256:a"), ("sec-b", "sha256:b")]);
        attest_all(&mut widened);
        widened.review.integrated = Some(IntegratedReview::over(
            id("int-1"),
            ContentCoverage::of(widened.sections.fingerprints()),
        ));
        widened.sections.upsert(section("sec-c", "sha256:c"));
        assert!(!widened.review_standing().integrated_current);
    }

    #[test]
    fn only_a_blocking_finding_that_is_undisposed_holds_the_gate() {
        let mut snapshot = run_holding(&[("sec-a", "sha256:a")]);

        // Non-blocking and undisposed: the gate does not wait for it.
        raise(&mut snapshot, "fnd-1", false, None);
        assert!(snapshot.review_standing().findings_disposed);

        // Blocking and undisposed: it does.
        raise(&mut snapshot, "fnd-2", true, None);
        assert!(!snapshot.review_standing().findings_disposed);

        // A blank resolution is not a disposition, so the predicate reads the
        // content and not the presence.
        snapshot.review.findings.clear();
        raise(&mut snapshot, "fnd-3", true, Some("   \n"));
        assert!(!snapshot.review_standing().findings_disposed);

        snapshot.review.findings.clear();
        raise(
            &mut snapshot,
            "fnd-4",
            true,
            Some("accepted — fixed in §5.4"),
        );
        assert!(snapshot.review_standing().findings_disposed);
    }
}
