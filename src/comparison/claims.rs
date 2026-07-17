// SPDX-License-Identifier: GPL-3.0-only
//! `comparison::claims` — the ledgered-value-claim resolution pass (SL-220
//! design §2): a pure fold between tier-1 resolution and tier-2 compilation.
//! Pure leaf (ADR-001): depends only on the `wire` model, sibling result
//! types (`resolve::ResolutionStatus`, `compile::AnchorMap` — the `RowState`
//! precedent), and std `BTree` collections. No clock, disk, config reads.
//!
//! Input selection is this pass's OWN (RV-278 F-2): value-domain anchor-form
//! rows with resolution status `Active` **or `InertLens`** — R5's lens
//! inertness is a *constraint compilation* gate, not a claim-capture gate
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

/// One item's resolved claim within one partition (design §2).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ResolvedClaim {
    /// The singleton magnitude, or the D3 conflict mean.
    pub value: f64,
    pub tier: ClaimTier,
    /// Present ⇔ >1 distinct magnitude in the winning tier.
    pub conflict: Option<ClaimConflict>,
    /// Active winning-tier row count (render): corroboration when the
    /// magnitudes agree, the conflict multiset size when they don't.
    pub rows: u32,
    /// SL-220 PHASE-06 render attribution (design §6) — captured from the
    /// winning-tier row ONLY when that tier resolved to a single row. A
    /// corroborated or conflicted claim has no one author to name (multiple
    /// rows), so this is `None` there. Deterministic under row permutation:
    /// present iff `rows == 1`.
    pub attribution: Option<ClaimAttribution>,
}

/// Render attribution for a singleton claim (design §6): the `explain`/`show`
/// value line surfaces `by`/`date` (or `observed_at` for a migrated row) and,
/// for a pin, `basis`. Pure carry-through of the winning row's ledger fields —
/// never consulted by resolution or scoring.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ClaimAttribution {
    pub by: Option<String>,
    pub date: Option<String>,
    pub observed_at: Option<String>,
    pub basis: Option<String>,
}

/// The D3 conflict interval: rendered bounds + distinct-magnitude count.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ClaimConflict {
    pub low: f64,
    pub high: f64,
    pub distinct: u32,
}

/// A claim-pass finding (design §2), domain-tagged at construction (SL-219
/// D9). Conflicts fire at EVERY tier — surfaced-never-silent — but reprobe
/// nomination is anchored-tiers-only (D14, [`ClaimFinding::nominates_reprobe`]).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ClaimFinding {
    /// Same-tier disagreement on one item's magnitude ("contested pin" when
    /// `tier` is [`ClaimTier::Pin`]): the winning tier's full multiset mean
    /// stands as the point value; the interval and distinct count ride here.
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

/// The claim-resolution output (design §2): per-item resolved claims routed
/// by the D4 anti-laundering split, the inert lensed partitions (D5,
/// IDE-035 seam), and the finding stream.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct ClaimResolution {
    /// Pin/Human — the [`AnchorMap`] feed + graph rung 1.
    pub anchored: BTreeMap<String, ResolvedClaim>,
    /// Agent/Migrated — graph-ladder priors below projection; these bypass
    /// the constraint layer entirely (D4).
    pub priors: BTreeMap<String, ResolvedClaim>,
    /// (lens, item) — resolved identically per partition, consumed by no
    /// scoring surface (D5); rendered on demand.
    pub lensed: BTreeMap<(String, String), ResolvedClaim>,
    pub findings: Vec<ClaimFinding>,
}

impl ClaimResolution {
    /// Anti-laundering (D4): EXACTLY the `anchored` magnitudes — an agent or
    /// migrated claim can never enter `compile` through this seam. Since the
    /// PHASE-05 flip this IS the value system's compile anchor source (D12).
    pub(crate) fn anchor_map(&self) -> AnchorMap {
        self.anchored
            .iter()
            .map(|(item, claim)| (item.clone(), claim.value))
            .collect()
    }
}

/// One selected claim row's evidence: its tier, magnitude, and the render
/// attribution carried for a singleton resolution (design §6).
struct ClaimRow {
    tier: ClaimTier,
    magnitude: f64,
    attribution: ClaimAttribution,
}

/// Resolve the value-domain claim ladder over the post-resolve row set
/// (design §2). Pure fold; deterministic under row permutation (`BTreeMap`s
/// throughout; the conflict mean is summed in `total_cmp` order).
pub(crate) fn resolve_claims(rows: &[(&Judgement, ResolutionStatus)]) -> ClaimResolution {
    // Own input selection (RV-278 F-2): Active OR InertLens value anchors.
    let mut unlensed: BTreeMap<String, Vec<ClaimRow>> = BTreeMap::new();
    let mut lensed_groups: BTreeMap<(String, String), Vec<ClaimRow>> = BTreeMap::new();
    for (j, status) in rows {
        if j.domain != DOMAIN_VALUE
            || !matches!(j.form, RowForm::Anchor)
            || !matches!(
                status,
                ResolutionStatus::Active | ResolutionStatus::InertLens
            )
        {
            continue;
        }
        // Magnitude presence is a §1 capture invariant on value anchors; a
        // row without one carries no claim (panic-free, never unwrapped).
        let Some(magnitude) = j.magnitude else {
            continue;
        };
        let row = ClaimRow {
            tier: tier_of(j),
            magnitude,
            attribution: ClaimAttribution {
                by: j.by.clone(),
                date: j.date.clone(),
                observed_at: j.observed_at.clone(),
                basis: j.basis.clone(),
            },
        };
        match &j.lens {
            // Partition by lens (D5): the unlensed partition drives
            // everything; lensed partitions resolve identically, inertly.
            Some(lens) => lensed_groups
                .entry((lens.clone(), j.a.clone()))
                .or_default()
                .push(row),
            None => unlensed.entry(j.a.clone()).or_default().push(row),
        }
    }

    let mut out = ClaimResolution::default();
    for (item, group) in unlensed {
        let claim = resolve_group(&group);
        if let Some(conflict) = &claim.conflict {
            // Findings fire from the unlensed partition at EVERY tier
            // (surfaced-never-silent); a lensed conflict stays queryable on
            // its ResolvedClaim (the pinned finding shape carries no lens).
            out.findings.push(ClaimFinding::Conflict {
                domain: DOMAIN_VALUE.to_string(),
                item: item.clone(),
                tier: claim.tier,
                low: conflict.low,
                high: conflict.high,
                distinct: conflict.distinct,
                rows: claim.rows,
            });
        }
        // Route (D4): Pin/Human → anchored; Agent/Migrated → priors.
        if claim.tier.is_anchored() {
            out.anchored.insert(item, claim);
        } else {
            out.priors.insert(item, claim);
        }
    }
    for (key, group) in lensed_groups {
        out.lensed.insert(key, resolve_group(&group));
    }
    out
}

/// Tier one row (design §2 step 2): `admission = pin` → Pin, else by rater.
/// Total — pin ⇒ human is a §1 capture invariant, so no arm is ambiguous.
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
/// One distinct magnitude → that value with the full row count
/// (corroboration). Multiple → the arithmetic mean over the FULL winning-tier
/// row multiset (D3 — no dedupe: (5, 5, 7) → 5.666…), summed in `total_cmp`
/// order so the float fold is permutation-free, with `{min, max, distinct}`.
fn resolve_group(group: &[ClaimRow]) -> ResolvedClaim {
    let winning = group
        .iter()
        .map(|r| r.tier)
        .max()
        .unwrap_or(ClaimTier::Migrated); // unreachable: groups are non-empty
    let winning_rows: Vec<&ClaimRow> = group.iter().filter(|r| r.tier == winning).collect();
    // Attribution is nameable only for a SINGLETON winning-tier row —
    // permutation-invariant (present iff exactly one such row).
    let attribution = match winning_rows.as_slice() {
        [only] => Some(only.attribution.clone()),
        _ => None,
    };
    let mut magnitudes: Vec<f64> = winning_rows.iter().map(|r| r.magnitude).collect();
    magnitudes.sort_by(f64::total_cmp);

    let rows = u32::try_from(magnitudes.len()).unwrap_or(u32::MAX);
    let mut distinct = 0_u32;
    let mut prev: Option<f64> = None;
    for &m in &magnitudes {
        if prev.is_none_or(|p| p.total_cmp(&m).is_ne()) {
            distinct += 1;
        }
        prev = Some(m);
    }

    let (value, conflict) = match (magnitudes.first(), magnitudes.last()) {
        (Some(&low), Some(&high)) if distinct > 1 => {
            let mean = magnitudes.iter().sum::<f64>() / f64::from(rows);
            (
                mean,
                Some(ClaimConflict {
                    low,
                    high,
                    distinct,
                }),
            )
        }
        (Some(&only), _) => (only, None),
        _ => (0.0, None), // unreachable: groups are non-empty
    };
    ResolvedClaim {
        value,
        tier: winning,
        conflict,
        rows,
        attribution,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{ClaimFinding, ClaimResolution, ClaimTier, resolve_claims};
    use crate::comparison::resolve::{ResolutionStatus, StatusMap, resolve};
    use crate::comparison::{
        AdmissionKind, COMPARISON_SCHEMA, COMPARISON_VERSION, ComparisonSession,
        FRAME_VALUE_ANCHOR, Judgement, RaterKind, RowForm, SessionHeader,
    };

    // ---- fixtures -----------------------------------------------------------

    /// A value-domain anchor row (SL-220 §1 shape).
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

    /// Tag every judgement Active (the direct-feed harness; the resolve
    /// integration tests below go through `resolve` instead).
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

    // ---- the RV-275 F-1 gate battery -----------------------------------------

    /// Tier ordering pinned: ascending declaration order makes the derived
    /// `Ord` + `Iterator::max` correct by construction.
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

    /// Permutation invariance: any row order yields the SAME resolution —
    /// maps, findings, and the conflict mean bit-for-bit.
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

    /// Corroboration: N rows, one distinct magnitude ⇒ no conflict, value =
    /// the magnitude, `rows` = N.
    #[test]
    fn corroborating_rows_resolve_without_conflict() {
        let rows = [
            anchor("h1", "SL-100", 5.0, RaterKind::Human),
            anchor("h2", "SL-100", 5.0, RaterKind::Human),
            anchor("h3", "SL-100", 5.0, RaterKind::Human),
        ];
        let claims = resolve_claims(&active(&rows));
        let claim = &claims.anchored["SL-100"];
        assert_eq!(claim.value, 5.0);
        assert_eq!(claim.tier, ClaimTier::Human);
        assert_eq!(claim.conflict, None);
        assert_eq!(claim.rows, 3);
        assert!(claims.findings.is_empty());
    }

    /// Conflict: the mean is over the FULL winning-tier row multiset (D3 —
    /// no dedupe: (5, 5, 7) → 17/3), interval {min, max}, distinct count.
    #[test]
    fn conflict_takes_the_multiset_mean_with_interval_and_distinct() {
        let rows = [
            anchor("h1", "SL-100", 5.0, RaterKind::Human),
            anchor("h2", "SL-100", 5.0, RaterKind::Human),
            anchor("h3", "SL-100", 7.0, RaterKind::Human),
        ];
        let claims = resolve_claims(&active(&rows));
        let claim = &claims.anchored["SL-100"];
        assert_eq!(claim.value, 17.0 / 3.0, "multiset mean, not deduped");
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

    /// Conflicting pins ⇒ a contested-pin finding: tier = Pin, and it
    /// nominates for the human reprobe queue (D14).
    #[test]
    fn conflicting_pins_raise_a_contested_pin_finding() {
        let rows = [pin("p1", "SL-100", 3.0), pin("p2", "SL-100", 9.0)];
        let claims = resolve_claims(&active(&rows));
        let claim = &claims.anchored["SL-100"];
        assert_eq!(claim.tier, ClaimTier::Pin);
        assert_eq!(claim.value, 6.0);
        let finding = conflict_of(&claims, "SL-100");
        assert!(
            matches!(
                finding,
                ClaimFinding::Conflict {
                    tier: ClaimTier::Pin,
                    ..
                }
            ),
            "contested pin carries its tier: {finding:?}"
        );
        assert!(finding.nominates_reprobe());
    }

    /// D14 both directions: anchored-tier conflicts nominate for reprobe;
    /// agent/migrated conflicts fire as ordinary findings and NEVER nominate.
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
        assert_eq!(claims.findings.len(), 3, "conflicts fire at EVERY tier");
    }

    /// Winning tier wins outright: lower tiers contribute NOTHING for the
    /// item — not even bounds, and no cross-tier conflict.
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
        assert_eq!(claim.value, 6.0);
        assert_eq!(claim.conflict, None, "agent disagreement is not a bound");
        assert_eq!(claim.rows, 1, "winning-tier rows only");
        assert!(!claims.priors.contains_key("SL-100"), "one route per item");
        assert!(claims.findings.is_empty());
    }

    /// Routing (D4): Agent/Migrated claims land in `priors`, never anchored.
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

    /// Cross-session concurrency: two sessions, same subject/tier, differing
    /// magnitudes ⇒ a conflict — never latest-wins (through `resolve`).
    #[test]
    fn cross_session_same_tier_claims_conflict_never_latest_wins() {
        let s1 = session("s1", vec![anchor("p", "SL-100", 4.0, RaterKind::Human)]);
        let s2 = session("s2", vec![anchor("q", "SL-100", 8.0, RaterKind::Human)]);
        let sessions = [s1, s2];
        let res = resolve(&sessions, &StatusMap::new()).expect("resolve ok");
        let claims = resolve_claims(&res.rows);
        let claim = &claims.anchored["SL-100"];
        assert_eq!(claim.value, 6.0, "mean of both sessions' claims");
        assert!(claim.conflict.is_some(), "concurrency surfaces, no winner");
        assert_eq!(claim.rows, 2);
    }

    /// Duplicate-merge posture (D3/D10): an identical re-fire is harmless
    /// corroboration — no resolved-value change, no conflict.
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
        assert_eq!(one.anchored["SL-100"].value, two.anchored["SL-100"].value);
        assert_eq!(two.anchored["SL-100"].conflict, None);
        assert_eq!(two.anchored["SL-100"].rows, 2, "corroboration counted");
        assert!(two.findings.is_empty());
    }

    /// Lens isolation, NON-VACUOUS both directions (RV-278 F-2): with
    /// lens-tagged anchor rows present (InertLens through `resolve`),
    /// `lensed` is non-empty AND deleting every lensed row leaves
    /// `anchored`/`priors` byte-identical.
    #[test]
    fn lens_isolation_holds_non_vacuously_in_both_directions() {
        // The two `user-value` claims ride separate sessions so they stay
        // concurrent (R3 revises same-identity rows WITHIN a session — the
        // supersession machinery reduces lensed threads too).
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
        // The InertLens selection is exercised for real: lensed rows resolve.
        for uid in ["l1", "l2", "l3"] {
            let (_, status) = res
                .rows
                .iter()
                .find(|(j, _)| j.uid == uid)
                .expect("row present");
            assert_eq!(status, &ResolutionStatus::InertLens);
        }
        let with_lensed = resolve_claims(&res.rows);
        assert!(!with_lensed.lensed.is_empty(), "direction 1: non-empty");
        let key = ("user-value".to_string(), "SL-100".to_string());
        let lensed_claim = &with_lensed.lensed[&key];
        assert!(lensed_claim.conflict.is_some(), "resolved identically");
        assert_eq!(lensed_claim.value, 6.0);

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
        assert_eq!(
            with_lensed.anchored, without_lensed.anchored,
            "direction 2: lensed rows never touch the unlensed partitions"
        );
        assert_eq!(with_lensed.priors, without_lensed.priors);
    }

    /// Lensed conflicts stay on their ResolvedClaim, not in the finding
    /// stream — the pinned finding shape carries no lens discriminator, and
    /// the lensed output is inert (D5).
    #[test]
    fn lensed_conflicts_do_not_enter_the_finding_stream() {
        let rows = [
            lensed("l1", "SL-100", 1.0, "user-value"),
            lensed("l2", "SL-100", 9.0, "user-value"),
        ];
        let claims = resolve_claims(&active(&rows));
        let key = ("user-value".to_string(), "SL-100".to_string());
        assert!(claims.lensed[&key].conflict.is_some(), "queryable in place");
        assert!(claims.findings.is_empty());
    }

    /// Anti-laundering property (D4) over deterministically generated
    /// ledgers: every tier-presence combination on three items — the anchor
    /// map is EXACTLY the anchored magnitudes; agent/migrated tiers are
    /// absent from it by construction.
    #[test]
    fn anchor_map_never_launders_agent_or_migrated_claims() {
        // Tier-specific magnitudes so any laundering shows as a wrong value.
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
                .map(|(k, c)| (k.clone(), c.value))
                .collect();
            assert_eq!(map, anchored_values, "anchor_map ≡ anchored, mask {mask}");
            for claim in claims.anchored.values() {
                assert!(claim.tier.is_anchored(), "anchored is Pin/Human only");
            }
            for (item, claim) in &claims.priors {
                assert!(!claim.tier.is_anchored(), "priors are Agent/Migrated");
                assert!(!map.contains_key(item), "a prior never enters the map");
            }
        }
    }

    /// Superseded and tombstoned rows are excluded — only Active/InertLens
    /// rows carry claims.
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
        assert_eq!((claim.value, claim.rows), (5.0, 1));
        assert_eq!(claim.conflict, None, "the superseded row is gone");
        assert!(!claims.anchored.contains_key("SL-200"));
    }

    /// Order/ratio rows and other domains never enter the pass — the claims
    /// selection is anchor-form value-domain rows only.
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
}
