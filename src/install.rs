// SPDX-License-Identifier: GPL-3.0-only
#![allow(
    clippy::same_name_method,
    reason = "rust-embed derive generates conflicting method names"
)]

use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;
use rust_embed::RustEmbed;
use serde::Deserialize;

/// Embedded install assets — everything under `install/`.
#[derive(RustEmbed)]
#[folder = "install/"]
struct Assets;

/// The `install/manifest.toml` schema.
#[derive(Debug, Deserialize)]
struct Manifest {
    /// Target directory relative to the project root (e.g. `".doctrine"`).
    #[serde(default = "default_target")]
    target: String,

    #[serde(default)]
    dirs: DirsSection,

    #[serde(default)]
    gitignore: GitignoreSection,

    #[serde(default)]
    root_markers: RootMarkersSection,
}

fn default_target() -> String {
    ".doctrine".to_string()
}

#[derive(Debug, Default, Deserialize)]
struct DirsSection {
    #[serde(default)]
    create: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct GitignoreSection {
    #[serde(default)]
    entries: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RootMarkersSection {
    #[serde(default = "crate::root::default_markers")]
    markers: Vec<String>,
}

impl Default for RootMarkersSection {
    fn default() -> Self {
        Self {
            markers: crate::root::default_markers(),
        }
    }
}

/// A planned action from the dry-run.
#[derive(Debug, PartialEq, Eq)]
enum Step {
    CreateDir(PathBuf),
    Install { source: String, dest: PathBuf },
    Skip { source: String, dest: PathBuf },
    Gitignore { entry: String, dest: PathBuf },
}

/// Everything needed to run the install.
#[derive(Debug)]
struct Plan {
    project_root: PathBuf,
    target_dir: PathBuf,
    steps: Vec<Step>,
}

// ---------------------------------------------------------------------------
// CLI entry point
// ---------------------------------------------------------------------------

/// Run `doctrine install`.
///
/// `project_path` is an explicit project root override; `dry_run_only` prints
/// and exits; `yes` skips the interactive prompt.
pub(crate) fn run(
    project_path: Option<PathBuf>,
    dry_run_only: bool,
    yes: bool,
) -> anyhow::Result<()> {
    let manifest = load_manifest()?;
    let project_root =
        detect_project_root(project_path, &manifest).context("Could not find project root")?;
    let plan = build_plan(&manifest, &project_root);

    print_plan(&plan)?;

    if dry_run_only {
        return Ok(());
    }

    if !yes && !prompt_confirm("\nProceed? [y/N] ")? {
        stdout_line("Aborted.")?;
        return Ok(());
    }

    execute_plan(&plan)?;
    stdout_line("Done.")?;
    stdout_line(sync_hint())?;
    Ok(())
}

/// The post-install next-step hint (SL-018 OQ-C): point the user at the standalone
/// `memory sync` verb. `install` does NOT orchestrate sync — the verb stays
/// standalone (skills parity), so the global corpus is materialized on demand, not
/// as a hidden side effect of install.
fn sync_hint() -> &'static str {
    "Next: run `doctrine memory sync` to materialize the global memory corpus."
}

// ---------------------------------------------------------------------------
// Manifest
// ---------------------------------------------------------------------------

/// Fetch an embedded asset (relative to `install/`) as UTF-8 text.
/// Shared with `slice` for template scaffolding.
pub(crate) fn asset_text(name: &str) -> anyhow::Result<String> {
    let file = Assets::get(name).with_context(|| format!("Embedded asset '{name}' is missing"))?;
    let text = std::str::from_utf8(&file.data)
        .with_context(|| format!("Embedded asset '{name}' is not valid UTF-8"))?;
    Ok(text.to_string())
}

fn load_manifest() -> anyhow::Result<Manifest> {
    let file = Assets::get("manifest.toml")
        .context("install/manifest.toml is missing from embedded assets")?;
    let text =
        std::str::from_utf8(&file.data).context("install/manifest.toml is not valid UTF-8")?;
    let manifest: Manifest =
        toml::from_str(text).context("Failed to parse install/manifest.toml")?;
    Ok(manifest)
}

// ---------------------------------------------------------------------------
// Project root detection
// ---------------------------------------------------------------------------

/// Walk up from CWD looking for any marker file/dir (see `crate::root`).
fn detect_project_root(explicit: Option<PathBuf>, manifest: &Manifest) -> anyhow::Result<PathBuf> {
    crate::root::find(explicit, &manifest.root_markers.markers)
}

// ---------------------------------------------------------------------------
// Planning
// ---------------------------------------------------------------------------

fn build_plan(manifest: &Manifest, project_root: &Path) -> Plan {
    let target_dir = project_root.join(&manifest.target);
    let mut steps = Vec::new();

    // 1. Explicit directories from manifest.
    for dir in &manifest.dirs.create {
        let p = project_root.join(dir);
        steps.push(Step::CreateDir(p));
    }

    // 2. Embedded files (except manifest.toml).
    for filename in embedded_filenames() {
        let source = filename.clone();
        let dest = target_dir.join(&filename);
        // Ensure parent directory exists in plan.
        if let Some(parent) = dest.parent()
            && !parent.exists()
        {
            steps.push(Step::CreateDir(parent.to_path_buf()));
        }
        if dest.exists() {
            steps.push(Step::Skip { source, dest });
        } else {
            steps.push(Step::Install { source, dest });
        }
    }

    // 3. Gitignore entries.
    let gitignore_path = project_root.join(".gitignore");
    let existing = read_gitignore_lines(&gitignore_path);
    for entry in &manifest.gitignore.entries {
        if !existing.contains(entry.as_str()) {
            steps.push(Step::Gitignore {
                entry: entry.clone(),
                dest: gitignore_path.clone(),
            });
        }
    }

    Plan {
        project_root: project_root.to_path_buf(),
        target_dir,
        steps,
    }
}

/// Sorted list of embedded asset names, excluding `manifest.toml`.
fn embedded_filenames() -> Vec<String> {
    let mut names: Vec<String> = Assets::iter()
        .map(|f| f.to_string())
        .filter(|n| n != "manifest.toml")
        .collect();
    names.sort();
    names
}

fn read_gitignore_lines(path: &Path) -> BTreeSet<String> {
    let Ok(content) = fs::read_to_string(path) else {
        return BTreeSet::new();
    };
    content.lines().map(str::to_string).collect()
}

/// Append `entry` to the project `.gitignore` when absent (idempotent, additive;
/// creates the file if missing). Shared seam so each command can self-enforce its
/// own derived-tree ignore invariant rather than depend on a prior `doctrine
/// install` (SL-010 F4): `skills install` reuses this for `.doctrine/skills/*`.
pub(crate) fn ensure_gitignored(root: &Path, entry: &str) -> anyhow::Result<()> {
    let path = root.join(".gitignore");
    if read_gitignore_lines(&path).contains(entry) {
        return Ok(());
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("Failed to open {} for appending", path.display()))?;
    writeln!(file, "{entry}")
        .with_context(|| format!("Failed to append gitignore entry to {}", path.display()))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Printing
// ---------------------------------------------------------------------------

fn stdout_line(msg: &str) -> io::Result<()> {
    let mut stdout = io::stdout();
    writeln!(stdout, "{msg}")
}

fn print_plan(plan: &Plan) -> io::Result<()> {
    let mut stdout = io::stdout();
    writeln!(stdout, "Project root: {}", plan.project_root.display())?;
    writeln!(stdout, "Target:       {}", plan.target_dir.display())?;
    writeln!(stdout)?;

    for step in &plan.steps {
        match step {
            Step::CreateDir(path) => {
                let flag = if path.exists() { " (exists)" } else { "" };
                writeln!(stdout, "  create dir   {}{}", path.display(), flag)?;
            }
            Step::Install { source, dest } => {
                writeln!(stdout, "  install      {} → {}", source, dest.display())?;
            }
            Step::Skip { source, dest } => {
                writeln!(
                    stdout,
                    "  skip         {} → {} (exists)",
                    source,
                    dest.display()
                )?;
            }
            Step::Gitignore { entry, dest } => {
                writeln!(stdout, "  gitignore    + \"{entry}\"  ({})", dest.display())?;
            }
        }
    }
    Ok(())
}

pub(crate) fn prompt_confirm(prompt: &str) -> anyhow::Result<bool> {
    let mut stdout = io::stdout();
    write!(stdout, "{prompt}")?;
    stdout.flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let trimmed = line.trim();
    Ok(trimmed.eq_ignore_ascii_case("y") || trimmed.eq_ignore_ascii_case("yes"))
}

// ---------------------------------------------------------------------------
// Execution
// ---------------------------------------------------------------------------

fn execute_plan(plan: &Plan) -> anyhow::Result<()> {
    for step in &plan.steps {
        match step {
            Step::CreateDir(path) => {
                fs::create_dir_all(path)
                    .with_context(|| format!("Failed to create directory {}", path.display()))?;
            }
            Step::Install { source, dest } => {
                let file = Assets::get(source)
                    .with_context(|| format!("Embedded file '{source}' not found"))?;
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).with_context(|| {
                        format!("Failed to create parent dir for {}", dest.display())
                    })?;
                }
                fs::write(dest, &file.data)
                    .with_context(|| format!("Failed to write {}", dest.display()))?;
            }
            Step::Skip { .. } => {
                // nothing to do
            }
            Step::Gitignore { entry, .. } => {
                ensure_gitignored(&plan.project_root, entry)?;
            }
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
    use std::fs;

    // ---------------------------------------------------------------
    // detect_project_root
    // ---------------------------------------------------------------

    #[test]
    fn detects_root_via_explicit_path() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for_tests();
        let result = detect_project_root(Some(dir.path().to_path_buf()), &manifest).unwrap();
        assert_eq!(result, dir.path());
    }

    #[test]
    fn detect_root_explicit_overrides_walking() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest::default_for_tests();
        let result = detect_project_root(Some(dir.path().to_path_buf()), &manifest).unwrap();
        assert_eq!(result, dir.path());
    }

    #[test]
    fn detect_root_custom_markers_uses_explicit() {
        let dir = tempfile::tempdir().unwrap();
        let marker = dir.path().join(".myproject");
        fs::write(&marker, "").unwrap();

        let sub = dir.path().join("deep/nested");
        fs::create_dir_all(&sub).unwrap();

        let manifest = Manifest {
            root_markers: RootMarkersSection {
                markers: vec![".myproject".to_string()],
            },
            ..Manifest::default_for_tests()
        };

        // Explicit path bypasses walking.
        let result = detect_project_root(Some(sub), &manifest).unwrap();
        assert_eq!(result, dir.path().join("deep/nested"));
    }

    // ---------------------------------------------------------------
    // plan / step logic
    // ---------------------------------------------------------------

    #[test]
    fn glossary_is_shipped() {
        // ADR-005 / SL-023 PHASE-01: the glossary must be in the embed/ship set
        // so a client install receives the foundational conventions. Guards the
        // regression where it lived unembedded under doc/.
        let names = embedded_filenames();
        assert!(
            names.contains(&"glossary.md".to_string()),
            "glossary.md must be embedded (shipped); got {names:?}"
        );
        assert!(
            !asset_text("glossary.md").unwrap().trim().is_empty(),
            "glossary.md asset must be non-empty"
        );
    }

    #[test]
    fn using_doctrine_is_shipped() {
        // ADR-005 / SL-023 PHASE-02: the operator's guide (verbs, hand-editing,
        // read-via-show) must ship so a client can reach the tier-2 reference.
        let names = embedded_filenames();
        assert!(
            names.contains(&"using-doctrine.md".to_string()),
            "using-doctrine.md must be embedded (shipped); got {names:?}"
        );
        assert!(
            !asset_text("using-doctrine.md").unwrap().trim().is_empty(),
            "using-doctrine.md asset must be non-empty"
        );
    }

    #[test]
    fn plan_creates_dirs_from_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest {
            dirs: DirsSection {
                create: vec!["foo/bar".to_string(), "baz".to_string()],
            },
            ..Manifest::default_for_tests()
        };
        let plan = build_plan(&manifest, dir.path());

        let dirs: Vec<_> = plan
            .steps
            .iter()
            .filter_map(|s| match s {
                Step::CreateDir(p) => Some(p.clone()),
                _ => None,
            })
            .collect();
        assert!(dirs.contains(&dir.path().join("foo/bar")));
        assert!(dirs.contains(&dir.path().join("baz")));
    }

    #[test]
    fn plan_skips_existing_files() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join(".doctrine");
        let nested = target.join("x");
        fs::create_dir_all(&nested).unwrap();
        // Pre-create the target file.
        let existing = nested.join("about.md");
        fs::write(&existing, "old content").unwrap();

        let manifest = Manifest::default_for_tests();
        let plan = build_plan(&manifest, dir.path());

        let has_skip = plan
            .steps
            .iter()
            .any(|s| matches!(s, Step::Skip { dest, .. } if dest == &existing));
        assert!(has_skip, "Expected a Skip step for the pre-existing file");
    }

    #[test]
    fn plan_includes_gitignore_entries() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest {
            gitignore: GitignoreSection {
                entries: vec!["ignored-dir/".to_string()],
            },
            ..Manifest::default_for_tests()
        };
        let plan = build_plan(&manifest, dir.path());

        let gi: Vec<_> = plan
            .steps
            .iter()
            .filter_map(|s| match s {
                Step::Gitignore { entry, .. } => Some(entry.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(gi, vec!["ignored-dir/".to_string()]);
    }

    #[test]
    fn gitignore_skips_duplicate_entries() {
        let dir = tempfile::tempdir().unwrap();
        let gi = dir.path().join(".gitignore");
        fs::write(&gi, "skip-me\n").unwrap();

        let manifest = Manifest {
            gitignore: GitignoreSection {
                entries: vec!["skip-me".to_string(), "new-one".to_string()],
            },
            ..Manifest::default_for_tests()
        };
        let plan = build_plan(&manifest, dir.path());

        let entries: Vec<_> = plan
            .steps
            .iter()
            .filter_map(|s| match s {
                Step::Gitignore { entry, .. } => Some(entry.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(entries, vec!["new-one"]);
    }

    // ---------------------------------------------------------------
    // embedded manifest
    // ---------------------------------------------------------------

    #[test]
    fn embedded_manifest_gitignores_the_runtime_state_surface() {
        let manifest = load_manifest().unwrap();
        for entry in [
            ".doctrine/state/",
            ".doctrine/slice/*/phases",
            ".doctrine/slice/*/handover.md",
        ] {
            assert!(
                manifest.gitignore.entries.iter().any(|e| e == entry),
                "manifest must gitignore {entry}"
            );
        }
    }

    #[test]
    fn embedded_manifest_creates_memory_items_and_ignores_derived_subtrees() {
        let manifest = load_manifest().unwrap();

        // items/ is the only memory subtree the installer materialises — it
        // holds committed, authored memory entities.
        assert!(
            manifest
                .dirs
                .create
                .iter()
                .any(|d| d == ".doctrine/memory/items"),
            "manifest must create the memory items tree"
        );
        // The derived subtrees are gitignored but NOT created (future slices own
        // their on-demand creation). `shipped/` is the SL-018 binary-materialized
        // corpus — derived/gitignored, never committed in a client repo.
        for derived in [
            ".doctrine/memory/index/*",
            ".doctrine/memory/embeddings/*",
            ".doctrine/memory/state/*",
            ".doctrine/memory/shipped/",
        ] {
            assert!(
                manifest.gitignore.entries.iter().any(|e| e == derived),
                "manifest must gitignore {derived}"
            );
            assert!(
                !manifest.dirs.create.iter().any(|d| d == derived),
                "manifest must not create the derived subtree {derived}"
            );
        }
        // A blanket ignore would swallow the committed items/ tree — must not exist.
        assert!(
            !manifest
                .gitignore
                .entries
                .iter()
                .any(|e| e == ".doctrine/memory/*" || e == ".doctrine/memory/"),
            "manifest must not blanket-ignore the memory tree"
        );
    }

    /// SL-030 PHASE-03: the policy tree is an authored governance kind, so the
    /// manifest must create it (parity with adr / memory-items) and must NOT
    /// ignore it — install surface 1 of 3. The `.gitignore` negation (surface 2)
    /// and the git-add round-trip (surface 3) are covered by the e2e commit test.
    #[test]
    fn embedded_manifest_creates_the_policy_tree() {
        let manifest = load_manifest().unwrap();
        assert!(
            manifest.dirs.create.iter().any(|d| d == ".doctrine/policy"),
            "manifest must create the authored policy tree"
        );
        assert!(
            !manifest
                .gitignore
                .entries
                .iter()
                .any(|e| e.starts_with(".doctrine/policy")),
            "the authored policy tree must not be gitignored by the manifest"
        );
    }

    #[test]
    fn embedded_manifest_creates_and_ignores_the_skills_derived_tree() {
        let manifest = load_manifest().unwrap();
        // The canonical skills tree is created-and-ignored: the installer
        // materialises the dir, but its contents are derived (regenerable from
        // the embed) and must not be committed (SL-010 D2). Without the ignore
        // entry a consumer would commit the derived tree — the blanket
        // `.doctrine/*` only masks it in this repo, the manifest writes additive
        // entries, not the blanket.
        assert!(
            manifest.dirs.create.iter().any(|d| d == ".doctrine/skills"),
            "manifest must create the canonical skills dir"
        );
        assert!(
            manifest
                .gitignore
                .entries
                .iter()
                .any(|e| e == ".doctrine/skills/*"),
            "manifest must gitignore the derived skills tree"
        );
    }

    #[test]
    fn ensure_gitignored_appends_once_and_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let gi = dir.path().join(".gitignore");

        // Creates the file when missing.
        ensure_gitignored(dir.path(), ".doctrine/skills/*").unwrap();
        assert!(gi.is_file());
        let after_first = fs::read_to_string(&gi).unwrap();
        assert!(after_first.contains(".doctrine/skills/*"));

        // Second call is a no-op — no duplicate line.
        ensure_gitignored(dir.path(), ".doctrine/skills/*").unwrap();
        let after_second = fs::read_to_string(&gi).unwrap();
        assert_eq!(after_first, after_second);
        assert_eq!(
            after_second.matches(".doctrine/skills/*").count(),
            1,
            "entry must appear exactly once"
        );
    }

    #[test]
    fn ensure_gitignored_preserves_existing_entries() {
        let dir = tempfile::tempdir().unwrap();
        let gi = dir.path().join(".gitignore");
        fs::write(&gi, "/pre-existing\n").unwrap();

        ensure_gitignored(dir.path(), ".doctrine/skills/*").unwrap();
        let content = fs::read_to_string(&gi).unwrap();
        assert!(content.contains("/pre-existing"));
        assert!(content.contains(".doctrine/skills/*"));
    }

    // ---------------------------------------------------------------
    // execution
    // ---------------------------------------------------------------

    #[test]
    fn execute_creates_dirs_and_files() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest {
            dirs: DirsSection {
                create: vec![".doctrine/custom-dir".to_string()],
            },
            target: ".doctrine".to_string(),
            ..Manifest::default_for_tests()
        };
        let plan = build_plan(&manifest, dir.path());
        execute_plan(&plan).unwrap();

        assert!(dir.path().join(".doctrine/custom-dir").is_dir());
        // The embedded file x/about.md should be installed.
        let about = dir.path().join(".doctrine/x/about.md");
        assert!(about.is_file());
        let content = fs::read_to_string(&about).unwrap();
        assert!(content.contains("About"));
    }

    #[test]
    fn execute_appends_gitignore() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest {
            gitignore: GitignoreSection {
                entries: vec!["/doctest-entry".to_string()],
            },
            target: ".doctrine".to_string(),
            ..Manifest::default_for_tests()
        };
        let plan = build_plan(&manifest, dir.path());
        execute_plan(&plan).unwrap();

        let gi_content = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert!(gi_content.contains("/doctest-entry"));
    }

    // SL-018 OQ-C / EX-3: install hints at `memory sync` but does NOT run it.
    #[test]
    fn install_hints_at_the_standalone_memory_sync_verb() {
        assert!(
            sync_hint().contains("memory sync"),
            "the post-install hint must point at `memory sync`"
        );
    }

    // SL-018 VT-3: install alone writes no shipped/ — sync is the standalone verb
    // that populates the derived corpus, never install (OQ-C).
    #[test]
    fn install_writes_no_shipped_tree() {
        let dir = tempfile::tempdir().unwrap();
        // The REAL manifest: items/ is created, shipped/ is gitignored-not-created.
        let manifest = load_manifest().unwrap();
        let plan = build_plan(&manifest, dir.path());
        execute_plan(&plan).unwrap();

        assert!(
            dir.path().join(".doctrine/memory/items").is_dir(),
            "install materializes the committed items/ tree"
        );
        assert!(
            !dir.path().join(".doctrine/memory/shipped").exists(),
            "install must not create the derived shipped/ tree — that is `memory sync`'s job"
        );
    }

    #[test]
    fn execute_skips_existing_files() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest {
            target: ".doctrine".to_string(),
            ..Manifest::default_for_tests()
        };
        let dest = dir.path().join(".doctrine/x/about.md");
        fs::create_dir_all(dest.parent().unwrap()).unwrap();
        let original = "original content";
        fs::write(&dest, original).unwrap();

        let plan = build_plan(&manifest, dir.path());
        execute_plan(&plan).unwrap();

        // Must still be original.
        let content = fs::read_to_string(&dest).unwrap();
        assert_eq!(content, original);
    }

    // SL-011 VT-1: the boot governance layer rides the existing seed path —
    // created create-if-missing, left untouched when already present.
    #[test]
    fn seeds_governance_when_missing_and_skips_when_present() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest {
            target: ".doctrine".to_string(),
            ..Manifest::default_for_tests()
        };
        let dest = dir.path().join(".doctrine/governance.md");

        // missing → seeded with the embedded template.
        execute_plan(&build_plan(&manifest, dir.path())).unwrap();
        assert!(dest.is_file(), "governance.md seeded when missing");
        assert!(
            fs::read_to_string(&dest)
                .unwrap()
                .contains("Governance (project)"),
            "seeded from the embedded template",
        );

        // present → a re-install leaves the user's edits untouched (Skip).
        let edited = "# Governance (project)\n\nmy own pointers\n";
        fs::write(&dest, edited).unwrap();
        execute_plan(&build_plan(&manifest, dir.path())).unwrap();
        assert_eq!(
            fs::read_to_string(&dest).unwrap(),
            edited,
            "an existing governance.md is never clobbered",
        );
    }

    // ---------------------------------------------------------------
    // helpers
    // ---------------------------------------------------------------

    impl Manifest {
        fn default_for_tests() -> Self {
            Manifest {
                target: default_target(),
                dirs: DirsSection::default(),
                gitignore: GitignoreSection::default(),
                root_markers: RootMarkersSection::default(),
            }
        }
    }
}
