// SPDX-License-Identifier: GPL-3.0-only
//! The dispatch funnel state machine — pure leaf (SL-228 PHASE-03, design §2).
//!
//! THE legality authority for the phase funnel: `Position`, `Transition`,
//! `attempt`, `preflight`, `expected_next`, and the `IllegalTransition` refusal
//! payload, all derived from ONE `const` transition table. That same table
//! renders the D7 golden artifact at `.doctrine/spec/tech/021/funnel-machine.md`,
//! which a golden test pins byte-for-byte.
//!
//! ADR-001 tier: **leaf** (`.doctrine/adr/001/layering.toml`). No git, disk,
//! clock, or engine imports — std plus the serde *derive* (the `boundary` leaf
//! precedent: a data-shape derive is neither IO nor domain knowledge, and the
//! module's crate out-degree stays 0). Facts are gathered by the engine shell
//! and passed in as `TransitionFacts` (CLAUDE.md pure/imperative split).

use std::fmt;

use serde::{Deserialize, Serialize};

// ======================================================================================
// Vocabulary — every state/transition/refusal token is single-sourced here (STD-001).
// ======================================================================================

/// A phase's funnel position: the durable, per-phase run-state (design §2/§3).
///
/// The declaration order IS the advance order (`Spawned < … < Reaped`), and the
/// derived `Ord` is read directly by the replay logic — a transition whose
/// milestone is at or behind `current` can never advance the phase (invariant I3:
/// positions only advance; evidence may update in place).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Position {
    Spawned,
    WorkerCommitted,
    Imported,
    Verified,
    Concluded,
    Reaped,
}

impl Position {
    /// The kebab token this position is spelled with everywhere — the record's
    /// `position` value, the refusal text, the D7 artifact (STD-001).
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Position::Spawned => "spawned",
            Position::WorkerCommitted => "worker-committed",
            Position::Imported => "imported",
            Position::Verified => "verified",
            Position::Concluded => "concluded",
            Position::Reaped => "reaped",
        }
    }

    /// The `already-<position>` refusal token for a milestone this phase has
    /// already passed. A test pins each against `already-{as_str}` so the family
    /// cannot drift from [`Position::as_str`].
    const fn already_token(self) -> &'static str {
        match self {
            Position::Spawned => "already-spawned",
            Position::WorkerCommitted => "already-worker-committed",
            Position::Imported => "already-imported",
            Position::Verified => "already-verified",
            Position::Concluded => "already-concluded",
            Position::Reaped => "already-reaped",
        }
    }
}

/// How a position is spelled when the phase has no row at all (pre-spawn).
const NO_POSITION: &str = "none";

/// The kind of a transition, without its payload. The pure PRE-ACT gate
/// ([`preflight`]) speaks in kinds because a verb whose evidence only exists
/// *after* acting cannot construct a full [`Transition`] beforehand (RV-304 F-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransitionKind {
    Spawn,
    RecordWorkerCommit,
    Import,
    Verify,
    Conclude,
    Reap,
}

impl TransitionKind {
    /// The kebab token this transition is spelled with (STD-001).
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            TransitionKind::Spawn => "spawn",
            TransitionKind::RecordWorkerCommit => "record-worker-commit",
            TransitionKind::Import => "import",
            TransitionKind::Verify => "verify",
            TransitionKind::Conclude => "conclude",
            TransitionKind::Reap => "reap",
        }
    }

    /// The position this kind lands at on its success arm, read off the ONE
    /// table's result column (uniform per kind — a `Verify` pass always lands
    /// `Verified`). `None` only if the kind is absent from the table.
    fn milestone(self) -> Option<Position> {
        TABLE
            .iter()
            .find(|r| matches!(r.transition, Some(k) if k == self))
            .and_then(|r| r.landing.pass_position())
    }
}

/// The machine's prescription from a position — what the operator should do next.
/// One vocabulary shared by refusals and (PHASE-06) the `next` oracle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Expected {
    Spawn,
    /// The fork is armed; the next event is the worker's own commit, not an
    /// orchestrator verb.
    AwaitWorker,
    Import,
    Verify,
    /// Red stored evidence: re-running the suite unchanged is not the next move.
    Triage,
    Conclude,
    Reap,
    Terminal,
}

impl Expected {
    /// The kebab token this prescription is rendered with (STD-001).
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Expected::Spawn => "spawn",
            Expected::AwaitWorker => "await-worker",
            Expected::Import => "import",
            Expected::Verify => "verify",
            Expected::Triage => "triage",
            Expected::Conclude => "conclude",
            Expected::Reap => "reap",
            Expected::Terminal => "terminal",
        }
    }
}

// --- refusal tokens (STD-001 named consts) --------------------------------------------

/// No row: the phase has never been spawned.
const NOT_SPAWNED: &str = "not-spawned";
/// The fork exists but has not landed its commit.
const WORKER_NOT_COMMITTED: &str = "worker-not-committed";
/// The worker delta is not on the coordination tip yet.
const NOT_IMPORTED: &str = "not-imported";
/// Conclude attempted with no verify evidence at all.
const CONCLUDE_UNVERIFIED: &str = "conclude-unverified";
/// Conclude attempted against red stored evidence.
const CONCLUDE_VERIFY_FAILED: &str = "conclude-verify-failed";
/// Conclude attempted after a non-funnel-record path changed since the verified
/// tip — the D3 modulo-funnel-record gate, NOT bare `verified_oid != tip`.
const CONCLUDE_VERIFY_STALE: &str = "conclude-verify-stale";
/// Reap attempted before the phase boundary was concluded.
const NOT_CONCLUDED: &str = "not-concluded";
/// The phase is reaped; no transition remains.
const TERMINAL: &str = "terminal";
/// How the `already-<position>` family is spelled in the rendered artifact; the
/// live tokens come from [`Position::already_token`].
const ALREADY_FAMILY: &str = "already-<position>";

/// The refusal vocabulary paired with the condition that raises it — the source of
/// the D7 artifact's token table. Every entry is a named const above (STD-001).
const REASON_DOC: &[(&str, &str)] = &[
    (NOT_SPAWNED, "the phase has no fork yet"),
    (WORKER_NOT_COMMITTED, "the fork has not landed its commit"),
    (
        NOT_IMPORTED,
        "the worker delta is not on the coordination tip",
    ),
    (CONCLUDE_UNVERIFIED, "no verify evidence recorded"),
    (CONCLUDE_VERIFY_FAILED, "the stored verify evidence is red"),
    (
        CONCLUDE_VERIFY_STALE,
        "non-funnel-record paths changed since the verified tip",
    ),
    (NOT_CONCLUDED, "the phase boundary is not recorded"),
    (
        ALREADY_FAMILY,
        "replay attempted with mismatched identity facts",
    ),
    (TERMINAL, "the phase is reaped; no transition remains"),
];

// ======================================================================================
// The record shape (design §3) — pure data, owned by the leaf so the machine and its
// sole writer share ONE row type (the writer adds the git/IO half, never a mirror).
// ======================================================================================

/// The verify verdict. `Fail` keeps the position and updates the evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum VerifyStatus {
    Pass,
    Fail,
}

impl VerifyStatus {
    /// The token this verdict is stored/rendered as (STD-001).
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            VerifyStatus::Pass => "pass",
            VerifyStatus::Fail => "fail",
        }
    }
}

/// The evidence a `Verify` transition carries and the record stores: the verdict,
/// the coord tip the suite ran against, the suite name, and the shell-supplied
/// timestamp. Replay identity is `(status, verified_oid)` only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct VerifyEvidence {
    pub status: VerifyStatus,
    pub verified_oid: String,
    pub suite: String,
    pub at: String,
}

/// `[phase.spawn]` — the fork armed for this phase and the base it forked from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SpawnRecord {
    pub fork: String,
    pub base_oid: String,
    pub at: String,
}

/// `[phase.worker_commit]` — the fork tip the worker landed (Class 2: recorded
/// server-side, strictly after the fork ref advanced).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct WorkerCommitRecord {
    pub fork_tip: String,
    pub at: String,
}

/// `[phase.import]` — the fork commit imported (`fork_tip`, the replay-identity
/// key) and the coord tip it was composed onto (`onto`, stored provenance only).
/// Deliberately NO `import_oid`: the row rides the import commit's own tree, so
/// the commit cannot name itself (RV-304 F-2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ImportRecord {
    pub fork_tip: String,
    pub onto: String,
    pub at: String,
}

/// `[phase.conclude]` / `[phase.reap]` — a bare timestamp; the boundary oids live
/// in `boundaries.toml` (D1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StampRecord {
    pub at: String,
}

/// One phase's row in the funnel record: where it stands, when it last moved, and
/// the per-transition provenance filled in as each transition is reached.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PhaseRow {
    pub id: String,
    pub position: Position,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spawn: Option<SpawnRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worker_commit: Option<WorkerCommitRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub import: Option<ImportRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verify: Option<VerifyEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conclude: Option<StampRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reap: Option<StampRecord>,
}

// ======================================================================================
// Transitions + facts
// ======================================================================================

/// A proposed transition and its typed CANDIDATE facts. The machine — not the
/// shell — decides the result position and act-vs-replay (RV-303 F-2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Transition {
    Spawn {
        fork: String,
        base_oid: String,
    },
    RecordWorkerCommit {
        fork_tip: String,
    },
    /// INPUTS only: `fork_tip` is the replay-identity key, `onto` is stored
    /// provenance and is EXCLUDED from replay identity (RV-304 F-1) — a
    /// lost-response retry re-resolves a fresh `onto`.
    Import {
        fork_tip: String,
        onto: String,
    },
    /// `Pass` ⇒ `Verified`; `Fail` ⇒ the position is kept, the evidence updated.
    Verify {
        evidence: VerifyEvidence,
    },
    Conclude,
    Reap,
}

impl Transition {
    /// This transition's kind (its payload-free identity).
    pub(crate) const fn kind(&self) -> TransitionKind {
        match self {
            Transition::Spawn { .. } => TransitionKind::Spawn,
            Transition::RecordWorkerCommit { .. } => TransitionKind::RecordWorkerCommit,
            Transition::Import { .. } => TransitionKind::Import,
            Transition::Verify { .. } => TransitionKind::Verify,
            Transition::Conclude => TransitionKind::Conclude,
            Transition::Reap => TransitionKind::Reap,
        }
    }

    /// The position this transition would land at from `current` — the table's
    /// result column resolved for a verdict. A failed `Verify` lands where the
    /// phase already stands, which is exactly what makes a repeated red verify a
    /// replay rather than a fresh act.
    fn target(&self, current: Option<Position>) -> Option<Position> {
        match self {
            Transition::Verify { evidence } if evidence.status == VerifyStatus::Fail => current,
            _ => self.kind().milestone(),
        }
    }
}

/// The STORED facts the shell gathers for a transition attempt. Pure data.
#[derive(Debug)]
pub(crate) struct TransitionFacts<'a> {
    /// The row as last landed (`None` ⇔ no row, pre-spawn).
    pub row: Option<&'a PhaseRow>,
    /// The live coord tip at gate time.
    pub coord_tip: &'a str,
    /// Paths changed in `verified_oid‥coord_tip`, shell-supplied. `None` means the
    /// shell established NO tree identity, so conclude fails closed with
    /// `conclude-verify-stale`.
    pub paths_since_verify: Option<&'a [String]>,
}

/// The FR-009 refusal payload, surfaced VERBATIM by verbs and rendered by `next`:
/// the refusal text IS the recovery procedure. `attempted` is the KIND, not the
/// full payload, so [`preflight`] can build it before the evidence exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IllegalTransition {
    pub current: Option<Position>,
    pub attempted: TransitionKind,
    pub expected: Expected,
    pub reason: &'static str,
}

impl fmt::Display for IllegalTransition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} refused at position `{}` — {}; expected: {}",
            self.attempted.as_str(),
            self.current.map_or(NO_POSITION, Position::as_str),
            self.reason,
            self.expected.as_str()
        )
    }
}

/// An approved transition: where the phase now stands, and whether this was an
/// idempotent no-op REPLAY (nothing to land) rather than a real advance. The sole
/// writer needs the distinction — landing a replay would mint a second commit for
/// a transition that already happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Advance {
    pub position: Position,
    pub replay: bool,
}

// ======================================================================================
// THE table — the single source `attempt`, `preflight`, `expected_next` and the D7
// artifact all derive from. A second table anywhere is the drift D7 exists to prevent.
// ======================================================================================

/// Where a legal transition lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Landing {
    /// Deterministic: this position, whatever the verdict.
    At(Position),
    /// Verdict-split: a pass advances, a fail keeps the position.
    ByVerdict { pass: Position, fail: Position },
    /// The row admits no transition at all (the terminal position).
    Terminal,
}

impl Landing {
    /// The position the success arm lands at; `None` for a terminal row.
    fn pass_position(self) -> Option<Position> {
        match self {
            Landing::At(p) | Landing::ByVerdict { pass: p, .. } => Some(p),
            Landing::Terminal => None,
        }
    }

    /// The artifact's `result` column.
    fn label(self) -> String {
        match self {
            Landing::At(p) => p.as_str().to_owned(),
            Landing::ByVerdict { pass, fail } => {
                format!("{} on pass, {} on fail", pass.as_str(), fail.as_str())
            }
            Landing::Terminal => Expected::Terminal.as_str().to_owned(),
        }
    }
}

/// One row of the normative transition table (design §2).
#[derive(Debug)]
struct Row {
    /// The position the row applies to; `None` ⇔ no row yet (pre-spawn).
    current: Option<Position>,
    /// The legal transition from `current`; `None` on the terminal row.
    transition: Option<TransitionKind>,
    /// The facts gate, as rendered in the artifact (`-` when there is none).
    gate: &'static str,
    landing: Landing,
    /// The prescription for `current` — set on EXACTLY ONE row per `current` (a
    /// test pins that), which is what makes `expected_next` a projection of this
    /// table rather than a second one.
    prescribes: Option<Expected>,
}

/// The artifact's "no gate" cell.
const GATE_NONE: &str = "-";

/// THE transition table (design §2, normative). Row order is the artifact's order.
const TABLE: &[Row] = &[
    Row {
        current: None,
        transition: Some(TransitionKind::Spawn),
        gate: GATE_NONE,
        landing: Landing::At(Position::Spawned),
        prescribes: Some(Expected::Spawn),
    },
    Row {
        current: Some(Position::Spawned),
        transition: Some(TransitionKind::RecordWorkerCommit),
        gate: GATE_NONE,
        landing: Landing::At(Position::WorkerCommitted),
        prescribes: Some(Expected::AwaitWorker),
    },
    Row {
        current: Some(Position::WorkerCommitted),
        transition: Some(TransitionKind::Import),
        gate: GATE_NONE,
        landing: Landing::At(Position::Imported),
        prescribes: Some(Expected::Import),
    },
    Row {
        current: Some(Position::Imported),
        transition: Some(TransitionKind::Verify),
        gate: "evidence recorded either way",
        landing: Landing::ByVerdict {
            pass: Position::Verified,
            fail: Position::Imported,
        },
        prescribes: Some(Expected::Verify),
    },
    Row {
        current: Some(Position::Verified),
        transition: Some(TransitionKind::Verify),
        gate: "evidence refreshed",
        landing: Landing::At(Position::Verified),
        prescribes: None,
    },
    Row {
        current: Some(Position::Verified),
        transition: Some(TransitionKind::Conclude),
        gate: "pass evidence, tree identical modulo funnel record",
        landing: Landing::At(Position::Concluded),
        prescribes: Some(Expected::Conclude),
    },
    Row {
        current: Some(Position::Concluded),
        transition: Some(TransitionKind::Reap),
        gate: "landed-oracle ok or fork already absent",
        landing: Landing::At(Position::Reaped),
        prescribes: Some(Expected::Reap),
    },
    Row {
        current: Some(Position::Reaped),
        transition: None,
        gate: GATE_NONE,
        landing: Landing::Terminal,
        prescribes: Some(Expected::Terminal),
    },
];

/// The table row for `(current, kind)`, if that pair is legal.
fn row_for(current: Option<Position>, kind: TransitionKind) -> Option<&'static Row> {
    TABLE
        .iter()
        .find(|r| r.current == current && matches!(r.transition, Some(k) if k == kind))
}

/// The table's prescription column for `current` — the factless base of
/// [`expected_next`]. Total over the positions the table covers.
fn prescription(current: Option<Position>) -> Expected {
    TABLE
        .iter()
        .filter(|r| r.current == current)
        .find_map(|r| r.prescribes)
        .unwrap_or(Expected::Terminal)
}

// ======================================================================================
// The authority
// ======================================================================================

/// THE legality authority (design §2). `current == None` ⇔ no row (pre-spawn).
/// A thin projection of [`attempt_advance`] — same table, same rules, minus the
/// act-vs-replay bit only the writer needs.
#[expect(
    clippy::needless_pass_by_value,
    reason = "design §2 pins the by-value signature; `attempt_advance` is the by-ref core"
)]
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the sole writer needs the act-vs-replay bit, so it calls `attempt_advance`; the by-value projection is the design's published seam for PHASE-04/05 verbs"
    )
)]
pub(crate) fn attempt(
    current: Option<Position>,
    t: Transition,
    facts: &TransitionFacts<'_>,
) -> Result<Position, IllegalTransition> {
    attempt_advance(current, &t, facts).map(|a| a.position)
}

/// The full verdict: the landed position PLUS whether this attempt was an
/// idempotent replay (design §2 "Idempotent replay"). Replay identity is each
/// transition's STABLE INPUT FACTS only — `fork_tip` for `RecordWorkerCommit` and
/// `Import` (`onto` excluded), `(status, verified_oid)` for `Verify` — compared
/// against the STORED row, never against the position.
pub(crate) fn attempt_advance(
    current: Option<Position>,
    t: &Transition,
    facts: &TransitionFacts<'_>,
) -> Result<Advance, IllegalTransition> {
    let kind = t.kind();

    // Terminal first: a reaped phase admits only its own replay.
    if current == Some(Position::Reaped) && kind != TransitionKind::Reap {
        return Err(refuse(current, kind, Some(facts), TERMINAL));
    }

    // Replay: the transition targets exactly where the phase already stands AND
    // its stable input facts match what is stored. Reported Ok, never refused.
    if let Some(position) = current
        && t.target(current) == current
        && replay_identity_matches(t, facts.row)
    {
        return Ok(Advance {
            position,
            replay: true,
        });
    }

    // A legal forward move?
    let Some(row) = row_for(current, kind) else {
        let token = reason(current, kind, Some(facts));
        return Err(refuse(current, kind, Some(facts), token));
    };

    // The facts gate, on the one row that carries a semantic one.
    if kind == TransitionKind::Conclude && !conclude_allowed(facts) {
        return Err(refuse(current, kind, Some(facts), conclude_reason(facts)));
    }

    let position = match row.landing {
        Landing::At(p) => p,
        Landing::ByVerdict { pass, fail } => match t {
            Transition::Verify { evidence } if evidence.status == VerifyStatus::Fail => fail,
            _ => pass,
        },
        // Unreachable by construction: a terminal row carries no transition, so
        // `row_for` never yields it. Refusing (rather than panicking) keeps the
        // machine total over its inputs.
        Landing::Terminal => return Err(refuse(current, kind, Some(facts), TERMINAL)),
    };
    Ok(Advance {
        position,
        replay: false,
    })
}

/// The result of folding an ORDERED transition list from one stored row — the
/// pure half of the sole writer's multi-transition splice (SL-228 PHASE-05, D1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Fold {
    /// The row after every ACCEPTED advance was applied, in order. `None` only when
    /// there was no stored row and every element replayed (⇒ nothing to write).
    pub row: Option<PhaseRow>,
    /// Where the phase stands after the last element.
    pub position: Position,
    /// EVERY element was an idempotent replay — the writer must land NOTHING.
    pub replay: bool,
}

/// Fold `ts` IN ORDER from the stored row, gating each element with
/// [`attempt_advance`] against the RUNNING position/row — the pure core of the
/// heal-forward splice (design §4/§5, D1). Pure: no git, no clock; `at` is the
/// shell-supplied stamp [`apply`] writes.
///
/// ALL-OR-NONE by signature: the first refusal short-circuits with the machine's
/// own [`IllegalTransition`], so a caller that only writes on `Ok` can never land a
/// partial prefix. Each element sees the row its predecessors produced, which is
/// what makes `[Spawn, RecordWorkerCommit, Import]` legal from no row at all while
/// `[Import]` alone is still `not-spawned`.
///
/// `Ok(None)` ⇔ `ts` was EMPTY — there is no position to report and nothing to
/// land; the caller decides whether an empty list is a programming error.
pub(crate) fn fold_transitions(
    stored: Option<&PhaseRow>,
    id: &str,
    ts: &[Transition],
    coord_tip: &str,
    paths_since_verify: Option<&[String]>,
    at: &str,
) -> Result<Option<Fold>, IllegalTransition> {
    let mut row = stored.cloned();
    let mut position = stored.map(|r| r.position);
    let mut replay = true;
    for t in ts {
        let facts = TransitionFacts {
            row: row.as_ref(),
            coord_tip,
            paths_since_verify,
        };
        let advance = attempt_advance(position, t, &facts)?;
        if !advance.replay {
            row = Some(apply(row.as_ref(), id, t, advance.position, at));
            replay = false;
        }
        position = Some(advance.position);
    }
    Ok(position.map(|position| Fold {
        row,
        position,
        replay,
    }))
}

/// Kind-level legality — the pure PRE-ACT gate for verbs whose evidence exists
/// only after acting (`dispatch verify`: status/oid are post-suite facts, so a
/// full `Transition::Verify{evidence}` cannot be constructed pre-act, RV-304
/// F-3). Derived from the SAME table as [`attempt`] (its kind column), never a
/// second one. It is FACTLESS by signature, so it neither sees nor decides
/// replay: [`attempt`] remains the sole authority at landing.
pub(crate) fn preflight(
    current: Option<Position>,
    kind: TransitionKind,
) -> Result<(), IllegalTransition> {
    if row_for(current, kind).is_some() {
        return Ok(());
    }
    let token = reason(current, kind, None);
    Err(refuse(current, kind, None, token))
}

/// The expected-next map (design §2), shared by refusals and the `next` oracle —
/// the table's prescription column plus the two facts-conditional overrides: red
/// stored evidence prescribes triage, and a stale tree at `verified` reverts to
/// verify.
pub(crate) fn expected_next(current: Option<Position>, facts: &TransitionFacts<'_>) -> Expected {
    match current {
        Some(Position::Imported | Position::Verified) if stored_verify_is_red(facts) => {
            Expected::Triage
        }
        Some(Position::Verified) if !conclude_allowed(facts) => Expected::Verify,
        _ => prescription(current),
    }
}

/// Build the refusal payload. `facts` is `None` on the factless [`preflight`] path,
/// where the prescription falls back to the table column alone.
fn refuse(
    current: Option<Position>,
    attempted: TransitionKind,
    facts: Option<&TransitionFacts<'_>>,
    reason: &'static str,
) -> IllegalTransition {
    IllegalTransition {
        current,
        attempted,
        expected: facts.map_or_else(|| prescription(current), |f| expected_next(current, f)),
        reason,
    }
}

/// The refusal token for an illegal `(current, kind)` pair: the EARLIEST unmet
/// prerequisite from `current`, except for the milestone-already-passed family
/// (`already-<position>`) and conclude's own three evidence refusals.
fn reason(
    current: Option<Position>,
    kind: TransitionKind,
    facts: Option<&TransitionFacts<'_>>,
) -> &'static str {
    let Some(cur) = current else {
        return NOT_SPAWNED;
    };
    if cur == Position::Reaped {
        return TERMINAL;
    }
    // A milestone at or behind `current` was already passed: either a replay whose
    // identity facts did not match, or a backward attempt.
    if kind.milestone().is_some_and(|m| m <= cur) {
        return cur.already_token();
    }
    match cur {
        Position::Spawned => WORKER_NOT_COMMITTED,
        Position::WorkerCommitted => NOT_IMPORTED,
        Position::Imported | Position::Verified => match kind {
            TransitionKind::Conclude => facts.map_or(CONCLUDE_UNVERIFIED, conclude_reason),
            _ => NOT_CONCLUDED,
        },
        // At `concluded` the only forward kind is `reap`, which the table admits,
        // so this arm is unreachable; `not-concluded` is the honest fallback.
        Position::Concluded | Position::Reaped => NOT_CONCLUDED,
    }
}

/// Whether conclude's facts gate is satisfied: pass evidence AND tree identity
/// with the verified tip MODULO the funnel record (D3 / RV-303 F-1) — either the
/// tip never moved off `verified_oid`, or every path that changed since is the
/// record itself, so the evidence commit (a funnel-record-only child of the tested
/// tree) never self-stales. With the tip moved and NO path list supplied the shell
/// established no tree identity at all, so the gate fails CLOSED.
fn conclude_allowed(facts: &TransitionFacts<'_>) -> bool {
    let Some(evidence) = stored_verify(facts) else {
        return false;
    };
    if evidence.status != VerifyStatus::Pass {
        return false;
    }
    evidence.verified_oid == facts.coord_tip
        || facts
            .paths_since_verify
            .is_some_and(|paths| paths.iter().all(|p| is_funnel_record(p)))
}

/// Which of conclude's three distinct refusals applies (reached only when
/// [`conclude_allowed`] is false).
fn conclude_reason(facts: &TransitionFacts<'_>) -> &'static str {
    match stored_verify(facts) {
        None => CONCLUDE_UNVERIFIED,
        Some(e) if e.status == VerifyStatus::Fail => CONCLUDE_VERIFY_FAILED,
        Some(_) => CONCLUDE_VERIFY_STALE,
    }
}

/// The stored verify evidence, if any.
fn stored_verify<'a>(facts: &'a TransitionFacts<'_>) -> Option<&'a VerifyEvidence> {
    facts.row.and_then(|r| r.verify.as_ref())
}

/// Whether the stored evidence is red.
fn stored_verify_is_red(facts: &TransitionFacts<'_>) -> bool {
    stored_verify(facts).is_some_and(|e| e.status == VerifyStatus::Fail)
}

/// Replay identity (design §2): each transition's STABLE INPUT FACTS compared
/// against the stored row. `Import`'s `onto` is deliberately excluded — a
/// lost-response retry re-resolves a fresh `onto`, and keying replay on it would
/// refuse the very retry REQ-384 promises (RV-304 F-1). `Conclude`/`Reap` carry no
/// input facts, so standing at their own position IS the identity.
fn replay_identity_matches(t: &Transition, row: Option<&PhaseRow>) -> bool {
    match t {
        Transition::Spawn { fork, base_oid } => row
            .and_then(|r| r.spawn.as_ref())
            .is_some_and(|s| &s.fork == fork && &s.base_oid == base_oid),
        Transition::RecordWorkerCommit { fork_tip } => row
            .and_then(|r| r.worker_commit.as_ref())
            .is_some_and(|w| &w.fork_tip == fork_tip),
        Transition::Import { fork_tip, .. } => row
            .and_then(|r| r.import.as_ref())
            .is_some_and(|i| &i.fork_tip == fork_tip),
        Transition::Verify { evidence } => row.and_then(|r| r.verify.as_ref()).is_some_and(|v| {
            v.status == evidence.status && v.verified_oid == evidence.verified_oid
        }),
        Transition::Conclude | Transition::Reap => true,
    }
}

/// Fold an approved transition into the phase's row: the new position, the
/// shell-supplied `at` as `updated_at`, and the transition's own provenance
/// sub-table. Pure — the clock is an input (CLAUDE.md pure/imperative split).
pub(crate) fn apply(
    row: Option<&PhaseRow>,
    id: &str,
    t: &Transition,
    landed: Position,
    at: &str,
) -> PhaseRow {
    let mut out = row.cloned().unwrap_or_else(|| PhaseRow {
        id: id.to_owned(),
        position: landed,
        updated_at: at.to_owned(),
        spawn: None,
        worker_commit: None,
        import: None,
        verify: None,
        conclude: None,
        reap: None,
    });
    id.clone_into(&mut out.id);
    out.position = landed;
    at.clone_into(&mut out.updated_at);
    match t {
        Transition::Spawn { fork, base_oid } => {
            out.spawn = Some(SpawnRecord {
                fork: fork.clone(),
                base_oid: base_oid.clone(),
                at: at.to_owned(),
            });
        }
        Transition::RecordWorkerCommit { fork_tip } => {
            out.worker_commit = Some(WorkerCommitRecord {
                fork_tip: fork_tip.clone(),
                at: at.to_owned(),
            });
        }
        Transition::Import { fork_tip, onto } => {
            out.import = Some(ImportRecord {
                fork_tip: fork_tip.clone(),
                onto: onto.clone(),
                at: at.to_owned(),
            });
        }
        Transition::Verify { evidence } => out.verify = Some(evidence.clone()),
        Transition::Conclude => out.conclude = Some(StampRecord { at: at.to_owned() }),
        Transition::Reap => out.reap = Some(StampRecord { at: at.to_owned() }),
    }
    out
}

// ======================================================================================
// The funnel record's path — single-sourced here because the conclude gate has to
// RECOGNISE the record (STD-001) and the sole writer has to WRITE it.
// ======================================================================================

/// The committed dispatch run-state directory, relative to the repo root.
const DISPATCH_DIR: &str = ".doctrine/dispatch";
/// The funnel record's file name.
const FUNNEL_FILE: &str = "funnel.toml";

/// The funnel record path for a slice: `.doctrine/dispatch/<NNN>/funnel.toml`.
pub(crate) fn funnel_record_path(slice: u32) -> String {
    format!("{DISPATCH_DIR}/{slice:03}/{FUNNEL_FILE}")
}

/// Whether a repo-relative path IS a funnel record — matched on path COMPONENTS
/// (`.doctrine` / `dispatch` / a 3-digit slice / `funnel.toml`), never a substring,
/// so neither `.doctrine/dispatch/001/funnel.toml.bak` nor a `not-dispatch/`
/// sibling can slip through conclude's tree-identity gate.
pub(crate) fn is_funnel_record(path: &str) -> bool {
    let mut segments = path.split('/');
    for want in DISPATCH_DIR.split('/') {
        if segments.next() != Some(want) {
            return false;
        }
    }
    let Some(slice) = segments.next() else {
        return false;
    };
    if slice.len() != 3 || !slice.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    segments.next() == Some(FUNNEL_FILE) && segments.next().is_none()
}

// ======================================================================================
// D7 — the golden artifact, rendered FROM the table above.
// ======================================================================================

/// The committed artifact this renderer is pinned to, relative to the repo root.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the D7 golden test is this path's consumer (it reads the artifact at runtime)"
    )
)]
pub(crate) const ARTIFACT_PATH: &str = ".doctrine/spec/tech/021/funnel-machine.md";

/// Render the D7 artifact from [`TABLE`] + [`REASON_DOC`]. The prose is fixed; every
/// table row and every diagram edge is derived, so a table edit that is not
/// re-rendered fails the golden test — structure pinned mechanically, semantics
/// governed socially (D7).
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the D7 golden test is the renderer's consumer; re-rendering is a dev act, not a shipped verb"
    )
)]
pub(crate) fn render_artifact() -> String {
    let mut out = String::from(ARTIFACT_HEADER);

    out.push_str(TRANSITIONS_OPEN);
    out.extend(TABLE.iter().map(|row| {
        format!(
            "| {} | {} | {} | {} |\n",
            row.current.map_or(NO_POSITION, Position::as_str),
            row.transition.map_or(GATE_NONE, TransitionKind::as_str),
            row.gate,
            row.landing.label(),
        )
    }));

    out.push_str(EXPECTED_OPEN);
    out.extend(TABLE.iter().filter_map(|row| {
        row.prescribes.map(|expected| {
            format!(
                "| {} | {} |\n",
                row.current.map_or(NO_POSITION, Position::as_str),
                expected.as_str(),
            )
        })
    }));

    out.push_str(REASONS_OPEN);
    out.extend(
        REASON_DOC
            .iter()
            .map(|(token, raised)| format!("| {token} | {raised} |\n")),
    );

    out.push_str(DIAGRAM_OPEN);
    out.extend(TABLE.iter().map(edges));
    out.push_str(DIAGRAM_CLOSE);
    out
}

/// The mermaid edge(s) a table row contributes: a verdict-split row draws both
/// arms, the terminal row draws the exit.
fn edges(row: &Row) -> String {
    let from = row.current.map_or_else(|| ENTRY_NODE.to_owned(), node);
    match (row.transition, row.landing) {
        (Some(kind), Landing::At(to)) => {
            format!("    {from} --> {}: {}\n", node(to), kind.as_str())
        }
        (Some(kind), Landing::ByVerdict { pass, fail }) => format!(
            "    {from} --> {}: {} ({})\n    {from} --> {}: {} ({})\n",
            node(pass),
            kind.as_str(),
            VerifyStatus::Pass.as_str(),
            node(fail),
            kind.as_str(),
            VerifyStatus::Fail.as_str(),
        ),
        _ => format!("    {from} --> {ENTRY_NODE}\n"),
    }
}

/// A position's mermaid node id (kebab is not a legal identifier there).
fn node(p: Position) -> String {
    p.as_str().replace('-', "_")
}

/// mermaid's start/end pseudo-node.
const ENTRY_NODE: &str = "[*]";

/// The artifact's fixed preamble — the do-not-hand-edit banner + title + intro.
const ARTIFACT_HEADER: &str = "\
<!-- GENERATED ARTIFACT — SL-228 D7. Rendered from the single `const` transition
     table in `src/funnel_machine.rs`. Do NOT hand-edit: a golden test pins this
     file to the code byte-for-byte. To change it, change the table and re-render. -->

# Dispatch funnel — transition machine

The phase funnel's legality authority. Every position advance is approved by
`funnel_machine::attempt`; no other code path may land a position.
";

/// Fixed prose + header opening the derived transition table.
const TRANSITIONS_OPEN: &str = "
## Transitions

| current | transition | gate | result |
| --- | --- | --- | --- |
";

/// Fixed prose between the transition table and the derived expected-next table.
const EXPECTED_OPEN: &str = "
Everything absent from this table refuses. Positions only advance; evidence may
update in place.

## Expected next

The refusal payload and the `next` oracle read this one projection — there is no
second table.

| current | expected next |
| --- | --- |
";

/// Fixed prose between the expected-next table and the derived refusal tokens.
const REASONS_OPEN: &str = "
Two entries are facts-conditional: at `imported` with red stored evidence the
prescription is triage rather than a bare re-verify, and at `verified` with a
stale tree it reverts to verify.

## Refusal tokens

| token | raised when |
| --- | --- |
";

/// The diagram section's fixed opening.
const DIAGRAM_OPEN: &str = "
## Diagram

```mermaid
stateDiagram-v2
";

/// The diagram section's fixed close (the fenced block's terminator).
const DIAGRAM_CLOSE: &str = "```\n";

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "tests: fail-fast unwrap on fixture setup is idiomatic"
)]
mod tests {
    use super::*;

    // --- fixtures ---------------------------------------------------------------------

    const FORK: &str = "dispatch/worker-a";
    const BASE: &str = "base000";
    const FORK_TIP: &str = "forktip1";
    const ONTO: &str = "coordtip1";
    const VERIFIED_OID: &str = "verifiedtip1";
    const AT: &str = "2026-07-25T09:00:00Z";
    const SUITE: &str = "gate";
    const RECORD: &str = ".doctrine/dispatch/228/funnel.toml";

    /// Every position, plus the pre-spawn `None` — the matrix's row axis.
    const ALL_CURRENTS: &[Option<Position>] = &[
        None,
        Some(Position::Spawned),
        Some(Position::WorkerCommitted),
        Some(Position::Imported),
        Some(Position::Verified),
        Some(Position::Concluded),
        Some(Position::Reaped),
    ];

    /// Every transition kind — the matrix's column axis.
    const ALL_KINDS: &[TransitionKind] = &[
        TransitionKind::Spawn,
        TransitionKind::RecordWorkerCommit,
        TransitionKind::Import,
        TransitionKind::Verify,
        TransitionKind::Conclude,
        TransitionKind::Reap,
    ];

    fn evidence(status: VerifyStatus, oid: &str) -> VerifyEvidence {
        VerifyEvidence {
            status,
            verified_oid: oid.to_owned(),
            suite: SUITE.to_owned(),
            at: AT.to_owned(),
        }
    }

    /// The transition of `kind` carrying facts that MATCH [`row_at`]'s provenance —
    /// so the matrix exercises the replay leg wherever one exists.
    fn matching(kind: TransitionKind) -> Transition {
        match kind {
            TransitionKind::Spawn => Transition::Spawn {
                fork: FORK.to_owned(),
                base_oid: BASE.to_owned(),
            },
            TransitionKind::RecordWorkerCommit => Transition::RecordWorkerCommit {
                fork_tip: FORK_TIP.to_owned(),
            },
            TransitionKind::Import => Transition::Import {
                fork_tip: FORK_TIP.to_owned(),
                onto: ONTO.to_owned(),
            },
            TransitionKind::Verify => Transition::Verify {
                evidence: evidence(VerifyStatus::Pass, VERIFIED_OID),
            },
            TransitionKind::Conclude => Transition::Conclude,
            TransitionKind::Reap => Transition::Reap,
        }
    }

    /// A row standing at `position`, with every earlier transition's provenance
    /// filled exactly as the sole writer would have landed it.
    fn row_at(position: Position) -> PhaseRow {
        let mut row = PhaseRow {
            id: "PHASE-01".to_owned(),
            position,
            updated_at: AT.to_owned(),
            spawn: Some(SpawnRecord {
                fork: FORK.to_owned(),
                base_oid: BASE.to_owned(),
                at: AT.to_owned(),
            }),
            worker_commit: None,
            import: None,
            verify: None,
            conclude: None,
            reap: None,
        };
        if position >= Position::WorkerCommitted {
            row.worker_commit = Some(WorkerCommitRecord {
                fork_tip: FORK_TIP.to_owned(),
                at: AT.to_owned(),
            });
        }
        if position >= Position::Imported {
            row.import = Some(ImportRecord {
                fork_tip: FORK_TIP.to_owned(),
                onto: ONTO.to_owned(),
                at: AT.to_owned(),
            });
        }
        if position >= Position::Verified {
            row.verify = Some(evidence(VerifyStatus::Pass, VERIFIED_OID));
        }
        if position >= Position::Concluded {
            row.conclude = Some(StampRecord { at: AT.to_owned() });
        }
        if position >= Position::Reaped {
            row.reap = Some(StampRecord { at: AT.to_owned() });
        }
        row
    }

    /// Facts over `row`, with the coord tip advanced PAST the verified oid — the
    /// shape a fresh evidence commit leaves behind.
    fn facts<'a>(row: Option<&'a PhaseRow>, paths: Option<&'a [String]>) -> TransitionFacts<'a> {
        TransitionFacts {
            row,
            coord_tip: "coordtip2",
            paths_since_verify: paths,
        }
    }

    fn only_the_record() -> Vec<String> {
        vec![RECORD.to_owned()]
    }

    // --- VT-1: the exhaustive legality matrix ------------------------------------------

    /// What the machine must answer for a `(current, kind)` pair.
    #[derive(Debug, PartialEq, Eq)]
    enum Verdict {
        /// A real advance to this position.
        Act(Position),
        /// An idempotent no-op replay at this position.
        Replay(Position),
        /// Refused with this reason token and this prescription.
        Refused(&'static str, Expected),
    }

    /// The normative matrix (design §2's table plus its "everything else refuses"
    /// closure), stated INDEPENDENTLY of the implementation and checked
    /// exhaustively by the compiler over `(current, kind)`.
    fn expected_verdict(current: Option<Position>, kind: TransitionKind) -> Verdict {
        use Position as P;
        use TransitionKind as K;
        match (current, kind) {
            // Pre-spawn: only spawn.
            (None, K::Spawn) => Verdict::Act(P::Spawned),
            (None, _) => Verdict::Refused(NOT_SPAWNED, Expected::Spawn),

            // Spawned: the worker's own commit is the only legal next event.
            (Some(P::Spawned), K::Spawn) => Verdict::Replay(P::Spawned),
            (Some(P::Spawned), K::RecordWorkerCommit) => Verdict::Act(P::WorkerCommitted),
            (Some(P::Spawned), _) => Verdict::Refused(WORKER_NOT_COMMITTED, Expected::AwaitWorker),

            // WorkerCommitted: import.
            (Some(P::WorkerCommitted), K::Spawn) => {
                Verdict::Refused("already-worker-committed", Expected::Import)
            }
            (Some(P::WorkerCommitted), K::RecordWorkerCommit) => {
                Verdict::Replay(P::WorkerCommitted)
            }
            (Some(P::WorkerCommitted), K::Import) => Verdict::Act(P::Imported),
            (Some(P::WorkerCommitted), _) => Verdict::Refused(NOT_IMPORTED, Expected::Import),

            // Imported: verify. Conclude has no evidence yet; reap has no boundary.
            (Some(P::Imported), K::Spawn | K::RecordWorkerCommit) => {
                Verdict::Refused("already-imported", Expected::Verify)
            }
            (Some(P::Imported), K::Import) => Verdict::Replay(P::Imported),
            (Some(P::Imported), K::Verify) => Verdict::Act(P::Verified),
            (Some(P::Imported), K::Conclude) => {
                Verdict::Refused(CONCLUDE_UNVERIFIED, Expected::Verify)
            }
            (Some(P::Imported), K::Reap) => Verdict::Refused(NOT_CONCLUDED, Expected::Verify),

            // Verified: conclude (re-verify stays legal, and replays on identical
            // evidence).
            (Some(P::Verified), K::Spawn | K::RecordWorkerCommit | K::Import) => {
                Verdict::Refused("already-verified", Expected::Conclude)
            }
            (Some(P::Verified), K::Verify) => Verdict::Replay(P::Verified),
            (Some(P::Verified), K::Conclude) => Verdict::Act(P::Concluded),
            (Some(P::Verified), K::Reap) => Verdict::Refused(NOT_CONCLUDED, Expected::Conclude),

            // Concluded: reap.
            (Some(P::Concluded), K::Spawn | K::RecordWorkerCommit | K::Import | K::Verify) => {
                Verdict::Refused("already-concluded", Expected::Reap)
            }
            (Some(P::Concluded), K::Conclude) => Verdict::Replay(P::Concluded),
            (Some(P::Concluded), K::Reap) => Verdict::Act(P::Reaped),

            // Reaped: terminal, bar its own replay.
            (Some(P::Reaped), K::Reap) => Verdict::Replay(P::Reaped),
            (Some(P::Reaped), _) => Verdict::Refused(TERMINAL, Expected::Terminal),
        }
    }

    #[test]
    fn attempt_answers_the_normative_matrix_for_every_position_by_transition() {
        let paths = only_the_record();
        for &current in ALL_CURRENTS {
            let row = current.map(row_at);
            for &kind in ALL_KINDS {
                let facts = facts(row.as_ref(), Some(&paths));
                let got = attempt_advance(current, &matching(kind), &facts);
                let want = expected_verdict(current, kind);
                let label = format!("{current:?} × {kind:?}");
                match (got, want) {
                    (Ok(advance), Verdict::Act(position)) => {
                        assert_eq!(advance.position, position, "{label}: landed position");
                        assert!(!advance.replay, "{label}: an act, not a replay");
                    }
                    (Ok(advance), Verdict::Replay(position)) => {
                        assert_eq!(advance.position, position, "{label}: replay position");
                        assert!(advance.replay, "{label}: a replay, not an act");
                    }
                    (Err(illegal), Verdict::Refused(reason, expected)) => {
                        assert_eq!(illegal.reason, reason, "{label}: reason token");
                        assert_eq!(illegal.expected, expected, "{label}: prescription");
                        assert_eq!(illegal.current, current, "{label}: refusal names current");
                        assert_eq!(illegal.attempted, kind, "{label}: refusal names the kind");
                    }
                    (got, want) => panic!("{label}: got {got:?}, want {want:?}"),
                }
            }
        }
    }

    #[test]
    fn attempt_is_the_thin_projection_of_attempt_advance() {
        let paths = only_the_record();
        for &current in ALL_CURRENTS {
            let row = current.map(row_at);
            for &kind in ALL_KINDS {
                let facts = facts(row.as_ref(), Some(&paths));
                let full = attempt_advance(current, &matching(kind), &facts);
                let thin = attempt(current, matching(kind), &facts);
                assert_eq!(
                    thin.as_ref().ok().copied(),
                    full.as_ref().ok().map(|a| a.position),
                    "{current:?} × {kind:?}: same landing"
                );
                assert_eq!(
                    thin.err(),
                    full.err(),
                    "{current:?} × {kind:?}: same refusal"
                );
            }
        }
    }

    // --- SL-228 PHASE-05 D1: the ORDERED multi-transition fold -------------------------

    /// The import heal-forward ladder from no row at all.
    fn heal_ladder() -> Vec<Transition> {
        vec![
            Transition::Spawn {
                fork: FORK.to_owned(),
                base_oid: BASE.to_owned(),
            },
            Transition::RecordWorkerCommit {
                fork_tip: FORK_TIP.to_owned(),
            },
            Transition::Import {
                fork_tip: FORK_TIP.to_owned(),
                onto: ONTO.to_owned(),
            },
        ]
    }

    #[test]
    fn fold_transitions_heals_a_whole_prefix_in_one_pass() {
        // From NO ROW, `[Spawn, RecordWorkerCommit, Import]` is legal end-to-end
        // BECAUSE each element is gated against the row its predecessor produced —
        // `Import` alone from `None` is `not-spawned` (the matrix pins that).
        let fold = fold_transitions(None, "PHASE-01", &heal_ladder(), ONTO, None, AT)
            .expect("the ladder is legal")
            .expect("non-empty");
        assert_eq!(fold.position, Position::Imported);
        assert!(!fold.replay, "a real heal is not a replay");
        let row = fold.row.expect("the healed row");
        // EVERY element's provenance is folded in — one row, not just the last stamp.
        assert_eq!(row.spawn.expect("spawn").base_oid, BASE);
        assert_eq!(row.worker_commit.expect("worker commit").fork_tip, FORK_TIP);
        assert_eq!(row.import.expect("import").onto, ONTO);
        assert_eq!(row.position, Position::Imported);
    }

    #[test]
    fn fold_transitions_is_all_or_none_on_a_mid_fold_refusal() {
        // A ladder whose SECOND element is illegal: conclude before any verify
        // evidence exists. The whole fold short-circuits with that refusal, so a
        // caller that writes only on `Ok` lands NOTHING — not even the legal prefix.
        let ts = vec![
            Transition::Spawn {
                fork: FORK.to_owned(),
                base_oid: BASE.to_owned(),
            },
            Transition::Conclude,
        ];
        let err = fold_transitions(None, "PHASE-01", &ts, ONTO, None, AT)
            .expect_err("the second element is illegal");
        assert_eq!(err.attempted, TransitionKind::Conclude);
        assert_eq!(
            err.current,
            Some(Position::Spawned),
            "the refusal names the RUNNING position, not the stored one"
        );
        assert_eq!(err.reason, WORKER_NOT_COMMITTED);
    }

    #[test]
    fn fold_transitions_of_an_all_replay_ladder_reports_replay() {
        // The lost-response retry: the row already stands at `imported` with the SAME
        // fork tip, so the ladder for that position (`[Import]`) replays wholesale and
        // NOTHING may be written.
        let stored = row_at(Position::Imported);
        let ts = vec![Transition::Import {
            fork_tip: FORK_TIP.to_owned(),
            onto: ONTO.to_owned(),
        }];
        let fold = fold_transitions(Some(&stored), "PHASE-01", &ts, ONTO, None, AT)
            .expect("legal")
            .expect("non-empty");
        assert!(fold.replay, "every element replayed");
        assert_eq!(fold.position, Position::Imported);
        assert_eq!(
            fold.row.as_ref(),
            Some(&stored),
            "an all-replay fold leaves the stored row byte-identical"
        );
    }

    #[test]
    fn fold_transitions_refuses_a_healed_prefix_replayed_past_its_milestone() {
        // Why the import ladder is POSITION-KEYED and not "always the full prefix":
        // once the row is at `imported`, replaying `Spawn` is not idempotent — the
        // milestone is behind `current`, so the machine refuses `already-imported`
        // rather than silently re-stamping provenance.
        let stored = row_at(Position::Imported);
        let err = fold_transitions(Some(&stored), "PHASE-01", &heal_ladder(), ONTO, None, AT)
            .expect_err("the full ladder is illegal from `imported`");
        assert_eq!(err.attempted, TransitionKind::Spawn);
        assert_eq!(err.reason, "already-imported");
    }

    #[test]
    fn fold_transitions_replay_identity_excludes_import_onto() {
        // RV-304 F-1: a retry re-resolves a FRESH `onto`, which must not defeat the
        // replay — only `fork_tip` keys import's identity.
        let stored = row_at(Position::Imported);
        let ts = vec![Transition::Import {
            fork_tip: FORK_TIP.to_owned(),
            onto: "a-freshly-resolved-tip".to_owned(),
        }];
        let fold = fold_transitions(Some(&stored), "PHASE-01", &ts, ONTO, None, AT)
            .expect("legal")
            .expect("non-empty");
        assert!(fold.replay, "a fresh `onto` is still the same import");
    }

    #[test]
    fn fold_transitions_of_an_empty_list_is_none() {
        // No element ⇒ no position to report and nothing to land; the caller decides
        // whether that is a programming error.
        assert_eq!(
            fold_transitions(None, "PHASE-01", &[], ONTO, None, AT).expect("legal"),
            None
        );
    }

    #[test]
    fn fold_transitions_agrees_with_attempt_advance_on_every_single_element() {
        // The fold is not a second authority: for a ONE-element list it must answer
        // exactly what `attempt_advance` answers, over the whole matrix.
        let paths = only_the_record();
        for current in ALL_CURRENTS {
            let stored = current.map(row_at);
            for kind in ALL_KINDS {
                let t = matching(*kind);
                let f = facts(stored.as_ref(), Some(&paths));
                let direct = attempt_advance(*current, &t, &f);
                let folded = fold_transitions(
                    stored.as_ref(),
                    "PHASE-01",
                    std::slice::from_ref(&t),
                    f.coord_tip,
                    Some(&paths),
                    AT,
                );
                match (direct, folded) {
                    (Ok(advance), Ok(Some(fold))) => {
                        assert_eq!(advance.position, fold.position, "{current:?} × {kind:?}");
                        assert_eq!(advance.replay, fold.replay, "{current:?} × {kind:?}");
                    }
                    (Err(a), Err(b)) => assert_eq!(a, b, "{current:?} × {kind:?}"),
                    (d, f) => panic!("{current:?} × {kind:?}: disagreement {d:?} vs {f:?}"),
                }
            }
        }
    }

    #[test]
    fn preflight_admits_exactly_the_tables_kind_column() {
        for &current in ALL_CURRENTS {
            for &kind in ALL_KINDS {
                let legal = row_for(current, kind).is_some();
                let got = preflight(current, kind);
                assert_eq!(
                    got.is_ok(),
                    legal,
                    "{current:?} × {kind:?}: preflight tracks the table"
                );
                if let Err(illegal) = got {
                    assert_eq!(illegal.attempted, kind);
                    assert_eq!(illegal.current, current);
                    // Factless: the prescription is the table column alone.
                    assert_eq!(illegal.expected, prescription(current));
                }
            }
        }
    }

    #[test]
    fn preflight_gates_verify_before_the_evidence_exists() {
        // The RV-304 F-3 case: `dispatch verify` gates at entry, when no
        // `Transition::Verify{evidence}` can be constructed yet.
        assert!(preflight(Some(Position::Imported), TransitionKind::Verify).is_ok());
        assert!(preflight(Some(Position::Verified), TransitionKind::Verify).is_ok());
        let refused = preflight(Some(Position::Spawned), TransitionKind::Verify).unwrap_err();
        assert_eq!(refused.reason, WORKER_NOT_COMMITTED);
    }

    // --- VT-1: replay identity ---------------------------------------------------------

    #[test]
    fn import_replay_identity_is_fork_tip_and_excludes_onto() {
        let row = row_at(Position::Imported);
        let paths = only_the_record();
        // A lost-response retry re-resolves a FRESH `onto` — still a replay (F-1).
        let retry = Transition::Import {
            fork_tip: FORK_TIP.to_owned(),
            onto: "a-freshly-resolved-coord-tip".to_owned(),
        };
        let advance = attempt_advance(
            Some(Position::Imported),
            &retry,
            &facts(Some(&row), Some(&paths)),
        )
        .unwrap();
        assert!(advance.replay, "a fresh `onto` must not defeat the replay");
        assert_eq!(advance.position, Position::Imported);

        // A DIFFERENT fork_tip is a different import: refused, not replayed.
        let other = Transition::Import {
            fork_tip: "someothertip".to_owned(),
            onto: ONTO.to_owned(),
        };
        let refused = attempt_advance(
            Some(Position::Imported),
            &other,
            &facts(Some(&row), Some(&paths)),
        )
        .unwrap_err();
        assert_eq!(refused.reason, "already-imported");
    }

    #[test]
    fn record_worker_commit_replay_identity_is_fork_tip() {
        let row = row_at(Position::WorkerCommitted);
        let mismatched = Transition::RecordWorkerCommit {
            fork_tip: "adifferenttip".to_owned(),
        };
        let refused = attempt_advance(
            Some(Position::WorkerCommitted),
            &mismatched,
            &facts(Some(&row), None),
        )
        .unwrap_err();
        assert_eq!(refused.reason, "already-worker-committed");
        assert!(
            refused.to_string().contains("already-worker-committed"),
            "the refusal renders its reason: {refused}"
        );
    }

    #[test]
    fn spawn_replay_identity_is_fork_and_base() {
        let row = row_at(Position::Spawned);
        let mismatched = Transition::Spawn {
            fork: FORK.to_owned(),
            base_oid: "adifferentbase".to_owned(),
        };
        let refused = attempt_advance(
            Some(Position::Spawned),
            &mismatched,
            &facts(Some(&row), None),
        )
        .unwrap_err();
        assert_eq!(refused.reason, "already-spawned");
    }

    #[test]
    fn verify_replay_identity_is_the_verified_oid_and_the_outcome() {
        let row = row_at(Position::Verified); // stored: pass @ VERIFIED_OID
        let paths = only_the_record();
        let at = Some(Position::Verified);

        // Identical evidence ⇒ replay.
        let same = Transition::Verify {
            evidence: evidence(VerifyStatus::Pass, VERIFIED_OID),
        };
        assert!(
            attempt_advance(at, &same, &facts(Some(&row), Some(&paths)))
                .unwrap()
                .replay
        );

        // A different verified_oid ⇒ a fresh act (re-verify).
        let fresh = Transition::Verify {
            evidence: evidence(VerifyStatus::Pass, "anewertip"),
        };
        let advance = attempt_advance(at, &fresh, &facts(Some(&row), Some(&paths))).unwrap();
        assert!(!advance.replay);
        assert_eq!(advance.position, Position::Verified);

        // The same oid with the OTHER outcome ⇒ an act, and the position is kept
        // (monotone: a failed re-verify never demotes).
        let red = Transition::Verify {
            evidence: evidence(VerifyStatus::Fail, VERIFIED_OID),
        };
        let advance = attempt_advance(at, &red, &facts(Some(&row), Some(&paths))).unwrap();
        assert!(!advance.replay, "an outcome flip is an act");
        assert_eq!(
            advance.position,
            Position::Verified,
            "positions only advance"
        );
    }

    #[test]
    fn a_first_failed_verify_at_imported_is_an_act_then_a_replay() {
        // RV-303 F-2: act-vs-replay is decided by comparing candidate evidence to
        // STORED evidence, never by comparing positions.
        let mut row = row_at(Position::Imported); // no stored evidence
        let red = Transition::Verify {
            evidence: evidence(VerifyStatus::Fail, VERIFIED_OID),
        };
        let advance =
            attempt_advance(Some(Position::Imported), &red, &facts(Some(&row), None)).unwrap();
        assert!(!advance.replay, "the first red verify is an ACT");
        assert_eq!(
            advance.position,
            Position::Imported,
            "a fail keeps position"
        );

        // Land it, then retry the identical evidence: now it IS a replay.
        let id = row.id.clone();
        row = apply(
            Some(&row),
            &id,
            &red,
            advance.position,
            "2026-07-25T10:00:00Z",
        );
        let advance =
            attempt_advance(Some(Position::Imported), &red, &facts(Some(&row), None)).unwrap();
        assert!(advance.replay, "the identical retry is a REPLAY");
    }

    // --- VT-1: expected_next -----------------------------------------------------------

    #[test]
    fn red_evidence_flips_the_prescription_to_triage() {
        let mut row = row_at(Position::Imported);
        row.verify = Some(evidence(VerifyStatus::Fail, VERIFIED_OID));
        assert_eq!(
            expected_next(Some(Position::Imported), &facts(Some(&row), None)),
            Expected::Triage
        );

        // The same at `verified` (a failed re-verify kept the position).
        let mut row = row_at(Position::Verified);
        row.verify = Some(evidence(VerifyStatus::Fail, VERIFIED_OID));
        let paths = only_the_record();
        assert_eq!(
            expected_next(Some(Position::Verified), &facts(Some(&row), Some(&paths))),
            Expected::Triage
        );
    }

    #[test]
    fn a_stale_tree_at_verified_reverts_the_prescription_to_verify() {
        let row = row_at(Position::Verified);
        let dirty = vec!["src/dispatch.rs".to_owned()];
        assert_eq!(
            expected_next(Some(Position::Verified), &facts(Some(&row), Some(&dirty))),
            Expected::Verify
        );
        // Clean (the funnel record only) ⇒ conclude.
        let paths = only_the_record();
        assert_eq!(
            expected_next(Some(Position::Verified), &facts(Some(&row), Some(&paths))),
            Expected::Conclude
        );
    }

    #[test]
    fn every_position_has_exactly_one_prescription_row() {
        for &current in ALL_CURRENTS {
            let prescribing = TABLE
                .iter()
                .filter(|r| r.current == current && r.prescribes.is_some())
                .count();
            assert_eq!(prescribing, 1, "{current:?}: exactly one prescription row");
        }
    }

    // --- VT-1: conclude's three refusals + the modulo-funnel-record gate ---------------

    #[test]
    fn conclude_refuses_unverified_failed_and_stale_distinctly() {
        let paths = only_the_record();

        // (a) no evidence at all.
        let mut row = row_at(Position::Verified);
        row.verify = None;
        let refused = attempt_advance(
            Some(Position::Verified),
            &Transition::Conclude,
            &facts(Some(&row), Some(&paths)),
        )
        .unwrap_err();
        assert_eq!(refused.reason, CONCLUDE_UNVERIFIED);

        // (b) red evidence.
        let mut row = row_at(Position::Verified);
        row.verify = Some(evidence(VerifyStatus::Fail, VERIFIED_OID));
        let refused = attempt_advance(
            Some(Position::Verified),
            &Transition::Conclude,
            &facts(Some(&row), Some(&paths)),
        )
        .unwrap_err();
        assert_eq!(refused.reason, CONCLUDE_VERIFY_FAILED);

        // (c) green evidence, but a non-funnel-record path changed since.
        let row = row_at(Position::Verified);
        let dirty = vec![RECORD.to_owned(), "src/dispatch.rs".to_owned()];
        let refused = attempt_advance(
            Some(Position::Verified),
            &Transition::Conclude,
            &facts(Some(&row), Some(&dirty)),
        )
        .unwrap_err();
        assert_eq!(refused.reason, CONCLUDE_VERIFY_STALE);
    }

    #[test]
    fn conclude_is_never_staled_by_its_own_evidence_commit() {
        // The D3 gate is modulo the funnel record — NOT bare `verified_oid != tip`,
        // which would stale every fresh pass on the very commit that recorded it
        // (RV-305 F-2). Here the coord tip HAS advanced past `verified_oid`, and the
        // only path that changed is the record.
        let row = row_at(Position::Verified);
        let paths = only_the_record();
        let f = facts(Some(&row), Some(&paths));
        assert_ne!(row.verify.as_ref().unwrap().verified_oid, f.coord_tip);
        let advance = attempt_advance(Some(Position::Verified), &Transition::Conclude, &f).unwrap();
        assert_eq!(advance.position, Position::Concluded);
        assert!(!advance.replay);
    }

    #[test]
    fn conclude_needs_no_path_list_when_the_tip_never_moved() {
        // Tree identity is trivial when the coord tip IS the verified oid, so the
        // shell owes no diff.
        let row = row_at(Position::Verified);
        let f = TransitionFacts {
            row: Some(&row),
            coord_tip: VERIFIED_OID,
            paths_since_verify: None,
        };
        let advance = attempt_advance(Some(Position::Verified), &Transition::Conclude, &f).unwrap();
        assert_eq!(advance.position, Position::Concluded);
    }

    #[test]
    fn conclude_fails_closed_when_no_tree_identity_was_established() {
        let row = row_at(Position::Verified);
        let refused = attempt_advance(
            Some(Position::Verified),
            &Transition::Conclude,
            &facts(Some(&row), None),
        )
        .unwrap_err();
        assert_eq!(refused.reason, CONCLUDE_VERIFY_STALE);
    }

    // --- VT-1: tokens, paths, folding --------------------------------------------------

    #[test]
    fn already_tokens_track_the_position_spelling() {
        for &current in ALL_CURRENTS.iter().flatten() {
            assert_eq!(
                current.already_token(),
                format!("already-{}", current.as_str()),
                "the already-<position> family is derived from as_str"
            );
        }
        assert!(ALREADY_FAMILY.starts_with("already-"));
    }

    #[test]
    fn position_and_status_tokens_round_trip_through_serde() {
        for &current in ALL_CURRENTS.iter().flatten() {
            let row = row_at(current);
            let text = toml::to_string(&row).unwrap();
            assert!(
                text.contains(&format!("position = \"{}\"", current.as_str())),
                "the serde token IS as_str for {current:?}: {text}"
            );
            let back: PhaseRow = toml::from_str(&text).unwrap();
            assert_eq!(back, row);
        }
        let green = evidence(VerifyStatus::Pass, VERIFIED_OID);
        let text = toml::to_string(&green).unwrap();
        assert!(text.contains(&format!("status = \"{}\"", VerifyStatus::Pass.as_str())));
    }

    #[test]
    fn is_funnel_record_matches_path_components_not_substrings() {
        assert!(is_funnel_record(".doctrine/dispatch/228/funnel.toml"));
        assert!(is_funnel_record(&funnel_record_path(7)));
        assert_eq!(funnel_record_path(7), ".doctrine/dispatch/007/funnel.toml");
        for path in [
            ".doctrine/dispatch/228/funnel.toml.bak",
            ".doctrine/dispatch/228/boundaries.toml",
            ".doctrine/not-dispatch/228/funnel.toml",
            "vendor/.doctrine/dispatch/228/funnel.toml",
            ".doctrine/dispatch/22/funnel.toml",
            ".doctrine/dispatch/228/nested/funnel.toml",
            "funnel.toml",
        ] {
            assert!(!is_funnel_record(path), "must not match `{path}`");
        }
    }

    #[test]
    fn apply_folds_the_position_the_stamp_and_the_transitions_provenance() {
        let row = apply(
            None,
            "PHASE-03",
            &Transition::Spawn {
                fork: FORK.to_owned(),
                base_oid: BASE.to_owned(),
            },
            Position::Spawned,
            AT,
        );
        assert_eq!(row.id, "PHASE-03");
        assert_eq!(row.position, Position::Spawned);
        assert_eq!(row.updated_at, AT);
        assert_eq!(row.spawn.as_ref().unwrap().fork, FORK);
        assert!(
            row.worker_commit.is_none(),
            "only the acted sub-table fills"
        );

        // A later transition keeps the earlier provenance.
        let row = apply(
            Some(&row),
            "PHASE-03",
            &Transition::Import {
                fork_tip: FORK_TIP.to_owned(),
                onto: ONTO.to_owned(),
            },
            Position::Imported,
            "2026-07-25T11:00:00Z",
        );
        assert_eq!(row.position, Position::Imported);
        assert_eq!(row.updated_at, "2026-07-25T11:00:00Z");
        assert_eq!(row.spawn.as_ref().unwrap().fork, FORK, "spawn survives");
        let import = row.import.as_ref().unwrap();
        assert_eq!(import.fork_tip, FORK_TIP);
        assert_eq!(import.onto, ONTO);
    }

    #[test]
    fn illegal_transition_renders_position_reason_and_prescription() {
        let refused = attempt(None, Transition::Conclude, &facts(None, None)).unwrap_err();
        let text = refused.to_string();
        assert!(text.contains("conclude"), "{text}");
        assert!(text.contains(NO_POSITION), "{text}");
        assert!(text.contains(NOT_SPAWNED), "{text}");
        assert!(text.contains(Expected::Spawn.as_str()), "{text}");
    }

    // --- VT-2: the D7 golden ------------------------------------------------------------

    #[test]
    fn rendered_artifact_matches_the_committed_golden_byte_for_byte() {
        // Read at RUNTIME (not `include_str!`) so a path slip is a test failure,
        // not a compile error — and resolve the root at runtime too (CHR-014 /
        // SL-162: no compile-time path baking).
        let path = crate::test_support::repo_root().join(ARTIFACT_PATH);
        let committed = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read the D7 artifact at {}: {e}", path.display()));
        let rendered = render_artifact();
        assert_eq!(
            rendered,
            committed,
            "the D7 artifact has drifted from the const table — re-render, never hand-edit ({})",
            path.display()
        );
        assert!(
            rendered.contains("mermaid"),
            "the artifact carries the diagram"
        );
    }
}
