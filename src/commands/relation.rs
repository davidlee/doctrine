// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine link` / `doctrine unlink` — relation edge verbs (SL-048 §5.4).
//! `doctrine relation list` / `doctrine relation census` — read-only relation
//! projection views (SL-137).
//! SL-129: uses `entity::id_path`, `integrity`, `relation`, `memory`

use std::path::PathBuf;
use std::str::FromStr;

use clap::Subcommand;

use crate::catalog::diagnostic::Severity;
use crate::catalog::scan::ScanMode;
use crate::listing::Format;
use crate::relation_query::ListFilter;

/// Resolve a `link`/`unlink` source+label to (the source entity's toml path, the
/// validated label). Shared by both verbs (design §5.4): parse the source ref →
/// `(KindRef, id)`; `relation::validate_link` (the `(source, label)` legality +
/// `link`-writability gate); compute the entity's `<stem>-NNN.toml` path. Target
/// validation is link-only (a dangling target must still be `unlink`-able), so it lives
/// in `run_link`, not here.
fn resolve_link_path(
    root: &std::path::Path,
    source: &str,
    label: &str,
    role: Option<crate::relation::Role>,
) -> anyhow::Result<(PathBuf, &'static crate::relation::RelationRule)> {
    let (kref, id) = crate::integrity::parse_canonical_ref(source)?;
    // SL-149 PHASE-04c: the parsed `--role` flows into the legality gate. `validate_link`
    // yields `MissingRole` (a roleful `references` with no role), `RoleNotApplicable`
    // (a role on a label-only label), or `IllegalRole` (a role outside the source's
    // legal set); the surviving `rule` is the `(source, label, role)` row.
    let rule = crate::relation::validate_link(kref.kind, label, role)?;
    let toml_path = crate::entity::id_path(root, kref.kind, id, crate::entity::Ext::Toml);
    Ok((toml_path, rule))
}

/// Parse the optional `--role <ROLE>` flag into a [`Role`](crate::relation::Role),
/// erroring on an unknown spelling BEFORE the legality gate (so a typo surfaces as a
/// clear "unknown role" rather than a misleading `IllegalRole`). `None` ⇒ no flag given.
fn parse_role(role: Option<&str>) -> anyhow::Result<Option<crate::relation::Role>> {
    role.map(|name| {
        crate::relation::Role::from_name(name).ok_or_else(|| {
            anyhow::anyhow!(
                "`{name}` is not a known role (expected: implements, scoped_from, concerns)"
            )
        })
    })
    .transpose()
}

/// `doctrine link <SOURCE-ID> <LABEL> <TARGET>` (SL-048 §5.4) — author a tier-1
/// `[[relation]]` edge. Validates the source/label ([`resolve_link_path`]) then the
/// forward target (§5.5 — `Unvalidated` `drift` is free text; every other label's
/// target must BOTH resolve (`ensure_ref_resolves` — never write a dangler) AND pass
/// the legal-KIND assertion), then appends edit-preservingly. Idempotent (a re-link
/// reports `already linked`, file untouched).
pub(crate) fn run_link(
    path: Option<PathBuf>,
    source: &str,
    label: &str,
    role: Option<&str>,
    target: &str,
) -> anyhow::Result<()> {
    use anyhow::Context;
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let role = parse_role(role)?;

    // Memory branch — detect mem_<uid> / mem.<key> / mem_<prefix> sources
    // and route to memory.toml relations (SL-090 §PHASE-03). Memory edges are
    // label-only (no role taxonomy), so `--role` on a memory source is refused
    // up front rather than silently dropped (SL-149 PHASE-04c).
    if let Ok(mref) = crate::memory::MemoryRef::parse(source) {
        anyhow::ensure!(
            role.is_none(),
            "memory relations do not take a role; remove `--role`"
        );
        let toml_path = crate::memory::resolve_memory_toml_path(&root, &mref)?;
        // Best-effort target validation: if target looks like a canonical ref,
        // validate it resolves. Free-text and mem_* targets pass through.
        if crate::integrity::parse_canonical_ref(target).is_ok() {
            crate::integrity::ensure_ref_resolves(&root, target).with_context(|| {
                format!("target `{target}` does not resolve to an existing entity")
            })?;
        }
        let outcome = crate::memory::append_memory_relation(&toml_path, label, target)?;
        match outcome {
            crate::relation::AppendOutcome::Wrote => {
                writeln!(std::io::stdout(), "linked: {source} {label} {target}")?;
            }
            crate::relation::AppendOutcome::Noop => {
                writeln!(
                    std::io::stdout(),
                    "already linked: {source} {label} {target}"
                )?;
            }
        }
        return Ok(());
    }

    let (toml_path, rule) = resolve_link_path(&root, source, label, role)?;
    // Forward-edge validation (§5.5): free-text labels skip both gates; validated
    // labels must resolve AND be of a legal target kind.
    if !matches!(rule.target, crate::relation::TargetSpec::Unvalidated) {
        crate::integrity::ensure_ref_resolves(&root, target)?;
        let (tkref, _tid) = crate::integrity::parse_canonical_ref(target)?;
        let (skref, _sid) = crate::integrity::parse_canonical_ref(source)?;
        crate::relation::check_target_kind(rule, skref.kind, tkref.kind.prefix)?;
    }
    let outcome = crate::relation::append_edge(&toml_path, rule.label, rule.role, target)?;
    match outcome {
        crate::relation::AppendOutcome::Wrote => {
            writeln!(std::io::stdout(), "linked: {source} {label} {target}")?;
        }
        crate::relation::AppendOutcome::Noop => {
            writeln!(
                std::io::stdout(),
                "already linked: {source} {label} {target}"
            )?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// RelationCommand — the SL-137 read-only projection sub-enum
// ---------------------------------------------------------------------------

/// The sub-variants of `doctrine relation`.
#[derive(Subcommand)]
pub(crate) enum RelationCommand {
    /// List relation edges with optional filtering.
    List {
        /// Include memory-source edges (excluded by default).
        #[arg(long)]
        include_memory: bool,

        /// Filter by exact edge label (e.g. `requirements`).
        #[arg(long)]
        label: Option<String>,

        /// Filter by target entity (canonical ref, normalised).
        #[arg(long)]
        target: Option<String>,

        /// Filter by source kind prefix (e.g. `SL`, `MEM`).
        #[arg(long = "source-kind")]
        source_kind: Option<String>,

        /// Show only edges whose target is unresolved or free-text.
        #[arg(long)]
        unresolved: bool,

        /// Output format.
        #[arg(long, value_parser = Format::from_str)]
        format: Option<Format>,

        /// Shorthand for `--format json`.
        #[arg(long)]
        json: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },

    /// Group edges by label with resolution tallies.
    Census {
        /// Include memory-source edges in the census.
        #[arg(long)]
        include_memory: bool,

        /// Output format.
        #[arg(long, value_parser = Format::from_str)]
        format: Option<Format>,

        /// Shorthand for `--format json`.
        #[arg(long)]
        json: bool,

        /// Explicit project root (default: auto-detect).
        #[arg(short = 'p', long)]
        path: Option<PathBuf>,
    },
}

/// Shell for `doctrine relation list` — scan the catalog, apply the diagnostics
/// policy, project & render, print to stdout.
#[expect(
    clippy::too_many_arguments,
    reason = "clap dispatch flatten — 8 fields from subcommand"
)]
pub(crate) fn run_relation_list(
    path: Option<PathBuf>,
    include_memory: bool,
    label: Option<String>,
    target: Option<String>,
    source_kind: Option<String>,
    unresolved: bool,
    format: Option<Format>,
    json: bool,
) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let catalog = crate::catalog::hydrate::scan_catalog(&root, ScanMode::default())?;

    // Diagnostics policy (design §5.4 F1):
    emit_diagnostics(&root, &catalog.diagnostics)?;

    let resolved_format = if json {
        Format::Json
    } else {
        format.unwrap_or(Format::Table)
    };
    let opts = crate::listing::RenderOpts {
        color: crate::tty::stdout_color_enabled(),
        term_width: crate::tty::stdout_terminal_width(),
    };

    let filter = ListFilter {
        include_memory,
        label,
        target,
        source_kind,
        unresolved,
    };

    let rows = crate::relation_query::project_list(&catalog, &filter);
    let out = crate::relation_query::render_list(&rows, resolved_format, opts)?;
    write!(std::io::stdout(), "{out}")?;
    Ok(())
}

/// Shell for `doctrine relation census` — scan the catalog, apply the diagnostics
/// policy, project & render, print to stdout.
pub(crate) fn run_relation_census(
    path: Option<PathBuf>,
    include_memory: bool,
    format: Option<Format>,
    json: bool,
) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let catalog = crate::catalog::hydrate::scan_catalog(&root, ScanMode::default())?;

    // Diagnostics policy (design §5.4 F1):
    emit_diagnostics(&root, &catalog.diagnostics)?;

    let resolved_format = if json {
        Format::Json
    } else {
        format.unwrap_or(Format::Table)
    };
    let opts = crate::listing::RenderOpts {
        color: crate::tty::stdout_color_enabled(),
        term_width: crate::tty::stdout_terminal_width(),
    };

    let rows = crate::relation_query::project_census(&catalog, include_memory);
    let out = crate::relation_query::render_census(&rows, resolved_format, opts)?;
    write!(std::io::stdout(), "{out}")?;
    Ok(())
}

/// Apply the SL-137 diagnostics policy (design §5.4 F1):
/// - Print every `Severity::Error` diagnostic to stderr.
/// - Suppress per-row `Warning`/`Info` diagnostics (recoverable via `--unresolved`/census).
/// - Count edge-dropping Warnings (empty memory label/target); when any fired,
///   print ONE summary line '\"{N} edge(s) dropped — run `doctrine validate` for detail\"'
fn emit_diagnostics(
    root: &std::path::Path,
    diagnostics: &[crate::catalog::diagnostic::CatalogDiagnostic],
) -> anyhow::Result<()> {
    use std::io::Write;
    let mut dropped_edges: usize = 0;

    for diag in diagnostics {
        match diag.severity {
            Severity::Error => {
                // Strip the root prefix for a clean relative path.
                let rel = diag.file.strip_prefix(root).unwrap_or(&diag.file);
                writeln!(std::io::stderr(), "{}: {}", rel.display(), diag.message)?;
            }
            Severity::Warning => {
                // Count edge-dropping Warnings (empty memory label/target).
                if diag.message.contains("empty relation") {
                    dropped_edges = dropped_edges.wrapping_add(1);
                }
                // All other Warnings (dangling refs) are suppressed silently.
            }
            Severity::Info => {
                // Unvalidated free-text targets — silently suppressed.
            }
        }
    }

    if dropped_edges > 0 {
        writeln!(
            std::io::stderr(),
            "{dropped_edges} edge(s) dropped — run `doctrine validate` for detail"
        )?;
    }

    Ok(())
}

/// `doctrine unlink <SOURCE-ID> <LABEL> <TARGET>` (SL-048 §5.4) — remove a tier-1
/// `[[relation]]` edge. Same validation pipeline (the source/label must still be legal
/// to name the right file); idempotent (an absent edge reports `not linked`).
pub(crate) fn run_unlink(
    path: Option<PathBuf>,
    source: &str,
    label: &str,
    role: Option<&str>,
    target: &str,
) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let role = parse_role(role)?;

    // Memory branch — detect mem_<uid> / mem.<key> / mem_<prefix> sources
    // and route to memory.toml relations (SL-090 §PHASE-03). Memory edges are
    // label-only; `--role` on a memory source is refused (SL-149 PHASE-04c).
    if let Ok(mref) = crate::memory::MemoryRef::parse(source) {
        anyhow::ensure!(
            role.is_none(),
            "memory relations do not take a role; remove `--role`"
        );
        let toml_path = crate::memory::resolve_memory_toml_path(&root, &mref)?;
        // No target validation for unlink (matching existing behaviour for numbered entities).
        let outcome = crate::memory::remove_memory_relation(&toml_path, label, target)?;
        match outcome {
            crate::relation::RemoveOutcome::Removed => {
                writeln!(std::io::stdout(), "unlinked: {source} {label} {target}")?;
            }
            crate::relation::RemoveOutcome::Absent => {
                writeln!(std::io::stdout(), "not linked: {source} {label} {target}")?;
            }
        }
        return Ok(());
    }

    let (toml_path, rule) = resolve_link_path(&root, source, label, role)?;
    let outcome = crate::relation::remove_edge(&toml_path, rule.label, rule.role, target)?;
    match outcome {
        crate::relation::RemoveOutcome::Removed => {
            writeln!(std::io::stdout(), "unlinked: {source} {label} {target}")?;
        }
        crate::relation::RemoveOutcome::Absent => {
            writeln!(std::io::stdout(), "not linked: {source} {label} {target}")?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    const MEM_TEST_UID: &str = "mem_018f3a1b2c3d4e5f60718293a4b5c6d7";

    fn seed_sl_toml(root: &std::path::Path, id: u32) {
        let padded = format!("{id:03}");
        let dir = root.join(".doctrine").join("slice").join(&padded);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("slice-{padded}.toml")),
            format!(
                "id = {id}\nslug = \"s{padded}\"\ntitle = \"Test S{padded}\"\n\
                 status = \"accepted\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 [relationships]\nsupersedes = []\nsuperseded_by = []\n",
            ),
        )
        .unwrap();
    }

    fn seed_adr_toml(root: &std::path::Path, id: u32) {
        let padded = format!("{id:03}");
        let dir = root.join(".doctrine").join("adr").join(&padded);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("adr-{padded}.toml")),
            format!(
                "id = {id}\nslug = \"a{padded}\"\ntitle = \"Test A{padded}\"\n\
                 status = \"accepted\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 [relationships]\nsupersedes = []\nsuperseded_by = []\n",
            ),
        )
        .unwrap();
    }

    fn seed_memory_toml(root: &std::path::Path, uid: &str, content: &str) {
        let mem_dir = root.join(".doctrine/memory/items").join(uid);
        std::fs::create_dir_all(&mem_dir).unwrap();
        let body = if content.is_empty() {
            format!(
                "uid = \"{uid}\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n\
                 source = \"test\"\nrelevance = \"test\"\n"
            )
        } else {
            content.to_string()
        };
        std::fs::write(mem_dir.join("memory.toml"), body).unwrap();
    }

    #[test]
    fn link_supersedes_on_record_is_lifecycle_only() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".doctrine")).unwrap();
        std::fs::write(root.join("doctrine.toml"), "").unwrap();
        seed_adr_toml(root, 1);
        seed_adr_toml(root, 2);

        // Link ADR-001 supersedes ADR-002 via `doctrine link` — lifecycle follows
        // typed supersedes, not the raw relation path, so this writes a plain
        // `label = "related"` (the unvalidated drift label if needed). Actually
        // the correct label for a link is validated; `supersedes` is a lifecycle
        // internal label, so the link command should use `related`.
        // This test verifies we can link adrs via the relation system.
        run_link(
            Some(root.to_path_buf()),
            "ADR-001",
            "related",
            None,
            "ADR-002",
        )
        .unwrap();
        let content = std::fs::read_to_string(root.join(".doctrine/adr/001/adr-001.toml")).unwrap();
        assert!(content.contains("[[relation]]"));
        assert!(content.contains("label = \"related\""));
    }

    #[test]
    fn link_memory_uid_appends_relation_row() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_sl_toml(root, 1);
        seed_memory_toml(root, MEM_TEST_UID, "");

        run_link(
            Some(root.to_path_buf()),
            MEM_TEST_UID,
            "related",
            None,
            "SL-001",
        )
        .unwrap();

        let content = std::fs::read_to_string(
            root.join(format!(".doctrine/memory/items/{MEM_TEST_UID}/memory.toml")),
        )
        .unwrap();
        assert!(content.contains("[[relation]]"));
        assert!(content.contains("target = \"SL-001\""));
    }

    #[test]
    fn link_memory_uid_repeat_is_noop() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_sl_toml(root, 1);
        seed_memory_toml(root, MEM_TEST_UID, "");

        run_link(
            Some(root.to_path_buf()),
            MEM_TEST_UID,
            "related",
            None,
            "SL-001",
        )
        .unwrap();
        // Second attempt is a noop.
        run_link(
            Some(root.to_path_buf()),
            MEM_TEST_UID,
            "related",
            None,
            "SL-001",
        )
        .unwrap();
        // Still has exactly one [[relation]] row.
        let content = std::fs::read_to_string(
            root.join(format!(".doctrine/memory/items/{MEM_TEST_UID}/memory.toml")),
        )
        .unwrap();
        let count = content.matches("[[relation]]").count();
        assert_eq!(count, 1, "should still have exactly one [[relation]] row");
    }

    #[test]
    fn unlink_memory_uid_then_repeat() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_sl_toml(root, 1);
        seed_memory_toml(root, MEM_TEST_UID, "");

        run_link(
            Some(root.to_path_buf()),
            MEM_TEST_UID,
            "related",
            None,
            "SL-001",
        )
        .unwrap();
        run_unlink(
            Some(root.to_path_buf()),
            MEM_TEST_UID,
            "related",
            None,
            "SL-001",
        )
        .unwrap();
        // Second unlink reports not linked.
        let result = run_unlink(
            Some(root.to_path_buf()),
            MEM_TEST_UID,
            "related",
            None,
            "SL-001",
        );
        assert!(result.is_ok(), "second unlink should succeed as noop");
        let content = std::fs::read_to_string(
            root.join(format!(".doctrine/memory/items/{MEM_TEST_UID}/memory.toml")),
        )
        .unwrap();
        assert!(!content.contains("[[relation]]"));
    }

    #[test]
    fn link_memory_uid_bad_target_errors() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_memory_toml(root, MEM_TEST_UID, "");
        // SL-999 doesn't exist.
        let result = run_link(
            Some(root.to_path_buf()),
            MEM_TEST_UID,
            "related",
            None,
            "SL-999",
        );
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("does not resolve"), "got: {err}");
    }

    #[test]
    fn link_memory_uid_free_text_target_ok() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_memory_toml(root, MEM_TEST_UID, "");
        // Free-text target passes through for memory relations.
        run_link(
            Some(root.to_path_buf()),
            MEM_TEST_UID,
            "related",
            None,
            "https://example.com",
        )
        .unwrap();
        let content = std::fs::read_to_string(
            root.join(format!(".doctrine/memory/items/{MEM_TEST_UID}/memory.toml")),
        )
        .unwrap();
        assert!(content.contains("target = \"https://example.com\""));
    }

    #[test]
    fn link_numbered_entity_still_works() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_sl_toml(root, 1);
        seed_sl_toml(root, 2);

        run_link(
            Some(root.to_path_buf()),
            "SL-001",
            "related",
            None,
            "SL-002",
        )
        .unwrap();
        let content =
            std::fs::read_to_string(root.join(".doctrine/slice/001/slice-001.toml")).unwrap();
        assert!(content.contains("[[relation]]"));
        assert!(content.contains("target = \"SL-002\""));
    }

    #[test]
    fn link_memory_key_appends_relation_row() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_sl_toml(root, 1);
        seed_memory_toml(root, "mem.fact.cli.skinny", "");

        run_link(
            Some(root.to_path_buf()),
            "mem.fact.cli.skinny",
            "related",
            None,
            "SL-001",
        )
        .unwrap();

        let content = std::fs::read_to_string(
            root.join(".doctrine/memory/items/mem.fact.cli.skinny/memory.toml"),
        )
        .unwrap();
        assert!(content.contains("[[relation]]"), "relation row written");
        assert!(content.contains("target = \"SL-001\""), "target present");
    }

    // -- SL-149 PHASE-04c: `link`/`unlink --role` round-trip (plan VT-4) ---------
    //
    // These exercise the library `run_link`/`run_unlink` against a temp fixture root
    // directly (NOT the built binary): the dispatch worker-mode guard fences the *CLI
    // entrypoint*, not these fns, so they author freely under a temp `-p` even inside a
    // confined worktree (the binary-based `e2e_link_unlink` goldens are the ones that
    // red under the worker marker — this in-crate behaviour test does not).

    /// Seed a SPEC entity dir so `ensure_ref_resolves(SPEC-NNN)` passes (a valid
    /// `references --role implements` target). The dir is derived from the canonical ref
    /// so the test stays decoupled from which SPEC sub-kind `kind_by_prefix` resolves.
    fn seed_spec_dir(root: &std::path::Path, reference: &str) {
        let (kref, id) = crate::integrity::parse_canonical_ref(reference).unwrap();
        let dir = root.join(kref.kind.dir).join(format!("{id:03}"));
        std::fs::create_dir_all(&dir).unwrap();
    }

    /// Parse the migrated tier-1 edges of a seeded SL toml — the read-back oracle.
    fn sl_edges(root: &std::path::Path, id: u32) -> Vec<crate::relation::RelationEdge> {
        let padded = format!("{id:03}");
        let text = std::fs::read_to_string(
            root.join(format!(".doctrine/slice/{padded}/slice-{padded}.toml")),
        )
        .unwrap();
        let (kref, _id) = crate::integrity::parse_canonical_ref("SL-001").unwrap();
        crate::relation::tier1_edges(kref.kind, &text).unwrap()
    }

    #[test]
    fn link_references_role_round_trips_and_reads_back() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_sl_toml(root, 1);
        seed_spec_dir(root, "SPEC-001");

        run_link(
            Some(root.to_path_buf()),
            "SL-001",
            "references",
            Some("implements"),
            "SPEC-001",
        )
        .unwrap();

        // On-disk: a roleful `[[relation]]` row carrying `role = "implements"`.
        let content =
            std::fs::read_to_string(root.join(".doctrine/slice/001/slice-001.toml")).unwrap();
        assert!(content.contains("label = \"references\""), "label written");
        assert!(
            content.contains("role = \"implements\""),
            "role cell written"
        );
        assert!(content.contains("target = \"SPEC-001\""), "target written");

        // Read-back: outbound `references(implements) SPEC-001` + the role-derived
        // inbound "implemented by".
        let edges = sl_edges(root, 1);
        assert_eq!(
            crate::relation::targets_for_role(
                &edges,
                crate::relation::RelationLabel::References,
                crate::relation::Role::Implements,
            ),
            vec!["SPEC-001".to_string()],
        );
        assert_eq!(
            crate::relation::inbound_name(
                crate::relation::RelationLabel::References,
                Some(crate::relation::Role::Implements),
            ),
            "implemented by",
        );

        // Unlink the exact triple round-trips the file back to roleless.
        run_unlink(
            Some(root.to_path_buf()),
            "SL-001",
            "references",
            Some("implements"),
            "SPEC-001",
        )
        .unwrap();
        let after =
            std::fs::read_to_string(root.join(".doctrine/slice/001/slice-001.toml")).unwrap();
        assert!(
            !after.contains("target = \"SPEC-001\""),
            "the references(implements) edge is gone after unlink"
        );
    }

    #[test]
    fn unlink_matches_the_full_triple_only() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_sl_toml(root, 1);
        seed_sl_toml(root, 2);

        // Two references edges to the SAME target, distinct roles: concerns vs the
        // implements gate (SL-002 is a slice, legal only for `concerns`/AnyNumbered).
        run_link(
            Some(root.to_path_buf()),
            "SL-001",
            "references",
            Some("concerns"),
            "SL-002",
        )
        .unwrap();

        // Unlinking a DIFFERENT role for the same target is a no-op (Absent) — the
        // triple, not just `(label, target)`, is the identity.
        run_unlink(
            Some(root.to_path_buf()),
            "SL-001",
            "references",
            Some("scoped_from"),
            "SL-002",
        )
        .unwrap();
        let still =
            std::fs::read_to_string(root.join(".doctrine/slice/001/slice-001.toml")).unwrap();
        assert!(
            still.contains("role = \"concerns\""),
            "the concerns edge survives an unlink of a different role"
        );

        // Unlinking the exact triple removes it.
        run_unlink(
            Some(root.to_path_buf()),
            "SL-001",
            "references",
            Some("concerns"),
            "SL-002",
        )
        .unwrap();
        let after =
            std::fs::read_to_string(root.join(".doctrine/slice/001/slice-001.toml")).unwrap();
        assert!(
            !after.contains("target = \"SL-002\""),
            "the matching triple is removed"
        );
    }

    #[test]
    fn link_references_without_role_is_missing_role() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_sl_toml(root, 1);
        seed_spec_dir(root, "SPEC-001");

        let result = run_link(
            Some(root.to_path_buf()),
            "SL-001",
            "references",
            None,
            "SPEC-001",
        );
        let err = result.unwrap_err().to_string();
        assert!(err.contains("requires a role"), "MissingRole: {err}");
        // Nothing written.
        let content =
            std::fs::read_to_string(root.join(".doctrine/slice/001/slice-001.toml")).unwrap();
        assert!(!content.contains("[[relation]]"), "no edge authored");
    }

    #[test]
    fn link_label_only_with_role_is_not_applicable() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_sl_toml(root, 1);
        seed_adr_toml(root, 1);

        // `governed_by` is a label-only label; a `--role` on it is refused.
        let result = run_link(
            Some(root.to_path_buf()),
            "SL-001",
            "governed_by",
            Some("implements"),
            "ADR-001",
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("does not take a role"),
            "RoleNotApplicable: {err}"
        );
        let content =
            std::fs::read_to_string(root.join(".doctrine/slice/001/slice-001.toml")).unwrap();
        assert!(!content.contains("[[relation]]"), "no edge authored");
    }

    #[test]
    fn link_illegal_role_for_source_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_sl_toml(root, 1);
        seed_sl_toml(root, 2);

        // `concerns` is legal for SL → any numbered (SL-002 ok)…
        run_link(
            Some(root.to_path_buf()),
            "SL-001",
            "references",
            Some("concerns"),
            "SL-002",
        )
        .unwrap();

        // …but `implements` targets SPEC/PRD/REQ only — an SL target is an illegal KIND
        // for the role, refused at the forward-edge check.
        let result = run_link(
            Some(root.to_path_buf()),
            "SL-001",
            "references",
            Some("implements"),
            "SL-002",
        );
        assert!(result.is_err(), "implements → a slice target is refused");
    }

    #[test]
    fn link_unknown_role_spelling_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        seed_sl_toml(root, 1);
        seed_spec_dir(root, "SPEC-001");

        let result = run_link(
            Some(root.to_path_buf()),
            "SL-001",
            "references",
            Some("realises"),
            "SPEC-001",
        );
        let err = result.unwrap_err().to_string();
        assert!(err.contains("is not a known role"), "unknown role: {err}");
    }
}
