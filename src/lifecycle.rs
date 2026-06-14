// SPDX-License-Identifier: GPL-3.0-only
//! `lifecycle` — the pure slice lifecycle FSM (axis A of ADR-009, SL-028 design
//! §5.4). Sibling to [`crate::conduct`] (axis B): *what* a change is — its movement
//! along `proposed → design → plan → ready → started → audit → reconcile → done`
//! (plus `abandoned`) — is modelled here as pure data over the edge table; *how* a
//! change is conducted lives in `conduct`.
//!
//! **Pure leaf tier (ADR-001).** No clock / disk / rng / git, and no kind module —
//! the FSM is total over its `&str` edges. The `slice` command shell reads the
//! authored status, injects the date, and writes; this module only classifies.

/// How a `from → to` slice-status move classifies under the lifecycle FSM
/// (design §5.4). Pure data over the edge table — no clock/disk; the verb stamps
/// the shell-injected date. `classify` is total over its `&str` inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Transition {
    /// A forward step along the chain, or the legitimate seam edges.
    Advance,
    /// A correction edge that walks back to re-do an invalidated stage.
    BackEdge,
    /// A move neither forward nor a named back-edge — written and surfaced.
    Skip,
    /// `* → abandoned` from any non-terminal source.
    Abandon,
    /// `from == to`; the writer no-ops.
    Noop,
    /// Leaving a terminal source (`{done, abandoned}`) — refused (reopening
    /// deferred).
    FromTerminal,
    /// A closure-seam breach (F12): `→ reconcile` from a non-`audit` source, or
    /// `→ done` from a non-`reconcile` source — refused structurally.
    SeamBreach,
}

/// Whether leaving this status is refused by the transition verb — the
/// *reopening-refusal* set (`{done, abandoned}`), F13. A **third**, distinct
/// slice-status predicate: it is NOT `is_terminal_status` (divergence,
/// `{done}` — adding `abandoned` there false-flags `⚠`) nor `is_hidden`
/// (presentation, semantically unrelated). Reopening a closed/abandoned slice is
/// deliberately deferred, so `set_slice_status` refuses a move out of either
/// (`FromTerminal`); the three predicates diverge by design (design §5.2/§5.3).
pub(crate) fn is_transition_terminal(status: &str) -> bool {
    matches!(status, "done" | "abandoned")
}

/// Whether a `from → to` move crosses the **closure seam** (design §7, D8): the
/// two legitimate terminal advances `audit → reconcile` and `reconcile → done`.
/// Pure — the reverse close-gate (`slice::run_status`) fires the RV-blocker scan
/// ONLY on these edges, never on any other transition (VT-4). These are exactly the
/// `to`-targets the `SeamBreach` guard protects (§5.5), taken from their one legal
/// source: structurally, `set_slice_status` is the sole writer of these moves, so
/// that shell is the sole seam-crosser (VT-5).
pub(crate) fn crosses_closure_seam(from: &str, to: &str) -> bool {
    matches!((from, to), ("audit", "reconcile") | ("reconcile", "done"))
}

/// Classify a `from → to` slice-status move against the FSM (design §5.4),
/// edge-table driven (NOT index arithmetic — `abandoned` is last in the const but
/// is not "after `done`" in the FSM). `to` is assumed in-vocab (the verb boundary
/// guards an out-of-vocab target); `from` may be drifted (out-of-vocab), in which
/// case a non-seam, non-terminal move falls through to `Skip` — but the seam still
/// binds by *target* edge (`→ reconcile`/`→ done` from a drifted source is a
/// `SeamBreach`, §5.5). Precedence: no-op → from-terminal → closure-seam (by
/// target) → abandon → forward/back edges → skip.
pub(crate) fn classify(from: &str, to: &str) -> Transition {
    if from == to {
        return Transition::Noop;
    }
    if is_transition_terminal(from) {
        return Transition::FromTerminal;
    }
    // Closure seam (F12), gated by the *target* edge — binds even from a drifted
    // `from`. The legitimate seam entries are the only way in.
    if to == "reconcile" {
        return if from == "audit" {
            Transition::Advance
        } else {
            Transition::SeamBreach
        };
    }
    if to == "done" {
        return if from == "reconcile" {
            Transition::Advance
        } else {
            Transition::SeamBreach
        };
    }
    if to == "abandoned" {
        return Transition::Abandon;
    }
    // Forward chain (the non-seam advances) and the named back-edges.
    match (from, to) {
        ("proposed", "design")
        | ("design", "plan")
        | ("plan", "ready")
        | ("ready", "started")
        | ("started", "audit") => Transition::Advance,
        ("audit", "started" | "design") | ("reconcile", "audit" | "design") => Transition::BackEdge,
        _ => Transition::Skip,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- SL-028 PHASE-01: lifecycle FSM ---

    // VT-1: classify table (design §5.4/§9). Edge-table driven; covers advance,
    // each back-edge, skip, abandon, noop, from-terminal, seam-breach (incl. from
    // a drifted source), and the legit seam path audit→reconcile→done = Advance.

    #[test]
    fn classify_forward_chain_is_advance() {
        for (from, to) in [
            ("proposed", "design"),
            ("design", "plan"),
            ("plan", "ready"),
            ("ready", "started"),
            ("started", "audit"),
        ] {
            assert_eq!(classify(from, to), Transition::Advance, "{from} → {to}");
        }
    }

    #[test]
    fn classify_legit_closure_seam_path_is_advance() {
        // audit → reconcile → done — the ADR-003 §7/§8 spine.
        assert_eq!(classify("audit", "reconcile"), Transition::Advance);
        assert_eq!(classify("reconcile", "done"), Transition::Advance);
    }

    #[test]
    fn classify_named_back_edges() {
        for (from, to) in [
            ("audit", "started"),
            ("audit", "design"),
            ("reconcile", "audit"),
            ("reconcile", "design"),
        ] {
            assert_eq!(classify(from, to), Transition::BackEdge, "{from} → {to}");
        }
    }

    #[test]
    fn classify_abandon_from_each_non_terminal() {
        for from in [
            "proposed",
            "design",
            "plan",
            "ready",
            "started",
            "audit",
            "reconcile",
        ] {
            assert_eq!(
                classify(from, "abandoned"),
                Transition::Abandon,
                "{from} → abandoned"
            );
        }
    }

    #[test]
    fn classify_noop_when_unchanged() {
        assert_eq!(classify("started", "started"), Transition::Noop);
        // No-op precedes from-terminal: done → done is a no-op, not a refusal.
        assert_eq!(classify("done", "done"), Transition::Noop);
    }

    #[test]
    fn classify_from_terminal_refused() {
        for from in ["done", "abandoned"] {
            assert_eq!(
                classify(from, "design"),
                Transition::FromTerminal,
                "{from} → design"
            );
        }
    }

    #[test]
    fn classify_seam_breach_to_reconcile_from_non_audit() {
        for from in ["proposed", "design", "plan", "ready", "started"] {
            assert_eq!(
                classify(from, "reconcile"),
                Transition::SeamBreach,
                "{from} → reconcile"
            );
        }
    }

    #[test]
    fn classify_seam_breach_to_done_from_non_reconcile() {
        for from in ["proposed", "design", "plan", "ready", "started", "audit"] {
            assert_eq!(
                classify(from, "done"),
                Transition::SeamBreach,
                "{from} → done"
            );
        }
    }

    #[test]
    fn classify_seam_binds_even_from_a_drifted_source() {
        // The seam is about the target edge, not the source's validity (§5.5).
        assert_eq!(classify("bogus", "reconcile"), Transition::SeamBreach);
        assert_eq!(classify("bogus", "done"), Transition::SeamBreach);
    }

    #[test]
    fn classify_move_out_of_drift_is_skip_not_refused() {
        // Out-of-vocab `from`, non-seam, non-terminal target → Skip (allowed).
        assert_eq!(classify("bogus", "started"), Transition::Skip);
    }

    #[test]
    fn classify_non_chain_move_is_skip() {
        // A legal-vocab pair the FSM never names (and not a seam target) → Skip.
        assert_eq!(classify("proposed", "started"), Transition::Skip);
        assert_eq!(classify("design", "started"), Transition::Skip);
    }
}
