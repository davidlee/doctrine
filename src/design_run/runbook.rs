// SPDX-License-Identifier: GPL-3.0-only
//! The ordered-obligation runbook (SL-233 PHASE-16, DEC-101).
//!
//! A runbook is a **transition guard** — the ordered ritual an agent discharges
//! while standing at a stage, guarding that stage's outbound edge. Guards belong
//! on edges, so a runbook is selected by [`super::gate::boundary_runbook`], the
//! third column on the table [`super::gate::boundary_conditions`] already is
//! (sketch §2.1). It adds no states: the cursor is run data a guard consults,
//! not a node in the machine, exactly as [`super::gate::ReviewStanding`] is.
//!
//! # What this module owns, and what it must not touch
//!
//! Rust owns the closed concept — ordered, one at a time, no skipping, bound to
//! the definition discharged against. The *contents* are asset data. That is
//! DEC-077's split one level deeper, applied to procedure rather than to prose.
//!
//! Leaf tier with crate out-degree zero, like every sibling here: this module
//! computes the digest **material** and never hashes it. The shell hashes with
//! `crate::git::sha256` and hands the digests back in through
//! [`super::run::DerivedInput`], which is what `EX-18`'s "computed shell-side,
//! compared in the pure core" means concretely. Reaching for `crate::git` here
//! would also stop `tests/e2e_design_*.rs` compiling, since they `#[path]`-include
//! this tree standalone.
//!
//! # What a step id does and does not do
//!
//! An id solves **reference**, not **equivalence**. It says *which* step you
//! mean; it says nothing about whether the step you mean today is the obligation
//! the discharge was made against. So a discharge binds
//! [`Step::material`]'s digest — `id`, `text`, `required`, `verify` — and any
//! edit to any of them makes it stale by construction (`EX-2`).

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::prompt::STORE;

/// The canonical encoding of a step definition, version-tagged (`EX-18`).
///
/// The tag is hashed **into** the material and stored **on** the record, so
/// changing the encoding later invalidates records *visibly* — every discharge
/// written under an older tag reports stale and is re-made deliberately —
/// rather than silently comparing incomparable bytes. It is what buys the right
/// to add a fourth outcome arm (sketch §7.1) cheaply, which is why that arm is
/// deliberately not guessed at now.
pub(crate) const RUNBOOK_STEP_DIGEST_VERSION: &str = "runbook-step.v1";

/// Admission bound on a step id, in bytes.
///
/// **Derived from [`super::bounds::DESIGN_ID_BYTES`], not chosen** (SL-233
/// PHASE-08, `F-P08.2`): discharging a step records its id as the identity term
/// of a `step_discharged` change payload — the shape
/// [`super::change_log::ChangeEvent::payload_terms`] declares for it — so a step
/// id *is* something that slot must carry, and the id slot's bound is therefore
/// exactly this one. Single-sourcing it here is what keeps the two ends from
/// drifting apart.
///
/// They had drifted. This was independently sized at 64 on the reasoning that a
/// step id is not a [`super::ids::DesignId`] (see [`Step::id`]) and so owed the
/// id slot's provenance nothing; meanwhile the construction site admitted the id
/// as a *label*, at 16 B. A 17-byte step id therefore parsed, blocked its edge,
/// and could never be discharged — an unclearable edge, silent until a run tried
/// it. The shipped five escaped only because `explore.research` is exactly 16 B.
///
/// A bound tightening (64 → 32) is breaking in general. It is not here: the
/// store is not project-overridable yet (IMP-375), so no runbook outside this
/// repo can hold a step id at all, and the widest shipped id is 17 B.
pub(crate) const RUNBOOK_STEP_ID_BYTES: usize = super::bounds::DESIGN_ID_BYTES;

/// The closed placeholder vocabulary a `verify` argv may interpolate (`EX-4`).
///
/// The closed thing is the **placeholders**, not the set of checks: a verifier
/// cannot check anything without knowing what it is checking, so interpolation
/// is mandatory, but that vocabulary is small and does not grow when checks
/// grow. Single-sourced here per STD-001; an unknown placeholder is refused at
/// validation, never discovered at execution.
pub(crate) const PLACEHOLDERS: [&str; 4] = [SLICE, RUN, REPO_ROOT, STEP];

/// The slice the run designs, in reference form.
const SLICE: &str = "slice";
/// The run's uid.
const RUN: &str = "run";
/// The project root the verifier runs against.
const REPO_ROOT: &str = "repo_root";
/// The step being discharged.
const STEP: &str = "step";

/// What a `verify` argv's placeholders stand for on one execution.
///
/// A struct rather than a map because the vocabulary is closed: a missing
/// binding should be a compile error, not a lookup that quietly yields nothing.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Bindings<'a> {
    pub(crate) slice: &'a str,
    pub(crate) run: &'a str,
    pub(crate) repo_root: &'a str,
    pub(crate) step: &'a str,
}

impl<'a> Bindings<'a> {
    /// The value bound to `name`. Total over [`PLACEHOLDERS`], which is the only
    /// domain [`interpolate`] asks it about and the only one validation admits.
    fn value(self, name: &str) -> &'a str {
        match name {
            SLICE => self.slice,
            RUN => self.run,
            REPO_ROOT => self.repo_root,
            STEP => self.step,
            _ => "",
        }
    }
}

/// Substitute `{placeholder}` occurrences within each argv element (`EX-4`).
///
/// **Element count is preserved and no element is ever re-split.** A value
/// holding spaces or shell metacharacters arrives at `exec` as ONE argument —
/// which is the whole reason `verify` is an array and not a command string.
/// Substituting into a string would turn data into syntax, and escaping it back
/// out again fails differently depending on the author's own quoting.
pub(crate) fn interpolate(argv: &[String], bindings: Bindings<'_>) -> Vec<String> {
    argv.iter()
        .map(|element| {
            PLACEHOLDERS.iter().fold(element.clone(), |acc, name| {
                acc.replace(&format!("{{{name}}}"), bindings.value(name))
            })
        })
        .collect()
}

/// One execution of a step's `verify`, as the shell observed it.
///
/// `exit` is `Option` because "the check could not be run at all" is a real
/// outcome and not the same as a non-zero one — but neither says yes, so the
/// two are one fact here and a refusal tells them apart.
///
/// It lives beside [`interpolate`] rather than with the carrier that transports
/// it ([`super::run::DerivedInput`]) because it is a fact about a *step*: what
/// the argv this module built and validated returned when the shell ran it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StepVerification {
    /// The step the shell ran the check for. Compared against the step it is
    /// offered for, so a result derived for one can never answer for another.
    pub(crate) step: String,
    pub(crate) exit: Option<i32>,
    /// Everything the check said, both streams, as the refusal's detail.
    pub(crate) output: String,
}

impl StepVerification {
    /// Whether this result says yes. Anything else — a non-zero exit or a check
    /// that could not be run — does not.
    fn cleared(&self) -> bool {
        self.exit == Some(0)
    }

    /// Whether `results` holds a verdict for `step`, and it is a refusal.
    ///
    /// Absence is deliberately not a refusal: a result is produced only where a
    /// check was *run*, and the paths that run none (a discharge-only payload,
    /// a step with no `verify`) must not be read as failures. Where a missing
    /// result must fail closed — a step being discharged on a claim — that is
    /// the discharge's own rule, stated at its own site.
    fn refutes(results: &[StepVerification], step: &str) -> bool {
        results
            .iter()
            .any(|result| result.step == step && !result.cleared())
    }
}

/// Which runbook — one variant per guarded edge.
///
/// Closed like [`super::prompt::Fragment`] and for the same reason: one variant,
/// one file, so "at most one runbook per edge" is structural rather than a
/// runtime check. PHASE-16 shipped the first; PHASE-08 converted the rest, so
/// every non-terminal stage's single outbound forward edge now carries one.
///
/// Named for the edge's ORIGIN stage, which is not a shorthand:
/// `Advance::from_stage` is total on non-terminal stages, so origin-keying and
/// edge-keying are the same thing (D5) and there is no fifth edge for a name to
/// become ambiguous over.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum RunbookKey {
    /// The ritual discharged while standing at [`super::Stage::Exploring`],
    /// guarding its outbound edge to [`super::Stage::Inquiring`].
    Exploring,
    /// Standing at [`super::Stage::Inquiring`], guarding the edge to
    /// [`super::Stage::Drafting`].
    Inquiring,
    /// Standing at [`super::Stage::Drafting`], guarding the edge to
    /// [`super::Stage::Reviewing`].
    Drafting,
    /// Standing at [`super::Stage::Reviewing`], guarding the edge to
    /// [`super::Stage::Locked`].
    Reviewing,
}

impl RunbookKey {
    /// Every key, single-sourced so an exhaustive table test cannot silently
    /// miss a new variant (STD-001, the `Stage::ALL` precedent).
    pub(crate) const ALL: [RunbookKey; 4] = [
        RunbookKey::Exploring,
        RunbookKey::Inquiring,
        RunbookKey::Drafting,
        RunbookKey::Reviewing,
    ];

    /// The key's stable name — what a discharge record identifies it by, and
    /// the stem of its file.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            RunbookKey::Exploring => "exploring",
            RunbookKey::Inquiring => "inquiring",
            RunbookKey::Drafting => "drafting",
            RunbookKey::Reviewing => "reviewing",
        }
    }

    /// The embedded asset key the shell resolves. Shares [`STORE`] with the
    /// process fragments because they are the same store — a runbook is the
    /// structured sibling of the prose, not a second corpus.
    pub(crate) fn asset_key(self) -> String {
        format!("{STORE}/{}.toml", self.name())
    }
}

/// How a runbook's steps are admitted.
///
/// One variant. `set` — a coverage set with no meaningful order — is deferred
/// to **IMP-373**, not to a phase of this slice.
///
/// The reassignment matters, because the reason changed with it. `set` was
/// first deferred for want of a user: PHASE-08 would convert two set-shaped
/// checklists and so acquire real instances to design the mode against. Both
/// halves lapsed — those checklists became stage framing (fragment prose), and
/// the runbooks that replaced them carry five steps whose order is arbitrary.
/// So instances exist, and *"nobody needs it"* is false.
///
/// What defers it now is the **render**: admission is a branch here, but a
/// cursorless runbook has no rendering rule under `EX-14`'s token bound, and
/// that is a design question rather than a variant addition.
///
/// Imposing an order on a coverage set is fake determinism — conceded for
/// those five steps, and judged cheap, not argued away. The `sequence`/`set`
/// distinction survives intact (DEC-101, amended 2026-08-01).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Mode {
    /// A cursor, one step at a time, no skipping.
    Sequence,
}

/// One obligation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Step {
    id: String,
    text: String,
    /// Whether the gate blocks on it. Defaults to **required**: an obligation
    /// runbook whose steps are advisory unless someone remembers to say
    /// otherwise has the default pointing the wrong way.
    #[serde(default = "Step::required_default")]
    required: bool,
    /// The check that corroborates the attestation, as an **argv array**.
    ///
    /// Never a shell string: `{repo_root}` can contain spaces and shell
    /// metacharacters, so substituting into a command string turns data into
    /// syntax, and escaping each value fails differently depending on the
    /// author's own quoting. Each element interpolates as one whole opaque
    /// value with no shell between it and `exec` (`EX-4`).
    ///
    /// Optional — presence of a command *is* the discharge mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    verify: Option<Vec<String>>,
}

impl Step {
    const fn required_default() -> bool {
        true
    }

    /// Which step this is.
    ///
    /// **Not a [`super::ids::DesignId`]**, and decided rather than discovered:
    /// `DesignId::parse` requires one of seven closed prefixes and a body of
    /// `[A-Za-z0-9_-]` only, and `explore.canon` satisfies neither. Discharge
    /// records are their own run state rather than evidence rows, so nothing
    /// constrains a step id to that grammar — but no step id may be routed into
    /// a slot typed `DesignId`.
    pub(crate) fn id(&self) -> &str {
        &self.id
    }

    /// What the agent is being told to do now.
    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    /// Whether the gate blocks while this step is undischarged.
    pub(crate) const fn required(&self) -> bool {
        self.required
    }

    /// The check to run, if this step has one.
    pub(crate) fn verify(&self) -> Option<&[String]> {
        self.verify.as_deref()
    }

    /// The bytes a discharge binds — the canonical, version-tagged encoding of
    /// this step's whole definition (`EX-18`).
    ///
    /// **Netstring-framed**, `len:value` per field, deliberately *not* the
    /// separator join `commands::design::acceptance_digest` uses. That one binds
    /// digests and closed labels, where no value can contain the separator;
    /// `text` here is arbitrary project prose and can contain any byte, so a
    /// separator join would let two different definitions encode identically.
    ///
    /// Field order is contract: version, `id`, `text`, `required`, then the
    /// argv arity and its elements in order. An absent `verify` encodes arity
    /// zero; the one ambiguity that could create — an empty argv encoding the
    /// same — is closed on the other side, because [`Runbook::parse`] refuses an
    /// empty argv outright.
    pub(crate) fn material(&self) -> String {
        let argv = self.verify();
        let mut parts = vec![
            framed(RUNBOOK_STEP_DIGEST_VERSION),
            framed(&self.id),
            framed(&self.text),
            framed(if self.required { "true" } else { "false" }),
            framed(&argv.map_or(0, <[String]>::len).to_string()),
        ];
        parts.extend(argv.unwrap_or(&[]).iter().map(|element| framed(element)));
        parts.concat()
    }
}

/// One field of [`Step::material`], length-prefixed so no value can be mistaken
/// for a delimiter. The length is in **bytes**, which is what `str::len` is.
fn framed(value: &str) -> String {
    format!("{}:{value}", value.len())
}

/// A parsed, validated runbook.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Runbook {
    key: RunbookKey,
    mode: Mode,
    steps: Vec<Step>,
}

/// The wire shape, before validation. Separate from [`Runbook`] so a `Runbook`
/// value cannot exist without having passed the checks.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RunbookWire {
    mode: Mode,
    #[serde(default, rename = "step")]
    steps: Vec<Step>,
}

impl Runbook {
    /// Admit a runbook asset.
    ///
    /// **Validation is owned, not inherited** (`EX-3`). `doctrine prompt check`
    /// loads `.doctrine/hymns` plus the embedded hymns and checks the replaces
    /// graph, stage vocabulary, seal integrity and hymn markers; it knows
    /// nothing about this store's `*.toml` siblings. So this is a second validation
    /// domain, and every one of its refusals happens **here**, never at
    /// execution — a check that only fails when the verifier runs has not
    /// validated anything.
    pub(crate) fn parse(key: RunbookKey, text: &str) -> anyhow::Result<Runbook> {
        let wire: RunbookWire = toml::from_str(text)?;
        if wire.steps.is_empty() {
            anyhow::bail!(
                "runbook `{}` declares no steps — an obligation runbook with nothing to \
                 discharge would clear its edge unconditionally",
                key.name()
            );
        }
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for step in &wire.steps {
            validate_step_id(&step.id)?;
            if !seen.insert(step.id.as_str()) {
                anyhow::bail!(
                    "runbook `{}` declares step `{}` twice — a discharge could not say \
                     which one it meant",
                    key.name(),
                    step.id
                );
            }
            if step.text.trim().is_empty() {
                anyhow::bail!(
                    "step `{}` has empty `text` — an obligation the agent cannot read is \
                     not an obligation",
                    step.id
                );
            }
            validate_verify(step)?;
        }
        Ok(Runbook {
            key,
            mode: wire.mode,
            steps: wire.steps,
        })
    }

    /// Which runbook this is — the address the discharge records name.
    pub(crate) const fn key(&self) -> RunbookKey {
        self.key
    }

    /// How its steps are admitted.
    pub(crate) const fn mode(&self) -> Mode {
        self.mode
    }

    /// Its steps, in declared order. The order is the sequence.
    pub(crate) fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// The step `id` names, if this runbook declares one.
    pub(crate) fn step(&self, id: &str) -> Option<&Step> {
        self.steps.iter().find(|step| step.id == id)
    }

    /// What this runbook's discharge records say, evaluated against the step
    /// definitions as they stand **now**.
    ///
    /// `current` maps each step id to the digest the shell computed over
    /// [`Step::material`] this invocation. Derived on every evaluation and never
    /// stored — the rule every other clearance in this machine follows
    /// (DEC-066/DEC-067, and [`super::gate::ReviewStanding`]'s doc). A stored
    /// cursor would be a second source of truth that can disagree with the
    /// records it summarises.
    ///
    /// `verified` carries what the shell's checks returned **this invocation**
    /// (`EX-11`). A record certifies the world as it stood when it was written;
    /// a result that contradicts it now withdraws it, so a run cannot carry a
    /// stale pass through the gate. That withdrawal also moves the cursor back
    /// to the step, which is what makes the repair reachable rather than
    /// wedging the run behind a record it can no longer act on.
    pub(crate) fn standing(
        &self,
        discharges: &[Discharge],
        current: &BTreeMap<String, String>,
        verified: &[StepVerification],
    ) -> RunbookStanding {
        let mut standing = RunbookStanding::default();
        for step in &self.steps {
            match self.live_discharge(step, discharges, current) {
                Some(_) if !StepVerification::refutes(verified, &step.id) => continue,
                // The definition still binds and the record still stands; what
                // it asserted is simply no longer true. Reporting that as
                // `stale` would name the wrong cause — nothing was edited.
                Some(_) => standing.regressed.push(step.id.clone()),
                // A record that exists but no longer binds the definition is a
                // different report from no record at all: the first warns that
                // work was done against something else, the second says it was
                // not done.
                None => {
                    if discharges
                        .iter()
                        .any(|held| held.names(self.key, step.id.as_str()))
                    {
                        standing.stale.push(step.id.clone());
                    }
                }
            }
            if step.required {
                standing.outstanding.push(step.id.clone());
            }
            if standing.cursor.is_none() {
                standing.cursor = Some(step.id.clone());
            }
        }
        standing
    }

    /// This runbook's lines in a turn read: the current obligation, what no
    /// longer stands, and what the run already holds.
    ///
    /// Takes exactly what [`Self::standing`] takes, and for the same reason — a
    /// rendering that could disagree with the standing it depicts would be a
    /// second account of one set of facts. It derives the standing rather than
    /// accepting one, so the two cannot be paired wrongly at a call site.
    ///
    /// **Only the step at the cursor carries its `text`** (`EX-14`). Rendering
    /// every step's prose is the `set` half, and it is deferred to **IMP-373,
    /// not to PHASE-08** — that assignment is withdrawn (DEC-101, amended
    /// 2026-08-01): what a cursorless runbook renders under `EX-14`'s token
    /// bound is unsketched. It is also the net token regression `EX-14`
    /// forbids: one step's text is cheaper than the whole process fragment,
    /// five steps' text is not.
    ///
    /// **On `verified` at a read.** A read runs no checks and passes `&[]`, so
    /// the regressed arm reports nothing there. That is deliberate rather than
    /// overlooked: spawning every required verifier behind `design resume` would
    /// put a subprocess of unbounded duration on a read path, which no criterion
    /// asks for. A regression is reported where it is detected — at the gate,
    /// which re-reads the world before it lets the run past (`EX-11`).
    pub(crate) fn section(
        &self,
        discharges: &[Discharge],
        current: &BTreeMap<String, String>,
        verified: &[StepVerification],
    ) -> Vec<String> {
        let standing = self.standing(discharges, current, verified);
        let name = self.key.name();
        let at_cursor = standing.cursor.as_ref().and_then(|id| {
            self.steps
                .iter()
                .enumerate()
                .find(|(_, step)| &step.id == id)
        });
        let mut lines = vec![match at_cursor {
            Some((index, step)) => format!(
                "runbook {name} obligation {}/{} {} — {}",
                index + 1,
                self.steps.len(),
                step.id,
                step.text
            ),
            None => format!("runbook {name} cleared — every step discharged"),
        }];

        // The two warnings lead, because they are the only lines that ask for
        // work the obligation does not already name. `EX-16`: they WARN — a
        // stage the run already cleared stays cleared.
        for id in &standing.stale {
            lines.push(format!(
                "  stale {id} — its definition changed after it was discharged; discharge it again"
            ));
        }
        for id in &standing.regressed {
            lines.push(format!(
                "  regressed {id} — its check no longer clears; repair what the check \
                 objects to, not the step"
            ));
        }

        for step in &self.steps {
            // A regressed step holds a live record AND a contradicting result.
            // Reporting both would print `verified` beside `regressed` for one
            // step; the regression is the fact that supersedes, and the record
            // itself is still in the snapshot for an auditor.
            if standing.regressed.iter().any(|id| id == &step.id) {
                continue;
            }
            if let Some(held) = self.live_discharge(step, discharges, current) {
                lines.push(format!("  discharged {} {}", step.id, held.rendered()));
            }
        }
        lines
    }

    /// This step's discharge, if it has one still bound to the definition as it
    /// stands now. A record written under a superseded digest **version** is not
    /// live either: it bound bytes this build no longer knows how to reproduce.
    pub(crate) fn live_discharge<'a>(
        &self,
        step: &Step,
        discharges: &'a [Discharge],
        current: &BTreeMap<String, String>,
    ) -> Option<&'a Discharge> {
        let digest = current.get(&step.id)?;
        discharges.iter().find(|held| {
            held.names(self.key, step.id.as_str())
                && held.version == RUNBOOK_STEP_DIGEST_VERSION
                && &held.digest == digest
        })
    }
}

/// A step id is not a [`super::ids::DesignId`], so it carries its own grammar:
/// dot-separated segments of lowercase ASCII alphanumerics, `-` and `_`, each
/// segment non-empty. One rule covers a leading dot, a trailing dot and a
/// doubled dot, which is why it is expressed over segments rather than as a
/// charset plus three special cases.
fn validate_step_id(id: &str) -> anyhow::Result<()> {
    if id.is_empty() {
        anyhow::bail!("a step id may not be empty");
    }
    if id.len() > RUNBOOK_STEP_ID_BYTES {
        anyhow::bail!(
            "step id `{id}` is {} bytes, over the {RUNBOOK_STEP_ID_BYTES}-byte bound — \
             refused rather than truncated, because a shortened identity is a wrong one",
            id.len()
        );
    }
    let well_formed = id.split('.').all(|segment| {
        !segment.is_empty()
            && segment.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
            })
    });
    if !well_formed {
        anyhow::bail!(
            "step id `{id}` is ill-formed — dot-separated segments of lowercase ASCII \
             alphanumerics, `-` and `_`, each segment non-empty"
        );
    }
    Ok(())
}

/// A `verify` argv must be non-empty and may interpolate only [`PLACEHOLDERS`].
fn validate_verify(step: &Step) -> anyhow::Result<()> {
    let Some(argv) = step.verify() else {
        return Ok(());
    };
    if argv.is_empty() {
        anyhow::bail!(
            "step `{}` declares an empty `verify` argv — omit `verify` to mean \
             `no check`, rather than declaring a check that cannot be executed",
            step.id
        );
    }
    for element in argv {
        for name in placeholders_in(element)? {
            if !PLACEHOLDERS.contains(&name.as_str()) {
                anyhow::bail!(
                    "step `{}` interpolates unknown placeholder `{{{name}}}` — the closed \
                     vocabulary is {PLACEHOLDERS:?}",
                    step.id
                );
            }
        }
    }
    Ok(())
}

/// Every `{name}` in one argv element. An unterminated `{` is an error rather
/// than a literal: a runbook author who meant a brace and got a silent pass
/// would find out at execution, which is the failure mode `EX-3` exists to close.
fn placeholders_in(element: &str) -> anyhow::Result<Vec<String>> {
    let mut names = Vec::new();
    let mut rest = element;
    while let Some(open) = rest.find('{') {
        let after = rest.get(open + 1..).unwrap_or_default();
        let Some(close) = after.find('}') else {
            anyhow::bail!("unterminated `{{` in `{element}`");
        };
        names.push(after.get(..close).unwrap_or_default().to_owned());
        rest = after.get(close + 1..).unwrap_or_default();
    }
    Ok(names)
}

/// What a discharge concluded.
///
/// Three arms in the record, **two** on the wire: a caller may say `attested`
/// or `skipped` and may never say `verified`. A verifier result is exactly the
/// kind of fact Doctrine must derive rather than accept on a caller's word. The
/// gate conditions used to make that point through a refusal; since `SL-244`
/// they make it structurally, by having no wire slot to claim one through at
/// all — which is this enum's own arrangement, one tier weaker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum DischargeOutcome {
    /// The agent says it did the work. Recorded, revision-bound, digest-bound,
    /// auditable — and *believed*. No exit code proves an agent read something.
    Attested,
    /// The attestation is corroborated by a check that exited zero.
    Verified,
    /// The step could not be done, with a stated reason. Not a nicety: once the
    /// gate blocks on required steps, a runbook carrying a step its agent cannot
    /// satisfy would otherwise wedge the run. Disclosed deviation beats both a
    /// wedged machine and a silent skip.
    Skipped,
}

/// One discharge — new run state, versioned.
///
/// The payload fields are `Option`s beside a unit-enum outcome rather than enum
/// variant payloads, which is [`super::attestation::RecoveryIntent`]'s idiom and
/// the shape that renders cleanly in TOML. The constructors are the only route
/// to the type, so an outcome that disagrees with its payload is unrepresentable
/// rather than merely discouraged — [`super::attestation::AcceptanceAttestation`]'s
/// rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Discharge {
    /// Which canonical encoding [`Self::digest`] was computed under.
    version: String,
    /// Which runbook's step, by [`RunbookKey::name`].
    runbook: String,
    step: String,
    /// [`Step::material`]'s digest at the moment of discharge — the binding that
    /// makes an edit to the definition surface as stale.
    digest: String,
    /// The run revision it was discharged at.
    revision: u64,
    outcome: DischargeOutcome,
    /// Why the step was skipped. Present iff [`DischargeOutcome::Skipped`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    /// The verifier's exit code. Present iff [`DischargeOutcome::Verified`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    exit: Option<i32>,
    /// The verifier's captured stdout and stderr. Present iff
    /// [`DischargeOutcome::Verified`] — a bare exit code tells an auditor the
    /// check passed and nothing about what it saw.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    output: Option<String>,
}

impl Discharge {
    fn bound(
        runbook: RunbookKey,
        step: &str,
        digest: &str,
        revision: u64,
        outcome: DischargeOutcome,
    ) -> Discharge {
        Discharge {
            version: RUNBOOK_STEP_DIGEST_VERSION.to_owned(),
            runbook: runbook.name().to_owned(),
            step: step.to_owned(),
            digest: digest.to_owned(),
            revision,
            outcome,
            reason: None,
            exit: None,
            output: None,
        }
    }

    /// The agent says it did the work, and no check corroborates it.
    pub(crate) fn attested(
        runbook: RunbookKey,
        step: &str,
        digest: &str,
        revision: u64,
    ) -> Discharge {
        Discharge::bound(runbook, step, digest, revision, DischargeOutcome::Attested)
    }

    /// The agent did the work and a check exited zero, capturing what it said.
    pub(crate) fn verified(
        runbook: RunbookKey,
        step: &str,
        digest: &str,
        revision: u64,
        exit: i32,
        output: impl Into<String>,
    ) -> Discharge {
        Discharge {
            exit: Some(exit),
            output: Some(output.into()),
            ..Discharge::bound(runbook, step, digest, revision, DischargeOutcome::Verified)
        }
    }

    /// The step could not be done, for this stated reason.
    pub(crate) fn skipped(
        runbook: RunbookKey,
        step: &str,
        digest: &str,
        revision: u64,
        reason: impl Into<String>,
    ) -> Discharge {
        Discharge {
            reason: Some(reason.into()),
            ..Discharge::bound(runbook, step, digest, revision, DischargeOutcome::Skipped)
        }
    }

    /// Whether this record is about `step` of `runbook`.
    fn names(&self, runbook: RunbookKey, step: &str) -> bool {
        self.runbook == runbook.name() && self.step == step
    }

    /// Which step this discharged.
    pub(crate) fn step(&self) -> &str {
        &self.step
    }

    /// What it concluded.
    pub(crate) const fn outcome(&self) -> DischargeOutcome {
        self.outcome
    }

    /// Why it was skipped, when it was.
    pub(crate) fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    /// What the verifier said, when there was one.
    pub(crate) fn output(&self) -> Option<&str> {
        self.output.as_deref()
    }

    /// How this record reads in a turn: the outcome word, and for a skip the
    /// disclosure that makes it one (`EX-9`).
    ///
    /// The word is read off the RECORD, which is the whole of `EX-15`'s
    /// guarantee: [`DischargeOutcome::Verified`] is reachable only through
    /// [`Discharge::verified`], and only a check that exited zero calls it. So a
    /// step carrying no verifier cannot acquire the stronger word by any route
    /// through this function — the rendering inherits the constructor's
    /// invariant rather than restating it as a second rule that could drift.
    fn rendered(&self) -> String {
        match self.outcome {
            DischargeOutcome::Attested => "attested".to_owned(),
            DischargeOutcome::Verified => "verified".to_owned(),
            // A skip with no reason is refused at admission, so the fallback is
            // unreachable through the writer path; it is here because a hand-
            // edited snapshot should read as defective, not panic a read.
            DischargeOutcome::Skipped => format!(
                "skipped — {}",
                self.reason.as_deref().unwrap_or("NO REASON RECORDED")
            ),
        }
    }
}

/// Every discharge a run holds. Mirrors [`super::snapshot::FragmentGroup`] — a
/// flat list under its own group, `#[serde(default)]` so a snapshot written
/// before this group existed reads as holding none.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RunbookGroup {
    #[serde(default, rename = "discharge")]
    pub(crate) discharges: Vec<Discharge>,
}

impl RunbookGroup {
    /// Record `discharge`, replacing any prior record for the same step of the
    /// same runbook.
    ///
    /// Replace rather than append: a step has at most one standing answer, and
    /// keeping superseded ones would make [`Runbook::live_discharge`] depend on
    /// which it happened to find first.
    pub(crate) fn upsert(&mut self, discharge: Discharge) {
        self.discharges
            .retain(|held| !(held.runbook == discharge.runbook && held.step == discharge.step));
        self.discharges.push(discharge);
    }
}

/// What a run's discharge records say about one runbook, right now.
///
/// Derived, never stored. Three answers rather than one, because they fail for
/// different reasons and are repaired by different acts —
/// [`super::gate::ReviewStanding`]'s four-booleans reasoning applied here.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RunbookStanding {
    /// Required steps with no live discharge, in runbook order. This is what
    /// the gate blocks on, and what its refusal names.
    pub(crate) outstanding: Vec<String>,
    /// Steps whose discharge exists but no longer matches the definition. These
    /// **warn**; they do not block a stage the run already cleared (`EX-16`).
    pub(crate) stale: Vec<String>,
    /// Steps whose discharge still binds its definition but whose check has
    /// since stopped clearing (`EX-11`). Separate from [`Self::stale`] because
    /// the cause is different — the world moved, not the runbook — and so is
    /// the repair: fix what the check objects to, not the step.
    pub(crate) regressed: Vec<String>,
    /// The step a discharge must name next: the first step, required or not,
    /// with no live discharge. `None` once every step is discharged.
    pub(crate) cursor: Option<String>,
}

impl RunbookStanding {
    /// Whether every required step is discharged against its current definition.
    pub(crate) const fn cleared(&self) -> bool {
        self.outstanding.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A binding set whose `repo_root` holds BOTH a space and a shell
    /// metacharacter — the pair `VA-6` asks for, in one value.
    const HOSTILE_ROOT: &str = "/tmp/a project; rm -rf /";

    fn hostile() -> Bindings<'static> {
        Bindings {
            slice: "SL-233",
            run: "dr-0001",
            repo_root: HOSTILE_ROOT,
            step: "explore.research",
        }
    }

    fn argv(elements: &[&str]) -> Vec<String> {
        elements.iter().map(|e| (*e).to_owned()).collect()
    }

    /// `EX-4` / `VA-6` — the array is the safety property.
    ///
    /// A value carrying a space and a `;` must arrive as ONE argument. If
    /// interpolation ever went through a command string, this value would become
    /// two arguments and a second command; the assertion that survives that
    /// regression is the element *count* plus the exact byte content.
    #[test]
    fn a_value_with_a_space_and_a_metacharacter_stays_one_argument() {
        let out = interpolate(&argv(&["check", "--root", "{repo_root}"]), hostile());
        assert_eq!(out.len(), 3, "interpolation must never re-split an element");
        assert_eq!(
            out.get(2).map(String::as_str),
            Some(HOSTILE_ROOT),
            "the value arrives byte-for-byte, unescaped and unsplit: {out:?}"
        );
    }

    /// A placeholder need not be the whole element, and substituting inside one
    /// still yields exactly one element.
    #[test]
    fn a_placeholder_embedded_in_an_element_substitutes_in_place() {
        let out = interpolate(&argv(&["--slice={slice}", "{step}"]), hostile());
        assert_eq!(
            out,
            argv(&["--slice=SL-233", "explore.research"]),
            "embedded placeholders substitute without changing the argv shape"
        );
    }

    /// Every name [`PLACEHOLDERS`] declares actually binds. A placeholder that
    /// validated but interpolated to nothing would be worse than one refused.
    #[test]
    fn every_declared_placeholder_binds_to_a_value() {
        let each: Vec<String> = PLACEHOLDERS
            .iter()
            .map(|name| format!("{{{name}}}"))
            .collect();
        let out = interpolate(&each, hostile());
        assert!(
            out.iter()
                .all(|value| !value.is_empty() && !value.contains('{')),
            "every declared placeholder must bind: {out:?}"
        );
    }

    const TWO_STEPS: &str = r#"mode = "sequence"

[[step]]
id       = "explore.scope"
text     = "Read the slice scope."

[[step]]
id       = "explore.research"
text     = "Ensure the research round has run."
verify   = ["doctrine", "verify", "research-current", "--slice", "{slice}"]
"#;

    fn book(text: &str) -> anyhow::Result<Runbook> {
        Runbook::parse(RunbookKey::Exploring, text)
    }

    fn digests(runbook: &Runbook) -> BTreeMap<String, String> {
        // A stand-in for the shell's `git::sha256`: this module may not hash,
        // and the derivation under test only compares digests for equality.
        runbook
            .steps()
            .iter()
            .map(|step| (step.id().to_owned(), step.material()))
            .collect()
    }

    #[test]
    fn a_step_defaults_to_required_and_a_verifier_is_optional() {
        let parsed = book(TWO_STEPS).unwrap();
        assert_eq!(parsed.mode(), Mode::Sequence);
        assert_eq!(parsed.key().asset_key(), format!("{STORE}/exploring.toml"));
        let scope = parsed.step("explore.scope").unwrap();
        assert!(
            scope.required(),
            "a step is required unless it says otherwise"
        );
        assert_eq!(scope.verify(), None);
        assert_eq!(
            parsed
                .step("explore.research")
                .unwrap()
                .verify()
                .unwrap()
                .first(),
            Some(&"doctrine".to_owned())
        );
    }

    #[test]
    fn the_digest_material_distinguishes_every_field_it_binds() {
        let base = book(TWO_STEPS).unwrap();
        let scope = base.step("explore.scope").unwrap().material();

        let retexted =
            book(&TWO_STEPS.replace("Read the slice scope.", "Read the scope.")).unwrap();
        assert_ne!(scope, retexted.step("explore.scope").unwrap().material());

        let advisory = book(&TWO_STEPS.replace(
            "id       = \"explore.scope\"",
            "id       = \"explore.scope\"\nrequired = false",
        ))
        .unwrap();
        assert_ne!(
            scope,
            advisory.step("explore.scope").unwrap().material(),
            "flipping `required` must change the binding — whether an old discharge \
             suffices for a now-required step is exactly what is undefined otherwise"
        );

        let substituted = book(&TWO_STEPS.replace("research-current", "research-strict")).unwrap();
        assert_ne!(
            base.step("explore.research").unwrap().material(),
            substituted.step("explore.research").unwrap().material(),
            "substituting a verifier must change the binding: the id carries the \
             contract's name, never the contract"
        );
    }

    #[test]
    fn framing_by_length_keeps_a_fields_content_from_reading_as_a_boundary() {
        // Two definitions whose fields concatenate identically: `a` + `5:hello`
        // and `a5` + `:hello`. Unframed — or framed by any delimiter a field can
        // itself contain — these encode alike and one discharge would satisfy
        // the other. Length prefixes make the boundary unforgeable.
        let left =
            book("mode = \"sequence\"\n\n[[step]]\nid = \"a\"\ntext = \"5:hello\"\n").unwrap();
        let right =
            book("mode = \"sequence\"\n\n[[step]]\nid = \"a5\"\ntext = \":hello\"\n").unwrap();
        let (left, right) = (
            left.steps().first().unwrap(),
            right.steps().first().unwrap(),
        );
        assert_eq!(
            format!("{}{}", left.id(), left.text()),
            format!("{}{}", right.id(), right.text()),
            "the fixture only demonstrates anything while these do collide unframed"
        );
        assert_ne!(left.material(), right.material());
    }

    #[test]
    fn validation_refuses_the_whole_domain_before_anything_executes() {
        let cases = [
            ("mode = \"sequence\"\n", "declares no steps"),
            (
                "mode = \"set\"\n\n[[step]]\nid = \"a\"\ntext = \"t\"\n",
                "unknown variant",
            ),
            (
                "mode = \"sequence\"\nextra = 1\n\n[[step]]\nid = \"a\"\ntext = \"t\"\n",
                "unknown field",
            ),
            (
                "mode = \"sequence\"\n\n[[step]]\nid = \"a\"\ntext = \"t\"\nnote = \"x\"\n",
                "unknown field",
            ),
            (
                "mode = \"sequence\"\n\n[[step]]\nid = \"a\"\ntext = \"   \"\n",
                "empty `text`",
            ),
            (
                "mode = \"sequence\"\n\n[[step]]\nid = \"a\"\ntext = \"t\"\n\n[[step]]\nid = \"a\"\ntext = \"u\"\n",
                "twice",
            ),
            (
                "mode = \"sequence\"\n\n[[step]]\nid = \"Explore.Canon\"\ntext = \"t\"\n",
                "ill-formed",
            ),
            (
                "mode = \"sequence\"\n\n[[step]]\nid = \"explore.\"\ntext = \"t\"\n",
                "ill-formed",
            ),
            (
                "mode = \"sequence\"\n\n[[step]]\nid = \"a\"\ntext = \"t\"\nverify = []\n",
                "empty `verify` argv",
            ),
            (
                "mode = \"sequence\"\n\n[[step]]\nid = \"a\"\ntext = \"t\"\nverify = [\"x\", \"{branch}\"]\n",
                "unknown placeholder",
            ),
            (
                "mode = \"sequence\"\n\n[[step]]\nid = \"a\"\ntext = \"t\"\nverify = [\"x\", \"{slice\"]\n",
                "unterminated",
            ),
        ];
        for (source, expected) in cases {
            let error = book(source)
                .expect_err(&format!("must be refused: {source}"))
                .to_string();
            assert!(
                error.contains(expected),
                "refusal for {source:?} should name `{expected}`, said: {error}"
            );
        }

        let over_bound = format!("x{}", "a".repeat(RUNBOOK_STEP_ID_BYTES));
        let error = book(&format!(
            "mode = \"sequence\"\n\n[[step]]\nid = \"{over_bound}\"\ntext = \"t\"\n"
        ))
        .expect_err("an over-bound id is refused")
        .to_string();
        assert!(error.contains("bound"), "said: {error}");
    }

    #[test]
    fn a_discharged_step_leaves_the_cursor_on_the_next_one() {
        let parsed = book(TWO_STEPS).unwrap();
        let bound = digests(&parsed);

        let empty = parsed.standing(&[], &bound, &[]);
        assert_eq!(empty.cursor.as_deref(), Some("explore.scope"));
        assert_eq!(empty.outstanding, ["explore.scope", "explore.research"]);
        assert!(!empty.cleared());

        let first = Discharge::attested(
            RunbookKey::Exploring,
            "explore.scope",
            bound.get("explore.scope").unwrap(),
            3,
        );
        let after = parsed.standing(std::slice::from_ref(&first), &bound, &[]);
        assert_eq!(after.cursor.as_deref(), Some("explore.research"));
        assert_eq!(after.outstanding, ["explore.research"]);
        assert!(after.stale.is_empty());
    }

    #[test]
    fn every_outcome_discharges_its_step_and_a_skip_keeps_its_reason() {
        let parsed = book(TWO_STEPS).unwrap();
        let bound = digests(&parsed);
        let scope = bound.get("explore.scope").unwrap();
        let research = bound.get("explore.research").unwrap();

        let held = [
            Discharge::skipped(
                RunbookKey::Exploring,
                "explore.scope",
                scope,
                1,
                "no specs govern this surface",
            ),
            Discharge::verified(
                RunbookKey::Exploring,
                "explore.research",
                research,
                2,
                0,
                "research baseline current",
            ),
        ];
        let standing = parsed.standing(&held, &bound, &[]);
        assert!(
            standing.cleared() && standing.cursor.is_none(),
            "a disclosed skip discharges its step rather than wedging the run: {standing:?}"
        );
        assert_eq!(
            held.first().unwrap().reason(),
            Some("no specs govern this surface")
        );
        assert_eq!(held.first().unwrap().outcome(), DischargeOutcome::Skipped);
        assert_eq!(
            held.last().unwrap().output(),
            Some("research baseline current")
        );
        assert_eq!(held.first().unwrap().step(), "explore.scope");
    }

    /// The one line of the section naming `needle`.
    fn line(lines: &[String], needle: &str) -> String {
        lines
            .iter()
            .find(|line| line.contains(needle))
            .unwrap_or_else(|| panic!("no line names `{needle}`: {lines:?}"))
            .clone()
    }

    /// `EX-14` — the CURRENT obligation, with its position, and its text alone.
    ///
    /// Position because an agent should not have to count to learn how far into
    /// a ritual it is. Its text alone because carrying every step's prose is
    /// both the `set` half IMP-373 owns — not PHASE-08; that assignment is
    /// withdrawn (DEC-101, amended 2026-08-01) — and the token regression
    /// `EX-14` forbids: the section has to beat re-sending the whole fragment.
    #[test]
    fn the_section_carries_one_obligation_with_its_position_and_only_its_text() {
        let parsed = book(TWO_STEPS).unwrap();
        let bound = digests(&parsed);

        let fresh = parsed.section(&[], &bound, &[]);
        let obligation = line(&fresh, "explore.scope");
        assert!(
            obligation.contains("1/2") && obligation.contains("Read the slice scope."),
            "the first obligation carries its position and its instruction: {obligation}"
        );
        assert!(
            !fresh
                .iter()
                .any(|l| l.contains("Ensure the research round")),
            "a step the run has not reached carries no prose: {fresh:?}"
        );

        let held = [
            Discharge::attested(
                RunbookKey::Exploring,
                "explore.scope",
                bound.get("explore.scope").unwrap(),
                1,
            ),
            Discharge::verified(
                RunbookKey::Exploring,
                "explore.research",
                bound.get("explore.research").unwrap(),
                2,
                0,
                "research baseline current",
            ),
        ];
        let done = parsed.section(&held, &bound, &[]);
        assert!(
            line(&done, "cleared").contains("every step discharged"),
            "a runbook with no cursor left says so rather than rendering an empty \
             obligation: {done:?}"
        );
    }

    /// `EX-9` — a skip is a *disclosed* deviation, so the disclosure is what the
    /// reader must see. A rendering that showed only `skipped` would turn the
    /// escape hatch back into the silent skip it was added to beat.
    #[test]
    fn a_skipped_step_renders_the_reason_that_makes_it_a_disclosure() {
        let parsed = book(TWO_STEPS).unwrap();
        let bound = digests(&parsed);
        let held = Discharge::skipped(
            RunbookKey::Exploring,
            "explore.scope",
            bound.get("explore.scope").unwrap(),
            1,
            "no specs govern this surface",
        );

        let rendered = line(
            &parsed.section(std::slice::from_ref(&held), &bound, &[]),
            "discharged explore.scope",
        );
        assert!(
            rendered.contains("skipped") && rendered.contains("no specs govern this surface"),
            "the reason rides with the outcome, not merely into the record: {rendered}"
        );
    }

    /// `EX-16` plus T8's call on the fourth report field: **stale** and
    /// **regressed** are different facts and get different words.
    ///
    /// They are repaired differently — a stale step's definition moved and it is
    /// re-discharged; a regressed step's definition is untouched and it is the
    /// world that must be fixed. A reader told "stale" about a regression is
    /// sent to re-read a step that has not changed. Both WARN: neither
    /// un-advances a stage the run already cleared.
    #[test]
    fn stale_and_regressed_are_rendered_as_the_different_facts_they_are() {
        let parsed = book(TWO_STEPS).unwrap();
        let bound = digests(&parsed);
        let held = [
            Discharge::attested(
                RunbookKey::Exploring,
                "explore.scope",
                "a-superseded-digest",
                1,
            ),
            Discharge::verified(
                RunbookKey::Exploring,
                "explore.research",
                bound.get("explore.research").unwrap(),
                2,
                0,
                "research baseline current",
            ),
        ];
        let rendered = parsed.section(
            &held,
            &bound,
            &[StepVerification {
                step: "explore.research".to_owned(),
                exit: Some(1),
                output: "baseline drifted".to_owned(),
            }],
        );

        assert!(
            line(&rendered, "stale explore.scope").contains("definition changed"),
            "a discharge whose definition moved names the edit as the cause: {rendered:?}"
        );
        assert!(
            line(&rendered, "regressed explore.research").contains("check no longer clears"),
            "a discharge the world moved under names the check as the cause: {rendered:?}"
        );
        assert!(
            !rendered
                .iter()
                .any(|l| l.contains("discharged explore.research")),
            "the regression supersedes the record it contradicts; printing `verified` \
             beside `regressed` for one step reads as a contradiction: {rendered:?}"
        );
    }

    #[test]
    fn a_record_written_under_a_superseded_encoding_is_not_live() {
        let parsed = book(TWO_STEPS).unwrap();
        let bound = digests(&parsed);
        let mut held = Discharge::attested(
            RunbookKey::Exploring,
            "explore.scope",
            bound.get("explore.scope").unwrap(),
            1,
        );
        held.version = "runbook-step.v0".to_owned();

        let standing = parsed.standing(std::slice::from_ref(&held), &bound, &[]);
        assert_eq!(
            standing.stale,
            ["explore.scope"],
            "a digest computed under an encoding this build cannot reproduce must \
             surface, not compare equal by accident"
        );
    }

    #[test]
    fn a_discharge_naming_another_runbooks_step_discharges_nothing() {
        let parsed = book(TWO_STEPS).unwrap();
        let bound = digests(&parsed);
        let mut foreign = Discharge::attested(
            RunbookKey::Exploring,
            "explore.scope",
            bound.get("explore.scope").unwrap(),
            1,
        );
        foreign.runbook = "drafting".to_owned();

        let standing = parsed.standing(std::slice::from_ref(&foreign), &bound, &[]);
        assert_eq!(standing.cursor.as_deref(), Some("explore.scope"));
        assert!(
            standing.stale.is_empty(),
            "a record about a different runbook is not this one's stale record"
        );
    }

    /// `EX-11` — a regressed check withdraws its discharge, and says so in its
    /// own words.
    ///
    /// `stale` and `regressed` report different causes and want different
    /// repairs — an edited definition versus a world that moved — so a
    /// regression that reported as stale would send the reader to re-read a
    /// step that has not changed. The cursor moving back is what makes the
    /// repair reachable at all: the sequence admits only the step at the
    /// cursor, so a withdrawn discharge the cursor had already passed would
    /// wedge the run.
    #[test]
    fn a_check_that_stops_clearing_withdraws_its_discharge_without_reading_as_stale() {
        let parsed = book(TWO_STEPS).unwrap();
        let bound = digests(&parsed);
        let held = [
            Discharge::attested(
                RunbookKey::Exploring,
                "explore.scope",
                bound.get("explore.scope").unwrap(),
                1,
            ),
            Discharge::verified(
                RunbookKey::Exploring,
                "explore.research",
                bound.get("explore.research").unwrap(),
                2,
                0,
                "research baseline current",
            ),
        ];
        let cleared = parsed.standing(&held, &bound, &[]);
        assert!(
            cleared.cleared() && cleared.cursor.is_none(),
            "the premise: with no check re-run, both records stand: {cleared:?}"
        );

        let regressed = parsed.standing(
            &held,
            &bound,
            &[StepVerification {
                step: "explore.research".to_owned(),
                exit: Some(1),
                output: "baseline drifted".to_owned(),
            }],
        );
        assert_eq!(regressed.regressed, ["explore.research"]);
        assert_eq!(
            regressed.outstanding,
            ["explore.research"],
            "a required step whose check no longer clears is outstanding again"
        );
        assert!(
            regressed.stale.is_empty(),
            "nothing was edited: reporting this as stale would name the wrong cause \
             and send the reader to the wrong repair: {regressed:?}"
        );
        assert_eq!(
            regressed.cursor.as_deref(),
            Some("explore.research"),
            "the cursor returns to the withdrawn step, or the sequence admits no \
             discharge that could repair it"
        );
    }

    #[test]
    fn every_key_has_a_distinct_name_and_asset_key_under_the_store() {
        let names: BTreeSet<&str> = RunbookKey::ALL.iter().map(|key| key.name()).collect();
        assert_eq!(names.len(), RunbookKey::ALL.len());
        for key in RunbookKey::ALL {
            assert!(key.asset_key().starts_with(STORE));
            assert!(key.asset_key().ends_with(".toml"));
        }
    }
}
