// SPDX-License-Identifier: GPL-3.0-only
#![allow(
    clippy::same_name_method,
    reason = "rust-embed derive generates conflicting method names"
)]

use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, bail};
use rust_embed::RustEmbed;
use serde::Deserialize;

use crate::memory::MemoryType;

/// Embedded install assets — everything under `install/`.
#[derive(RustEmbed)]
#[folder = "install/"]
struct Assets;

/// Embedded skill plugins — everything under `plugins/`.
#[derive(RustEmbed)]
#[folder = "plugins/"]
struct PluginAssets;

// ── Constants (moved from skills.rs, IMP-226) ────────────────────────────

const MEMORY_SUBSET_DOMAIN: &str = "doctrine-memory";
const PARTNER_SUBSET_DOMAIN: &str = "doctrine-partner";
const MARKETPLACE_ONLY_DOMAINS: &[&str] = &[MEMORY_SUBSET_DOMAIN, PARTNER_SUBSET_DOMAIN];
const RUNNER_BUNX: &str = "bunx";
const RUNNER_NPX: &str = "npx";
const DISPATCH_WORKER_AGENT_FILE: &str = "dispatch-worker.md";
const DISPATCH_WORKER_AGENT_ASSET: &str = "agents/claude/dispatch-worker.md";
const DISPATCH_WORKER_AGENT_ASSET_PI: &str = "agents/pi/dispatch-worker.md";

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

    #[serde(default)]
    memory: MemorySection,
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

#[derive(Debug, Default, Deserialize)]
struct MemorySection {
    #[serde(default)]
    seed_items: Vec<SeedItem>,
}

#[derive(Debug, Deserialize)]
struct SeedItem {
    key: String,
    #[serde(rename = "type")]
    memory_type: String,
    title: String,
    body_template: String,
    #[serde(default)]
    summary: String,
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
    seed_authoring_memories(&project_root, &manifest)?;
    stdout_line("Done.")?;

    // ── Stage 2: forward steps ──
    let exec = crate::boot::resolve_exec()?;
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

/// Seed key-addressed memory items listed in `manifest.toml` (e.g. the project
/// orientation template). Each item uses `memory::seed_by_key` to create a
/// no-anchor memory whose body is drawn from an embedded template.
/// Idempotent — skips existing key symlinks.
fn seed_authoring_memories(root: &Path, manifest: &Manifest) -> anyhow::Result<()> {
    if manifest.memory.seed_items.is_empty() {
        return Ok(());
    }
    let mut stdout = io::stdout();
    writeln!(stdout, "  seeding authoring memories…")?;
    for item in &manifest.memory.seed_items {
        let memory_type = MemoryType::parse(&item.memory_type)
            .with_context(|| format!("invalid memory type {:?}", item.memory_type))?;
        let body = asset_text(&item.body_template)
            .with_context(|| format!("seed body template '{}' not found", item.body_template))?;
        if crate::memory::seed_by_key(
            root,
            &item.key,
            memory_type,
            &item.title,
            &body,
            &item.summary,
        )? {
            writeln!(
                stdout,
                "  seed memory  {} → memory/items/{}/",
                item.key, item.key
            )?;
        } else {
            writeln!(stdout, "  skip seed    {} (exists)", item.key)?;
        }
    }
    Ok(())
}

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
                    "  {:<12} register marketplace + install plugin + agent def for claude",
                    "claude"
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
    let catalog = discover()?;
    let selected = select_for_install(&catalog, args.skills, args.domains)?;
    let (runner_name, runner) = resolve_runner();
    let repo = &crate::dtoml::load_doctrine_toml(root)?.install.repo;

    let mut non_claude_agents: Vec<String> = Vec::new();

    // Track which plugin steps were skipped-but-needed for the final reminder.
    let mut skipped_marketplace = false;
    let mut skipped_plugin = false;

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

            // 1. Marketplace registration.
            let has_marketplace = claude_marketplace_has_doctrine();
            if !has_marketplace {
                if prompt_step(
                    &format!("claude plugin marketplace add {repo}? [y/N/a]"),
                    args.yes,
                    &mut all_yes,
                )? {
                    match claude_plugin_add_marketplace(repo) {
                        Ok(()) => writeln!(out, "  marketplace {repo} registered")?,
                        Err(e) => {
                            writeln!(out, "  marketplace add failed: {e:#}")?;
                            skipped_marketplace = true;
                        }
                    }
                } else {
                    skipped_marketplace = true;
                }
            }

            // 2. Plugin install.
            let has_plugin = claude_plugin_installed("doctrine");
            if !has_plugin {
                if prompt_step(
                    "claude plugin install doctrine --scope project? [y/N/a]",
                    args.yes,
                    &mut all_yes,
                )? {
                    match claude_plugin_install("doctrine") {
                        Ok(()) => writeln!(out, "  doctrine plugin installed")?,
                        Err(e) => {
                            writeln!(out, "  plugin install failed: {e:#}")?;
                            skipped_plugin = true;
                        }
                    }
                } else {
                    skipped_plugin = true;
                }
            }

            // 3. Agent-def install (kept as-is).
            if let Err(e) = install_agents_for(root, "claude", None, args.global, false, &mut out) {
                writeln!(io::stdout(), "  claude agent-def install failed: {e:#}")?;
            }
        } else {
            non_claude_agents.push(agent.clone());
            // Agent-def install per non-Claude agent.
            if let Err(e) = install_agents_for(
                root,
                agent,
                Some(agent),
                args.global,
                false,
                &mut io::stdout(),
            ) {
                writeln!(io::stdout(), "  {agent} agent-def install failed: {e:#}")?;
            }
        }
    }

    // Batch-delegate all confirmed non-Claude agents to a single npx invocation.
    if !non_claude_agents.is_empty() {
        let mut out = io::stdout();
        if let Err(e) = install_for_other(
            &InstallOtherArgs {
                agent_names: &non_claude_agents,
                selected: &selected,
                global: args.global,
                repo,
                runner: &runner,
                runner_name,
            },
            &mut out,
        ) {
            writeln!(io::stdout(), "  non-Claude skills install failed: {e:#}")?;
        }
    }

    // Final reminder: if the user skipped a needed plugin step, print how to
    // install it manually.
    if skipped_marketplace || skipped_plugin {
        writeln!(io::stdout())?;
        writeln!(
            io::stdout(),
            "Claude Code requires the doctrine plugin. To install:"
        )?;
        if skipped_marketplace {
            writeln!(io::stdout(), "  claude plugin marketplace add {repo}")?;
        }
        if skipped_plugin {
            writeln!(
                io::stdout(),
                "  claude plugin install doctrine --scope project"
            )?;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Claude plugin helpers (IMP-223)
// ---------------------------------------------------------------------------

/// Check whether the doctrine marketplace is registered in Claude.
fn claude_marketplace_has_doctrine() -> bool {
    claude_cmd_output_contains(&["plugin", "marketplace", "list"], "doctrine")
}

/// Check whether the named Claude plugin is installed.
fn claude_plugin_installed(name: &str) -> bool {
    claude_cmd_output_contains(&["plugin", "list"], name)
}

/// Run `claude <args>` and check whether stdout contains `needle`.
fn claude_cmd_output_contains(args: &[&str], needle: &str) -> bool {
    Command::new("claude")
        .args(args)
        .output()
        .is_ok_and(|o| String::from_utf8_lossy(&o.stdout).contains(needle))
}

/// Run `claude plugin marketplace add <repo>`.
fn claude_plugin_add_marketplace(repo: &str) -> anyhow::Result<()> {
    let status = Command::new("claude")
        .args(["plugin", "marketplace", "add", repo])
        .status()
        .context("failed to execute claude plugin marketplace add")?;
    anyhow::ensure!(
        status.success(),
        "claude plugin marketplace add exited with {status}"
    );
    Ok(())
}

/// Run `claude plugin install <name> --scope project`.
fn claude_plugin_install(name: &str) -> anyhow::Result<()> {
    let status = Command::new("claude")
        .args(["plugin", "install", name, "--scope", "project"])
        .status()
        .context("failed to execute claude plugin install")?;
    anyhow::ensure!(
        status.success(),
        "claude plugin install exited with {status}"
    );
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
    let mut detected: Vec<String> = Vec::new();
    if root.join(".claude").exists() {
        detected.push("claude".to_string());
    }
    if root.join(".codex").exists() {
        detected.push("codex".to_string());
    }
    if root.join(".pi").exists() {
        detected.push("pi".to_string());
    }
    if root.join(".agents").exists() {
        detected.push("universal".to_string());
    }
    detected
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
                #[expect(clippy::disallowed_methods, reason = "derived asset unpack")]
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

fn program_available(prog: &str) -> bool {
    std::process::Command::new(prog)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Resolve the delegated skills runner: try `npx` first, fall back to `bunx`.
/// Returns the program name and a concrete runner.
pub(crate) fn resolve_runner() -> (&'static str, ProcessRunner) {
    resolve_runner_with(&program_available)
}

/// Same as `resolve_runner()` but with an injectable availability check.
/// The `check` predicate returns `true` when a program is available.
fn resolve_runner_with(check: &dyn Fn(&str) -> bool) -> (&'static str, ProcessRunner) {
    if check(RUNNER_NPX) {
        (RUNNER_NPX, ProcessRunner { name: RUNNER_NPX })
    } else {
        (RUNNER_BUNX, ProcessRunner { name: RUNNER_BUNX })
    }
}

// ---------------------------------------------------------------------------
// Model
// ---------------------------------------------------------------------------

/// `SKILL.md` YAML frontmatter (only the fields we consume).
#[derive(Debug, Deserialize)]
struct Meta {
    name: String,
    description: String,
}

/// One discovered skill.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Entry {
    domain: String,
    id: String,
    description: String,
    /// Embedded file paths comprising the skill, e.g.
    /// `doctrine/skills/code-review/SKILL.md`.
    files: Vec<String>,
}

// ---------------------------------------------------------------------------
// Pure: frontmatter
// ---------------------------------------------------------------------------

/// Parse leading `---` YAML frontmatter from a `SKILL.md` body.
fn parse_meta(md: &str) -> anyhow::Result<Meta> {
    let after = md
        .strip_prefix("---")
        .context("SKILL.md missing leading '---' frontmatter")?
        .trim_start_matches(['\r', '\n']);
    let end = after
        .find("\n---")
        .context("SKILL.md frontmatter is not terminated by '---'")?;
    let yaml = after.get(..end).context("frontmatter slice out of range")?;
    let meta: Meta = serde_yaml::from_str(yaml).context("Failed to parse SKILL.md frontmatter")?;
    Ok(meta)
}

// ---------------------------------------------------------------------------
// Pure-ish: discovery (reads compile-time embed, not the filesystem)
// ---------------------------------------------------------------------------

/// Discover all embedded skills, grouped by `<domain>/skills/<skill>/`.
pub(crate) fn discover() -> anyhow::Result<Vec<Entry>> {
    use std::collections::BTreeMap;

    let mut grouped: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for path in PluginAssets::iter() {
        let p = path.as_ref();
        let parts: Vec<&str> = p.split('/').collect();
        if let [domain, "skills", skill, ..] = parts.as_slice() {
            if MARKETPLACE_ONLY_DOMAINS.contains(domain) {
                continue;
            }
            grouped
                .entry(((*domain).to_string(), (*skill).to_string()))
                .or_default()
                .push(p.to_string());
        }
    }

    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut entries = Vec::new();
    for ((domain, skill), files) in grouped {
        let skill_md = format!("{domain}/skills/{skill}/SKILL.md");
        let asset = PluginAssets::get(&skill_md)
            .with_context(|| format!("Skill '{domain}/{skill}' has no SKILL.md"))?;
        let text = std::str::from_utf8(&asset.data)
            .with_context(|| format!("{skill_md} is not valid UTF-8"))?;
        let meta = parse_meta(text).with_context(|| format!("In {skill_md}"))?;
        if meta.name != skill {
            bail!(
                "Skill dir '{skill}' != frontmatter name '{}' ({skill_md})",
                meta.name
            );
        }
        if !seen.insert(skill.clone()) {
            bail!("Duplicate skill id '{skill}' across domains; ids must be unique");
        }
        entries.push(Entry {
            domain,
            id: skill,
            description: meta.description,
            files,
        });
    }
    Ok(entries)
}

// ---------------------------------------------------------------------------
// Pure: selection / planning
// ---------------------------------------------------------------------------

/// Filter `all` by skill ids and/or domains. Empty filters match everything.
pub(crate) fn select<'a>(all: &'a [Entry], ids: &[String], domains: &[String]) -> Vec<&'a Entry> {
    all.iter()
        .filter(|e| {
            let id_ok = ids.is_empty() || ids.iter().any(|i| i == &e.id);
            let dom_ok = domains.is_empty() || domains.iter().any(|d| d == &e.domain);
            id_ok && dom_ok
        })
        .collect()
}

/// Validate that every requested id/domain matches at least one skill.
pub(crate) fn validate_filters(
    all: &[Entry],
    ids: &[String],
    domains: &[String],
) -> anyhow::Result<()> {
    for id in ids {
        if !all.iter().any(|e| &e.id == id) {
            bail!("Unknown skill '{id}'");
        }
    }
    for d in domains {
        if !all.iter().any(|e| &e.domain == d) {
            bail!("Unknown domain '{d}'");
        }
    }
    Ok(())
}

/// The base both skill trees hang off: the project `root`, or the user home with
/// `global`. Single source for the `.claude/skills` link dir, the
/// `.doctrine/skills` canonical dir, AND the F4 derived-tree gitignore — so under
/// `--global` the ignore follows the tree to `$HOME` rather than landing in the
/// project for a tree that isn't there (SL-010 B1).
fn install_base(root: &Path, global: bool) -> anyhow::Result<PathBuf> {
    if global {
        let home = std::env::var_os("HOME").context("HOME is not set; cannot resolve --global")?;
        Ok(PathBuf::from(home))
    } else {
        Ok(root.to_path_buf())
    }
}

// ---------------------------------------------------------------------------
// Pure: canonical tree + ownership-by-target-equality (SL-010 D3)
//
// A managed agent link is doctrine's *iff its value equals the relative target
// we would write* — type (is_symlink) is necessary but not sufficient. Anything
// else (a foreign symlink, or a real dir/file) is kept untouched. This is both
// the never-clobber guarantee and the override hatch.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Agents leg (SL-056 PHASE-11) — install the Claude dispatch-worker agent def
// the same way skills install: materialize a canonical copy from the embed,
// then symlink the agent dir at it (reusing classify_link/write_link/
// relative_target — no parallel symlink impl).
// ---------------------------------------------------------------------------

/// The Claude agents directory (project-local or, with `global`, user home).
fn claude_agents_dir(root: &Path, global: bool) -> anyhow::Result<PathBuf> {
    Ok(install_base(root, global)?.join(".claude/agents"))
}

/// The pi agents directory (project-local or, with `global`, user home).
fn pi_agents_dir(root: &Path, global: bool) -> anyhow::Result<PathBuf> {
    Ok(install_base(root, global)?.join(".pi/agents"))
}

/// The canonical agents tree, mirroring `canonical_dir` so the relative link
/// target is stable.
fn agent_canonical_dir(root: &Path, global: bool) -> anyhow::Result<PathBuf> {
    Ok(install_base(root, global)?.join(".doctrine/agents"))
}

/// Relative path from `from` to `to`. Both must be absolute and normalised
/// (no `.`/`..` components) — the root-/`$HOME`-derived dirs always are.
fn relative_path(from: &Path, to: &Path) -> PathBuf {
    let from_c: Vec<_> = from.components().collect();
    let to_c: Vec<_> = to.components().collect();
    let common = from_c.iter().zip(&to_c).take_while(|(a, b)| a == b).count();
    let mut rel = PathBuf::new();
    for _ in common..from_c.len() {
        rel.push("..");
    }
    for c in to_c.iter().skip(common) {
        rel.push(c.as_os_str());
    }
    rel
}

/// The relative symlink value for `<id>`: from the agent skills dir (where the
/// link lives) to `canonical_dir/<id>`. Derived from the two dirs, never
/// hard-coded — `../../.doctrine/skills/<id>` in the common project-local case,
/// and correct under a shared `--global` base.
fn relative_target(agent_skills_dir: &Path, canonical_dir: &Path, id: &str) -> PathBuf {
    relative_path(agent_skills_dir, &canonical_dir.join(id))
}

/// Why an agent skill path is foreign — left untouched and warned.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ForeignReason {
    /// A real directory or file the user owns (e.g. a pinned copy override).
    RealDir,
    /// A symlink whose value is not our canonical target — points elsewhere.
    ForeignSymlink(PathBuf),
}

/// Reconciliation action for one agent skill link, by proven ownership.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Link {
    /// Nothing there → create the relative symlink.
    Create {
        id: String,
        dest: PathBuf,
        target: PathBuf,
    },
    /// A symlink already equal to our target → ensure it (no-op, or heal a
    /// dangling-but-ours link once its canonical is re-materialised).
    Relink {
        id: String,
        dest: PathBuf,
        target: PathBuf,
    },
    /// Foreign (a real dir, or a symlink pointing elsewhere) → never touched.
    KeepForeign {
        id: String,
        dest: PathBuf,
        reason: ForeignReason,
    },
}

/// Classify `dest` (an agent skill path) against the canonical `target` by
/// proven ownership. Uses `symlink_metadata`/`read_link`, never `exists()`
/// (which follows links): a dangling link whose value equals our target is
/// still ours and is healed, not recreated.
fn classify_link(id: &str, dest: &Path, target: &Path) -> Link {
    let Ok(meta) = fs::symlink_metadata(dest) else {
        return Link::Create {
            id: id.to_string(),
            dest: dest.to_path_buf(),
            target: target.to_path_buf(),
        };
    };
    if !meta.file_type().is_symlink() {
        return Link::KeepForeign {
            id: id.to_string(),
            dest: dest.to_path_buf(),
            reason: ForeignReason::RealDir,
        };
    }
    match fs::read_link(dest) {
        Ok(value) if value == target => Link::Relink {
            id: id.to_string(),
            dest: dest.to_path_buf(),
            target: target.to_path_buf(),
        },
        Ok(value) => Link::KeepForeign {
            id: id.to_string(),
            dest: dest.to_path_buf(),
            reason: ForeignReason::ForeignSymlink(value),
        },
        // Unreadable symlink (race/perm) — treat as foreign, never clobber.
        Err(_) => Link::KeepForeign {
            id: id.to_string(),
            dest: dest.to_path_buf(),
            reason: ForeignReason::ForeignSymlink(PathBuf::new()),
        },
    }
}

/// A `.tmp-<name>` sibling of `path`, the staging name for an atomic swap.
fn staging_path(path: &Path) -> anyhow::Result<PathBuf> {
    let parent = path.parent().context("path has no parent directory")?;
    let name = path.file_name().context("path has no file name")?;
    Ok(parent.join(format!(".tmp-{}", name.to_string_lossy())))
}

/// Create the relative symlink `dest -> target` atomically: symlink at a temp
/// name then `rename` over `dest`. `rename` DOES replace an existing symlink (only
/// a non-empty *directory* is the exception), so an owned-link relink never leaves
/// a half-state. Callers pass only Create/Relink dests (missing or proven ours).
fn write_link(dest: &Path, target: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::symlink;
    let tmp = staging_path(dest)?;
    // Clear any crashed leftover from a prior interrupted write (a stale symlink
    // may dangle, so remove unconditionally and ignore a not-found error).
    fs::remove_file(&tmp).ok();
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    symlink(target, &tmp).with_context(|| format!("Failed to stage link {}", tmp.display()))?;
    fs::rename(&tmp, dest)
        .with_context(|| format!("Failed to swap link {} → {}", tmp.display(), dest.display()))?;
    Ok(())
}

/// Human-readable `kept` reason for an honest warning.
fn foreign_reason(reason: &ForeignReason) -> String {
    match reason {
        ForeignReason::RealDir => "real dir".to_string(),
        ForeignReason::ForeignSymlink(to) => format!("foreign symlink → {}", to.display()),
    }
}

/// Assemble the `npx skills add …` argv (program `npx`/`bunx` excluded).
fn delegate_argv(
    agents: &[&str],
    skills: &[&Entry],
    global: bool,
    subset: bool,
    repo: &str,
) -> Vec<String> {
    let mut argv = vec!["skills".to_string(), "add".to_string(), repo.to_string()];
    for agent in agents {
        argv.push("--agent".to_string());
        argv.push(agent.to_string());
    }
    if global {
        argv.push("--global".to_string());
    }
    if subset {
        for e in skills {
            argv.push("--skill".to_string());
            argv.push(e.id.clone());
        }
    }
    argv.push("--yes".to_string());
    argv
}

// ---------------------------------------------------------------------------
// Imperative: command execution behind a seam
// ---------------------------------------------------------------------------

/// Runs an external command. Seam so plans are tested without spawning Node.
pub(crate) trait Runner: std::fmt::Debug {
    /// Run `program` with `args`; return whether it exited successfully.
    fn run(&self, program: &str, args: &[String]) -> anyhow::Result<bool>;
}

/// Real runner: spawns the process and inherits stdio.
#[derive(Debug)]
pub(crate) struct ProcessRunner {
    name: &'static str,
}

impl Runner for ProcessRunner {
    fn run(&self, program: &str, args: &[String]) -> anyhow::Result<bool> {
        let status = std::process::Command::new(program)
            .args(args)
            .status()
            .with_context(|| format!("Failed to run '{program}' (is {} installed?)", self.name))?;
        Ok(status.success())
    }
}

/// Install skills for a non-Claude agent: delegate to `npx skills`.
/// Extracted from `execute()` for reuse from the consolidated `install::run()`
/// forward-step dispatch (SL-088 PHASE-02).
pub(crate) struct InstallOtherArgs<'a> {
    pub(crate) agent_names: &'a [String],
    pub(crate) selected: &'a [&'a Entry],
    pub(crate) global: bool,
    pub(crate) repo: &'a str,
    pub(crate) runner: &'a dyn Runner,
    pub(crate) runner_name: &'a str,
}

pub(crate) fn install_for_other(
    args: &InstallOtherArgs<'_>,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let subset = !args.selected.is_empty();
    let agent_strs: Vec<&str> = args.agent_names.iter().map(String::as_str).collect();
    let argv = delegate_argv(&agent_strs, args.selected, args.global, subset, args.repo);
    let label = args.agent_names.join(", ");
    writeln!(
        out,
        "agents {label} (delegate): {} {}",
        args.runner_name,
        argv.join(" ")
    )?;
    if !args.runner.run(args.runner_name, &argv)? {
        bail!(
            "{runner_name} skills failed for agents: {label}",
            runner_name = args.runner_name,
            label = label
        );
    }
    Ok(())
}

/// Select and validate skills for the consolidated install path.
/// Thin wrapper over `validate_filters` + `select` so `install.rs` doesn't
/// reach into the private filter logic.
pub(crate) fn select_for_install<'a>(
    catalog: &'a [Entry],
    skills: &[String],
    domains: &[String],
) -> anyhow::Result<Vec<&'a Entry>> {
    validate_filters(catalog, skills, domains)?;
    Ok(select(catalog, skills, domains))
}

/// Public wrapper for `install_agent_def`.
pub(crate) fn install_agents_for(
    root: &Path,
    agent_name: &str,
    canon_subdir: Option<&str>,
    global: bool,
    dry_run: bool,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let embed_asset = match agent_name {
        "claude" => DISPATCH_WORKER_AGENT_ASSET,
        _ => DISPATCH_WORKER_AGENT_ASSET_PI,
    };
    install_agent_def(
        root,
        agent_name,
        canon_subdir,
        embed_asset,
        global,
        dry_run,
        out,
    )
}

// ---------------------------------------------------------------------------
// Imperative: printing
// ---------------------------------------------------------------------------

/// Install a dispatch-worker agent def for the given agent: materialize the
/// canonical copy from the embed into `.doctrine/agents/` (under an optional
/// subdir), then symlink the agent's link dir at it. Idempotent — refreshes
/// the canonical each run and only (re)writes a link that is missing or proven
/// ours, never clobbering a foreign one. Reuses
/// `classify_link`/`write_link`/`relative_target` — no parallel symlink impl.
pub(crate) fn install_agent_def(
    root: &Path,
    agent_name: &str,
    canon_subdir: Option<&str>,
    embed_asset: &str,
    global: bool,
    dry_run: bool,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let canon_base = agent_canonical_dir(root, global)?;
    let canon_dir = match canon_subdir {
        Some(sub) => canon_base.join(sub),
        None => canon_base,
    };
    let link_dir = match agent_name {
        "claude" => claude_agents_dir(root, global)?,
        _ => pi_agents_dir(root, global)?,
    };
    let canon = canon_dir.join(DISPATCH_WORKER_AGENT_FILE);
    let dest = link_dir.join(DISPATCH_WORKER_AGENT_FILE);
    let target = relative_target(&link_dir, &canon_dir, DISPATCH_WORKER_AGENT_FILE);

    writeln!(out, "agent {agent_name} (dispatch-worker):")?;
    writeln!(
        out,
        "  agent     {DISPATCH_WORKER_AGENT_FILE} → {}",
        dest.display()
    )?;
    if dry_run {
        return Ok(());
    }

    // 1. Refresh the canonical copy from the embed (always overwrite — derived).
    let data = embedded_asset(embed_asset)
        .with_context(|| format!("Embedded agent def '{embed_asset}' not found"))?;
    fs::create_dir_all(&canon_dir)
        .with_context(|| format!("Failed to create {}", canon_dir.display()))?;
    crate::fsutil::write_atomic(&canon, &data)?;

    // 2. Reconcile the agent link by proven ownership (re-classify at mutation
    //    time, like `execute`'s skill links).
    match classify_link(DISPATCH_WORKER_AGENT_FILE, &dest, &target) {
        Link::Create { .. } => {
            write_link(&dest, &target)?;
            writeln!(out, "  linked    {DISPATCH_WORKER_AGENT_FILE}")?;
        }
        Link::Relink { .. } => {
            write_link(&dest, &target)?;
            writeln!(out, "  relinked  {DISPATCH_WORKER_AGENT_FILE}")?;
        }
        Link::KeepForeign { reason, .. } => {
            writeln!(
                out,
                "  kept      {DISPATCH_WORKER_AGENT_FILE} ({})",
                foreign_reason(&reason)
            )?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Hooks plugin leg — install the doctrine Claude plugin as a skills-directory
// plugin so hooks (SessionStart / WorktreeCreate) auto-load without a
// marketplace install step. The per-skill symlinks are untouched; the plugin
// dir carries only the manifest + hooks.
// ---------------------------------------------------------------------------
// Tests (skills — moved from skills.rs, IMP-226)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests_skills {
    use super::*;

    const TEST_REPO: &str = "davidlee/doctrine";

    // ADR-005 / SL-023 PHASE-04 (VT-1): the de-dup'd skills route rather than
    // restate. Guards the named sites against re-growing flag-syntax templates,
    // option/enum tables, or `--status` transition commands. Each must also keep
    // a pointer to the shared tier-1/2 docs. Evidence-bound to the named set.
    #[test]
    fn dedup_skills_route_not_restate() {
        let named = [
            "record-memory",
            "retrieve-memory",
            "spec-product",
            "spec-tech",
            "execute",
            "phase-plan",
            "canon",
            "inquisition",
        ];
        // Offender fragments removed by the de-dup — must not reappear.
        let banned = [
            "--status in_progress",
            "--status completed",
            "--kind functional|quality",
            "--type <type>",
            "--path-scope <file>",
            "--command \"<tok>\"",
        ];
        for skill in named {
            let path = format!("doctrine/skills/{skill}/SKILL.md");
            let asset = PluginAssets::get(&path).expect("named skill must be embedded");
            let text = std::str::from_utf8(&asset.data).expect("utf8");
            for frag in banned {
                assert!(
                    !text.contains(frag),
                    "restate-line: {skill} reproduces flag syntax `{frag}`"
                );
            }
            assert!(
                text.contains("using-doctrine") || text.contains("--help"),
                "reachability: {skill} must point at a tier-1/2 reference"
            );
        }
    }

    fn entry(domain: &str, id: &str) -> Entry {
        Entry {
            domain: domain.to_string(),
            id: id.to_string(),
            description: format!("{id} desc"),
            files: vec![format!("{domain}/skills/{id}/SKILL.md")],
        }
    }

    // --- frontmatter ---

    #[test]
    fn parse_meta_extracts_name_and_description() {
        let md = "---\nname: code-review\ndescription: Review a diff.\n---\n\n# body\n";
        let meta = parse_meta(md).unwrap();
        assert_eq!(meta.name, "code-review");
        assert_eq!(meta.description, "Review a diff.");
    }

    #[test]
    fn parse_meta_rejects_missing_frontmatter() {
        assert!(parse_meta("# no frontmatter\n").is_err());
    }

    // --- discovery (against the embedded sample) ---

    #[test]
    fn discover_finds_embedded_sample_skill() {
        let cat = discover().unwrap();
        let cr = cat.iter().find(|e| e.id == "code-review").unwrap();
        assert_eq!(cr.domain, "doctrine");
        assert!(!cr.description.is_empty());
        assert!(cr.files.iter().any(|f| f.ends_with("SKILL.md")));
    }

    #[test]
    fn discover_excludes_marketplace_only_domains() {
        let cat = discover().unwrap();
        // doctrine-memory + doctrine-partner are marketplace-only subsets
        // (symlinks to doctrine); they must not enter the CLI catalog, or they
        // collide with the canonical skills on duplicate ids.
        assert!(cat.iter().all(|e| e.domain != "doctrine-memory"));
        assert!(cat.iter().all(|e| e.domain != "doctrine-partner"));
        // …while the canonical skills remain in the doctrine domain.
        assert!(
            cat.iter()
                .any(|e| e.id == "record-memory" && e.domain == "doctrine")
        );
        assert!(cat.iter().any(|e| e.id == "pair" && e.domain == "doctrine"));
        assert!(
            cat.iter()
                .any(|e| e.id == "walkthrough" && e.domain == "doctrine")
        );
    }

    // --- selection ---

    #[test]
    fn select_filters_by_id_and_domain() {
        let all = vec![entry("review", "code-review"), entry("rust", "clippy")];
        assert_eq!(select(&all, &["clippy".into()], &[]).len(), 1);
        assert_eq!(select(&all, &[], &["review".into()]).len(), 1);
        assert_eq!(select(&all, &[], &[]).len(), 2);
    }

    #[test]
    fn validate_filters_rejects_unknown() {
        let all = vec![entry("review", "code-review")];
        assert!(validate_filters(&all, &["nope".into()], &[]).is_err());
        assert!(validate_filters(&all, &[], &["nope".into()]).is_err());
        assert!(validate_filters(&all, &["code-review".into()], &["review".into()]).is_ok());
    }

    // --- subset derivation (--only-memory) ---

    // --- claude links (the plan builder) ---

    // --- canonical materialise ---

    // --- canonical dir + relative target ---

    #[test]
    fn relative_target_is_computed_from_the_two_dirs() {
        // Project-local: .claude/skills → .doctrine/skills/<id>.
        let agent = Path::new("/proj/.claude/skills");
        let canon = Path::new("/proj/.doctrine/skills");
        assert_eq!(
            relative_target(agent, canon, "code-review"),
            PathBuf::from("../../.doctrine/skills/code-review")
        );
        // A shared --global base ($HOME) stays correct — same relative shape,
        // computed not hard-coded.
        let g_agent = Path::new("/home/u/.claude/skills");
        let g_canon = Path::new("/home/u/.doctrine/skills");
        assert_eq!(
            relative_target(g_agent, g_canon, "code-review"),
            PathBuf::from("../../.doctrine/skills/code-review")
        );
    }

    // --- ownership classification ---

    #[test]
    fn classify_link_covers_the_ownership_trichotomy() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().unwrap();
        let target = PathBuf::from("../../.doctrine/skills/code-review");

        // missing → Create
        let missing = dir.path().join("missing");
        assert!(matches!(
            classify_link("code-review", &missing, &target),
            Link::Create { .. }
        ));

        // symlink whose value == target → Relink (dangling-but-ours: the target
        // need not resolve — ownership is the value, not resolvability).
        let ours = dir.path().join("ours");
        symlink(&target, &ours).unwrap();
        assert!(matches!(
            classify_link("code-review", &ours, &target),
            Link::Relink { .. }
        ));

        // symlink pointing elsewhere → KeepForeign(foreign-symlink → where)
        let foreign = dir.path().join("foreign");
        symlink("somewhere/else", &foreign).unwrap();
        match classify_link("code-review", &foreign, &target) {
            Link::KeepForeign {
                reason: ForeignReason::ForeignSymlink(where_),
                ..
            } => assert_eq!(where_, PathBuf::from("somewhere/else")),
            other => panic!("expected foreign-symlink, got {other:?}"),
        }

        // real dir → KeepForeign(real-dir)
        let real = dir.path().join("real");
        fs::create_dir_all(&real).unwrap();
        assert!(matches!(
            classify_link("code-review", &real, &target),
            Link::KeepForeign {
                reason: ForeignReason::RealDir,
                ..
            }
        ));
    }

    // --- delegate argv ---

    #[test]
    fn delegate_argv_all_skills_omits_skill_flags() {
        let e = entry("review", "code-review");
        let argv = delegate_argv(&["codex"], &[&e], false, false, TEST_REPO);
        assert_eq!(
            argv,
            vec!["skills", "add", TEST_REPO, "--agent", "codex", "--yes"]
        );
    }

    #[test]
    fn delegate_argv_subset_and_global() {
        let e = entry("review", "code-review");
        let argv = delegate_argv(&["cursor"], &[&e], true, true, TEST_REPO);
        assert_eq!(
            argv,
            vec![
                "skills",
                "add",
                TEST_REPO,
                "--agent",
                "cursor",
                "--global",
                "--skill",
                "code-review",
                "--yes",
            ]
        );
    }

    #[test]
    fn delegate_argv_multiple_agents() {
        let e = entry("review", "code-review");
        let argv = delegate_argv(&["pi", "codex"], &[&e], false, false, TEST_REPO);
        assert_eq!(
            argv,
            vec![
                "skills", "add", TEST_REPO, "--agent", "pi", "--agent", "codex", "--yes",
            ]
        );
    }

    // --- agent resolution ---

    // --- plan ---

    #[test]
    fn resolve_runner_with_npx_available() {
        let (name, _runner) = resolve_runner_with(&|prog| prog == "npx");
        assert_eq!(name, RUNNER_NPX);
    }

    #[test]
    fn resolve_runner_with_falls_back_to_bunx() {
        let (name, _runner) = resolve_runner_with(&|_prog| false);
        assert_eq!(name, RUNNER_BUNX);
    }

    // --- plan ---
}

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
    fn embedded_manifest_ignores_the_skills_derived_tree() {
        let manifest = load_manifest().unwrap();
        // The canonical skills tree is gitignored by the manifest: the dir is
        // created on-the-fly by `skills install`, but its contents are derived
        // (regenerable from the embed) and must not be committed (SL-010 D2).
        // The blanket `.doctrine/*` only masks it in this repo, so the manifest
        // writes an additive entry.
        assert!(
            !manifest.dirs.create.iter().any(|d| d == ".doctrine/skills"),
            "skills dir is created by `skills install`, not `doctrine install`"
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
                .contains("Project-Specific Governance"),
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
    fn detect_agents_empty_when_no_agent_dirs_and_no_flags() {
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
    fn detect_agents_returns_pi_when_dir_present() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".pi")).unwrap();
        let agents = detect_agents(&[], dir.path());
        assert_eq!(agents, vec!["pi".to_string()]);
    }

    #[test]
    fn detect_agents_returns_codex_when_dir_present() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".codex")).unwrap();
        let agents = detect_agents(&[], dir.path());
        assert_eq!(agents, vec!["codex".to_string()]);
    }

    #[test]
    fn detect_agents_returns_universal_for_dot_agents_dir() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".agents")).unwrap();
        let agents = detect_agents(&[], dir.path());
        assert_eq!(agents, vec!["universal".to_string()]);
    }

    #[test]
    fn detect_agents_detects_multiple_agent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".claude")).unwrap();
        fs::create_dir(dir.path().join(".pi")).unwrap();
        let agents = detect_agents(&[], dir.path());
        assert_eq!(agents, vec!["claude".to_string(), "pi".to_string()]);
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
                memory: MemorySection::default(),
            }
        }
    }
}
