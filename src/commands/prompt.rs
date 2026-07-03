// SPDX-License-Identifier: GPL-3.0-only
//! Prompt-cascade CLI verbs (SL-186 PHASE-04).
//!
//! The impure corpus loader lives in `src/install.rs` (beside `embedded_hymns`,
//! the embed it reads). This module houses CLI parsing, dispatch, `build_ctx`,
//! and `check_corpus`.

use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::hymns::{Arm, Band, Provenance, SealSet, Snippet};

// ── PromptCommand + dispatch (PHASE-03) ────────────────────────────────────

use clap::Subcommand;

#[derive(Subcommand)]
pub(crate) enum PromptCommand {
    /// Resolve the prompt cascade and emit assembled markdown to stdout.
    Resolve {
        /// The role: "worker" or "orchestrator".
        #[arg(long, required = true)]
        role: String,

        /// The harness (e.g. "claude", "pi").
        #[arg(long)]
        harness: Option<String>,

        /// Model key (e.g. "anthropic/claude-sonnet-4"). Repeatable — each
        /// occurrence adds a member to the context trait set (membership matching).
        #[arg(long)]
        model: Vec<String>,

        /// The dispatch arm: "subagent" or "subprocess".
        #[arg(long)]
        arm: Option<String>,

        /// The stage label (e.g. "execute").
        #[arg(long)]
        stage: Option<String>,

        /// Restrict output to specific bands (repeatable). Empty = all bands.
        #[arg(long, value_name = "BAND")]
        band: Vec<String>,

        /// Wrap stdout as a Cursor `sessionStart` hook JSON envelope
        /// (`{"additional_context": "<cascade>"}`) instead of raw markdown.
        #[arg(long)]
        json: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// List every model snippet label (a full relative key) in the corpus.
    #[clap(name = "model-keys")]
    ModelKeys {
        /// Filter to models matching the given harness (or None/universal).
        #[arg(long)]
        harness: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Show the resolution order with specificity and provenance for each matched snippet.
    Explain {
        /// The role: "worker" or "orchestrator".
        #[arg(long, required = true)]
        role: String,

        /// The harness (e.g. "claude", "pi").
        #[arg(long)]
        harness: Option<String>,

        /// Model key (e.g. "anthropic/claude-sonnet-4"). Repeatable — each
        /// occurrence adds a member to the context trait set (membership matching).
        #[arg(long)]
        model: Vec<String>,

        /// The dispatch arm: "subagent" or "subprocess".
        #[arg(long)]
        arm: Option<String>,

        /// The stage label (e.g. "execute").
        #[arg(long)]
        stage: Option<String>,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Validate the replaces graph + stage vocabulary over the whole corpus.
    Check {
        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

pub(crate) fn dispatch(cmd: PromptCommand, command_map: fn() -> String) -> anyhow::Result<()> {
    use std::io::Write;
    match cmd {
        PromptCommand::Resolve {
            role,
            harness,
            model,
            arm,
            stage,
            band,
            json,
            path,
        } => {
            let root = crate::root::find(path, &crate::root::default_markers())?;
            let disk_root = root.join(".doctrine").join(crate::install::HYMNS_DIRNAME);
            let embedded = crate::install::embedded_hymns();
            let sealed = crate::install::embedded_seal_set()?;
            let corpus = crate::install::load_full_corpus(&disk_root, &embedded, &sealed)?;

            // Delivery (SL-187 PHASE-04): unstale the UNIVERSAL on-disk boot.md via
            // the shared boot generator (axis-invariant, INV-D1), then emit the
            // universal snapshot ++ the role/harness hymns to stdout. The disk write
            // is idempotent; stdout carries the axis-specific cascade.
            let exec = crate::boot::resolve_exec()?;
            let universal = crate::boot::resolve_universal_snapshot(&root, &exec, command_map)?;
            let ctx = build_ctx(&role, harness, model, arm.as_deref(), stage, &band)?;
            let hymns = crate::hymns::resolve(&ctx, &corpus, &sealed)?;
            let cascade = format!("{}\n{hymns}", universal.trim_end());
            if json {
                write!(
                    std::io::stdout(),
                    "{}",
                    crate::boot::session_start_hook_json(&cascade)?
                )?;
            } else {
                writeln!(std::io::stdout(), "{cascade}")?;
            }
            Ok(())
        }
        PromptCommand::ModelKeys { harness, path } => {
            let root = crate::root::find(path, &crate::root::default_markers())?;
            let labels = model_keys(&root, harness.as_deref())?;
            let mut stdout = std::io::stdout();
            for label in labels {
                writeln!(stdout, "{label}")?;
            }
            Ok(())
        }
        PromptCommand::Explain {
            role,
            harness,
            model,
            arm,
            stage,
            path,
        } => {
            let root = crate::root::find(path, &crate::root::default_markers())?;
            let disk_root = root.join(".doctrine").join(crate::install::HYMNS_DIRNAME);
            let embedded = crate::install::embedded_hymns();
            let sealed = crate::install::embedded_seal_set()?;
            let corpus = crate::install::load_full_corpus(&disk_root, &embedded, &sealed)?;

            let ctx = build_ctx(&role, harness, model, arm.as_deref(), stage, &[])?;

            // Active set: same filters as resolve
            let mut active: Vec<&Snippet> = corpus
                .iter()
                .filter(|s| !(s.provenance == Provenance::User && sealed.0.contains(&s.slot)))
                .filter(|s| ctx.bands.includes(s.slot.band))
                .filter(|s| crate::hymns::matches(&s.selector, &ctx))
                .collect();
            active.sort_by_key(|s| crate::hymns::precedence_key(s));

            let mut stdout = std::io::stdout();
            for (rank, s) in active.iter().enumerate() {
                let winner = rank == active.len() - 1;
                let spec = crate::hymns::specificity(s.slot.band, &s.selector);
                // Render the root-wise primary as `[root:depth,…]` (house style:
                // build via Vec<String>+join, never push_str(&format!) — the
                // `format_push_string` deny).
                let pairs = spec
                    .0
                    .iter()
                    .map(|(pair_root, depth)| format!("{pair_root}:{depth}"))
                    .collect::<Vec<_>>()
                    .join(",");
                let prov_str = match s.provenance {
                    crate::hymns::Provenance::Framework => "Framework",
                    crate::hymns::Provenance::User => "User",
                };
                writeln!(
                    stdout,
                    "{:<50} prov={prov_str} spec=([{pairs}],{}) rank={}{}",
                    s.slot.path(),
                    spec.1,
                    rank,
                    if winner { " ★ WINNER" } else { "" }
                )?;
            }
            Ok(())
        }
        PromptCommand::Check { path } => {
            let root = crate::root::find(path, &crate::root::default_markers())?;
            let disk_root = root.join(".doctrine").join(crate::install::HYMNS_DIRNAME);
            let embedded = crate::install::embedded_hymns();
            let sealed = crate::install::embedded_seal_set()?;
            let corpus = crate::install::load_full_corpus(&disk_root, &embedded, &sealed)?;

            let problems = check_corpus(&corpus, &sealed);
            if problems.is_empty() {
                writeln!(std::io::stdout(), "check: corpus OK")?;
            } else {
                for p in &problems {
                    writeln!(std::io::stderr(), "{p}")?;
                }
                anyhow::bail!("{} problem(s) found", problems.len());
            }
            Ok(())
        }
    }
}

fn build_ctx(
    role: &str,
    harness: Option<String>,
    model: Vec<String>,
    arm: Option<&str>,
    stage: Option<String>,
    band: &[String],
) -> anyhow::Result<crate::hymns::ContextVector> {
    let role = crate::install::parse_role(role)?;
    let arm: Option<Arm> = arm.map(crate::install::parse_arm).transpose()?;

    let bands = if band.is_empty() {
        crate::hymns::BandFilter::All
    } else {
        let mut set = std::collections::BTreeSet::new();
        for seg in band {
            let b = Band::from_segment(seg).with_context(|| format!("unknown band {seg:?}"))?;
            set.insert(b);
        }
        crate::hymns::BandFilter::Only(set)
    };

    // Repeatable `--model` builds the context trait set (membership matching);
    // absent = empty set = the unpinned don't-care, one occurrence = singleton.
    let model = model
        .into_iter()
        .collect::<std::collections::BTreeSet<String>>();

    Ok(crate::hymns::ContextVector {
        role,
        harness,
        model,
        arm,
        stage,
        bands,
    })
}

/// The sorted, de-duplicated set of model-band snippet labels (the `--model`
/// key strings) in the corpus, optionally filtered to a harness (universal
/// model snippets always pass the filter). Shared by the `prompt model-keys`
/// verb and the `doctrine_onboard` MCP tool.
pub(crate) fn model_keys(root: &Path, harness: Option<&str>) -> anyhow::Result<Vec<String>> {
    let disk_root = root.join(".doctrine").join(crate::install::HYMNS_DIRNAME);
    let embedded = crate::install::embedded_hymns();
    let sealed = crate::install::embedded_seal_set()?;
    let corpus = crate::install::load_full_corpus(&disk_root, &embedded, &sealed)?;

    let mut labels: Vec<String> = corpus
        .iter()
        .filter(|s| s.slot.band == Band::Model)
        .filter(|s| match harness {
            Some(h) => s.selector.harness.is_none() || s.selector.harness.as_deref() == Some(h),
            None => true,
        })
        .map(|s| s.slot.label.clone())
        .collect();
    labels.sort_unstable();
    labels.dedup();
    Ok(labels)
}

/// Collect all problems in the corpus: replaces graph errors, unknown stage
/// labels, sealed-slot integrity breaches, and def-marker integrity.
/// Returns human-readable messages.
pub(crate) fn check_corpus(corpus: &[Snippet], sealed: &SealSet) -> Vec<String> {
    let mut problems = Vec::new();

    // (a) replaces graph
    if let Err(e) = crate::hymns::validate_replaces(corpus) {
        problems.push(format!("replaces graph: {e}"));
    }

    // (b) unknown stage labels
    let known: std::collections::BTreeSet<&str> =
        crate::install::KNOWN_STAGE_LABELS.iter().copied().collect();
    for s in corpus {
        if s.slot.band == Band::Stage && !known.contains(s.slot.label.as_str()) {
            problems.push(format!(
                "unknown stage label {:?} in slot {}",
                s.slot.label,
                s.slot.path()
            ));
        }
    }

    // (c) sealed-slot integrity
    // Every sealed slot must exist among embedded (Framework) snippets.
    for slot in &sealed.0 {
        let present = corpus
            .iter()
            .any(|s| s.slot == *slot && s.provenance == Provenance::Framework);
        if !present {
            problems.push(format!(
                "sealed slot {} has no Framework snippet",
                slot.path()
            ));
        }
    }
    // No User snippet may occupy a sealed slot (the loader already drops them).
    for s in corpus {
        if s.provenance == Provenance::User && sealed.0.contains(&s.slot) {
            problems.push(format!(
                "User snippet occupies sealed slot {}",
                s.slot.path()
            ));
        }
    }

    // (d) def-marker integrity (SL-186 PHASE-04 / T6)
    for (name, bytes) in crate::install::embedded_agent_defs() {
        let Ok(def_text) = std::str::from_utf8(&bytes) else {
            continue;
        };
        if !def_text.contains(crate::install::WORKER_RESOLVE_MARKER) {
            continue;
        }
        match crate::install::resolve_worker_role_body(corpus, sealed) {
            Ok(body) if !body.trim().is_empty() => {}
            Ok(_) | Err(_) => {
                problems.push(format!(
                    "def {name}: marker present but role band unresolvable"
                ));
            }
        }
    }

    problems
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hymns::{Selector, Slot};

    fn corpus_with_resolvable_worker_role(mut corpus: Vec<Snippet>) -> Vec<Snippet> {
        corpus.push(Snippet {
            slot: Slot::new(Band::Role, "worker"),
            selector: Selector {
                role: Some(crate::hymns::Role::Worker),
                ..Default::default()
            },
            provenance: Provenance::Framework,
            body: "worker body".into(),
        });
        corpus
    }

    // ── VT-3: check_corpus unit tests ────────────────────────────────────

    #[test]
    fn check_duplicate_replaces_target_is_flagged() {
        let target = Slot::new(Band::Project, "target");
        let corpus = corpus_with_resolvable_worker_role(vec![
            Snippet {
                slot: Slot::new(Band::Project, "a"),
                selector: Selector {
                    replaces: Some(target.clone()),
                    ..Default::default()
                },
                provenance: Provenance::Framework,
                body: "A".into(),
            },
            Snippet {
                slot: Slot::new(Band::Project, "b"),
                selector: Selector {
                    replaces: Some(target.clone()),
                    ..Default::default()
                },
                provenance: Provenance::Framework,
                body: "B".into(),
            },
        ]);
        let sealed = SealSet::default();
        let problems = check_corpus(&corpus, &sealed);
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("two active snippets")),
            "expected DuplicateTarget, got: {problems:?}"
        );
    }

    #[test]
    fn check_unknown_stage_label_is_flagged() {
        let corpus = corpus_with_resolvable_worker_role(vec![Snippet {
            slot: Slot::new(Band::Stage, "bogus-stage"),
            selector: Selector {
                stage: Some("bogus-stage".into()),
                ..Default::default()
            },
            provenance: Provenance::Framework,
            body: "X".into(),
        }]);
        let sealed = SealSet::default();
        let problems = check_corpus(&corpus, &sealed);
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("unknown stage label")),
            "expected unknown stage, got: {problems:?}"
        );
    }

    #[test]
    fn check_clean_corpus_passes() {
        let corpus = corpus_with_resolvable_worker_role(vec![Snippet {
            slot: Slot::new(Band::Preamble, "core"),
            selector: Selector::default(),
            provenance: Provenance::Framework,
            body: "FW".into(),
        }]);
        let sealed = SealSet::default();
        let problems = check_corpus(&corpus, &sealed);
        assert!(problems.is_empty(), "expected clean, got {:?}", problems);
    }

    // ── VT-2: def-marker integrity (SL-186 PHASE-04) ────────────────────

    #[test]
    fn check_def_marker_present_role_unresolvable_is_flagged() {
        let problems = check_corpus(&[], &SealSet::default());
        let has_dispatch_worker = problems.iter().any(|problem| {
            problem.contains("dispatch-worker")
                && problem.contains("marker present but role band unresolvable")
        });
        assert!(
            has_dispatch_worker,
            "expected marker resolve problem, got {problems:?}"
        );
    }

    // ── VT-5: explain ordering unit test ─────────────────────────────────

    #[test]
    fn explain_framework_exact_model_outranks_user_vendor_default() {
        use crate::hymns;

        let fw_exact = Snippet {
            slot: Slot::new(Band::Model, "anthropic/claude-sonnet-4"),
            selector: Selector {
                model: ["anthropic/claude-sonnet-4".into()].into(),
                ..Default::default()
            },
            provenance: Provenance::Framework,
            body: "FW-EXACT".into(),
        };
        let user_default = Snippet {
            slot: Slot::new(Band::Model, "anthropic/_default"),
            selector: Selector {
                model: ["anthropic/_default".into()].into(),
                ..Default::default()
            },
            provenance: Provenance::User,
            body: "USER-DEFAULT".into(),
        };

        // specificity: fw_exact = ([(anthropic,2)],0), user_default = ([(anthropic,1)],0)
        let fw_spec = hymns::specificity(fw_exact.slot.band, &fw_exact.selector);
        let user_spec = hymns::specificity(user_default.slot.band, &user_default.selector);
        assert!(
            fw_spec > user_spec,
            "expected ([(anthropic,2)],0) > ([(anthropic,1)],0)"
        );

        let mut active: Vec<&Snippet> = vec![&fw_exact, &user_default];
        active.sort_by_key(|s| hymns::precedence_key(s));

        // User-default (lower spec) before framework exact (higher spec).
        assert_eq!(active[0].body, "USER-DEFAULT");
        assert_eq!(active[1].body, "FW-EXACT");
    }
}
