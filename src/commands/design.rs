// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine design` — the persistence shell for CLI-managed design runs
//! (SL-233 PHASE-03, design §5.2/§5.3).
//!
//! ADR-001 tier **command**: everything impure lives here — the clock-free run
//! UID, the digests, the reads and the atomic writes — and
//! [`crate::design_run`] stays a leaf with crate out-degree zero. The pure core
//! is handed facts ([`design_run::run::DerivedInput`]) and hands back a
//! validated candidate; this module decides *when* bytes hit the disk and in
//! what order.
//!
//! # DEC-092's three rules, and why they are three
//!
//! The snapshot's monotonic revision guards runtime writers against each other
//! and structurally cannot see the authored tier, so the **authored watermark**
//! — the fingerprint of `design.md` as Doctrine last left it — guards that tier
//! separately.
//!
//! 1. **Ordinary mutation entry-refuses divergence.** `start`, ordinary `apply`
//!    and `materialise` fingerprint `design.md` on entry and refuse when it
//!    differs from the watermark, rather than clearing another gate against
//!    prose the snapshot no longer describes. An absent `design.md` before first
//!    materialisation is *cold*, not divergent; absent after it is divergent.
//! 2. **Re-adopt is the sole lawful crossing, and it is a protocol, not a
//!    bypass.** [`design_run::run::apply`]'s adoption path is exempt from rule 1
//!    only through the §5.5 admission path, and the watermark re-baselines only
//!    after the candidate validates in full.
//! 3. **The pre-write check narrows the window; it does not close it.**
//!    [`recheck_watermark_before_write`] re-reads and re-fingerprints immediately
//!    before the snapshot write. The comparison basis differs by class and the
//!    difference is load-bearing ([`PreWriteBasis`]): a rule-2 re-adoption
//!    compares against the fingerprint it was *admitted on*, never against the
//!    watermark it exists to replace — that conflation is F-20's self-refusal.
//!
//! The shape is borrowed from [`crate::review`]'s `with_turn` /
//! `with_turn_hooked`, which runs the same entry-then-pre-write comparison for
//! the review ledger. DEC-092 states in writing why the borrowing is **partial**:
//! `with_turn` holds a writer lock and hashes the very file it atomically
//! replaces, whereas a design run hashes `design.md` while writing a runtime
//! snapshot, a checkpoint journal, and possibly an authored record. That is why
//! the guarantee here is stated as **the run does not advance** rather than as
//! `with_turn`'s stronger, and for `with_turn` entirely true, no-write claim:
//! effects ordered before the snapshot under DEC-083/DEC-086 remain, and remain
//! recoverable through the submission-keyed journal without duplication.

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Subcommand, ValueEnum as _};

use crate::design_run;
use crate::design_run::Stage;
use crate::design_run::TurnEnvelope;
use crate::design_run::attestation::{AcceptanceAttestation, IntentState, RecoveryIntent};
use crate::design_run::delegation::Delegation;
use crate::design_run::ids::{DesignId, Fingerprint, IdKind};
use crate::design_run::render::envelope::{self, Detail};
use crate::design_run::run::{Admission, DerivedInput};
use crate::design_run::snapshot::{self, CheckpointGroup, DesignSnapshot};
use crate::design_run::submission::{
    ApplyRequest, Declaration, DelegationAct, DischargeClaim, Dispose,
};

// ── Named constants (STD-001) ─────────────────────────────────────────────

/// The authored design document, under the slice's authored tree.
const DESIGN_DOC: &str = "design.md";
// The stable section marker `materialise` writes and re-adoption addresses is
// `design_run::document::MARKER_OPEN`/`MARKER_CLOSE`, single-sourced beside the
// grammar that recognises it (STD-001). This module never re-spells it.
/// The run UID prefix — a design run, not a slice or a record.
const RUN_UID_PREFIX: &str = "dr-";
/// The `--input` value that means "read the payload from stdin", following the
/// sentinel `memory record --body` established.
const STDIN_SENTINEL: &str = "-";
/// The id a `create` disposition is validated against **before** one is claimed.
/// Never written anywhere: the provisional candidate is discarded (see [`apply`]).
const PROVISIONAL_RECORD_ID: u32 = 0;
/// What the DEC-088 acceptance digest joins its bound facts with.
const ACCEPTANCE_BINDING_SEPARATOR: &str = "\u{1f}";
/// What a lock rests on, said out loud (SL-233 PHASE-12 EX-5, design R12).
///
/// v1 trusts a cooperative agent: the acceptance attestation is that agent's
/// attributed, auditable claim that the user accepted the design — it is **not**
/// independent authentication of a human act. Stated here, once, so the surface
/// carries the limit rather than the design document alone.
const LOCK_ACCEPTANCE_DISCLOSURE: &str = "locked on an auditable agent claim of user acceptance — not authenticated proof of a human act";
/// The contract an exported assignment carries, said out loud (SL-233 PHASE-10
/// `EX-1`/`EX-2`, DEC-068).
///
/// The assignment is emitted where it is *created*, the way the lock discloses
/// what it rests on where the lock is taken. It is self-contained on purpose: a
/// fresh session holding these lines needs nothing else to work the obligation —
/// and needs to be told, in the same breath, that it may propose and may not
/// write.
const ASSIGNMENT_CONTRACT: &str = "propose back with a `delegation` act of `propose` carrying `by`, `summary` and any `declare`; a proposal may carry nothing that writes — the coordinator applies what it accepts";

// ── CLI surface ───────────────────────────────────────────────────────────

/// The design-run verbs (design §5.2).
///
/// Every read below is a **rendering of one [`design_run::TurnEnvelope`]**
/// (DEC-064) — never a second model. The shell's job is to choose a rendering
/// and hand over bytes; the bounds are applied before it ever sees the value.
#[derive(Subcommand, Debug)]
pub(crate) enum DesignCommand {
    /// Create a design run for a slice.
    Start(StartArgs),
    /// Show the current turn: active path, nearby frontier, blockers, counts and
    /// material changes. `--full` widens it.
    Show(ShowArgs),
    /// Validate and apply one sparse idempotent mutation.
    Apply(ApplyArgs),
    /// Re-enter a run with the compact projection a fresh context needs.
    Resume(ResumeArgs),
    /// Render runtime sections into authored prose.
    Materialise(MaterialiseArgs),
}

/// Which rendering of the turn envelope to emit (DEC-064).
///
/// **Only `prompt` is budgeted.** `json`'s framing overhead differs and `status`
/// is for a human at a terminal, so neither is what the token-cost claim is
/// about — the sketch says so, and this enum is where that distinction becomes
/// operational.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum ShowFormat {
    /// The budgeted rendering that enters an agent's context.
    #[default]
    Prompt,
    /// The same envelope, as a machine surface.
    Json,
    /// The same envelope, for a human at a terminal.
    Status,
}

/// Arguments for `design start`.
#[derive(clap::Args, Debug)]
pub(crate) struct StartArgs {
    /// The slice this run designs, e.g. `SL-233`.
    slice: String,
    /// Adopt an existing `design.md` as the run's baseline.
    #[arg(long)]
    from_design: bool,
    /// Explicit project root (default: auto-detect).
    #[arg(short = 'p', long)]
    path: Option<PathBuf>,
}

/// Arguments for `design show`.
#[derive(clap::Args, Debug)]
pub(crate) struct ShowArgs {
    /// The slice, e.g. `SL-233`.
    slice: String,
    /// Project changes since this revision (default: the previous revision).
    #[arg(long)]
    known_revision: Option<u64>,
    /// Widen the projection: the caps lift and the output may scale with the run.
    #[arg(long)]
    full: bool,
    /// Which rendering of the turn envelope to emit.
    #[arg(long, value_enum, default_value_t = ShowFormat::Prompt)]
    format: ShowFormat,
    /// Explicit project root (default: auto-detect).
    #[arg(short = 'p', long)]
    path: Option<PathBuf>,
}

/// Arguments for `design resume`.
///
/// The happy path is `doctrine design resume SL-NNN` and nothing else. All three
/// optional flags **add** something — an explicit assumption check, or a
/// change-only projection — and none is ever required addressing.
#[derive(clap::Args, Debug)]
pub(crate) struct ResumeArgs {
    /// The slice, e.g. `SL-233`.
    slice: String,
    /// Assert the run you think you are resuming. A mismatch is refused rather
    /// than silently resumed against a different run.
    #[arg(long)]
    run: Option<String>,
    /// Project changes since this revision instead of since the previous one.
    #[arg(long)]
    known_revision: Option<u64>,
    /// Declare a prompt fragment you already hold, so resume reports whether the
    /// run agrees rather than re-sending it.
    #[arg(long)]
    known_fragment: Vec<String>,
    /// Explicit project root (default: auto-detect).
    #[arg(short = 'p', long)]
    path: Option<PathBuf>,
}

/// Arguments for `design apply`.
#[derive(clap::Args, Debug)]
pub(crate) struct ApplyArgs {
    /// The slice, e.g. `SL-233`.
    slice: String,
    /// The payload: a JSON object, a path to one, or `-` for stdin.
    #[arg(long)]
    input: String,
    /// Explicit project root (default: auto-detect).
    #[arg(short = 'p', long)]
    path: Option<PathBuf>,
}

/// Arguments for `design materialise`.
#[derive(clap::Args, Debug)]
pub(crate) struct MaterialiseArgs {
    /// The slice, e.g. `SL-233`.
    slice: String,
    /// Explicit project root (default: auto-detect).
    #[arg(short = 'p', long)]
    path: Option<PathBuf>,
}

/// Route a design verb.
pub(crate) fn dispatch(command: DesignCommand) -> Result<()> {
    match command {
        DesignCommand::Start(args) => run_start(args),
        DesignCommand::Show(args) => run_show(args),
        DesignCommand::Apply(args) => run_apply(args),
        DesignCommand::Resume(args) => run_resume(args),
        DesignCommand::Materialise(args) => run_materialise(args),
    }
}

// ── paths, reads, writes ──────────────────────────────────────────────────

/// Resolve the project root from the optional path argument.
fn resolve_root(path: Option<PathBuf>) -> Result<PathBuf> {
    crate::root::find(path, &crate::root::default_markers())
}

/// The slice id a `SL-233`/`233` reference names.
fn slice_id(reference: &str) -> Result<u32> {
    crate::listing::parse_ref(crate::kinds::SL, "a slice", reference)
}

/// The authored design document for a slice.
fn design_doc_path(root: &Path, slice: u32) -> PathBuf {
    root.join(crate::kinds::SLICE_DIR)
        .join(format!("{slice:03}"))
        .join(DESIGN_DOC)
}

/// The authored bytes, or `None` when the document is absent.
fn read_design_doc(root: &Path, slice: u32) -> Result<Option<String>> {
    let path = design_doc_path(root, slice);
    match std::fs::read_to_string(&path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("read {}", path.display())),
    }
}

/// The fingerprint of the authored document as Doctrine reads it *now*.
fn read_authored_fingerprint(root: &Path, slice: u32) -> Result<Option<Fingerprint>> {
    Ok(read_design_doc(root, slice)?
        .as_deref()
        .map(authored_fingerprint))
}

/// The watermark derivation, single-sourced: the paths that read-then-hash and
/// the paths that hash bytes already in hand must not spell it twice (STD-001).
fn authored_fingerprint(text: &str) -> Fingerprint {
    Fingerprint::new(crate::git::sha256(text.as_bytes()))
}

/// Every marker-addressed section Doctrine reads out of `design.md` — the map a
/// re-adoption's declaration must match exactly, with the bodies it adopts.
///
/// The decomposition itself lives in [`design_run::document`], beside the
/// renderer it inverts, and so does the FIXED ORDER of the seven §5.5 checks:
/// this seam surfaces the refusal it returns rather than deciding anything. It
/// used to swallow it (`unwrap_or_default`), so a document the grammar could not
/// decompose yielded no digests and the adoption refused with the generic
/// `AdoptionMarkersInvalid` — the right outcome by the wrong name, and a name a
/// user cannot act on.
///
/// `held` makes the run-aware rows decidable: the document's marker set is
/// compared against the run's sections here, which is a DIFFERENT comparison
/// from the completeness check inside `adopt_authored` (that one reads the
/// caller's declared map, and a legal-but-unheld marker is invisible to it).
fn authored_sections(
    text: &str,
    held: &std::collections::BTreeSet<DesignId>,
) -> Result<std::collections::BTreeMap<DesignId, design_run::run::AuthoredSection>> {
    Ok(design_run::document::parse(text, Some(held))
        .map_err(|refused| refusal(&refused))?
        .into_iter()
        .enumerate()
        .map(|(position, section)| {
            let fingerprint = Fingerprint::new(crate::git::sha256(section.body.as_bytes()));
            (
                section.id,
                design_run::run::AuthoredSection {
                    position,
                    body: section.body,
                    fingerprint,
                },
            )
        })
        .collect())
}

/// Read the snapshot, refusing when there is no run to speak of.
fn read_snapshot(root: &Path, slice: u32) -> Result<DesignSnapshot> {
    let path = crate::state::design_snapshot_path(root, slice);
    let text = std::fs::read_to_string(&path).with_context(|| no_run_here(root, slice, &path))?;
    snapshot::parse(&text).with_context(|| format!("parse {}", path.display()))
}

/// What to tell a caller who reached for a run that is not there.
///
/// Two different situations wear one symptom, and the authored tier tells them
/// apart: no `design.md` is a run that was never started, while a `design.md`
/// standing beside a missing snapshot is **runtime loss** — the snapshot tier is
/// gitignored and disposable by design, so this is an ordinary event, not a
/// corruption.
///
/// The second names `start --from-design` and says plainly what it is
/// (DEC-057/DEC-084, `EX-5`/`EX-6`): semantic **reconstruction** from authored
/// prose and linked knowledge, a NEW run, and weaker than exact resume, which
/// depended on the file that is gone. Doctrine will not quietly reconstruct
/// procedural history it cannot support — but leaving the caller stuck with
/// `design start`, which refuses outright while a `design.md` exists, would be
/// its own kind of dishonesty.
///
/// It lives here rather than in `run_resume` because absence is a property of
/// the run, not of the verb that asked: `show` and `apply` reach the same wall.
fn no_run_here(root: &Path, slice: u32, path: &Path) -> String {
    let base = format!("no design run for slice {slice:03} at {}", path.display());
    if design_doc_path(root, slice).exists() {
        return format!(
            "{base} — the runtime snapshot is gone but {DESIGN_DOC} is still here, so this \
             is runtime loss, not a missing run. `doctrine design start {} --from-design` \
             RECONSTRUCTS a new run from the authored prose and linked knowledge: a new run \
             uid, and weaker than exact resume, which needed the snapshot. No attestation, \
             receipt or gate clearance is inferred — plain resume never reconstructs them",
            crate::listing::canonical_id(crate::kinds::SLICE_KIND.prefix, slice)
        );
    }
    format!("{base} — run `doctrine design start` first")
}

/// Write the snapshot atomically (`fsutil::write_atomic`'s sibling replacement —
/// the existing helper, not a second writer).
fn write_snapshot(root: &Path, slice: u32, value: &DesignSnapshot) -> Result<()> {
    let path = crate::state::design_snapshot_path(root, slice);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    crate::fsutil::write_atomic(&path, snapshot::to_toml(value)?.as_bytes())
}

/// The checkpoint journal — the same [`design_run::snapshot::CheckpointGroup`]
/// the snapshot carries, stored in its own file because DEC-083/DEC-086 order it
/// *before* the snapshot and it must survive a snapshot that never lands.
fn read_journal(root: &Path, slice: u32) -> Result<design_run::snapshot::CheckpointGroup> {
    let path = crate::state::design_journal_path(root, slice);
    match std::fs::read_to_string(&path) {
        Ok(text) => toml::from_str(&text).with_context(|| format!("parse {}", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(design_run::snapshot::CheckpointGroup::default())
        }
        Err(error) => Err(error).with_context(|| format!("read {}", path.display())),
    }
}

/// Replace the journal atomically. The only writer of that file.
fn store_journal(root: &Path, slice: u32, journal: &CheckpointGroup) -> Result<()> {
    let path = crate::state::design_journal_path(root, slice);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    crate::fsutil::write_atomic(&path, toml::to_string(journal)?.as_bytes())
}

/// Upsert one intent, keyed by `(submission, checkpoint)`.
///
/// Keyed by submission and not by checkpoint id alone, so a retry of the same
/// submission finds *its own* intent and resumes it rather than writing a second
/// one (DEC-086).
fn journal_intent(root: &Path, slice: u32, intent: &RecoveryIntent) -> Result<()> {
    let mut journal = read_journal(root, slice)?;
    journal.intents.retain(|held| {
        held.submission() != intent.submission() || held.checkpoint() != intent.checkpoint()
    });
    journal.intents.push(intent.clone());
    store_journal(root, slice, &journal)
}

/// The journalled intent for `(submission, checkpoint)`, if this submission has
/// been here before.
fn journalled_intent(
    root: &Path,
    slice: u32,
    submission: &str,
    checkpoint: &DesignId,
) -> Result<Option<RecoveryIntent>> {
    Ok(read_journal(root, slice)?
        .intents
        .into_iter()
        .find(|held| held.submission() == submission && held.checkpoint() == checkpoint))
}

/// Journal any candidate intent the shell did not already journal itself — the
/// note-bearing and pre-resolved forms, which ask for no authored effect and so
/// have no six-step execution of their own.
///
/// They land at [`IntentState::Journalled`], not `Complete`: the snapshot that
/// completes them has not been written yet, and a journal claiming an effect
/// landed before it did is the one state recovery cannot recover from.
fn write_journal(root: &Path, slice: u32, candidate: &DesignSnapshot) -> Result<()> {
    let mut journal = read_journal(root, slice)?;
    let mut added = false;
    for intent in &candidate.checkpoint.intents {
        let known = journal.intents.iter().any(|held| {
            held.submission() == intent.submission() && held.checkpoint() == intent.checkpoint()
        });
        if !known {
            let mut fresh =
                RecoveryIntent::journalled(intent.submission(), intent.checkpoint().clone());
            if let Some(record) = intent.reserved_record() {
                fresh = fresh.reserving(record);
            }
            journal.intents.push(fresh);
            added = true;
        }
    }
    if !added {
        return Ok(());
    }
    store_journal(root, slice, &journal)
}

/// DEC-086 step 6's second half: the snapshot carries the canonical disposition,
/// so every intent this submission journalled is now complete.
fn complete_journal(root: &Path, slice: u32, submission: &str) -> Result<()> {
    let mut journal = read_journal(root, slice)?;
    let mut changed = false;
    for intent in &mut journal.intents {
        if intent.submission() == submission && intent.state() != IntentState::Complete {
            *intent = intent.clone().reaching(IntentState::Complete);
            changed = true;
        }
    }
    if !changed {
        return Ok(());
    }
    store_journal(root, slice, &journal)
}

/// Emit lines without tripping the crate's `print_stdout` denial.
fn emit(lines: &[String]) -> Result<()> {
    let mut out = std::io::stdout().lock();
    for line in lines {
        writeln!(out, "{line}")?;
    }
    Ok(())
}

// ── the authored watermark ────────────────────────────────────────────────

/// What the authored tier looks like, relative to the watermark.
#[derive(Debug, Clone, PartialEq, Eq)]
enum AuthoredState {
    /// No `design.md`, and Doctrine has never written one. **Cold, not
    /// divergent** — there is nothing for the snapshot to have described.
    Cold,
    /// `design.md` is exactly as Doctrine last left it.
    Aligned,
    /// The bytes moved, or vanished after a materialisation.
    Diverged {
        expected: Option<String>,
        observed: Option<String>,
    },
}

/// Compare the authored tier against the watermark (DEC-092 rule 1).
///
/// The `materialised` flag is what makes "absent" answerable: without it, an
/// absent document before Doctrine ever wrote one and an absent document after
/// it did are the same observation, and one of those is cold while the other is
/// a deletion the run must not build on.
fn observe_watermark(run: &DesignSnapshot, observed: Option<&Fingerprint>) -> AuthoredState {
    let expected = run.authored.watermark.as_ref();
    match (expected, observed) {
        (None, None) if !run.authored.materialised => AuthoredState::Cold,
        (Some(expected), Some(observed)) if expected == observed => AuthoredState::Aligned,
        (expected, observed) => AuthoredState::Diverged {
            expected: expected.map(|f| f.as_str().to_owned()),
            observed: observed.map(|f| f.as_str().to_owned()),
        },
    }
}

/// The rule-1 entry check every ordinary mutating verb runs.
fn refuse_authored_divergence(run: &DesignSnapshot, observed: Option<&Fingerprint>) -> Result<()> {
    match observe_watermark(run, observed) {
        AuthoredState::Cold | AuthoredState::Aligned => Ok(()),
        AuthoredState::Diverged { expected, observed } => anyhow::bail!(
            "{DESIGN_DOC} has been edited outside this run — the watermark says `{}` and \
             Doctrine reads `{}`. Ordinary mutation is refused against prose the snapshot \
             no longer describes; re-adopt the document with an `adopt_authored` \
             declaration naming its exact current fingerprint.",
            expected.as_deref().unwrap_or("absent"),
            observed.as_deref().unwrap_or("absent"),
        ),
    }
}

/// What the pre-write re-check compares against (DEC-092 rule 3, RV-315 F-20).
///
/// Named in the type rather than selected by a boolean, because the two classes
/// compare against genuinely different things: a rule-2 re-adoption exists
/// *because* the bytes diverged from the watermark, so checking it against the
/// watermark would make every valid adoption refuse itself.
#[derive(Debug, Clone, PartialEq, Eq)]
enum PreWriteBasis {
    /// Rule-1 verbs: the watermark the run has been operating against.
    Watermark(Option<Fingerprint>),
    /// A rule-2 re-adoption: the exact fingerprint it was admitted on.
    AdmittedAt(Fingerprint),
}

/// Re-read and re-fingerprint `design.md` immediately before the snapshot write,
/// and abandon the write if the bytes moved since they were observed.
///
/// The guarantee this buys is **the run does not advance** — no snapshot, no
/// stage or gate movement. It is not a claim about the journal: Doctrine holds no
/// lock against a human editor, so an edit landing between this comparison and the
/// atomic rename is not caught by *this* invocation.
///
/// Whether it is caught at all depends on which file the write replaced, and the
/// distinction is easy to get wrong. Where the bytes hashed are not the bytes
/// replaced (a runtime-snapshot write), `design.md` still holds the other writer's
/// edit afterwards, so rule 1 refuses at the next entry — delayed detection, never
/// silent acceptance. Where they ARE the same file (`materialise`), the rename
/// destroys the edit, the read-back matches, and nothing is left for any later
/// check to see: that lost-update window is **consciously tolerated** for v0.0.1
/// (DEC-100). Do not read this comment as promising detection there.
fn recheck_watermark_before_write(root: &Path, slice: u32, basis: &PreWriteBasis) -> Result<()> {
    let observed = read_authored_fingerprint(root, slice)?;
    let expected = match basis {
        PreWriteBasis::Watermark(watermark) => watermark.clone(),
        PreWriteBasis::AdmittedAt(fingerprint) => Some(fingerprint.clone()),
    };
    if observed == expected {
        return Ok(());
    }
    anyhow::bail!(
        "{DESIGN_DOC} changed underneath this invocation — a hand-edit landed in the \
         pre-write window, so the run does not advance (no snapshot, no stage or gate \
         movement). Re-read the document and resubmit; any effect already journalled \
         remains and resumes under the same submission id."
    )
}

/// Re-baseline the watermark to the bytes the run now stands on. The only
/// routes here are a materialisation (Doctrine wrote them) and a validated
/// re-adoption (the caller proved it read them).
fn rebaseline_watermark(candidate: &mut DesignSnapshot, fingerprint: Option<Fingerprint>) {
    candidate.authored.watermark = fingerprint;
}

// ── the hooked seam ───────────────────────────────────────────────────────

/// The pre-write window as an injectable closure. In production it is a no-op;
/// a unit test injects a hand-edit here to fire deterministically, without
/// threads, in the exact window rule 3 must catch. Modelled on
/// `review.rs`'s `MidTurnHook` (DEC-092 cites that seam).
type PreWriteHook<'a> = &'a dyn Fn();

// ── DEC-086's six steps, and the fault seam that makes them testable ──────

/// The six DEC-086 steps, in order — a closed vocabulary with ONE owner
/// (STD-001), so the fault token, the doc and the call sites cannot drift.
///
/// Step 2 is the id **claim** and step 3 is the id **journal**, and the gap
/// between them is the whole recoverability argument: a crash before step 3 can
/// leave an empty or partial reservation but cannot leave an authored record
/// nobody can name, and from step 3 onward the journal names the exact target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckpointStep {
    /// 1 — persist the checkpoint intent, keyed by the apply submission id.
    IntentJournal,
    /// 2 — claim a fresh canonical id through the existing reservation backend.
    IdClaim,
    /// 3 — persist the claimed canonical id in the checkpoint journal.
    IdJournal,
    /// 4 — materialise the record scaffold at that held reservation.
    RecordMaterialise,
    /// 5 — apply the requested status and legal relation edges.
    EffectsApply,
    /// 6 — persist the design snapshot with the canonical disposition.
    SnapshotPersist,
}

/// **Debug builds only**, like the fault seam it exists for. The step tokens
/// are read by exactly one caller — `design_fault_hook`, matching
/// `DOCTRINE_DESIGN_FAULT` — which is itself `#[cfg(debug_assertions)]`. In a
/// release build there is no fault hook to name a step to, so an ungated `impl`
/// is dead code and `-D unused` fails the nix build with no compile error in
/// the dev loop to warn of it.
#[cfg(debug_assertions)]
impl CheckpointStep {
    /// The token each step is named by, at the fault seam and in prose (STD-001).
    const fn as_str(self) -> &'static str {
        match self {
            CheckpointStep::IntentJournal => "intent-journal",
            CheckpointStep::IdClaim => "id-claim",
            CheckpointStep::IdJournal => "id-journal",
            CheckpointStep::RecordMaterialise => "record-materialise",
            CheckpointStep::EffectsApply => "effects-apply",
            CheckpointStep::SnapshotPersist => "snapshot-persist",
        }
    }
}

/// A deterministic fault point, as an injectable closure — [`PreWriteHook`]'s
/// twin, and for the same reason.
///
/// Each of the six steps calls it immediately **before** performing that step's
/// effect, so "crash before step N" is expressible without threads, without
/// timing, and without a second code path. In production it is a no-op.
type FaultHook<'a> = &'a dyn Fn(CheckpointStep);

/// The env var naming the step to crash before, e.g.
/// `DOCTRINE_DESIGN_FAULT=id-journal`.
#[cfg(debug_assertions)]
const ENV_DESIGN_FAULT: &str = "DOCTRINE_DESIGN_FAULT";

/// The exit status an injected fault dies with — distinct from an ordinary
/// refusal's 1, so a test can tell "crashed where I asked" from "refused".
#[cfg(debug_assertions)]
const FAULT_EXIT_CODE: i32 = 70;

/// The fault hook, derived from the environment. **Debug builds only.**
///
/// A crash is not an error return: nothing unwinds, no destructor runs, and no
/// cleanup path fires — which is precisely the failure DEC-086's ordering is a
/// claim about. An `Err` here would exercise the recovery paths that already
/// work; only a hard exit exercises the ones that must.
///
/// The env is read **once**, when the hook is built, so the six call sites cannot
/// observe a different answer from each other.
#[cfg(debug_assertions)]
fn injected_fault() -> impl Fn(CheckpointStep) {
    // `var_os`, not `var`: `std::env::var` is a disallowed method in this crate
    // (`src/reserve.rs`'s `env_fallback_optin` is the precedent).
    let declared = std::env::var_os(ENV_DESIGN_FAULT);
    move |step| {
        let asked = declared
            .as_deref()
            .is_some_and(|value| value == std::ffi::OsStr::new(step.as_str()));
        if asked {
            #[expect(
                clippy::disallowed_methods,
                reason = "SL-233 EX-8: this SIMULATES a crash, so it must not unwind — \
                          returning an error would exercise a different path entirely. \
                          Debug-only and env-gated; compiled out of a release binary."
            )]
            std::process::exit(FAULT_EXIT_CODE);
        }
    }
}

/// The fault hook in a release build: no env read, no branch, no surface.
#[cfg(not(debug_assertions))]
fn injected_fault() -> impl Fn(CheckpointStep) {
    |_| {}
}

/// The env var naming a file whose bytes are written over `design.md` at the
/// simulated-editor point, e.g. `DOCTRINE_DESIGN_EDIT=/tmp/foreign.md`.
#[cfg(debug_assertions)]
const ENV_DESIGN_EDIT: &str = "DOCTRINE_DESIGN_EDIT";

/// A foreign write over `design.md`, injectable at one point in
/// [`materialise`]. **Debug builds only.** [`injected_fault`]'s twin, same shape
/// and same reasons: `var_os` not `var`, env read once, release no-op.
///
/// It fires **after** the atomic rename, which is the only point a reread can
/// observe (D7-R). It is deliberately *not* placed between the pre-write re-check
/// and the rename: an edit landing there is destroyed by the rename, so there is
/// nothing left for any in-process check to see, and simulating it would assert a
/// refusal Doctrine does not perform. That window is the residual v0.0.1 tolerates.
#[cfg(debug_assertions)]
fn injected_authored_edit() -> impl Fn(&Path) -> std::io::Result<()> {
    let declared = std::env::var_os(ENV_DESIGN_EDIT);
    move |path| match declared.as_deref() {
        None => Ok(()),
        Some(source) => {
            let bytes = std::fs::read(source)?;
            #[expect(
                clippy::disallowed_methods,
                reason = "SL-233 EX-6: this SIMULATES a writer that does NOT cooperate with \
                          Doctrine, so routing it through fsutil would model the wrong actor \
                          entirely. Debug-only and env-gated; compiled out of a release binary."
            )]
            std::fs::write(path, bytes)
        }
    }
}

/// The injected-editor hook in a release build: no env read, no branch, no surface.
#[cfg(not(debug_assertions))]
fn injected_authored_edit() -> impl Fn(&Path) -> std::io::Result<()> {
    |_| Ok(())
}

// ── DEC-086: the recoverable checkpoint ───────────────────────────────────

/// What one checkpoint asks Doctrine to *effect* on the authored tier.
///
/// The two note-bearing dispositions are absent by construction: they produce no
/// record, so they have no six-step execution and never reach here.
enum CheckpointEffect {
    /// Doctrine creates the record (DEC-086 steps 2–4).
    Create {
        kind: crate::knowledge::RecordKind,
        title: String,
        slug: String,
    },
    /// The record exists; only steps 5–6 remain.
    Adopt { record: String },
}

/// One checkpoint's plan: everything the six steps need, resolved against the
/// knowledge surface **before** any of them runs.
struct CheckpointPlan {
    checkpoint: DesignId,
    effect: CheckpointEffect,
    /// The user's acceptance, already bound to the digest Doctrine derived.
    acceptance: Option<AcceptanceAttestation>,
}

impl CheckpointPlan {
    /// The canonical ref this checkpoint will name, as far as it is known before
    /// execution. `Adopt` already knows it; `Create` stands in a **provisional**
    /// id of the right prefix so the candidate validates against a value of the
    /// real shape — the provisional candidate is discarded and never stored.
    fn provisional_record(&self) -> String {
        match &self.effect {
            CheckpointEffect::Create { kind, .. } => {
                crate::listing::canonical_id(kind.prefix(), PROVISIONAL_RECORD_ID)
            }
            CheckpointEffect::Adopt { record } => record.clone(),
        }
    }
}

/// DEC-088's binding, **derived by Doctrine**: the checkpoint payload's
/// fingerprint, the disposition it declares, and the run revision current when
/// the acceptance was given.
///
/// The agent supplies the basis, never this. Rebinding an acceptance to different
/// content, a different disposition, or a later revision changes the digest, so a
/// transplanted acceptance stops matching what it claims to accept.
fn acceptance_digest(payload: &str, disposition: &str, node: &str, revision: u64) -> String {
    crate::git::sha256(
        [payload, disposition, node, &revision.to_string()]
            .join(ACCEPTANCE_BINDING_SEPARATOR)
            .as_bytes(),
    )
}

/// Resolve every checkpoint that asks for an authored effect, refusing before
/// anything is written.
///
/// Everything that can be refused is refused here: an unknown knowledge kind, an
/// underivable slug, an adoption target that does not exist or has been withdrawn
/// from the corpus (EX-4). Nothing here mutates.
///
/// **Both origins, one protocol (`EX-1`, RV-324 F-1).** An accepted proposal's
/// declarations are the coordinator's own act, and the pure core has always
/// treated them as one batch (`run.rs`'s merge). This walk used to see only
/// `request.declare`, so a *proposed* checkpoint reached the core with no shell
/// work done for it at all: a proposed `adopt` skipped [`knowledge::adoptable`]
/// and could record an adoption against a nonexistent, wrong-kind or withdrawn
/// record, while a proposed `create` never entered DEC-086's protocol and was
/// refused as unresolved. The chain here is the same idiom [`section_digests`]'s
/// caller already used for exactly the same reason, and for exactly the same
/// pair of origins.
fn plan_checkpoints(
    root: &Path,
    prior: &DesignSnapshot,
    request: &ApplyRequest,
) -> Result<Vec<CheckpointPlan>> {
    let mut plans = Vec::new();
    for declaration in request
        .declare
        .iter()
        .chain(accepted_declarations(prior, request))
    {
        if declaration.subject().kind() != IdKind::Checkpoint {
            continue;
        }
        // A checkpoint declaring no disposition is skipped rather than refused
        // here: the pure core owns that refusal
        // ([`Refusal::CheckpointDispositionMissing`]) and a second one here
        // would be a second implementation of it. Planning only resolves the
        // declarations that ask for an authored effect.
        let Some(dispose) = declaration.dispose() else {
            continue;
        };
        let (effect, acceptance) = match dispose {
            Dispose::Create(create) => {
                let kind = crate::knowledge::RecordKind::from_str(&create.kind, true)
                    .map_err(|known| anyhow::anyhow!("{known}"))
                    .with_context(|| {
                        format!(
                            "checkpoint {} asks to create a `{}` record",
                            declaration.subject(),
                            create.kind
                        )
                    })?;
                let title = crate::input::resolve_title(Some(create.title.clone()))?;
                let slug = crate::input::resolve_slug(&title, create.slug.clone())?;
                (
                    CheckpointEffect::Create { kind, title, slug },
                    create.acceptance.as_ref(),
                )
            }
            Dispose::Adopt { record } => {
                // EX-4: kind and usable status, checked against the real record.
                crate::knowledge::adoptable(root, record)?;
                (
                    CheckpointEffect::Adopt {
                        record: record.clone(),
                    },
                    None,
                )
            }
            Dispose::Unresolved { .. } | Dispose::NonDurable { .. } => continue,
        };
        let acceptance = acceptance
            .map(|declared| -> Result<AcceptanceAttestation> {
                // Refuse rather than default: a discarded serialisation error
                // would digest the EMPTY STRING, so every declaration that
                // failed to serialise would attest identically and silently.
                let payload = crate::git::sha256(serde_json::to_string(declaration)?.as_bytes());
                let node = declaration.disposes().map_or("", DesignId::as_str);
                Ok(AcceptanceAttestation::bind(
                    declared.basis.clone(),
                    declared.turn.clone(),
                    Fingerprint::new(acceptance_digest(
                        &payload,
                        dispose.form().as_str(),
                        node,
                        prior.run.revision,
                    )),
                ))
            })
            .transpose()?;
        plans.push(CheckpointPlan {
            checkpoint: declaration.subject().clone(),
            effect,
            acceptance,
        });
    }
    Ok(plans)
}

/// Run DEC-086 steps 1–5 for one checkpoint, resuming the first incomplete
/// effect, and return the canonical record it names.
///
/// Every step is guarded by the journalled state, so a retry of the same
/// submission repeats nothing: from [`IntentState::Reserved`] onward the resumed
/// work runs against the id the journal names, **never against a fresh one**.
/// Nothing here removes or rewrites an authored record to repair a runtime
/// failure (DEC-083) — a step that cannot be resumed is reported, and the run
/// does not advance.
fn execute_checkpoint(
    root: &Path,
    slice: u32,
    submission: &str,
    plan: &CheckpointPlan,
    fault: FaultHook<'_>,
) -> Result<String> {
    // Step 1 — the intent, before anything else exists.
    let held = journalled_intent(root, slice, submission, &plan.checkpoint)?;
    let mut intent = if let Some(held) = held {
        held
    } else {
        fault(CheckpointStep::IntentJournal);
        let fresh = RecoveryIntent::journalled(submission, plan.checkpoint.clone())
            .accepted(plan.acceptance.clone());
        journal_intent(root, slice, &fresh)?;
        fresh
    };

    match &plan.effect {
        // An adoption needs no reservation: the id exists, so step 3 records it
        // directly and step 4 has nothing to materialise.
        CheckpointEffect::Adopt { record } => {
            if intent.state() < IntentState::Materialised {
                fault(CheckpointStep::IdJournal);
                intent = intent.reserving(record).reaching(IntentState::Materialised);
                journal_intent(root, slice, &intent)?;
            }
        }
        CheckpointEffect::Create { kind, title, slug } => {
            if intent.state() < IntentState::Reserved {
                // Steps 2–4, with step 3 riding the id-claim midpoint.
                fault(CheckpointStep::IdClaim);
                let base = intent.clone();
                crate::knowledge::create_record(root, *kind, title, slug, |_id, canonical| {
                    fault(CheckpointStep::IdJournal);
                    journal_intent(
                        root,
                        slice,
                        &base
                            .clone()
                            .reserving(canonical)
                            .reaching(IntentState::Reserved),
                    )?;
                    fault(CheckpointStep::RecordMaterialise);
                    Ok(())
                })?;
                // Re-read rather than reconstruct: the id this run continues
                // against is the one the JOURNAL holds, which is the same fact
                // recovery would read.
                intent = journalled_intent(root, slice, submission, &plan.checkpoint)?
                    .context("the claimed canonical id was journalled but cannot be read back")?
                    .reaching(IntentState::Materialised);
                journal_intent(root, slice, &intent)?;
            } else if intent.state() < IntentState::Materialised {
                // Resume: the id is journalled, the bytes are not. Step 4 runs
                // against the EXACT reserved id.
                let reserved = intent
                    .reserved_record()
                    .context("a reserved checkpoint intent with no canonical id")?
                    .to_owned();
                let (held_kind, id) = crate::knowledge::resolve_ref(&reserved)?;
                fault(CheckpointStep::RecordMaterialise);
                crate::knowledge::materialise_record_at(root, held_kind, id, title, slug)?;
                intent = intent.reaching(IntentState::Materialised);
                journal_intent(root, slice, &intent)?;
            }
        }
    }

    let record = intent
        .reserved_record()
        .context("a materialised checkpoint intent with no canonical id")?
        .to_owned();

    // Step 5 — status and the legal relation edges.
    if intent.state() < IntentState::Applied {
        fault(CheckpointStep::EffectsApply);
        apply_record_effects(root, slice, &record, intent.acceptance())?;
        intent = intent.reaching(IntentState::Applied);
        journal_intent(root, slice, &intent)?;
    }
    Ok(record)
}

/// DEC-086 step 5: the record's requested status and its legal relation edges.
///
/// Both are idempotent, which is what lets a resumed step 5 simply run again:
/// `set_authored_status` writes only on a change, and `append_edge` returns
/// `Noop` when the edge is already there — so "apply the edge when absent" needs
/// no absence check of its own.
fn apply_record_effects(
    root: &Path,
    slice: u32,
    record: &str,
    acceptance: Option<&AcceptanceAttestation>,
) -> Result<()> {
    let (kind, id) = crate::knowledge::resolve_ref(record)?;
    // EX-6/DEC-088: only a user-acceptance attestation moves a created record off
    // its kind's seeded default. A payload cannot ask for `accepted`; there is no
    // field for it, and this is the only route to the state.
    if acceptance.is_some()
        && let Some(state) = crate::knowledge::accepted_status(kind)
    {
        crate::knowledge::set_record_status(root, kind, id, state)?;
    }
    let target = crate::listing::canonical_id(crate::kinds::SL, slice);
    let outcome = crate::relation::append_edge(
        &crate::knowledge::record_toml_path(root, kind, id),
        crate::relation::RelationLabel::Shapes,
        None,
        None,
        None,
        &target,
    )?;
    debug_assert!(
        matches!(
            outcome,
            crate::relation::AppendOutcome::Wrote | crate::relation::AppendOutcome::Noop
        ),
        "append_edge is total over its outcome vocabulary"
    );
    Ok(())
}

// `bind_resolved` lived here. It walked `request.declare`, where an accepted
// proposal's declarations never appear, so it could not reach them however it
// was called (`EX-3`). The binding now happens in `design_run::run::apply`, at
// the point the two origins become one batch — see D1 there.

// ── start ─────────────────────────────────────────────────────────────────

fn run_start(args: StartArgs) -> Result<()> {
    let root = resolve_root(args.path)?;
    let slice = slice_id(&args.slice)?;
    start(&root, slice, args.from_design, &|| {})
}

/// Create the run. One active run per slice in v1, so an existing snapshot is a
/// refusal rather than a silent replacement.
fn start(root: &Path, slice: u32, from_design: bool, pre_write: PreWriteHook<'_>) -> Result<()> {
    let path = crate::state::design_snapshot_path(root, slice);
    if path.exists() {
        anyhow::bail!(
            "slice {slice:03} already has a design run at {} — v1 holds one active run \
             per slice; use `doctrine design show` to read it",
            path.display()
        );
    }
    // Read the bytes ONCE. Import fingerprints and decomposes the same string,
    // so a second read could see a different document than the watermark
    // certifies — the divergence DEC-092 exists to make impossible.
    let document = read_design_doc(root, slice)?;
    let observed = document.as_deref().map(authored_fingerprint);
    if observed.is_some() && !from_design {
        anyhow::bail!(
            "slice {slice:03} already has a {DESIGN_DOC} — start the run with \
             `--from-design` to adopt it as the run's baseline, which records its \
             fingerprint as the authored watermark"
        );
    }
    let uid = [RUN_UID_PREFIX, &uuid::Uuid::now_v7().to_string()].concat();
    let mut run = DesignSnapshot::new(uid.clone(), slice, observed.clone());
    run.authored.materialised = observed.is_some();

    // The import itself. It is refusable, and it is refused here — before
    // `pre_write`, so a document import cannot read leaves no run behind at all.
    let imported = match document.as_deref().filter(|_| from_design) {
        Some(text) => import_authored(root, slice, &mut run, text)?,
        None => 0,
    };

    pre_write();
    recheck_watermark_before_write(root, slice, &PreWriteBasis::Watermark(observed))?;
    write_snapshot(root, slice, &run)?;
    let mut lines = vec![
        format!(
            "run {uid} revision {} stage {}",
            run.run.revision,
            run.run.stage.as_str()
        ),
        format!(
            "snapshot {}",
            crate::state::design_snapshot_path(root, slice).display()
        ),
    ];
    if from_design {
        // The honest label, on EVERY import (`EX-6`, DEC-057). Import IS the
        // runtime-loss path (design §5.5), and it cannot tell a first adoption
        // from a reconstruction after loss — nothing observable distinguishes
        // them once the snapshot is gone. So it does not guess: what it says is
        // true of both, and it says the weakness out loud rather than letting a
        // caller read a fresh uid as a resumed run.
        lines.push(format!(
            "reconstructed {imported} section(s) from {DESIGN_DOC} as unreviewed prose — \
             a NEW run, weaker than exact resume: no attestation, receipt or gate \
             clearance is inferred from authored prose"
        ));
    }
    emit(&lines)
}

/// Seat every region of a legacy `design.md` into a fresh run, returning how
/// many were seated (`EX-2`).
///
/// The shell's whole contribution is the digests and the linked knowledge —
/// [`design_run::legacy`] owns the decomposition and [`design_run::run::import`]
/// owns the seating, so this seam surfaces a refusal rather than deciding
/// anything, exactly as [`authored_sections`] does for the adoption path.
fn import_authored(root: &Path, slice: u32, run: &mut DesignSnapshot, text: &str) -> Result<usize> {
    let regions: Vec<(design_run::legacy::Region<'_>, Fingerprint)> =
        design_run::legacy::read(text)
            .map_err(|refused| refusal(&refused))?
            .into_iter()
            .map(|region| {
                let fingerprint = authored_fingerprint(region.body);
                (region, fingerprint)
            })
            .collect();
    let entry_digests = entry_digests(&regions);
    Ok(design_run::run::import(
        run,
        &regions,
        &shaping_questions(root, slice),
        &entry_digests,
    )
    .map_err(|refused| refusal(&refused))?
    .len())
}

/// The digest of every conventional `OQ-*` entry's **headline**, keyed by the
/// document line the entry stands on — the shell's job, because the pure core
/// never hashes (`design_run` is a leaf of crate out-degree zero).
///
/// This is [`section_digests`]'s idiom one level down, and it is *additive*: no
/// existing derivation is merged into it (PHASE-15 `EX-12`).
///
/// **Extent (DEC-085 via PHASE-15 D8): the headline alone** — the exact bytes
/// the node stores as its question. Not the continuation lines, which import
/// deliberately leaves in the section body, so hashing them would invalidate a
/// node's provenance when the content it represents had not changed. Not the
/// entry text past the label either: the supported bold form carries same-line
/// prose after the headline.
///
/// Keyed by line rather than by `(section, line)`: section ids are minted by
/// [`design_run::run::import`] from region position, and duplicating that rule
/// here to build a compound key would be two implementations of one identity.
/// A line is document-global and unique, which is all the key needs to be.
///
/// Derived over EVERY region rather than only the Open Questions ones, so the
/// map is a superset of what the core will look up and its totality is a
/// property of both sides calling the same parse on the same regions — not of
/// this function reproducing the core's section-title test.
fn entry_digests(
    regions: &[(design_run::legacy::Region<'_>, Fingerprint)],
) -> std::collections::BTreeMap<usize, Fingerprint> {
    regions
        .iter()
        .flat_map(|(region, _)| design_run::legacy::open_questions(region))
        .map(|entry| {
            (
                entry.line,
                Fingerprint::new(crate::git::sha256(entry.question.as_bytes())),
            )
        })
        .collect()
}

/// The slice's **direct non-terminal shaping QUEs** — the linked knowledge
/// import seeds durable inquiry nodes from (`EX-3`, DEC-085).
///
/// Direct: one hop, the `shapes` edge the record itself authored (ADR-004 stores
/// outbound only, so this reads the source side and never a derived reciprocal).
/// Non-terminal: a settled question is not an open line of inquiry, and seeding
/// it would hand the run work somebody already finished.
///
/// A corpus that cannot be scanned yields **no** shaping questions rather than a
/// failed import: a legacy `design.md` in a tree with no knowledge records is
/// precisely the case import exists for, and refusing it would make the linked
/// half of DEC-085 a precondition of the authored half.
fn shaping_questions(root: &Path, slice: u32) -> Vec<design_run::run::ShapingQuestion> {
    let target = crate::listing::canonical_id(crate::kinds::SLICE_KIND.prefix, slice);
    let Ok(scanned) = crate::catalog::scan::scan_entities(
        root,
        &mut Vec::new(),
        crate::catalog::scan::ScanMode::default(),
    ) else {
        return Vec::new();
    };
    scanned
        .into_iter()
        .filter(|entity| entity.kind.prefix == crate::kinds::QUESTION_KIND.prefix)
        .filter(|entity| {
            !entity
                .status
                .as_deref()
                .is_some_and(|status| crate::knowledge::RecordKind::Question.is_terminal(status))
        })
        .filter(|entity| {
            crate::relation::targets_for(&entity.outbound, crate::relation::RelationLabel::Shapes)
                .iter()
                .any(|shaped| shaped == &target)
        })
        .map(|entity| design_run::run::ShapingQuestion {
            record: entity.key.canonical(),
            question: entity.title,
        })
        .collect()
}

// ── apply ─────────────────────────────────────────────────────────────────

fn run_apply(args: ApplyArgs) -> Result<()> {
    let root = resolve_root(args.path)?;
    let slice = slice_id(&args.slice)?;
    let payload = read_payload(&args.input)?;
    let fault = injected_fault();
    apply(&root, slice, &payload, &|| {}, &fault)
}

/// Resolve the payload from `--input`: stdin, a literal JSON object, or a file.
fn read_payload(input: &str) -> Result<String> {
    if input == STDIN_SENTINEL {
        let mut text = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut text)
            .context("read payload from stdin")?;
        return Ok(text);
    }
    if input.trim_start().starts_with('{') {
        return Ok(input.to_owned());
    }
    std::fs::read_to_string(input).with_context(|| format!("read payload from {input}"))
}

/// Validate and apply one sparse idempotent mutation.
///
/// The ordering is the contract, and it is DEC-083/DEC-086's: admit, validate
/// the whole candidate, run each checkpoint's six steps, re-check the watermark,
/// then land the snapshot. An abandoned write therefore leaves the journal
/// behind on purpose — the guarantee is that the run does not advance.
///
/// **Why the candidate is validated twice.** A `create` disposition has no
/// canonical id until DEC-086 step 3 has claimed one, and DEC-063 says the whole
/// candidate is validated before any mutation. So the first pass validates
/// against a *provisional* id of the right shape and its result is discarded: an
/// unknown node, a cycle, a duplicate subject or an uncleared gate is refused
/// while the authored tier is still untouched. The second pass — the one whose
/// snapshot is stored — runs against the ids actually claimed. The pure core is
/// a function of its inputs, so running it twice costs a clone and buys the
/// guarantee that a refusable batch never leaves an orphaned record behind.
fn apply(
    root: &Path,
    slice: u32,
    payload: &str,
    pre_write: PreWriteHook<'_>,
    fault: FaultHook<'_>,
) -> Result<()> {
    let prior = read_snapshot(root, slice)?;
    let request: ApplyRequest =
        serde_json::from_str(payload).context("parse the apply payload as JSON")?;
    let digest = crate::git::sha256(payload.as_bytes());

    match design_run::run::admit(&prior, &request.envelope, &digest)
        .map_err(|refused| refusal(&refused))?
    {
        Admission::Resumed { revision } => {
            return emit(&[format!(
                "resumed submission {} — already applied at revision {revision}; \
                 the run does not advance",
                request.envelope.submission_id
            )]);
        }
        Admission::Fresh => {}
    }

    let observed = read_authored_fingerprint(root, slice)?;
    let readopting = request.adopt_authored.is_some();
    if !readopting {
        refuse_authored_divergence(&prior, observed.as_ref())?;
    }

    // The authored read is the ADOPTION path's alone. Nothing else consumes it,
    // and reading it unconditionally would apply the §5.5 document checks to a
    // run whose document is not being adopted (a `--from-design` baseline that
    // predates the markers is the obvious casualty). An ABSENT document is left
    // to the pure layer too: §5.5 classifies how a document departs from the
    // run, and `AdoptionStale` — "design.md is absent" — is the accurate answer
    // where there is no document to classify.
    let authored = match read_design_doc(root, slice)?.filter(|_| readopting) {
        Some(text) => authored_sections(&text, &prior.sections.ids())?,
        None => std::collections::BTreeMap::new(),
    };
    let runbook = runbook_facts(prior.run.stage)?;
    let derived = DerivedInput {
        // An `accept` contributes the delegate's stored declarations to this
        // batch, so their bodies need digesting too — the pure core never hashes,
        // and a proposed section whose digest nobody computed would be refused as
        // bodyless for a reason the caller cannot see.
        section_digests: section_digests(
            request
                .declare
                .iter()
                .chain(accepted_declarations(&prior, &request)),
        ),
        authored_sections: authored,
        authored_fingerprint: observed.clone(),
        verifications: verifications(root, slice, &prior, &request, runbook.as_ref()),
        runbook,
    };

    // Everything refusable, refused while the authored tier is untouched.
    let plans = plan_checkpoints(root, &prior, &request)?;
    let provisional: std::collections::BTreeMap<DesignId, String> = plans
        .iter()
        .map(|plan| (plan.checkpoint.clone(), plan.provisional_record()))
        .collect();
    design_run::run::apply(&prior, &request, &derived, &digest, &provisional)
        .map_err(|refused| refusal(&refused))?;

    // DEC-086 steps 1–5, per checkpoint, resuming the first incomplete effect.
    let mut resolved: std::collections::BTreeMap<DesignId, String> =
        std::collections::BTreeMap::new();
    for plan in &plans {
        let record = execute_checkpoint(root, slice, &request.envelope.submission_id, plan, fault)?;
        resolved.insert(plan.checkpoint.clone(), record);
    }

    let mut applied = design_run::run::apply(&prior, &request, &derived, &digest, &resolved)
        .map_err(|refused| refusal(&refused))?;

    // Rule 2: the watermark re-baselines only after the candidate validates in
    // full, and only on the adoption path.
    if readopting {
        rebaseline_watermark(&mut applied.snapshot, observed.clone());
    }

    // DEC-083/DEC-086: the journal is ordered BEFORE the snapshot. The effected
    // checkpoints already journalled themselves; this catches the forms that ask
    // for no authored effect.
    write_journal(root, slice, &applied.snapshot)?;

    pre_write();
    let basis = match (readopting, observed) {
        (true, Some(fingerprint)) => PreWriteBasis::AdmittedAt(fingerprint),
        (true, None) => PreWriteBasis::Watermark(None),
        (false, _) => PreWriteBasis::Watermark(prior.authored.watermark),
    };
    recheck_watermark_before_write(root, slice, &basis)?;
    // Step 6.
    fault(CheckpointStep::SnapshotPersist);
    write_snapshot(root, slice, &applied.snapshot)?;
    complete_journal(root, slice, &request.envelope.submission_id)?;

    let mut lines = vec![format!(
        "revision {} stage {}",
        applied.snapshot.run.revision,
        applied.snapshot.run.stage.as_str()
    )];
    // EX-5 / design R12: the residual risk is stated **in the surface**, at the
    // moment the claim is made, and not only in `design.md`. A lock rests on an
    // agent's attributed report that the user accepted — auditable, and not
    // authentication. Saying so where the lock is taken is the difference between
    // a disclosed limit and a buried one.
    if applied.snapshot.run.stage == Stage::Locked && prior.run.stage != Stage::Locked {
        lines.push(LOCK_ACCEPTANCE_DISCLOSURE.to_owned());
    }
    lines.extend(assignment_lines(
        &applied.snapshot,
        request.delegation.as_ref(),
    ));
    lines.extend(applied.rows.iter().map(design_run::render::render_row));
    emit(&lines)
}

/// The digest of every declared section body — the shell's job, because the pure
/// core never hashes.
///
/// Takes the declarations rather than the request so the one function serves both
/// the payload's own and an accepted proposal's: two callers computing digests two
/// ways is how the two sets end up disagreeing.
fn section_digests<'a>(
    declarations: impl Iterator<Item = &'a Declaration>,
) -> std::collections::BTreeMap<DesignId, Fingerprint> {
    declarations
        .filter_map(|declaration| {
            declaration.body().map(|body| {
                (
                    declaration.subject().clone(),
                    Fingerprint::new(crate::git::sha256(body.as_bytes())),
                )
            })
        })
        .collect()
}

/// The declarations an `accept` contributes, read off the **prior** snapshot.
///
/// The delegate's bytes are already stored; acceptance is the coordinator making
/// them its own. Read here only so their digests can be computed — the pure core
/// reads the same proposal out of the same snapshot, so neither side is trusting
/// the other's copy.
fn accepted_declarations<'a>(
    prior: &'a DesignSnapshot,
    request: &ApplyRequest,
) -> impl Iterator<Item = &'a Declaration> {
    let accepted = match request.delegation.as_ref() {
        Some(DelegationAct::Accept { id }) => prior.delegation.find(id.as_str()),
        _ => None,
    };
    accepted
        .and_then(Delegation::proposal)
        .map_or(&[][..], |proposal| proposal.declarations())
        .iter()
}

/// The assignment an `export` cut, as self-contained lines (`EX-1`).
///
/// Everything a fresh session needs and nothing it has to go and read: the run it
/// belongs to, the assignment's own id, the obligation **and its question text**,
/// the revision it was cut at, and the proposal-only contract. Unbudgeted, like
/// every rendering that is not the `prompt` projection — an assignment that
/// elided its own question would not be self-contained.
fn assignment_lines(run: &DesignSnapshot, act: Option<&DelegationAct>) -> Vec<String> {
    let Some(DelegationAct::Export { id, .. }) = act else {
        return Vec::new();
    };
    let Some(delegation) = run.delegation.find(id.as_str()) else {
        return Vec::new();
    };
    vec![
        format!(
            "assignment {} run {} obligation {} exported_at {}",
            delegation.id(),
            run.run.uid,
            delegation.obligation(),
            delegation.exported_at()
        ),
        format!("obligation_question {}", delegation.question()),
        ASSIGNMENT_CONTRACT.to_owned(),
    ]
}

/// A pure-core refusal, crossing into the shell's error channel. The *data* is
/// the contract; this is the one place it becomes prose.
fn refusal(refused: &design_run::refusal::Refusal) -> anyhow::Error {
    anyhow::anyhow!("{refused}")
}

// ── materialise ───────────────────────────────────────────────────────────

fn run_materialise(args: MaterialiseArgs) -> Result<()> {
    let root = resolve_root(args.path)?;
    let slice = slice_id(&args.slice)?;
    materialise(&root, slice, &|| {})
}

/// `doctrine slice design <id>` — the DEPRECATED compatibility shim
/// (SL-233 PHASE-14 EX-4, design §5.2).
///
/// Two arms, mutually exclusive with each other exactly as the legacy fallback
/// and the managed writer are mutually exclusive (DEC-075):
///
/// - **live run** — forward to [`materialise`], the function `design
///   materialise` itself calls. Not a copy of it: the foreign-edit guard, the
///   renderer, the pre-write re-check and the re-baseline are reached through
///   ONE seam, so the two verbs produce identical bytes and identical refusals
///   by construction rather than by two implementations agreeing;
/// - **no run** — [`crate::slice::scaffold_design_doc`], the legacy
///   scaffold-only contract, unchanged.
///
/// The entry lives HERE rather than in `crate::slice` because `materialise` is
/// this module's own: routing the other way would put a production
/// `slice → commands` edge opposite the `commands → slice` edge the CLI
/// dispatch already carries, closing a command-tier cycle (ADR-001). The notice
/// it emits stays a named constant in `crate::slice` beside the incumbent it
/// deprecates (STD-001).
///
/// The notice goes to **stderr**, before anything can fail: stdout carries the
/// command's own output, and a warning that arrives only on the success path is
/// not emitted on every invocation.
pub(crate) fn run_deprecated_slice_design(path: Option<PathBuf>, slice: u32) -> Result<()> {
    writeln!(
        std::io::stderr(),
        "{}",
        crate::slice::DESIGN_DEPRECATION_NOTICE
    )?;
    let root = resolve_root(path)?;
    // Presence of the snapshot IS the live-run question — the same file
    // `read_snapshot` refuses on, asked without turning its absence into an
    // error the legacy arm would have to interpret.
    if crate::state::design_snapshot_path(&root, slice).exists() {
        return materialise(&root, slice, &|| {});
    }
    crate::slice::scaffold_design_doc(&root, slice)
}

/// Render the run's sections into authored prose, then re-baseline.
fn materialise(root: &Path, slice: u32, pre_write: PreWriteHook<'_>) -> Result<()> {
    let foreign_edit = injected_authored_edit();
    let prior = read_snapshot(root, slice)?;
    let observed = read_authored_fingerprint(root, slice)?;
    refuse_authored_divergence(&prior, observed.as_ref())?;

    let body = render_document(&prior);
    let path = design_doc_path(root, slice);

    pre_write();
    recheck_watermark_before_write(
        root,
        slice,
        &PreWriteBasis::Watermark(prior.authored.watermark.clone()),
    )?;

    // `write_body` is the product's prose-write primitive and takes `dir` + `file`
    // rather than a composed path, so kind layout stays here — including creating
    // the directory, which it does not do. `Replace` stores `text` verbatim, so
    // byte-exactness survives the seam. It also skips the write when the bytes are
    // byte-identical; the revision, the watermark and the change-log floor below
    // are unconditional either way, so a no-op materialise is still a lawful
    // revision that produces no material rows.
    let dir = path
        .parent()
        .with_context(|| format!("{} has no parent directory", path.display()))?;
    std::fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
    crate::entity::write_body(dir, DESIGN_DOC, &body, crate::entity::BodyMode::Replace)?;
    foreign_edit(&path)?;

    // DEC-100: re-baseline only to bytes demonstrably on disk, tolerating the
    // check-to-rename lost-update window. (DEC-105 is the superseded record —
    // its writer-lock-plus-CAS mechanism is NOT what landed.)
    //
    // The watermark below is a claim about `design.md`, so it is derived from what
    // `design.md` now *reads* rather than from what this process rendered. Deriving
    // it from `body` made the claim unfalsifiable: any write landing after the
    // re-check above was certified as Doctrine's own, and rule 1 at the next entry
    // then saw an aligned document, so the divergence became permanently invisible.
    //
    // This is NOT detection of an overwritten edit, and must not be described as
    // one. A write landing between the re-check and the rename is destroyed by the
    // rename; the bytes below then match and the run advances. That lost-update
    // window is consciously tolerated for v0.0.1 (SPEC-029). What this closes is
    // narrower and worth having on its own: Doctrine never certifies bytes it did
    // not write.
    let rendered = authored_fingerprint(&body);
    let settled = read_authored_fingerprint(root, slice)?;
    if settled.as_ref() != Some(&rendered) {
        anyhow::bail!(
            "{DESIGN_DOC} was written, then changed again before Doctrine could \
             confirm it — Doctrine wrote `{}` and now reads `{}`. The snapshot was \
             NOT advanced: its watermark still describes the previous revision, so \
             nothing has certified prose this run did not write. The document now \
             holds the other writer's bytes; re-run `design materialise` once it is \
             settled, or re-adopt it with an `adopt_authored` declaration naming its \
             exact current fingerprint.",
            rendered.as_str(),
            settled.as_ref().map_or("absent", Fingerprint::as_str),
        );
    }

    let mut candidate = prior;
    candidate.run.revision = candidate.run.revision.saturating_add(1);
    candidate.authored.materialised = true;
    rebaseline_watermark(&mut candidate, Some(rendered));
    // A materialisation is a revision that produces no material rows — which is
    // exactly why the change-log floor is recorded rather than inferred from the
    // oldest surviving row.
    candidate
        .change_log
        .record(candidate.run.revision, Vec::new());
    write_snapshot(root, slice, &candidate)?;
    emit(&[format!(
        "materialised {} at revision {}",
        path.display(),
        candidate.run.revision
    )])
}

/// The authored document: one marker-addressed block per section, in **document
/// order** (`seq`) — which is not the id order `SectionGroup` keeps for
/// serialisation determinism.
///
/// The framing itself belongs to [`design_run::document::render`], with the
/// parse that inverts it. This function's whole job is choosing the order.
fn render_document(run: &DesignSnapshot) -> String {
    design_run::document::render(
        run.sections
            .document_order()
            .into_iter()
            .map(|section| (&section.id, section.body.as_str())),
    )
}

// ── show ──────────────────────────────────────────────────────────────────

/// The baseline a read projects against: the caller's declared revision, or the
/// immediately previous one.
fn baseline(run: &DesignSnapshot, declared: Option<u64>) -> u64 {
    declared.unwrap_or_else(|| run.run.revision.saturating_sub(1))
}

/// Project one turn. The shell never applies a bound — it receives an
/// already-bounded envelope and chooses a rendering (DEC-064).
fn project(run: &DesignSnapshot, known: u64, detail: Detail) -> Result<TurnEnvelope> {
    envelope::project(run, known, detail).map_err(|refused| refusal(&refused))
}

fn run_show(args: ShowArgs) -> Result<()> {
    let root = resolve_root(args.path)?;
    let slice = slice_id(&args.slice)?;
    let run = read_snapshot(&root, slice)?;
    let known = baseline(&run, args.known_revision);
    let detail = if args.full {
        Detail::Full
    } else {
        Detail::Normal
    };
    let turn = project(&run, known, detail)?;
    match args.format {
        ShowFormat::Prompt => emit(&envelope::prompt(&turn)),
        ShowFormat::Status => emit(&envelope::status(&turn)),
        ShowFormat::Json => {
            emit(
                &[serde_json::to_string_pretty(&turn)
                    .context("render the turn envelope as JSON")?],
            )
        }
    }
}

// ── resume ────────────────────────────────────────────────────────────────

fn run_resume(args: ResumeArgs) -> Result<()> {
    let root = resolve_root(args.path)?;
    let slice = slice_id(&args.slice)?;
    let run = read_snapshot(&root, slice)?;

    // `--run` is an explicit assumption CHECK, not addressing: absent, resume
    // still works; present and wrong, it refuses rather than resuming against a
    // run the caller did not mean.
    if let Some(declared) = args.run.as_ref()
        && declared != &run.run.uid
    {
        anyhow::bail!(
            "this slice's live design run is `{}`, not `{declared}` — the context you are \
             resuming from is stale; drop `--run` to resume the live one",
            run.run.uid
        );
    }

    let known = baseline(&run, args.known_revision);
    let turn = project(&run, known, Detail::Normal)?;
    let mut lines = envelope::resume(&turn);
    lines.extend(fragment_lines(&run, &args.known_fragment));
    lines.extend(fragment_section(&run, &args.known_fragment)?);
    lines.extend(runbook_section(&run)?);
    emit(&lines)
}

/// The runbook obligation section of a turn read — the shell half of `A4`.
///
/// Beside [`fragment_section`] and for the same reason: the step text comes from
/// the embed, so the asset read is Doctrine's and the *rendering* is the pure
/// core's ([`design_run::runbook::Runbook::section`]). This function joins them
/// and owns nothing else.
///
/// `&[]` for the verifications is the honest input, not a stub: a read runs no
/// checks. What that costs is stated where the rendering states it.
///
/// A stage whose outbound edge carries no runbook renders nothing — the same
/// real answer [`fragment_section`] gives a locked run.
fn runbook_section(run: &DesignSnapshot) -> Result<Vec<String>> {
    let Some(facts) = runbook_facts(run.run.stage)? else {
        return Ok(Vec::new());
    };
    Ok(facts
        .book
        .section(&run.runbook.discharges, &facts.digests, &[]))
}

/// The runbook guarding `stage`'s outbound edge, read and digested — the shell
/// half of `EX-18`'s split (SL-233 PHASE-16).
///
/// Doctrine hashes; the pure core compares. [`design_run::runbook::Step::material`]
/// is the canonical, version-tagged encoding and lives in the leaf; the
/// `sha256` over it is here, because a leaf may not reach the impure `git` seam
/// without losing the out-degree-zero property `tests/e2e_design_*.rs` compile
/// against.
///
/// Resolved from the **embed**, through the same path the process fragments use.
/// Per the owner's 2026-07-31 ruling (b) the project-override seam is identified
/// and deferred (IMP-372), so there is deliberately no project-path lookup here.
///
/// A stage whose outbound edge carries no runbook yields `None`, which is a real
/// answer rather than a missing case — the shape [`design_run::prompt::Fragment::for_stage`]
/// already uses for a locked run.
fn runbook_facts(stage: Stage) -> Result<Option<design_run::run::RunbookFacts>> {
    let Some(key) = forward_runbook(stage) else {
        return Ok(None);
    };
    let text = crate::install::asset_text(&key.asset_key())?;
    let book = design_run::runbook::Runbook::parse(key, &text)
        .with_context(|| format!("the shipped runbook `{}` is invalid", key.asset_key()))?;
    let digests = book
        .steps()
        .iter()
        .map(|step| {
            (
                step.id().to_owned(),
                crate::git::sha256(step.material().as_bytes()),
            )
        })
        .collect();
    Ok(Some(design_run::run::RunbookFacts { key, book, digests }))
}

/// Every `verify` this submission calls for, executed.
///
/// **Outside the admit→persist span, deliberately** (VA-2). A subprocess of
/// unbounded duration between admission and the snapshot write would widen
/// exactly the window DEC-092's pre-write recheck exists to narrow. These run
/// while nothing is held; the results enter the pure core as derived facts and
/// the core decides what they mean.
///
/// Two seams ask for a check, and one payload may carry both acts:
///
/// - **`EX-10`** — the step a `discharge` names, so a step carrying a check is
///   never recorded on the claim alone.
/// - **`EX-11`** — every already-verified step of the runbook a `stage` act
///   would cross, re-read *now*. A record certifies the world as it stood when
///   it was written; without this the run could carry a stale pass through the
///   gate, which is the adherence theatre the clause exists to beat.
///
/// A step is checked at most once per submission even when both seams want it:
/// two spawns of the same command microseconds apart answer one question.
fn verifications(
    root: &Path,
    slice: u32,
    prior: &DesignSnapshot,
    request: &ApplyRequest,
    facts: Option<&design_run::run::RunbookFacts>,
) -> Vec<design_run::runbook::StepVerification> {
    let Some(facts) = facts else {
        return Vec::new();
    };
    let mut results = Vec::new();
    if let Some(declared) = request.discharge.as_ref() {
        // A skip is a DISCLOSED deviation from doing the step at all, so there
        // is nothing to corroborate. Running the check anyway would refuse the
        // one escape hatch a step the agent cannot satisfy has (`EX-9`).
        if declared.outcome != DischargeClaim::Skipped {
            results.extend(
                facts
                    .book
                    .step(&declared.step)
                    .and_then(|step| check(root, slice, &prior.run.uid, step)),
            );
        }
    }
    let Some(stage) = request.stage.as_ref() else {
        return results;
    };
    // The edge this act would cross, and only it — the guard is evaluated per
    // edge (`EX-16`), so re-checking a runbook the move does not face would
    // block on an obligation that is not at hand.
    if design_run::gate::boundary_runbook(prior.run.stage, stage.to) != Some(facts.key) {
        return results;
    }
    for step in facts.book.steps() {
        // A SKIPPED step is not re-checked, for the reason a skipped discharge
        // is not checked: the deviation is disclosed and there is nothing to
        // corroborate. An undischarged one already blocks the gate on its own.
        let verified = facts
            .book
            .live_discharge(step, &prior.runbook.discharges, &facts.digests)
            .is_some_and(|held| held.outcome() == design_run::runbook::DischargeOutcome::Verified);
        if verified && !results.iter().any(|held| held.step == step.id()) {
            results.extend(check(root, slice, &prior.run.uid, step));
        }
    }
    results
}

/// Run one step's `verify`, if it carries one.
///
/// Returns `None` only when there is nothing to run. A failure to *spawn* is
/// not `None` — it is a result with no exit code, so the core fails closed on
/// it rather than treating an unrunnable check as an absent one.
fn check(
    root: &Path,
    slice: u32,
    run: &str,
    step: &design_run::runbook::Step,
) -> Option<design_run::runbook::StepVerification> {
    let argv = design_run::runbook::interpolate(
        step.verify()?,
        design_run::runbook::Bindings {
            slice: &crate::listing::canonical_id(crate::kinds::SL, slice),
            run,
            repo_root: &root.display().to_string(),
            step: step.id(),
        },
    );
    // Non-empty by validation (`EX-3` refuses an empty argv), and each element
    // goes to `exec` whole — there is no shell between them.
    let (program, args) = argv.split_first()?;
    // Spelled in full: bare `Command` in this crate is the CLI enum.
    let outcome = std::process::Command::new(program)
        .args(args)
        .current_dir(root)
        .output();
    Some(match outcome {
        Ok(out) => design_run::runbook::StepVerification {
            step: step.id().to_owned(),
            exit: out.status.code(),
            output: [
                String::from_utf8_lossy(&out.stdout).into_owned(),
                String::from_utf8_lossy(&out.stderr).into_owned(),
            ]
            .concat(),
        },
        Err(err) => design_run::runbook::StepVerification {
            step: step.id().to_owned(),
            exit: None,
            output: err.to_string(),
        },
    })
}

/// The runbook on `stage`'s single outbound forward edge.
///
/// `can_advance` is total on non-terminal stages — each has exactly one outbound
/// forward edge — so asking "which runbook applies where the run stands" and
/// asking "which runbook guards this edge" are the same question. This walks the
/// stage order to find that edge rather than re-stating the table.
fn forward_runbook(stage: Stage) -> Option<design_run::runbook::RunbookKey> {
    Stage::ALL
        .iter()
        .find(|to| design_run::gate::can_advance(stage, **to))
        .and_then(|to| design_run::gate::boundary_runbook(stage, *to))
}

/// The process fragment the run's stage obliges, bound to its bytes by digest.
///
/// The digest is a *bare* `sha256` string, not a [`Fingerprint`]: a fragment
/// receipt identifies an embedded asset, which is a different kind from the
/// authored-document watermark, and `FragmentReceipt::digest` is typed to match.
///
/// A locked run obliges nothing, so it emits nothing — `for_stage` returning
/// `None` is a real answer, not a missing case.
///
/// What a receipt elides is the BODY and only the body. The `name@digest` header
/// rides unconditionally, and so does the `TurnEnvelope` above: a caller that
/// declared a stale receipt, or lost the bytes it claimed, must still be able to
/// tell what it is missing. Omitting the identity as well would make recovery
/// depend on the caller already knowing what it failed to keep.
fn fragment_section(run: &DesignSnapshot, declared: &[String]) -> Result<Vec<String>> {
    let Some(fragment) = design_run::prompt::Fragment::for_stage(run.run.stage) else {
        return Ok(Vec::new());
    };
    let body = crate::install::asset_text(&fragment.asset_key())?;
    let digest = crate::git::sha256(body.as_bytes());

    // A receipt is current iff it names THIS fragment at THESE bytes. Anything
    // else — a stale digest, another fragment, a bare name, no receipt at all —
    // re-sends the body.
    let holds_current = declared.iter().any(|entry| {
        design_run::prompt::Fragment::parse_receipt(entry) == Some((fragment, digest.as_str()))
    });

    let mut lines = vec![format!("fragment {}", fragment.receipt(&digest))];
    if !holds_current {
        lines.push(body);
    }
    Ok(lines)
}

/// What the run says about the fragments the caller claims to hold.
///
/// Reports agreement or absence; it never *withholds* anything the projection
/// would otherwise have carried, because a flag that changes what a read means
/// is a flag a caller has to remember.
fn fragment_lines(run: &DesignSnapshot, declared: &[String]) -> Vec<String> {
    declared
        .iter()
        .map(|name| {
            let held = run
                .fragments
                .fragments
                .iter()
                .find(|fragment| &fragment.name == name);
            match held {
                Some(fragment) => {
                    format!("known_fragment {name} held digest {}", fragment.digest)
                }
                None => format!("known_fragment {name} NOT held by this run"),
            }
        })
        .collect()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test code: fail-fast on internal invariant violations"
)]
mod tests {
    use super::*;

    /// The fault hook these tests inject: none. They exercise the pre-write
    /// window, not the six-step crash points, which are e2e because a crash is
    /// only observable across a process boundary.
    fn no_fault(_: CheckpointStep) {}

    /// A repo root with a slice tree and a started run. Returns the root.
    fn fixture(dir: &Path) -> u32 {
        std::fs::create_dir_all(dir.join(".doctrine")).unwrap();
        std::fs::create_dir_all(dir.join(crate::kinds::SLICE_DIR).join("001")).unwrap();
        start(dir, 1, false, &|| {}).unwrap();
        1
    }

    /// The payload envelope for a fresh submission at `revision`.
    fn envelope(root: &Path, slice: u32, revision: u64, submission: &str) -> String {
        let run = read_snapshot(root, slice).unwrap();
        format!(
            "\"run_uid\":\"{}\",\"known_revision\":{revision},\"submission_id\":\"{submission}\"",
            run.run.uid
        )
    }

    /// VT-9 / EX-11 / §9.2 — an edit injected into the named pre-write window
    /// abandons the write. Both halves are asserted, because a test that only
    /// checks the error does not prove the write was abandoned: the snapshot
    /// file is byte-identical, and the run advanced no stage and cleared no
    /// gate.
    #[test]
    fn edit_in_prewrite_window_abandons_the_write() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let slice = fixture(root);
        let snapshot_path = crate::state::design_snapshot_path(root, slice);
        let doc = design_doc_path(root, slice);

        let before = std::fs::read(&snapshot_path).unwrap();
        let payload = format!(
            "{{{},\"declare\":[{{\"subject\":\"inq-1\",\"question\":\"q\"}}]}}",
            envelope(root, slice, 1, "sub-1")
        );

        // The hook lands a hand-edit AFTER the candidate is built and BEFORE the
        // snapshot write — the exact window rule 3 must catch.
        let hook = || {
            std::fs::write(&doc, b"a human wrote this mid-invocation\n").unwrap();
        };
        let error = apply(root, slice, &payload, &hook, &no_fault).unwrap_err();
        assert!(
            error.to_string().contains("pre-write window"),
            "the pre-write re-check abandoned the write: {error}"
        );

        // The run did not advance.
        assert_eq!(
            std::fs::read(&snapshot_path).unwrap(),
            before,
            "the snapshot is byte-identical"
        );
        let after = read_snapshot(root, slice).unwrap();
        assert_eq!(after.run.revision, 1, "no revision advance");
        assert_eq!(after.run.stage, crate::design_run::Stage::Exploring);
        assert!(
            after
                .map
                .inquiry
                .get(&DesignId::parse("inq-1").unwrap())
                .is_none()
        );
        assert_eq!(after.change_log.rows.len(), 0, "no rows recorded");
    }

    /// VT-9 / §9.2 — for a checkpoint-bearing `apply` abandoned in that window,
    /// effects ordered before the snapshot under DEC-083/DEC-086 remain, are
    /// recoverable through the submission-keyed journal, and produce no
    /// duplicate on retry. The assertion is that the run did not advance, never
    /// that no byte was written.
    #[test]
    fn checkpoint_effects_survive_an_abandoned_write_without_duplicate() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let slice = fixture(root);
        let doc = design_doc_path(root, slice);

        // A node to dispose.
        apply(
            root,
            slice,
            &format!(
                "{{{},\"declare\":[{{\"subject\":\"inq-1\",\"question\":\"q\"}}]}}",
                envelope(root, slice, 1, "sub-seed")
            ),
            &|| {},
            &no_fault,
        )
        .unwrap();

        // EX-12: `dispose` is the only spelling of a disposition. The pre-effect
        // `record`/`adopt_record` annotation this fixture used to send reached
        // `Disposition::Adopted` without any authored effect at all, so it was
        // the weaker fixture for a test about effects surviving an abandoned
        // write.
        let checkpoint = format!(
            "{{{},\"declare\":[{{\"subject\":\"cp-1\",\"disposes\":\"inq-1\",\
             \"dispose\":{{\"form\":\"create\",\"kind\":\"decision\",\
             \"title\":\"Checkpointed decision\"}}}}]}}",
            envelope(root, slice, 2, "sub-cp")
        );

        let hook = || {
            std::fs::write(&doc, b"mid-invocation hand edit\n").unwrap();
        };
        let error = apply(root, slice, &checkpoint, &hook, &no_fault).unwrap_err();
        assert!(error.to_string().contains("does not advance"), "{error}");

        // The journalled effect REMAINS — that is the stated bound on rule 3's
        // guarantee, not a leak.
        let journal = read_journal(root, slice).unwrap();
        assert_eq!(journal.intents.len(), 1, "the intent was journalled first");
        assert_eq!(journal.intents[0].submission(), "sub-cp");
        assert_eq!(read_snapshot(root, slice).unwrap().run.revision, 2);

        // The retry resumes the same submission and produces NO duplicate.
        std::fs::remove_file(&doc).unwrap();
        apply(root, slice, &checkpoint, &|| {}, &no_fault).unwrap();
        let journal = read_journal(root, slice).unwrap();
        assert_eq!(journal.intents.len(), 1, "no duplicate intent on retry");
        let after = read_snapshot(root, slice).unwrap();
        assert_eq!(after.run.revision, 3, "the retry advanced the run");
        assert_eq!(after.checkpoint.intents.len(), 1);
    }
}
