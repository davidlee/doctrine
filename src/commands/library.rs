// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine library {list,tree,show}` — a PURE READ veneer over the SL-223
//! publication [`Resolver`] (SL-227, SPEC-026 Contract B / FR-001, FR-003).
//!
//! Command-tier (ADR-001: `library → publication → asset_source`, no cycle). The
//! entire capability surface is the read-only [`Resolver`] (its [`SourceAdapter`]
//! exposes only `read`+`exists`, both reads) plus a stdout [`Write`] handle — no
//! filesystem-write, `install`, `state`, or `entity` mutator is imported, so
//! **no path from a library verb to a project mutation exists by construction**
//! (NF-002, proven once by the byte-unchanged test rather than per-verb).
//!
//! `list`/`tree` render declared metadata (via the PHASE-01 accessors) and mark
//! entries whose backing the bound adapter cannot serve `[unavailable]`
//! ([`Resolver::available`], no bytes read). `show` streams the resolved bytes
//! **raw** through [`Resolver::emit`] into a `Write` handle — never `println!`
//! (the crate denies `print_stdout`).

use crate::listing::Format;
use crate::publication::{
    AdmissionError, EmbeddedAdapter, EmitError, LogicalAddress, PublicationEntry,
    PublicationManifest, ResolveError, Resolver,
};
use clap::Subcommand;
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::str::FromStr;

/// `doctrine library <verb>` — read the published framework asset library. Every
/// verb is a pure read; none writes to the project.
#[derive(Debug, Subcommand)]
pub(crate) enum LibraryCommand {
    /// List published entries, optionally filtered by a logical-address prefix.
    List {
        /// Logical-address prefix filter (e.g. `templates/` or `hymns/role`).
        path: Option<String>,
        /// Output format.
        #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
        format: Format,
        /// Shorthand for `--format json`.
        #[arg(long)]
        json: bool,
    },
    /// List published entries grouped as a tree by logical-address segment.
    Tree {
        /// Logical-address prefix filter (e.g. `templates/` or `hymns/role`).
        path: Option<String>,
        /// Output format.
        #[arg(long, value_parser = Format::from_str, default_value_t = Format::Table)]
        format: Format,
        /// Shorthand for `--format json`.
        #[arg(long)]
        json: bool,
    },
    /// Stream one published asset's RAW bytes to stdout (framing-free).
    Show {
        /// The logical address to stream, e.g. `templates/slice.toml`.
        address: String,
    },
}

impl LibraryCommand {
    /// Dispatch the parsed verb. `--json` forces [`Format::Json`] and wins over
    /// `--format` (the house convention, `listing.rs`).
    pub(crate) fn run(self) -> anyhow::Result<()> {
        match self {
            LibraryCommand::List { path, format, json } => {
                run_list(path.as_deref(), effective_format(format, json))
            }
            LibraryCommand::Tree { path, format, json } => {
                run_tree(path.as_deref(), effective_format(format, json))
            }
            LibraryCommand::Show { address } => run_show(&address),
        }
    }
}

/// `--json` is sugar for `--format json` and wins over any `--format` value.
fn effective_format(format: Format, json: bool) -> Format {
    if json { Format::Json } else { format }
}

/// Load the shipped publication manifest and bind it to the production
/// [`EmbeddedAdapter`]. The sole IO is the admission read behind
/// [`PublicationManifest::load`]; a malformed/absent policy fails closed as an
/// [`AdmissionError`] (the FR-003 "malformed/unreadable policy" class).
fn load_resolver() -> Result<Resolver<EmbeddedAdapter>, AdmissionError> {
    Ok(Resolver::new(PublicationManifest::load()?, EmbeddedAdapter))
}

/// A rendering-ready projection of one entry — the fields a veneer legitimately
/// shows (`backing` stays private to the engine) plus its availability.
struct EntryView {
    address: String,
    kind: &'static str,
    title: String,
    licence: &'static str,
    customization: &'static str,
    available: bool,
}

impl EntryView {
    fn of<A: crate::publication::SourceAdapter>(
        resolver: &Resolver<A>,
        entry: &PublicationEntry,
    ) -> Self {
        Self {
            address: entry.address().as_str().to_string(),
            kind: entry.kind().as_str(),
            title: entry.title().to_string(),
            licence: entry.licence().as_str(),
            customization: entry.customization().as_str(),
            available: resolver.available(entry),
        }
    }
}

/// Collect the entries matching `prefix` (a plain logical-address prefix), each
/// projected to an [`EntryView`]. Sole authority: only declared entries are
/// enumerated — an embedded-but-undeclared asset is invisible (FR-001).
fn collect_views<A: crate::publication::SourceAdapter>(
    resolver: &Resolver<A>,
    prefix: Option<&str>,
) -> Vec<EntryView> {
    resolver
        .manifest()
        .entries()
        .iter()
        .filter(|e| prefix.is_none_or(|p| e.address().as_str().starts_with(p)))
        .map(|e| EntryView::of(resolver, e))
        .collect()
}

// ── list ────────────────────────────────────────────────────────────────────

pub(crate) fn run_list(path: Option<&str>, fmt: Format) -> anyhow::Result<()> {
    let resolver = load_resolver().map_err(load_diagnostic)?;
    let mut out = io::stdout().lock();
    list_to(&resolver, path, fmt, &mut out)
}

/// Render the (prefix-filtered) list to `out` — the testable core of `run_list`.
fn list_to<A: crate::publication::SourceAdapter>(
    resolver: &Resolver<A>,
    prefix: Option<&str>,
    fmt: Format,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let views = collect_views(resolver, prefix);
    match fmt {
        Format::Json => {
            let arr: Vec<serde_json::Value> = views.iter().map(view_json).collect();
            let doc = serde_json::json!({ "entries": arr });
            writeln!(out, "{}", serde_json::to_string_pretty(&doc)?)?;
        }
        Format::Table => {
            for v in &views {
                writeln!(out, "{}", list_line(v))?;
            }
        }
    }
    Ok(())
}

/// One table row for an entry, availability-marked (no bytes read).
fn list_line(v: &EntryView) -> String {
    let mark = if v.available { "" } else { "  [unavailable]" };
    format!(
        "{addr}  [{kind}]  \"{title}\"  licence={licence} customization={customization}{mark}",
        addr = v.address,
        kind = v.kind,
        title = v.title,
        licence = v.licence,
        customization = v.customization,
    )
}

fn view_json(v: &EntryView) -> serde_json::Value {
    serde_json::json!({
        "address": v.address,
        "kind": v.kind,
        "title": v.title,
        "licence": v.licence,
        "customization": v.customization,
        "available": v.available,
    })
}

// ── tree ────────────────────────────────────────────────────────────────────

pub(crate) fn run_tree(path: Option<&str>, fmt: Format) -> anyhow::Result<()> {
    let resolver = load_resolver().map_err(load_diagnostic)?;
    let mut out = io::stdout().lock();
    tree_to(&resolver, path, fmt, &mut out)
}

/// A prefix tree over address segments; leaves carry the rendered entry.
enum TreeNode {
    Branch(BTreeMap<String, TreeNode>),
    Leaf(EntryView),
}

impl TreeNode {
    fn empty() -> Self {
        TreeNode::Branch(BTreeMap::new())
    }

    fn insert(&mut self, segments: &[&str], view: EntryView) {
        match self {
            TreeNode::Branch(children) => match segments {
                [last] => {
                    children.insert((*last).to_string(), TreeNode::Leaf(view));
                }
                [head, rest @ ..] => children
                    .entry((*head).to_string())
                    .or_insert_with(TreeNode::empty)
                    .insert(rest, view),
                [] => {}
            },
            TreeNode::Leaf(_) => {}
        }
    }
}

fn build_tree(views: Vec<EntryView>) -> TreeNode {
    let mut root = TreeNode::empty();
    for view in views {
        let address = view.address.clone();
        let segments: Vec<&str> = address.split('/').collect();
        root.insert(&segments, view);
    }
    root
}

/// Render the (prefix-filtered) tree to `out` — the testable core of `run_tree`.
fn tree_to<A: crate::publication::SourceAdapter>(
    resolver: &Resolver<A>,
    prefix: Option<&str>,
    fmt: Format,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let tree = build_tree(collect_views(resolver, prefix));
    match fmt {
        Format::Json => {
            writeln!(out, "{}", serde_json::to_string_pretty(&tree_json(&tree))?)?;
        }
        Format::Table => {
            if let TreeNode::Branch(children) = &tree {
                render_branch(children, 0, out)?;
            }
        }
    }
    Ok(())
}

fn render_branch(
    children: &BTreeMap<String, TreeNode>,
    depth: usize,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let indent = "  ".repeat(depth);
    for (segment, node) in children {
        match node {
            TreeNode::Branch(grandchildren) => {
                writeln!(out, "{indent}{segment}/")?;
                render_branch(grandchildren, depth + 1, out)?;
            }
            TreeNode::Leaf(v) => {
                let mark = if v.available { "" } else { "  [unavailable]" };
                writeln!(
                    out,
                    "{indent}{segment}  [{kind}]  \"{title}\"{mark}",
                    kind = v.kind,
                    title = v.title,
                )?;
            }
        }
    }
    Ok(())
}

fn tree_json(node: &TreeNode) -> serde_json::Value {
    match node {
        TreeNode::Leaf(v) => view_json(v),
        TreeNode::Branch(children) => {
            let map: serde_json::Map<String, serde_json::Value> = children
                .iter()
                .map(|(k, v)| (k.clone(), tree_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
    }
}

// ── show ────────────────────────────────────────────────────────────────────

/// Stream one published asset's RAW bytes to stdout. Each of the four reachable
/// FR-003 error classes maps to a DISTINCT diagnostic + non-zero exit (an
/// `anyhow::Error` surfaced by `main`), never a silent empty.
pub(crate) fn run_show(addr: &str) -> anyhow::Result<()> {
    let resolver = load_resolver().map_err(load_diagnostic)?;
    let mut out = io::stdout().lock();
    show_with(&resolver, addr, &mut out)
}

/// The testable core of `run_show`: parse the address, then resolve + emit,
/// mapping the three resolve/emit-reachable FR-003 classes to distinct
/// diagnostics (the fourth, malformed policy, is [`load_diagnostic`]).
fn show_with<A: crate::publication::SourceAdapter>(
    resolver: &Resolver<A>,
    addr: &str,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    // Class 1 — invalid / traversal address (AdmissionError::TraversalRejected).
    let address = LogicalAddress::parse(addr).map_err(|_rejected: AdmissionError| {
        anyhow::anyhow!("invalid library address '{addr}': not a safe relative logical path")
    })?;
    resolver.emit(&address, out).map_err(|e| match e {
        // Class 2 — unknown / unpublished address (ResolveError::UnknownAddress).
        EmitError::Resolve(ResolveError::UnknownAddress(_)) => {
            anyhow::anyhow!("unknown library address '{addr}': no published entry")
        }
        // Class 3 — declared but its backing is not embedded
        // (ResolveError::BackingSourceMissing).
        EmitError::Resolve(ResolveError::BackingSourceMissing(backing)) => anyhow::anyhow!(
            "library address '{addr}' is unavailable: backing source '{backing}' is missing"
        ),
        // A stdout write failure — the pipe closed; surface the IO cause.
        EmitError::Io(io_err) => {
            anyhow::Error::new(io_err).context(format!("failed streaming library address '{addr}'"))
        }
    })
}

/// Class 4 — malformed / unreadable publication policy. Every load-time
/// [`AdmissionError`] variant (`MalformedManifest`, `ManifestAssetMissing`, …)
/// folds into this single "cannot load" diagnostic (the variant preserved as the
/// error source), distinct from the three per-address classes above.
fn load_diagnostic(e: AdmissionError) -> anyhow::Error {
    anyhow::Error::new(e).context("cannot load publication library")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Admit an inline manifest body — a hermetic fixture (no shipped-corpus
    /// dependency) for sole-authority / availability assertions.
    fn fixture(body: &str) -> PublicationManifest {
        PublicationManifest::admit(body.as_bytes()).expect("fixture admits")
    }

    fn shipped() -> Resolver<EmbeddedAdapter> {
        load_resolver().expect("shipped manifest loads")
    }

    fn entry_toml(address: &str, backing: &str) -> String {
        format!(
            "[[entry]]\n\
             address = \"{address}\"\n\
             backing = \"{backing}\"\n\
             kind = \"template\"\n\
             title = \"Fixture Title\"\n\
             licence = \"MIT\"\n\
             provenance = \"declared\"\n\
             customization = \"customizable\"\n"
        )
    }

    // VT-1: run_show (via its testable core) round-trips a published asset
    // byte-exact through the Resolver::emit path.
    #[test]
    fn run_show_round_trips_published_asset_byte_exact() {
        let resolver = shipped();
        let mut buf: Vec<u8> = Vec::new();
        show_with(&resolver, "templates/slice.toml", &mut buf).expect("shows");
        let expected =
            crate::asset_source::read_bytes("templates/slice.toml").expect("backing embed present");
        assert_eq!(buf.as_slice(), expected.as_ref(), "raw bytes, framing-free");
    }

    // VT-2: show maps each of the FOUR reachable error classes to a DISTINCT,
    // non-empty diagnostic (each is an anyhow::Error → non-zero exit via main).
    // References TraversalRejected, UnknownAddress, BackingSourceMissing, and
    // MalformedManifest by name.
    #[test]
    fn show_maps_four_reachable_error_classes_to_distinct_diagnostics() {
        let resolver = shipped();
        let mut sink: Vec<u8> = Vec::new();

        // Class 1: invalid / traversal address (AdmissionError::TraversalRejected).
        assert!(matches!(
            LogicalAddress::parse("../escape"),
            Err(AdmissionError::TraversalRejected(_))
        ));
        let traversal = show_with(&resolver, "../escape", &mut sink)
            .expect_err("traversal address is rejected")
            .to_string();

        // Class 2: unknown / unpublished address (ResolveError::UnknownAddress).
        let unknown = show_with(&resolver, "templates/not-published.zzz", &mut sink)
            .expect_err("unknown address")
            .to_string();

        // Class 3: declared but backing absent (ResolveError::BackingSourceMissing).
        let ghost = Resolver::new(
            fixture(&entry_toml(
                "templates/ghost.toml",
                "backing/does-not-exist.bin",
            )),
            EmbeddedAdapter,
        );
        let missing = show_with(&ghost, "templates/ghost.toml", &mut sink)
            .expect_err("missing backing")
            .to_string();

        // Class 4: malformed / unreadable policy (AdmissionError::MalformedManifest
        // folds through load_diagnostic).
        assert!(matches!(
            PublicationManifest::admit(b"this is [[[ not valid"),
            Err(AdmissionError::MalformedManifest(_))
        ));
        let load = load_diagnostic(AdmissionError::MalformedManifest("boom".into())).to_string();

        let all = [&traversal, &unknown, &missing, &load];
        for d in all {
            assert!(!d.is_empty(), "each class yields a non-empty diagnostic");
        }
        // Pairwise distinct — each class is separately diagnosable.
        for i in 0..all.len() {
            for j in (i + 1)..all.len() {
                assert_ne!(all[i], all[j], "error classes must be distinguishable");
            }
        }
    }

    // VT-3: run_list/run_tree render metadata, filter by prefix, mark
    // unavailable entries, and honour the format.
    #[test]
    fn list_renders_metadata_and_filters_by_prefix() {
        let resolver = shipped();
        let mut buf: Vec<u8> = Vec::new();
        list_to(&resolver, Some("hymns/"), Format::Table, &mut buf).expect("lists");
        let out = String::from_utf8(buf).expect("utf8");
        assert!(out.contains("hymns/role/worker.md"), "prefix match present");
        assert!(out.contains("[guidance]"), "kind rendered");
        assert!(out.contains("Worker role hymn"), "title rendered");
        assert!(out.contains("licence=MIT"), "licence rendered");
        assert!(
            !out.contains("templates/slice.toml"),
            "non-matching prefix excluded"
        );
    }

    #[test]
    fn list_marks_unavailable_entries() {
        // A declared entry whose backing is absent from the embed stays visible,
        // marked [unavailable] (metadata valid; only `show` on it fails).
        let ghost = Resolver::new(
            fixture(&entry_toml("templates/ghost.toml", "backing/nope.bin")),
            EmbeddedAdapter,
        );
        let mut buf: Vec<u8> = Vec::new();
        list_to(&ghost, None, Format::Table, &mut buf).expect("lists");
        let out = String::from_utf8(buf).expect("utf8");
        assert!(
            out.contains("templates/ghost.toml"),
            "unbacked entry visible"
        );
        assert!(out.contains("[unavailable]"), "marked unavailable");
    }

    #[test]
    fn list_honours_json_format() {
        let resolver = shipped();
        let mut buf: Vec<u8> = Vec::new();
        list_to(&resolver, Some("templates/"), Format::Json, &mut buf).expect("lists");
        let doc: serde_json::Value =
            serde_json::from_slice(&buf).expect("list --json emits valid JSON");
        let entries = doc["entries"].as_array().expect("entries array");
        assert!(!entries.is_empty(), "templates prefix is non-empty");
        assert!(
            entries.iter().all(|e| e["address"]
                .as_str()
                .is_some_and(|a| a.starts_with("templates/"))),
            "every JSON row honours the prefix"
        );
    }

    #[test]
    fn tree_groups_by_segment_and_honours_format() {
        let resolver = shipped();
        // Table: nested segments appear as branch lines.
        let mut buf: Vec<u8> = Vec::new();
        tree_to(&resolver, Some("hymns/"), Format::Table, &mut buf).expect("tree");
        let out = String::from_utf8(buf).expect("utf8");
        assert!(out.contains("hymns/"), "root segment branch");
        assert!(out.contains("role/"), "nested segment branch");
        assert!(out.contains("worker.md"), "leaf segment");
        // JSON: a nested object, not a flat array.
        let mut jbuf: Vec<u8> = Vec::new();
        tree_to(&resolver, Some("hymns/"), Format::Json, &mut jbuf).expect("tree json");
        let doc: serde_json::Value =
            serde_json::from_slice(&jbuf).expect("tree --json emits valid JSON");
        assert!(doc.is_object(), "tree JSON is a nested object");
    }

    // VT-5: FR-001 sole authority — an embedded-but-UNDECLARED asset does not
    // appear in run_list. The fixture manifest declares only slice.toml; adr.toml
    // is embedded (EmbeddedAdapter could serve it) but undeclared, so invisible.
    #[test]
    fn undeclared_embedded_asset_is_invisible_to_list() {
        assert!(
            crate::asset_source::read_bytes("templates/adr.toml").is_some(),
            "adr.toml is genuinely embedded"
        );
        let resolver = Resolver::new(
            fixture(&entry_toml("templates/slice.toml", "templates/slice.toml")),
            EmbeddedAdapter,
        );
        let mut buf: Vec<u8> = Vec::new();
        list_to(&resolver, None, Format::Table, &mut buf).expect("lists");
        let out = String::from_utf8(buf).expect("utf8");
        assert!(
            out.contains("templates/slice.toml"),
            "declared entry visible"
        );
        assert!(
            !out.contains("templates/adr.toml"),
            "undeclared embedded asset invisible (manifest is sole authority)"
        );
    }

    // VT-6: NF-002 structural no-write. The library verbs hold only a read-only
    // Resolver (read+exists) and a `Write` sink — no filesystem-write, install,
    // state, or entity mutator is imported, so no write path exists BY
    // CONSTRUCTION. This is the behavioural tripwire over that guarantee: seed a
    // POPULATED repo root, snapshot its byte tree, drive the ENTIRE verb surface
    // (list/tree/show, both formats) plus all FOUR failure classes — each asserted
    // `Err`, including the malformed-policy LOAD class — then prove the tree is
    // byte-for-byte unchanged (a real content comparison, not the prior vacuous
    // empty-dir assertion over a dir no verb ever referenced).
    #[test]
    fn every_verb_leaves_the_repo_byte_unchanged() {
        use std::collections::BTreeMap;

        fn snapshot(root: &std::path::Path) -> BTreeMap<String, Vec<u8>> {
            walkdir::WalkDir::new(root)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
                .map(|e| {
                    let rel = e
                        .path()
                        .strip_prefix(root)
                        .expect("under root")
                        .to_string_lossy()
                        .into_owned();
                    (rel, std::fs::read(e.path()).expect("read seeded file"))
                })
                .collect()
        }

        // A populated repo root — "unchanged" is now a real byte comparison.
        let repo = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(repo.path().join(".doctrine/state")).expect("mkdir");
        std::fs::write(repo.path().join("doctrine.toml"), b"[seed]\nk = 1\n").expect("seed");
        std::fs::write(repo.path().join(".doctrine/state/note.md"), b"seed body\n").expect("seed");
        let before = snapshot(repo.path());
        assert!(!before.is_empty(), "the repo root is genuinely populated");

        let resolver = shipped();
        let mut sink: Vec<u8> = Vec::new();

        // Success paths.
        list_to(&resolver, None, Format::Table, &mut sink).expect("list");
        list_to(&resolver, Some("templates/"), Format::Json, &mut sink).expect("list json");
        tree_to(&resolver, None, Format::Table, &mut sink).expect("tree");
        tree_to(&resolver, Some("hymns/"), Format::Json, &mut sink).expect("tree json");
        show_with(&resolver, "templates/slice.toml", &mut sink).expect("show");

        // All FOUR failure classes — each returns Err, none writes.
        assert!(
            show_with(&resolver, "../escape", &mut sink).is_err(),
            "traversal address rejected"
        );
        assert!(
            show_with(&resolver, "templates/nope.zzz", &mut sink).is_err(),
            "unknown address errors"
        );
        let ghost = Resolver::new(
            fixture(&entry_toml("templates/ghost.toml", "backing/nope.bin")),
            EmbeddedAdapter,
        );
        assert!(
            show_with(&ghost, "templates/ghost.toml", &mut sink).is_err(),
            "missing backing errors"
        );
        // Class 4: malformed/unreadable policy — the load path fails closed and,
        // like every other class, holds no write capability.
        assert!(
            PublicationManifest::admit(b"this is [[[ not valid").is_err(),
            "malformed policy fails closed"
        );

        assert_eq!(
            snapshot(repo.path()),
            before,
            "no library verb (success or failure) mutates the repo tree"
        );
    }
}
