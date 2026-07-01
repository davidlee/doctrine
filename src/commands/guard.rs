// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine` worker-mode guard — `WriteClass`, `write_class`, `worker_guard`.
//! SL-129: uses `entity::id_path`, `clock::today`

use crate::boot::BootCommand;
use crate::commands::cli::Command;
use crate::commands::config::ConfigCommand;
use crate::commands::reservation::ReservationCommand;
use crate::knowledge::KnowledgeCommand;
use crate::policy::PolicyCommand;
use crate::rec::RecCommand;
use crate::rfc::RfcCommand;
use crate::skills::SkillsCommand;
use crate::standard::StandardCommand;

/// Mutation classification for the worker-mode guard (ADR-006 D2a). `Write`
/// carries the verb label named in the refusal. EXHAUSTIVE by design (§7-D6):
/// no wildcard arm, so a future `Command` variant is a compile error — never a
/// silently-permitted write (the X4 self-defence).
pub(crate) enum WriteClass {
    Read,
    Write(&'static str),
    /// Orchestrator-only privileged verbs (SL-056 PHASE-06): `fork` is the FIRST
    /// member; later phases add `import`/`land`/`gc`. Carries the verb label like
    /// `Write`. REFUSED under worker-mode — these are the orchestrator's funnel
    /// operations, never a worker's.
    Orchestrator(&'static str),
    /// `worktree marker --clear` (SL-056 §3, §5): a bespoke class that the
    /// worker-mode guard does NOT refuse (locking the marker's only remover behind
    /// the marker is a self-brick we reject). Its own bespoke refusals live in
    /// `run_marker_clear`.
    MarkerClear,
    /// `worktree marker --stamp-subagent` (SL-056 PHASE-10): the claude harness
    /// spawn path's provision+mark step. REFUSED under worker-mode via the SAME
    /// branch as `Orchestrator`/`Write` — NO verb-identity carve-out. The legit
    /// first stamp passes automatically: the target worktree bears no marker yet,
    /// so `worker_mode == false` (marker-absent ⇒ allow). Carries the verb label.
    Hookmint(&'static str),
}

#[expect(
    clippy::match_same_arms,
    reason = "consecutive Read arms across different nested-match shapes; merging would degrade readability"
)]
pub(crate) fn write_class(cmd: &Command) -> WriteClass {
    use super::cli::ExportCommand;
    use super::coverage::CoverageCommand;
    use crate::adr::AdrCommand;
    use crate::backlog::BacklogCommand;
    use crate::concept_map::ConceptMapCommand;
    use crate::dispatch::{CandidateCommand, DispatchCommand};
    use crate::memory::{MemoryCommand, SyncCommand};
    use crate::review::ReviewCommand;
    use crate::revision::{RevisionChangeCommand, RevisionCommand};
    use crate::spec::{SpecCommand, SpecReqCommand};
    use crate::worktree::WorktreeCommand;
    use WriteClass::{Hookmint, MarkerClear, Orchestrator, Read, Write};
    match cmd {
        Command::Install { .. } => Write("install"),
        Command::Skills { command } => match command {
            SkillsCommand::List { .. } => Read,
        },
        Command::Map { .. } => Write("map"),
        Command::ConceptMap { command } => match command {
            ConceptMapCommand::New { .. } => Write("concept-map new"),
            ConceptMapCommand::Add { .. } => Write("concept-map add"),
            ConceptMapCommand::Remove { .. } => Write("concept-map remove"),
            ConceptMapCommand::RenameNode { .. } => Write("concept-map rename-node"),
            ConceptMapCommand::List { .. }
            | ConceptMapCommand::Show { .. }
            | ConceptMapCommand::Check { .. }
            | ConceptMapCommand::Export { .. }
            | ConceptMapCommand::Paths { .. } => Read,
        },
        Command::Slice { command } => match command {
            crate::slice::SliceCommand::New { .. } => Write("slice new"),
            crate::slice::SliceCommand::Design { .. } => Write("slice design"),
            crate::slice::SliceCommand::Plan { .. } => Write("slice plan"),
            crate::slice::SliceCommand::Phases { .. } => Write("slice phases"),
            crate::slice::SliceCommand::Notes { .. } => Write("slice notes"),
            crate::slice::SliceCommand::Phase { .. } => Write("slice phase"),
            crate::slice::SliceCommand::RecordDelta { .. } => Write("slice record-delta"),
            crate::slice::SliceCommand::Status { .. } => Write("slice status"),
            crate::slice::SliceCommand::List { .. }
            | crate::slice::SliceCommand::Show { .. }
            | crate::slice::SliceCommand::Conformance { .. }
            | crate::slice::SliceCommand::VerifyVt { .. }
            | crate::slice::SliceCommand::Paths { .. } => Read,
            crate::slice::SliceCommand::Selector { .. } => Write("slice selector"),
        },
        Command::Memory { command } => match command {
            MemoryCommand::Record { .. } => Write("memory record"),
            MemoryCommand::Verify { .. } => Write("memory verify"),
            MemoryCommand::Sync { command, .. } => match command {
                None => Write("memory sync"),
                Some(SyncCommand::Install { .. }) => Write("memory sync install"),
            },
            MemoryCommand::Tag { .. } => Write("memory tag"),
            MemoryCommand::Status { .. } => Write("memory status"),
            MemoryCommand::Edit { .. } => Write("memory edit"),
            MemoryCommand::Validate { .. }
            | MemoryCommand::Show { .. }
            | MemoryCommand::List { .. }
            | MemoryCommand::Find { .. }
            | MemoryCommand::Retrieve { .. }
            | MemoryCommand::ResolveLinks { .. }
            | MemoryCommand::Backlinks { .. }
            | MemoryCommand::Paths { .. } => Read,
        },
        Command::Review { command } => match command {
            ReviewCommand::New { .. } => Write("review new"),
            ReviewCommand::Raise { .. } => Write("review raise"),
            ReviewCommand::Dispose { .. } => Write("review dispose"),
            ReviewCommand::Verify { .. } => Write("review verify"),
            ReviewCommand::Contest { .. } => Write("review contest"),
            ReviewCommand::Withdraw { .. } => Write("review withdraw"),
            ReviewCommand::Unlock { .. } => Write("review unlock"),
            ReviewCommand::List { .. }
            | ReviewCommand::Show { .. }
            | ReviewCommand::Status { .. }
            | ReviewCommand::Prime { .. }
            | ReviewCommand::Paths { .. } => Read,
        },
        Command::Rec { command } => match command {
            RecCommand::New { .. } => Write("rec new"),
            RecCommand::List { .. } | RecCommand::Show { .. } | RecCommand::Paths { .. } => Read,
        },
        Command::Revision { command } => match command {
            RevisionCommand::New { .. } => Write("revision new"),
            RevisionCommand::Status { .. } => Write("revision status"),
            RevisionCommand::Show { .. }
            | RevisionCommand::Paths { .. }
            | RevisionCommand::List { .. } => Read,
            RevisionCommand::Change { command } => match command {
                RevisionChangeCommand::Add { .. } => Write("revision change add"),
            },
            RevisionCommand::Approve { .. } => Write("revision approve"),
            RevisionCommand::Apply { .. } => Write("revision apply"),
        },
        // Writes authored requirement status + an authored REC — an authored write.
        Command::Reconcile { .. } => Write("reconcile"),
        Command::Adr { command } => match command {
            AdrCommand::New { .. } => Write("adr new"),
            AdrCommand::Status { .. } => Write("adr status"),
            AdrCommand::List { .. } | AdrCommand::Show { .. } | AdrCommand::Paths { .. } => Read,
        },
        Command::Policy { command } => match command {
            PolicyCommand::New { .. } => Write("policy new"),
            PolicyCommand::Status { .. } => Write("policy status"),
            PolicyCommand::List { .. }
            | PolicyCommand::Show { .. }
            | PolicyCommand::Paths { .. } => Read,
        },
        Command::Standard { command } => match command {
            StandardCommand::New { .. } => Write("standard new"),
            StandardCommand::Status { .. } => Write("standard status"),
            StandardCommand::List { .. }
            | StandardCommand::Show { .. }
            | StandardCommand::Paths { .. } => Read,
        },
        Command::Rfc { command } => match command {
            RfcCommand::New { .. } => Write("rfc new"),
            RfcCommand::Status { .. } => Write("rfc status"),
            RfcCommand::List { .. } | RfcCommand::Show { .. } | RfcCommand::Paths { .. } => Read,
        },
        Command::Spec { command } => match command {
            SpecCommand::New { .. } => Write("spec new"),
            SpecCommand::Req { command } => match command {
                SpecReqCommand::Add { .. } => Write("spec req add"),
                SpecReqCommand::Status { .. } => Write("spec req status"),
                // Read-only authored roster (design §5.3).
                SpecReqCommand::List { .. } => Read,
            },
            SpecCommand::Interactions { .. } => Write("spec interactions"),
            SpecCommand::Edit { .. } => Write("spec edit"),
            SpecCommand::List { .. }
            | SpecCommand::Show { .. }
            | SpecCommand::Validate { .. }
            | SpecCommand::Paths { .. } => Read,
        },
        // Export is read-only (RO proof): load + serialize, no mutation path.
        Command::Export { command } => match command {
            ExportCommand::Lazyspec { .. } => Read,
        },
        Command::Backlog { command } => match command {
            BacklogCommand::New { .. } => Write("backlog new"),
            BacklogCommand::Edit { .. } => Write("backlog edit"),
            BacklogCommand::Needs { .. } => Write("backlog needs"),
            BacklogCommand::After { .. } => Write("backlog after"),
            BacklogCommand::Tag { .. } => Write("backlog tag"),
            BacklogCommand::List { .. }
            | BacklogCommand::Show { .. }
            | BacklogCommand::Inspect { .. }
            | BacklogCommand::Paths { .. } => Read,
        },
        Command::Knowledge { command } => match command {
            KnowledgeCommand::New { .. } => Write("knowledge new"),
            KnowledgeCommand::Status { .. } => Write("knowledge status"),
            KnowledgeCommand::List { .. }
            | KnowledgeCommand::Show { .. }
            | KnowledgeCommand::Inspect { .. }
            | KnowledgeCommand::Paths { .. } => Read,
        },
        Command::Tag { .. } => Write("tag"),
        // The reservation survey only fetches + reads refs — no authored write.
        Command::Reservation { command } => match command {
            ReservationCommand::List { .. } => Read,
        },
        Command::Serve { .. } => Read,
        Command::Boot { command, .. } => match command {
            None => Write("boot"),
            Some(BootCommand::Install { .. }) => Write("boot install"),
        },
        Command::Worktree { command } => match command {
            // Provision/check-allowlist write *fork* files, not the doctrine state
            // the guard protects, and never run in worker context (§5.2) — Read.
            // branch-point-check is a HEAD read + ref compare — no authored write,
            // callable under worker-mode by construction (§5.2, C-V).
            // status reads the resolved mode (SL-056 §3) — open to workers.
            // verify-worker is a HEAD read + marker probe + is-ancestor compare on
            // the worker dir — no authored write, diagnostic only; harmless under
            // worker-mode (design §8.4/§8.6 lists no impersonation test for it).
            // pretooluse is the claude `PreToolUse` hook verb (SL-182 PHASE-03) —
            // it reads stdin + git topology and emits a decision, writing NO
            // authored state. It fires INSIDE the confined subagent (worker
            // context) on every tool call, so it MUST be open under worker-mode —
            // Read.
            WorktreeCommand::Provision { .. }
            | WorktreeCommand::CheckAllowlist { .. }
            | WorktreeCommand::BranchPointCheck { .. }
            | WorktreeCommand::VerifyWorker { .. }
            | WorktreeCommand::Pretooluse
            | WorktreeCommand::Status { .. } => Read,
            // fork creates an orchestrator-owned worktree (SL-056 PHASE-06) — the
            // first Orchestrator-classed verb; refused under worker-mode.
            WorktreeCommand::Fork { .. } => Orchestrator("fork"),
            // create-fork is the claude `WorktreeCreate` hook verb (SL-152) — it
            // fires in the MARKERLESS parent coord tree (process cwd), so the
            // worker_guard resolves non-worker mode and it is allowed; a spawn from
            // inside a marked fork is refused fail-closed (acceptable — workers carry
            // no Agent tool). Orchestrator and Hookmint are functionally identical
            // under worker_guard; Orchestrator is the plan-locked class (G8).
            WorktreeCommand::CreateFork => Orchestrator("create-fork"),
            // coordinate creates/resumes the orchestrator's OWN coordination
            // worktree (SL-064 §2) — markerless, but still an orchestrator funnel
            // operation; refused under worker-mode via the SAME guard as fork (EX-4).
            WorktreeCommand::Coordinate { .. } => Orchestrator("coordinate"),
            // import lands a worker delta into the coordination index (SL-056
            // PHASE-07) — Orchestrator-classed; refused under worker-mode.
            WorktreeCommand::Import { .. } => Orchestrator("import"),
            // land lands a solo fork onto the coordination branch via --no-ff merge
            // (SL-056 PHASE-08) — Orchestrator-classed; refused under worker-mode.
            WorktreeCommand::Land { .. } => Orchestrator("land"),
            // gc reaps a spent worktree fork once provably landed (SL-056 PHASE-09)
            // — Orchestrator-classed; refused under worker-mode.
            WorktreeCommand::Gc { .. } => Orchestrator("gc"),
            // marker --stamp-subagent is the claude harness spawn path's provision+mark
            // step (SL-056 PHASE-10) — Hookmint, refused under worker-mode (the
            // legit first stamp lands on a marker-absent worktree ⇒ allowed). All
            // other marker forms (--clear, bare) are the bespoke self-brick cure —
            // NOT refused by the worker-mode guard; their fences live in the handler.
            WorktreeCommand::Marker {
                stamp_subagent: true,
                ..
            } => Hookmint("marker --stamp-subagent"),
            WorktreeCommand::Marker { .. } => MarkerClear,
        },
        // dispatch sync projects coordination refs (SL-064 PHASE-04 / ADR-012
        // §4) — Orchestrator-classed across the whole verb class; refused under
        // worker-mode via the SAME guard as coordinate/fork (EX-1).
        Command::Dispatch { command } => match command {
            DispatchCommand::Sync { .. } => Orchestrator("dispatch-sync"),
            DispatchCommand::RecordBoundary { .. } => Orchestrator("dispatch-record-boundary"),
            DispatchCommand::RefreshBase { .. } => Orchestrator("dispatch-refresh-base"),
            DispatchCommand::Setup { .. } => Orchestrator("dispatch-setup"),
            // arm-spawn writes the coord tree's arming base file (SL-152 PHASE-03,
            // sole-writer) — Orchestrator-classed; refused under worker-mode.
            DispatchCommand::ArmSpawn { .. } => Orchestrator("dispatch-arm-spawn"),
            // candidate create publishes coordination refs + ledger rows (SL-068
            // §5.3) — Orchestrator-classed like sync/record-boundary; refused
            // under worker-mode.
            DispatchCommand::Candidate { command } => match command {
                CandidateCommand::Create { .. } => Orchestrator("dispatch-candidate-create"),
                // candidate status is a read-only self-describing surface (SL-068
                // PHASE-04) — Read-classed so it works under worker-mode; it
                // mutates no ref and no ledger row.
                CandidateCommand::Status { .. } => Read,
                // candidate admit pins an immutable OID into candidates.toml
                // (SL-068 PHASE-05) — Orchestrator-classed like create; refused
                // under worker-mode.
                CandidateCommand::Admit { .. } => Orchestrator("dispatch-candidate-admit"),
            },
            // plan-next / status — read plan + phase sheets; never mutates a
            // ref or ledger row — Read-classed so it works under worker-mode.
            DispatchCommand::PlanNext { .. }
            | DispatchCommand::Status { .. }
            | DispatchCommand::DeliverTo { .. } => Read,
        },
        // The coverage group splits per inner verb (SL-057 D2a): `show` is the
        // read-only drift view; `record`/`forget` mutate the observed store, and
        // `verify` re-derives + saves per slice — all authored writes.
        Command::Coverage { command } => match command {
            CoverageCommand::Show { .. } => Read,
            CoverageCommand::Record { .. } => Write("coverage record"),
            CoverageCommand::Verify { .. } => Write("coverage verify"),
            CoverageCommand::Forget { .. } => Write("coverage forget"),
        },
        // Read-only: the corpus integrity scan (INV-3), and the cross-kind relation
        // view (SL-046 — reads only, never mints/derives status).
        // Read-only priority surfaces (SL-047 — derive per query, never write /
        // mint / derive status; ADR-004 stores no reverse field).
        Command::Catalog { .. }
        | Command::Search { .. }
        | Command::Relation { .. }
        | Command::Validate { .. }
        | Command::Doctor { .. }
        | Command::Inspect { .. }
        | Command::Survey { .. }
        | Command::Next { .. }
        | Command::Blockers { .. }
        | Command::Explain { .. }
        // The check proxy writes NO authored doctrine state; a proxied command
        // that mutates source (e.g. `cargo fmt`) is a worker-legal source delta,
        // not an authored write — and a worker running `check gate` to verify its
        // fork is the intended use, so Read is correct AND necessary (SL-163 §5.3).
        | Command::Check { .. }
        | Command::Status { .. } => Read,
        // Mutates the canonical-id triple — an authored write (D2/D6).
        Command::Reseat { .. } => Write("reseat"),
        // Author / remove a tier-1 `[[relation]]` edge — authored writes (SL-048 §5.4).
        Command::Link { .. } => Write("link"),
        Command::Config { command } => match command {
            ConfigCommand::Set { .. } | ConfigCommand::Unset { .. } => Write("config"),
            ConfigCommand::Show { .. } | ConfigCommand::Get { .. } | ConfigCommand::Validate => {
                Read
            }
        },
        Command::Unlink { .. } => Write("unlink"),
        // Author a dep/seq edge into `[relationships]` — authored writes (SL-060 §5.4).
        Command::Needs { .. } => Write("needs"),
        Command::After { .. } => Write("after"),
        // Record a supersession — writes NEW.supersedes, OLD.superseded_by, OLD.status
        // in one transaction (SL-062 §5.4).
        Command::Supersede { .. } => Write("supersede"),
        // Estimate / Value / Risk facet writes (SL-118 PHASE-03, SL-134 PHASE-02).
        Command::Estimate { .. } => Write("estimate"),
        Command::Value { .. } => Write("value"),
        Command::Risk { .. } => Write("risk"),
    }
}

/// Worker-mode guard (ADR-006 D2a / SL-056 §3): refuse a Write-classed verb when
/// the cwd tree resolves to worker mode (marker in a linked worktree OR the
/// `DOCTRINE_WORKER` env optimisation). Read / `MarkerClear` pass through. The
/// marker leg is evaluated LAZILY — only a Write verb resolves the root, so a Read
/// verb in a non-doctrine cwd never gains a new failure path (design §3).
pub(crate) fn worker_guard(cmd: &Command) -> anyhow::Result<()> {
    // Write and Orchestrator are both refused under worker-mode with the SAME
    // branches; Read and the bespoke MarkerClear pass through (SL-056 PHASE-06).
    let verb = match write_class(cmd) {
        WriteClass::Write(verb) | WriteClass::Orchestrator(verb) | WriteClass::Hookmint(verb) => {
            verb
        }
        WriteClass::Read | WriteClass::MarkerClear => return Ok(()),
    };
    // No doctrine/project root above the cwd: the marker leg cannot apply. Fall
    // back to the env leg alone (a leaked env on a rootless cwd), never a new error.
    let Ok(root) = crate::root::find(None, &crate::root::default_markers()) else {
        if crate::worktree::env_worker_set() {
            anyhow::bail!(
                "{}: refusing authored write `{verb}`",
                crate::worktree::DUAL_CAUSE
            );
        }
        return Ok(());
    };
    let mode = crate::worktree::resolve_mode(&root);
    if !mode.refused {
        return Ok(());
    }
    // The env leg on a NON-linked tree carries the NAMED dual-cause message (never
    // a bare "worker refused"); the marker / linked-fork legs name the verb plainly.
    if mode.is_env_on_nonlinked() {
        anyhow::bail!(
            "{}: refusing authored write `{verb}`",
            crate::worktree::DUAL_CAUSE
        );
    }
    anyhow::bail!(
        "worker fork (signal: {}): refusing authored write `{verb}` — workers return a source delta; doctrine-mediated writes funnel through the orchestrator.",
        mode.cause_token()
    );
}
