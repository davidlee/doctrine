// SPDX-License-Identifier: GPL-3.0-only
//! `review new`/`raise`/`dispose`/`verify`/`contest`/`withdraw`/`conclude` — the
//! mint and write verbs (SL-268 PHASE-02 T5).

use super::turn::{resolve_review_root, with_turn};
use super::{
    Act, Context, Deserialize, Facet, FindingStatus, Materialised, Path, PathBuf, REVIEW_DIR,
    REVIEW_KIND, ReviewError, ReviewMeta, ReviewOutput, Role, Severity, Target, TurnFields, Vocab,
    admissible_from, append_finding, append_review_turn, apply_act, can, canonical_id, entity,
    finding_status_of, finding_table_mut, parse_ref, review_table_mut,
};
use crate::tomlfmt::toml_string;

/// Render `review-NNN.toml` from the embedded template (design §4). Every
/// closed-vocab field (`facet`) and user-supplied string (`slug`/`title`/
/// `target.ref`/`phase`/role labels) is spliced through `toml_string` so a
/// hostile value cannot break the document or inject a key
/// (mem.pattern.render.toml-splice-escape-user-values). The optional `[target].
/// phase` line is present iff a phase was given.
pub(super) fn render_review_toml(
    id: u32,
    slug: &str,
    title: &str,
    review: &ReviewMeta,
    target: &Target,
) -> anyhow::Result<String> {
    let phase_line = match &target.phase {
        Some(p) => {
            let mut line = String::from("phase = ");
            line.push_str(&toml_string(p));
            line
        }
        None => String::new(),
    };
    Ok(crate::install::asset_text("templates/review.toml")?
        .replace("{{id}}", &id.to_string())
        .replace("{{slug}}", &toml_string(slug))
        .replace("{{title}}", &toml_string(title))
        .replace("{{facet}}", &toml_string(&review.facet))
        .replace("{{raiser}}", &toml_string(&review.raiser))
        .replace("{{responder}}", &toml_string(&review.responder))
        .replace("{{target_ref}}", &toml_string(&target.reference))
        .replace("{{target_phase}}", &phase_line))
}

/// Render `review-NNN.md` — the `## Brief` companion (design §5/D-C6). Plain
/// markdown token substitution (no toml-splice escaping: markdown body, not a
/// structured value).
fn render_review_md(canonical: &str, facet: &str, target_ref: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/review.md")?
        .replace("{{ref}}", canonical)
        .replace("{{facet}}", facet)
        .replace("{{target}}", target_ref))
}

// ---------------------------------------------------------------------------
// CLI: `review new`
// ---------------------------------------------------------------------------

/// The bundled `review new` arguments — one struct to dodge the clippy arg-ceiling
/// (mem.pattern.lint.cli-handler-args-struct).
#[derive(Deserialize)]
pub(crate) struct NewArgs {
    pub(crate) facet: Facet,
    pub(crate) target: String,
    pub(crate) phase: Option<String>,
    pub(crate) title: Option<String>,
    pub(crate) raiser: Option<String>,
    pub(crate) responder: Option<String>,
}

/// `doctrine review new --facet F --target REF [--phase P]` — allocate a fresh RV
/// and write its authored ledger (empty findings) plus the `## Brief` md. The
/// `[target].ref` is validated up front (design §7): a dangling / unknown-prefix
/// ref is refused BEFORE any id is claimed, so a bad edge never mints an entity.
/// The empty-ledger RV is the real `Active`/await=`Raiser` state (D-C8).
pub(crate) fn run_new(path: Option<PathBuf>, args: &NewArgs) -> anyhow::Result<ReviewOutput> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    mint_review(&root, args, entity::no_midpoint())
}

/// The content of a review about to be placed — derived from [`NewArgs`] once,
/// then rendered for whichever id the placement yields. One renderer serves both
/// placements (fresh mint and resume), so the two cannot drift into two shapes of
/// the same record.
struct ReviewDraft {
    title: String,
    slug: String,
    review: ReviewMeta,
    target: Target,
}

impl ReviewDraft {
    fn from_args(args: &NewArgs) -> anyhow::Result<Self> {
        let title = args
            .title
            .clone()
            .unwrap_or_else(|| format!("{} review of {}", args.facet.as_str(), args.target));
        let slug = crate::input::resolve_slug(&title, None)?;
        Ok(Self {
            title,
            slug,
            review: ReviewMeta {
                facet: args.facet.as_str().to_owned(),
                raiser: args.raiser.clone().unwrap_or_else(|| "raiser".to_owned()),
                responder: args
                    .responder
                    .clone()
                    .unwrap_or_else(|| "responder".to_owned()),
                // A pass is unconcluded until its raiser says otherwise, and the
                // renderer emits no key for it — absence carries the same answer.
                concluded: false,
                // The journal and its counter seed are written at first use, not
                // at mint: a fresh ledger carries neither.
                rounds_base: None,
                contests_base: None,
                turn: Vec::new(),
            },
            target: Target {
                reference: args.target.clone(),
                phase: args.phase.clone(),
            },
        })
    }

    /// The ledger, the brief, and the `NNN-slug` alias for a claimed `(id, canonical)`.
    fn fileset(&self, id: u32, canonical: &str) -> anyhow::Result<entity::Fileset> {
        let name = format!("{id:03}");
        Ok(vec![
            entity::Artifact::File {
                rel_path: PathBuf::from(format!("{name}/review-{name}.toml")),
                body: render_review_toml(id, &self.slug, &self.title, &self.review, &self.target)?,
            },
            entity::Artifact::File {
                rel_path: PathBuf::from(format!("{name}/review-{name}.md")),
                body: render_review_md(canonical, &self.review.facet, &self.target.reference)?,
            },
            entity::Artifact::Symlink {
                rel_path: PathBuf::from(format!("{name}-{}", self.slug)),
                target: name,
            },
        ])
    }
}

/// Mint an RV at a freshly reserved id, exposing the **id-claim midpoint**
/// (DEC-086 steps 2–4). `review new` passes [`entity::no_midpoint`]; the design
/// run's review pass passes a closure that journals the claimed canonical id
/// before any byte is written, so recovery names the exact target from step 3 on.
///
/// Refactored out of `run_new` rather than added beside it — a second
/// reservation+scaffold path is exactly the parallel implementation that drifts
/// (`knowledge::create_record` records the same move for the record kinds).
pub(crate) fn mint_review(
    root: &Path,
    args: &NewArgs,
    on_reserved: impl FnMut(u32, &str) -> anyhow::Result<()>,
) -> anyhow::Result<ReviewOutput> {
    // Forward-edge validation (design §7): refuse a dangling / unknown target
    // BEFORE claiming an id. Reuses the corpus id table (crate::kinds::KINDS).
    // Structured as `DanglingRef` (IMP-107) so the MCP transport maps it to
    // `DANGLING_REF` carrying the target, not a generic Internal.
    crate::kinds::ensure_ref_resolves(root, &args.target).map_err(|_unresolved| {
        ReviewError::DanglingRef {
            target: args.target.clone(),
        }
    })?;

    let draft = ReviewDraft::from_args(args)?;
    let trunk_ids = crate::git::trunk_entity_ids(root, REVIEW_DIR)?;
    let (backend, mut reserved) =
        crate::reserve::backend(root, REVIEW_KIND.prefix, crate::install::prompt_confirm)?;
    let out: Materialised = entity::materialise_fresh_prebuilt_hooked(
        &*backend,
        root,
        REVIEW_DIR,
        REVIEW_KIND.prefix,
        &trunk_ids,
        &mut reserved,
        on_reserved,
        |id, canonical| draft.fileset(id, canonical),
    )?;

    let id = out
        .eid
        .numeric_id()
        .context("review kind must yield a numeric id")?;
    Ok(ReviewOutput::Created {
        id,
        canonical: canonical_id(id),
        dir: out.dir,
    })
}

/// Scaffold a review into an **already-reserved** id (DEC-086 step 4, on resume).
///
/// The reservation is the caller's — claimed and journalled before the crash —
/// so this claims nothing and refuses to clobber. It deliberately does **not**
/// re-run the forward-edge check: the target resolved when the intent was
/// journalled, and a refusal here would strand an intent nothing can discharge.
pub(crate) fn materialise_review_at(
    root: &Path,
    reference: &str,
    args: &NewArgs,
) -> anyhow::Result<()> {
    // The ref comes off the journal, so `review` parses it rather than teaching
    // the design command the RV grammar.
    let id = parse_ref(reference)?;
    let draft = ReviewDraft::from_args(args)?;
    entity::materialise_prebuilt_at(
        root,
        REVIEW_DIR,
        REVIEW_KIND.prefix,
        id,
        &draft.fileset(id, &canonical_id(id))?,
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// The write verbs — each rides `with_turn`; the closure owns the per-finding gate
// ---------------------------------------------------------------------------

/// Bundled `review raise` args (the clippy arg-ceiling — `cli-handler-args-struct`).
#[derive(Deserialize)]
pub(crate) struct RaiseArgs {
    pub(crate) reference: String,
    pub(crate) severity: Severity,
    pub(crate) title: String,
    pub(crate) detail: String,
}

/// `doctrine review raise <RV-NNN> --severity --title --detail [--as raiser]` —
/// append a fresh `open` finding (design §5). Append-only; `raise` is the raiser's
/// and is NOT await-blocked (it may fire even while `await=Responder`, D7/§8).
pub(crate) fn run_raise(
    path: Option<PathBuf>,
    args: &RaiseArgs,
    role: Role,
) -> anyhow::Result<ReviewOutput> {
    let root = resolve_review_root(path)?;
    let id = parse_ref(&args.reference)?;
    let new_id = with_turn(&root, id, Act::Raise, role, |doc, existing| {
        // Per-finding gate: `raise` targets a fresh (None) finding (design §5).
        if !can(Act::Raise, None, role) {
            return Err(ReviewError::RoleMismatch {
                expected: Act::Raise.required_role(),
                actual: role,
                act: Act::Raise,
            }
            .into());
        }
        Ok(append_finding(
            doc,
            existing,
            args.severity,
            &args.title,
            &args.detail,
        ))
    })?;
    Ok(ReviewOutput::Raised {
        finding_id: new_id,
        review_id: id,
    })
}

/// Bundled `review dispose` args.
#[derive(Deserialize)]
pub(crate) struct DisposeArgs {
    pub(crate) reference: String,
    pub(crate) finding: String,
    pub(crate) disposition: String,
    pub(crate) response: String,
}

/// `doctrine review dispose <RV-NNN> --finding F-n --disposition --response
/// [--as responder]` — the responder answers a finding (open|contested →
/// answered, design §5). Sets the responder-owned `disposition`/`response`.
pub(crate) fn run_dispose(
    path: Option<PathBuf>,
    args: &DisposeArgs,
    role: Role,
) -> anyhow::Result<ReviewOutput> {
    let root = resolve_review_root(path)?;
    let id = parse_ref(&args.reference)?;
    with_turn(&root, id, Act::Dispose, role, |doc, existing| {
        let from = finding_status_of(existing, &args.finding)?;
        gate(Act::Dispose, from, role, &args.finding)?;
        let table = finding_table_mut(doc, &args.finding)?;
        // The dispose turn snapshots the answer it gives (sec-2), so a later
        // re-dispose cannot erase what a contest argued against.
        apply_act(
            table,
            Act::Dispose,
            role,
            FindingStatus::Answered,
            TurnFields {
                note: None,
                disposition: Some(&args.disposition),
                response: Some(&args.response),
            },
        )
    })?;
    Ok(ReviewOutput::Disposed {
        finding_id: args.finding.clone(),
        review_id: id,
    })
}

/// `doctrine review verify <RV-NNN> --finding F-n [--as raiser] [--note …]` — the
/// raiser accepts an answered finding (answered → verified, terminal, design §5).
/// The optional `--note` is recorded on the verify turn as its reasoning.
pub(crate) fn run_verify(
    path: Option<PathBuf>,
    reference: &str,
    finding: &str,
    note: Option<&str>,
    role: Role,
) -> anyhow::Result<ReviewOutput> {
    let root = resolve_review_root(path)?;
    let id = parse_ref(reference)?;
    run_raiser_transition(
        &root,
        id,
        Act::Verify,
        FindingStatus::Verified,
        finding,
        note,
        role,
    )?;
    Ok(ReviewOutput::Verified {
        finding_id: finding.to_owned(),
        review_id: id,
    })
}

/// `doctrine review contest <RV-NNN> --finding F-n --note … [--as raiser]` — the
/// raiser rejects an answered finding (answered → contested, design §5), handing
/// it back to the responder. The note is **required** (SL-268 sec-2): it is what
/// the contest argues, recorded on the turn. A blank note refuses before the
/// lock is taken, so nothing is read or written.
pub(crate) fn run_contest(
    path: Option<PathBuf>,
    reference: &str,
    finding: &str,
    note: &str,
    role: Role,
) -> anyhow::Result<ReviewOutput> {
    if note.trim().is_empty() {
        return Err(ReviewError::NoteRequired { act: Act::Contest }.into());
    }
    let root = resolve_review_root(path)?;
    let id = parse_ref(reference)?;
    run_raiser_transition(
        &root,
        id,
        Act::Contest,
        FindingStatus::Contested,
        finding,
        Some(note),
        role,
    )?;
    Ok(ReviewOutput::Contested {
        finding_id: finding.to_owned(),
        review_id: id,
    })
}

/// `doctrine review withdraw <RV-NNN> --finding F-n [--note …] [--as raiser]` —
/// the raiser retracts a finding (open|answered → withdrawn, terminal, design
/// §5). The optional `--note` is recorded on the withdraw turn.
pub(crate) fn run_withdraw(
    path: Option<PathBuf>,
    reference: &str,
    finding: &str,
    note: Option<&str>,
    role: Role,
) -> anyhow::Result<ReviewOutput> {
    let root = resolve_review_root(path)?;
    let id = parse_ref(reference)?;
    run_raiser_transition(
        &root,
        id,
        Act::Withdraw,
        FindingStatus::Withdrawn,
        finding,
        note,
        role,
    )?;
    Ok(ReviewOutput::Withdrawn {
        finding_id: finding.to_owned(),
        review_id: id,
    })
}

/// `doctrine review conclude <RV-NNN> [--as raiser]` — the raiser declares the
/// pass finished, setting the concluded marker (SL-244 `sec-4`, IMP-392).
///
/// The marker is what a design run's `Conducted` disposition is admissible over,
/// and it is deliberately **not** a function of the finding set: a clean pass and
/// a pass never run present identical findings, so nothing derivable can tell
/// them apart. Hence a verb of its own — no existing one could set it as a side
/// effect without acquiring a second meaning, and `dispose` least of all, being
/// per-finding and the responder's.
///
/// Rides [`with_turn`] like every mutating verb, so the per-review lock and both
/// CAS windows apply unchanged. **Open findings are fine**: disposing them is the
/// responder's work afterwards, and requiring a clean ledger would make this a
/// second, stricter spelling of the gate it feeds.
///
/// Idempotent, and there is no unset — a pass that concluded happened. A run that
/// wants another pass gets a new `RV`.
pub(crate) fn run_conclude(
    path: Option<PathBuf>,
    reference: &str,
    role: Role,
) -> anyhow::Result<ReviewOutput> {
    let root = resolve_review_root(path)?;
    let id = parse_ref(reference)?;
    // The latch is written unconditionally: re-writing `true` costs an identical
    // byte sequence, where reading it before the turn would race the lock this
    // turn exists to hold.
    // Every conclude journals its own `[[review.turn]]` (SL-268 sec-2), so a
    // re-conclude is idempotent on the latch but not on the file.
    let already = with_turn(&root, id, Act::Conclude, role, |doc, _findings| {
        let meta = review_table_mut(doc)?;
        let already = meta
            .get("concluded")
            .and_then(toml_edit::Item::as_bool)
            .unwrap_or(false);
        meta["concluded"] = toml_edit::value(true);
        append_review_turn(doc, Act::Conclude, role, None)?;
        Ok(already)
    })?;
    Ok(ReviewOutput::Concluded {
        review_id: id,
        already,
    })
}

/// The shared shell for the three raiser status-only transitions
/// (verify/contest/withdraw): gate per-finding, then apply the status and journal
/// the turn, its `note` included, in one edit (SL-268 sec-2). Disposition /
/// response are responder-owned, so these never touch them.
fn run_raiser_transition(
    root: &Path,
    id: u32,
    act: Act,
    to: FindingStatus,
    finding: &str,
    note: Option<&str>,
    role: Role,
) -> anyhow::Result<()> {
    with_turn(root, id, act, role, |doc, existing| {
        let from = finding_status_of(existing, finding)?;
        gate(act, from, role, finding)?;
        let table = finding_table_mut(doc, finding)?;
        apply_act(
            table,
            act,
            role,
            to,
            TurnFields {
                note,
                ..TurnFields::default()
            },
        )
    })
}

/// The per-finding gate (design §6 — the closure's half): refuse an out-of-turn
/// write with a message naming the act, the finding, its current state and the
/// states the act admits. An out-of-vocabulary current status refuses before the
/// table is consulted (SL-268 D15): no edge leaves a state the table does not know.
pub(super) fn gate(
    act: Act,
    from: Vocab<FindingStatus>,
    role: Role,
    finding: &str,
) -> anyhow::Result<()> {
    let from = match from {
        Vocab::Known(status) => status,
        Vocab::Unknown(raw) => {
            return Err(ReviewError::UnknownStatus {
                finding: finding.to_owned(),
                raw,
            }
            .into());
        }
    };
    if !can(act, Some(from), role) {
        // Role mismatch already caught by `with_turn` step 4; here it is always
        // a state mismatch.
        return Err(ReviewError::StateMismatch {
            finding: finding.to_owned(),
            act,
            current: from,
            admissible: admissible_from(act),
        }
        .into());
    }
    Ok(())
}

/// Parse a `--as` role token (the cooperative role assertion, design §5 — NOT a
/// security boundary, ADR-007 Negative). Defaults to the verb's required role when
/// omitted, so a single-party drive need not toggle `--as` on every call.
pub(crate) fn parse_role(token: Option<&str>, default: Role) -> anyhow::Result<Role> {
    match token {
        None => Ok(default),
        Some("raiser") => Ok(Role::Raiser),
        Some("responder") => Ok(Role::Responder),
        Some(other) => anyhow::bail!("unknown --as role `{other}` (known: raiser, responder)"),
    }
}
