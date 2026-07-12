// SPDX-License-Identifier: GPL-3.0-only
//! Tension detection + evidence grading (SL-218 PHASE-02, design §2).
//!
//! A **tension** is a `value_dim`-order vs delivery-order inversion: an item `A`
//! ranks above `B` on `value_dim` yet `B` surfaces first in the delivery order.
//! Phase D's job is to *surface* these rather than resolve them silently
//! (RFC-019). Two primitives, both **pure**:
//!
//! - [`detect`] — the pair scan (design D4 predicate). Over passed-in projections
//!   only; it makes **no** comparison-compile calls (ADR-001 / VA-1 layering
//!   fence — the caller assembles graph data and hands it in). Classifies each
//!   surviving inversion as [`TensionCause::Structure`] (a surviving seq/dep path
//!   constrains `A` behind `B`) or [`TensionCause::Composition`] (full-score
//!   dimensions lift `B`).
//! - [`grade`] — the evidence vocabulary (design D6). A pure decision over two
//!   determinacy verdicts (verdict system + full system) and their rater counts;
//!   the impure assembly that *builds* those verdicts via the shipped
//!   `PairSide`/`determined()` machinery lives in [`super::surface`] (the elicit
//!   pattern), keeping this module free of compile calls.
//!
//! Detection emits [`DetectedTension`] (pair + cause, grade-free); the surface
//! layer grades each and assembles the final [`Tension`] (design: grade is
//! injected at the assembly tier).

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::comparison::RaterCounts;
use crate::relation_graph::EntityKey;

/// A surviving structural edge cited by a [`TensionCause::Structure`]: `from`
/// precedes `to` (i.e. `to` is `after`/`needs` `from`). It is the first forward
/// edge on a path from the surfaced member to the preferred member — the
/// concrete constraint the callout names ("`after SL-009` sequence survives").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StructuralEdge {
    pub from: EntityKey,
    pub to: EntityKey,
    pub kind: EdgeKind,
}

/// Which precedence overlay a structural edge belongs to — drives the callout
/// verb (`after` vs `needs`); render is PHASE-03.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EdgeKind {
    /// A `seq` (`after`) edge.
    Seq,
    /// A `dep` (`needs`) edge.
    Dep,
}

/// One direct surviving predecessor of a node, with the overlay it rode in on —
/// the [`detect`] adjacency entry (`node` is `after`/`needs` `pred`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PredEdge {
    pub pred: EntityKey,
    pub kind: EdgeKind,
}

/// Per-dimension full-score advantage of the surfaced member over the preferred
/// member (`surfaced − preferred`) — why `B` lifts above `A` on full score even
/// though `A` outranks it on `value_dim` alone (design D4 Composition).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ComponentDeltas {
    pub risk_dim: f64,
    pub leverage: f64,
    pub optionality: f64,
}

/// Why the delivery order inverts the `value_dim` order for a pair (design D4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum TensionCause {
    /// A surviving seq/dep path constrains the preferred member behind the
    /// surfaced one; the cited edge is the first forward hop from surfaced.
    Structure { edge: StructuralEdge },
    /// No structural path — full-score dimensions (risk/leverage/optionality)
    /// lift the surfaced member above the preferred one.
    Composition { deltas: ComponentDeltas },
}

/// The `value_dim` ordering's evidential standing (design D6). Counts always
/// come from the system that produced the verdict (RV-271 F-2/F-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EvidenceGrade {
    /// The verdict system determines the ordering; counts are THAT system's
    /// constraining rows (the human system when the knob is on).
    Determined { counts: RaterCounts },
    /// Knob-on only: the full system determines but the human system does not —
    /// the ordering is proposed by agent evidence, unconfirmed. Counts are the
    /// full system's, labelled as agent-proposed at render.
    AgentProposed { counts: RaterCounts },
    /// No determining evidence in any consulted system.
    Projected,
}

/// A detected inversion before grading — the pure [`detect`] output. The surface
/// layer grades each into a final [`Tension`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DetectedTension {
    /// Ranked higher on `value_dim`, surfaces later.
    pub preferred: EntityKey,
    /// Surfaces first in delivery order.
    pub surfaced: EntityKey,
    pub cause: TensionCause,
}

/// A graded tension — the surface-layer assembly product (design §2 Types).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Tension {
    pub preferred: EntityKey,
    pub surfaced: EntityKey,
    pub cause: TensionCause,
    pub grade: EvidenceGrade,
}

impl DetectedTension {
    /// Inject a computed grade, producing the final [`Tension`].
    pub(crate) fn with_grade(self, grade: EvidenceGrade) -> Tension {
        Tension {
            preferred: self.preferred,
            surfaced: self.surfaced,
            cause: self.cause,
            grade,
        }
    }
}

/// Passed-in projections for [`detect`] — all the data the pure scan needs, so
/// the fn touches no graph or compile itself (VA-1). The caller
/// ([`super::surface`], PHASE-03) assembles these from the [`super::graph`].
pub(crate) struct DetectInputs<'a> {
    /// The full frontier order in delivery order (position = delivery rank);
    /// NOT the rendered page (design predicate: preferred ranges over all of F).
    pub delivery_order: &'a [EntityKey],
    /// The rendered page bound `K`: the surfaced member must sit at `pos < K`.
    pub page_k: usize,
    /// The point `value_dim` projection the engine consumed (do not re-derive).
    pub value_dim: &'a BTreeMap<EntityKey, f64>,
    /// The value multiplier `m` (design D6). A member with `m = 0` is
    /// value-insensitive → the pair is excluded (F-6).
    pub multiplier: &'a BTreeMap<EntityKey, f64>,
    /// Full display score — an exact tie (`total_cmp`) excludes the pair as an
    /// id-tiebreak artifact rather than asserting a Composition cause.
    pub full_score: &'a BTreeMap<EntityKey, f64>,
    /// Composition-dimension maps (surfaced − preferred deltas).
    pub risk_dim: &'a BTreeMap<EntityKey, f64>,
    pub leverage: &'a BTreeMap<EntityKey, f64>,
    pub optionality: &'a BTreeMap<EntityKey, f64>,
    /// Direct surviving seq+dep predecessors per node (merged overlays). Structure
    /// iff the preferred member is reachable from the surfaced one over this graph.
    pub preds: &'a BTreeMap<EntityKey, Vec<PredEdge>>,
}

/// `m == 0` without a float-cmp footgun (mirrors `query::is_zero`).
fn is_zero(w: f64) -> bool {
    w.abs().total_cmp(&0.0).is_eq()
}

/// The first forward structural edge on a path from `surfaced` to `preferred`
/// over the surviving predecessor graph, or `None` if `preferred` is not
/// reachable from `surfaced` (⇒ not a Structure tension). BFS from `preferred`
/// backward over `preds`; the edge adjacent to `surfaced` is the citation
/// (design D4: cite the first edge on one such path). Deterministic: predecessors
/// are visited in sorted order, shortest path wins.
fn structural_edge(
    preds: &BTreeMap<EntityKey, Vec<PredEdge>>,
    preferred: EntityKey,
    surfaced: EntityKey,
) -> Option<StructuralEdge> {
    // parent[x] = (child, kind): x was discovered as a predecessor of `child`
    // via edge `x -> child` (x precedes child). BFS outward from `preferred`
    // over predecessor edges; when `surfaced` is reached, parent[surfaced] is
    // the first forward hop leaving surfaced.
    let mut parent: BTreeMap<EntityKey, (EntityKey, EdgeKind)> = BTreeMap::new();
    let mut visited: BTreeSet<EntityKey> = BTreeSet::from([preferred]);
    let mut queue: VecDeque<EntityKey> = VecDeque::from([preferred]);
    while let Some(node) = queue.pop_front() {
        if node == surfaced {
            return parent.get(&surfaced).map(|&(to, kind)| StructuralEdge {
                from: surfaced,
                to,
                kind,
            });
        }
        let mut edges = preds.get(&node).cloned().unwrap_or_default();
        edges.sort_by_key(|e| e.pred);
        for PredEdge { pred, kind } in edges {
            if visited.insert(pred) {
                parent.insert(pred, (node, kind));
                queue.push_back(pred);
            }
        }
    }
    None
}

/// Detect `value_dim`-order vs delivery-order tensions (design D4). Pure over
/// [`DetectInputs`]; emits in delivery-order position order (surfaced outer,
/// preferred inner) for determinism.
///
/// For each on-page surfaced `B` (`pos(B) < K`) and each later `A`
/// (`pos(A) > pos(B)`) with `value_dim(A) > value_dim(B)` and neither member
/// `m = 0`:
/// - **Structure** iff `A` is reachable from `B` over the surviving graph (cite
///   the first edge);
/// - else **excluded** if the full scores tie (`total_cmp`) — an id-tiebreak
///   artifact, not an override;
/// - else **Composition** with per-dimension `surfaced − preferred` deltas.
pub(crate) fn detect(inputs: &DetectInputs<'_>) -> Vec<DetectedTension> {
    let val = |k: &EntityKey| inputs.value_dim.get(k).copied().unwrap_or(0.0);
    let mult = |k: &EntityKey| inputs.multiplier.get(k).copied().unwrap_or(0.0);
    let comp = |m: &BTreeMap<EntityKey, f64>, k: &EntityKey| m.get(k).copied().unwrap_or(0.0);

    let mut out = Vec::new();
    for (bi, &surfaced) in inputs.delivery_order.iter().enumerate() {
        if bi >= inputs.page_k {
            continue; // surfaced member must be on the rendered page
        }
        for (ai, &preferred) in inputs.delivery_order.iter().enumerate() {
            if ai <= bi {
                continue; // preferred must surface strictly later than surfaced
            }
            // Value-insensitive member ⇒ not a value tension (F-6, SL-217 D6).
            if is_zero(mult(&preferred)) || is_zero(mult(&surfaced)) {
                continue;
            }
            // The value_dim inversion: preferred ranks strictly above surfaced.
            if val(&preferred).total_cmp(&val(&surfaced)) != Ordering::Greater {
                continue;
            }
            let cause = if let Some(edge) = structural_edge(inputs.preds, preferred, surfaced) {
                TensionCause::Structure { edge }
            } else if inputs
                .full_score
                .get(&preferred)
                .copied()
                .unwrap_or(0.0)
                .total_cmp(&inputs.full_score.get(&surfaced).copied().unwrap_or(0.0))
                .is_eq()
            {
                continue; // equal full score ⇒ id-tiebreak artifact, not a cause
            } else {
                TensionCause::Composition {
                    deltas: ComponentDeltas {
                        risk_dim: comp(inputs.risk_dim, &surfaced)
                            - comp(inputs.risk_dim, &preferred),
                        leverage: comp(inputs.leverage, &surfaced)
                            - comp(inputs.leverage, &preferred),
                        optionality: comp(inputs.optionality, &surfaced)
                            - comp(inputs.optionality, &preferred),
                    },
                }
            };
            out.push(DetectedTension {
                preferred,
                surfaced,
                cause,
            });
        }
    }
    out
}

/// Grade a detected tension's `value_dim` ordering (design D6) — a pure decision
/// over two determinacy verdicts and their rater counts. The impure work of
/// building those verdicts (`compile` → `side_in` → `determined`) is the surface
/// layer's, so grade/queue read the SAME predicate + system selection (F-1/F-7,
/// one truth per question).
///
/// - determined in the verdict system ⇒ [`EvidenceGrade::Determined`] with that
///   system's counts (the human system's when the knob is on);
/// - else, knob-on and determined in the full system ⇒
///   [`EvidenceGrade::AgentProposed`] with the full system's counts;
/// - else [`EvidenceGrade::Projected`].
///
/// Knob-off, `AgentProposed` is unreachable (the full system IS the verdict
/// system, so the first arm already fired).
pub(crate) fn grade(
    knob_on: bool,
    determined_verdict: bool,
    verdict_counts: RaterCounts,
    determined_full: bool,
    full_counts: RaterCounts,
) -> EvidenceGrade {
    if determined_verdict {
        EvidenceGrade::Determined {
            counts: verdict_counts,
        }
    } else if knob_on && determined_full {
        EvidenceGrade::AgentProposed {
            counts: full_counts,
        }
    } else {
        EvidenceGrade::Projected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(id: u32) -> EntityKey {
        EntityKey { prefix: "SL", id }
    }

    /// Build `DetectInputs` from per-node tuples for terse fixtures. Each node:
    /// `(id, value_dim, multiplier, full_score, risk_dim, leverage, optionality)`.
    /// `delivery_order` is the given id order; `preds` maps id → its predecessors.
    struct Fx {
        order: Vec<EntityKey>,
        value_dim: BTreeMap<EntityKey, f64>,
        multiplier: BTreeMap<EntityKey, f64>,
        full_score: BTreeMap<EntityKey, f64>,
        risk_dim: BTreeMap<EntityKey, f64>,
        leverage: BTreeMap<EntityKey, f64>,
        optionality: BTreeMap<EntityKey, f64>,
        preds: BTreeMap<EntityKey, Vec<PredEdge>>,
    }

    impl Fx {
        fn new(nodes: &[(u32, f64, f64, f64, f64, f64, f64)]) -> Self {
            let mut fx = Fx {
                order: Vec::new(),
                value_dim: BTreeMap::new(),
                multiplier: BTreeMap::new(),
                full_score: BTreeMap::new(),
                risk_dim: BTreeMap::new(),
                leverage: BTreeMap::new(),
                optionality: BTreeMap::new(),
                preds: BTreeMap::new(),
            };
            for &(id, v, m, s, r, l, o) in nodes {
                let k = key(id);
                fx.order.push(k);
                fx.value_dim.insert(k, v);
                fx.multiplier.insert(k, m);
                fx.full_score.insert(k, s);
                fx.risk_dim.insert(k, r);
                fx.leverage.insert(k, l);
                fx.optionality.insert(k, o);
            }
            fx
        }

        /// `node` is `after`/`needs` `pred`.
        fn edge(mut self, node: u32, pred: u32, kind: EdgeKind) -> Self {
            self.preds.entry(key(node)).or_default().push(PredEdge {
                pred: key(pred),
                kind,
            });
            self
        }

        fn inputs(&self, page_k: usize) -> DetectInputs<'_> {
            DetectInputs {
                delivery_order: &self.order,
                page_k,
                value_dim: &self.value_dim,
                multiplier: &self.multiplier,
                full_score: &self.full_score,
                risk_dim: &self.risk_dim,
                leverage: &self.leverage,
                optionality: &self.optionality,
                preds: &self.preds,
            }
        }
    }

    // ── VT-1: detection fixtures (design VT-D) ────────────────────────────────

    #[test]
    fn no_tension_when_delivery_matches_value() {
        // Delivery order == value order: no inversion.
        let fx = Fx::new(&[
            (1, 10.0, 1.0, 10.0, 0.0, 0.0, 0.0),
            (2, 5.0, 1.0, 5.0, 0.0, 0.0, 0.0),
        ]);
        assert!(detect(&fx.inputs(2)).is_empty());
    }

    #[test]
    fn structure_direct_edge_cited() {
        // Delivery [B=1, A=2]; A outranks B on value_dim; A after B (direct seq).
        let fx = Fx::new(&[
            (1, 5.0, 1.0, 8.0, 0.0, 0.0, 0.0),
            (2, 10.0, 1.0, 4.0, 0.0, 0.0, 0.0),
        ])
        .edge(2, 1, EdgeKind::Seq);
        let out = detect(&fx.inputs(2));
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].preferred, key(2));
        assert_eq!(out[0].surfaced, key(1));
        assert_eq!(
            out[0].cause,
            TensionCause::Structure {
                edge: StructuralEdge {
                    from: key(1),
                    to: key(2),
                    kind: EdgeKind::Seq,
                }
            }
        );
    }

    #[test]
    fn structure_transitive_path_through_on_page_node() {
        // Delivery [B=1, M=2, A=3]; A after M after B. First forward edge from B
        // is B->M (2 needs 1). A outranks B on value_dim.
        let fx = Fx::new(&[
            (1, 3.0, 1.0, 9.0, 0.0, 0.0, 0.0),
            (2, 4.0, 1.0, 7.0, 0.0, 0.0, 0.0),
            (3, 10.0, 1.0, 5.0, 0.0, 0.0, 0.0),
        ])
        .edge(3, 2, EdgeKind::Dep)
        .edge(2, 1, EdgeKind::Dep);
        let out = detect(&fx.inputs(3));
        let t = out
            .iter()
            .find(|t| t.preferred == key(3) && t.surfaced == key(1))
            .expect("3-over-1 tension present");
        assert_eq!(
            t.cause,
            TensionCause::Structure {
                edge: StructuralEdge {
                    from: key(1),
                    to: key(2),
                    kind: EdgeKind::Dep,
                }
            },
            "first forward edge from surfaced B=1 is B->M (1<-2)"
        );
    }

    #[test]
    fn structure_path_through_off_page_node() {
        // Page K=1: only B=1 on page. A=3 is off-page (pos 2) but reachable from
        // B via off-page M=2. Preferred ranges over the full order (F-4), so the
        // tension is detected even though A sits below the cutoff.
        let fx = Fx::new(&[
            (1, 3.0, 1.0, 9.0, 0.0, 0.0, 0.0),
            (2, 2.0, 1.0, 8.0, 0.0, 0.0, 0.0),
            (3, 10.0, 1.0, 5.0, 0.0, 0.0, 0.0),
        ])
        .edge(3, 2, EdgeKind::Seq)
        .edge(2, 1, EdgeKind::Seq);
        let out = detect(&fx.inputs(1));
        let t = out
            .iter()
            .find(|t| t.preferred == key(3) && t.surfaced == key(1))
            .expect("off-page preferred still detected (F-4)");
        assert!(matches!(t.cause, TensionCause::Structure { .. }));
    }

    #[test]
    fn composition_when_no_structural_path() {
        // A=2 outranks B=1 on value_dim, no edge; full scores differ. Deltas are
        // surfaced(B) − preferred(A) per dimension.
        let fx = Fx::new(&[
            (1, 5.0, 1.0, 12.0, 0.8, 2.1, 0.3),
            (2, 10.0, 1.0, 4.0, 0.0, 0.0, 0.0),
        ]);
        let out = detect(&fx.inputs(2));
        assert_eq!(out.len(), 1);
        assert_eq!(
            out[0].cause,
            TensionCause::Composition {
                deltas: ComponentDeltas {
                    risk_dim: 0.8,
                    leverage: 2.1,
                    optionality: 0.3,
                }
            }
        );
    }

    #[test]
    fn value_dim_tie_is_not_a_tension() {
        // Equal value_dim ⇒ no inversion claim, even if delivery orders them.
        let fx = Fx::new(&[
            (1, 5.0, 1.0, 8.0, 0.0, 0.0, 0.0),
            (2, 5.0, 1.0, 4.0, 0.0, 0.0, 0.0),
        ]);
        assert!(detect(&fx.inputs(2)).is_empty());
    }

    #[test]
    fn equal_full_score_tiebreak_excluded() {
        // A outranks B on value_dim, no structure, but full scores are EQUAL:
        // the inversion is an id-tiebreak artifact, not a Composition override.
        let fx = Fx::new(&[
            (1, 5.0, 1.0, 7.0, 0.0, 0.0, 0.0),
            (2, 10.0, 1.0, 7.0, 0.0, 0.0, 0.0),
        ]);
        assert!(detect(&fx.inputs(2)).is_empty());
    }

    #[test]
    fn zero_multiplier_pair_excluded() {
        // B=1 has m=0 (value-insensitive) — excluded from tension claims (F-6).
        let fx = Fx::new(&[
            (1, 5.0, 0.0, 8.0, 0.0, 0.0, 0.0),
            (2, 10.0, 1.0, 4.0, 0.0, 0.0, 0.0),
        ])
        .edge(2, 1, EdgeKind::Seq);
        assert!(detect(&fx.inputs(2)).is_empty());
    }

    /// A mixed corpus: B=1 and C=2 tie on value_dim (so the 1-2 pair is NOT a
    /// tension); A=3 outranks both. A after B ⇒ 3-over-1 is Structure; 3-over-2
    /// has no path ⇒ Composition. Emission is surfaced-position outer, preferred
    /// inner — so 3-over-1 (surfaced pos 0) precedes 3-over-2 (surfaced pos 1).
    fn mixed_fx() -> Fx {
        Fx::new(&[
            (1, 3.0, 1.0, 9.0, 0.0, 0.0, 0.0),
            (2, 3.0, 1.0, 8.0, 0.0, 1.0, 0.0),
            (3, 10.0, 1.0, 5.0, 0.0, 0.0, 0.0),
        ])
        .edge(3, 1, EdgeKind::Seq)
    }

    #[test]
    fn mixed_structure_and_composition_both_emitted_in_order() {
        let out = detect(&mixed_fx().inputs(3));
        assert_eq!(out.len(), 2, "value-tied 1-2 pair excluded: {out:?}");
        assert_eq!((out[0].surfaced, out[0].preferred), (key(1), key(3)));
        assert!(matches!(out[0].cause, TensionCause::Structure { .. }));
        assert_eq!((out[1].surfaced, out[1].preferred), (key(2), key(3)));
        assert!(matches!(out[1].cause, TensionCause::Composition { .. }));
    }

    #[test]
    fn detection_is_deterministic() {
        assert_eq!(detect(&mixed_fx().inputs(3)), detect(&mixed_fx().inputs(3)));
    }

    // ── VT-2: grade vocabulary (design VT-J state-level) ──────────────────────

    fn counts(human: u32, agent: u32) -> RaterCounts {
        RaterCounts { human, agent }
    }

    #[test]
    fn knob_on_human_determined_is_determined_with_human_counts() {
        let g = grade(true, true, counts(3, 0), false, counts(3, 4));
        assert_eq!(
            g,
            EvidenceGrade::Determined {
                counts: counts(3, 0)
            }
        );
    }

    #[test]
    fn knob_on_agent_only_determined_is_agent_proposed_with_full_counts() {
        // Human system indeterminate, full system determined ⇒ AgentProposed,
        // counts from the full system (the rows that actually decided it).
        let g = grade(true, false, counts(0, 0), true, counts(0, 4));
        assert_eq!(
            g,
            EvidenceGrade::AgentProposed {
                counts: counts(0, 4)
            }
        );
    }

    #[test]
    fn neither_system_determines_is_projected() {
        let g = grade(true, false, counts(0, 0), false, counts(0, 0));
        assert_eq!(g, EvidenceGrade::Projected);
    }

    #[test]
    fn knob_off_agent_proposed_unreachable() {
        // Knob-off: the full system IS the verdict system. An undetermined verdict
        // is Projected even though `determined_full` is true — AgentProposed is
        // gated on knob-on, so it can never appear.
        let g = grade(false, false, counts(0, 0), true, counts(0, 5));
        assert_eq!(g, EvidenceGrade::Projected);
    }

    #[test]
    fn knob_off_determined_uses_verdict_counts() {
        let g = grade(false, true, counts(2, 3), false, counts(2, 3));
        assert_eq!(
            g,
            EvidenceGrade::Determined {
                counts: counts(2, 3)
            }
        );
    }
}
