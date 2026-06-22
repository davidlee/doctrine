// SPDX-License-Identifier: GPL-3.0-only
mod adr;
mod backlog;
mod backlog_order;
mod boot;
mod catalog;
mod clock;
mod commands;
mod concept_map;
mod conduct;
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
mod requirement;
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
mod tomlfmt;
mod tty;
mod value;
mod verify;
mod worktree;

use std::io::Write;
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
                    let help = crate::commands::cli::render_top_level_help(color, term_width);
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
    use crate::commands::cli::{Command, EstimateAction, ValueAction};
    use crate::commands::facet::{EstimateSetArgs, ValueSetArgs};
    use crate::commands::guard::{WriteClass, write_class};
    use crate::concept_map::ConceptMapCommand;
    use crate::dispatch::DispatchCommand;
    use crate::memory::{FindRetrieveArgs, MemoryCommand, SyncCommand};
    use crate::review::ReviewCommand;
    use std::path::PathBuf;

    // Read => None, Write(label) => Some(label). The compiler's totality (no
    // wildcard in `write_class`) proves every variant is *handled*; this table
    // pins the Read/Write split + verb labels (VT-1).
    fn cls(cmd: Command) -> Option<&'static str> {
        match write_class(&cmd) {
            WriteClass::Read => None,
            // All refused classes carry a verb label; the guard refuses each.
            WriteClass::Write(v) | WriteClass::Orchestrator(v) | WriteClass::Hookmint(v) => Some(v),
            // The bespoke MarkerClear class is neither Read nor a guarded Write;
            // the dedicated `worktree_marker_is_bespoke_class` test pins it.
            WriteClass::MarkerClear => None,
        }
    }

    // The 8-field shared list flags — every `list` verb is a Read; a helper
    // tames the construction noise across the kinds.
    fn clist() -> CommonListArgs {
        CommonListArgs {
            filter: None,
            regexp: None,
            case_insensitive: false,
            status: Vec::new(),
            tag: Vec::new(),
            all: false,
            format: Format::Table,
            json: false,
            columns: None,
        }
    }

    #[test]
    fn install_is_write() {
        assert_eq!(
            cls(Command::Install {
                path: None,
                agent: Vec::new(),
                skill: Vec::new(),
                domain: Vec::new(),
                only_memory: false,
                global: false,
                dry_run: false,
                yes: false
            }),
            Some("install")
        );
    }

    #[test]
    fn skills_list_is_read() {
        assert_eq!(
            cls(Command::Skills {
                command: crate::skills::SkillsCommand::List {
                    agent: None,
                    installed: false
                }
            }),
            None
        );
    }

    #[test]
    fn slice_split() {
        use crate::slice::SliceCommand;
        let w = |c| cls(Command::Slice { command: c });
        assert_eq!(
            w(SliceCommand::New {
                title: None,
                slug: None,
                path: None
            }),
            Some("slice new")
        );
        assert_eq!(
            w(SliceCommand::Design { id: 0, path: None }),
            Some("slice design")
        );
        assert_eq!(
            w(SliceCommand::Plan { id: 0, path: None }),
            Some("slice plan")
        );
        assert_eq!(
            w(SliceCommand::Phases {
                id: 0,
                prune: false,
                path: None
            }),
            Some("slice phases")
        );
        assert_eq!(
            w(SliceCommand::Notes { id: 0, path: None }),
            Some("slice notes")
        );
        assert_eq!(
            w(SliceCommand::Phase {
                id: 0,
                phase_id: String::new(),
                status: state::PhaseStatus::Planned,
                note: None,
                path: None,
            }),
            Some("slice phase")
        );
        assert_eq!(
            w(SliceCommand::Status {
                id: 0,
                state: slice::SliceStatus::Proposed,
                note: None,
                path: None,
            }),
            Some("slice status")
        );
        assert_eq!(
            w(SliceCommand::List {
                list: clist(),
                path: None
            }),
            None
        );
        assert_eq!(
            w(SliceCommand::Show {
                reference: String::new(),
                format: Format::Table,
                json: false,
                path: None,
            }),
            None
        );
    }

    #[test]
    fn memory_split() {
        let w = |c| cls(Command::Memory { command: c });
        assert_eq!(
            w(MemoryCommand::Record {
                title: String::new(),
                memory_type: memory::MemoryType::Concept,
                key: None,
                lifespan: None,
                status: memory::Status::Active,
                summary: None,
                review_by: None,
                provenance_source: Vec::new(),
                trust: None,
                severity: None,
                tag: Vec::new(),
                path_scope: Vec::new(),
                glob: Vec::new(),
                command: Vec::new(),
                repo: None,
                global: false,
                path: None,
            }),
            Some("memory record")
        );
        assert_eq!(
            w(MemoryCommand::Verify {
                reference: String::new(),
                allow_dirty: false,
                path: None
            }),
            Some("memory verify")
        );
        assert_eq!(
            w(MemoryCommand::Show {
                reference: String::new(),
                format: Format::Table,
                json: false,
                path: None,
            }),
            None
        );
        assert_eq!(
            w(MemoryCommand::List {
                memory_type: None,
                list: clist(),
                path: None,
            }),
            None
        );
        assert_eq!(
            w(MemoryCommand::Find {
                query: None,
                args: FindRetrieveArgs {
                    path_scope: Vec::new(),
                    glob: Vec::new(),
                    command: Vec::new(),
                    tag: Vec::new(),
                    flag_query: None,
                    memory_type: None,
                    status: None,
                    lifespan: None,
                    include_draft: false,
                    format: Format::Table,
                    json: false,
                    offset: 0,
                    page: None,
                    limit: None,
                    path: None,
                    expand: None,
                },
            }),
            None
        );
        assert_eq!(
            w(MemoryCommand::Retrieve {
                args: FindRetrieveArgs {
                    path_scope: Vec::new(),
                    glob: Vec::new(),
                    command: Vec::new(),
                    tag: Vec::new(),
                    flag_query: None,
                    memory_type: None,
                    status: None,
                    lifespan: None,
                    include_draft: false,
                    format: Format::Table,
                    json: false,
                    offset: 0,
                    page: None,
                    limit: None,
                    path: None,
                    expand: None,
                },
                min_trust: None,
            }),
            None
        );
        assert_eq!(
            w(MemoryCommand::ResolveLinks {
                reference: None,
                path: None,
            }),
            None
        );
        assert_eq!(
            w(MemoryCommand::Backlinks {
                reference: String::new(),
                path: None,
            }),
            None
        );
        // Nested Option — bare `memory sync` AND `memory sync install` are both Write.
        assert_eq!(
            w(MemoryCommand::Sync {
                command: None,
                dry_run: false,
                yes: false,
                path: None,
            }),
            Some("memory sync")
        );
        assert_eq!(
            w(MemoryCommand::Sync {
                command: Some(SyncCommand::Install {
                    path: None,
                    dry_run: false,
                    yes: false,
                }),
                dry_run: false,
                yes: false,
                path: None,
            }),
            Some("memory sync install")
        );
        assert_eq!(
            w(MemoryCommand::Status {
                reference: String::new(),
                state: String::new(),
                by: None,
                path: None,
            }),
            Some("memory status")
        );
        assert_eq!(
            w(MemoryCommand::Edit {
                reference: String::new(),
                title: None,
                summary: None,
                status: None,
                lifespan: None,
                review_by: None,
                trust: None,
                severity: None,
                key: None,
                path_scope: vec![],
                glob: vec![],
                command: vec![],
                path: None,
            }),
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
        use crate::adr::AdrCommand;
        let w = |c| cls(Command::Adr { command: c });
        assert_eq!(
            w(AdrCommand::New {
                title: None,
                slug: None,
                path: None
            }),
            Some("adr new")
        );
        assert_eq!(
            w(AdrCommand::Status {
                id: 0,
                status: adr::AdrStatus::Proposed,
                path: None,
            }),
            Some("adr status")
        );
        assert_eq!(
            w(AdrCommand::List {
                list: clist(),
                path: None
            }),
            None
        );
        assert_eq!(
            w(AdrCommand::Show {
                reference: String::new(),
                format: Format::Table,
                json: false,
                path: None,
            }),
            None
        );
    }

    #[test]
    fn policy_split() {
        let w = |c| cls(Command::Policy { command: c });
        assert_eq!(
            w(crate::policy::PolicyCommand::New {
                title: None,
                slug: None,
                path: None
            }),
            Some("policy new")
        );
        assert_eq!(
            w(crate::policy::PolicyCommand::Status {
                id: 0,
                status: policy::PolicyStatus::Draft,
                path: None,
            }),
            Some("policy status")
        );
        assert_eq!(
            w(crate::policy::PolicyCommand::List {
                list: clist(),
                path: None
            }),
            None
        );
        assert_eq!(
            w(crate::policy::PolicyCommand::Show {
                reference: String::new(),
                format: Format::Table,
                json: false,
                path: None,
            }),
            None
        );
    }

    #[test]
    fn standard_split() {
        let w = |c| cls(Command::Standard { command: c });
        assert_eq!(
            w(crate::standard::StandardCommand::New {
                title: None,
                slug: None,
                path: None
            }),
            Some("standard new")
        );
        assert_eq!(
            w(crate::standard::StandardCommand::Status {
                id: 0,
                status: standard::StandardStatus::Draft,
                path: None,
            }),
            Some("standard status")
        );
        assert_eq!(
            w(crate::standard::StandardCommand::List {
                list: clist(),
                path: None
            }),
            None
        );
        assert_eq!(
            w(crate::standard::StandardCommand::Show {
                reference: String::new(),
                format: Format::Table,
                json: false,
                path: None,
            }),
            None
        );
    }

    #[test]
    fn spec_split() {
        use crate::spec::{SpecCommand, SpecReqCommand};
        let w = |c| cls(Command::Spec { command: c });
        assert_eq!(
            w(SpecCommand::New {
                subtype: spec::SpecSubtype::Product,
                title: None,
                slug: None,
                path: None,
            }),
            Some("spec new")
        );
        // Three levels deep: Spec -> Req -> Add.
        assert_eq!(
            w(SpecCommand::Req {
                command: SpecReqCommand::Add {
                    spec_ref: String::new(),
                    title: None,
                    kind: requirement::ReqKind::Functional,
                    label: None,
                    slug: None,
                    path: None,
                }
            }),
            Some("spec req add")
        );
        // sibling: Spec -> Req -> Status is also a Write.
        assert_eq!(
            w(SpecCommand::Req {
                command: SpecReqCommand::Status {
                    req_ref: String::new(),
                    to: requirement::ReqStatus::Active,
                    note: None,
                    path: None,
                }
            }),
            Some("spec req status")
        );
        assert_eq!(
            w(SpecCommand::List {
                list: clist(),
                path: None
            }),
            None
        );
        assert_eq!(
            w(SpecCommand::Show {
                spec_ref: String::new(),
                format: Format::Table,
                json: false,
                path: None,
            }),
            None
        );
        assert_eq!(
            w(SpecCommand::Validate {
                spec_ref: None,
                path: None
            }),
            None
        );
    }

    #[test]
    fn backlog_split() {
        use crate::backlog::BacklogCommand;
        let w = |c| cls(Command::Backlog { command: c });
        assert_eq!(
            w(BacklogCommand::New {
                kind: backlog::ItemKind::Issue,
                title: None,
                slug: None,
                path: None,
            }),
            Some("backlog new")
        );
        assert_eq!(
            w(BacklogCommand::Edit {
                id: String::new(),
                status: backlog::Status::Open,
                resolution: None,
                path: None,
            }),
            Some("backlog edit")
        );
        assert_eq!(
            w(BacklogCommand::List {
                kind: None,
                by: backlog::OrderBy::Sequence,
                list: clist(),
                substr: None,
                path: None,
            }),
            None
        );
        assert_eq!(
            w(BacklogCommand::Show {
                id: String::new(),
                format: Format::Table,
                json: false,
                path: None,
            }),
            None
        );
    }

    #[test]
    fn knowledge_split() {
        let w = |c| cls(Command::Knowledge { command: c });
        assert_eq!(
            w(crate::knowledge::KnowledgeCommand::New {
                kind: knowledge::RecordKind::Assumption,
                title: None,
                slug: None,
                path: None,
            }),
            Some("knowledge new")
        );
        assert_eq!(
            w(crate::knowledge::KnowledgeCommand::Status {
                id: String::new(),
                state: String::new(),
                path: None,
            }),
            Some("knowledge status")
        );
        assert_eq!(
            w(crate::knowledge::KnowledgeCommand::List {
                list: clist(),
                path: None,
            }),
            None
        );
        assert_eq!(
            w(crate::knowledge::KnowledgeCommand::Show {
                id: String::new(),
                format: Format::Table,
                json: false,
                path: None,
            }),
            None
        );
    }

    #[test]
    fn boot_split() {
        // Bare regenerate (None) AND `boot install` are both Write. `--check` is
        // a read-only sentry but the superset (§5.2) sweeps the whole verb to
        // Write — workers never run it, and over-refusing a read is the safe side.
        assert_eq!(
            cls(Command::Boot {
                command: None,
                emit: false,
                check: false,
                path: None
            }),
            Some("boot")
        );
        assert_eq!(
            cls(Command::Boot {
                command: None,
                emit: false,
                check: true,
                path: None
            }),
            Some("boot")
        );
        assert_eq!(
            cls(Command::Boot {
                command: Some(crate::boot::BootCommand::Install {
                    path: None,
                    agent: Vec::new(),
                    dry_run: false,
                    yes: false,
                }),
                emit: false,
                check: false,
                path: None,
            }),
            Some("boot install")
        );
    }

    #[test]
    fn worktree_is_read() {
        // Deliberate (§5.2): these write *fork* files, not the doctrine state the
        // guard protects, and never run in worker context.
        assert_eq!(
            cls(Command::Worktree {
                command: crate::worktree::WorktreeCommand::Provision {
                    fork: PathBuf::from("x"),
                    path: None,
                }
            }),
            None
        );
        assert_eq!(
            cls(Command::Worktree {
                command: crate::worktree::WorktreeCommand::CheckAllowlist { path: None }
            }),
            None
        );
        // SL-056 §3: `worktree status` reads the resolved mode — Read (open to
        // workers), so it survives the guard.
        assert_eq!(
            cls(Command::Worktree {
                command: crate::worktree::WorktreeCommand::Status {
                    assert: false,
                    path: None,
                }
            }),
            None
        );
    }

    // SL-056 §3/§5: `worktree marker --clear` is the bespoke MarkerClear class —
    // NOT a guarded Write (locking the marker's remover behind the marker is a
    // self-brick). The guard must not refuse it; its own fences live in the handler.
    #[test]
    fn worktree_marker_is_bespoke_class() {
        let c = Command::Worktree {
            command: crate::worktree::WorktreeCommand::Marker {
                clear: true,
                operator: false,
                stamp_subagent: false,
                path: None,
            },
        };
        assert!(
            matches!(write_class(&c), WriteClass::MarkerClear),
            "marker --clear must be the bespoke MarkerClear class"
        );
        // And therefore not seen as a guarded Write by `cls`.
        assert_eq!(cls(c), None);
    }

    // SL-056 PHASE-10: `worktree marker --stamp-subagent` is the Hookmint class —
    // refused under worker-mode via the SAME branch as Orchestrator/Write (NO
    // verb-identity carve-out), carries the "marker --stamp-subagent" verb label.
    #[test]
    fn worktree_marker_stamp_subagent_is_hookmint() {
        let c = Command::Worktree {
            command: crate::worktree::WorktreeCommand::Marker {
                clear: false,
                operator: false,
                stamp_subagent: true,
                path: None,
            },
        };
        assert!(
            matches!(
                write_class(&c),
                WriteClass::Hookmint("marker --stamp-subagent")
            ),
            "marker --stamp-subagent must be the Hookmint class"
        );
        // A guarded Write to `cls` — the worker-mode guard refuses it.
        assert_eq!(cls(c), Some("marker --stamp-subagent"));
    }

    // SL-056 PHASE-06: `worktree fork` is the FIRST Orchestrator-classed verb —
    // refused under worker-mode, carries the "fork" verb label.
    #[test]
    fn worktree_fork_is_orchestrator() {
        let c = Command::Worktree {
            command: crate::worktree::WorktreeCommand::Fork {
                base: "B".to_string(),
                branch: "wkr".to_string(),
                dir: PathBuf::from("x"),
                worker: false,
                path: None,
            },
        };
        assert!(
            matches!(write_class(&c), WriteClass::Orchestrator("fork")),
            "fork must be Orchestrator(\"fork\")"
        );
        // The guard treats it like a Write: cls surfaces the verb label.
        assert_eq!(cls(c), Some("fork"));
    }

    // SL-064 PHASE-04: `dispatch sync --prepare-review` is Orchestrator-classed —
    // refused under worker-mode, carries the "dispatch-sync" verb label (EX-1).
    #[test]
    fn dispatch_sync_is_orchestrator() {
        let c = Command::Dispatch {
            command: DispatchCommand::Sync {
                slice: 64,
                prepare_review: true,
                integrate: false,
                show_journal_trunk_oid: false,
                trunk: None,
                edge: None,
                path: None,
            },
        };
        assert!(
            matches!(write_class(&c), WriteClass::Orchestrator("dispatch-sync")),
            "dispatch sync must be Orchestrator(\"dispatch-sync\")"
        );
        assert_eq!(cls(c), Some("dispatch-sync"));
    }

    // SL-064 PHASE-05: `dispatch sync --integrate` is the same Orchestrator verb
    // class (EX-6) — the trunk-writing stage inherits the worker-mode refusal.
    #[test]
    fn dispatch_sync_integrate_is_orchestrator() {
        let c = Command::Dispatch {
            command: DispatchCommand::Sync {
                slice: 64,
                prepare_review: false,
                integrate: true,
                show_journal_trunk_oid: false,
                trunk: None,
                edge: None,
                path: None,
            },
        };
        assert!(
            matches!(write_class(&c), WriteClass::Orchestrator("dispatch-sync")),
            "dispatch sync --integrate must be Orchestrator(\"dispatch-sync\")"
        );
        assert_eq!(cls(c), Some("dispatch-sync"));
    }

    #[test]
    fn inspect_is_read() {
        // SL-046: the cross-kind relation view reads only — never mints/derives.
        assert_eq!(
            cls(Command::Inspect {
                id: "SL-046".to_string(),
                format: Format::Table,
                json: false,
                path: None,
            }),
            None
        );
    }

    #[test]
    fn validate_is_read_reseat_is_write() {
        // Corpus integrity: the scan reads (INV-3); reseat mutates the canonical
        // triple, so it is a worker-refused authored write (D2/D6).
        assert_eq!(cls(Command::Validate { path: None }), None);
        assert_eq!(
            cls(Command::Reseat {
                reference: "SL-001".to_string(),
                to: None,
                path: None,
            }),
            Some("reseat")
        );
    }

    // SL-118 PHASE-03: Estimate/Value write-class tests.

    fn estimate_cmd() -> Command {
        Command::Estimate {
            action: EstimateAction::Set(EstimateSetArgs {
                id: "SL-001".into(),
                lower: Some(1.0),
                upper: Some(3.0),
                exact: None,
                path: None,
            }),
        }
    }

    #[test]
    fn estimate_is_write() {
        assert_eq!(cls(estimate_cmd()), Some("estimate"));
    }

    #[test]
    fn value_is_write() {
        let c = Command::Value {
            action: ValueAction::Set(ValueSetArgs {
                id: "SL-001".into(),
                magnitude: 42.0,
                path: None,
            }),
        };
        assert_eq!(cls(c), Some("value"));
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
