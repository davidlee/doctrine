// SPDX-License-Identifier: GPL-3.0-only
//! `comparison::claims` — the ledgered-value-claim resolution pass (SL-220
//! design §2, SL-222 design §2/E3): a generic fold between tier-1 resolution
//! and tier-2 compilation, instantiated for value (`f64`) and estimate
//! (`EstimatePayload`) payloads. Pure leaf (ADR-001): depends only on the
//! `wire` model, sibling result types (`resolve::ResolutionStatus`,
//! `compile::AnchorMap` — the `RowState` precedent), and std `BTree`
//! collections. No clock, disk, config reads.
//!
//! Input selection is this pass's OWN (RV-278 F-2): anchor-form rows of the
//! selected domain with resolution status `Active` **or `InertLens`** — R5's
//! lens inertness is a *constraint compilation* gate, not a claim-capture gate
//! (`resolve` marks every lens-tagged row `InertLens` unconditionally, so an
//! Active-only selection would empty the `lensed` output forever).
//! Superseded/tombstoned/malformed rows stay excluded — supersession reduces
//! lensed threads too.
//!
//! No mutation path exists here (RV-275 F-5): demoting a pin = appending a
//! superseding row, reduced by resolution before this pass ever sees it.

use std::collections::BTreeMap;

use super::compile::AnchorMap;
use super::resolve::ResolutionStatus;
use super::{AdmissionKind, DOMAIN_VALUE, Judgement, RaterKind, RowForm};

// ---------------------------------------------------------------------------
// ClaimPayload trait (SL-222 design §2/E3)
// ---------------------------------------------------------------------------

/// The generic claim payload (SL-222 E3): the domain-specific evidence
/// columns that a claim pass extracts from a judgement row, combines via
/// multiset mean, and reduces to an operative scalar via domain parameters.
pub(crate) trait ClaimPayload: Clone + PartialEq {
    /// Extract the payload columns from a judgement row — `None` if the row
    /// carries no claim in this domain (e.g. a value-domain row in an
    /// estimate pass, or a missing payload field).
    fn extract(j: &Judgement) -> Option<Self>;

    /// The multiset mean over the winning tier's payloads — the resolution
    /// point (D3).
    fn mean(rows: &[Self]) -> Self;

    /// Reduce the payload to an operative scalar using domain parameters.
    /// For value: identity. For estimate: affine(skew) + floor.
    fn operative(&self, params: &Self::Params) -> f64;

    /// Domain-specific parameters passed into the fold (pure — no config
    /// reads inside the pass). For value: `()`. For estimate: `f64` (skew).
    type Params: Clone;
}

// ---------------------------------------------------------------------------
// Value-domain instantiation (SL-220 §2, behaviour-preservation gate)
// ---------------------------------------------------------------------------

impl ClaimPayload for f64 {
    fn extract(j: &Judgement) -> Option<Self> {
        j.magnitude
    }
    fn mean(rows: &[Self]) -> Self {
        let count = u32::try_from(rows.len()).unwrap_or(u32::MAX);
        rows.iter().sum::<f64>() / f64::from(count)
    }
    fn operative(&self, _params: &()) -> f64 {
        *self
    }
    type Params = ();
}

/// Two-dimensional estimate payload (`est_lower`, `est_upper`) — SL-222 E4.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct EstimatePayload(pub(crate) f64, pub(crate) f64);

impl ClaimPayload for EstimatePayload {
    fn extract(j: &Judgement) -> Option<Self> {
        Some(EstimatePayload(j.est_lower?, j.est_upper?))
    }
    fn mean(rows: &[Self]) -> Self {
        let count = u32::try_from(rows.len()).unwrap_or(u32::MAX);
        let len = f64::from(count);
        let l = rows.iter().map(|p| p.0).sum::<f64>() / len;
        let u = rows.iter().map(|p| p.1).sum::<f64>() / len;
        EstimatePayload(l, u)
    }
    fn operative(&self, params: &f64) -> f64 {
        crate::estimate::operative_cost((self.0, self.1), *params)
    }
    type Params = f64;
}

// ---------------------------------------------------------------------------
// Tier model
// ---------------------------------------------------------------------------

/// The claim evidence ladder (design §2). Declaration order is ASCENDING so
/// the derived `Ord` and `Iterator::max` are correct by construction — pinned
/// by `pin_outranks_all_tiers_under_derived_ord`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ClaimTier {
    Migrated,
    Agent,
    Human,
    Pin,
}

impl ClaimTier {
    /// The D4 anti-laundering split, and D14's reprobe gate — ONE predicate
    /// for both questions: Pin/Human claims anchor the constraint layer and
    /// (when conflicted) nominate for the human reprobe queue; Agent/Migrated
    /// claims are below-projection priors and never do either.
    pub(crate) fn is_anchored(self) -> bool {
        matches!(self, ClaimTier::Pin | ClaimTier::Human)
    }
}

// ---------------------------------------------------------------------------
// Resolved claim (generic)
// ---------------------------------------------------------------------------

/// One item's resolved claim within one partition (design §2, SL-222 E3).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ResolvedClaimGeneric<P: ClaimPayload> {
    /// The singleton operative scalar, or the D3 conflict mean — cached at
    /// resolution time so consumers (scoring, surface, reprobe) read a stable
    /// value. For value payloads this equals the magnitude mean; for estimate
    /// payloads this is the β-skewed, EPSILON-floored collapse.
    pub(crate) operative: f64,
    /// The winning-tier payload (the domain-specific evidence columns).
    pub(crate) payload: P,
    pub(crate) tier: ClaimTier,
    /// Present ⇔ >1 distinct payload in the winning tier.
    pub(crate) conflict: Option<ClaimConflict>,
    /// Active winning-tier row count (render).
    pub(crate) rows: u32,
    /// SL-220 PHASE-06 render attribution (design §6).
    pub(crate) attribution: Option<ClaimAttribution>,
}

/// Value-domain alias — consumers that previously used `ResolvedClaim` now
/// refer to `ResolvedClaimGeneric<f64>` via this type.
pub(crate) type ResolvedClaim = ResolvedClaimGeneric<f64>;

// ---------------------------------------------------------------------------
// Shared types
// ---------------------------------------------------------------------------

/// Render attribution for a singleton claim (design §6).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ClaimAttribution {
    pub by: Option<String>,
    pub date: Option<String>,
    pub observed_at: Option<String>,
    pub basis: Option<String>,
}

/// The D3 conflict interval: rendered bounds + distinct-payload count.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ClaimConflict {
    pub low: f64,
    pub high: f64,
    pub distinct: u32,
}

// ---------------------------------------------------------------------------
// Finding shape
// ---------------------------------------------------------------------------

/// A claim-pass finding (design §2), domain-tagged at construction (SL-219
/// D9). Conflicts fire at EVERY tier — surfaced-never-silent — but reprobe
/// nomination is anchored-tiers-only (D14, [`ClaimFinding::nominates_reprobe`]).
/// The interval is over operative values (for value, operative ≡ magnitude;
/// for estimate, operative ≡ β-skewed cost).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ClaimFinding {
    /// Same-tier disagreement on one item's operative ("contested pin" when
    /// `tier` is [`ClaimTier::Pin`]): the winning tier's full multiset mean
    /// stands as the point operative; the interval and distinct count ride here.
    Conflict {
        domain: String,
        item: String,
        tier: ClaimTier,
        low: f64,
        high: f64,
        distinct: u32,
        rows: u32,
    },
}

impl ClaimFinding {
    /// D14: a Pin/Human conflict is a "the humans must talk" reprobe
    /// candidate — `priority::elicit`'s claim-reprobe source keys on this,
    /// KNOB-INDEPENDENTLY; an Agent/Migrated conflict carries
    /// calibrate-via-comparison guidance (rendered at PHASE-06) and NEVER
    /// enters the human queue.
    pub(crate) fn nominates_reprobe(&self) -> bool {
        let ClaimFinding::Conflict { tier, .. } = self;
        tier.is_anchored()
    }
}

// ---------------------------------------------------------------------------
// Claim resolution (generic)
// ---------------------------------------------------------------------------

/// The claim-resolution output (design §2): per-item resolved claims routed
/// by the D4 anti-laundering split, the inert lensed partitions (D5,
/// IDE-035 seam), and the finding stream.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ClaimResolutionGeneric<P: ClaimPayload> {
    /// Pin/Human — the [`AnchorMap`] feed + graph rung 1.
    pub anchored: BTreeMap<String, ResolvedClaimGeneric<P>>,
    /// Agent/Migrated — graph-ladder priors below projection; these bypass
    /// the constraint layer entirely (D4).
    pub priors: BTreeMap<String, ResolvedClaimGeneric<P>>,
    /// (lens, item) — resolved identically per partition, consumed by no
    /// scoring surface (D5); rendered on demand.
    pub lensed: BTreeMap<(String, String), ResolvedClaimGeneric<P>>,
    pub findings: Vec<ClaimFinding>,
}

/// Value-domain alias.
pub(crate) type ClaimResolution = ClaimResolutionGeneric<f64>;

impl<P: ClaimPayload> Default for ClaimResolutionGeneric<P> {
    fn default() -> Self {
        Self {
            anchored: BTreeMap::new(),
            priors: BTreeMap::new(),
            lensed: BTreeMap::new(),
            findings: Vec::new(),
        }
    }
}

impl<P: ClaimPayload> ClaimResolutionGeneric<P> {
    /// Anti-laundering (D4): EXACTLY the `anchored` operative scalars — an
    /// agent or migrated claim can never enter `compile` through this seam.
    pub(crate) fn anchor_map(&self) -> AnchorMap {
        self.anchored
            .iter()
            .map(|(item, claim)| (item.clone(), claim.operative))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Internal state & generic fold (SL-222 E3)
// ---------------------------------------------------------------------------

/// One selected claim row's evidence: its tier, domain payload, render
/// attribution, and the domain params needed to compute the operative scalar.
struct PayloadRow<P: ClaimPayload> {
    tier: ClaimTier,
    payload: P,
    params: P::Params,
    attribution: ClaimAttribution,
}

/// Resolve claims for a given domain and payload type (SL-222 E3). Pure fold;
/// deterministic under row permutation (`BTreeMap`s throughout; the conflict
/// mean is summed in `total_cmp` order).
///
/// `domain` selects the judgement domain; `params` is the domain-specific
/// parameters (`()` for value, `f64` skew for estimate).
pub(crate) fn resolve_claims_generic<P: ClaimPayload>(
    rows: &[(&Judgement, ResolutionStatus)],
    domain: &str,
    params: &P::Params,
) -> ClaimResolutionGeneric<P> {
    let mut unlensed: BTreeMap<String, Vec<PayloadRow<P>>> = BTreeMap::new();
    let mut lensed_groups: BTreeMap<(String, String), Vec<PayloadRow<P>>> = BTreeMap::new();
    for (j, status) in rows {
        if j.domain != domain
            || !matches!(j.form, RowForm::Anchor)
            || !matches!(
                status,
                ResolutionStatus::Active | ResolutionStatus::InertLens
            )
        {
            continue;
        }
        let Some(payload) = P::extract(j) else {
            continue;
        };
        let row = PayloadRow {
            tier: tier_of(j),
            payload,
            params: params.clone(),
            attribution: ClaimAttribution {
                by: j.by.clone(),
                date: j.date.clone(),
                observed_at: j.observed_at.clone(),
                basis: j.basis.clone(),
            },
        };
        match &j.lens {
            Some(lens) => lensed_groups
                .entry((lens.clone(), j.a.clone()))
                .or_default()
                .push(row),
            None => unlensed.entry(j.a.clone()).or_default().push(row),
        }
    }

    let mut out = ClaimResolutionGeneric::default();
    for (item, group) in unlensed {
        let claim = resolve_group::<P>(&group, params);
        if let Some(conflict) = &claim.conflict {
            out.findings.push(ClaimFinding::Conflict {
                domain: domain.to_string(),
                item: item.clone(),
                tier: claim.tier,
                low: conflict.low,
                high: conflict.high,
                distinct: conflict.distinct,
                rows: claim.rows,
            });
        }
        if claim.tier.is_anchored() {
            out.anchored.insert(item, claim);
        } else {
            out.priors.insert(item, claim);
        }
    }
    for (key, group) in lensed_groups {
        out.lensed.insert(key, resolve_group::<P>(&group, params));
    }
    out
}

// ---------------------------------------------------------------------------
// Value-domain entry point (concrete, behaviour-preservation gate)
// ---------------------------------------------------------------------------

/// Resolve the value-domain claim ladder over the post-resolve row set
/// (design §2, SL-220). Delegates to the generic fold with `P = f64`.
pub(crate) fn resolve_claims(rows: &[(&Judgement, ResolutionStatus)]) -> ClaimResolution {
    resolve_claims_generic::<f64>(rows, DOMAIN_VALUE, &())
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Tier one row (design §2 step 2): `admission = pin` → Pin, else by rater.
fn tier_of(j: &Judgement) -> ClaimTier {
    match (&j.admission, &j.rater) {
        (Some(AdmissionKind::Pin), _) => ClaimTier::Pin,
        (None, RaterKind::Human) => ClaimTier::Human,
        (None, RaterKind::Agent) => ClaimTier::Agent,
        (None, RaterKind::Migrated) => ClaimTier::Migrated,
    }
}

/// Resolve one partition's group (design §2 steps 3–4): winning tier =
/// highest non-empty; lower tiers contribute NOTHING (not even bounds).
/// One distinct payload → that operative scalar with the full row count
/// (corroboration). Multiple → the arithmetic mean over the FULL winning-tier
/// row multiset (D3 — no dedupe), with `{min, max}` over operative values.
/// `distinct` counts distinct payloads — two rows disagreeing on range but
/// coinciding on operative cost still conflict, with a degenerate interval.
fn resolve_group<P: ClaimPayload>(
    group: &[PayloadRow<P>],
    params: &P::Params,
) -> ResolvedClaimGeneric<P> {
    let winning = group
        .iter()
        .map(|r| r.tier)
        .max()
        .unwrap_or(ClaimTier::Migrated); // unreachable: groups are non-empty
    let winning_rows: Vec<&PayloadRow<P>> = group.iter().filter(|r| r.tier == winning).collect();
    // Attribution is nameable only for a SINGLETON winning-tier row —
    // permutation-invariant (present iff exactly one such row).
    let attribution = match winning_rows.as_slice() {
        [only] => Some(only.attribution.clone()),
        _ => None,
    };

    // Payloads (for distinct counting) and their pre-computed operatives.
    let payloads: Vec<P> = winning_rows.iter().map(|r| r.payload.clone()).collect();
    let operatives: Vec<f64> = winning_rows
        .iter()
        .map(|r| r.payload.operative(&r.params))
        .collect();

    let rows = u32::try_from(payloads.len()).unwrap_or(u32::MAX);
    let mut distinct = 0_u32;
    for i in 0..payloads.len() {
        let Some(current) = payloads.get(i) else {
            break;
        };
        if !payloads.get(..i).is_some_and(|s| s.contains(current)) {
            distinct += 1;
        }
    }

    let (operative, payload, conflict) = if distinct > 1 {
        let mean_payload = P::mean(&payloads);
        let mean_op = mean_payload.operative(params);
        let (low, high) = (
            operatives
                .iter()
                .copied()
                .min_by(f64::total_cmp)
                .unwrap_or(mean_op),
            operatives
                .iter()
                .copied()
                .max_by(f64::total_cmp)
                .unwrap_or(mean_op),
        );
        (
            mean_op,
            mean_payload,
            Some(ClaimConflict {
                low,
                high,
                distinct,
            }),
        )
    } else if let Some(only) = winning_rows.first() {
        (
            only.payload.operative(&only.params),
            only.payload.clone(),
            None,
        )
    } else {
        // Unreachable: groups are non-empty; winning_rows is non-empty.
        // Return degenerate result — the value doesn't matter because this
        // branch is never taken; typed as a tuple to match the let-binding.
        let fallback = P::mean(&[]);
        (fallback.operative(params), fallback, None)
    };

    ResolvedClaimGeneric {
        operative,
        payload,
        tier: winning,
        conflict,
        rows,
        attribution,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        ClaimFinding, ClaimPayload, ClaimResolution, ClaimResolutionGeneric, ClaimTier,
        EstimatePayload, resolve_claims, resolve_claims_generic,
    };
    use crate::comparison::resolve::{ResolutionStatus, StatusMap, resolve};
    use crate::comparison::{
        AdmissionKind, COMPARISON_SCHEMA, COMPARISON_VERSION, ComparisonSession, DOMAIN_ESTIMATE,
        FRAME_COST_ANCHOR, FRAME_VALUE_ANCHOR, Judgement, RaterKind, RowForm, SessionHeader,
    };

    // -----------------------------------------------------------------------
    // Fixtures — value domain
    // -----------------------------------------------------------------------

    fn anchor(uid: &str, item: &str, magnitude: f64, rater: RaterKind) -> Judgement {
        let migrated = matches!(rater, RaterKind::Migrated);
        Judgement {
            uid: uid.to_string(),
            seq: 0,
            a: item.to_string(),
            b: None,
            response: None,
            domain: crate::comparison::DOMAIN_VALUE.to_string(),
            frame: FRAME_VALUE_ANCHOR.to_string(),
            form: RowForm::Anchor,
            magnitude: Some(magnitude),
            supersedes: None,
            lens: None,
            rater,
            by: None,
            note: None,
            date: (!migrated).then(|| "2026-07-16".to_string()),
            observed_at: migrated.then(|| "2026-07-16".to_string()),
            basis: None,
            est_lower: None,
            est_upper: None,
            admission: None,
        }
    }

    fn pin(uid: &str, item: &str, magnitude: f64) -> Judgement {
        let mut j = anchor(uid, item, magnitude, RaterKind::Human);
        j.admission = Some(AdmissionKind::Pin);
        j
    }

    fn lensed(uid: &str, item: &str, magnitude: f64, lens: &str) -> Judgement {
        let mut j = anchor(uid, item, magnitude, RaterKind::Human);
        j.lens = Some(lens.to_string());
        j
    }

    fn active(rows: &[Judgement]) -> Vec<(&Judgement, ResolutionStatus)> {
        rows.iter()
            .map(|j| {
                let status = if j.lens.is_some() {
                    ResolutionStatus::InertLens
                } else {
                    ResolutionStatus::Active
                };
                (j, status)
            })
            .collect()
    }

    fn session(uid: &str, judgements: Vec<Judgement>) -> ComparisonSession {
        ComparisonSession {
            schema: COMPARISON_SCHEMA.to_string(),
            version: COMPARISON_VERSION,
            session: SessionHeader {
                uid: uid.to_string(),
                date: "2026-07-16".to_string(),
                audience: None,
            },
            judgements,
            tombstones: Vec::new(),
        }
    }

    fn conflict_of<'a>(claims: &'a ClaimResolution, item: &str) -> &'a ClaimFinding {
        claims
            .findings
            .iter()
            .find(|f| {
                let ClaimFinding::Conflict { item: i, .. } = f;
                i == item
            })
            .expect("conflict finding present")
    }

    // -----------------------------------------------------------------------
    // Fixtures — estimate domain
    // -----------------------------------------------------------------------

    fn est_anchor(uid: &str, item: &str, lower: f64, upper: f64, rater: RaterKind) -> Judgement {
        let migrated = matches!(rater, RaterKind::Migrated);
        Judgement {
            uid: uid.to_string(),
            seq: 0,
            a: item.to_string(),
            b: None,
            response: None,
            domain: DOMAIN_ESTIMATE.to_string(),
            frame: FRAME_COST_ANCHOR.to_string(),
            form: RowForm::Anchor,
            magnitude: None,
            supersedes: None,
            lens: None,
            rater,
            by: None,
            note: None,
            date: (!migrated).then(|| "2026-07-16".to_string()),
            observed_at: migrated.then(|| "2026-07-16".to_string()),
            basis: None,
            est_lower: Some(lower),
            est_upper: Some(upper),
            admission: None,
        }
    }

    fn est_pin(uid: &str, item: &str, lower: f64, upper: f64) -> Judgement {
        let mut j = est_anchor(uid, item, lower, upper, RaterKind::Human);
        j.admission = Some(AdmissionKind::Pin);
        j
    }

    fn est_lensed(uid: &str, item: &str, lower: f64, upper: f64, lens: &str) -> Judgement {
        let mut j = est_anchor(uid, item, lower, upper, RaterKind::Human);
        j.lens = Some(lens.to_string());
        j
    }

    fn est_session(uid: &str, judgements: Vec<Judgement>) -> ComparisonSession {
        ComparisonSession {
            schema: COMPARISON_SCHEMA.to_string(),
            version: COMPARISON_VERSION,
            session: SessionHeader {
                uid: uid.to_string(),
                date: "2026-07-16".to_string(),
                audience: None,
            },
            judgements,
            tombstones: Vec::new(),
        }
    }

    fn est_conflict_of<'a, P: ClaimPayload>(
        claims: &'a ClaimResolutionGeneric<P>,
        item: &str,
    ) -> &'a ClaimFinding {
        claims
            .findings
            .iter()
            .find(|f| {
                let ClaimFinding::Conflict { item: i, .. } = f;
                i == item
            })
            .expect("conflict finding present")
    }

    // -----------------------------------------------------------------------
    // the RV-275 F-1 gate battery (value domain)
    // -----------------------------------------------------------------------

    #[test]
    fn pin_outranks_all_tiers_under_derived_ord() {
        assert!(ClaimTier::Migrated < ClaimTier::Agent);
        assert!(ClaimTier::Agent < ClaimTier::Human);
        assert!(ClaimTier::Human < ClaimTier::Pin);
        let all = [
            ClaimTier::Pin,
            ClaimTier::Migrated,
            ClaimTier::Human,
            ClaimTier::Agent,
        ];
        assert_eq!(all.iter().max(), Some(&ClaimTier::Pin));
    }

    #[test]
    fn resolution_is_invariant_under_row_permutation() {
        let rows = [
            anchor("h1", "SL-100", 5.0, RaterKind::Human),
            anchor("h2", "SL-100", 5.0, RaterKind::Human),
            anchor("h3", "SL-100", 7.0, RaterKind::Human),
            anchor("a1", "SL-200", 2.0, RaterKind::Agent),
            anchor("m1", "SL-300", 1.0, RaterKind::Migrated),
            lensed("l1", "SL-100", 9.0, "user-value"),
        ];
        let forward = active(&rows);
        let baseline = resolve_claims(&forward);
        let mut reversed = forward.clone();
        reversed.reverse();
        assert_eq!(resolve_claims(&reversed), baseline);
        let mut rotated = forward.clone();
        rotated.rotate_left(3);
        assert_eq!(resolve_claims(&rotated), baseline);
    }

    #[test]
    fn corroborating_rows_resolve_without_conflict() {
        let rows = [
            anchor("h1", "SL-100", 5.0, RaterKind::Human),
            anchor("h2", "SL-100", 5.0, RaterKind::Human),
            anchor("h3", "SL-100", 5.0, RaterKind::Human),
        ];
        let claims = resolve_claims(&active(&rows));
        let claim = &claims.anchored["SL-100"];
        assert_eq!(claim.operative, 5.0);
        assert_eq!(claim.tier, ClaimTier::Human);
        assert_eq!(claim.conflict, None);
        assert_eq!(claim.rows, 3);
        assert!(claims.findings.is_empty());
    }

    #[test]
    fn conflict_takes_the_multiset_mean_with_interval_and_distinct() {
        let rows = [
            anchor("h1", "SL-100", 5.0, RaterKind::Human),
            anchor("h2", "SL-100", 5.0, RaterKind::Human),
            anchor("h3", "SL-100", 7.0, RaterKind::Human),
        ];
        let claims = resolve_claims(&active(&rows));
        let claim = &claims.anchored["SL-100"];
        assert_eq!(claim.operative, 17.0 / 3.0, "multiset mean");
        let conflict = claim.conflict.as_ref().expect("conflict present");
        assert_eq!((conflict.low, conflict.high), (5.0, 7.0));
        assert_eq!(conflict.distinct, 2);
        assert_eq!(claim.rows, 3);
        assert_eq!(
            conflict_of(&claims, "SL-100"),
            &ClaimFinding::Conflict {
                domain: "value".to_string(),
                item: "SL-100".to_string(),
                tier: ClaimTier::Human,
                low: 5.0,
                high: 7.0,
                distinct: 2,
                rows: 3,
            }
        );
    }

    #[test]
    fn conflicting_pins_raise_a_contested_pin_finding() {
        let rows = [pin("p1", "SL-100", 3.0), pin("p2", "SL-100", 9.0)];
        let claims = resolve_claims(&active(&rows));
        let claim = &claims.anchored["SL-100"];
        assert_eq!(claim.tier, ClaimTier::Pin);
        assert_eq!(claim.operative, 6.0);
        let finding = conflict_of(&claims, "SL-100");
        assert!(
            matches!(
                finding,
                ClaimFinding::Conflict {
                    tier: ClaimTier::Pin,
                    ..
                }
            ),
            "contested pin: {finding:?}"
        );
        assert!(finding.nominates_reprobe());
    }

    #[test]
    fn reprobe_nomination_is_anchored_tiers_only() {
        let rows = [
            anchor("h1", "SL-100", 4.0, RaterKind::Human),
            anchor("h2", "SL-100", 6.0, RaterKind::Human),
            anchor("a1", "SL-200", 1.0, RaterKind::Agent),
            anchor("a2", "SL-200", 3.0, RaterKind::Agent),
            anchor("m1", "SL-300", 1.0, RaterKind::Migrated),
            anchor("m2", "SL-300", 5.0, RaterKind::Migrated),
        ];
        let claims = resolve_claims(&active(&rows));
        assert!(conflict_of(&claims, "SL-100").nominates_reprobe());
        assert!(!conflict_of(&claims, "SL-200").nominates_reprobe());
        assert!(!conflict_of(&claims, "SL-300").nominates_reprobe());
        assert_eq!(claims.findings.len(), 3);
    }

    #[test]
    fn lower_tiers_contribute_nothing_to_a_won_item() {
        let rows = [
            anchor("h1", "SL-100", 6.0, RaterKind::Human),
            anchor("a1", "SL-100", 1.0, RaterKind::Agent),
            anchor("a2", "SL-100", 99.0, RaterKind::Agent),
            anchor("m1", "SL-100", 42.0, RaterKind::Migrated),
        ];
        let claims = resolve_claims(&active(&rows));
        let claim = &claims.anchored["SL-100"];
        assert_eq!(claim.operative, 6.0);
        assert_eq!(claim.conflict, None);
        assert_eq!(claim.rows, 1);
        assert!(!claims.priors.contains_key("SL-100"));
        assert!(claims.findings.is_empty());
    }

    #[test]
    fn agent_and_migrated_claims_route_to_priors() {
        let rows = [
            anchor("a1", "SL-100", 2.0, RaterKind::Agent),
            anchor("m1", "SL-200", 3.0, RaterKind::Migrated),
        ];
        let claims = resolve_claims(&active(&rows));
        assert!(claims.anchored.is_empty());
        assert_eq!(claims.priors["SL-100"].tier, ClaimTier::Agent);
        assert_eq!(claims.priors["SL-200"].tier, ClaimTier::Migrated);
    }

    #[test]
    fn cross_session_same_tier_claims_conflict_never_latest_wins() {
        let s1 = session("s1", vec![anchor("p", "SL-100", 4.0, RaterKind::Human)]);
        let s2 = session("s2", vec![anchor("q", "SL-100", 8.0, RaterKind::Human)]);
        let sessions = [s1, s2];
        let res = resolve(&sessions, &StatusMap::new()).expect("resolve ok");
        let claims = resolve_claims(&res.rows);
        let claim = &claims.anchored["SL-100"];
        assert_eq!(claim.operative, 6.0);
        assert!(claim.conflict.is_some());
        assert_eq!(claim.rows, 2);
    }

    #[test]
    fn identical_refire_changes_no_value_and_raises_no_conflict() {
        let once = [session(
            "s1",
            vec![anchor("p", "SL-100", 4.0, RaterKind::Human)],
        )];
        let twice = [
            session("s1", vec![anchor("p", "SL-100", 4.0, RaterKind::Human)]),
            session("s2", vec![anchor("q", "SL-100", 4.0, RaterKind::Human)]),
        ];
        let res_once = resolve(&once, &StatusMap::new()).expect("resolve ok");
        let res_twice = resolve(&twice, &StatusMap::new()).expect("resolve ok");
        let one = resolve_claims(&res_once.rows);
        let two = resolve_claims(&res_twice.rows);
        assert_eq!(
            one.anchored["SL-100"].operative,
            two.anchored["SL-100"].operative
        );
        assert_eq!(two.anchored["SL-100"].conflict, None);
        assert_eq!(two.anchored["SL-100"].rows, 2);
        assert!(two.findings.is_empty());
    }

    #[test]
    fn lens_isolation_holds_non_vacuously_in_both_directions() {
        let mixed = [
            session(
                "s1",
                vec![
                    anchor("h1", "SL-100", 5.0, RaterKind::Human),
                    anchor("a1", "SL-200", 2.0, RaterKind::Agent),
                    lensed("l1", "SL-100", 9.0, "user-value"),
                    lensed("l3", "SL-300", 1.0, "ops-value"),
                ],
            ),
            session("s2", vec![lensed("l2", "SL-100", 3.0, "user-value")]),
        ];
        let res = resolve(&mixed, &StatusMap::new()).expect("resolve ok");
        for uid in ["l1", "l2", "l3"] {
            let (_, status) = res
                .rows
                .iter()
                .find(|(j, _)| j.uid == uid)
                .expect("row present");
            assert_eq!(status, &ResolutionStatus::InertLens);
        }
        let with_lensed = resolve_claims(&res.rows);
        assert!(!with_lensed.lensed.is_empty());
        let key = ("user-value".to_string(), "SL-100".to_string());
        assert!(with_lensed.lensed[&key].conflict.is_some());
        assert_eq!(with_lensed.lensed[&key].operative, 6.0);

        let unlensed_only = [
            session(
                "s1",
                vec![
                    anchor("h1", "SL-100", 5.0, RaterKind::Human),
                    anchor("a1", "SL-200", 2.0, RaterKind::Agent),
                ],
            ),
            session("s2", vec![]),
        ];
        let res2 = resolve(&unlensed_only, &StatusMap::new()).expect("resolve ok");
        let without_lensed = resolve_claims(&res2.rows);
        assert_eq!(with_lensed.anchored, without_lensed.anchored);
        assert_eq!(with_lensed.priors, without_lensed.priors);
    }

    #[test]
    fn lensed_conflicts_do_not_enter_the_finding_stream() {
        let rows = [
            lensed("l1", "SL-100", 1.0, "user-value"),
            lensed("l2", "SL-100", 9.0, "user-value"),
        ];
        let claims = resolve_claims(&active(&rows));
        let key = ("user-value".to_string(), "SL-100".to_string());
        assert!(claims.lensed[&key].conflict.is_some());
        assert!(claims.findings.is_empty());
    }

    #[test]
    fn anchor_map_never_launders_agent_or_migrated_claims() {
        let minted = |tier: usize, item: &str, uid: &str| -> Judgement {
            match tier {
                0 => anchor(uid, item, 1.0, RaterKind::Migrated),
                1 => anchor(uid, item, 2.0, RaterKind::Agent),
                2 => anchor(uid, item, 3.0, RaterKind::Human),
                _ => pin(uid, item, 4.0),
            }
        };
        for mask in 0..16_u32.pow(3) {
            let mut rows: Vec<Judgement> = Vec::new();
            for (i, item) in ["SL-100", "SL-200", "SL-300"].iter().enumerate() {
                let tiers = (mask / 16_u32.pow(u32::try_from(i).unwrap())) % 16;
                for tier in 0..4 {
                    if tiers & (1 << tier) != 0 {
                        rows.push(minted(tier, item, &format!("j{i}t{tier}")));
                    }
                }
            }
            let claims = resolve_claims(&active(&rows));
            let map = claims.anchor_map();
            let anchored_values: BTreeMap<String, f64> = claims
                .anchored
                .iter()
                .map(|(k, c)| (k.clone(), c.operative))
                .collect();
            assert_eq!(map, anchored_values, "anchor_map ≡ anchored, mask {mask}");
            for claim in claims.anchored.values() {
                assert!(claim.tier.is_anchored());
            }
            for (item, claim) in &claims.priors {
                assert!(!claim.tier.is_anchored());
                assert!(!map.contains_key(item));
            }
        }
    }

    #[test]
    fn non_live_rows_carry_no_claim() {
        let rows = [
            anchor("dead", "SL-100", 9.0, RaterKind::Human),
            anchor("live", "SL-100", 5.0, RaterKind::Human),
            anchor("tomb", "SL-200", 3.0, RaterKind::Human),
        ];
        let tagged: Vec<(&Judgement, ResolutionStatus)> = vec![
            (
                &rows[0],
                ResolutionStatus::Superseded {
                    by: "live".to_string(),
                },
            ),
            (&rows[1], ResolutionStatus::Active),
            (&rows[2], ResolutionStatus::Tombstoned),
        ];
        let claims = resolve_claims(&tagged);
        let claim = &claims.anchored["SL-100"];
        assert_eq!((claim.operative, claim.rows), (5.0, 1));
        assert_eq!(claim.conflict, None);
        assert!(!claims.anchored.contains_key("SL-200"));
    }

    #[test]
    fn pairwise_rows_never_enter_the_claims_pass() {
        let mut order = anchor("ord", "SL-100", 5.0, RaterKind::Human);
        order.form = RowForm::Order;
        order.b = Some("SL-200".to_string());
        order.response = Some(crate::comparison::Response::PreferA);
        order.frame = crate::comparison::FRAME_EQUAL_EFFORT.to_string();
        let rows = [order];
        let claims = resolve_claims(&active(&rows));
        assert_eq!(claims, ClaimResolution::default());
    }

    // -----------------------------------------------------------------------
    // SL-222 estimate payload gate battery
    // -----------------------------------------------------------------------

    #[test]
    fn estimate_claim_payload_tier_ordering() {
        let est_rows = [
            est_anchor("h1", "SL-100", 2.0, 8.0, RaterKind::Human),
            est_anchor("a1", "SL-100", 1.0, 3.0, RaterKind::Agent),
            est_anchor("m1", "SL-100", 5.0, 5.0, RaterKind::Migrated),
        ];
        let active_rows: Vec<(&Judgement, ResolutionStatus)> = est_rows
            .iter()
            .map(|j| (j, ResolutionStatus::Active))
            .collect();
        let claims =
            resolve_claims_generic::<EstimatePayload>(&active_rows, DOMAIN_ESTIMATE, &0.65);
        let claim = &claims.anchored["SL-100"];
        assert_eq!(claim.tier, ClaimTier::Human);
        assert!((claim.operative - 5.9).abs() < 1e-12);
    }

    #[test]
    fn estimate_permutation_invariance() {
        let h1 = est_anchor("h1", "SL-100", 2.0, 8.0, RaterKind::Human);
        let h2 = est_anchor("h2", "SL-100", 2.0, 8.0, RaterKind::Human);
        let h3 = est_anchor("h3", "SL-100", 4.0, 6.0, RaterKind::Human);
        let rows = [
            (&h1, ResolutionStatus::Active),
            (&h2, ResolutionStatus::Active),
            (&h3, ResolutionStatus::Active),
        ];
        let baseline = resolve_claims_generic::<EstimatePayload>(&rows, DOMAIN_ESTIMATE, &0.65);
        let mut reversed = rows.to_vec();
        reversed.reverse();
        assert_eq!(
            resolve_claims_generic::<EstimatePayload>(&reversed, DOMAIN_ESTIMATE, &0.65),
            baseline
        );
    }

    #[test]
    fn estimate_corroboration_without_conflict() {
        let rows = [
            est_anchor("h1", "SL-100", 2.0, 8.0, RaterKind::Human),
            est_anchor("h2", "SL-100", 2.0, 8.0, RaterKind::Human),
        ];
        let active_rows: Vec<(&Judgement, ResolutionStatus)> =
            rows.iter().map(|j| (j, ResolutionStatus::Active)).collect();
        let claims =
            resolve_claims_generic::<EstimatePayload>(&active_rows, DOMAIN_ESTIMATE, &0.65);
        let claim = &claims.anchored["SL-100"];
        assert!((claim.operative - 5.9).abs() < 1e-12);
        assert_eq!(claim.conflict, None);
        assert_eq!(claim.rows, 2);
    }

    #[test]
    fn estimate_conflict_over_ranges() {
        let rows = [
            est_anchor("h1", "SL-100", 2.0, 8.0, RaterKind::Human),
            est_anchor("h2", "SL-100", 1.0, 3.0, RaterKind::Human),
        ];
        let active_rows: Vec<(&Judgement, ResolutionStatus)> =
            rows.iter().map(|j| (j, ResolutionStatus::Active)).collect();
        let claims =
            resolve_claims_generic::<EstimatePayload>(&active_rows, DOMAIN_ESTIMATE, &0.65);
        let claim = &claims.anchored["SL-100"];
        // Mean bounds: l=1.5, u=5.5 → op = 1.5 + 0.65*4.0 = 4.1
        assert!((claim.operative - 4.1).abs() < 1e-12);
        let conflict = claim.conflict.as_ref().expect("conflict present");
        // h1 op = 2 + 0.65*6 = 5.9; h2 op = 1 + 0.65*2 = 2.3
        assert!((conflict.low - 2.3).abs() < 1e-12);
        assert!((conflict.high - 5.9).abs() < 1e-12);
        assert_eq!(conflict.distinct, 2);
    }

    #[test]
    fn estimate_conflicting_pins_contested() {
        let rows = [
            est_pin("p1", "SL-100", 2.0, 8.0),
            est_pin("p2", "SL-100", 1.0, 3.0),
        ];
        let active_rows: Vec<(&Judgement, ResolutionStatus)> =
            rows.iter().map(|j| (j, ResolutionStatus::Active)).collect();
        let claims =
            resolve_claims_generic::<EstimatePayload>(&active_rows, DOMAIN_ESTIMATE, &0.65);
        assert_eq!(claims.anchored["SL-100"].tier, ClaimTier::Pin);
        assert!(claims.anchored["SL-100"].conflict.is_some());
        let finding = est_conflict_of(&claims, "SL-100");
        assert!(matches!(
            finding,
            ClaimFinding::Conflict {
                tier: ClaimTier::Pin,
                ..
            }
        ));
        assert!(finding.nominates_reprobe());
    }

    #[test]
    fn estimate_cross_session_concurrency() {
        let s1 = est_session(
            "s1",
            vec![est_anchor("h1", "SL-100", 2.0, 8.0, RaterKind::Human)],
        );
        let s2 = est_session(
            "s2",
            vec![est_anchor("h2", "SL-100", 4.0, 6.0, RaterKind::Human)],
        );
        let sessions = [s1, s2];
        let res = resolve(&sessions, &StatusMap::new()).expect("resolve ok");
        let claims = resolve_claims_generic::<EstimatePayload>(&res.rows, DOMAIN_ESTIMATE, &0.65);
        assert!((claims.anchored["SL-100"].operative - 5.6).abs() < 1e-12);
        assert!(claims.anchored["SL-100"].conflict.is_some());
        assert_eq!(claims.anchored["SL-100"].rows, 2);
    }

    #[test]
    fn estimate_lens_isolation_both_directions() {
        let mixed = [
            est_session(
                "s1",
                vec![
                    est_anchor("h1", "SL-100", 2.0, 8.0, RaterKind::Human),
                    est_lensed("l1", "SL-100", 1.0, 3.0, "user-value"),
                ],
            ),
            est_session(
                "s2",
                vec![est_lensed("l2", "SL-100", 4.0, 6.0, "user-value")],
            ),
        ];
        let res = resolve(&mixed, &StatusMap::new()).expect("resolve ok");
        let claims = resolve_claims_generic::<EstimatePayload>(&res.rows, DOMAIN_ESTIMATE, &0.65);
        assert!(!claims.lensed.is_empty());
        assert!(
            claims.lensed[&("user-value".into(), "SL-100".into())]
                .conflict
                .is_some()
        );

        let unlensed_only = [est_session(
            "s1",
            vec![est_anchor("h1", "SL-100", 2.0, 8.0, RaterKind::Human)],
        )];
        let res2 = resolve(&unlensed_only, &StatusMap::new()).expect("resolve ok");
        let without = resolve_claims_generic::<EstimatePayload>(&res2.rows, DOMAIN_ESTIMATE, &0.65);
        assert_eq!(claims.anchored, without.anchored);
    }

    #[test]
    fn estimate_anchor_map_no_laundering() {
        let rows = [
            est_anchor("h1", "SL-100", 2.0, 8.0, RaterKind::Human),
            est_anchor("a1", "SL-200", 1.0, 3.0, RaterKind::Agent),
            est_anchor("m1", "SL-300", 5.0, 5.0, RaterKind::Migrated),
        ];
        let active_rows: Vec<(&Judgement, ResolutionStatus)> =
            rows.iter().map(|j| (j, ResolutionStatus::Active)).collect();
        let claims =
            resolve_claims_generic::<EstimatePayload>(&active_rows, DOMAIN_ESTIMATE, &0.65);
        let map = claims.anchor_map();
        assert_eq!(map.len(), 1);
        assert!((map["SL-100"] - 5.9).abs() < 1e-12);
        assert!(!map.contains_key("SL-200"));
        assert!(!map.contains_key("SL-300"));
    }

    #[test]
    fn estimate_distinct_payloads_same_operative_conflict() {
        // (2,8) and (4,6) at skew 0.5 both → 5.0
        let rows = [
            est_anchor("h1", "SL-100", 2.0, 8.0, RaterKind::Human),
            est_anchor("h2", "SL-100", 4.0, 6.0, RaterKind::Human),
        ];
        let active_rows: Vec<(&Judgement, ResolutionStatus)> =
            rows.iter().map(|j| (j, ResolutionStatus::Active)).collect();
        let claims = resolve_claims_generic::<EstimatePayload>(&active_rows, DOMAIN_ESTIMATE, &0.5);
        let claim = &claims.anchored["SL-100"];
        let conflict = claim.conflict.as_ref().expect("conflict fires");
        assert_eq!(conflict.distinct, 2);
        assert!((conflict.low - 5.0).abs() < 1e-12);
        assert!((conflict.high - 5.0).abs() < 1e-12);
        assert!((claim.operative - 5.0).abs() < 1e-12);
    }

    #[test]
    fn estimate_no_compile_consumer_noop() {
        // Estimate rows with no payload (magnitude present, not est_lower)
        // should be excluded by extract — the fold handles it gracefully.
        let value_row = anchor("v1", "SL-100", 5.0, RaterKind::Human);
        let active_rows = vec![(&value_row, ResolutionStatus::Active)];
        let claims =
            resolve_claims_generic::<EstimatePayload>(&active_rows, DOMAIN_ESTIMATE, &0.65);
        assert!(claims.anchored.is_empty());
        assert!(claims.priors.is_empty());
        assert!(claims.findings.is_empty());
    }

    #[test]
    fn estimate_duplicate_posture_no_op() {
        let rows = [
            est_anchor("h1", "SL-100", 2.0, 8.0, RaterKind::Human),
            est_anchor("h2", "SL-100", 2.0, 8.0, RaterKind::Human),
        ];
        let active_rows: Vec<(&Judgement, ResolutionStatus)> =
            rows.iter().map(|j| (j, ResolutionStatus::Active)).collect();
        let claims =
            resolve_claims_generic::<EstimatePayload>(&active_rows, DOMAIN_ESTIMATE, &0.65);
        assert!(claims.anchored["SL-100"].conflict.is_none());
        assert_eq!(claims.anchored["SL-100"].rows, 2);
    }

    // -----------------------------------------------------------------------
    // E4 linearity property battery (design E4 / RV-282 F-1; PHASE-03 VT-2)
    // -----------------------------------------------------------------------

    /// E4 linearity lemma: the affine β-resolution distributes over the
    /// per-field mean — `affine(mean(payloads)) == mean(affine(payload))` to
    /// float tolerance, for range multisets away from the sub-EPSILON corner.
    #[test]
    fn linearity_affine_of_mean_equals_mean_of_affines() {
        let skew = 0.65;
        let payloads = [
            EstimatePayload(2.0, 8.0),
            EstimatePayload(1.0, 3.0),
            EstimatePayload(4.0, 4.0),
            EstimatePayload(0.5, 10.0),
        ];
        let affine_of_mean = EstimatePayload::mean(&payloads).operative(&skew);
        let mean_of_affines = payloads.iter().map(|p| p.operative(&skew)).sum::<f64>()
            / f64::from(u32::try_from(payloads.len()).unwrap());
        assert!(
            (affine_of_mean - mean_of_affines).abs() < 1e-9,
            "linearity: affine(mean)={affine_of_mean} vs mean(affines)={mean_of_affines}"
        );
    }

    /// E4 exact-composition rule: the EPSILON floor composes AFTER
    /// aggregation — `operative = floor_eps(affine(mean_payload))`, i.e. the
    /// cached operative of the resolved mean payload IS the resolved cost.
    #[test]
    fn linearity_floor_composes_after_aggregation() {
        let skew = 0.65;
        let rows = [
            est_anchor("h1", "SL-100", 2.0, 8.0, RaterKind::Human),
            est_anchor("h2", "SL-100", 4.0, 6.0, RaterKind::Human),
        ];
        let active_rows: Vec<(&Judgement, ResolutionStatus)> =
            rows.iter().map(|j| (j, ResolutionStatus::Active)).collect();
        let claims =
            resolve_claims_generic::<EstimatePayload>(&active_rows, DOMAIN_ESTIMATE, &skew);
        let claim = &claims.anchored["SL-100"];
        // mean payload = (3.0, 7.0); affine = 3 + 0.65·4 = 5.6; floor is a
        // no-op above EPSILON — the cached operative equals it exactly.
        assert_eq!(claim.operative, claim.payload.operative(&skew));
        assert!((claim.operative - 5.6).abs() < 1e-12);
    }

    /// E4 sub-EPSILON corner: an affine cost that dips to/below EPSILON
    /// floors deterministically — the invariant is determinism + positivity
    /// (every operative > 0), NOT mean/floor commutation, which is allowed to
    /// diverge below the floor.
    #[test]
    fn linearity_sub_epsilon_corner_floors_deterministically() {
        let skew = 0.0;
        let zero = EstimatePayload(0.0, 0.0);
        let once = zero.operative(&skew);
        assert!(once > 0.0, "positivity axiom: floored above zero");
        assert_eq!(once, zero.operative(&skew), "determinism");
        assert_eq!(once, crate::estimate::EPSILON);
        // Composition below the floor: floor(affine(mean)) still equals the
        // resolved operative by definition (the cached scalar is authoritative).
        let mixed = [EstimatePayload(0.0, 0.0), EstimatePayload(0.0, 0.0)];
        assert_eq!(
            EstimatePayload::mean(&mixed).operative(&skew),
            crate::estimate::EPSILON
        );
    }
}
