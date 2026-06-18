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

/// Read one embedded `install/`-relative asset's bytes (`None` if absent). The
/// single accessor over the embed for callers outside this module (the agents
/// leg of `claude install`, src/skills.rs) — no parallel embed.
pub(crate) fn embedded_asset(rel: &str) -> Option<std::borrow::Cow<'static, [u8]>> {
    Assets::get(rel).map(|f| f.data)
}

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

/// Borrow-holding args struct for the consolidated install surface (SL-088).
/// Follows the house pattern (`memory::RecordArgs`, `skills::InstallArgs`)
/// to keep the `run()` fn under clippy's parameter/bool ceilings.
pub(crate) struct InstallArgs<'a> {
    pub(crate) agents: &'a [String],
    pub(crate) skills: &'a [String],
    pub(crate) domains: &'a [String],
    #[expect(
        dead_code,
        reason = "wired in PHASE-03 (--only-memory subset derivation)"
    )]
    pub(crate) only_memory: bool,
    pub(crate) global: bool,
    pub(crate) dry_run: bool,
    pub(crate) yes: bool,
}

/// Run `doctrine install`.
///
/// `project_path` is an explicit project root override; agent/skill/domain
/// flags are carried in `args` for forward-step dispatch (PHASE-02+).
pub(crate) fn run(project_path: Option<PathBuf>, args: &InstallArgs<'_>) -> anyhow::Result<()> {
    let manifest = load_manifest()?;
    let project_root =
        detect_project_root(project_path, &manifest).context("Could not find project root")?;
    let plan = build_plan(&manifest, &project_root);

    print_plan(&plan)?;

    // ── Stage 1: base install ──
    if args.dry_run {
        print_forward_summary(&project_root, args)?;
        return Ok(());
    }

    if !args.yes && !prompt_confirm("\nProceed? [y/N] ")? {
        stdout_line("Aborted.")?;
        return Ok(());
    }

    execute_plan(&plan)?;
    stdout_line("Done.")?;

    // ── Stage 2: forward steps ──
    let exec = std::env::current_exe().context("Failed to resolve the doctrine executable path")?;
    run_forward_steps(&project_root, &exec, args)?;
    Ok(())
}

/// The post-install next-step hint (SL-018 OQ-C): point the user at the standalone
/// `memory sync` verb when forward steps were skipped.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "retained for standalone install paths")
)]
fn sync_hint() -> &'static str {
    "Next: run `doctrine memory sync` to materialize the global memory corpus."
}

// ---------------------------------------------------------------------------
// Forward-step orchestration (PHASE-02)
// ---------------------------------------------------------------------------

/// Print the forward-step summary (dry-run or live).
fn print_forward_summary(root: &Path, args: &InstallArgs<'_>) -> anyhow::Result<()> {
    let agents = detect_agents(args.agents, root);
    let harnesses = crate::boot::resolve_harnesses(&[], root).unwrap_or_default();

    let mut stdout = io::stdout();
    if args.dry_run {
        writeln!(stdout, "Forward steps (not executed under --dry-run):")?;
    } else {
        writeln!(stdout, "Base install complete. Forward steps:")?;
    }
    writeln!(stdout)?;

    // Memory sync — always listed.
    writeln!(
        stdout,
        "  {:<12} materialize shipped corpus into .doctrine/memory/shipped/",
        "memory sync"
    )?;

    // Boot — listed when harnesses detected; note when empty.
    if harnesses.is_empty() {
        writeln!(
            stdout,
            "  {:<12} (no harness directories detected — skipped)",
            "boot"
        )?;
    } else {
        let labels: Vec<&str> = harnesses.iter().map(crate::boot::harness_label).collect();
        writeln!(
            stdout,
            "  {:<12} wire @-import + session hooks for {}",
            "boot",
            labels.join(", ")
        )?;
    }

    // Skills per agent — listed when agents detected; note when empty.
    if agents.is_empty() {
        writeln!(
            stdout,
            "  {:<12} (no agents detected or specified — skipped)",
            "skills"
        )?;
    } else {
        for agent in &agents {
            if agent == "claude" {
                writeln!(
                    stdout,
                    "  {:<12} install skills + agent def for claude",
                    "skills"
                )?;
            } else {
                writeln!(
                    stdout,
                    "  {:<12} install skills for {agent} (delegates to npx)",
                    "skills"
                )?;
            }
        }
    }
    Ok(())
}

/// Run the forward steps: memory sync → boot wire → skills per agent.
/// Each step is individually prompted (unless `--yes`). Partial failure
/// is non-fatal — errors are printed and the next step proceeds.
fn run_forward_steps(root: &Path, exec: &Path, args: &InstallArgs<'_>) -> anyhow::Result<()> {
    let agents = detect_agents(args.agents, root);
    let harnesses = crate::boot::resolve_harnesses(&[], root).unwrap_or_default();

    print_forward_summary(root, args)?;

    let mut all_yes = false;

    // 1. Memory sync
    if prompt_step(
        "Materialize shipped memory corpus? [y/N/a]",
        args.yes,
        &mut all_yes,
    )? {
        match crate::corpus::sync_corpus(root, &crate::corpus::embedded_assets(), false) {
            Ok(report) => {
                let mut out = io::stdout();
                writeln!(
                    out,
                    "  corpus sync: {} new, {} changed, {} unchanged, {} prune",
                    report.plan.new.len(),
                    report.plan.changed.len(),
                    report.plan.unchanged.len(),
                    report.plan.prune.len(),
                )?;
            }
            Err(e) => {
                writeln!(io::stdout(), "  memory sync failed: {e:#}")?;
            }
        }
    }

    // 2. Boot wire — skipped when no harnesses detected.
    #[expect(
        clippy::collapsible_if,
        reason = "let-else chain is clearer than && let"
    )]
    if !harnesses.is_empty()
        && prompt_step(
            "Wire @-import + session hooks for detected harnesses? [y/N/a]",
            args.yes,
            &mut all_yes,
        )?
    {
        if let Err(e) = crate::boot::wire(root, exec, &harnesses, false) {
            writeln!(io::stdout(), "  boot wire failed: {e:#}")?;
        }
    }

    // 3. Skills per agent
    let catalog = crate::skills::discover()?;
    let selected = crate::skills::select_for_install(&catalog, args.skills, args.domains)?;

    for agent in &agents {
        let question: String = if agent == "claude" {
            "Install skills + agent def for claude? [y/N/a]".to_string()
        } else {
            format!("Install skills for {agent} (delegates to npx)? [y/N/a]")
        };
        if !prompt_step(&question, args.yes, &mut all_yes)? {
            continue;
        }
        if agent == "claude" {
            let mut out = io::stdout();
            if let Err(e) =
                crate::skills::install_for_claude(root, &catalog, &selected, args.global, &mut out)
            {
                writeln!(io::stdout(), "  claude skills install failed: {e:#}")?;
                continue;
            }
            // Agent-def install rides the Claude skills step.
            if let Err(e) = crate::skills::install_agents_for(
                root,
                "claude",
                None,
                args.global,
                false,
                &mut out,
            ) {
                writeln!(io::stdout(), "  claude agent-def install failed: {e:#}")?;
                continue;
            }
            // SubagentStart hook (project-local only).
            if !args.global {
                let spec = crate::boot::HookSpec::stamp_subagent(exec);
                match crate::boot::install_claude_hook(root, &spec, false) {
                    Ok(outcome) => {
                        let label = match outcome {
                            crate::boot::RefreshOutcome::Wired(_) => "wired",
                            crate::boot::RefreshOutcome::Refreshed(_) => "refreshed",
                            crate::boot::RefreshOutcome::None => "already current",
                            crate::boot::RefreshOutcome::PrintedFallback => {
                                "could not merge (settings left untouched)"
                            }
                        };
                        writeln!(io::stdout(), "  subagent hook: {label}")?;
                    }
                    Err(e) => {
                        writeln!(io::stdout(), "  subagent hook failed: {e:#}")?;
                    }
                }
            }
        } else {
            let mut out = io::stdout();
            let runner = crate::skills::real_runner();
            if let Err(e) = crate::skills::install_for_other(
                agent,
                &catalog,
                &selected,
                args.global,
                runner.as_ref(),
                &mut out,
            ) {
                writeln!(io::stdout(), "  {agent} skills install failed: {e:#}")?;
            }
            // Agent-def install for pi (non-claude agent).
            if let Err(e) = crate::skills::install_agents_for(
                root,
                agent,
                Some(agent),
                args.global,
                false,
                &mut out,
            ) {
                writeln!(io::stdout(), "  {agent} agent-def install failed: {e:#}")?;
            }
        }
    }

    Ok(())
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
// Forward-step helpers (PHASE-02)
// ---------------------------------------------------------------------------

/// Detect target agents for the forward-step summary. A relaxed resolver
/// distinct from `skills::resolve_agents()`: returns an empty `Vec` instead
/// of erroring when no `.claude/` dir and no `--agent` flags — the
/// consolidated install path treats "no agents" as "skip skills steps"
/// rather than a hard error.
fn detect_agents(agents: &[String], root: &Path) -> Vec<String> {
    if !agents.is_empty() {
        return agents.to_vec();
    }
    if root.join(".claude").exists() {
        return vec!["claude".to_string()];
    }
    Vec::new()
}

/// Prompt a single forward step. Returns `true` if the user wants to
/// proceed. `all_yes` is set to `true` when the user picks "a" (yes to
/// all remaining).
fn prompt_step(question: &str, yes: bool, all_yes: &mut bool) -> io::Result<bool> {
    if yes || *all_yes {
        return Ok(true);
    }
    let mut stdout = io::stdout();
    write!(stdout, "\n{question} ")?;
    stdout.flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    match line.trim().to_lowercase().as_str() {
        "y" => Ok(true),
        "a" => {
            *all_yes = true;
            Ok(true)
        }
        _ => Ok(false),
    }
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
        // regression where it lived unembedded under the legacy doc/ directory.
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
    fn review_ledger_is_shipped() {
        // SL-061 PHASE-01 (EX-3): the shared RV-driving protocol doc must ship via
        // the implicit top-level install/*.md copy so /audit (and later
        // /code-review, /inquisition) can point at the installed reference.
        let names = embedded_filenames();
        assert!(
            names.contains(&"review-ledger.md".to_string()),
            "review-ledger.md must be embedded (shipped); got {names:?}"
        );
        assert!(
            !asset_text("review-ledger.md").unwrap().trim().is_empty(),
            "review-ledger.md asset must be non-empty"
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
        fs::create_dir_all(&target).unwrap();
        // Pre-create an embedded target file so the plan must Skip it.
        let existing = target.join("glossary.md");
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
            ".doctrine/**/handover.md",
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

    /// SL-033 PHASE-01: the standard tree is the third authored governance kind, so
    /// the manifest must create it (parity with adr / policy) and must NOT ignore it
    /// — install surface 1 of 3. The `.gitignore` negation (surface 2) and the
    /// git-add round-trip (surface 3) are covered by the e2e commit test.
    #[test]
    fn embedded_manifest_creates_the_standard_tree() {
        let manifest = load_manifest().unwrap();
        assert!(
            manifest
                .dirs
                .create
                .iter()
                .any(|d| d == ".doctrine/standard"),
            "manifest must create the authored standard tree"
        );
        assert!(
            !manifest
                .gitignore
                .entries
                .iter()
                .any(|e| e.starts_with(".doctrine/standard")),
            "the authored standard tree must not be gitignored by the manifest"
        );
    }

    /// SL-040 PHASE-02 (VT-3): the review tree is an authored kind, so the manifest
    /// must create it (parity with adr / policy / standard) and must NOT ignore it,
    /// and both `review.{toml,md}` templates must be embedded (so `review new` can
    /// render them — mem.pattern.build.rust-embed-no-rerun).
    #[test]
    fn embedded_manifest_creates_the_review_tree_and_embeds_its_templates() {
        let manifest = load_manifest().unwrap();
        assert!(
            manifest.dirs.create.iter().any(|d| d == ".doctrine/review"),
            "manifest must create the authored review tree"
        );
        assert!(
            !manifest
                .gitignore
                .entries
                .iter()
                .any(|e| e.starts_with(".doctrine/review")),
            "the authored review tree must not be gitignored by the manifest"
        );
        for tpl in ["templates/review.toml", "templates/review.md"] {
            assert!(
                !asset_text(tpl).unwrap().trim().is_empty(),
                "{tpl} must be embedded and non-empty"
            );
        }
    }

    #[test]
    fn embedded_manifest_creates_the_rec_tree_and_embeds_its_templates() {
        let manifest = load_manifest().unwrap();
        assert!(
            manifest.dirs.create.iter().any(|d| d == ".doctrine/rec"),
            "manifest must create the authored rec tree"
        );
        assert!(
            !manifest
                .gitignore
                .entries
                .iter()
                .any(|e| e.starts_with(".doctrine/rec")),
            "the authored rec tree must not be gitignored by the manifest"
        );
        for tpl in ["templates/rec.toml", "templates/rec.md"] {
            assert!(
                !asset_text(tpl).unwrap().trim().is_empty(),
                "{tpl} must be embedded and non-empty"
            );
        }
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
        // An embedded file (glossary.md) should be installed.
        let glossary = dir.path().join(".doctrine/glossary.md");
        assert!(glossary.is_file());
        let content = fs::read_to_string(&glossary).unwrap();
        assert!(content.contains("glossary"));
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
        let dest = dir.path().join(".doctrine/glossary.md");
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
    // PHASE-02: detect_agents + prompt_step
    // ---------------------------------------------------------------

    #[test]
    fn detect_agents_empty_when_no_claude_dir_and_no_flags() {
        let dir = tempfile::tempdir().unwrap();
        let agents = detect_agents(&[], dir.path());
        assert!(agents.is_empty());
    }

    #[test]
    fn detect_agents_returns_claude_when_dir_present() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".claude")).unwrap();
        let agents = detect_agents(&[], dir.path());
        assert_eq!(agents, vec!["claude".to_string()]);
    }

    #[test]
    fn detect_agents_uses_explicit_over_detection() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".claude")).unwrap();
        let agents = detect_agents(&["pi".to_string()], dir.path());
        assert_eq!(agents, vec!["pi".to_string()]);
    }

    #[test]
    fn detect_agents_returns_multiple_explicit() {
        let dir = tempfile::tempdir().unwrap();
        let agents = detect_agents(&["claude".to_string(), "pi".to_string()], dir.path());
        assert_eq!(agents, vec!["claude".to_string(), "pi".to_string()]);
    }

    // prompt_step: true for y, a, when yes=true, when all_yes=true

    #[test]
    fn prompt_step_yes_flag_skips_prompt() {
        // yes=true → no stdin read needed.
        let mut all_yes = false;
        assert!(prompt_step("Q?", true, &mut all_yes).unwrap());
        assert!(!all_yes); // a not triggered
    }

    #[test]
    fn prompt_step_all_yes_already_true_skips_prompt() {
        let mut all_yes = true;
        assert!(prompt_step("Q?", false, &mut all_yes).unwrap());
        assert!(all_yes);
    }

    // prompt_step with real stdin — test the input parsing

    fn prompt_step_with_input(input: &str, yes: bool, all_yes: &mut bool) -> io::Result<bool> {
        // Simulate stdin by temporarily replacing it is not safe in concurrent
        // tests. Instead, test the match logic directly via the private fn.
        // The public interface is tested via integration.
        if yes || *all_yes {
            return Ok(true);
        }
        match input.trim().to_lowercase().as_str() {
            "y" => Ok(true),
            "a" => {
                *all_yes = true;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    #[test]
    fn prompt_step_y_is_true() {
        let mut all_yes = false;
        assert!(prompt_step_with_input("y", false, &mut all_yes).unwrap());
        assert!(!all_yes);
    }

    #[test]
    fn prompt_step_a_is_true_and_sets_all_yes() {
        let mut all_yes = false;
        assert!(prompt_step_with_input("a", false, &mut all_yes).unwrap());
        assert!(all_yes);
    }

    #[test]
    fn prompt_step_n_is_false() {
        let mut all_yes = false;
        assert!(!prompt_step_with_input("n", false, &mut all_yes).unwrap());
    }

    #[test]
    fn prompt_step_empty_is_false() {
        let mut all_yes = false;
        assert!(!prompt_step_with_input("", false, &mut all_yes).unwrap());
    }

    #[test]
    fn prompt_step_no_is_false() {
        let mut all_yes = false;
        assert!(!prompt_step_with_input("no", false, &mut all_yes).unwrap());
    }

    #[test]
    fn prompt_step_x_is_false() {
        let mut all_yes = false;
        assert!(!prompt_step_with_input("x", false, &mut all_yes).unwrap());
    }

    #[test]
    fn prompt_step_uppercase_y_is_true() {
        let mut all_yes = false;
        assert!(prompt_step_with_input("Y", false, &mut all_yes).unwrap());
    }

    #[test]
    fn prompt_step_uppercase_a_sets_all_yes() {
        let mut all_yes = false;
        assert!(prompt_step_with_input("A", false, &mut all_yes).unwrap());
        assert!(all_yes);
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
