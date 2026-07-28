// SPDX-License-Identifier: GPL-3.0-only
//! Pure active/history projection and deterministic per-control diagnostics
//! (SL-231 PHASE-01, design §4).
//!
//! No clock, RNG, disk, environment, terminal, or MCP imports.

use crate::observation::wire::{Diagnostic, Envelope, Payload};
use std::collections::{BTreeMap, BTreeSet};

// ── Resolution outcome ────────────────────────────────────────────────────

/// The resolved state of a single primary observation after applying
/// all controls in canonical order.
#[derive(Debug, Clone)]
pub(crate) enum ResolvedState {
    /// The observation is active and has not been corrected.
    Active(Envelope),
    /// The observation has been superseded by another.
    Superseded {
        /// The uid of the effective replacement.
        by: String,
        /// The original observation.
        original: Envelope,
        /// The supersession control that created this edge.
        control_uid: String,
    },
    /// The observation has been retracted.
    Retracted {
        /// The original observation.
        original: Envelope,
        /// The retraction control.
        control_uid: String,
    },
}

impl ResolvedState {
    /// Returns `true` when the record is currently active (not superseded or retracted).
    pub(crate) fn is_active(&self) -> bool {
        matches!(self, ResolvedState::Active(_))
    }

    /// Returns the original envelope regardless of state.
    pub(crate) fn original(&self) -> &Envelope {
        match self {
            ResolvedState::Active(e)
            | ResolvedState::Superseded { original: e, .. }
            | ResolvedState::Retracted { original: e, .. } => e,
        }
    }
}

/// The result of attempting to apply one control.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ControlOutcome {
    /// The control was applied successfully.
    Applied,
    /// The control was a no-op (duplicate of an already-applied equivalent control).
    Duplicate,
    /// The control was rejected and left a diagnostic.
    Inert {
        /// Why the control was rejected.
        reason: String,
    },
}

// ── Resolution engine ─────────────────────────────────────────────────────

/// The complete resolution of a corpus of observations.
#[derive(Debug, Clone)]
pub(crate) struct Resolution {
    /// Per-uid resolved state for every primary observation.
    states: BTreeMap<String, ResolvedState>,
    /// Diagnostics emitted during resolution (inert controls, etc.).
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "PHASE-04 / unreachable by design")
    )]
    diagnostics: Vec<Diagnostic>,
    /// All envelopes (primaries and controls) for history access.
    all: BTreeMap<String, Envelope>,
    /// The canonical ordering key for each uid: `(recorded_at, uid)`.
    #[expect(dead_code, reason = "populated in resolve(), read only in tests")]
    ordering: BTreeMap<String, (String, String)>,
}

impl Resolution {
    /// Returns the resolved state for a given uid, if it is a primary.
    pub(crate) fn state(&self, uid: &str) -> Option<&ResolvedState> {
        self.states.get(uid)
    }

    /// Returns all diagnostics collected during resolution.
    #[cfg_attr(not(test), expect(dead_code, reason = "PHASE-04"))]
    pub(crate) fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns all envelopes (history view).
    pub(crate) fn all_envelopes(&self) -> impl Iterator<Item = &Envelope> {
        self.all.values()
    }

    /// Returns the active (non-corrected) primaries, sorted by canonical order desc.
    pub(crate) fn active(&self) -> Vec<&Envelope> {
        let mut active: Vec<&Envelope> = self
            .states
            .values()
            .filter(|s| s.is_active())
            .map(ResolvedState::original)
            .collect();
        active.sort_by(|a, b| {
            canonical_cmp_key(a, b)
                .reverse()
                .then_with(|| a.uid.cmp(&b.uid).reverse())
        });
        active
    }

    /// Returns all primary envelopes (including superseded/retracted), sorted
    /// by canonical order desc. This is the "history" projection.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "superseded by all_envelopes() in query path; retained for clarity"
        )
    )]
    pub(crate) fn history(&self) -> Vec<&Envelope> {
        let mut all: Vec<&Envelope> = self.states.values().map(ResolvedState::original).collect();
        all.sort_by(|a, b| {
            canonical_cmp_key(a, b)
                .reverse()
                .then_with(|| a.uid.cmp(&b.uid).reverse())
        });
        all
    }

    /// Returns the envelope for any uid (primary or control), if present.
    #[expect(dead_code, reason = "PHASE-04")]
    pub(crate) fn envelope(&self, uid: &str) -> Option<&Envelope> {
        self.all.get(uid)
    }

    /// Follow the supersession chain from `uid` to its terminus, returning
    /// the terminus uid (as a copy). If `uid` is retracted, it is the terminus.
    #[cfg_attr(not(test), expect(dead_code, reason = "PHASE-04"))]
    pub(crate) fn resolved_terminus(&self, uid: &str) -> String {
        let mut current: String = uid.to_string();
        for _ in 0..1_000 {
            match self.states.get(&current) {
                Some(ResolvedState::Superseded { by, .. }) => {
                    if *by == current {
                        break;
                    }
                    current = by.clone();
                }
                _ => break,
            }
        }
        current
    }

    /// Returns all control envelopes targeting a given uid, in canonical order.
    #[expect(dead_code, reason = "PHASE-04")]
    pub(crate) fn controls_targeting(&self, uid: &str) -> Vec<&Envelope> {
        let mut controls: Vec<&Envelope> = self
            .all
            .values()
            .filter(|e| match &e.payload {
                Payload::Supersession { old_uid, .. } => old_uid == uid,
                Payload::Retraction { target_uid, .. } => target_uid == uid,
                _ => false,
            })
            .collect();
        controls.sort_by(|a, b| canonical_cmp(a, b));
        controls
    }

    /// Returns the correction chain from a uid's original through each
    /// supersession/replacement to the terminus, including the terminus.
    /// Each step is (uid, envelope).
    pub(crate) fn correction_chain<'a>(&'a self, uid: &'a str) -> Vec<(&'a str, &'a Envelope)> {
        let mut chain: Vec<(&str, &Envelope)> = Vec::new();
        let mut current = uid;
        for _ in 0..1_000 {
            let Some(envelope) = self.all.get(current) else {
                break;
            };
            chain.push((current, envelope));
            match self.states.get(current) {
                Some(ResolvedState::Superseded { by, .. }) => {
                    if by == current {
                        break;
                    }
                    current = by;
                }
                _ => break,
            }
        }
        chain
    }
}

// ── Canonical ordering ────────────────────────────────────────────────────

/// The canonical comparison key for an envelope: `(recorded_at, uid)`.
fn canonical_cmp_key(a: &Envelope, b: &Envelope) -> std::cmp::Ordering {
    a.recorded_at
        .cmp(&b.recorded_at)
        .then_with(|| a.uid.cmp(&b.uid))
}

/// Canonical ordering: by `recorded_at` ascending, then `uid` ascending.
fn canonical_cmp(a: &Envelope, b: &Envelope) -> std::cmp::Ordering {
    canonical_cmp_key(a, b)
}

/// Sort a slice of envelopes in canonical order.
fn sort_canonical(envelopes: &mut [Envelope]) {
    envelopes.sort_by(canonical_cmp);
}

// ── Resolution ────────────────────────────────────────────────────────────

/// Resolve a corpus of observations into a [`Resolution`].
///
/// Controls are applied in canonical `(recorded_at, uid)` order.
/// - Supersession: the old uid must be a primary, and the replacement
///   must exist as a kind-compatible primary.
/// - Retraction: the target must be a primary.
/// - Controls cannot target controls.
/// - Duplicate controls (same kind, same target, same effective payload)
///   are idempotent and produce [`ControlOutcome::Duplicate`].
/// - Retraction dominates supersession for the same target.
/// - Among distinct successors, the earliest valid supersession is effective;
///   later alternatives produce inert diagnostics.
/// - A cycle-introducing supersession edge is inert without cancelling
///   earlier valid edges.
///
/// Returns `(Resolution, Vec<(String, ControlOutcome)>)` where the second
/// element maps each control uid to its outcome.
pub(crate) fn resolve(envelopes: Vec<Envelope>) -> (Resolution, BTreeMap<String, ControlOutcome>) {
    let mut envelopes = envelopes;
    sort_canonical(&mut envelopes);

    // Partition into primaries and controls, index by uid
    let mut primaries: BTreeMap<String, Envelope> = BTreeMap::new();
    let mut controls: Vec<Envelope> = Vec::new();
    let mut all: BTreeMap<String, Envelope> = BTreeMap::new();
    let mut ordering: BTreeMap<String, (String, String)> = BTreeMap::new();

    for e in envelopes {
        ordering.insert(e.uid.clone(), (e.recorded_at.clone(), e.uid.clone()));
        if e.is_primary() {
            primaries.insert(e.uid.clone(), e.clone());
        } else {
            controls.push(e.clone());
        }
        all.insert(e.uid.clone(), e);
    }

    // Track active supersession edges: old -> (replacement, control_uid)
    let mut supersessions: BTreeMap<String, (String, String)> = BTreeMap::new();
    // Track retractions: uid -> control_uid
    let mut retractions: BTreeMap<String, String> = BTreeMap::new();
    // Track applied controls for duplicate detection
    let mut applied_supersessions: BTreeSet<(String, String)> = BTreeSet::new(); // (old, replacement)
    let mut applied_retractions: BTreeSet<String> = BTreeSet::new(); // target_uid

    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let mut outcomes: BTreeMap<String, ControlOutcome> = BTreeMap::new();

    // Apply controls in canonical order (already sorted)
    for c in &controls {
        let outcome = match &c.payload {
            Payload::Supersession {
                old_uid,
                replacement_uid,
                ..
            } => apply_supersession(
                old_uid,
                replacement_uid,
                &c.uid,
                &primaries,
                &supersessions,
                &retractions,
                &applied_supersessions,
                &mut diagnostics,
            ),
            Payload::Retraction { target_uid, .. } => apply_retraction(
                target_uid,
                &c.uid,
                &primaries,
                &supersessions,
                &retractions,
                &applied_retractions,
                &mut diagnostics,
            ),
            _ => ControlOutcome::Inert {
                reason: "control payload does not match control kind".to_string(),
            },
        };

        match &outcome {
            ControlOutcome::Applied => match &c.payload {
                Payload::Supersession {
                    old_uid,
                    replacement_uid,
                    ..
                } => {
                    supersessions.insert(old_uid.clone(), (replacement_uid.clone(), c.uid.clone()));
                    applied_supersessions.insert((old_uid.clone(), replacement_uid.clone()));
                }
                Payload::Retraction { target_uid, .. } => {
                    retractions.insert(target_uid.clone(), c.uid.clone());
                    applied_retractions.insert(target_uid.clone());
                }
                _ => {}
            },
            ControlOutcome::Duplicate => {
                // Still mark as applied for dedup
                match &c.payload {
                    Payload::Supersession {
                        old_uid,
                        replacement_uid,
                        ..
                    } => {
                        applied_supersessions.insert((old_uid.clone(), replacement_uid.clone()));
                    }
                    Payload::Retraction { target_uid, .. } => {
                        applied_retractions.insert(target_uid.clone());
                    }
                    _ => {}
                }
            }
            ControlOutcome::Inert { .. } => {}
        }
        outcomes.insert(c.uid.clone(), outcome);
    }

    // Build resolved states for each primary
    let mut states: BTreeMap<String, ResolvedState> = BTreeMap::new();
    for (uid, env) in &primaries {
        let state = if let Some(control_uid) = retractions.get(uid) {
            ResolvedState::Retracted {
                original: env.clone(),
                control_uid: control_uid.clone(),
            }
        } else if let Some((by, control_uid)) = supersessions.get(uid) {
            ResolvedState::Superseded {
                by: by.clone(),
                original: env.clone(),
                control_uid: control_uid.clone(),
            }
        } else {
            ResolvedState::Active(env.clone())
        };
        states.insert(uid.clone(), state);
    }

    (
        Resolution {
            states,
            diagnostics,
            all,
            ordering,
        },
        outcomes,
    )
}

/// Attempt to apply a supersession control.
///
/// Returns [`ControlOutcome::Applied`] if the supersession is the first
/// valid one for this target. Returns [`ControlOutcome::Duplicate`] if
/// an equivalent supersession was already applied. Returns
/// [`ControlOutcome::Inert`] with a diagnostic otherwise.
#[expect(
    clippy::too_many_arguments,
    reason = "resolution context requires all state maps"
)]
fn apply_supersession(
    old_uid: &str,
    replacement_uid: &str,
    control_uid: &str,
    primaries: &BTreeMap<String, Envelope>,
    supersessions: &BTreeMap<String, (String, String)>,
    retractions: &BTreeMap<String, String>,
    applied_supersessions: &BTreeSet<(String, String)>,
    diagnostics: &mut Vec<Diagnostic>,
) -> ControlOutcome {
    // Duplicate check first
    if applied_supersessions.contains(&(old_uid.to_string(), replacement_uid.to_string())) {
        return ControlOutcome::Duplicate;
    }

    // old_uid must exist as a primary
    let Some(old_primary) = primaries.get(old_uid) else {
        let reason =
            format!("supersession target {old_uid} does not exist as a primary observation");
        diagnostics.push(Diagnostic::new(control_uid.to_string(), reason.clone()));
        return ControlOutcome::Inert { reason };
    };

    // replacement_uid must exist as a primary
    let Some(replacement_primary) = primaries.get(replacement_uid) else {
        let reason = format!(
            "supersession replacement {replacement_uid} does not exist as a primary observation"
        );
        diagnostics.push(Diagnostic::new(control_uid.to_string(), reason.clone()));
        return ControlOutcome::Inert { reason };
    };

    // Kind compatibility: replacement must be the same kind as old
    if old_primary.kind() != replacement_primary.kind() {
        let reason = format!(
            "supersession replacement kind {:?} is incompatible with target kind {:?}",
            replacement_primary.kind(),
            old_primary.kind()
        );
        diagnostics.push(Diagnostic::new(control_uid.to_string(), reason.clone()));
        return ControlOutcome::Inert { reason };
    }

    // old_uid must not already be superseded
    if supersessions.contains_key(old_uid) {
        let reason = format!("supersession target {old_uid} already has an effective supersession");
        diagnostics.push(Diagnostic::new(control_uid.to_string(), reason.clone()));
        return ControlOutcome::Inert { reason };
    }

    // old_uid must not be retracted (retraction dominates)
    if retractions.contains_key(old_uid) {
        let reason =
            format!("supersession target {old_uid} is already retracted (retraction dominates)");
        diagnostics.push(Diagnostic::new(control_uid.to_string(), reason.clone()));
        return ControlOutcome::Inert { reason };
    }

    // Cycle detection: traversing supersession forward from replacement
    // must not reach old_uid.
    if would_create_cycle(old_uid, replacement_uid, supersessions) {
        let reason = format!("supersession {old_uid} -> {replacement_uid} would create a cycle");
        diagnostics.push(Diagnostic::new(control_uid.to_string(), reason.clone()));
        return ControlOutcome::Inert { reason };
    }

    ControlOutcome::Applied
}

/// Attempt to apply a retraction control.
fn apply_retraction(
    target_uid: &str,
    control_uid: &str,
    primaries: &BTreeMap<String, Envelope>,
    _supersessions: &BTreeMap<String, (String, String)>,
    _retractions: &BTreeMap<String, String>,
    applied_retractions: &BTreeSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) -> ControlOutcome {
    // Duplicate check first
    if applied_retractions.contains(target_uid) {
        return ControlOutcome::Duplicate;
    }

    // target must exist as a primary
    match primaries.get(target_uid) {
        None => {
            let reason =
                format!("retraction target {target_uid} does not exist as a primary observation");
            diagnostics.push(Diagnostic::new(control_uid.to_string(), reason.clone()));
            ControlOutcome::Inert { reason }
        }
        Some(_target) => {
            // Retraction dominates supersession: if there's already a
            // supersession for this target, we still apply retraction.
            // The supersession effectively becomes irrelevant because
            // the target is retracted. We leave the supersession in the
            // map (it already fired and was valid at the time), but the
            // resolved state will show Retracted because retraction is
            // checked first in state construction.

            // No cycle check needed for retraction.

            // Also reject if target is already superseded — retraction
            // of a superseded target is still valid: it retracts the
            // original; the resolution state construction checks retraction
            // before supersession.

            // If already retracted, duplicate caught above.
            ControlOutcome::Applied
        }
    }
}

/// Check whether adding `old -> replacement` to `supersessions` would create
/// a cycle: follow the chain forward from `replacement` and see if it reaches `old`.
fn would_create_cycle(
    old: &str,
    replacement: &str,
    supersessions: &BTreeMap<String, (String, String)>,
) -> bool {
    let mut current = replacement;
    for _ in 0..1_000 {
        if current == old {
            return true;
        }
        match supersessions.get(current) {
            Some((next, _)) => {
                current = next;
            }
            None => return false,
        }
    }
    // Safety valve: if we exceed the iteration cap, assume a cycle
    true
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test code is exempt from panic-family lints"
)]
mod tests {
    use super::*;
    use crate::observation::wire::{Payload, SCHEMA, SCHEMA_VERSION};

    fn friction_env(uid: &str, recorded_at: &str, summary: &str) -> Envelope {
        Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: uid.to_string(),
            recorded_at: recorded_at.to_string(),
            facets: None,
            payload: Payload::Friction {
                summary: summary.to_string(),
                detail: None,
            },
        }
    }

    fn ss_env(uid: &str, recorded_at: &str, old: &str, replacement: &str) -> Envelope {
        Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: uid.to_string(),
            recorded_at: recorded_at.to_string(),
            facets: None,
            payload: Payload::Supersession {
                old_uid: old.to_string(),
                replacement_uid: replacement.to_string(),
                reason: None,
            },
        }
    }

    fn ret_env(uid: &str, recorded_at: &str, target: &str) -> Envelope {
        Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: uid.to_string(),
            recorded_at: recorded_at.to_string(),
            facets: None,
            payload: Payload::Retraction {
                target_uid: target.to_string(),
                reason: None,
            },
        }
    }

    // ── Basic resolution ──────────────────────────────────────────────

    #[test]
    fn single_primary_is_active() {
        let e = friction_env(
            "01909a0a-0000-7000-8000-000000000001",
            "2026-07-26T10:11:12Z",
            "test",
        );
        let (res, _outcomes) = resolve(vec![e.clone()]);
        let state = res.state(&e.uid).unwrap();
        assert!(state.is_active());
        assert_eq!(res.diagnostics().len(), 0);
    }

    // ── Supersession chain ────────────────────────────────────────────

    #[test]
    fn supersession_chain_resolves_to_terminus() {
        let a = friction_env("a", "2026-01-01T00:00:00Z", "original");
        let b = friction_env("b", "2026-01-02T00:00:00Z", "replacement");
        let c = friction_env("c", "2026-01-03T00:00:00Z", "replacement-2");
        let ss1 = ss_env("ss1", "2026-01-04T00:00:00Z", "a", "b");
        let ss2 = ss_env("ss2", "2026-01-05T00:00:00Z", "b", "c");

        let (res, outcomes) = resolve(vec![a, b, c, ss1, ss2]);

        assert_eq!(res.diagnostics().len(), 0);
        assert_eq!(outcomes.get("ss1").unwrap(), &ControlOutcome::Applied);
        assert_eq!(outcomes.get("ss2").unwrap(), &ControlOutcome::Applied);

        // a is superseded by b
        match res.state("a").unwrap() {
            ResolvedState::Superseded { by, .. } => assert_eq!(by, "b"),
            other => panic!("expected Superseded, got {other:?}"),
        }
        // b is superseded by c
        match res.state("b").unwrap() {
            ResolvedState::Superseded { by, .. } => assert_eq!(by, "c"),
            other => panic!("expected Superseded, got {other:?}"),
        }
        // c is active
        assert!(res.state("c").unwrap().is_active());

        // Resolved terminus: a → b → c
        assert_eq!(res.resolved_terminus("a"), "c");
    }

    // ── Retraction ────────────────────────────────────────────────────

    #[test]
    fn retraction_makes_primary_inactive() {
        let a = friction_env("a", "2026-01-01T00:00:00Z", "original");
        let ret = ret_env("ret1", "2026-01-02T00:00:00Z", "a");

        let (res, outcomes) = resolve(vec![a, ret]);

        assert_eq!(res.diagnostics().len(), 0);
        assert_eq!(outcomes.get("ret1").unwrap(), &ControlOutcome::Applied);
        assert!(matches!(
            res.state("a").unwrap(),
            ResolvedState::Retracted { .. }
        ));
    }

    // ── Invalid control cannot resurrect ──────────────────────────────

    #[test]
    fn invalid_control_cannot_resurrect() {
        // a is retracted, then someone tries to supersede a → b.
        // The supersession must be inert because retraction dominates.
        let a = friction_env("a", "2026-01-01T00:00:00Z", "original");
        let b = friction_env("b", "2026-01-02T00:00:00Z", "replacement");
        let ret = ret_env("ret1", "2026-01-03T00:00:00Z", "a");
        let ss = ss_env("ss1", "2026-01-04T00:00:00Z", "a", "b");

        let (res, outcomes) = resolve(vec![a, b, ret, ss]);

        // retraction applies
        assert_eq!(outcomes.get("ret1").unwrap(), &ControlOutcome::Applied);
        // supersession is inert (retraction dominates)
        assert!(matches!(
            outcomes.get("ss1").unwrap(),
            ControlOutcome::Inert { .. }
        ));
        // a is still retracted
        assert!(matches!(
            res.state("a").unwrap(),
            ResolvedState::Retracted { .. }
        ));
        // a's terminus is itself (retracted)
        assert_eq!(res.resolved_terminus("a"), "a");
        // diagnostics should contain the inert reason
        assert!(!res.diagnostics().is_empty());
    }

    // ── Duplicate controls are idempotent ─────────────────────────────

    #[test]
    fn duplicate_controls_are_idempotent() {
        let a = friction_env("a", "2026-01-01T00:00:00Z", "original");
        let b = friction_env("b", "2026-01-02T00:00:00Z", "replacement");
        let ss1 = ss_env("ss1", "2026-01-03T00:00:00Z", "a", "b");
        let ss2 = ss_env("ss2", "2026-01-04T00:00:00Z", "a", "b"); // same (old, replacement)

        let (res, outcomes) = resolve(vec![a, b, ss1, ss2]);

        assert_eq!(outcomes.get("ss1").unwrap(), &ControlOutcome::Applied);
        assert_eq!(outcomes.get("ss2").unwrap(), &ControlOutcome::Duplicate);

        // a is superseded by b (only once)
        match res.state("a").unwrap() {
            ResolvedState::Superseded { by, .. } => assert_eq!(by, "b"),
            other => panic!("expected Superseded, got {other:?}"),
        }

        // Duplicate retraction
        let x = friction_env("x", "2026-01-01T00:00:00Z", "x");
        let r1 = ret_env("r1", "2026-01-02T00:00:00Z", "x");
        let r2 = ret_env("r2", "2026-01-03T00:00:00Z", "x");
        let (res2, outcomes2) = resolve(vec![x, r1, r2]);
        assert_eq!(outcomes2.get("r1").unwrap(), &ControlOutcome::Applied);
        assert_eq!(outcomes2.get("r2").unwrap(), &ControlOutcome::Duplicate);
        assert!(matches!(
            res2.state("x").unwrap(),
            ResolvedState::Retracted { .. }
        ));
    }

    // ── Resolved chain reports retracted terminus ─────────────────────

    #[test]
    fn resolved_chain_reports_retracted_terminus() {
        // a → b → c, then c is retracted
        let a = friction_env("a", "2026-01-01T00:00:00Z", "a");
        let b = friction_env("b", "2026-01-02T00:00:00Z", "b");
        let c = friction_env("c", "2026-01-03T00:00:00Z", "c");
        let ss1 = ss_env("ss1", "2026-01-04T00:00:00Z", "a", "b");
        let ss2 = ss_env("ss2", "2026-01-05T00:00:00Z", "b", "c");
        let ret = ret_env("ret1", "2026-01-06T00:00:00Z", "c");

        let (res, _outcomes) = resolve(vec![a, b, c, ss1, ss2, ret]);

        // Chain: a → b → c (retracted)
        assert_eq!(res.resolved_terminus("a"), "c");
        // c is retracted — the terminus reports its retracted state
        assert!(matches!(
            res.state("c").unwrap(),
            ResolvedState::Retracted { .. }
        ));
        // a is still superseded by b
        match res.state("a").unwrap() {
            ResolvedState::Superseded { by, .. } => assert_eq!(by, "b"),
            other => panic!("expected Superseded, got {other:?}"),
        }
    }

    // ── Cycle edge is inert ───────────────────────────────────────────

    #[test]
    fn cycle_introducing_edge_is_inert() {
        let a = friction_env("a", "2026-01-01T00:00:00Z", "a");
        let b = friction_env("b", "2026-01-02T00:00:00Z", "b");
        let ss1 = ss_env("ss1", "2026-01-03T00:00:00Z", "a", "b");
        let ss2 = ss_env("ss2", "2026-01-04T00:00:00Z", "b", "a"); // would create cycle

        let (res, outcomes) = resolve(vec![a, b, ss1, ss2]);

        assert_eq!(outcomes.get("ss1").unwrap(), &ControlOutcome::Applied);
        assert!(matches!(
            outcomes.get("ss2").unwrap(),
            ControlOutcome::Inert { .. }
        ));
        // ss1 still effective
        match res.state("a").unwrap() {
            ResolvedState::Superseded { by, .. } => assert_eq!(by, "b"),
            other => panic!("expected Superseded, got {other:?}"),
        }
        assert!(res.state("b").unwrap().is_active());
    }

    // ── Dangling control is inert ─────────────────────────────────────

    #[test]
    fn dangling_supersession_is_inert() {
        let ss = ss_env(
            "ss1",
            "2026-01-01T00:00:00Z",
            "nonexistent",
            "01909a0a-0000-7000-8000-000000000099",
        );
        let (res, outcomes) = resolve(vec![ss]);
        assert!(matches!(
            outcomes.get("ss1").unwrap(),
            ControlOutcome::Inert { .. }
        ));
        assert!(!res.diagnostics().is_empty());
    }

    #[test]
    fn dangling_retraction_is_inert() {
        let ret = ret_env("ret1", "2026-01-01T00:00:00Z", "nonexistent");
        let (_res, outcomes) = resolve(vec![ret]);
        assert!(matches!(
            outcomes.get("ret1").unwrap(),
            ControlOutcome::Inert { .. }
        ));
    }

    // ── Kind-incompatible supersession is inert ───────────────────────

    #[test]
    fn kind_incompatible_supersession_is_inert() {
        let a = friction_env("a", "2026-01-01T00:00:00Z", "friction");
        let m = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: "m".to_string(),
            recorded_at: "2026-01-02T00:00:00Z".to_string(),
            facets: None,
            payload: Payload::Measurement {
                source: "test".to_string(),
                counters: BTreeMap::new(),
                gauges: BTreeMap::new(),
                scope: None,
                units: None,
                completeness: None,
            },
        };
        // Try to supersede friction with measurement — incompatible
        let ss = ss_env("ss1", "2026-01-03T00:00:00Z", "a", "m");
        let (res, outcomes) = resolve(vec![a, m, ss]);
        assert!(matches!(
            outcomes.get("ss1").unwrap(),
            ControlOutcome::Inert { .. }
        ));
        assert!(res.state("a").unwrap().is_active());
    }

    // ── Conflicting successors ────────────────────────────────────────

    #[test]
    fn conflicting_successors_earliest_wins() {
        let a = friction_env("a", "2026-01-01T00:00:00Z", "a");
        let b = friction_env("b", "2026-01-02T00:00:00Z", "b");
        let c = friction_env("c", "2026-01-03T00:00:00Z", "c");
        let ss1 = ss_env("ss1", "2026-01-04T00:00:00Z", "a", "b");
        let ss2 = ss_env("ss2", "2026-01-05T00:00:00Z", "a", "c"); // later, different replacement

        let (res, outcomes) = resolve(vec![a, b, c, ss1, ss2]);

        // First supersession wins
        assert_eq!(outcomes.get("ss1").unwrap(), &ControlOutcome::Applied);
        assert!(matches!(
            outcomes.get("ss2").unwrap(),
            ControlOutcome::Inert { .. }
        ));
        match res.state("a").unwrap() {
            ResolvedState::Superseded { by, .. } => assert_eq!(by, "b"),
            other => panic!("expected Superseded by b, got {other:?}"),
        }
    }

    // ── Retraction dominates supersession ─────────────────────────────

    #[test]
    fn retraction_dominates_supersession() {
        let a = friction_env("a", "2026-01-01T00:00:00Z", "a");
        let b = friction_env("b", "2026-01-02T00:00:00Z", "b");
        let ss = ss_env("ss1", "2026-01-03T00:00:00Z", "a", "b");
        let ret = ret_env("ret1", "2026-01-04T00:00:00Z", "a");

        // Retraction comes AFTER supersession in time
        let (res, outcomes) = resolve(vec![a.clone(), b, ss, ret]);

        // Both apply: supersession first, then retraction
        assert_eq!(outcomes.get("ss1").unwrap(), &ControlOutcome::Applied);
        assert_eq!(outcomes.get("ret1").unwrap(), &ControlOutcome::Applied);

        // a is Retracted (retraction dominates in state construction)
        assert!(matches!(
            res.state("a").unwrap(),
            ResolvedState::Retracted { .. }
        ));
    }

    // ── Active/history projections ────────────────────────────────────

    #[test]
    fn active_projection_excludes_corrected() {
        let a = friction_env("a", "2026-01-01T00:00:00Z", "active");
        let b = friction_env("b", "2026-01-02T00:00:00Z", "superseded");
        let ss = ss_env("ss1", "2026-01-03T00:00:00Z", "b", "a");

        let (res, _outcomes) = resolve(vec![a.clone(), b.clone(), ss]);

        let active: Vec<&str> = res.active().iter().map(|e| e.uid.as_str()).collect();
        assert!(active.contains(&"a"));
        assert!(
            !active.contains(&"b"),
            "b is superseded, should not be in active"
        );

        let history: Vec<&str> = res.history().iter().map(|e| e.uid.as_str()).collect();
        assert!(history.contains(&"a"));
        assert!(history.contains(&"b"));
    }

    // ── Correction chain ──────────────────────────────────────────────

    #[test]
    fn correction_chain_reports_complete_history() {
        let a = friction_env("a", "2026-01-01T00:00:00Z", "a");
        let b = friction_env("b", "2026-01-02T00:00:00Z", "b");
        let c = friction_env("c", "2026-01-03T00:00:00Z", "c");
        let ss1 = ss_env("ss1", "2026-01-04T00:00:00Z", "a", "b");
        let ss2 = ss_env("ss2", "2026-01-05T00:00:00Z", "b", "c");

        let (res, _outcomes) = resolve(vec![a, b, c, ss1, ss2]);

        let chain = res.correction_chain("a");
        assert_eq!(chain.len(), 3); // a, b, c
        assert_eq!(chain[0].0, "a");
        assert_eq!(chain[1].0, "b");
        assert_eq!(chain[2].0, "c");
    }

    // ── Controls targeting controls are rejected ──────────────────────

    #[test]
    fn control_targeting_control_is_inert() {
        // ss1 is a control, and we try to supersede it with ss2
        let a = friction_env("a", "2026-01-01T00:00:00Z", "a");
        let b = friction_env("b", "2026-01-02T00:00:00Z", "b");
        let ss1 = ss_env("ss1", "2026-01-03T00:00:00Z", "a", "b");
        // ss2 targets ss1 (a control), not a primary
        let ss2 = ss_env("ss2", "2026-01-04T00:00:00Z", "ss1", "b");

        let (_res, outcomes) = resolve(vec![a, b, ss1, ss2]);

        // ss1 applies (targets primary a)
        assert_eq!(outcomes.get("ss1").unwrap(), &ControlOutcome::Applied);
        // ss2 is inert (targets control ss1, not a primary)
        assert!(matches!(
            outcomes.get("ss2").unwrap(),
            ControlOutcome::Inert { .. }
        ));
    }

    // ── Empty input ───────────────────────────────────────────────────

    #[test]
    fn empty_corpus_is_valid() {
        let (res, outcomes) = resolve(vec![]);
        assert!(res.diagnostics().is_empty());
        assert!(outcomes.is_empty());
        assert!(res.active().is_empty());
        assert!(res.history().is_empty());
    }

    // ── Canonical ordering ────────────────────────────────────────────

    #[test]
    fn canonical_ordering_is_by_recorded_at_then_uid() {
        let a = friction_env("a", "2026-01-02T00:00:00Z", "later");
        let b = friction_env("b", "2026-01-01T00:00:00Z", "earlier");
        let c = friction_env("c", "2026-01-01T00:00:00Z", "earlier-same-time");

        // b and c have same time, b < c by uid
        let (res, _outcomes) = resolve(vec![a, b, c]);

        let active = res.active();
        // Descending: a (most recent), c, b (least recent)
        assert_eq!(active[0].uid, "a");
        assert_eq!(active[1].uid, "c");
        assert_eq!(active[2].uid, "b");
    }

    // ── Controls targeting superseded observation still validated ─────

    #[test]
    fn retraction_of_already_superseded_target_is_applied() {
        // a → b, then retract a. The retraction should apply
        // (retraction dominates supersession in state construction).
        let a = friction_env("a", "2026-01-01T00:00:00Z", "a");
        let b = friction_env("b", "2026-01-02T00:00:00Z", "b");
        let ss = ss_env("ss1", "2026-01-03T00:00:00Z", "a", "b");
        let ret = ret_env("ret1", "2026-01-04T00:00:00Z", "a");

        let (res, outcomes) = resolve(vec![a, b, ss, ret]);

        assert_eq!(outcomes.get("ss1").unwrap(), &ControlOutcome::Applied);
        assert_eq!(outcomes.get("ret1").unwrap(), &ControlOutcome::Applied);
        assert!(matches!(
            res.state("a").unwrap(),
            ResolvedState::Retracted { .. }
        ));
    }
}
