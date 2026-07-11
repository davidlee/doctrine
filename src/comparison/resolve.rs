// SPDX-License-Identifier: GPL-3.0-only
//! `comparison::resolve` — tier-1 row-validity resolution (SL-213 design §2,
//! rules R1–R6). Pure leaf: depends only on the `wire` model plus std `BTree`
//! collections. No clock, disk, rng, or git.
//!
//! Given the parsed sessions (any merge order) plus an entity status map, every
//! judgement row is tagged with exactly one [`ResolutionStatus`] under the
//! first-matching-rule discipline. Supersession is a single, order-free pass
//! (design R2: `supersedes` is a durable act, not testimony) — cycles among
//! supersession edges deactivate their participants (`Malformed`) and surface a
//! [`MalformedSupersession`] finding rather than looping to a fixpoint.
//!
//! PHASE-06 adds [`RowState`] — the join of this tier's [`ResolutionStatus`]
//! with tier-2's `compile::CompilationStatus` into the single `compare list`/
//! `explain` display token (design §1 D13). That one type's sole purpose pulls
//! in `compile`'s result type; every actual resolution rule above stays pure
//! over `wire` alone.

use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};

use super::compile::{CompilationStatus, QuarantineReason};
use super::{ComparisonSession, DOMAIN_PRIORITY, Judgement, RaterKind, RowForm};

/// The tier-1 validity verdict for one judgement row (design §2). Deliberately
/// distinct from compilation status (web review corr. 2): quarantine is a later
/// tier layered over an otherwise-active row. `Malformed` is the R2 extension —
/// a supersession-cycle participant, deactivated; the cycle detail rides a
/// separate [`MalformedSupersession`] finding, never the status variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolutionStatus {
    Active,
    Superseded { by: String },
    Tombstoned,
    InertLens,
    InertDomain,
    InertLifecycle,
    Malformed,
}

/// The design §1/D13 join of a row's [`ResolutionStatus`] with its (Active-only)
/// [`CompilationStatus`] verdict into ONE display token — PHASE-06's shared
/// consumer seam for `compare list`'s status column and `explain`'s citations.
/// Lives here (the phase sheet's A1 decision): `resolve` and `compile` are
/// sibling leaf tiers with no directional dependency between them (`compile`
/// takes plain `&[&Judgement]`, agnostic of how a caller resolved them), so a
/// `resolve → compile` type reference to compose their two outputs crosses no
/// ADR-001 layer boundary — it only names a sibling's result type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RowState {
    pub resolution: ResolutionStatus,
    /// `None` unless `resolution` is `Active` (design §1).
    pub compilation: Option<CompilationStatus>,
}

impl RowState {
    pub(crate) fn new(
        resolution: ResolutionStatus,
        compilation: Option<CompilationStatus>,
    ) -> Self {
        Self {
            resolution,
            compilation,
        }
    }

    /// The single display token (design §4 S2). The design's nine-token
    /// enumeration predates PHASE-02's `Malformed` variant (a supersession-
    /// cycle participant — R2's `MalformedSupersession` finding carries the
    /// cycle detail); this join appends a tenth, `malformed`, following the
    /// same lowercase-token convention rather than leaving it undisplayable.
    pub(crate) fn display_token(&self) -> String {
        match &self.resolution {
            ResolutionStatus::Superseded { by } => format!("superseded→{by}"),
            ResolutionStatus::Tombstoned => "tombstoned".to_string(),
            ResolutionStatus::InertLens => "inert(lens)".to_string(),
            ResolutionStatus::InertDomain => "inert(domain)".to_string(),
            ResolutionStatus::InertLifecycle => "inert(lifecycle)".to_string(),
            ResolutionStatus::Malformed => "malformed".to_string(),
            ResolutionStatus::Active => match &self.compilation {
                Some(CompilationStatus::NoConstraint) => "no-constraint".to_string(),
                Some(CompilationStatus::Quarantined(QuarantineReason::PreferenceCycle {
                    ..
                })) => "quarantined(cycle)".to_string(),
                Some(CompilationStatus::Quarantined(QuarantineReason::AnchorConflict {
                    ..
                })) => "quarantined(anchors)".to_string(),
                Some(CompilationStatus::Constraining) | None => "active".to_string(),
            },
        }
    }
}

/// A supersession cycle (R2): the participating row uids in deterministic
/// order. Emitted as finding data, kept out of [`ResolutionStatus`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MalformedSupersession {
    pub cycle: Vec<String>,
}

/// A live supersession edge whose target uid is not present in the loaded
/// corpus (R2 "T unknown uid" — the edge is ignored, the dangling reference is
/// reported). Uid-bearing so a surface can direct the fix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UnknownSupersedesTarget {
    pub row: String,
    pub target: String,
}

/// Minimal entity lifecycle for R6 — the only distinctions tier-1 acts on.
/// Populated in a later phase; an empty [`StatusMap`] makes R6 a no-op.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EntityLifecycle {
    /// done / rejected — inert for elicitation only; the row stays `Active` for
    /// inference (design R6).
    Terminal,
    /// The entity was replaced; its rows go `InertLifecycle`, and `by` carries
    /// the successor id for the later reprobe hint (no silent transfer).
    Superseded { by: String },
}

/// Entity id → its lifecycle. Defined here, populated by a later phase.
pub(crate) type StatusMap = BTreeMap<String, EntityLifecycle>;

/// The tier-1 output: every retained row tagged with its status in display
/// order `(date, session_uid, seq)`, plus the two structured finding streams.
/// Rows borrow the source judgements (the downstream `compile` consumes
/// `&[&Judgement]`), so no clone of the un-`Clone` wire model is needed.
#[derive(Debug, PartialEq)]
pub(crate) struct Resolution<'a> {
    pub rows: Vec<(&'a Judgement, ResolutionStatus)>,
    pub unknown_supersedes: Vec<UnknownSupersedesTarget>,
    pub malformed: Vec<MalformedSupersession>,
}

/// One retained row plus the session it is attributed to (the min-uid session
/// on a cherry-picked duplicate).
#[derive(Debug)]
struct RowMeta<'a> {
    judgement: &'a Judgement,
    session_uid: &'a str,
}

/// Resolve parsed sessions to tier-1 statuses (design §2). Deterministic across
/// any merge order of the session slice. `Err` only on a corruption guard: the
/// same uid carrying differing content across two files.
pub(crate) fn resolve<'a>(
    sessions: &'a [ComparisonSession],
    statuses: &StatusMap,
) -> anyhow::Result<Resolution<'a>> {
    // Order-free by construction: sort the sessions by uid before any work, so
    // duplicate collapse and every tiebreak are independent of input order.
    let mut ordered: Vec<&'a ComparisonSession> = sessions.iter().collect();
    ordered.sort_by(|a, b| a.session.uid.cmp(&b.session.uid));

    let rows = collect_rows(&ordered)?;
    let tombstoned = tombstoned_targets(&ordered);
    let (on_cycle, malformed) = supersession_cycles(&rows, &tombstoned);
    let (superseded_by, unknown_supersedes) = supersession_effects(&rows, &tombstoned, &on_cycle);
    let superseded_r3 = implicit_revisions(&rows, &tombstoned);

    let mut out: Vec<(&'a Judgement, &'a str, ResolutionStatus)> = Vec::new();
    for (&uid, meta) in &rows {
        let j = meta.judgement;
        // First matching rule wins (R1 → R6).
        let status = if tombstoned.contains(&uid) {
            ResolutionStatus::Tombstoned
        } else if on_cycle.contains(&uid) {
            ResolutionStatus::Malformed
        } else if let Some(&by) = superseded_by.get(&uid) {
            ResolutionStatus::Superseded { by: by.to_string() }
        } else if let Some(&by) = superseded_r3.get(&uid) {
            ResolutionStatus::Superseded { by: by.to_string() }
        } else if j.domain == DOMAIN_PRIORITY {
            ResolutionStatus::InertDomain
        } else if j.lens.is_some() {
            ResolutionStatus::InertLens
        } else if entity_superseded(j, statuses) {
            ResolutionStatus::InertLifecycle
        } else {
            ResolutionStatus::Active
        };
        out.push((j, meta.session_uid, status));
    }

    out.sort_by(|a, b| {
        a.0.date
            .cmp(&b.0.date)
            .then_with(|| a.1.cmp(b.1))
            .then_with(|| a.0.seq.cmp(&b.0.seq))
    });

    let ordered_rows = out.into_iter().map(|(j, _sid, st)| (j, st)).collect();
    Ok(Resolution {
        rows: ordered_rows,
        unknown_supersedes,
        malformed,
    })
}

/// Collect every judgement keyed by uid, collapsing byte-identical duplicates
/// (cherry-picks) and rejecting the same uid with differing content. Iterating
/// the uid-sorted sessions means the first (min-uid) session wins attribution.
fn collect_rows<'a>(
    ordered: &[&'a ComparisonSession],
) -> anyhow::Result<BTreeMap<&'a str, RowMeta<'a>>> {
    let mut by_uid: BTreeMap<&'a str, RowMeta<'a>> = BTreeMap::new();
    for sess in ordered {
        for j in &sess.judgements {
            match by_uid.entry(j.uid.as_str()) {
                Entry::Occupied(existing) => {
                    if existing.get().judgement != j {
                        anyhow::bail!(
                            "comparison row uid `{}` appears with differing content across \
                             session files — resolve the conflict at source",
                            j.uid
                        );
                    }
                }
                Entry::Vacant(slot) => {
                    slot.insert(RowMeta {
                        judgement: j,
                        session_uid: sess.session.uid.as_str(),
                    });
                }
            }
        }
    }
    Ok(by_uid)
}

/// Every uid targeted by a tombstone (R1). A target that names no loaded row is
/// simply inert.
fn tombstoned_targets<'a>(ordered: &[&'a ComparisonSession]) -> BTreeSet<&'a str> {
    let mut out = BTreeSet::new();
    for sess in ordered {
        for t in &sess.tombstones {
            out.insert(t.target.as_str());
        }
    }
    out
}

/// Detect supersession cycles (R2). The supersession relation is a functional
/// graph — each row names at most one target — so a trivial coloured walk over
/// the live edges (non-tombstoned holder, non-tombstoned known target) finds
/// every participant; no graph machinery imported. Returns the participant set
/// (each is `Malformed`) plus one finding per distinct cycle, uids sorted.
fn supersession_cycles<'a>(
    rows: &BTreeMap<&'a str, RowMeta<'a>>,
    tombstoned: &BTreeSet<&'a str>,
) -> (BTreeSet<&'a str>, Vec<MalformedSupersession>) {
    let mut edge: BTreeMap<&'a str, &'a str> = BTreeMap::new();
    for (&uid, meta) in rows {
        if tombstoned.contains(&uid) {
            continue; // holder tombstoned: edge disarmed
        }
        if let Some(target) = meta.judgement.supersedes.as_deref()
            && !tombstoned.contains(&target)
            && rows.contains_key(target)
        {
            edge.insert(uid, target);
        }
    }

    // Coloured walk: 0 unvisited, 1 on the current chain, 2 settled.
    let mut state: BTreeMap<&'a str, u8> = BTreeMap::new();
    let mut on_cycle: BTreeSet<&'a str> = BTreeSet::new();
    for &start in edge.keys() {
        if state.get(&start).copied().unwrap_or(0) != 0 {
            continue;
        }
        let mut chain: Vec<&'a str> = Vec::new();
        let mut cur = start;
        loop {
            match state.get(&cur).copied().unwrap_or(0) {
                0 => {
                    state.insert(cur, 1);
                    chain.push(cur);
                    match edge.get(&cur) {
                        Some(&next) => cur = next,
                        None => break,
                    }
                }
                1 => {
                    // `cur` is on the current chain: walk the loop back to it.
                    let mut c = cur;
                    loop {
                        on_cycle.insert(c);
                        match edge.get(&c) {
                            Some(&n) if n != cur => c = n,
                            _ => break,
                        }
                    }
                    break;
                }
                _ => break, // settled: any cycle already recorded
            }
        }
        for n in chain {
            state.insert(n, 2);
        }
    }

    let mut findings: Vec<MalformedSupersession> = Vec::new();
    let mut seen: BTreeSet<&'a str> = BTreeSet::new();
    for &node in &on_cycle {
        if seen.contains(&node) {
            continue;
        }
        let mut members: BTreeSet<String> = BTreeSet::new();
        let mut c = node;
        loop {
            if !seen.insert(c) {
                break;
            }
            members.insert(c.to_string());
            match edge.get(&c) {
                Some(&next) if on_cycle.contains(&next) => c = next,
                _ => break,
            }
        }
        findings.push(MalformedSupersession {
            cycle: members.into_iter().collect(),
        });
    }

    (on_cycle, findings)
}

/// Apply R2's durable-replacement effect and collect dangling-target warnings.
/// A row T is `Superseded { by: X }` iff some non-tombstoned, non-cycle row X
/// names it — X need not itself be active (a superseded holder's act stands).
/// Iterating the uid-sorted rows makes the min-uid holder win on contention.
fn supersession_effects<'a>(
    rows: &BTreeMap<&'a str, RowMeta<'a>>,
    tombstoned: &BTreeSet<&'a str>,
    on_cycle: &BTreeSet<&'a str>,
) -> (BTreeMap<&'a str, &'a str>, Vec<UnknownSupersedesTarget>) {
    let mut superseded_by: BTreeMap<&'a str, &'a str> = BTreeMap::new();
    let mut unknown: Vec<UnknownSupersedesTarget> = Vec::new();
    for (&uid, meta) in rows {
        if tombstoned.contains(&uid) {
            continue; // holder tombstoned: whole row withdrawn, act included
        }
        let Some(target) = meta.judgement.supersedes.as_deref() else {
            continue;
        };
        if !rows.contains_key(target) {
            unknown.push(UnknownSupersedesTarget {
                row: uid.to_string(),
                target: target.to_string(),
            });
            continue;
        }
        if on_cycle.contains(&uid) {
            continue; // holder is a cycle participant: Malformed, confers nothing
        }
        superseded_by.entry(target).or_insert(uid);
    }
    (superseded_by, unknown)
}

/// R3 implicit revision — within a single session file only, the highest-`seq`
/// row of a shared identity key wins; the losers are `Superseded { by: winner }`.
/// Cross-session same-key rows land in distinct groups (keyed by session uid)
/// and so stay concurrent. Tombstoned rows are withdrawn from the contest.
fn implicit_revisions<'a>(
    rows: &BTreeMap<&'a str, RowMeta<'a>>,
    tombstoned: &BTreeSet<&'a str>,
) -> BTreeMap<&'a str, &'a str> {
    let mut groups: BTreeMap<IdentityKey, Vec<(&'a str, u32)>> = BTreeMap::new();
    for (&uid, meta) in rows {
        if tombstoned.contains(&uid) {
            continue;
        }
        let key = identity_key(meta.session_uid, meta.judgement);
        groups
            .entry(key)
            .or_default()
            .push((uid, meta.judgement.seq));
    }

    let mut superseded: BTreeMap<&'a str, &'a str> = BTreeMap::new();
    for members in groups.values() {
        if members.len() < 2 {
            continue;
        }
        let Some(&(winner, _)) = members
            .iter()
            .max_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(b.0)))
        else {
            continue;
        };
        for &(uid, _) in members {
            if uid != winner {
                superseded.insert(uid, winner);
            }
        }
    }
    superseded
}

/// The R3 identity key. The pair is unordered — asking `a` vs `b` and later `b`
/// vs `a` is the same question — so the two entity refs are stored sorted.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct IdentityKey {
    session: String,
    pair_lo: String,
    pair_hi: String,
    domain: String,
    frame: String,
    form: &'static str,
    lens: Option<String>,
    rater: &'static str,
}

fn identity_key(session_uid: &str, j: &Judgement) -> IdentityKey {
    let (pair_lo, pair_hi) = if j.a <= j.b {
        (j.a.clone(), j.b.clone())
    } else {
        (j.b.clone(), j.a.clone())
    };
    IdentityKey {
        session: session_uid.to_string(),
        pair_lo,
        pair_hi,
        domain: j.domain.clone(),
        frame: j.frame.clone(),
        form: form_key(&j.form),
        lens: j.lens.clone(),
        rater: rater_key(&j.rater),
    }
}

/// Stable ordering tokens for the closed wire enums (they carry no `Ord`).
fn form_key(form: &RowForm) -> &'static str {
    match form {
        RowForm::Order => "order",
        RowForm::Ratio => "ratio",
    }
}

pub(crate) fn rater_key(rater: &RaterKind) -> &'static str {
    match rater {
        RaterKind::Human => "human",
        RaterKind::Agent => "agent",
    }
}

/// R6: a row is `InertLifecycle` when either of its entities is superseded.
/// Terminal entities keep their rows active (inert for elicitation only).
fn entity_superseded(j: &Judgement, statuses: &StatusMap) -> bool {
    matches!(statuses.get(&j.a), Some(EntityLifecycle::Superseded { .. }))
        || matches!(statuses.get(&j.b), Some(EntityLifecycle::Superseded { .. }))
}

#[cfg(test)]
mod tests {
    use super::{
        EntityLifecycle, MalformedSupersession, Resolution, ResolutionStatus, StatusMap, resolve,
    };
    use crate::comparison::{
        COMPARISON_SCHEMA, COMPARISON_VERSION, ComparisonSession, DOMAIN_PRIORITY, DOMAIN_VALUE,
        FRAME_EQUAL_EFFORT, FRAME_PREFER_FIRST, Judgement, RaterKind, Response, RowForm,
        SessionHeader, Tombstone,
    };

    // ---- fixtures -----------------------------------------------------------

    fn judgement(uid: &str, seq: u32, a: &str, b: &str) -> Judgement {
        Judgement {
            uid: uid.to_string(),
            seq,
            a: a.to_string(),
            b: b.to_string(),
            response: Response::PreferA,
            domain: DOMAIN_VALUE.to_string(),
            frame: FRAME_EQUAL_EFFORT.to_string(),
            form: RowForm::Order,
            magnitude: None,
            supersedes: None,
            lens: None,
            rater: RaterKind::Human,
            by: None,
            note: None,
            date: "2026-07-10".to_string(),
        }
    }

    fn session(
        uid: &str,
        judgements: Vec<Judgement>,
        tombstones: Vec<Tombstone>,
    ) -> ComparisonSession {
        ComparisonSession {
            schema: COMPARISON_SCHEMA.to_string(),
            version: COMPARISON_VERSION,
            session: SessionHeader {
                uid: uid.to_string(),
                date: "2026-07-10".to_string(),
                audience: None,
            },
            judgements,
            tombstones,
        }
    }

    fn tombstone(uid: &str, target: &str) -> Tombstone {
        Tombstone {
            uid: uid.to_string(),
            seq: 0,
            target: target.to_string(),
            date: "2026-07-10".to_string(),
            note: None,
        }
    }

    fn run(sessions: &[ComparisonSession]) -> Resolution<'_> {
        resolve(sessions, &StatusMap::new()).expect("resolve ok")
    }

    fn status_of<'a>(res: &'a Resolution<'_>, uid: &str) -> &'a ResolutionStatus {
        let row = res
            .rows
            .iter()
            .find(|(j, _)| j.uid.as_str() == uid)
            .expect("row present");
        &row.1
    }

    fn superseded_by(uid: &str) -> ResolutionStatus {
        ResolutionStatus::Superseded {
            by: uid.to_string(),
        }
    }

    // ---- R1 -----------------------------------------------------------------

    #[test]
    fn r1_tombstone_evicts_its_target() {
        let s = session(
            "s1",
            vec![judgement("j1", 0, "A", "B")],
            vec![tombstone("t1", "j1")],
        );
        let sessions = [s];
        let res = run(&sessions);
        assert_eq!(status_of(&res, "j1"), &ResolutionStatus::Tombstoned);
    }

    // ---- R2 state table -----------------------------------------------------

    #[test]
    fn r2_chain_leaves_only_the_head_active() {
        let mut b = judgement("b", 1, "R", "S");
        b.supersedes = Some("a".to_string());
        let mut c = judgement("c", 2, "T", "U");
        c.supersedes = Some("b".to_string());
        let s = session("s1", vec![judgement("a", 0, "P", "Q"), b, c], vec![]);
        let sessions = [s];
        let res = run(&sessions);
        assert_eq!(status_of(&res, "c"), &ResolutionStatus::Active);
        assert_eq!(status_of(&res, "b"), &superseded_by("c"));
        assert_eq!(status_of(&res, "a"), &superseded_by("b"));
    }

    #[test]
    fn r2_tombstoning_chain_head_revives_middle_but_not_root() {
        // A ← B ← C, then tombstone C: C's edge disarms so B revives, but B's
        // durable edge still suppresses A.
        let mut b = judgement("b", 1, "R", "S");
        b.supersedes = Some("a".to_string());
        let mut c = judgement("c", 2, "T", "U");
        c.supersedes = Some("b".to_string());
        let s = session(
            "s1",
            vec![judgement("a", 0, "P", "Q"), b, c],
            vec![tombstone("t1", "c")],
        );
        let sessions = [s];
        let res = run(&sessions);
        assert_eq!(status_of(&res, "c"), &ResolutionStatus::Tombstoned);
        assert_eq!(status_of(&res, "b"), &ResolutionStatus::Active);
        assert_eq!(status_of(&res, "a"), &superseded_by("b"));
    }

    #[test]
    fn r2_mutual_supersession_cycle_deactivates_all_participants() {
        let mut a = judgement("a", 0, "P", "Q");
        a.supersedes = Some("b".to_string());
        let mut b = judgement("b", 1, "R", "S");
        b.supersedes = Some("a".to_string());
        let s = session("s1", vec![a, b], vec![]);
        let sessions = [s];
        let res = run(&sessions);
        assert_eq!(status_of(&res, "a"), &ResolutionStatus::Malformed);
        assert_eq!(status_of(&res, "b"), &ResolutionStatus::Malformed);
        assert_eq!(
            res.malformed,
            vec![MalformedSupersession {
                cycle: vec!["a".to_string(), "b".to_string()],
            }]
        );
    }

    #[test]
    fn r2_self_supersession_is_a_degenerate_malformed_cycle() {
        let mut x = judgement("x", 0, "P", "Q");
        x.supersedes = Some("x".to_string());
        let sessions = [session("s1", vec![x], vec![])];
        let res = run(&sessions);
        assert_eq!(status_of(&res, "x"), &ResolutionStatus::Malformed);
        assert_eq!(res.malformed[0].cycle, vec!["x".to_string()]);
    }

    #[test]
    fn r2_tombstoned_superseder_revives_its_target() {
        let mut x = judgement("x", 1, "R", "S");
        x.supersedes = Some("t".to_string());
        let s = session(
            "s1",
            vec![judgement("t", 0, "P", "Q"), x],
            vec![tombstone("tomb", "x")],
        );
        let sessions = [s];
        let res = run(&sessions);
        assert_eq!(status_of(&res, "x"), &ResolutionStatus::Tombstoned);
        assert_eq!(status_of(&res, "t"), &ResolutionStatus::Active);
    }

    #[test]
    fn r2_superseder_of_a_tombstoned_target_stays_active() {
        let mut x = judgement("x", 1, "R", "S");
        x.supersedes = Some("t".to_string());
        let s = session(
            "s1",
            vec![judgement("t", 0, "P", "Q"), x],
            vec![tombstone("tomb", "t")],
        );
        let sessions = [s];
        let res = run(&sessions);
        assert_eq!(status_of(&res, "t"), &ResolutionStatus::Tombstoned);
        assert_eq!(status_of(&res, "x"), &ResolutionStatus::Active);
    }

    #[test]
    fn r2_unknown_supersedes_target_warns_and_holder_stays_active() {
        let mut x = judgement("x", 0, "P", "Q");
        x.supersedes = Some("ghost".to_string());
        let sessions = [session("s1", vec![x], vec![])];
        let res = run(&sessions);
        assert_eq!(status_of(&res, "x"), &ResolutionStatus::Active);
        assert_eq!(res.unknown_supersedes.len(), 1);
        assert_eq!(res.unknown_supersedes[0].row, "x");
        assert_eq!(res.unknown_supersedes[0].target, "ghost");
    }

    // ---- R3 identity revision & cross-session concurrency -------------------

    #[test]
    fn r3_within_file_identity_higher_seq_wins_unordered_pair() {
        // Same question re-asked with the pair flipped: one identity key.
        let sessions = [session(
            "s1",
            vec![judgement("p", 0, "X", "Y"), judgement("q", 1, "Y", "X")],
            vec![],
        )];
        let res = run(&sessions);
        assert_eq!(status_of(&res, "q"), &ResolutionStatus::Active);
        assert_eq!(status_of(&res, "p"), &superseded_by("q"));
    }

    #[test]
    fn r3_cross_session_same_key_is_concurrent_both_active() {
        let s1 = session("s1", vec![judgement("p", 0, "X", "Y")], vec![]);
        let s2 = session("s2", vec![judgement("q", 0, "X", "Y")], vec![]);
        let sessions = [s1, s2];
        let res = run(&sessions);
        assert_eq!(status_of(&res, "p"), &ResolutionStatus::Active);
        assert_eq!(status_of(&res, "q"), &ResolutionStatus::Active);
    }

    // ---- R4 / R5 / R6 -------------------------------------------------------

    #[test]
    fn r4_priority_domain_row_is_inert() {
        let mut j = judgement("j1", 0, "A", "B");
        j.domain = DOMAIN_PRIORITY.to_string();
        j.frame = FRAME_PREFER_FIRST.to_string();
        let sessions = [session("s1", vec![j], vec![])];
        let res = run(&sessions);
        assert_eq!(status_of(&res, "j1"), &ResolutionStatus::InertDomain);
    }

    #[test]
    fn r5_lens_tagged_row_is_inert() {
        let mut j = judgement("j1", 0, "A", "B");
        j.lens = Some("user-value".to_string());
        let sessions = [session("s1", vec![j], vec![])];
        let res = run(&sessions);
        assert_eq!(status_of(&res, "j1"), &ResolutionStatus::InertLens);
    }

    #[test]
    fn r6_superseded_entity_makes_its_rows_inert() {
        let life = EntityLifecycle::Superseded {
            by: "SL-999".to_string(),
        };
        // The successor id is retained for a later reprobe hint.
        if let EntityLifecycle::Superseded { by } = &life {
            assert_eq!(by, "SL-999");
        }
        let mut statuses = StatusMap::new();
        statuses.insert("SL-100".to_string(), life);
        let s = session("s1", vec![judgement("j1", 0, "SL-100", "SL-200")], vec![]);
        let sessions = [s];
        let res = resolve(&sessions, &statuses).expect("resolve ok");
        assert_eq!(status_of(&res, "j1"), &ResolutionStatus::InertLifecycle);
    }

    #[test]
    fn r6_terminal_entity_rows_stay_active() {
        let mut statuses = StatusMap::new();
        statuses.insert("SL-100".to_string(), EntityLifecycle::Terminal);
        let s = session("s1", vec![judgement("j1", 0, "SL-100", "SL-200")], vec![]);
        let sessions = [s];
        let res = resolve(&sessions, &statuses).expect("resolve ok");
        assert_eq!(status_of(&res, "j1"), &ResolutionStatus::Active);
    }

    #[test]
    fn r6_empty_status_map_is_a_no_op() {
        let sessions = [session("s1", vec![judgement("j1", 0, "A", "B")], vec![])];
        let res = run(&sessions);
        assert_eq!(status_of(&res, "j1"), &ResolutionStatus::Active);
    }

    // ---- cross-file mechanics ----------------------------------------------

    #[test]
    fn duplicate_uid_identical_content_collapses_to_one_row() {
        let s1 = session("s1", vec![judgement("dup", 0, "A", "B")], vec![]);
        let s2 = session("s2", vec![judgement("dup", 0, "A", "B")], vec![]);
        let sessions = [s1, s2];
        let res = run(&sessions);
        let count = res
            .rows
            .iter()
            .filter(|(j, _)| j.uid.as_str() == "dup")
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn same_uid_differing_content_is_a_load_error() {
        let s1 = session("s1", vec![judgement("x", 0, "A", "B")], vec![]);
        let mut conflicting = judgement("x", 0, "A", "B");
        conflicting.response = Response::PreferB;
        let s2 = session("s2", vec![conflicting], vec![]);
        let err = resolve(&[s1, s2], &StatusMap::new()).unwrap_err();
        assert!(err.to_string().contains("differing content"));
        assert!(err.to_string().contains("`x`"));
    }

    // ---- determinism --------------------------------------------------------

    fn det_fixture(name: &str) -> ComparisonSession {
        match name {
            "s1" => {
                let mut b = judgement("b", 0, "R", "S");
                b.supersedes = Some("a".to_string());
                session("s1", vec![b], vec![])
            }
            "s2" => session("s2", vec![judgement("a", 0, "P", "Q")], vec![]),
            _ => {
                let mut c = judgement("c", 0, "T", "U");
                c.supersedes = Some("d".to_string());
                let mut d = judgement("d", 1, "V", "W");
                d.supersedes = Some("c".to_string());
                session("s3", vec![c, d], vec![])
            }
        }
    }

    #[test]
    fn resolution_is_deterministic_across_session_merge_order() {
        let build = |order: [&str; 3]| -> Vec<ComparisonSession> {
            order.iter().map(|&n| det_fixture(n)).collect()
        };
        let p1 = build(["s1", "s2", "s3"]);
        let p2 = build(["s3", "s2", "s1"]);
        let p3 = build(["s2", "s3", "s1"]);
        let r1 = resolve(&p1, &StatusMap::new()).expect("resolve ok");
        let r2 = resolve(&p2, &StatusMap::new()).expect("resolve ok");
        let r3 = resolve(&p3, &StatusMap::new()).expect("resolve ok");
        assert_eq!(r1, r2);
        assert_eq!(r1, r3);
        // Sanity: the fixture exercises a cross-session edge and a cycle.
        assert_eq!(status_of(&r1, "a"), &superseded_by("b"));
        assert_eq!(status_of(&r1, "c"), &ResolutionStatus::Malformed);
    }
}
