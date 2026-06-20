// SPDX-License-Identifier: GPL-3.0-only
#![allow(
    clippy::same_name_method,
    reason = "rust-embed derive generates conflicting method names"
)]

//! `doctrine memory sync` — materialize the embedded global-memory corpus into
//! the gitignored `.doctrine/memory/shipped/` (SL-018 design D4/D5/D6).
//!
//! The masters ship inside the binary (`CorpusAssets`, a `RustEmbed` over the
//! repo-root `memory/`), parallel to `skills`'/`install`'s embeds. The write path
//! is the skills materialize pattern, NOT folded into install/boot: a distinct
//! sync surface (cohesion).
//!
//! Layering (ADR-001): the diff is **pure** (`plan_corpus`) — embedded assets vs
//! the on-disk shipped state in, an idempotent plan out, no clock/disk/git. The
//! **impure** shell (`gather_children`, `sync_corpus`, `run_sync`) reads the embed
//! and the disk and applies the plan. The plan NEVER references `items/`; the
//! target is always the validated `<root>/.doctrine/memory/shipped`.
//!
//! Bounded prune (Charge III / D8): a shipped dir is removed ONLY when it parses
//! as a memory bearing the INV signature (`repo=""`, anchor `none`) AND its uid is
//! absent from the embed. A foreign file or unparseable dir is left UNTOUCHED and
//! surfaced in the report — shipped/ is doctrine-owned, but the prune verb still
//! refuses to delete anything it does not recognise as its own.

use std::collections::BTreeMap;
#[cfg(test)]
use std::fmt;
use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rust_embed::RustEmbed;

use crate::git::AnchorKind;
use crate::memory::{MEMORY_SHIPPED_DIR, Memory};

/// The embedded global-memory corpus — every file under the repo-root `memory/`.
/// Committed and may be EMPTY (only `.gitkeep`) until the corpus is backfilled
/// (PHASE-05): an empty embed yields zero assets, so `sync` no-ops.
#[derive(RustEmbed)]
#[folder = "memory/"]
struct CorpusAssets;

/// One embedded master: a shipped memory keyed by uid, carrying both files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Asset {
    pub(crate) uid: String,
    pub(crate) toml: String,
    pub(crate) md: String,
}

/// A raw child of `shipped/` as read from disk — no parsing yet (classification
/// is pure, in `plan_corpus`). `toml`/`md` are the dir's two file bodies when
/// present; a non-dir child carries neither.
#[derive(Debug, Clone)]
pub(crate) struct RawChild {
    pub(crate) name: String,
    pub(crate) is_dir: bool,
    pub(crate) toml: Option<String>,
    pub(crate) md: Option<String>,
}

/// Why an on-disk child was left untouched (never pruned, always reported).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SkipReason {
    /// A dir that does not parse as an INV-signatured doctrine master.
    ForeignDir,
    /// A non-directory child (a stray file or symlink).
    StrayFile,
}

/// An on-disk child the plan refuses to touch, surfaced for the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Skipped {
    pub(crate) name: String,
    pub(crate) reason: SkipReason,
}

/// The idempotent sync plan: the diff of the embed against the on-disk shipped
/// state. `new`/`changed` are written; `prune` dirs are removed; `unchanged` and
/// `skipped` are no-ops (the latter reported as left-untouched).
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct CorpusPlan {
    pub(crate) new: Vec<Asset>,
    pub(crate) changed: Vec<Asset>,
    pub(crate) unchanged: Vec<String>,
    pub(crate) prune: Vec<String>,
    pub(crate) skipped: Vec<Skipped>,
}

impl CorpusPlan {
    /// No writes and no prunes — `sync` would touch nothing (an all-`unchanged`
    /// or all-`skipped` plan). The idempotency witness.
    pub(crate) fn is_inert(&self) -> bool {
        self.new.is_empty() && self.changed.is_empty() && self.prune.is_empty()
    }
}

/// The doctrine-owned shipped INV signature: a global memory (no repo coordinate)
/// that is unanchored (anchor `none`). The prune gate (D8 / Charge III); the
/// scope floor is a master-lint concern (PHASE-04), not a prune concern.
fn is_inv(m: &Memory) -> bool {
    m.scope.repo.is_empty() && m.anchor.kind == AnchorKind::None
}

// ---------------------------------------------------------------------------
// master-lint (SL-018 PHASE-04, Charges VII/VIII/X) — author-time validation of a
// global orientation master, layered ON TOP of the INV check. `is_inv` (the prune
// gate) stays repo+anchor only; the lint re-expresses the two INV halves as
// distinct signals and adds the type-≠-`reference` and scope-floor gates.
// ---------------------------------------------------------------------------

/// A master-lint violation. Distinct variants so each planted defect fails with
/// its own author-time signal (PHASE-04 VT-2).
///
/// Master-lint is a test/validation gate THIS phase (PHASE-04) — exercised by the
/// fixture + embedded-corpus tests, not yet on a runtime path — so it is
/// `#[cfg(test)]` (repo rule: a test-only item is `cfg(test)`, never dead code). It
/// becomes load-bearing over the real corpus in PHASE-05, still via the test gate.
#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Violation {
    /// A repo coordinate is present — a master must be global (`repo=""`).
    NonEmptyRepo(String),
    /// A born anchor is present — a master must be unanchored (`anchor_kind=none`).
    Anchored(&'static str),
    /// `memory_type = "reference"` — references are authored as `signpost`
    /// (Charge VIII / OQ-B); `MemoryType::parse` would otherwise hard-bail, so the
    /// literal is caught on the raw token for a clear signal.
    ReferenceType,
    /// The `memory.toml` fails schema validation (a bad field, or an unknown
    /// `memory_type` other than the special-cased `reference`).
    Schema(String),
    /// The scope floor (Charge X): a master needs ≥1 of paths/globs/commands —
    /// never tag-only (memory-spec §299/§333).
    ScopeFloor,
}

#[cfg(test)]
impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonEmptyRepo(r) => {
                write!(
                    f,
                    "repo coordinate must be empty for a global master (found '{r}')"
                )
            }
            Self::Anchored(k) => {
                write!(
                    f,
                    "master must be unanchored (anchor_kind=none), found '{k}'"
                )
            }
            Self::ReferenceType => write!(
                f,
                "memory_type \"reference\" is forbidden — author references as \"signpost\""
            ),
            Self::Schema(e) => write!(f, "master fails schema validation: {e}"),
            Self::ScopeFloor => write!(
                f,
                "scope floor unmet — a master needs >=1 of paths/globs/commands (never tag-only)"
            ),
        }
    }
}

/// Master-lint one master's `memory.toml` text. Asserts the global-orientation
/// class invariants: the INV signature (`repo=""` AND `anchor_kind=none`), a valid
/// `memory_type` that is NOT the `reference` literal, and the scope floor
/// (>=1 path/glob/command). Returns every violation found (not just the first).
///
/// Text-level (not `&Memory`) by necessity: a `reference` master fails
/// `Memory::parse`, so the raw token must be inspected before parsing to give the
/// author a dedicated signal rather than a generic schema bail.
#[cfg(test)]
pub(crate) fn lint_master(toml: &str) -> Result<(), Vec<Violation>> {
    let mut violations = Vec::new();
    let is_reference = raw_memory_type(toml).as_deref() == Some("reference");
    if is_reference {
        violations.push(Violation::ReferenceType);
    }
    match Memory::parse(toml) {
        Ok(m) => {
            // The two INV halves, surfaced separately (distinct signals); `is_inv`
            // itself remains the repo+anchor prune gate, untouched.
            if !m.scope.repo.is_empty() {
                violations.push(Violation::NonEmptyRepo(m.scope.repo.clone()));
            }
            if m.anchor.kind != AnchorKind::None {
                violations.push(Violation::Anchored(m.anchor.kind.as_str()));
            }
            if m.scope.paths.is_empty() && m.scope.globs.is_empty() && m.scope.commands.is_empty() {
                violations.push(Violation::ScopeFloor);
            }
        }
        // A `reference` literal already carries its own signal; any OTHER parse
        // failure (bad field, unknown type) is a schema violation.
        Err(e) if !is_reference => violations.push(Violation::Schema(e.to_string())),
        Err(_) => {}
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Read the raw `memory_type` token before `Memory::parse` rejects it — so a
/// `reference` literal yields a dedicated author signal. `None` when the TOML is
/// unparseable or the field is missing / non-string.
#[cfg(test)]
fn raw_memory_type(toml: &str) -> Option<String> {
    toml.parse::<toml::Value>()
        .ok()?
        .get("memory_type")?
        .as_str()
        .map(str::to_owned)
}

/// Pure diff: embedded `assets` vs the on-disk shipped `children`. Identical input
/// ⇒ every asset `unchanged`, no writes/prunes (idempotent). Keyed by uid via
/// `BTreeMap` for determinism. NEVER constructs an `items/` path.
pub(crate) fn plan_corpus(assets: &[Asset], children: &[RawChild]) -> CorpusPlan {
    let mut plan = CorpusPlan::default();

    // Classify on-disk children. Only INV-signatured dirs are prune/diff-eligible;
    // a stray file, or a dir that fails to parse / is not INV-class, is reported
    // and left untouched (never a prune candidate — the Charge III safety point).
    let mut inv: BTreeMap<String, (String, String)> = BTreeMap::new();
    for child in children {
        if !child.is_dir {
            plan.skipped.push(Skipped {
                name: child.name.clone(),
                reason: SkipReason::StrayFile,
            });
            continue;
        }
        let parsed = child.toml.as_deref().and_then(|t| Memory::parse(t).ok());
        match parsed {
            Some(m) if is_inv(&m) => {
                inv.insert(
                    child.name.clone(),
                    (
                        child.toml.clone().unwrap_or_default(),
                        child.md.clone().unwrap_or_default(),
                    ),
                );
            }
            _ => plan.skipped.push(Skipped {
                name: child.name.clone(),
                reason: SkipReason::ForeignDir,
            }),
        }
    }

    // Diff each embedded asset against the INV on-disk set.
    for asset in assets {
        match inv.get(&asset.uid) {
            Some((toml, md)) if *toml == asset.toml && *md == asset.md => {
                plan.unchanged.push(asset.uid.clone());
            }
            Some(_) => plan.changed.push(asset.clone()),
            None => plan.new.push(asset.clone()),
        }
    }

    // Prune: an INV dir whose uid is absent from the embed is a doctrine-owned
    // orphan — remove it. Foreign/stray children were never admitted to `inv`.
    for uid in inv.keys() {
        if !assets.iter().any(|a| &a.uid == uid) {
            plan.prune.push(uid.clone());
        }
    }
    plan
}

// ---------------------------------------------------------------------------
// Impure shell: gather the embed + the on-disk state, apply the plan.
// ---------------------------------------------------------------------------

/// Gather the embedded corpus into uid-keyed assets. Groups
/// `memory/<uid>/memory.toml` + `memory.md`; ignores root files (`.gitkeep`),
/// non-UTF-8 bytes, and incomplete pairs (a uid lacking either file is dropped).
pub(crate) fn embedded_assets() -> Vec<Asset> {
    let files = CorpusAssets::iter().filter_map(|p| {
        let path = p.as_ref().to_owned();
        CorpusAssets::get(&path).map(|f| (path, f.data.into_owned()))
    });
    gather_assets(files)
}

/// Pure asset grouping — split out so tests inject synthetic `(path, bytes)`
/// pairs without an embed. A path of form `<uid>/memory.{toml,md}` contributes;
/// anything else (a root `.gitkeep`, a nested path) is ignored.
fn gather_assets<I>(files: I) -> Vec<Asset>
where
    I: IntoIterator<Item = (String, Vec<u8>)>,
{
    let mut tomls: BTreeMap<String, String> = BTreeMap::new();
    let mut mds: BTreeMap<String, String> = BTreeMap::new();
    for (path, data) in files {
        let mut parts = path.splitn(2, '/');
        let (Some(uid), Some(file)) = (parts.next(), parts.next()) else {
            continue;
        };
        // Admit only canonical uid dirs; skip `mem.<key>` alias symlinks (RustEmbed
        // follows them, yielding each master a second time under its alias name).
        // Mirrors `memory::scan_named`, which scans real dirs only (design § 5.5).
        if !crate::memory::is_uid(uid) {
            continue;
        }
        let Ok(text) = String::from_utf8(data) else {
            continue;
        };
        match file {
            "memory.toml" => {
                tomls.insert(uid.to_owned(), text);
            }
            "memory.md" => {
                mds.insert(uid.to_owned(), text);
            }
            _ => {}
        }
    }
    tomls
        .into_iter()
        .filter_map(|(uid, toml)| {
            let md = mds.get(&uid)?.clone();
            Some(Asset { uid, toml, md })
        })
        .collect()
}

/// Read the direct children of `shipped/` — `<uid>/memory.{toml,md}` bodies for a
/// dir, nothing for a non-dir. A missing `shipped/` yields an empty listing (a
/// shipped-absent store is a clean no-op). `file_type` does not follow symlinks,
/// so a symlink reads as a non-dir (stray) and is never pruned.
fn gather_children(shipped: &Path) -> Result<Vec<RawChild>> {
    let mut out = Vec::new();
    let entries = match fs::read_dir(shipped) {
        Ok(entries) => entries,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(out),
        Err(e) => {
            return Err(e).with_context(|| format!("Failed to read {}", shipped.display()));
        }
    };
    for entry in entries {
        let entry = entry?;
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if !entry.file_type()?.is_dir() {
            out.push(RawChild {
                name,
                is_dir: false,
                toml: None,
                md: None,
            });
            continue;
        }
        let dir = entry.path();
        out.push(RawChild {
            name,
            is_dir: true,
            toml: fs::read_to_string(dir.join("memory.toml")).ok(),
            md: fs::read_to_string(dir.join("memory.md")).ok(),
        });
    }
    Ok(out)
}

/// The outcome of a sync: the plan that was computed (and, unless `dry_run`,
/// applied). The per-action lists drive the printer and the test assertions.
#[derive(Debug)]
pub(crate) struct SyncReport {
    pub(crate) plan: CorpusPlan,
}

/// Plan + (unless `dry_run`) apply the corpus into `<root>/shipped/`. `assets` is
/// a PARAMETER (not a reach into the static embed) so integration tests drive
/// synthetic masters through the full write path. The on-disk state is read here.
pub(crate) fn sync_corpus(root: &Path, assets: &[Asset], dry_run: bool) -> Result<SyncReport> {
    let shipped = root.join(MEMORY_SHIPPED_DIR);
    let children = gather_children(&shipped)?;
    let plan = plan_corpus(assets, &children);
    if !dry_run {
        apply(&shipped, &plan)?;
    }
    Ok(SyncReport { plan })
}

/// Apply a plan under `shipped/`: write New/Changed (`create_dir_all` + the two
/// files, skills `:565-571`), remove Prune dirs. Unchanged/Skipped are no-ops.
fn apply(shipped: &Path, plan: &CorpusPlan) -> Result<()> {
    for asset in plan.new.iter().chain(plan.changed.iter()) {
        let dir = shipped.join(&asset.uid);
        fs::create_dir_all(&dir).with_context(|| format!("Failed to create {}", dir.display()))?;
        let toml_path = dir.join("memory.toml");
        #[expect(
            clippy::disallowed_methods,
            reason = "derived: shipped-corpus sync into the items tree"
        )]
        fs::write(&toml_path, &asset.toml)
            .with_context(|| format!("Failed to write {}", toml_path.display()))?;
        let md_path = dir.join("memory.md");
        #[expect(
            clippy::disallowed_methods,
            reason = "derived: shipped-corpus sync into the items tree"
        )]
        fs::write(&md_path, &asset.md)
            .with_context(|| format!("Failed to write {}", md_path.display()))?;
    }
    for uid in &plan.prune {
        let dir = shipped.join(uid);
        fs::remove_dir_all(&dir).with_context(|| format!("Failed to prune {}", dir.display()))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Shell: the `memory sync` verb + `memory sync install`.
// ---------------------------------------------------------------------------

/// `doctrine memory sync [--dry-run|--yes]`. Outside a doctrine repo `root::find`
/// errors ⇒ a clean no-op (Charge XI: the M1 session hook is harmless in foreign
/// repos). `--dry-run` prints the plan and writes nothing; otherwise apply, after
/// a confirmation prompt unless `--yes`.
pub(crate) fn run_sync(path: Option<PathBuf>, dry_run: bool, yes: bool) -> Result<()> {
    let Ok(root) = crate::root::find(path, &crate::root::default_markers()) else {
        writeln!(io::stdout(), "Not in a doctrine repo — nothing to sync.")?;
        return Ok(());
    };
    let shipped = root.join(MEMORY_SHIPPED_DIR);
    let assets = embedded_assets();
    // Preview the plan (no writes), print it, then apply only on confirmation.
    // Both passes route through `sync_corpus` — one write path, shared with the
    // integration tests; the inputs are identical, so the plan is stable.
    let preview = sync_corpus(&root, &assets, true)?;
    print_plan(&preview.plan, &shipped, dry_run)?;
    if dry_run || preview.plan.is_inert() {
        return Ok(());
    }
    if !yes
        && !crate::install::prompt_confirm(&format!(
            "Apply corpus sync to {}? [y/N] ",
            shipped.display()
        ))?
    {
        writeln!(io::stdout(), "Aborted.")?;
        return Ok(());
    }
    sync_corpus(&root, &assets, false)?;
    Ok(())
}

/// Render a plan summary (counts + the skipped/untouched children). `--dry-run`
/// tags the header so the same printer serves both the preview and the apply path.
fn print_plan(plan: &CorpusPlan, shipped: &Path, dry_run: bool) -> Result<()> {
    let mut out = io::stdout();
    let tag = if dry_run { "[dry-run] " } else { "" };
    writeln!(
        out,
        "{tag}corpus sync → {}: {} new, {} changed, {} unchanged, {} prune",
        shipped.display(),
        plan.new.len(),
        plan.changed.len(),
        plan.unchanged.len(),
        plan.prune.len(),
    )?;
    for skip in &plan.skipped {
        let what = match skip.reason {
            SkipReason::ForeignDir => "foreign dir (not a doctrine master)",
            SkipReason::StrayFile => "stray file",
        };
        writeln!(out, "  left untouched: {} — {what}", skip.name)?;
    }
    Ok(())
}

/// `doctrine memory sync install` — wire the `SessionStart` hook that refreshes
/// the shipped corpus each session. A SEPARATE entry from `boot install`'s
/// (OQ-E), riding the generalized boot hook-merge seam. Claude-only: the hook is
/// a `.claude/settings.local.json` `SessionStart` command (codex has no equivalent).
pub(crate) fn run_sync_install(path: Option<PathBuf>, dry_run: bool, yes: bool) -> Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let exec = crate::boot::resolve_exec()?;
    let spec = crate::boot::HookSpec::sync(&exec);

    if !yes && !dry_run {
        let proceed = crate::install::prompt_confirm(&format!(
            "Wire the doctrine memory-sync session hook into {}? [y/N] ",
            root.display()
        ))?;
        if !proceed {
            writeln!(io::stdout(), "Aborted.")?;
            return Ok(());
        }
    }

    let mut out = io::stdout();
    let tag = if dry_run { "[dry-run] " } else { "" };
    match crate::boot::install_claude_hook(&root, &spec, dry_run)? {
        crate::boot::RefreshOutcome::Wired(cmd) => {
            writeln!(out, "  {tag}claude: wired sync hook: {cmd}")?;
        }
        crate::boot::RefreshOutcome::Refreshed(cmd) => {
            writeln!(out, "  {tag}claude: refreshed sync hook: {cmd}")?;
        }
        crate::boot::RefreshOutcome::None => {
            writeln!(out, "  {tag}claude: sync hook already current")?;
        }
        crate::boot::RefreshOutcome::PrintedFallback => {
            writeln!(
                out,
                "  claude: settings are malformed — add this hook manually:"
            )?;
            writeln!(out, "{}", crate::boot::fallback_for(&spec))?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal valid INV-signatured master (`repo=""`, anchor `none`) for a uid.
    /// Mirrors the shipped master schema (design §5.3) at the fields `Memory::parse`
    /// requires; the body is irrelevant to the diff (byte-compared opaquely).
    fn inv_master(uid: &str, body: &str) -> Asset {
        let toml = format!(
            r#"memory_uid = "{uid}"
schema_version = 1
memory_type = "fact"
status = "active"
title = "t"
summary = "s"
created = "2026-01-01"
updated = "2026-01-01"

[scope]
workspace = "global"
repo = ""

[git]
anchor_kind = ""

[review]
verification_state = "unverified"
reviewed = ""
review_by = ""

[trust]
trust_level = "standard"

[ranking]
severity = "info"
weight = 0
"#
        );
        Asset {
            uid: uid.to_owned(),
            toml,
            md: format!("# {uid}\n\n{body}\n"),
        }
    }

    /// An on-disk INV dir mirroring an asset's bytes.
    fn inv_child(asset: &Asset) -> RawChild {
        RawChild {
            name: asset.uid.clone(),
            is_dir: true,
            toml: Some(asset.toml.clone()),
            md: Some(asset.md.clone()),
        }
    }

    const UID_A: &str = "mem_00000000000000000000000000000001";
    const UID_B: &str = "mem_00000000000000000000000000000002";

    // --- T2: plan_corpus diff matrix ---

    #[test]
    fn empty_embed_and_empty_disk_is_inert() {
        let plan = plan_corpus(&[], &[]);
        assert!(plan.is_inert());
        assert_eq!(plan, CorpusPlan::default());
    }

    #[test]
    fn absent_on_disk_is_new() {
        let a = inv_master(UID_A, "alpha");
        let plan = plan_corpus(&[a.clone()], &[]);
        assert_eq!(plan.new, vec![a]);
        assert!(plan.changed.is_empty() && plan.prune.is_empty());
    }

    #[test]
    fn identical_on_disk_is_unchanged_and_inert() {
        let a = inv_master(UID_A, "alpha");
        let plan = plan_corpus(&[a.clone()], &[inv_child(&a)]);
        assert_eq!(plan.unchanged, vec![UID_A.to_owned()]);
        assert!(plan.is_inert(), "identical input must produce zero writes");
    }

    #[test]
    fn differing_body_is_changed() {
        let asset = inv_master(UID_A, "new-body");
        let stale = inv_master(UID_A, "old-body");
        let plan = plan_corpus(&[asset.clone()], &[inv_child(&stale)]);
        assert_eq!(plan.changed, vec![asset]);
        assert!(plan.unchanged.is_empty());
    }

    #[test]
    fn inv_orphan_absent_from_embed_is_pruned() {
        let orphan = inv_master(UID_B, "orphan");
        let plan = plan_corpus(&[], &[inv_child(&orphan)]);
        assert_eq!(plan.prune, vec![UID_B.to_owned()]);
    }

    #[test]
    fn plan_never_names_items_path() {
        // Structural: the plan carries only uids/names, never a path under items/.
        let a = inv_master(UID_A, "alpha");
        let plan = plan_corpus(&[a.clone()], &[]);
        for entry in plan.new.iter().chain(plan.changed.iter()) {
            assert!(!entry.uid.contains('/'), "uid must be a bare dir name");
        }
    }

    // --- T3: bounded-prune classification (survival) ---

    #[test]
    fn stray_file_survives_and_is_reported() {
        let stray = RawChild {
            name: "README".to_owned(),
            is_dir: false,
            toml: None,
            md: None,
        };
        let plan = plan_corpus(&[], &[stray]);
        assert!(plan.prune.is_empty(), "a stray file must never be pruned");
        assert_eq!(
            plan.skipped,
            vec![Skipped {
                name: "README".to_owned(),
                reason: SkipReason::StrayFile,
            }]
        );
    }

    #[test]
    fn unparseable_dir_survives_and_is_reported() {
        let junk = RawChild {
            name: "mem_garbage".to_owned(),
            is_dir: true,
            toml: Some("not valid toml {{{".to_owned()),
            md: None,
        };
        let plan = plan_corpus(&[], &[junk]);
        assert!(plan.prune.is_empty(), "an unparseable dir must survive");
        assert_eq!(plan.skipped[0].reason, SkipReason::ForeignDir);
    }

    #[test]
    fn parseable_non_inv_dir_survives() {
        // A real memory but repo-scoped (non-INV) — not doctrine's to prune.
        let mut scoped = inv_master(UID_A, "scoped");
        scoped.toml = scoped
            .toml
            .replace(r#"repo = """#, r#"repo = "github.com/x/y""#);
        // A repo-scoped memory needs an anchor to parse; give it a commit anchor.
        scoped.toml = scoped.toml.replace(
            r#"anchor_kind = """#,
            "anchor_kind = \"commit\"\ncommit = \"abc\"\ntree = \"def\"",
        );
        let child = RawChild {
            name: UID_A.to_owned(),
            is_dir: true,
            toml: Some(scoped.toml.clone()),
            md: Some(scoped.md.clone()),
        };
        let plan = plan_corpus(&[], &[child]);
        assert!(
            plan.prune.is_empty(),
            "a parseable non-INV dir must survive prune"
        );
        assert_eq!(plan.skipped[0].reason, SkipReason::ForeignDir);
    }

    // --- T2/gather: asset grouping ---

    #[test]
    fn gather_assets_pairs_toml_and_md_and_ignores_gitkeep() {
        let files = vec![
            (".gitkeep".to_owned(), Vec::new()),
            (format!("{UID_A}/memory.toml"), b"toml-a".to_vec()),
            (format!("{UID_A}/memory.md"), b"md-a".to_vec()),
        ];
        let assets = gather_assets(files);
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].uid, UID_A);
        assert_eq!(assets[0].toml, "toml-a");
        assert_eq!(assets[0].md, "md-a");
    }

    #[test]
    fn gather_assets_drops_incomplete_pair() {
        let files = vec![(format!("{UID_A}/memory.toml"), b"lonely".to_vec())];
        assert!(gather_assets(files).is_empty());
    }

    #[test]
    fn gather_assets_skips_key_symlink_aliases() {
        // The authored corpus carries `mem.<key>` alias symlinks beside each uid
        // dir; RustEmbed follows them, so the embed yields the master twice — once
        // under the uid, once under the alias name. Only the canonical uid dir is a
        // master (mirrors `memory::scan_named`, which skips key symlinks). An alias
        // path must not contribute a duplicate asset.
        let files = vec![
            (format!("{UID_A}/memory.toml"), b"toml-a".to_vec()),
            (format!("{UID_A}/memory.md"), b"md-a".to_vec()),
            (
                "mem.signpost.doctrine.overview/memory.toml".to_owned(),
                b"toml-a".to_vec(),
            ),
            (
                "mem.signpost.doctrine.overview/memory.md".to_owned(),
                b"md-a".to_vec(),
            ),
        ];
        let assets = gather_assets(files);
        assert_eq!(assets.len(), 1, "the alias must not double the master");
        assert_eq!(assets[0].uid, UID_A);
    }

    // --- T4: impure sync_corpus over a temp dir ---

    fn shipped_dir(root: &Path) -> PathBuf {
        root.join(MEMORY_SHIPPED_DIR)
    }

    #[test]
    fn sync_populates_then_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let a = inv_master(UID_A, "alpha");
        let b = inv_master(UID_B, "beta");
        let assets = vec![a.clone(), b.clone()];

        let first = sync_corpus(tmp.path(), &assets, false).unwrap();
        assert_eq!(first.plan.new.len(), 2);
        let toml = shipped_dir(tmp.path()).join(UID_A).join("memory.toml");
        assert_eq!(fs::read_to_string(&toml).unwrap(), a.toml);

        let second = sync_corpus(tmp.path(), &assets, false).unwrap();
        assert!(
            second.plan.is_inert(),
            "re-sync of identical assets must write nothing"
        );
        assert_eq!(second.plan.unchanged.len(), 2);
    }

    #[test]
    fn sync_prunes_inv_orphan_but_spares_foreign_file() {
        let tmp = tempfile::tempdir().unwrap();
        let shipped = shipped_dir(tmp.path());
        // Plant an INV orphan dir + a foreign file directly under shipped/.
        let orphan = inv_master(UID_B, "orphan");
        let odir = shipped.join(UID_B);
        fs::create_dir_all(&odir).unwrap();
        fs::write(odir.join("memory.toml"), &orphan.toml).unwrap();
        fs::write(odir.join("memory.md"), &orphan.md).unwrap();
        let foreign = shipped.join("KEEP_ME");
        fs::write(&foreign, "hands off").unwrap();

        // Sync an embed that does NOT contain the orphan.
        let a = inv_master(UID_A, "alpha");
        let report = sync_corpus(tmp.path(), &[a.clone()], false).unwrap();

        assert_eq!(report.plan.prune, vec![UID_B.to_owned()]);
        assert!(!odir.exists(), "the INV orphan must be pruned");
        assert!(foreign.exists(), "the foreign file must survive");
        assert!(shipped.join(UID_A).join("memory.toml").exists());
    }

    #[test]
    fn dry_run_writes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let a = inv_master(UID_A, "alpha");
        let report = sync_corpus(tmp.path(), &[a], true).unwrap();
        assert_eq!(report.plan.new.len(), 1);
        assert!(!shipped_dir(tmp.path()).join(UID_A).exists());
    }

    #[test]
    fn sync_with_no_shipped_root_is_clean() {
        let tmp = tempfile::tempdir().unwrap();
        let report = sync_corpus(tmp.path(), &[], false).unwrap();
        assert!(report.plan.is_inert());
    }

    // --- PHASE-04: master-lint (Charges VII/VIII/X) ---

    /// A clean global master: `repo=""`, anchor `none`, a valid type, and a path
    /// scope (the floor). Each bad-fixture test mutates exactly one axis.
    fn clean_master_toml() -> String {
        r#"memory_uid = "mem_00000000000000000000000000000009"
schema_version = 1
memory_type = "fact"
status = "active"
title = "t"
summary = "s"
created = "2026-01-01"
updated = "2026-01-01"

[scope]
workspace = "global"
repo = ""
paths = [".doctrine/spec/tech/"]

[git]
anchor_kind = ""

[review]
verification_state = "unverified"
reviewed = ""
review_by = ""

[trust]
trust_level = "standard"

[ranking]
severity = "info"
weight = 0
"#
        .to_owned()
    }

    #[test]
    fn lint_passes_a_clean_master() {
        assert!(
            lint_master(&clean_master_toml()).is_ok(),
            "a repo=\"\"/anchor-none master with a path scope must lint clean"
        );
    }

    #[test]
    fn lint_flags_a_non_empty_repo() {
        // repo set, anchor still none — parses fine (the repo⇒anchor gate is a
        // write-path concern, not a parse one), so the repo signal is isolated.
        let toml = clean_master_toml().replace(r#"repo = """#, r#"repo = "github.com/x/y""#);
        let v = lint_master(&toml).unwrap_err();
        assert!(
            v.iter()
                .any(|x| matches!(x, Violation::NonEmptyRepo(r) if r == "github.com/x/y")),
            "expected NonEmptyRepo, got {v:?}"
        );
    }

    #[test]
    fn lint_flags_a_present_anchor() {
        // anchor=commit, repo still "" — isolates the anchor signal.
        let toml = clean_master_toml().replace(
            r#"anchor_kind = """#,
            "anchor_kind = \"commit\"\ncommit = \"abc\"\ntree = \"def\"",
        );
        let v = lint_master(&toml).unwrap_err();
        assert!(
            v.iter().any(|x| matches!(x, Violation::Anchored("commit"))),
            "expected Anchored(commit), got {v:?}"
        );
    }

    #[test]
    fn lint_flags_a_reference_type_with_a_dedicated_signal() {
        // `reference` fails Memory::parse, so the raw token is caught first —
        // a ReferenceType signal, NOT a generic Schema bail.
        let toml =
            clean_master_toml().replace(r#"memory_type = "fact""#, r#"memory_type = "reference""#);
        let v = lint_master(&toml).unwrap_err();
        assert!(
            v.contains(&Violation::ReferenceType),
            "expected ReferenceType, got {v:?}"
        );
        assert!(
            !v.iter().any(|x| matches!(x, Violation::Schema(_))),
            "the reference literal must not also surface as a generic Schema bail: {v:?}"
        );
    }

    #[test]
    fn lint_flags_a_tag_only_scope() {
        // Drop the path scope, add a tag — tag-only fails the floor (Charge X).
        let toml = clean_master_toml().replace(
            r#"paths = [".doctrine/spec/tech/"]"#,
            r#"tags = ["doctrine"]"#,
        );
        let v = lint_master(&toml).unwrap_err();
        assert!(
            v.contains(&Violation::ScopeFloor),
            "expected ScopeFloor, got {v:?}"
        );
    }

    #[test]
    fn lint_flags_an_unknown_type_as_schema() {
        let toml =
            clean_master_toml().replace(r#"memory_type = "fact""#, r#"memory_type = "bogus""#);
        let v = lint_master(&toml).unwrap_err();
        assert!(
            v.iter().any(|x| matches!(x, Violation::Schema(_))),
            "an unknown (non-reference) type must surface as Schema, got {v:?}"
        );
    }

    /// PHASE-04 EX-2: every EMBEDDED master lints clean. The embed is empty this
    /// phase (only `.gitkeep` ⇒ zero assets), so this passes trivially now and
    /// becomes load-bearing over the real corpus in PHASE-05.
    #[test]
    fn every_embedded_master_lints_clean() {
        for asset in embedded_assets() {
            if let Err(violations) = lint_master(&asset.toml) {
                let detail: Vec<String> = violations.iter().map(ToString::to_string).collect();
                panic!(
                    "embedded master {} fails master-lint: {}",
                    asset.uid,
                    detail.join("; ")
                );
            }
        }
    }
}
