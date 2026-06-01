#![allow(clippy::same_name_method, reason = "rust-embed derive generates conflicting method names")]

use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
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
    #[serde(default = "default_markers")]
    markers: Vec<String>,
}

impl Default for RootMarkersSection {
    fn default() -> Self {
        Self {
            markers: default_markers(),
        }
    }
}

fn default_markers() -> Vec<String> {
    vec![
        ".git".to_string(),
        ".jj".to_string(),
        ".project".to_string(),
        "Cargo.toml".to_string(),
    ]
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

/// Run `heresy install`.
///
/// `project_path` is an explicit project root override; `dry_run_only` prints
/// and exits; `yes` skips the interactive prompt.
pub(crate) fn run(project_path: Option<PathBuf>, dry_run_only: bool, yes: bool) -> anyhow::Result<()> {
    let manifest = load_manifest()?;
    let project_root = detect_project_root(project_path, &manifest)
        .context("Could not find project root")?;
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
    Ok(())
}

// ---------------------------------------------------------------------------
// Manifest
// ---------------------------------------------------------------------------

fn load_manifest() -> anyhow::Result<Manifest> {
    let file = Assets::get("manifest.toml")
        .context("install/manifest.toml is missing from embedded assets")?;
    let text = std::str::from_utf8(&file.data)
        .context("install/manifest.toml is not valid UTF-8")?;
    let manifest: Manifest = toml::from_str(text)
        .context("Failed to parse install/manifest.toml")?;
    Ok(manifest)
}

// ---------------------------------------------------------------------------
// Project root detection
// ---------------------------------------------------------------------------

/// Walk up from `start` (or CWD) looking for any marker file/dir.
fn detect_project_root(explicit: Option<PathBuf>, manifest: &Manifest) -> anyhow::Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path);
    }

    let cwd =
        std::env::current_dir().context("Failed to get current working directory")?;
    let markers = &manifest.root_markers.markers;

    for ancestor in cwd.ancestors() {
        for marker in markers {
            let candidate = ancestor.join(marker);
            if candidate.exists() {
                return Ok(ancestor.to_path_buf());
            }
        }
    }

    bail!(
        "No project root found. Walked up from '{}' looking for any of: {:?}",
        cwd.display(),
        markers,
    )
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
                writeln!(stdout, "  skip         {} → {} (exists)", source, dest.display())?;
            }
            Step::Gitignore { entry, dest } => {
                writeln!(stdout, "  gitignore    + \"{entry}\"  ({})", dest.display())?;
            }
        }
    }
    Ok(())
}

fn prompt_confirm(prompt: &str) -> anyhow::Result<bool> {
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
                    fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create parent dir for {}", dest.display()))?;
                }
                fs::write(dest, &file.data)
                    .with_context(|| format!("Failed to write {}", dest.display()))?;
            }
            Step::Skip { .. } => {
                // nothing to do
            }
            Step::Gitignore { entry, dest } => {
                let mut file = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(dest)
                    .with_context(|| format!("Failed to open {} for appending", dest.display()))?;
                writeln!(file, "{entry}")
                    .with_context(|| format!("Failed to append gitignore entry to {}", dest.display()))?;
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

        let has_skip = plan.steps.iter().any(|s| {
            matches!(s, Step::Skip { dest, .. } if dest == &existing)
        });
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
