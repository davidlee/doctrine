// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine review` — the RV adversarial-review ledger kind (SL-040, ADR-007),
//! split along the engine/command tier line (SL-268 PHASE-02, design sec-12):
//! this module is the command-tier shell — dispatch, the structured output/
//! error types, and the verb submodules. The engine-tier ledger (vocabulary,
//! schema, derivation, transition table, blocker predicates) lives in
//! `crate::review_ledger`.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::contentset::{self, ContentSet};
use crate::entity::{self, Materialised};
use crate::kinds::{REVIEW_DIR, REVIEW_KIND};
use crate::listing::{self, Column, Format, ListArgs};
use crate::review_ledger::{
    Await, Facet, FindingRow, FindingState, FindingStatus, REVIEW_STATUSES, ReviewDoc, ReviewMeta,
    ReviewStatus, Role, Severity, Target, TurnAct, Verb, append_finding, apply_transition,
    authored_path, can, canonical_id, derived_status, finding_states_of, finding_status_of,
    finding_table_mut, parse_finding_status, parse_ref, read_authored, read_review, read_reviews,
    required_for,
};

mod cli;
mod prime;
mod read;
mod turn;
mod verbs;

pub(crate) use cli::ReviewCommand;
pub(crate) use prime::{PrimeArgs, run_prime};
pub(crate) use read::{Finding, ListRow, run_list, run_show, run_status};
pub(crate) use turn::run_unlock;
pub(crate) use verbs::{
    DisposeArgs, NewArgs, RaiseArgs, materialise_review_at, mint_review, parse_role, run_conclude,
    run_contest, run_dispose, run_new, run_raise, run_verify, run_withdraw,
};

pub(crate) fn dispatch(cmd: ReviewCommand, color: bool) -> anyhow::Result<()> {
    match cmd {
        ReviewCommand::New {
            facet,
            target,
            phase,
            title,
            raiser,
            responder,
            path,
        } => {
            use std::io::Write;
            let out = run_new(
                path,
                &NewArgs {
                    facet,
                    target,
                    phase,
                    title,
                    raiser,
                    responder,
                },
            )?;
            let rendered = print_review(&out);
            write!(std::io::stdout(), "{rendered}")?;
            Ok(())
        }
        ReviewCommand::List { list, target, path } => {
            use std::io::Write;
            let out = run_list(path, list.into_list_args(color), target.as_deref())?;
            let rendered = print_review(&out);
            write!(std::io::stdout(), "{rendered}")?;
            Ok(())
        }
        ReviewCommand::Show {
            reference,
            format,
            json,
            path,
        } => {
            use std::io::Write;
            let out = run_show(path, &reference, if json { Format::Json } else { format })?;
            let rendered = print_review(&out);
            write!(std::io::stdout(), "{rendered}")?;
            Ok(())
        }
        ReviewCommand::Raise {
            reference,
            severity,
            title,
            detail,
            role,
            path,
        } => {
            use std::io::Write;
            let role = parse_role(role.as_deref(), Role::Raiser)?;
            let out = run_raise(
                path,
                &RaiseArgs {
                    reference,
                    severity,
                    title,
                    detail,
                },
                role,
            )?;
            let rendered = print_review(&out);
            write!(std::io::stdout(), "{rendered}")?;
            Ok(())
        }
        ReviewCommand::Dispose {
            reference,
            finding,
            disposition,
            response,
            role,
            path,
        } => {
            use std::io::Write;
            let role = parse_role(role.as_deref(), Role::Responder)?;
            let out = run_dispose(
                path,
                &DisposeArgs {
                    reference,
                    finding,
                    disposition,
                    response,
                },
                role,
            )?;
            let rendered = print_review(&out);
            write!(std::io::stdout(), "{rendered}")?;
            Ok(())
        }
        ReviewCommand::Verify {
            reference,
            finding,
            note,
            role,
            path,
        } => {
            use std::io::Write;
            let role = parse_role(role.as_deref(), Role::Raiser)?;
            let out = run_verify(path, &reference, &finding, note.as_deref(), role)?;
            let rendered = print_review(&out);
            write!(std::io::stdout(), "{rendered}")?;
            Ok(())
        }
        ReviewCommand::Contest {
            reference,
            finding,
            note,
            role,
            path,
        } => {
            use std::io::Write;
            let role = parse_role(role.as_deref(), Role::Raiser)?;
            let out = run_contest(path, &reference, &finding, note.as_deref(), role)?;
            let rendered = print_review(&out);
            write!(std::io::stdout(), "{rendered}")?;
            Ok(())
        }
        ReviewCommand::Withdraw {
            reference,
            finding,
            role,
            path,
        } => {
            use std::io::Write;
            let role = parse_role(role.as_deref(), Role::Raiser)?;
            let out = run_withdraw(path, &reference, &finding, role)?;
            let rendered = print_review(&out);
            write!(std::io::stdout(), "{rendered}")?;
            Ok(())
        }
        ReviewCommand::Conclude {
            reference,
            role,
            path,
        } => {
            use std::io::Write;
            let role = parse_role(role.as_deref(), Role::Raiser)?;
            let out = run_conclude(path, &reference, role)?;
            let rendered = print_review(&out);
            write!(std::io::stdout(), "{rendered}")?;
            Ok(())
        }
        ReviewCommand::Status { reference, path } => {
            use std::io::Write;
            let out = run_status(path, &reference)?;
            let rendered = print_review(&out);
            write!(std::io::stdout(), "{rendered}")?;
            Ok(())
        }
        ReviewCommand::Prime { reference, path } => {
            use std::io::Write;
            let out = run_prime(path, &PrimeArgs { reference })?;
            let rendered = print_review(&out);
            write!(std::io::stdout(), "{rendered}")?;
            Ok(())
        }
        ReviewCommand::Unlock { reference, path } => {
            use std::io::Write;
            let out = run_unlock(path, &reference)?;
            let rendered = print_review(&out);
            write!(std::io::stdout(), "{rendered}")?;
            Ok(())
        }
        ReviewCommand::Paths {
            refs,
            toml,
            md,
            entity,
            single,
            path,
        } => {
            use std::io::Write;
            let root = crate::root::find(path, &crate::root::default_markers())?;
            let review_root = root.join(REVIEW_DIR);
            let sel = crate::paths::PathSelection {
                toml,
                md,
                entity,
                single,
            };
            let mut all_lines: Vec<String> = Vec::new();
            for r in &refs {
                let id = parse_ref(r)?;
                let name = format!("{id:03}");
                let entity_dir = review_root.join(&name);
                let toml_name = format!("review-{name}.toml");
                let md_name = format!("review-{name}.md");
                let set = crate::paths::scan_entity_dir(
                    &entity_dir,
                    &entity_dir.join(&toml_name),
                    Some(&entity_dir.join(&md_name)),
                    &root,
                )?;
                let lines = crate::paths::select_paths(&set, &sel)?;
                all_lines.extend(lines);
            }
            write!(std::io::stdout(), "{}", all_lines.join("\n"))?;
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Structured return types (SL-109, design D1/D8)
// ---------------------------------------------------------------------------

/// The structured output of a review verb — one variant per verb, carrying
/// exactly the data its consumers need. `#[derive(Serialize)]` for MCP
/// transport; the CLI path formats via `print_review()` in `main.rs`.
#[derive(Debug, Serialize)]
pub(crate) enum ReviewOutput {
    Created {
        id: u32,
        canonical: String,
        dir: PathBuf,
    },
    Raised {
        finding_id: String,
        review_id: u32,
    },
    Disposed {
        finding_id: String,
        review_id: u32,
    },
    Verified {
        finding_id: String,
        review_id: u32,
    },
    Contested {
        finding_id: String,
        review_id: u32,
    },
    Withdrawn {
        finding_id: String,
        review_id: u32,
    },
    /// The pass-level conclude (IMP-392). `already` distinguishes the call that
    /// set the latch from the one that found it set — both succeed, and a caller
    /// who cannot know which it made deserves to be told.
    Concluded {
        review_id: u32,
        already: bool,
    },
    Showed {
        id: u32,
        canonical: String,
        title: String,
        status: String,
        awaiting: String,
        facet: String,
        target: String,
        #[serde(rename = "finding_count")]
        findings_count: usize,
        findings: Vec<Finding>,
        body: String,
        #[serde(skip)]
        formatted: String,
    },
    Listed {
        rows: Vec<ListRow>,
        /// Pre-truncation row count, set MCP-side only when an output cap dropped
        /// rows (IMP-114). `None` (absent on the wire) ⇒ the rows are complete —
        /// keeps uncapped lists and the CLI path byte-unchanged.
        #[serde(skip_serializing_if = "Option::is_none")]
        total: Option<usize>,
        #[serde(skip)]
        formatted: String,
    },
    Primed {
        canonical: String,
        tracked_paths: Vec<String>,
        tracked_count: usize,
    },
    Status {
        canonical: String,
        status: String,
        awaiting: String,
        findings_count: usize,
        rounds: usize,
        cache_primed: bool,
        stale_paths: Vec<String>,
        #[serde(skip)]
        formatted: String,
    },
    Unlocked {
        canonical: String,
        #[serde(skip)]
        formatted: String,
    },
}

/// Format a [`ReviewOutput`] for CLI human consumption — the single formatting
/// pass, one match arm per variant, following the output contract (§4 design.md).
/// Returns the formatted string; the caller writes it to stdout.
pub(crate) fn print_review(out: &ReviewOutput) -> String {
    match out {
        ReviewOutput::Created {
            id,
            canonical: _,
            dir,
        } => {
            format!("Created review {:03}: {}\n", id, dir.display())
        }
        ReviewOutput::Raised {
            finding_id,
            review_id,
        } => {
            format!("Raised {} on {}\n", finding_id, canonical_id(*review_id))
        }
        ReviewOutput::Disposed {
            finding_id,
            review_id,
        } => {
            format!(
                "Disposed {} on {} (answered)\n",
                finding_id,
                canonical_id(*review_id)
            )
        }
        ReviewOutput::Verified {
            finding_id,
            review_id,
        } => {
            format!(
                "Verified {} on {} (verified)\n",
                finding_id,
                canonical_id(*review_id)
            )
        }
        ReviewOutput::Contested {
            finding_id,
            review_id,
        } => {
            format!(
                "Contested {} on {} (contested)\n",
                finding_id,
                canonical_id(*review_id)
            )
        }
        ReviewOutput::Withdrawn {
            finding_id,
            review_id,
        } => {
            format!(
                "Withdrew {} on {} (withdrawn)\n",
                finding_id,
                canonical_id(*review_id)
            )
        }
        ReviewOutput::Concluded { review_id, already } => {
            let tail = if *already { " (already concluded)" } else { "" };
            format!("Concluded the pass on {}{tail}\n", canonical_id(*review_id))
        }
        ReviewOutput::Showed { formatted, .. }
        | ReviewOutput::Listed { formatted, .. }
        | ReviewOutput::Status { formatted, .. } => formatted.clone(),
        ReviewOutput::Primed {
            canonical,
            tracked_paths: _,
            tracked_count,
        } => {
            format!(
                "{canonical} primed — {tracked_count} tracked path(s) from the target slice's selectors\n"
            )
        }
        ReviewOutput::Unlocked {
            canonical,
            formatted,
        } => {
            if formatted.is_empty() {
                format!("{canonical} is not locked\n")
            } else {
                formatted.clone()
            }
        }
    }
}

/// Structured error from the review engine — each variant carries typed fields
/// so the MCP transport layer can map to JSON-RPC error codes by variant
/// identity, never by string-parsing (design D8; RV-092 F-1).
#[derive(Debug)]
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "NotFound/Internal are constructed only by mcp_server::tools's own tests \
                  (SL-268 PHASE-02 T6 split)"
    )
)]
pub(crate) enum ReviewError {
    NotFound {
        reference: String,
    },
    RoleMismatch {
        expected: Role,
        actual: Role,
        /// The act refused. A [`TurnAct`] rather than a [`Verb`] because the
        /// pass-level `conclude` takes the same static role check and has no
        /// `Verb` to name itself with.
        act: TurnAct,
    },
    StateMismatch {
        finding: String,
        current: FindingStatus,
        required: FindingStatus,
    },
    DanglingRef {
        target: String,
    },
    LockContention {
        canonical: String,
        details: String,
    },
    Internal {
        source: anyhow::Error,
    },
}

impl fmt::Display for ReviewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { reference } => {
                write!(f, "review not found: {reference}")
            }
            Self::RoleMismatch {
                expected,
                actual,
                act,
            } => {
                write!(
                    f,
                    "`{}` is the {}'s verb; --as {} cannot assert it",
                    act.as_str(),
                    expected.as_str(),
                    actual.as_str()
                )
            }
            Self::StateMismatch {
                finding,
                current,
                required,
            } => {
                write!(
                    f,
                    "out of turn on {finding}: current status {} != required {}",
                    current.as_str(),
                    required.as_str()
                )
            }
            Self::DanglingRef { target } => {
                write!(f, "target not found: {target}")
            }
            Self::LockContention { canonical, details } => {
                write!(f, "{canonical}: {details}")
            }
            Self::Internal { source } => {
                write!(f, "{source}")
            }
        }
    }
}

impl std::error::Error for ReviewError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Internal { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}

/// The past-tense label for a verb's success line.
#[expect(
    dead_code,
    reason = "used by print_review in main.rs via pub(crate) export"
)]
pub(crate) fn verb_past(verb: Verb) -> &'static str {
    match verb {
        Verb::Raise => "Raised",
        Verb::Dispose => "Disposed",
        Verb::Verify => "Verified",
        Verb::Contest => "Contested",
        Verb::Withdraw => "Withdrew",
    }
}

#[cfg(test)]
mod tests;
