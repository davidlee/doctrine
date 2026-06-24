// SPDX-License-Identifier: GPL-3.0-only
mod adr;
mod backlog;
mod backlog_order;
mod boot;
mod boundary;
mod catalog;
mod clock;
mod commands;
mod concept_map;
mod conduct;
mod conformance;
mod contentset;
mod corpus;
mod coverage;
mod coverage_scan;
mod coverage_store;
mod coverage_verify;
mod coverage_view;
mod dep_seq;
mod dispatch;
mod dispatch_config;
mod dtoml;
mod entity;
mod estimate;
mod facet;
mod facet_write;
mod fsutil;
mod git;
mod globmatch;
mod governance;
mod input;
mod install;
mod integrity;
mod kinds;
mod knowledge;
mod lazyspec;
mod ledger;
mod lexical;
mod lifecycle;
pub(crate) mod links;
mod listing;
mod map_server;
mod mcp_server;
mod memory;
mod meta;
mod paths;
mod plan;
mod policy;
mod priority;
mod projection;
mod rec;
mod reconcile;
mod registry;
mod relation;
mod relation_graph;
mod relation_query;
mod requirement;
mod reserve;
mod retrieve;
mod review;
mod revision;
mod rfc;
mod risk;
mod root;
mod search;
mod skills;
mod slice;
mod spec;
mod standard;
mod state;
mod status;
mod supersede;
mod tag;
#[cfg(test)]
mod test_support;
mod tomlfmt;
mod tty;
mod value;
mod verify;
mod worktree;

use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use clap::{Args, Parser};

use crate::listing::{Format, ListArgs};

/// doctrine — project tooling.
#[derive(Parser)]
#[command(name = "doctrine", about = "doctrine CLI")]
struct Cli {
    /// Control colour output
    #[arg(long, default_value = "auto", global = true)]
    color: clap::ColorChoice,

    #[command(subcommand)]
    command: crate::commands::cli::Command,
}

/// The shared, invariant list-surface flags (SL-025 §5.2) — one composable
/// `#[derive(Args)]` bundle flattened into every kind's `list` variant. It is the
/// mandatory spine of the read surface: a kind cannot quietly grow bespoke list
/// flags. Lives command-side (not in the `listing` leaf) so `clap` stays out of
/// the leaf (ADR-001 / A-3); `--format` wires `Format::from_str` via `value_parser`
/// rather than `ValueEnum`, which would drag clap into the leaf.
#[derive(Args, Debug)]
pub(crate) struct CommonListArgs {
    /// Substring filter on slug+title (case-insensitive).
    #[arg(long, short = 'f')]
    pub(crate) filter: Option<String>,

    /// Regex over canonical-id + slug + title.
    #[arg(long, short = 'r')]
    pub(crate) regexp: Option<String>,

    /// Make the regex case-insensitive.
    #[arg(long, short = 'i')]
    pub(crate) case_insensitive: bool,

    /// Status filter, multi-value (`-s draft,active`); any value reveals the
    /// hide-set.
    #[arg(long, short = 's', value_delimiter = ',')]
    pub(crate) status: Vec<String>,

    /// Tag filter, repeatable (OR logic).
    #[arg(long, short = 't')]
    pub(crate) tag: Vec<String>,

    /// Show every state, including the kind's terminal hide-set.
    #[arg(long, short = 'a')]
    pub(crate) all: bool,

    /// Output format.
    #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
    pub(crate) format: Format,

    /// Shorthand for `--format json`.
    #[arg(long)]
    pub(crate) json: bool,

    /// Select/order visible table columns, e.g. `--columns id,status,slug`.
    /// Unknown names error with the available set. No effect with `--json`
    /// (JSON rows are faithful/full — SL-037 D7).
    #[arg(long, value_delimiter = ',')]
    pub(crate) columns: Option<Vec<String>>,
}

impl CommonListArgs {
    /// Lower the parsed clap bundle onto the clap-free leaf input ([`ListArgs`]).
    /// The seam where command-layer clap types stop and the pure spine begins.
    pub(crate) fn into_list_args(self, color: bool) -> ListArgs {
        ListArgs {
            substr: self.filter,
            regexp: self.regexp,
            case_insensitive: self.case_insensitive,
            status: self.status,
            tags: self.tag,
            all: self.all,
            format: self.format,
            json: self.json,
            columns: self.columns,
            // Resolve terminal capability ONCE at the clap→leaf seam (SL-053 SL-079 D3):
            // colour is now injected by the caller via --color flag resolution;
            // term_width is still resolved here (no flag override).
            render: crate::listing::RenderOpts {
                color,
                term_width: crate::tty::stdout_terminal_width(),
            },
        }
    }
}

/// Shared `show`/`inspect` arg bundle — every kind that has both verbs flattens this
/// struct via `#[command(flatten)]`. DRY: the four fields are defined once.
#[derive(Args, Debug, Clone)]
pub(crate) struct CommonShowArgs {
    /// Canonical entity ref (e.g. ISS-007); the prefix selects the kind.
    pub(crate) id: String,

    /// Output format (table | json).
    #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
    pub(crate) format: Format,

    /// Shorthand for `--format json`.
    #[arg(long)]
    pub(crate) json: bool,

    /// Explicit project root (default: auto-detect).
    #[arg(short = 'p', long)]
    pub(crate) path: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    match Cli::try_parse() {
        Ok(cli) => {
            let color = crate::tty::resolve_color(cli.color);
            crate::commands::guard::worker_guard(&cli.command)?;
            let Cli { command, .. } = cli;
            crate::commands::cli::dispatch(command, color)
        }
        Err(e) => {
            if matches!(
                e.kind(),
                clap::error::ErrorKind::DisplayHelp
                    | clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
                    | clap::error::ErrorKind::MissingSubcommand
            ) {
                // Intercept top-level help; subcommand help falls through to clap.
                let args: Vec<String> = std::env::args().skip(1).collect();
                let has_real_subcommand = args.iter().any(|a| !a.starts_with('-') && a != "help");
                if !has_real_subcommand {
                    let color = crate::tty::stdout_color_enabled();
                    let term_width = crate::tty::stdout_terminal_width();
                    // `--boot-map` wins over `--commands` when both are passed
                    // (SL-150 §5.5 EDGE — documented precedence, not enforced
                    // mutual exclusion).
                    let help = if args.iter().any(|a| a == "--boot-map") {
                        crate::commands::cli::render_boot_map()
                    } else if args.iter().any(|a| a == "--commands") {
                        crate::commands::cli::render_commands_table(color, term_width)
                    } else {
                        crate::commands::cli::render_top_level_help(color, term_width)
                    };
                    writeln!(std::io::stdout(), "{help}")?;
                    return Ok(());
                }
            }
            e.exit()
        }
    }
}
#[cfg(test)]
mod tests {
    use {crate::Cli, clap::Parser};
    #[test]
    fn only_memory_conflicts_with_skill() {
        let r = Cli::try_parse_from([
            "doctrine",
            "skills",
            "install",
            "--only-memory",
            "--skill",
            "code-review",
        ]);
        assert!(r.is_err());
    }

    #[test]
    fn only_memory_conflicts_with_domain() {
        let r = Cli::try_parse_from([
            "doctrine",
            "skills",
            "install",
            "--only-memory",
            "--domain",
            "doctrine",
        ]);
        assert!(r.is_err());
    }

    #[test]
    fn only_memory_alone_parses() {
        // The consolidated install surface (SL-088); --only-memory moved from
        // the removed `skills install` to `install`.
        let r = Cli::try_parse_from(["doctrine", "install", "--only-memory"]);
        assert!(r.is_ok());
    }

    #[test]
    fn skills_install_is_gone() {
        let r = Cli::try_parse_from(["doctrine", "skills", "install"]);
        assert!(r.is_err());
    }
}

// ---------------------------------------------------------------------------
// The estimate / value handler tests moved to commands/facet.rs
// This placeholder keeps the line count stable until fmt.

#[cfg(test)]

mod write_class_tests {
    use super::*;
    use crate::commands::cli::Command;
    use crate::commands::guard::{WriteClass, write_class};
    use crate::concept_map::ConceptMapCommand;
    use crate::memory::MemoryCommand;
    use crate::review::ReviewCommand;

    // Read => None, Write(label) => Some(label). The compiler's totality (no
    // wildcard in `write_class`) proves every variant is *handled*; this table
    // pins the Read/Write split + verb labels (VT-1) via argv-driven assertions
    // (IMP-010 F-3) — no struct-literal command construction.
    fn cls(args: &[&str]) -> Option<&'static str> {
        match write_class(&Cli::try_parse_from(args).unwrap().command) {
            WriteClass::Read => None,
            // All refused classes carry a verb label; the guard refuses each.
            WriteClass::Write(v) | WriteClass::Orchestrator(v) | WriteClass::Hookmint(v) => Some(v),
            // The bespoke MarkerClear class is neither Read nor a guarded Write;
            // the dedicated `worktree_marker_is_bespoke_class` test pins it.
            WriteClass::MarkerClear => None,
        }
    }

    #[test]
    fn install_is_write() {
        assert_eq!(cls(&["doctrine", "install"]), Some("install"));
    }

    #[test]
    fn skills_list_is_read() {
        assert_eq!(cls(&["doctrine", "skills", "list"]), None);
    }

    #[test]
    fn slice_split() {
        assert_eq!(cls(&["doctrine", "slice", "new"]), Some("slice new"));
        assert_eq!(
            cls(&["doctrine", "slice", "design", "0"]),
            Some("slice design")
        );
        assert_eq!(cls(&["doctrine", "slice", "plan", "0"]), Some("slice plan"));
        assert_eq!(
            cls(&["doctrine", "slice", "phases", "0"]),
            Some("slice phases")
        );
        assert_eq!(
            cls(&["doctrine", "slice", "notes", "0"]),
            Some("slice notes")
        );
        assert_eq!(
            cls(&[
                "doctrine", "slice", "phase", "0", "PHASE-01", "--status", "planned"
            ]),
            Some("slice phase")
        );
        assert_eq!(
            cls(&["doctrine", "slice", "status", "0", "proposed"]),
            Some("slice status")
        );
        assert_eq!(cls(&["doctrine", "slice", "list"]), None);
        assert_eq!(cls(&["doctrine", "slice", "show", ""]), None);
    }

    #[test]
    fn memory_split() {
        assert_eq!(
            cls(&["doctrine", "memory", "record", "T", "--type", "concept"]),
            Some("memory record")
        );
        assert_eq!(
            cls(&["doctrine", "memory", "verify", ""]),
            Some("memory verify")
        );
        assert_eq!(cls(&["doctrine", "memory", "show", ""]), None);
        assert_eq!(cls(&["doctrine", "memory", "list"]), None);
        assert_eq!(cls(&["doctrine", "memory", "find"]), None);
        assert_eq!(cls(&["doctrine", "memory", "retrieve"]), None);
        assert_eq!(cls(&["doctrine", "memory", "resolve-links"]), None);
        assert_eq!(cls(&["doctrine", "memory", "backlinks", ""]), None);
        // Nested Option — bare `memory sync` AND `memory sync install` are both Write.
        assert_eq!(cls(&["doctrine", "memory", "sync"]), Some("memory sync"));
        assert_eq!(
            cls(&["doctrine", "memory", "sync", "install"]),
            Some("memory sync install")
        );
        assert_eq!(
            cls(&["doctrine", "memory", "status", "", ""]),
            Some("memory status")
        );
        assert_eq!(
            cls(&["doctrine", "memory", "edit", ""]),
            Some("memory edit")
        );
    }

    #[test]
    fn memory_record_new_flags_parse_and_reach_the_variant() {
        let cli = Cli::try_parse_from([
            "doctrine",
            "memory",
            "record",
            "T",
            "--type",
            "fact",
            "--lifespan",
            "semantic",
            "--review-by",
            "2026-08-01",
            "--provenance-source",
            "code:src/main.rs:42",
            "--trust",
            "low",
            "--severity",
            "critical",
        ])
        .unwrap();
        let Command::Memory {
            command:
                MemoryCommand::Record {
                    lifespan,
                    review_by,
                    provenance_source,
                    trust,
                    severity,
                    ..
                },
        } = cli.command
        else {
            panic!("expected memory record");
        };
        assert_eq!(lifespan, Some(memory::Lifespan::Semantic));
        assert_eq!(review_by.as_deref(), Some("2026-08-01"));
        assert_eq!(provenance_source.len(), 1);
        assert_eq!(provenance_source[0].kind, "code");
        assert_eq!(provenance_source[0].ref_, "src/main.rs:42");
        assert_eq!(trust.as_deref(), Some("low"));
        assert_eq!(severity.as_deref(), Some("critical"));
    }

    #[test]
    fn memory_record_invalid_lifespan_is_rejected() {
        let cli = Cli::try_parse_from([
            "doctrine",
            "memory",
            "record",
            "T",
            "--type",
            "fact",
            "--lifespan",
            "bogus",
        ]);
        assert!(cli.is_err());
    }

    #[test]
    fn memory_find_retrieve_lifespan_flag_parses_on_the_shared_args() {
        let find =
            Cli::try_parse_from(["doctrine", "memory", "find", "--lifespan", "semantic"]).unwrap();
        let Command::Memory {
            command: MemoryCommand::Find { args, .. },
        } = find.command
        else {
            panic!("expected memory find");
        };
        assert_eq!(args.lifespan, Some(memory::Lifespan::Semantic));

        let retrieve =
            Cli::try_parse_from(["doctrine", "memory", "retrieve", "--lifespan", "working"])
                .unwrap();
        let Command::Memory {
            command: MemoryCommand::Retrieve { args, .. },
        } = retrieve.command
        else {
            panic!("expected memory retrieve");
        };
        assert_eq!(args.lifespan, Some(memory::Lifespan::Working));
    }

    #[test]
    fn memory_find_invalid_lifespan_is_rejected() {
        let cli = Cli::try_parse_from(["doctrine", "memory", "find", "--lifespan", "garbage"]);
        assert!(cli.is_err());
    }

    #[test]
    fn adr_split() {
        assert_eq!(cls(&["doctrine", "adr", "new"]), Some("adr new"));
        assert_eq!(
            cls(&["doctrine", "adr", "status", "0", "--status", "proposed"]),
            Some("adr status")
        );
        assert_eq!(cls(&["doctrine", "adr", "list"]), None);
        assert_eq!(cls(&["doctrine", "adr", "show", ""]), None);
    }

    #[test]
    fn policy_split() {
        assert_eq!(cls(&["doctrine", "policy", "new"]), Some("policy new"));
        assert_eq!(
            cls(&["doctrine", "policy", "status", "0", "--status", "draft"]),
            Some("policy status")
        );
        assert_eq!(cls(&["doctrine", "policy", "list"]), None);
        assert_eq!(cls(&["doctrine", "policy", "show", ""]), None);
    }

    #[test]
    fn standard_split() {
        assert_eq!(cls(&["doctrine", "standard", "new"]), Some("standard new"));
        assert_eq!(
            cls(&["doctrine", "standard", "status", "0", "--status", "draft"]),
            Some("standard status")
        );
        assert_eq!(cls(&["doctrine", "standard", "list"]), None);
        assert_eq!(cls(&["doctrine", "standard", "show", ""]), None);
    }

    #[test]
    fn spec_split() {
        assert_eq!(
            cls(&["doctrine", "spec", "new", "product"]),
            Some("spec new")
        );
        assert_eq!(
            cls(&["doctrine", "spec", "req", "add", "", "--kind", "functional"]),
            Some("spec req add")
        );
        assert_eq!(
            cls(&["doctrine", "spec", "req", "status", "", "--to", "active"]),
            Some("spec req status")
        );
        assert_eq!(cls(&["doctrine", "spec", "list"]), None);
        assert_eq!(cls(&["doctrine", "spec", "show", ""]), None);
        assert_eq!(cls(&["doctrine", "spec", "validate"]), None);
    }

    #[test]
    fn backlog_split() {
        assert_eq!(
            cls(&["doctrine", "backlog", "new", "issue"]),
            Some("backlog new")
        );
        assert_eq!(
            cls(&["doctrine", "backlog", "edit", "", "--status", "open"]),
            Some("backlog edit")
        );
        assert_eq!(cls(&["doctrine", "backlog", "list"]), None);
        assert_eq!(cls(&["doctrine", "backlog", "show", ""]), None);
    }

    #[test]
    fn knowledge_split() {
        assert_eq!(
            cls(&["doctrine", "knowledge", "new", "assumption"]),
            Some("knowledge new")
        );
        assert_eq!(
            cls(&["doctrine", "knowledge", "status", "", ""]),
            Some("knowledge status")
        );
        assert_eq!(cls(&["doctrine", "knowledge", "list"]), None);
        assert_eq!(cls(&["doctrine", "knowledge", "show", ""]), None);
    }

    #[test]
    fn boot_split() {
        // Bare regenerate (None) AND `boot install` are both Write. `--check` is
        // a read-only sentry but the superset (§5.2) sweeps the whole verb to
        // Write — workers never run it, and over-refusing a read is the safe side.
        assert_eq!(cls(&["doctrine", "boot"]), Some("boot"));
        assert_eq!(cls(&["doctrine", "boot", "--check"]), Some("boot"));
        assert_eq!(cls(&["doctrine", "boot", "install"]), Some("boot install"));
    }

    #[test]
    fn worktree_is_read() {
        // Deliberate (§5.2): these write *fork* files, not the doctrine state the
        // guard protects, and never run in worker context.
        assert_eq!(cls(&["doctrine", "worktree", "provision", "x"]), None);
        assert_eq!(cls(&["doctrine", "worktree", "check-allowlist"]), None);
        // SL-056 §3: `worktree status` reads the resolved mode — Read (open to
        // workers), so it survives the guard.
        assert_eq!(cls(&["doctrine", "worktree", "status"]), None);
    }

    // SL-056 §3/§5: `worktree marker --clear` is the bespoke MarkerClear class —
    // NOT a guarded Write (locking the marker's remover behind the marker is a
    // self-brick). The guard must not refuse it; its own fences live in the handler.
    #[test]
    fn worktree_marker_is_bespoke_class() {
        let c = Cli::try_parse_from(["doctrine", "worktree", "marker", "--clear"])
            .unwrap()
            .command;
        assert!(
            matches!(write_class(&c), WriteClass::MarkerClear),
            "marker --clear must be the bespoke MarkerClear class"
        );
        // And therefore not seen as a guarded Write by `cls`.
        assert_eq!(cls(&["doctrine", "worktree", "marker", "--clear"]), None);
    }

    // SL-056 PHASE-10: `worktree marker --stamp-subagent` is the Hookmint class —
    // refused under worker-mode via the SAME branch as Orchestrator/Write (NO
    // verb-identity carve-out), carries the "marker --stamp-subagent" verb label.
    #[test]
    fn worktree_marker_stamp_subagent_is_hookmint() {
        let c = Cli::try_parse_from(["doctrine", "worktree", "marker", "--stamp-subagent"])
            .unwrap()
            .command;
        assert!(
            matches!(
                write_class(&c),
                WriteClass::Hookmint("marker --stamp-subagent")
            ),
            "marker --stamp-subagent must be the Hookmint class"
        );
        assert_eq!(
            cls(&["doctrine", "worktree", "marker", "--stamp-subagent"]),
            Some("marker --stamp-subagent")
        );
    }

    // SL-056 PHASE-06: `worktree fork` is the FIRST Orchestrator-classed verb —
    // refused under worker-mode, carries the "fork" verb label.
    #[test]
    fn worktree_fork_is_orchestrator() {
        let c = Cli::try_parse_from([
            "doctrine", "worktree", "fork", "--base", "B", "--branch", "wkr", "--dir", "x",
        ])
        .unwrap()
        .command;
        assert!(
            matches!(write_class(&c), WriteClass::Orchestrator("fork")),
            "fork must be Orchestrator(\"fork\")"
        );
        assert_eq!(
            cls(&[
                "doctrine", "worktree", "fork", "--base", "B", "--branch", "wkr", "--dir", "x"
            ]),
            Some("fork")
        );
    }

    // SL-064 PHASE-04: `dispatch sync --prepare-review` is Orchestrator-classed —
    // refused under worker-mode, carries the "dispatch-sync" verb label (EX-1).
    #[test]
    fn dispatch_sync_is_orchestrator() {
        let c = Cli::try_parse_from([
            "doctrine",
            "dispatch",
            "sync",
            "--slice",
            "64",
            "--prepare-review",
        ])
        .unwrap()
        .command;
        assert!(
            matches!(write_class(&c), WriteClass::Orchestrator("dispatch-sync")),
            "dispatch sync must be Orchestrator(\"dispatch-sync\")"
        );
        assert_eq!(
            cls(&[
                "doctrine",
                "dispatch",
                "sync",
                "--slice",
                "64",
                "--prepare-review"
            ]),
            Some("dispatch-sync")
        );
    }

    // SL-064 PHASE-05: `dispatch sync --integrate` is the same Orchestrator verb
    // class (EX-6) — the trunk-writing stage inherits the worker-mode refusal.
    #[test]
    fn dispatch_sync_integrate_is_orchestrator() {
        let c = Cli::try_parse_from([
            "doctrine",
            "dispatch",
            "sync",
            "--slice",
            "64",
            "--integrate",
        ])
        .unwrap()
        .command;
        assert!(
            matches!(write_class(&c), WriteClass::Orchestrator("dispatch-sync")),
            "dispatch sync --integrate must be Orchestrator(\"dispatch-sync\")"
        );
        assert_eq!(
            cls(&[
                "doctrine",
                "dispatch",
                "sync",
                "--slice",
                "64",
                "--integrate"
            ]),
            Some("dispatch-sync")
        );
    }

    #[test]
    fn inspect_is_read() {
        // SL-046: the cross-kind relation view reads only — never mints/derives.
        assert_eq!(cls(&["doctrine", "inspect", "SL-046"]), None);
    }

    #[test]
    fn validate_is_read_reseat_is_write() {
        // Corpus integrity: the scan reads (INV-3); reseat mutates the canonical
        // triple, so it is a worker-refused authored write (D2/D6).
        assert_eq!(cls(&["doctrine", "validate"]), None);
        assert_eq!(cls(&["doctrine", "reseat", "SL-001"]), Some("reseat"));
    }

    // SL-118 PHASE-03: Estimate/Value write-class tests.

    #[test]
    fn estimate_is_write() {
        assert_eq!(
            cls(&["doctrine", "estimate", "set", "SL-001", "1", "3"]),
            Some("estimate")
        );
    }

    #[test]
    fn value_is_write() {
        assert_eq!(
            cls(&["doctrine", "value", "set", "SL-001", "42"]),
            Some("value")
        );
    }

    // ── PHASE-01: Behaviour-preservation verification net (SL-115) ──────────────

    #[test]
    fn help_snapshot_top_level() {
        let help = <Cli as clap::CommandFactory>::command()
            .render_help()
            .to_string();
        assert!(help.contains("doctrine CLI"), "top-level about text");
        assert!(help.contains("Usage: doctrine"), "usage line");
        assert!(help.contains("Commands:"), "commands section");
        // Representative subcommands that would be visibly absent if
        // the top-level command tree is accidentally restructured.
        assert!(help.contains("  install"), "install command present");
        assert!(help.contains("  slice"), "slice command present");
        assert!(help.contains("  memory"), "memory command present");
        assert!(help.contains("  adr"), "adr command present");
        assert!(help.contains("  spec"), "spec command present");
        assert!(help.contains("  dispatch"), "dispatch command present");
        assert!(help.contains("  help"), "help command always present");
        assert!(help.contains("Options:"), "global options");
        assert!(help.contains("--color"), "color flag in help");
    }

    #[test]
    fn help_snapshot_slice_subcommand() {
        let help = <Cli as clap::CommandFactory>::command()
            .find_subcommand_mut("slice")
            .unwrap()
            .render_help()
            .to_string();
        assert!(help.contains("Create and list slices"));
        assert!(help.contains("new"));
        assert!(help.contains("design"));
        assert!(help.contains("plan"));
        assert!(help.contains("list"));
        assert!(help.contains("show"));
    }

    #[test]
    fn help_snapshot_memory_subcommand() {
        let help = <Cli as clap::CommandFactory>::command()
            .find_subcommand_mut("memory")
            .unwrap()
            .render_help()
            .to_string();
        assert!(help.contains("Record, show, and list memories"));
        assert!(help.contains("record"));
        assert!(help.contains("find"));
        assert!(help.contains("retrieve"));
        assert!(help.contains("list"));
    }

    #[test]
    fn help_snapshot_adr_subcommand() {
        let help = <Cli as clap::CommandFactory>::command()
            .find_subcommand_mut("adr")
            .unwrap()
            .render_help()
            .to_string();
        assert!(help.contains("Create and list architecture decision records"));
        assert!(help.contains("new"));
        assert!(help.contains("list"));
        assert!(help.contains("show"));
        assert!(help.contains("status"));
    }

    #[test]
    fn help_snapshot_spec_subcommand() {
        let help = <Cli as clap::CommandFactory>::command()
            .find_subcommand_mut("spec")
            .unwrap()
            .render_help()
            .to_string();
        assert!(help.contains("Create and list product / technical specifications"));
        assert!(help.contains("new"));
        assert!(help.contains("list"));
        assert!(help.contains("show"));
        assert!(help.contains("validate"));
        assert!(help.contains("req"));
    }

    #[test]
    fn commands_table_structure() {
        let out = crate::commands::cli::render_commands_table(false, Some(80));
        // Footer present
        assert!(
            out.contains("For arguments & options: doctrine <command> <verb> --help"),
            "footer"
        );
        // Representative commands present
        assert!(out.contains("install"), "install");
        assert!(out.contains("slice"), "slice");
        assert!(out.contains("review"), "review");
        // Subcommand verbs grouped under slice (not mismatched)
        assert!(out.contains("list"), "list verb");
        assert!(out.contains("new"), "new verb");
        assert!(out.contains("show"), "show verb");
        // help auto-subcommand filtered out
        assert!(!out.contains("│ help"), "help not listed as verb");
        // Leaf commands have em-dash placeholder
        assert!(out.contains("—"), "em-dash for leaf commands");
        // Three-column headers
        assert!(
            out.contains("command") && out.contains("verb") && out.contains("description"),
            "headers"
        );
    }

    // ── Parse-regression tests ──────────────────────────────────────────────────

    // (a) CommonListArgs value_delimiter
    #[test]
    fn parse_list_status_value_delimiter_equivalence() {
        use crate::slice::SliceCommand;
        let a =
            Cli::try_parse_from(["doctrine", "slice", "list", "--status", "draft,active"]).unwrap();
        let b = Cli::try_parse_from([
            "doctrine", "slice", "list", "--status", "draft", "--status", "active",
        ])
        .unwrap();
        let Command::Slice {
            command: SliceCommand::List { list: la, .. },
        } = a.command
        else {
            panic!("expected SliceCommand::List");
        };
        let Command::Slice {
            command: SliceCommand::List { list: lb, .. },
        } = b.command
        else {
            panic!("expected SliceCommand::List");
        };
        assert_eq!(la.status, lb.status);
        assert_eq!(la.status, ["draft", "active"]);
    }

    // (b) FindRetrieveArgs conflicts_with="offset"
    #[test]
    fn parse_find_retrieve_offset_conflicts_with_page() {
        let r = Cli::try_parse_from(["doctrine", "memory", "find", "--offset", "5", "--page", "2"]);
        assert!(r.is_err(), "offset + page should conflict");
    }

    // (c) FindRetrieveArgs value_parser on MemoryType, Status, Lifespan
    #[test]
    fn parse_find_memory_type_valid() {
        let r = Cli::try_parse_from(["doctrine", "memory", "find", "--type", "concept"]);
        assert!(r.is_ok());
    }

    #[test]
    fn parse_find_memory_type_invalid() {
        let r = Cli::try_parse_from(["doctrine", "memory", "find", "--type", "banana"]);
        assert!(r.is_err());
    }

    #[test]
    fn parse_find_status_valid() {
        let r = Cli::try_parse_from(["doctrine", "memory", "find", "--status", "active"]);
        assert!(r.is_ok());
    }

    #[test]
    fn parse_find_status_invalid() {
        let r = Cli::try_parse_from(["doctrine", "memory", "find", "--status", "foobar"]);
        assert!(r.is_err());
    }

    #[test]
    fn parse_find_lifespan_valid() {
        let r = Cli::try_parse_from(["doctrine", "memory", "find", "--lifespan", "semantic"]);
        assert!(r.is_ok());
    }

    #[test]
    fn parse_find_lifespan_invalid() {
        let r = Cli::try_parse_from(["doctrine", "memory", "find", "--lifespan", "quantum"]);
        assert!(r.is_err());
    }

    // (d) Retrieve value_parser=retrieve::parse_min_trust
    #[test]
    fn parse_retrieve_min_trust_valid() {
        let r = Cli::try_parse_from(["doctrine", "memory", "retrieve", "--min-trust", "high"]);
        assert!(r.is_ok());
    }

    #[test]
    fn parse_retrieve_min_trust_invalid() {
        let r = Cli::try_parse_from(["doctrine", "memory", "retrieve", "--min-trust", "banana"]);
        assert!(r.is_err());
    }

    // (e) DispatchCommand::Sync stage selection
    #[test]
    fn parse_dispatch_sync_prepare_review_parses() {
        let r = Cli::try_parse_from([
            "doctrine",
            "dispatch",
            "sync",
            "--slice",
            "99",
            "--prepare-review",
        ]);
        assert!(r.is_ok(), "sync with --prepare-review");
    }

    #[test]
    fn parse_dispatch_sync_integrate_parses_without_trunk() {
        let r = Cli::try_parse_from([
            "doctrine",
            "dispatch",
            "sync",
            "--slice",
            "99",
            "--integrate",
        ]);
        assert!(r.is_ok(), "sync with --integrate alone (trunk is optional)");
    }

    #[test]
    fn parse_dispatch_sync_missing_stage_errors() {
        let r = Cli::try_parse_from(["doctrine", "dispatch", "sync", "--slice", "99"]);
        assert!(r.is_err(), "sync without a stage selector should error");
    }

    // ── Non-SPINE_KINDS CommonListArgs consumers ────────────────────────────────

    #[test]
    fn parse_concept_map_list_common_list_args() {
        let r = Cli::try_parse_from([
            "doctrine",
            "concept-map",
            "list",
            "--filter",
            "test",
            "--tag",
            "a,b",
            "--all",
            "--json",
        ]);
        assert!(r.is_ok(), "concept-map list with CommonListArgs");
        let parsed = r.unwrap();
        let Command::ConceptMap {
            command: ConceptMapCommand::List { .. },
        } = parsed.command
        else {
            panic!("expected ConceptMapCommand::List");
        };
    }

    #[test]
    fn parse_review_list_common_list_args() {
        let r = Cli::try_parse_from([
            "doctrine",
            "review",
            "list",
            "--status",
            "open",
            "--format",
            "json",
            "--columns",
            "id,title",
        ]);
        assert!(r.is_ok(), "review list with CommonListArgs");
        let parsed = r.unwrap();
        let Command::Review {
            command: ReviewCommand::List { .. },
        } = parsed.command
        else {
            panic!("expected ReviewCommand::List");
        };
    }

    #[test]
    fn parse_rec_list_common_list_args() {
        let r = Cli::try_parse_from([
            "doctrine",
            "rec",
            "list",
            "--all",
            "--regexp",
            "test.*",
            "--case-insensitive",
        ]);
        assert!(r.is_ok(), "rec list with CommonListArgs");
        let parsed = r.unwrap();
        let Command::Rec {
            command: crate::rec::RecCommand::List { .. },
        } = parsed.command
        else {
            panic!("expected RecCommand::List");
        };
    }

    #[test]
    fn parse_concept_map_show_with_json_flag() {
        let r = Cli::try_parse_from(["doctrine", "concept-map", "show", "1", "--json"]);
        let parsed = r.unwrap();
        let Command::ConceptMap {
            command: ConceptMapCommand::Show { json, .. },
        } = parsed.command
        else {
            panic!("expected ConceptMapCommand::Show");
        };
        assert!(json, "--json flag should set json: true");
    }
}
