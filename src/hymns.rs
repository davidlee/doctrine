// SPDX-License-Identifier: GPL-3.0-only
//! Pure prompt-cascade resolver engine (SL-186 PHASE-01, design §5.1/§5.2). Given a
//! context vector and an in-memory snippet corpus, emit ordered, composed markdown.
//! This is the leaf/engine layer — no disk, clock, or env (INV-5); the `SealSet` is
//! passed in, never read here. Deterministic (INV-7): the same `(corpus, ctx, sealed)`
//! yields byte-identical output.
//!
//! The precedence key is `band → specificity → provenance(framework<user) → alpha`,
//! ordered ascending so the most-specific / user snippet gets the last word (design
//! §5.1, D1/D3). Every matching snippet concatenates exactly once (INV-2); only
//! `replaces` suppresses, and only from the unique most-specific active snippet of a
//! slot (INV-3). A disk-provenance snippet whose slot is sealed is dropped before
//! matching (INV-6). The loader (PHASE-02) is the only impurity; nothing here reads I/O.
#![allow(
    dead_code,
    reason = "SL-186 PHASE-01: inert pure engine authored ahead of its callers; the \
              PHASE-02 loader and PHASE-03 `prompt` verbs consume this surface. Until \
              then it is exercised only by the in-module goldens (the slice ships inert \
              by design — see slice-186.md)."
)]

use std::collections::{BTreeMap, BTreeSet};

/// The selector/band axes. `arm` is an axis but never a band; `preamble`/`project`
/// are bands with no namesake axis (design §5.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Axis {
    Harness,
    Model,
    Role,
    Arm,
    Stage,
}

const ALL_AXES: [Axis; 5] = [
    Axis::Harness,
    Axis::Model,
    Axis::Role,
    Axis::Arm,
    Axis::Stage,
];

/// The closed band registry. Declaration order **is** the fixed band order (INV-1):
/// `preamble · harness · model · role · stage · project`. `Ord` derives from it, so a
/// band's position never depends on its label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Band {
    Preamble,
    Harness,
    Model,
    Role,
    Stage,
    Project,
}

/// Every band, in declaration order — the closed registry (STD-001 single source; used
/// to drive `from_segment` off the same `as_str` mapping).
const ALL_BANDS: [Band; 6] = [
    Band::Preamble,
    Band::Harness,
    Band::Model,
    Band::Role,
    Band::Stage,
    Band::Project,
];

/// The wildcard tail segment in a `model` key (`anthropic/_default`). Matches any ctx
/// segment at its position and does not add specificity.
const DEFAULT_SEGMENT: &str = "_default";

impl Band {
    /// The path-segment name for this band — the single source for band ⇄ segment.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Band::Preamble => "preamble",
            Band::Harness => "harness",
            Band::Model => "model",
            Band::Role => "role",
            Band::Stage => "stage",
            Band::Project => "project",
        }
    }

    /// Parse the first path segment under the corpus root into a band.
    pub(crate) fn from_segment(seg: &str) -> Option<Band> {
        ALL_BANDS.into_iter().find(|b| b.as_str() == seg)
    }

    /// The band's namesake axis, which leads its specificity metric (D3). `preamble`
    /// and `project` have none.
    pub(crate) fn primary_axis(self) -> Option<Axis> {
        match self {
            Band::Harness => Some(Axis::Harness),
            Band::Model => Some(Axis::Model),
            Band::Role => Some(Axis::Role),
            Band::Stage => Some(Axis::Stage),
            Band::Preamble | Band::Project => None,
        }
    }
}

/// `orchestrator | worker` — the role-axis value; also selects the assembly shape at
/// the command layer (F15/A), which is not this engine's concern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Role {
    Orchestrator,
    Worker,
}

/// `subagent | subprocess` — the dispatch arm axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Arm {
    Subagent,
    Subprocess,
}

/// A snippet's source root determines provenance; it is derived, never stored on disk.
/// `Framework < User` (derived `Ord`) so that at equal specificity the user's same-slot
/// edit wins the tiebreak (design §5.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Provenance {
    Framework,
    User,
}

/// `<band>/<label>` — a snippet's identity. Path-derived; two snippets sharing a slot
/// are the framework original and its user twin.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Slot {
    pub band: Band,
    pub label: String,
}

impl Slot {
    pub(crate) fn new(band: Band, label: impl Into<String>) -> Slot {
        Slot {
            band,
            label: label.into(),
        }
    }

    /// Full `band/label` path — the deterministic alpha tiebreak key.
    fn path(&self) -> String {
        format!("{}/{}", self.band.as_str(), self.label)
    }
}

/// Axis→pattern constraints plus an optional `replaces` target slot. A `None` axis is
/// "don't care"; a pinned axis must match the context (non-match ≠ override).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Selector {
    pub harness: Option<String>,
    /// A `model` key: `<vendor>/<segment…>`, matched left-to-right; `_default` is the
    /// wildcard tail.
    pub model: Option<String>,
    pub role: Option<Role>,
    pub arm: Option<Arm>,
    pub stage: Option<String>,
    /// The slot this snippet suppresses when it is the unique most-specific active
    /// snippet of its own slot (INV-3).
    pub replaces: Option<Slot>,
}

impl Selector {
    /// The pinned depth of an axis: 1 for a single-token axis, the count of
    /// non-`_default` segments for `model`, 0 when unpinned.
    fn depth_of(&self, axis: Axis) -> u32 {
        match axis {
            Axis::Harness => u32::from(self.harness.is_some()),
            Axis::Role => u32::from(self.role.is_some()),
            Axis::Arm => u32::from(self.arm.is_some()),
            Axis::Stage => u32::from(self.stage.is_some()),
            Axis::Model => self.model.as_deref().map_or(0, model_depth),
        }
    }
}

/// One prose snippet: its slot, selector, source provenance, and markdown body.
#[derive(Debug, Clone)]
pub(crate) struct Snippet {
    pub slot: Slot,
    pub selector: Selector,
    pub provenance: Provenance,
    pub body: String,
}

/// Restrict output to a subset of bands, or emit every band the shape includes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BandFilter {
    All,
    Only(BTreeSet<Band>),
}

impl BandFilter {
    fn includes(&self, band: Band) -> bool {
        match self {
            BandFilter::All => true,
            BandFilter::Only(set) => set.contains(&band),
        }
    }
}

/// The resolution context ("the element"): the role plus optional axis pins and a band
/// filter. `role` is always present; the rest degrade gracefully when absent.
#[derive(Debug, Clone)]
pub(crate) struct ContextVector {
    pub role: Role,
    pub harness: Option<String>,
    pub model: Option<String>,
    pub arm: Option<Arm>,
    pub stage: Option<String>,
    pub bands: BandFilter,
}

/// The sealed-slot set (built by the loader from the embedded manifest, passed in to
/// keep the engine pure). A disk twin of a sealed slot is dropped before matching.
#[derive(Debug, Clone, Default)]
pub(crate) struct SealSet(pub BTreeSet<Slot>);

impl SealSet {
    fn contains(&self, slot: &Slot) -> bool {
        self.0.contains(slot)
    }
}

/// An authoring error in the `replaces` graph, surfaced rather than silently
/// alpha-ordered (INV-3). Consumed by `doctrine prompt check`/`validate` (PHASE-03).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolveError {
    /// A snippet carries `replaces` but is not the unique most-specific active snippet
    /// of its own slot.
    NonTopReplacer(Slot),
    /// Two active snippets both `replaces` the same target slot.
    DuplicateTarget(Slot),
    /// The cross-slot `replaces` edges form a cycle.
    Cycle(Vec<Slot>),
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveError::NonTopReplacer(s) => {
                write!(
                    f,
                    "`replaces` on {} which is not the unique most-specific active snippet for its slot",
                    s.path()
                )
            }
            ResolveError::DuplicateTarget(s) => {
                write!(
                    f,
                    "two active snippets `replaces` the same slot {}",
                    s.path()
                )
            }
            ResolveError::Cycle(slots) => {
                let path = slots.iter().map(Slot::path).collect::<Vec<_>>().join(" → ");
                write!(f, "`replaces` cycle: {path}")
            }
        }
    }
}

impl std::error::Error for ResolveError {}

/// The ordering key: `band → specificity → provenance → alpha(full slot path)`,
/// compared ascending so the most-specific / user snippet lands last (last word wins).
type PrecedenceKey = (Band, (u32, u32), Provenance, String);

fn precedence_key(s: &Snippet) -> PrecedenceKey {
    (
        s.slot.band,
        specificity(s.slot.band, &s.selector),
        s.provenance,
        s.slot.path(),
    )
}

/// The `model`-key depth: non-`_default` segment count.
fn model_depth(pat: &str) -> u32 {
    let count = pat.split('/').filter(|s| *s != DEFAULT_SEGMENT).count();
    u32::try_from(count).unwrap_or(u32::MAX)
}

/// Does a `model` selector pattern match a context model key? Left-to-right segment
/// match; `_default` is a per-level wildcard; a pattern longer than the context cannot
/// match (it is more specific than the context provides).
fn model_matches(pat: &str, ctx: Option<&str>) -> bool {
    let Some(ctx) = ctx else { return false };
    let pat_segs: Vec<&str> = pat.split('/').collect();
    let ctx_segs: Vec<&str> = ctx.split('/').collect();
    if pat_segs.len() > ctx_segs.len() {
        return false;
    }
    pat_segs
        .iter()
        .zip(ctx_segs.iter())
        .all(|(p, c)| *p == DEFAULT_SEGMENT || p == c)
}

/// Within-band specificity: `(band-primary-axis depth, Σ other-axis depths)`,
/// lexicographic (D3). Leading with the band's own axis means axis-count can never bury
/// an exact-model match, without introducing a global axis ranking.
pub(crate) fn specificity(band: Band, sel: &Selector) -> (u32, u32) {
    let primary = band.primary_axis();
    let primary_depth = primary.map_or(0, |ax| sel.depth_of(ax));
    let other_depth = ALL_AXES
        .iter()
        .filter(|&&ax| Some(ax) != primary)
        .map(|&ax| sel.depth_of(ax))
        .sum();
    (primary_depth, other_depth)
}

/// Does a selector's pinned axes all match the context? An unpinned axis is don't-care;
/// a pinned axis whose context value is absent or different does not match.
pub(crate) fn matches(sel: &Selector, ctx: &ContextVector) -> bool {
    if let Some(h) = &sel.harness
        && ctx.harness.as_deref() != Some(h.as_str())
    {
        return false;
    }
    if let Some(m) = &sel.model
        && !model_matches(m, ctx.model.as_deref())
    {
        return false;
    }
    if let Some(r) = sel.role
        && ctx.role != r
    {
        return false;
    }
    if let Some(a) = sel.arm
        && ctx.arm != Some(a)
    {
        return false;
    }
    if let Some(s) = &sel.stage
        && ctx.stage.as_deref() != Some(s.as_str())
    {
        return false;
    }
    true
}

/// Compose the assembled markdown for a context over a corpus.
///
/// Pipeline (design §5.4): drop sealed disk twins (INV-6) → band filter + selector match
/// → validate & apply `replaces` suppression (INV-3) → order by the precedence key →
/// concatenate bodies (INV-2). A missing tier contributes nothing and is not an error
/// (INV-4). Pure and deterministic (INV-5/INV-7).
pub(crate) fn resolve(
    ctx: &ContextVector,
    corpus: &[Snippet],
    sealed: &SealSet,
) -> Result<String, ResolveError> {
    let active: Vec<&Snippet> = corpus
        .iter()
        .filter(|s| !(s.provenance == Provenance::User && sealed.contains(&s.slot)))
        .filter(|s| ctx.bands.includes(s.slot.band))
        .filter(|s| matches(&s.selector, ctx))
        .collect();

    let suppressed = replaces_suppression(&active)?;

    let mut kept: Vec<&Snippet> = active
        .iter()
        .enumerate()
        .filter(|(i, _)| !suppressed.contains(i))
        .map(|(_, s)| *s)
        .collect();
    kept.sort_by_key(|s| precedence_key(s));

    Ok(kept
        .iter()
        .map(|s| s.body.as_str())
        .collect::<Vec<_>>()
        .join("\n"))
}

/// A validated active replacer: its carrier index, own slot, and the slot it targets.
struct Replacer {
    carrier: usize,
    own: Slot,
    target: Slot,
}

/// Validate the `replaces` graph over the active snippets and return the set of active
/// indices to suppress. Errors (never silent alpha): a non-unique-top replacer, two
/// replacers targeting one slot, or a cross-slot cycle (INV-3).
fn replaces_suppression(active: &[&Snippet]) -> Result<BTreeSet<usize>, ResolveError> {
    let mut replacers: Vec<Replacer> = Vec::new();
    for (i, s) in active.iter().enumerate() {
        if let Some(target) = &s.selector.replaces {
            if !is_unique_top_of_slot(active, s) {
                return Err(ResolveError::NonTopReplacer(s.slot.clone()));
            }
            replacers.push(Replacer {
                carrier: i,
                own: s.slot.clone(),
                target: target.clone(),
            });
        }
    }

    // One replacer per target slot.
    let mut by_target: BTreeMap<Slot, usize> = BTreeMap::new();
    for (i, r) in replacers.iter().enumerate() {
        if by_target.insert(r.target.clone(), i).is_some() {
            return Err(ResolveError::DuplicateTarget(r.target.clone()));
        }
    }

    // Cross-slot edges (own → target where target ≠ own) must be acyclic.
    if let Some(cycle) = find_replaces_cycle(&replacers) {
        return Err(ResolveError::Cycle(cycle));
    }

    // Suppress every active member of a targeted slot except the replacer itself.
    let mut suppressed = BTreeSet::new();
    for r in &replacers {
        for (j, s) in active.iter().enumerate() {
            if s.slot == r.target && j != r.carrier {
                suppressed.insert(j);
            }
        }
    }
    Ok(suppressed)
}

/// Is `me` the unique strict maximum (by precedence key) among active snippets sharing
/// its slot? True iff exactly one active member of the slot has a key ≥ `me`'s (itself).
fn is_unique_top_of_slot(active: &[&Snippet], me: &Snippet) -> bool {
    let my_key = precedence_key(me);
    let at_or_above = active
        .iter()
        .filter(|s| s.slot == me.slot && precedence_key(s) >= my_key)
        .count();
    at_or_above == 1
}

/// Detect a cycle in the cross-slot `replaces` edge graph (self-edges — replacing one's
/// own slot — are excluded). Returns the slots on the cycle, or `None`.
fn find_replaces_cycle(replacers: &[Replacer]) -> Option<Vec<Slot>> {
    // Edge: own slot → target slot (skip self-edges). Out-degree ≤ 1 per source (one
    // replacer per target), so a walk revisiting a node is a cycle.
    let mut adj: BTreeMap<Slot, Slot> = BTreeMap::new();
    for r in replacers {
        if r.own != r.target {
            adj.insert(r.own.clone(), r.target.clone());
        }
    }
    for start in adj.keys() {
        let mut seen: Vec<Slot> = Vec::new();
        let mut node = start.clone();
        loop {
            if let Some(pos) = seen.iter().position(|s| s == &node) {
                return Some(seen.into_iter().skip(pos).collect());
            }
            seen.push(node.clone());
            match adj.get(&node) {
                Some(next) => node = next.clone(),
                None => break,
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── fixture builders ──────────────────────────────────────────────────────

    fn snip(band: Band, label: &str, sel: Selector, prov: Provenance, body: &str) -> Snippet {
        Snippet {
            slot: Slot::new(band, label),
            selector: sel,
            provenance: prov,
            body: body.into(),
        }
    }

    fn ctx(role: Role) -> ContextVector {
        ContextVector {
            role,
            harness: None,
            model: None,
            arm: None,
            stage: None,
            bands: BandFilter::All,
        }
    }

    fn resolve_ok(ctx: &ContextVector, corpus: &[Snippet]) -> String {
        resolve(ctx, corpus, &SealSet::default()).expect("valid corpus resolves")
    }

    // ── band ⇄ segment round-trip, role/arm axes ──────────────────────────────

    #[test]
    fn band_segment_round_trips() {
        for band in ALL_BANDS {
            assert_eq!(Band::from_segment(band.as_str()), Some(band));
        }
        assert_eq!(Band::from_segment("nope"), None);
    }

    #[test]
    fn role_selector_discriminates_orchestrator_from_worker() {
        let corpus = vec![
            snip(
                Band::Role,
                "orchestrator",
                Selector {
                    role: Some(Role::Orchestrator),
                    ..Default::default()
                },
                Provenance::Framework,
                "ORCH",
            ),
            snip(
                Band::Role,
                "worker",
                Selector {
                    role: Some(Role::Worker),
                    ..Default::default()
                },
                Provenance::Framework,
                "WORKER",
            ),
        ];
        assert_eq!(resolve_ok(&ctx(Role::Orchestrator), &corpus), "ORCH");
        assert_eq!(resolve_ok(&ctx(Role::Worker), &corpus), "WORKER");
    }

    #[test]
    fn arm_axis_matches_only_the_pinned_arm() {
        let corpus = vec![snip(
            Band::Project,
            "subprocess-note",
            Selector {
                arm: Some(Arm::Subprocess),
                ..Default::default()
            },
            Provenance::Framework,
            "SUBPROC",
        )];
        let mut sub = ctx(Role::Worker);
        sub.arm = Some(Arm::Subprocess);
        assert_eq!(resolve_ok(&sub, &corpus), "SUBPROC");
        let mut agent = ctx(Role::Worker);
        agent.arm = Some(Arm::Subagent);
        assert_eq!(resolve_ok(&agent, &corpus), "");
    }

    // ── VT-2: specificity() unit table (D3, band-primary-axis) ─────────────────

    #[test]
    fn specificity_model_band_exact_beats_shallow_plus_extra_axes() {
        // exact model (depth 2) → (2, 0)
        let exact = Selector {
            model: Some("anthropic/claude-sonnet-4".into()),
            ..Default::default()
        };
        // shallow model (depth 1) + harness + role → (1, 2) — the finding-3 footgun
        let shallow = Selector {
            model: Some("anthropic".into()),
            harness: Some("claude".into()),
            role: Some(Role::Worker),
            ..Default::default()
        };
        assert_eq!(specificity(Band::Model, &exact), (2, 0));
        assert_eq!(specificity(Band::Model, &shallow), (1, 2));
        // lexicographic: exact-model outranks shallow-model-plus-extras
        assert!(specificity(Band::Model, &exact) > specificity(Band::Model, &shallow));
    }

    #[test]
    fn specificity_default_tail_does_not_add_depth() {
        let vendor_default = Selector {
            model: Some("anthropic/_default".into()),
            ..Default::default()
        };
        let bare_vendor = Selector {
            model: Some("anthropic".into()),
            ..Default::default()
        };
        assert_eq!(specificity(Band::Model, &vendor_default), (1, 0));
        assert_eq!(specificity(Band::Model, &bare_vendor), (1, 0));
    }

    #[test]
    fn specificity_bandless_axis_counts_as_other() {
        // In the harness band, model is a non-primary ("other") axis.
        let sel = Selector {
            harness: Some("claude".into()),
            model: Some("anthropic/claude-sonnet-4".into()),
            ..Default::default()
        };
        // primary=harness depth 1; other = model depth 2 = 2
        assert_eq!(specificity(Band::Harness, &sel), (1, 2));
    }

    #[test]
    fn specificity_preamble_has_no_primary_axis() {
        let sel = Selector {
            harness: Some("claude".into()),
            ..Default::default()
        };
        // preamble primary depth 0; harness counts as other
        assert_eq!(specificity(Band::Preamble, &sel), (0, 1));
    }

    // ── VT-1: engine goldens ───────────────────────────────────────────────────

    #[test]
    fn band_order_is_fixed_regardless_of_label() {
        let corpus = vec![
            snip(
                Band::Project,
                "z",
                Selector::default(),
                Provenance::Framework,
                "PROJECT",
            ),
            snip(
                Band::Preamble,
                "a",
                Selector::default(),
                Provenance::Framework,
                "PREAMBLE",
            ),
            snip(
                Band::Role,
                "worker",
                Selector {
                    role: Some(Role::Worker),
                    ..Default::default()
                },
                Provenance::Framework,
                "ROLE",
            ),
        ];
        assert_eq!(
            resolve_ok(&ctx(Role::Worker), &corpus),
            "PREAMBLE\nROLE\nPROJECT"
        );
    }

    #[test]
    fn specificity_dominates_provenance_framework_exact_beats_user_vendor_default() {
        let mut c = ctx(Role::Worker);
        c.model = Some("anthropic/claude-sonnet-4".into());
        let corpus = vec![
            // user vendor-default (broad): specificity (1,0)
            snip(
                Band::Model,
                "anthropic/_default",
                Selector {
                    model: Some("anthropic/_default".into()),
                    ..Default::default()
                },
                Provenance::User,
                "USER-DEFAULT",
            ),
            // framework exact-model (narrow): specificity (2,0) → later / last word
            snip(
                Band::Model,
                "anthropic/claude-sonnet-4",
                Selector {
                    model: Some("anthropic/claude-sonnet-4".into()),
                    ..Default::default()
                },
                Provenance::Framework,
                "FW-EXACT",
            ),
        ];
        // both match; framework exact is more specific ⇒ ordered last (gets the last word)
        assert_eq!(resolve_ok(&c, &corpus), "USER-DEFAULT\nFW-EXACT");
    }

    #[test]
    fn same_slot_equal_specificity_user_wins_the_provenance_tiebreak() {
        // Framework + user twin at the SAME slot, same selector ⇒ equal specificity;
        // user is ordered last (the legitimate customisation gets the last word).
        let corpus = vec![
            snip(
                Band::Harness,
                "claude",
                Selector {
                    harness: Some("claude".into()),
                    ..Default::default()
                },
                Provenance::Framework,
                "FW",
            ),
            snip(
                Band::Harness,
                "claude",
                Selector {
                    harness: Some("claude".into()),
                    ..Default::default()
                },
                Provenance::User,
                "USER",
            ),
        ];
        let mut c = ctx(Role::Worker);
        c.harness = Some("claude".into());
        assert_eq!(resolve_ok(&c, &corpus), "FW\nUSER");
    }

    #[test]
    fn alpha_breaks_equal_band_specificity_provenance() {
        // Two project-band snippets, no axes ⇒ equal (band, spec, provenance); alpha on
        // the full slot path decides order deterministically.
        let corpus = vec![
            snip(
                Band::Project,
                "zebra",
                Selector::default(),
                Provenance::Framework,
                "Z",
            ),
            snip(
                Band::Project,
                "alpha",
                Selector::default(),
                Provenance::Framework,
                "A",
            ),
        ];
        assert_eq!(resolve_ok(&ctx(Role::Worker), &corpus), "A\nZ");
    }

    #[test]
    fn non_match_is_absence_not_override() {
        // A harness=claude snippet simply does not match a pi context.
        let corpus = vec![
            snip(
                Band::Preamble,
                "u",
                Selector::default(),
                Provenance::Framework,
                "UNIVERSAL",
            ),
            snip(
                Band::Harness,
                "claude",
                Selector {
                    harness: Some("claude".into()),
                    ..Default::default()
                },
                Provenance::Framework,
                "CLAUDE",
            ),
        ];
        let mut c = ctx(Role::Worker);
        c.harness = Some("pi".into());
        assert_eq!(resolve_ok(&c, &corpus), "UNIVERSAL");
    }

    #[test]
    fn missing_tier_degrades_to_empty_no_error() {
        let corpus: Vec<Snippet> = vec![];
        assert_eq!(resolve_ok(&ctx(Role::Worker), &corpus), "");
    }

    #[test]
    fn band_filter_restricts_output() {
        let corpus = vec![
            snip(
                Band::Preamble,
                "u",
                Selector::default(),
                Provenance::Framework,
                "PREAMBLE",
            ),
            snip(
                Band::Role,
                "worker",
                Selector {
                    role: Some(Role::Worker),
                    ..Default::default()
                },
                Provenance::Framework,
                "ROLE",
            ),
        ];
        let mut c = ctx(Role::Worker);
        c.bands = BandFilter::Only([Band::Role].into_iter().collect());
        assert_eq!(resolve_ok(&c, &corpus), "ROLE");
    }

    #[test]
    fn model_default_tail_matches_but_exact_is_not_required() {
        let mut c = ctx(Role::Worker);
        c.model = Some("anthropic/claude-sonnet-4".into());
        // bare vendor and vendor/_default both match anthropic/claude-sonnet-4
        let corpus = vec![
            snip(
                Band::Model,
                "anthropic",
                Selector {
                    model: Some("anthropic".into()),
                    ..Default::default()
                },
                Provenance::Framework,
                "VENDOR",
            ),
            snip(
                Band::Model,
                "anthropic/_default",
                Selector {
                    model: Some("anthropic/_default".into()),
                    ..Default::default()
                },
                Provenance::Framework,
                "VENDOR-DEFAULT",
            ),
        ];
        // equal specificity (1,0); alpha on slot path: "anthropic" < "anthropic/_default"
        assert_eq!(resolve_ok(&c, &corpus), "VENDOR\nVENDOR-DEFAULT");
    }

    #[test]
    fn resolve_is_deterministic_regardless_of_corpus_order() {
        let mk = |prov_order: bool| {
            let a = snip(
                Band::Preamble,
                "a",
                Selector::default(),
                Provenance::Framework,
                "A",
            );
            let z = snip(
                Band::Project,
                "z",
                Selector::default(),
                Provenance::Framework,
                "Z",
            );
            if prov_order { vec![a, z] } else { vec![z, a] }
        };
        assert_eq!(
            resolve_ok(&ctx(Role::Worker), &mk(true)),
            resolve_ok(&ctx(Role::Worker), &mk(false))
        );
    }

    // ── VT-4: seal disk-twin drop (INV-6) ──────────────────────────────────────

    #[test]
    fn sealed_slot_drops_user_disk_twin_but_keeps_framework() {
        let slot = Slot::new(Band::Role, "worker");
        let corpus = vec![
            snip(
                Band::Role,
                "worker",
                Selector {
                    role: Some(Role::Worker),
                    ..Default::default()
                },
                Provenance::Framework,
                "FW-WORKER",
            ),
            // a user hand-created twin at the sealed slot — must be dropped
            snip(
                Band::Role,
                "worker",
                Selector {
                    role: Some(Role::Worker),
                    ..Default::default()
                },
                Provenance::User,
                "USER-SHADOW",
            ),
        ];
        let sealed = SealSet([slot].into_iter().collect());
        let out = resolve(&ctx(Role::Worker), &corpus, &sealed).unwrap();
        assert_eq!(out, "FW-WORKER");
    }

    #[test]
    fn unsealed_slot_keeps_user_twin() {
        // Same corpus, but the slot is NOT sealed ⇒ user twin survives (and wins last word).
        let corpus = vec![
            snip(
                Band::Role,
                "worker",
                Selector {
                    role: Some(Role::Worker),
                    ..Default::default()
                },
                Provenance::Framework,
                "FW-WORKER",
            ),
            snip(
                Band::Role,
                "worker",
                Selector {
                    role: Some(Role::Worker),
                    ..Default::default()
                },
                Provenance::User,
                "USER-WORKER",
            ),
        ];
        assert_eq!(
            resolve_ok(&ctx(Role::Worker), &corpus),
            "FW-WORKER\nUSER-WORKER"
        );
    }

    // ── VT-3: replaces (INV-3) ─────────────────────────────────────────────────

    #[test]
    fn replaces_unique_most_specific_suppresses_lower_in_slot() {
        // A user snippet at slot harness/claude replaces the framework twin (its own slot).
        let slot = Slot::new(Band::Harness, "claude");
        let corpus = vec![
            snip(
                Band::Harness,
                "claude",
                Selector {
                    harness: Some("claude".into()),
                    ..Default::default()
                },
                Provenance::Framework,
                "FW",
            ),
            snip(
                Band::Harness,
                "claude",
                Selector {
                    harness: Some("claude".into()),
                    replaces: Some(slot),
                    ..Default::default()
                },
                Provenance::User,
                "USER-ONLY",
            ),
        ];
        let mut c = ctx(Role::Worker);
        c.harness = Some("claude".into());
        // framework twin suppressed; only the replacer remains
        assert_eq!(resolve_ok(&c, &corpus), "USER-ONLY");
    }

    #[test]
    fn replaces_by_non_top_snippet_is_rejected() {
        // The framework snippet carries replaces, but the user twin is more specific-by-
        // provenance ⇒ the framework snippet is NOT the unique top of its slot.
        let slot = Slot::new(Band::Harness, "claude");
        let corpus = vec![
            snip(
                Band::Harness,
                "claude",
                Selector {
                    harness: Some("claude".into()),
                    replaces: Some(slot),
                    ..Default::default()
                },
                Provenance::Framework,
                "FW",
            ),
            snip(
                Band::Harness,
                "claude",
                Selector {
                    harness: Some("claude".into()),
                    ..Default::default()
                },
                Provenance::User,
                "USER",
            ),
        ];
        let mut c = ctx(Role::Worker);
        c.harness = Some("claude".into());
        let err = resolve(&c, &corpus, &SealSet::default()).unwrap_err();
        assert_eq!(
            err,
            ResolveError::NonTopReplacer(Slot::new(Band::Harness, "claude"))
        );
    }

    #[test]
    fn two_active_replaces_targeting_one_slot_is_rejected() {
        // Two distinct snippets (each unique-top of its own slot) both replace project/target.
        let target = Slot::new(Band::Project, "target");
        let corpus = vec![
            snip(
                Band::Project,
                "a",
                Selector {
                    replaces: Some(target.clone()),
                    ..Default::default()
                },
                Provenance::Framework,
                "A",
            ),
            snip(
                Band::Project,
                "b",
                Selector {
                    replaces: Some(target.clone()),
                    ..Default::default()
                },
                Provenance::Framework,
                "B",
            ),
            snip(
                Band::Project,
                "target",
                Selector::default(),
                Provenance::Framework,
                "TARGET",
            ),
        ];
        let err = resolve(&ctx(Role::Worker), &corpus, &SealSet::default()).unwrap_err();
        assert_eq!(err, ResolveError::DuplicateTarget(target));
    }

    #[test]
    fn replaces_cycle_is_rejected() {
        // project/a replaces slot b; project/b replaces slot a ⇒ mutual suppression cycle.
        let a = Slot::new(Band::Project, "a");
        let b = Slot::new(Band::Project, "b");
        let corpus = vec![
            snip(
                Band::Project,
                "a",
                Selector {
                    replaces: Some(b.clone()),
                    ..Default::default()
                },
                Provenance::Framework,
                "A",
            ),
            snip(
                Band::Project,
                "b",
                Selector {
                    replaces: Some(a.clone()),
                    ..Default::default()
                },
                Provenance::Framework,
                "B",
            ),
        ];
        let err = resolve(&ctx(Role::Worker), &corpus, &SealSet::default()).unwrap_err();
        match err {
            ResolveError::Cycle(slots) => {
                assert_eq!(slots.len(), 2);
                assert!(slots.contains(&a) && slots.contains(&b));
            }
            other => panic!("expected cycle, got {other:?}"),
        }
    }

    #[test]
    fn replaces_own_slot_self_edge_is_not_a_cycle() {
        // The common case: a top snippet replacing its own slot is a self-edge, allowed.
        let slot = Slot::new(Band::Project, "only");
        let corpus = vec![
            snip(
                Band::Project,
                "only",
                Selector {
                    replaces: Some(slot),
                    ..Default::default()
                },
                Provenance::Framework,
                "ONLY",
            ),
            snip(
                Band::Project,
                "other",
                Selector::default(),
                Provenance::Framework,
                "OTHER",
            ),
        ];
        assert_eq!(resolve_ok(&ctx(Role::Worker), &corpus), "ONLY\nOTHER");
    }
}
