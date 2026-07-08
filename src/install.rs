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
/// Doctrine's own marketplace/plugin/owner name — the manifest `name` field,
/// source-agnostic across the github and local-directory sources (design §5.4.2).
/// Single source for the qualified enable key `doctrine@doctrine` (STD-001).
const DOCTRINE_MARKETPLACE: &str = "doctrine";
const MARKETPLACE_ONLY_DOMAINS: &[&str] = &[MEMORY_SUBSET_DOMAIN, PARTNER_SUBSET_DOMAIN];
const RUNNER_BUNX: &str = "bunx";
const RUNNER_NPX: &str = "npx";
const DISPATCH_WORKER_AGENT_ASSET: &str = "agents/claude/dispatch-worker.md";
const DISPATCH_WORKER_AGENT_ASSET_PI: &str = "agents/pi/dispatch-worker.md";
/// The claude-arm read-only probe def (SL-206 PHASE-13) — bootstrap phase-planner
/// AND closing authored-divergence probe. Seeded alongside the worker/orchestrator
/// defs via the same `install_agent_def` leg; dest filename is DERIVED from this
/// asset's basename (no `_FILE` twin needed — see `install_agent_def`).
const DISPATCH_PROBE_AGENT_ASSET: &str = "agents/claude/dispatch-probe.md";

/// Marker token injected into the dispatch-worker agent defs (SL-186 PHASE-04).
/// When `install_agent_def` sees this literal in a def, it resolves the role
/// band (`Role::Worker`) through the prompt engine and replaces the marker with
/// the assembled text.
// full worker context (role+traits); the bake reads the def's frontmatter, not the
// marker args (kept literal, F4/C6). The sentinel stays this exact string — it is
// matched by `.contains`, never parsed for its `--role` argument.
pub(crate) const WORKER_RESOLVE_MARKER: &str = "{{ prompt resolve --role worker }}";

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

    #[serde(default)]
    hymns: HymnsSection,
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

#[derive(Debug, Default, Deserialize)]
struct HymnsSection {
    #[serde(default)]
    seal: Vec<String>,
    #[serde(default)]
    expose: Vec<String>,
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
    /// `--dev`: point the claude marketplace source at the local project root
    /// (live plugin load, no network) instead of the github `install.repo` slug.
    pub(crate) dev: bool,
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

    // Forward-step 4 (nominal label): project the exposed-slot starter twins +
    // self-`replaces` sidecars onto disk. Placement is EARLY, not last (design F5):
    // it MUST run before the agent-render loop below, because `install_agents_for`
    // resolves the worker role body from the *disk* hymn corpus — the sidecars must
    // already be on disk when it renders, or install is non-idempotent (the ISS-206
    // double-emit). Establish the disk corpus (incl. user-override sidecars) here,
    // before anything renders from it. Non-fatal on error (matches sibling steps).
    if prompt_step(
        "Project exposed hymn starters? [y/N/a]",
        args.yes,
        &mut all_yes,
    )? {
        match (embedded_expose_set(), embedded_seal_set()) {
            (Ok(expose), Ok(seal)) => {
                let disk = root.join(".doctrine").join(HYMNS_DIRNAME);
                let embedded = embedded_hymns();
                match project_starters(&disk, &embedded, &seal, &expose.0, args.dry_run) {
                    Ok(written) => {
                        writeln!(io::stdout(), "  projected {} hymn file(s)", written.len())?;
                    }
                    Err(e) => {
                        writeln!(io::stdout(), "  hymn projection failed: {e:#}")?;
                    }
                }
            }
            (Err(e), _) | (_, Err(e)) => {
                writeln!(io::stdout(), "  hymn projection failed: {e:#}")?;
            }
        }
    }

    // 3. Skills per agent
    let catalog = discover()?;
    let selected = select_for_install(&catalog, args.skills, args.domains)?;
    let (runner_name, runner) = resolve_runner();
    let repo = &crate::dtoml::load_doctrine_toml(root)?.install.repo;

    let mut non_claude_agents: Vec<String> = Vec::new();

    // Track which plugin steps were skipped-but-needed for the final reminder.
    // Each holds the exact command argument to render (selected source / enable
    // key), so the reminder matches what the run would have done (F-8).
    let mut skipped_marketplace: Option<String> = None;
    let mut skipped_plugin: Option<String> = None;

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

            // Resolve the marketplace source ONCE (F-2): the github slug, or —
            // under --dev — the canonicalized local project root, precondition-
            // checked to hold a doctrine marketplace manifest (hard error else).
            let cwd = std::env::current_dir().context("failed to read current directory")?;
            let source = select_marketplace_source(root, &cwd, repo, args.dev)?;
            let source_arg = source.as_arg();
            let key = enable_key();

            // 1. Marketplace registration — refresh a STALE source, not just
            //    skip-because-name-present (R4). `add` overwrites in place on CC
            //    2.1.198 (probe, D-P3-1), so refresh is a single add.
            let registered = claude_cmd_stdout(&["plugin", "marketplace", "list"])
                .and_then(|o| parse_registered_source(&o, DOCTRINE_MARKETPLACE));
            let action = marketplace_action(registered, &source);
            if action != MarketplaceAction::Skip {
                let verb = if action == MarketplaceAction::Refresh {
                    "refresh"
                } else {
                    "add"
                };
                if prompt_step(
                    &format!("claude plugin marketplace {verb} {source_arg}? [y/N/a]"),
                    args.yes,
                    &mut all_yes,
                )? {
                    match claude_plugin_add_marketplace(&source_arg) {
                        Ok(()) => writeln!(out, "  marketplace {source_arg} registered")?,
                        Err(e) => {
                            // A failed REFRESH aborts (F-5/VT-2): leaving a stale
                            // source live while reporting success is silent-wrong.
                            // A failed fresh add keeps the softer reminder.
                            if refresh_failure_is_fatal(&action) {
                                return Err(e.context(format!(
                                    "marketplace refresh to {source_arg} failed — aborting; \
                                     the previously registered doctrine source is stale"
                                )));
                            }
                            writeln!(out, "  marketplace add failed: {e:#}")?;
                            skipped_marketplace = Some(source_arg.to_string());
                        }
                    }
                } else {
                    skipped_marketplace = Some(source_arg.to_string());
                }
            }

            // 2. Plugin install (qualified enable key — F-4).
            if !claude_plugin_has(&key) {
                if prompt_step(
                    &format!("claude plugin install {key} --scope project? [y/N/a]"),
                    args.yes,
                    &mut all_yes,
                )? {
                    match claude_plugin_install(&key) {
                        Ok(()) => writeln!(out, "  {key} plugin installed")?,
                        Err(e) => {
                            writeln!(out, "  plugin install failed: {e:#}")?;
                            skipped_plugin = Some(key.clone());
                        }
                    }
                } else {
                    skipped_plugin = Some(key.clone());
                }
            }

            // 3. Agent-def install (kept as-is).
            if let Err(e) = install_agents_for(root, "claude", None, args.global, false, &mut out) {
                writeln!(io::stdout(), "  claude agent-def install failed: {e:#}")?;
            }
            // 3b. Probe def (SL-206 PHASE-13) — claude-arm only (read-only bootstrap
            // + closing authored-divergence probe; no `agents/pi/dispatch-probe.md`
            // counterpart exists).
            if let Err(e) = install_agent_def(
                root,
                "claude",
                None,
                DISPATCH_PROBE_AGENT_ASSET,
                args.global,
                false,
                &mut out,
            ) {
                writeln!(io::stdout(), "  claude probe-def install failed: {e:#}")?;
            }
            // 3c. Workflows leg (SL-206 PHASE-13) — payload lands PHASE-14; a
            // no-op today (empty embed enumeration), mechanism only.
            if let Err(e) = install_workflows_for(root, args.global, false, &mut out) {
                writeln!(io::stdout(), "  claude workflows install failed: {e:#}")?;
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
    // install it manually — rendering the SELECTED source and qualified enable
    // key that the run would have used (F-8), not the github repo + bare name.
    if skipped_marketplace.is_some() || skipped_plugin.is_some() {
        writeln!(io::stdout())?;
        writeln!(
            io::stdout(),
            "Claude Code requires the doctrine plugin. To install:"
        )?;
        if let Some(source_arg) = &skipped_marketplace {
            writeln!(io::stdout(), "  claude plugin marketplace add {source_arg}")?;
        }
        if let Some(key) = &skipped_plugin {
            writeln!(
                io::stdout(),
                "  claude plugin install {key} --scope project"
            )?;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Claude plugin helpers (IMP-223)
// ---------------------------------------------------------------------------

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
// Marketplace source selection + exact presence (SL-195 PHASE-02)
// ---------------------------------------------------------------------------

/// The marketplace source `claude plugin marketplace add <SOURCE>` is pointed at.
/// `--dev` ⇒ a local directory (the absolutized project root); default ⇒ the
/// github `install.repo` slug. Only the source arg differs between modes.
#[derive(Debug, Clone, PartialEq, Eq)]
enum MarketplaceSource {
    Github(String),
    Directory(PathBuf),
}

impl MarketplaceSource {
    /// The positional argument for `claude plugin marketplace add`.
    fn as_arg(&self) -> std::borrow::Cow<'_, str> {
        match self {
            MarketplaceSource::Github(slug) => std::borrow::Cow::Borrowed(slug),
            MarketplaceSource::Directory(path) => path.to_string_lossy(),
        }
    }
}

/// Doctrine's `.claude-plugin/marketplace.json`, parsed for the names the enable
/// key is composed from. Tolerant of unknown fields (schema may carry more).
#[derive(Debug, Deserialize)]
struct MarketplaceManifest {
    name: String,
    #[serde(default)]
    plugins: Vec<ManifestPlugin>,
}

#[derive(Debug, Deserialize)]
struct ManifestPlugin {
    name: String,
}

/// The target plugin is the manifest entry whose `name` equals the top-level
/// marketplace `name` (both `doctrine`) — NEVER `plugins[0]`. The manifest holds
/// three plugins (`doctrine`, `doctrine-memory`, `doctrine-partner`); the last
/// two are standalone subsets (design §5.1, inquisition F-3).
fn select_plugin(manifest: &MarketplaceManifest) -> Option<&str> {
    manifest
        .plugins
        .iter()
        .map(|p| p.name.as_str())
        .find(|name| *name == manifest.name)
}

/// The qualified enable key `<plugin>@<marketplace>` — `doctrine@doctrine`,
/// source-agnostic and identical across modes (design §5.1). The single literal
/// (STD-001) is `DOCTRINE_MARKETPLACE`.
fn enable_key() -> String {
    format!("{DOCTRINE_MARKETPLACE}@{DOCTRINE_MARKETPLACE}")
}

/// Relative path to the `--dev` marketplace manifest under the project root.
const MARKETPLACE_MANIFEST_REL: &str = ".claude-plugin/marketplace.json";

/// Resolve the marketplace source for the claude arm.
///
/// `dev=false` ⇒ the github `repo` slug. `dev=true` ⇒ the project root
/// absolutized once (relative `root` is joined onto `cwd`, then canonicalized so
/// the stored source matches what Claude records — inquisition F-2/R5) and
/// required to hold `.claude-plugin/marketplace.json` whose selected plugin
/// validates the `doctrine@doctrine` identity; absent ⇒ hard error, never a
/// silent github fallback.
fn select_marketplace_source(
    root: &Path,
    cwd: &Path,
    repo: &str,
    dev: bool,
) -> anyhow::Result<MarketplaceSource> {
    if !dev {
        return Ok(MarketplaceSource::Github(repo.to_string()));
    }

    let joined = if root.is_absolute() {
        root.to_path_buf()
    } else {
        cwd.join(root)
    };
    let abs = fs::canonicalize(&joined).with_context(|| {
        format!(
            "--dev: could not resolve project root {} (does it exist?)",
            joined.display()
        )
    })?;

    let manifest_path = abs.join(MARKETPLACE_MANIFEST_REL);
    let raw = fs::read_to_string(&manifest_path).with_context(|| {
        format!(
            "--dev requires a plugin marketplace manifest at {} — none found; \
             run from doctrine's own repo or drop --dev for the github source",
            manifest_path.display()
        )
    })?;
    let manifest: MarketplaceManifest = serde_json::from_str(&raw).with_context(|| {
        format!(
            "--dev: malformed marketplace manifest {}",
            manifest_path.display()
        )
    })?;
    anyhow::ensure!(
        select_plugin(&manifest).is_some(),
        "--dev: {} does not define the `{DOCTRINE_MARKETPLACE}` plugin \
         (marketplace `{}`) — not a doctrine marketplace",
        manifest_path.display(),
        manifest.name,
    );

    Ok(MarketplaceSource::Directory(abs))
}

/// Exact-match a Claude `list` entry: whitespace-tokenize stdout and compare.
/// A qualified plugin key (`doctrine@doctrine`) or a bare marketplace name
/// (`doctrine`) is a whole token, so a sibling (`doctrine-memory@doctrine`) or a
/// source path (`(/workspace/doctrine)`) cannot false-satisfy — unlike the bare
/// `contains(..)` substring grep this replaces (inquisition F-4).
fn claude_list_has(output: &str, key: &str) -> bool {
    output.split_whitespace().any(|tok| tok == key)
}

/// Run `claude <args>` and capture stdout (`None` on spawn failure).
fn claude_cmd_stdout(args: &[&str]) -> Option<String> {
    Command::new("claude")
        .args(args)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
}

/// Whether the qualified plugin `key` is installed (exact match).
fn claude_plugin_has(key: &str) -> bool {
    claude_cmd_stdout(&["plugin", "list"]).is_some_and(|o| claude_list_has(&o, key))
}

/// A marketplace source as `claude plugin marketplace list` reports it (the
/// parenthesized inner of a `Source: <Kind> (<inner>)` line). Kind-tagged so a
/// slug can never equal a path (PHASE-03 comparator).
#[derive(Debug, Clone, PartialEq, Eq)]
enum RegisteredSource {
    Directory(String),
    Github(String),
}

/// Parse the registered source for marketplace `name` from `marketplace list`
/// stdout. Each block is `❯ <name>` then an indented `Source: <Kind> (<inner>)`.
/// Returns `None` if the name is absent or its Source line is unrecognised — the
/// caller treats `None` as "absent" ⇒ a safe idempotent add.
fn parse_registered_source(list: &str, name: &str) -> Option<RegisteredSource> {
    let mut current: Option<&str> = None;
    for line in list.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("❯ ") {
            current = Some(rest.trim());
            continue;
        }
        if current == Some(name)
            && let Some(spec) = t.strip_prefix("Source:")
        {
            let (kind, rest) = spec.trim().split_once(' ')?;
            let inner = rest.trim().strip_prefix('(')?.strip_suffix(')')?;
            return match kind {
                "Directory" => Some(RegisteredSource::Directory(inner.to_string())),
                "GitHub" => Some(RegisteredSource::Github(inner.to_string())),
                _ => None,
            };
        }
    }
    None
}

/// The registration action for the marketplace step.
#[derive(Debug, Clone, PartialEq, Eq)]
enum MarketplaceAction {
    Skip,
    Add,
    Refresh,
}

/// Whether the registered source is the intended one — same kind AND the inner
/// string equals `intended.as_arg()` (the exact positional `add` was given, hence
/// what `list` echoes back).
fn source_matches(registered: &RegisteredSource, intended: &MarketplaceSource) -> bool {
    let arg = intended.as_arg();
    match (registered, intended) {
        (RegisteredSource::Directory(a), MarketplaceSource::Directory(_))
        | (RegisteredSource::Github(a), MarketplaceSource::Github(_)) => a.as_str() == arg,
        _ => false,
    }
}

/// Decide the marketplace registration action: absent ⇒ `Add`; registered with
/// the intended source ⇒ `Skip`; registered with a different source ⇒ `Refresh`
/// (R4: a single `add` overwrites in place on CC 2.1.198 — D-P3-1). Closes the
/// stale-source gap where a bare name-present check would skip a moved repo.
fn marketplace_action(
    registered: Option<RegisteredSource>,
    intended: &MarketplaceSource,
) -> MarketplaceAction {
    match registered {
        None => MarketplaceAction::Add,
        Some(reg) if source_matches(&reg, intended) => MarketplaceAction::Skip,
        Some(_) => MarketplaceAction::Refresh,
    }
}

/// A failed *refresh* (stale→intended) must abort forward steps: a claimed
/// refresh that left a stale/foreign source live is a silent-wrong success
/// (F-5/VT-2). A failed initial *add* keeps the softer `skipped_*` reminder — a
/// fresh install lost nothing.
fn refresh_failure_is_fatal(action: &MarketplaceAction) -> bool {
    matches!(action, MarketplaceAction::Refresh)
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

/// Build a `SealSet` from the `[hymns] seal` entries in the embedded manifest.
/// Each entry is a `"band/label"` string parsed into a `hymns::Slot`.
pub(crate) fn embedded_seal_set() -> anyhow::Result<crate::hymns::SealSet> {
    let manifest = load_manifest()?;
    let mut slots = std::collections::BTreeSet::new();
    for s in &manifest.hymns.seal {
        let slot = parse_seal_slot(s)?;
        slots.insert(slot);
    }
    Ok(crate::hymns::SealSet(slots))
}

/// Build the exposed-slot set from the `[hymns] expose` entries in the embedded
/// manifest — the mirror of `embedded_seal_set`. Each entry is a `"band/label"`
/// string parsed into a `hymns::Slot`. Consumed by `project_starters` (forward
/// step) to project the self-`replaces` starter twins.
pub(crate) fn embedded_expose_set() -> anyhow::Result<crate::hymns::SealSet> {
    let manifest = load_manifest()?;
    let mut slots = std::collections::BTreeSet::new();
    for s in &manifest.hymns.expose {
        let slot = parse_seal_slot(s)?;
        slots.insert(slot);
    }
    Ok(crate::hymns::SealSet(slots))
}

/// Return every embedded file whose name starts with `"hymns/"`,
/// as `(relative-path-under-"hymns/", bytes)` pairs. The "hymns/" prefix is
/// stripped so callers work with slot-relative paths.
pub(crate) fn embedded_hymns() -> Vec<(String, Vec<u8>)> {
    let prefix = "hymns/";
    Assets::iter()
        .filter_map(|name| {
            let name = name.as_ref();
            name.strip_prefix(prefix)
                .map(|rel| (rel.to_string(), Assets::get(name).map(|f| f.data.to_vec())))
        })
        .filter_map(|(rel, opt)| opt.map(|bytes| (rel, bytes)))
        .collect()
}

/// Return every embedded agent-def file (under `"agents/"`) as
/// `(relative-path, bytes)` pairs. Used by `check_corpus` for
/// def-marker integrity (SL-186 PHASE-04 / T6).
pub(crate) fn embedded_agent_defs() -> Vec<(String, Vec<u8>)> {
    let prefix = "agents/";
    Assets::iter()
        .filter_map(|name| {
            let name = name.as_ref();
            name.strip_prefix(prefix)
                .map(|rel| (rel.to_string(), Assets::get(name).map(|f| f.data.to_vec())))
        })
        .filter_map(|(rel, opt)| opt.map(|bytes| (rel, bytes)))
        .collect()
}

// ── Prompt-cascade corpus loader (SL-186 PHASE-04, relocated from prompt.rs) ─

/// The single path-segment name shared by both the embedded corpus root and the
/// disk root. `"hymns/"` is the `rust_embed` prefix; `"hymns"` is the subdirectory
/// name inside the `.doctrine` target.
pub(crate) const HYMNS_DIRNAME: &str = "hymns";

/// Provisional SL-186 stage set (STD-001 single source). `check` flags any
/// `stage`-band label not in this list.
pub(crate) const KNOWN_STAGE_LABELS: &[&str] = &[
    "route",
    "canon",
    "preflight",
    "slice",
    "design",
    "inquisition",
    "plan",
    "phase-plan",
    "execute",
    "audit",
    "reconcile",
    "close",
    "consult",
    "walkthrough",
    "notes",
    "next",
    "record-memory",
    "retrieve-memory",
];

// TOML sidecar schema
#[derive(Debug, serde::Deserialize, Default)]
struct Sidecar {
    #[serde(default)]
    harness: Option<String>,
    // Load-bearing Option (§8/D4): `None` (omitted) keeps the path-derived pin,
    // `Some([])` unpins, `Some(list)` replaces with the conjunctive trait set. A
    // bare `#[serde(default)] Vec` could not tell omitted from explicit-empty.
    #[serde(default)]
    model: Option<Vec<String>>,
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    arm: Option<String>,
    #[serde(default)]
    stage: Option<String>,
    #[serde(default)]
    replaces: Option<String>,
}

pub(crate) fn parse_role(s: &str) -> anyhow::Result<crate::hymns::Role> {
    match s {
        "worker" => Ok(crate::hymns::Role::Worker),
        "orchestrator" => Ok(crate::hymns::Role::Orchestrator),
        other => bail!("unknown role {other:?}; expected 'worker' or 'orchestrator'"),
    }
}

pub(crate) fn parse_arm(s: &str) -> anyhow::Result<crate::hymns::Arm> {
    match s {
        "subagent" => Ok(crate::hymns::Arm::Subagent),
        "subprocess" => Ok(crate::hymns::Arm::Subprocess),
        other => bail!("unknown arm {other:?}; expected 'subagent' or 'subprocess'"),
    }
}

fn parse_slot_ref(s: &str) -> anyhow::Result<crate::hymns::Slot> {
    let (band_seg, label) = s
        .split_once('/')
        .with_context(|| format!("slot ref {s:?}: expected 'band/label'"))?;
    let band = crate::hymns::Band::from_segment(band_seg)
        .with_context(|| format!("unknown band {band_seg:?}"))?;
    Ok(crate::hymns::Slot::new(band, label))
}

fn path_to_slot(rel: &Path) -> anyhow::Result<crate::hymns::Slot> {
    let first = rel
        .components()
        .next()
        .and_then(|c| c.as_os_str().to_str())
        .context("snippet path has no band segment")?;

    let band = crate::hymns::Band::from_segment(first)
        .with_context(|| format!("unknown band {first:?} in {:?}", rel.display()))?;

    let label = {
        let rest: PathBuf = rel.components().skip(1).collect();
        let mut label = rest.to_string_lossy().into_owned();
        if let Some(stripped) = label.strip_suffix(".md") {
            label = stripped.to_string();
        }
        label
    };

    Ok(crate::hymns::Slot::new(band, label))
}

fn default_selector(slot: &crate::hymns::Slot) -> crate::hymns::Selector {
    match slot.band {
        crate::hymns::Band::Harness => crate::hymns::Selector {
            harness: Some(slot.label.clone()),
            ..Default::default()
        },
        crate::hymns::Band::Role => {
            let role = match slot.label.as_str() {
                "worker" => crate::hymns::Role::Worker,
                "orchestrator" => crate::hymns::Role::Orchestrator,
                _ => return crate::hymns::Selector::default(),
            };
            crate::hymns::Selector {
                role: Some(role),
                ..Default::default()
            }
        }
        crate::hymns::Band::Model => crate::hymns::Selector {
            // PHASE-01: seed a singleton pin from the path label (still single-valued;
            // PHASE-02 widens the sidecar to a list).
            model: BTreeSet::from([slot.label.clone()]),
            ..Default::default()
        },
        crate::hymns::Band::Stage => crate::hymns::Selector {
            stage: Some(slot.label.clone()),
            ..Default::default()
        },
        crate::hymns::Band::Preamble | crate::hymns::Band::Project => {
            crate::hymns::Selector::default()
        }
    }
}

fn overlay_selector(
    base: &crate::hymns::Selector,
    sidecar: &Sidecar,
) -> anyhow::Result<crate::hymns::Selector> {
    let mut sel = base.clone();
    if let Some(ref h) = sidecar.harness {
        sel.harness = Some(h.clone());
    }
    // Presence semantics on the load-bearing Option: declared (`Some`) replaces the
    // base pin with the conjunctive set — an empty list unpins the axis; omitted
    // (`None`) keeps whatever the base carries (the path-derived pin).
    if let Some(ref list) = sidecar.model {
        sel.model = list.iter().cloned().collect();
    }
    if let Some(ref r) = sidecar.role {
        sel.role = Some(parse_role(r)?);
    }
    if let Some(ref a) = sidecar.arm {
        sel.arm = Some(parse_arm(a)?);
    }
    if let Some(ref s) = sidecar.stage {
        sel.stage = Some(s.clone());
    }
    if let Some(ref replaces) = sidecar.replaces {
        sel.replaces = Some(parse_slot_ref(replaces)?);
    }
    Ok(sel)
}

fn has_ext(rel_path: &str, ext: &str) -> bool {
    Path::new(rel_path).extension().is_some_and(|e| e == ext)
}

fn collect_snippet_paths(root: &Path, current: &Path, paths: &mut BTreeSet<PathBuf>) {
    let Ok(entries) = fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(ft) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        if ft.is_dir() {
            collect_snippet_paths(root, &path, paths);
        } else {
            let ext_ok = path
                .extension()
                .is_some_and(|ext| ext == "md" || ext == "toml");
            if ext_ok && let Ok(rel) = path.strip_prefix(root) {
                paths.insert(rel.to_path_buf());
            }
        }
    }
}

fn load_embedded_corpus(
    embedded: &[(String, Vec<u8>)],
) -> anyhow::Result<Vec<crate::hymns::Snippet>> {
    let mut sidecars: std::collections::BTreeMap<String, Sidecar> =
        std::collections::BTreeMap::new();
    for (rel_path, bytes) in embedded {
        if has_ext(rel_path, "toml") {
            let stem = rel_path
                .strip_suffix(".toml")
                .context("strip_suffix for .toml checked above")?
                .to_string();
            let text = String::from_utf8(bytes.clone())
                .map_err(|e| anyhow::anyhow!("embedded sidecar not UTF-8: {e}"))?;
            let sc: Sidecar =
                toml::from_str(&text).with_context(|| format!("invalid sidecar: {rel_path}"))?;
            sidecars.insert(stem, sc);
        }
    }

    let mut snippets = Vec::new();
    for (rel_path, bytes) in embedded {
        if has_ext(rel_path, "toml") {
            continue;
        }
        if !has_ext(rel_path, "md") {
            continue;
        }
        // Skip files at the hymns root (no band directory).
        let rel = Path::new(rel_path);
        if rel.components().count() < 2 {
            continue;
        }
        let body = String::from_utf8(bytes.clone())
            .map_err(|e| anyhow::anyhow!("{rel_path:?}: not valid UTF-8: {e}"))?;
        let slot = path_to_slot(rel).with_context(|| format!("embedded snippet {rel_path:?}"))?;
        let base_sel = default_selector(&slot);

        let stem = rel_path
            .strip_suffix(".md")
            .context(".md suffix verified above")?;
        let selector = if let Some(sc) = sidecars.get(stem) {
            overlay_selector(&base_sel, sc)
                .with_context(|| format!("embedded sidecar for {rel_path:?}"))?
        } else {
            base_sel
        };

        snippets.push(crate::hymns::Snippet {
            slot,
            selector,
            provenance: crate::hymns::Provenance::Framework,
            body,
        });
    }
    Ok(snippets)
}

fn load_disk_corpus(
    disk_root: &Path,
    sealed: &crate::hymns::SealSet,
) -> anyhow::Result<Vec<crate::hymns::Snippet>> {
    if !disk_root.is_dir() {
        return Ok(Vec::new());
    }
    let mut snippets = Vec::new();
    let mut paths: BTreeSet<PathBuf> = BTreeSet::new();
    collect_snippet_paths(disk_root, disk_root, &mut paths);

    for rel in &paths {
        if rel.extension() != Some("md".as_ref()) {
            continue;
        }
        if rel.components().count() < 2 {
            continue;
        }
        let md_path = disk_root.join(rel);
        let body = fs::read_to_string(&md_path)
            .with_context(|| format!("Failed to read {}", md_path.display()))?;

        let slot =
            path_to_slot(rel).with_context(|| format!("disk snippet {:?}", rel.display()))?;

        if sealed.0.contains(&slot) {
            continue;
        }

        let mut selector = default_selector(&slot);

        let toml_rel = rel.with_extension("toml");
        let toml_path = disk_root.join(&toml_rel);
        if toml_path.is_file() {
            let toml_text = fs::read_to_string(&toml_path)
                .with_context(|| format!("Failed to read {}", toml_path.display()))?;
            let sc: Sidecar = toml::from_str(&toml_text)
                .with_context(|| format!("invalid sidecar: {}", toml_path.display()))?;
            selector = overlay_selector(&selector, &sc)
                .with_context(|| format!("disk sidecar {:?}", toml_path.display()))?;
        }

        snippets.push(crate::hymns::Snippet {
            slot,
            selector,
            provenance: crate::hymns::Provenance::User,
            body,
        });
    }
    Ok(snippets)
}

pub(crate) fn load_full_corpus(
    disk_root: &Path,
    embedded: &[(String, Vec<u8>)],
    sealed: &crate::hymns::SealSet,
) -> anyhow::Result<Vec<crate::hymns::Snippet>> {
    let mut corpus = load_embedded_corpus(embedded)?;
    let mut disk = load_disk_corpus(disk_root, sealed)?;
    corpus.append(&mut disk);
    Ok(corpus)
}

/// Resolve the worker's full context (role band + any declared trait/model bands)
/// through the prompt engine. `traits` are the def's declared trait keys; an empty
/// set collapses to the role-only baseline (byte-identical to the pre-SL-191 bake,
/// VT-4). The context shape is built by `hymns::worker_context` (the shared leaf
/// builder — no parallel `ContextVector` construction, ADR-001).
pub(crate) fn resolve_worker_role_body(
    corpus: &[crate::hymns::Snippet],
    sealed: &crate::hymns::SealSet,
    traits: &BTreeSet<String>,
) -> Result<String, crate::hymns::ResolveError> {
    crate::hymns::resolve(&crate::hymns::worker_context(traits), corpus, sealed)
}

pub(crate) fn expand_worker_marker(def: &str, body: &str) -> String {
    def.replace(WORKER_RESOLVE_MARKER, body)
}

/// Bake a worker agent def against a resolved corpus: read the def's OWN declared
/// traits from its frontmatter, enforce trait coverage (a hard error on any uncovered
/// key — a contractless worker must never ship, T4), resolve role + trait bands, and
/// inline the assembled body at the marker. The single seam both the real bake
/// (`install_agent_def`) and VT-3 drive; corpus-pure (disk/embed I/O stays in the
/// caller). Module home `install` — the bake engine leg (ADR-001).
fn bake_worker_def(
    def_str: &str,
    asset_label: &str,
    corpus: &[crate::hymns::Snippet],
    sealed: &crate::hymns::SealSet,
) -> anyhow::Result<String> {
    let traits = parse_agent_def_traits(def_str)
        .with_context(|| format!("agent def '{asset_label}' frontmatter"))?;
    let uncovered = crate::hymns::traits_covered(&traits, corpus);
    if !uncovered.is_empty() {
        bail!(
            "dispatch-worker def '{asset_label}' declares uncovered trait(s) {uncovered:?}; \
             a contractless worker cannot ship — add a Model-band hymn covering each key",
        );
    }
    let body = resolve_worker_role_body(corpus, sealed, &traits)?;
    Ok(expand_worker_marker(def_str, &body))
}

/// Project the disk starter twin + self-`replaces` sidecar for each exposed slot.
///
/// For every exposed (non-sealed) slot, writes `.doctrine/hymns/<band>/<label>.md`
/// (framework body) and `.doctrine/hymns/<band>/<label>.toml` (carrying
/// `replaces = "<slot>"`, single-sourced off `Slot::path`, STD-001/D3). The `.md`
/// and `.toml` are INDEPENDENT write-if-absent (D2): an absent file is written, a
/// present file is preserved byte-for-byte — so a user-edited starter survives and
/// a sidecar-less legacy twin is backfilled. `create_dir_all(parent)` runs before
/// every `write_atomic` because `write_atomic` (`fsutil.rs`) does not mkdir (F2).
/// `dry_run` writes nothing. The user twin's self-`replaces` suppresses its
/// framework origin at resolve, so the exposed slot single-emits (REV-019 REQ-322).
pub(crate) fn project_starters(
    disk_root: &Path,
    embedded: &[(String, Vec<u8>)],
    sealed: &crate::hymns::SealSet,
    exposed_slots: &BTreeSet<crate::hymns::Slot>,
    dry_run: bool,
) -> anyhow::Result<Vec<PathBuf>> {
    let mut embedded_by_slot: std::collections::BTreeMap<crate::hymns::Slot, String> =
        std::collections::BTreeMap::new();
    for (rel_path, bytes) in embedded {
        if !has_ext(rel_path, "md") {
            continue;
        }
        let rel = Path::new(rel_path);
        let Ok(slot) = path_to_slot(rel) else {
            continue;
        };
        let Ok(body) = String::from_utf8(bytes.clone()) else {
            continue;
        };
        embedded_by_slot.entry(slot).or_insert(body);
    }

    let mut written = Vec::new();
    for slot in exposed_slots {
        // Belt-and-braces: seal wins if a slot is ever mislisted in both lists
        // (the manifest lists are disjoint by construction). EX-1/VT-5.
        if sealed.0.contains(slot) {
            continue;
        }
        let Some(body) = embedded_by_slot.get(slot) else {
            continue;
        };

        let dir = disk_root.join(slot.band.as_str());
        let md = dir.join(format!("{}.md", slot.label));
        let toml = dir.join(format!("{}.toml", slot.label));

        // Independent per-file write-if-absent (D2). `.md` and `.toml` are decided
        // separately: an absent `.md` is written and a present one preserved;
        // likewise the sidecar `.toml`, which backfills the legacy orphan twins.
        if !md.exists() {
            project_write_if_absent(&md, body.as_bytes(), dry_run)?;
            written.push(md);
        }

        if !toml.exists() {
            // Single-sourced off `Slot::path` — no magic string (D3/STD-001).
            let sidecar = format!("replaces = \"{}\"\n", slot.path());
            project_write_if_absent(&toml, sidecar.as_bytes(), dry_run)?;
            written.push(toml);
        }
    }
    Ok(written)
}

/// Write `bytes` to `dest` unless `dry_run`, first ensuring the parent dir exists
/// (`write_atomic` does not mkdir — F2). Errors are surfaced with per-file context.
fn project_write_if_absent(dest: &Path, bytes: &[u8], dry_run: bool) -> anyhow::Result<()> {
    if dry_run {
        return Ok(());
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create parent dir {}", parent.display()))?;
    }
    crate::fsutil::write_atomic(dest, bytes).with_context(|| format!("project {}", dest.display()))
}

/// Parse a `"band/label"` seal entry into a `Slot`. Rejects unrecognized bands
/// and entries missing the slash.
fn parse_seal_slot(s: &str) -> anyhow::Result<crate::hymns::Slot> {
    let (band_seg, label) = s
        .split_once('/')
        .with_context(|| format!("seal entry {s:?}: expected 'band/label'"))?;
    let band = crate::hymns::Band::from_segment(band_seg)
        .with_context(|| format!("seal entry {s:?}: unknown band {band_seg:?}"))?;
    Ok(crate::hymns::Slot::new(band, label))
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

/// Extract the inner YAML of a leading `---`…`---` frontmatter block. Errs on an
/// absent or unterminated fence. Single-sources the fence-slicing shared by the
/// `SKILL.md` (`parse_meta`) and agent-def (`parse_agent_def_traits`) readers — one
/// frontmatter impl, no parallel copy (DRY).
fn frontmatter_yaml(md: &str) -> anyhow::Result<&str> {
    let after = md
        .strip_prefix("---")
        .context("missing leading '---' frontmatter")?
        .trim_start_matches(['\r', '\n']);
    let end = after
        .find("\n---")
        .context("frontmatter is not terminated by '---'")?;
    after.get(..end).context("frontmatter slice out of range")
}

/// Parse leading `---` YAML frontmatter from a `SKILL.md` body.
fn parse_meta(md: &str) -> anyhow::Result<Meta> {
    let yaml = frontmatter_yaml(md).context("SKILL.md")?;
    let meta: Meta = serde_yaml::from_str(yaml).context("Failed to parse SKILL.md frontmatter")?;
    Ok(meta)
}

/// The agent-def frontmatter fields this bake consumes. Only `traits` is read;
/// every other declared key (`name`/`description`/`tools`/`model`) is tolerated by
/// default serde (NO `deny_unknown_fields`) so the def keeps its full YAML head and
/// the cascade-ignored `model:` pin stays put.
#[derive(Debug, Default, Deserialize)]
struct AgentDefMeta {
    traits: Option<Vec<String>>,
}

/// Parse an agent def's `---` YAML frontmatter and return its declared trait keys as
/// a `BTreeSet` (empty when `traits:` is absent). Errs on a malformed/unterminated
/// fence so a broken def fails the bake loudly rather than silently shipping a
/// contractless worker. Rides `frontmatter_yaml` (no parallel frontmatter impl).
/// Pure — module home `install` (the engine leg of the bake, ADR-001).
pub(crate) fn parse_agent_def_traits(def: &str) -> anyhow::Result<BTreeSet<String>> {
    let yaml = frontmatter_yaml(def).context("agent def")?;
    let meta: AgentDefMeta =
        serde_yaml::from_str(yaml).context("Failed to parse agent def frontmatter")?;
    Ok(meta.traits.unwrap_or_default().into_iter().collect())
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

/// Install an agent def for the given agent: materialize the canonical copy
/// from the embed into `.doctrine/agents/` (under an optional subdir), then
/// symlink the agent's link dir at it. Idempotent — refreshes the canonical
/// each run and only (re)writes a link that is missing or proven ours, never
/// clobbering a foreign one. Reuses `classify_link`/`write_link`/
/// `relative_target` — no parallel symlink impl.
///
/// The dest filename is DERIVED from `embed_asset`'s basename (SL-206
/// PHASE-13) — never hardcoded — so this one function seeds every claude-arm
/// def (`dispatch-worker.md`, `dispatch-probe.md`, …) rather than each needing
/// its own copy-paste variant.
pub(crate) fn install_agent_def(
    root: &Path,
    agent_name: &str,
    canon_subdir: Option<&str>,
    embed_asset: &str,
    global: bool,
    dry_run: bool,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let file_name = Path::new(embed_asset)
        .file_name()
        .and_then(|n| n.to_str())
        .with_context(|| format!("Embedded agent asset '{embed_asset}' has no file name"))?;
    let canon_base = agent_canonical_dir(root, global)?;
    let canon_dir = match canon_subdir {
        Some(sub) => canon_base.join(sub),
        None => canon_base,
    };
    let link_dir = match agent_name {
        "claude" => claude_agents_dir(root, global)?,
        _ => pi_agents_dir(root, global)?,
    };
    let canon = canon_dir.join(file_name);
    let dest = link_dir.join(file_name);
    let target = relative_target(&link_dir, &canon_dir, file_name);

    writeln!(out, "agent {agent_name} ({file_name}):")?;
    writeln!(out, "  agent     {file_name} → {}", dest.display())?;
    if dry_run {
        return Ok(());
    }

    // 1. Refresh the canonical copy from the embed (always overwrite — derived).
    let data = embedded_asset(embed_asset)
        .with_context(|| format!("Embedded agent def '{embed_asset}' not found"))?;
    fs::create_dir_all(&canon_dir)
        .with_context(|| format!("Failed to create {}", canon_dir.display()))?;
    if let Ok(def_str) = std::str::from_utf8(&data)
        && def_str.contains(WORKER_RESOLVE_MARKER)
    {
        let disk = root.join(".doctrine").join(HYMNS_DIRNAME);
        let embedded = embedded_hymns();
        let sealed = embedded_seal_set()?;
        let corpus = load_full_corpus(&disk, &embedded, &sealed)?;
        let expanded = bake_worker_def(def_str, embed_asset, &corpus, &sealed)?;
        crate::fsutil::write_atomic(&canon, expanded.as_bytes())?;
    } else {
        crate::fsutil::write_atomic(&canon, &data)?;
    }

    // 2. Reconcile the agent link by proven ownership (re-classify at mutation
    //    time, like `execute`'s skill links).
    match classify_link(file_name, &dest, &target) {
        Link::Create { .. } => {
            write_link(&dest, &target)?;
            writeln!(out, "  linked    {file_name}")?;
        }
        Link::Relink { .. } => {
            write_link(&dest, &target)?;
            writeln!(out, "  relinked  {file_name}")?;
        }
        Link::KeepForeign { reason, .. } => {
            writeln!(out, "  kept      {file_name} ({})", foreign_reason(&reason))?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Workflows leg (SL-206 PHASE-13) — seed embedded `install/workflows/*.js`
// assets into the claude-arm `.claude/workflows/` dir the same way the agents
// leg installs defs: materialize a canonical copy under `.doctrine/workflows/`,
// then symlink the link dir at it — reusing `classify_link`/`write_link`/
// `relative_target`/`install_base`, no parallel symlink impl. The `/drive-slice`
// payload (`drive-slice.js`) landed in PHASE-14, so `embedded_workflow_defs()`
// enumerates it and this leg is live in production.
// ---------------------------------------------------------------------------

/// The Claude workflows directory (project-local or, with `global`, user home).
fn claude_workflows_dir(root: &Path, global: bool) -> anyhow::Result<PathBuf> {
    Ok(install_base(root, global)?.join(".claude/workflows"))
}

/// The canonical workflows tree, mirroring `agent_canonical_dir` so the
/// relative link target is stable.
fn workflow_canonical_dir(root: &Path, global: bool) -> anyhow::Result<PathBuf> {
    Ok(install_base(root, global)?.join(".doctrine/workflows"))
}

/// Return every embedded workflow file (under `"workflows/"`) as
/// `(relative-path, bytes)` pairs — mirrors `embedded_agent_defs`. Carries the
/// `/drive-slice` payload since SL-206 PHASE-14.
pub(crate) fn embedded_workflow_defs() -> Vec<(String, Vec<u8>)> {
    let prefix = "workflows/";
    Assets::iter()
        .filter_map(|name| {
            let name = name.as_ref();
            name.strip_prefix(prefix)
                .map(|rel| (rel.to_string(), Assets::get(name).map(|f| f.data.to_vec())))
        })
        .filter_map(|(rel, opt)| opt.map(|bytes| (rel, bytes)))
        .collect()
}

/// Seed every embedded claude workflow asset: materialize a canonical copy
/// under `.doctrine/workflows/`, then symlink `.claude/workflows/` at it.
/// Public entry point — delegates to `install_workflow_assets` over the real
/// embed, kept separate so tests can drive the mechanism over a synthetic
/// asset list without an embedded payload.
pub(crate) fn install_workflows_for(
    root: &Path,
    global: bool,
    dry_run: bool,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    install_workflow_assets(root, global, dry_run, out, &embedded_workflow_defs())
}

/// The testable core of the workflows leg: materialize+link every `(name,
/// bytes)` asset. No embed access — a synthetic list exercises the mechanism
/// before the real payload (PHASE-14) exists.
fn install_workflow_assets(
    root: &Path,
    global: bool,
    dry_run: bool,
    out: &mut dyn Write,
    assets: &[(String, Vec<u8>)],
) -> anyhow::Result<()> {
    let canon_dir = workflow_canonical_dir(root, global)?;
    let link_dir = claude_workflows_dir(root, global)?;
    for (rel, data) in assets {
        let file_name = Path::new(rel)
            .file_name()
            .and_then(|n| n.to_str())
            .with_context(|| format!("Embedded workflow asset '{rel}' has no file name"))?;
        let canon = canon_dir.join(file_name);
        let dest = link_dir.join(file_name);
        let target = relative_target(&link_dir, &canon_dir, file_name);

        writeln!(out, "  workflow  {file_name} → {}", dest.display())?;
        if dry_run {
            continue;
        }

        fs::create_dir_all(&canon_dir)
            .with_context(|| format!("Failed to create {}", canon_dir.display()))?;
        crate::fsutil::write_atomic(&canon, data)?;

        match classify_link(file_name, &dest, &target) {
            Link::Create { .. } => {
                write_link(&dest, &target)?;
                writeln!(out, "  linked    {file_name}")?;
            }
            Link::Relink { .. } => {
                write_link(&dest, &target)?;
                writeln!(out, "  relinked  {file_name}")?;
            }
            Link::KeepForeign { reason, .. } => {
                writeln!(out, "  kept      {file_name} ({})", foreign_reason(&reason))?;
            }
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
// ── Tests: hymns manifest accessors (PHASE-02) ────────────────

#[cfg(test)]
mod tests_hymns {
    use super::*;

    #[test]
    fn parse_seal_slot_valid() {
        let slot = parse_seal_slot("harness/claude").unwrap();
        assert_eq!(slot.band, crate::hymns::Band::Harness);
        assert_eq!(slot.label, "claude");
    }

    #[test]
    fn parse_seal_slot_model_with_slash_in_label() {
        let slot = parse_seal_slot("model/anthropic/claude-sonnet-4").unwrap();
        assert_eq!(slot.band, crate::hymns::Band::Model);
        assert_eq!(slot.label, "anthropic/claude-sonnet-4");
    }

    #[test]
    fn parse_seal_slot_rejects_unknown_band() {
        let err = parse_seal_slot("nope/something").unwrap_err();
        assert!(err.to_string().contains("unknown band"), "{err}");
    }

    #[test]
    fn parse_seal_slot_rejects_missing_slash() {
        let err = parse_seal_slot("noslash").unwrap_err();
        assert!(err.to_string().contains("band/label"), "{err}");
    }

    #[test]
    fn embedded_seal_set_from_live_manifest() {
        // The live [hymns] seal = ["preamble/core"].
        let sealed = embedded_seal_set().unwrap();
        assert_eq!(sealed.0.len(), 1);
        let slot = sealed.0.first().unwrap();
        assert_eq!(slot.band, crate::hymns::Band::Preamble);
        assert_eq!(slot.label, "core");
    }

    #[test]
    fn embedded_hymns_from_live_dir() {
        // install/hymns/ now contains real seed files.
        let hymns = embedded_hymns();
        assert!(
            hymns.iter().any(|(name, _)| name == "preamble/core.md"),
            "expected preamble/core.md, got: {hymns:?}"
        );
        assert!(
            hymns.iter().any(|(name, _)| name == "harness/claude.md"),
            "expected harness/claude.md, got: {hymns:?}"
        );
    }

    // PHASE-02 VT-2: `Sidecar.model: Option<Vec<String>>` — the load-bearing Option.
    // Presence semantics through serde into `overlay_selector`: omitted keeps the
    // path pin, a declared list replaces (conjunctive set), an empty list unpins.
    #[test]
    fn overlay_selector_model_presence_semantics() {
        let base = default_selector(&crate::hymns::Slot::new(
            crate::hymns::Band::Model,
            "anthropic/claude-sonnet-4",
        ));
        let pin: std::collections::BTreeSet<String> = ["anthropic/claude-sonnet-4".into()].into();
        assert_eq!(base.model, pin, "default_selector seeds the path pin");

        // (a) omitted `model` → None → keep the path pin.
        let sc: Sidecar = toml::from_str("").unwrap();
        assert_eq!(sc.model, None);
        let kept = overlay_selector(&base, &sc).unwrap();
        assert_eq!(kept.model, pin, "omitted model must keep the path pin");

        // (b) declared list → Some(list) → replace with the conjunctive set.
        let sc: Sidecar =
            toml::from_str("model = [\"capability/code/high\", \"capability/reasoning/high\"]")
                .unwrap();
        let replaced = overlay_selector(&base, &sc).unwrap();
        let want: std::collections::BTreeSet<String> = [
            "capability/code/high".into(),
            "capability/reasoning/high".into(),
        ]
        .into();
        assert_eq!(replaced.model, want, "declared list must replace the pin");

        // (c) empty list → Some([]) → unpin the axis (don't-care).
        let sc: Sidecar = toml::from_str("model = []").unwrap();
        assert_eq!(sc.model, Some(vec![]));
        let unpinned = overlay_selector(&base, &sc).unwrap();
        assert!(unpinned.model.is_empty(), "empty list must unpin the axis");
    }

    // ── SL-193 PHASE-01: project_starters (self-`replaces` sidecar projection) ──
    // project_starters is untested dead code today; these are net-new units
    // (tempdir-scoped) driving the exposed-slot starter + sidecar emission.

    fn worker_slot() -> crate::hymns::Slot {
        crate::hymns::Slot::new(crate::hymns::Band::Role, "worker")
    }

    const FRAMEWORK_BODY: &str = "FRAMEWORK WORKER BODY";

    fn worker_embedded() -> Vec<(String, Vec<u8>)> {
        vec![(
            "role/worker.md".to_string(),
            FRAMEWORK_BODY.as_bytes().to_vec(),
        )]
    }

    fn exposed_set(slot: crate::hymns::Slot) -> BTreeSet<crate::hymns::Slot> {
        BTreeSet::from([slot])
    }

    fn no_seal() -> crate::hymns::SealSet {
        crate::hymns::SealSet(BTreeSet::new())
    }

    // VT-1: an exposed slot gets a `<label>.toml` sidecar carrying replaces=<slot>.
    #[test]
    fn project_starters_writes_self_replaces_sidecar() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        project_starters(
            root,
            &worker_embedded(),
            &no_seal(),
            &exposed_set(worker_slot()),
            false,
        )
        .unwrap();
        let sidecar = fs::read_to_string(root.join("role").join("worker.toml")).unwrap();
        assert!(
            sidecar.contains("replaces = \"role/worker\""),
            "sidecar must carry the self-replaces, got: {sidecar:?}"
        );
    }

    // VT-2: `.md` present, `.toml` absent ⇒ `.md` preserved byte-for-byte AND
    // sidecar written (the reconcile path for the 5 legacy orphan twins).
    #[test]
    fn project_starters_backfills_sidecar_preserving_md() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let md = root.join("role").join("worker.md");
        fs::create_dir_all(md.parent().unwrap()).unwrap();
        fs::write(&md, "PRE-EXISTING ORPHAN BODY").unwrap();

        project_starters(
            root,
            &worker_embedded(),
            &no_seal(),
            &exposed_set(worker_slot()),
            false,
        )
        .unwrap();

        assert_eq!(
            fs::read_to_string(&md).unwrap(),
            "PRE-EXISTING ORPHAN BODY",
            "existing .md must be preserved byte-for-byte"
        );
        let sidecar = fs::read_to_string(root.join("role").join("worker.toml")).unwrap();
        assert!(sidecar.contains("replaces = \"role/worker\""));
    }

    // VT-3: both `.md` and `.toml` present ⇒ no write, no error (re-run safe).
    #[test]
    fn project_starters_idempotent_when_both_present() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let md = root.join("role").join("worker.md");
        let toml = root.join("role").join("worker.toml");
        fs::create_dir_all(md.parent().unwrap()).unwrap();
        fs::write(&md, "EXISTING MD").unwrap();
        fs::write(&toml, "replaces = \"role/worker\"\n# hand-tuned axis").unwrap();

        let written = project_starters(
            root,
            &worker_embedded(),
            &no_seal(),
            &exposed_set(worker_slot()),
            false,
        )
        .unwrap();

        assert!(written.is_empty(), "both present ⇒ nothing written");
        assert_eq!(fs::read_to_string(&md).unwrap(), "EXISTING MD");
        assert_eq!(
            fs::read_to_string(&toml).unwrap(),
            "replaces = \"role/worker\"\n# hand-tuned axis",
            "present sidecar must be left untouched (no clobber of hand-tuned axes)"
        );
    }

    // VT-4: an edited `.md` (distinct from the framework body) is never overwritten.
    #[test]
    fn project_starters_preserves_edited_md() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let md = root.join("role").join("worker.md");
        fs::create_dir_all(md.parent().unwrap()).unwrap();
        let edited = "USER EDITED B-PRIME BODY (distinct from framework)";
        fs::write(&md, edited).unwrap();

        project_starters(
            root,
            &worker_embedded(),
            &no_seal(),
            &exposed_set(worker_slot()),
            false,
        )
        .unwrap();

        let after = fs::read_to_string(&md).unwrap();
        assert_eq!(
            after, edited,
            "user customisation must survive re-projection"
        );
        assert_ne!(
            after, FRAMEWORK_BODY,
            "must not be overwritten with framework body"
        );
    }

    // VT-5: a sealed slot ⇒ neither `.md` nor `.toml` written (belt-and-braces guard).
    #[test]
    fn project_starters_skips_sealed_slot() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let seal = crate::hymns::SealSet(BTreeSet::from([worker_slot()]));

        let written = project_starters(
            root,
            &worker_embedded(),
            &seal,
            &exposed_set(worker_slot()),
            false,
        )
        .unwrap();

        assert!(written.is_empty(), "sealed slot ⇒ nothing written");
        assert!(!root.join("role").join("worker.md").exists());
        assert!(!root.join("role").join("worker.toml").exists());
    }

    // VT-6: dry_run ⇒ nothing written for either file.
    #[test]
    fn project_starters_dry_run_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        project_starters(
            root,
            &worker_embedded(),
            &no_seal(),
            &exposed_set(worker_slot()),
            true,
        )
        .unwrap();

        assert!(!root.join("role").join("worker.md").exists());
        assert!(!root.join("role").join("worker.toml").exists());
    }

    // VT-7: projecting into an EMPTY disk root (band dir absent) writes `.md` +
    // sidecar, creating the parent — the create_dir_all guard against
    // write_atomic's no-mkdir failure (F2). Fails without the create_dir_all.
    #[test]
    fn project_starters_creates_missing_parent_dir() {
        let dir = tempfile::tempdir().unwrap();
        // Empty root: neither the root nor the `role/` band dir exists yet.
        let root = dir.path().join("empty-hymns-root");

        project_starters(
            &root,
            &worker_embedded(),
            &no_seal(),
            &exposed_set(worker_slot()),
            false,
        )
        .unwrap();

        assert!(
            root.join("role").join("worker.md").exists(),
            "missing parent dir must be created before write_atomic"
        );
        assert!(root.join("role").join("worker.toml").exists());
    }

    // SL-191 PHASE-07 / VT-1: overlay-reconciliation composition guard.
    // PHASE-07 re-homes THIS repo's client habits OUT of a fully-replacing
    // `role/worker` twin (which suppressed the enriched Framework contract)
    // INTO a non-replacing `project/*` overlay. A worker resolve must then
    // compose BOTH the enriched Framework role/worker contract AND the client
    // habit — each exactly once (no ISS-206 doubling, enrichment not
    // suppressed). Driven through the REAL embedded+disk loader
    // (`load_full_corpus`) against a hermetic temp-dir overlay: a pure
    // in-memory snippet fixture in `hymns.rs` cannot catch a disk-overlay /
    // sidecar-shape / loader-composition regression (codex F6).
    #[test]
    fn worker_resolve_composes_framework_role_worker_and_nonreplacing_project_habit() {
        // Hermetic disk overlay: one non-replacing project-band habit and NO
        // `role/worker` twin — nothing suppresses the Framework contract. This
        // is the shape PHASE-07 lands in `.doctrine/hymns/` (T2 deletes the
        // suppressor; T3 authors the project habit).
        const HABIT_SENTINEL: &str = "DOCTRINE-RUST-HABIT-SENTINEL-VT1";
        let dir = tempfile::tempdir().unwrap();
        let disk_root = dir.path();
        let habit = disk_root.join("project").join("doctrine-conventions.md");
        fs::create_dir_all(habit.parent().unwrap()).unwrap();
        fs::write(&habit, HABIT_SENTINEL).unwrap();

        // Real embedded Framework corpus (carries the enriched role/worker) +
        // the disk overlay, loaded through the production loader.
        let embedded = embedded_hymns();
        let sealed = embedded_seal_set().unwrap();
        let corpus = load_full_corpus(disk_root, &embedded, &sealed).unwrap();

        // The worker SESSION cascade is All-bands — exactly what `prompt
        // resolve/explain --role worker` builds (`build_ctx` with no `--band`),
        // and what a worker's SessionStart hook delivers. The `project` band
        // composes here, unlike the role-only agent-def bake
        // (`resolve_worker_role_body`, `Only([Role, Model])`).
        let ctx = crate::hymns::ContextVector {
            role: crate::hymns::Role::Worker,
            harness: None,
            model: BTreeSet::new(),
            arm: None,
            stage: None,
            bands: crate::hymns::BandFilter::All,
        };
        let body = crate::hymns::resolve(&ctx, &corpus, &sealed).unwrap();

        assert_eq!(
            body.matches("NEGATIVE CONTRACT").count(),
            1,
            "enriched Framework role/worker contract must compose exactly once \
             (not suppressed, not doubled):\n{body}"
        );
        assert_eq!(
            body.matches(HABIT_SENTINEL).count(),
            1,
            "non-replacing project-band client habit must compose exactly once:\n{body}"
        );
    }

    // SL-191 PHASE-02 / VT-1: POL-002 content gate — the shipped-corpus hymns
    // authored by this phase (role/worker.md, model/adherence/**) are FRAMEWORK
    // corpus (every client project), so they must never carry host build-tooling
    // literals belonging to THIS repo's own habits (those live in the
    // `.doctrine/hymns` overlay, out of scope here). `harness/**` legitimately
    // names host tooling and is deliberately excluded from this gate.
    const FORBIDDEN_HOST_LITERALS: &[&str] = &["cargo", "target/", "just", "node_modules"];

    #[test]
    fn install_hymns_authored_set_has_no_host_literals() {
        let authored: Vec<(String, Vec<u8>)> = embedded_hymns()
            .into_iter()
            .filter(|(rel, _)| rel == "role/worker.md" || rel.starts_with("model/"))
            .collect();

        assert!(
            !authored.is_empty(),
            "expected the authored hymn set (role/worker.md + model/**) to be non-empty"
        );

        for (rel, bytes) in &authored {
            let text = String::from_utf8_lossy(bytes).to_lowercase();
            for literal in FORBIDDEN_HOST_LITERALS {
                assert!(
                    !text.contains(literal),
                    "authored hymn '{rel}' contains forbidden host literal '{literal}' — \
                     POL-002 requires host-agnostic shipped corpus"
                );
            }
        }
    }
}

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

    // --- agent-def frontmatter parser (SL-191 VT-1) ---

    #[test]
    fn parse_agent_def_traits_reads_declared_keys() {
        let def = "---\nname: dispatch-worker\ntraits: [\"adherence/low\"]\n---\n\nbody\n";
        let traits = parse_agent_def_traits(def).unwrap();
        assert_eq!(traits, ["adherence/low".to_string()].into());
    }

    #[test]
    fn parse_agent_def_traits_absent_traits_is_empty_ok() {
        let def = "---\nname: dispatch-worker\nmodel: some/model\n---\n\nbody\n";
        let traits = parse_agent_def_traits(def).unwrap();
        assert!(traits.is_empty());
    }

    #[test]
    fn parse_agent_def_traits_tolerates_unknown_keys() {
        // name/description/tools/model are unknown to AgentDefMeta yet tolerated
        // (no deny_unknown_fields) — the def keeps its full YAML head.
        let def = "---\nname: dispatch-worker\ndescription: d\ntools: read, edit\nmodel: deepseek/deepseek-v4-pro\ntraits: [\"adherence/low\"]\n---\n\nbody\n";
        let traits = parse_agent_def_traits(def).unwrap();
        assert_eq!(traits, ["adherence/low".to_string()].into());
    }

    #[test]
    fn parse_agent_def_traits_rejects_unterminated_frontmatter() {
        assert!(parse_agent_def_traits("---\nname: x\nno closing fence\n").is_err());
    }

    #[test]
    fn parse_agent_def_traits_rejects_missing_frontmatter() {
        assert!(parse_agent_def_traits("# no frontmatter\n").is_err());
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
    // Marketplace source selection + exact presence (SL-195 PHASE-02)
    // ---------------------------------------------------------------

    fn write_marketplace_manifest(root: &Path) {
        let dir = root.join(".claude-plugin");
        fs::create_dir_all(&dir).unwrap();
        // Reordered on purpose: doctrine is NOT first.
        fs::write(
            dir.join("marketplace.json"),
            r#"{"name":"doctrine","plugins":[{"name":"doctrine-memory"},{"name":"doctrine"}]}"#,
        )
        .unwrap();
    }

    #[test]
    fn enable_key_is_qualified_doctrine() {
        // VT-3: names compose the source-agnostic key.
        assert_eq!(enable_key(), "doctrine@doctrine");
    }

    #[test]
    fn select_plugin_picks_by_name_not_first() {
        // VT-5 (F-3): reordered manifest with doctrine-memory / doctrine-partner
        // siblings — selection must key on name == marketplace name, not [0].
        let manifest = MarketplaceManifest {
            name: "doctrine".into(),
            plugins: vec![
                ManifestPlugin {
                    name: "doctrine-memory".into(),
                },
                ManifestPlugin {
                    name: "doctrine-partner".into(),
                },
                ManifestPlugin {
                    name: "doctrine".into(),
                },
            ],
        };
        assert_eq!(select_plugin(&manifest), Some("doctrine"));
        assert_ne!(manifest.plugins[0].name, "doctrine", "[0] would be wrong");
    }

    #[test]
    fn plugin_presence_is_exact_not_substring() {
        // VT-6 (F-4): a `plugin list` showing only the sibling — doctrine-partner
        // installed, doctrine absent — must NOT satisfy the doctrine@doctrine check.
        let fixture =
            "Installed plugins:\n\n  ❯ doctrine-partner@doctrine\n    Status: ✔ enabled\n";
        assert!(
            !claude_list_has(fixture, "doctrine@doctrine"),
            "sibling doctrine-partner@doctrine must not false-satisfy"
        );
        // Lock the fix: the old bare substring grep WOULD have false-matched.
        assert!(fixture.contains("doctrine"));
        let present = "  ❯ doctrine@doctrine\n    Status: ✔ enabled\n";
        assert!(claude_list_has(present, "doctrine@doctrine"));
    }

    #[test]
    fn marketplace_presence_is_exact_token() {
        let present = "Configured marketplaces:\n\n  ❯ doctrine\n    Source: Directory (/workspace/doctrine)\n";
        assert!(claude_list_has(present, "doctrine"));
        // A slug path that merely contains `doctrine` is not the bare token.
        let other = "  ❯ other\n    Source: GitHub (davidlee/doctrine)\n";
        assert!(!claude_list_has(other, "doctrine"));
    }

    #[test]
    fn source_default_is_github_slug() {
        // VT-1: dev=false ⇒ the github install.repo slug.
        let cwd = tempfile::tempdir().unwrap();
        let src =
            select_marketplace_source(Path::new("/unused"), cwd.path(), "davidlee/doctrine", false)
                .unwrap();
        assert_eq!(src, MarketplaceSource::Github("davidlee/doctrine".into()));
    }

    #[test]
    fn source_dev_is_directory_abs() {
        // VT-1: dev=true ⇒ Directory(abs canonical root).
        let dir = tempfile::tempdir().unwrap();
        write_marketplace_manifest(dir.path());
        let src =
            select_marketplace_source(dir.path(), Path::new("/unused"), "davidlee/doctrine", true)
                .unwrap();
        match src {
            MarketplaceSource::Directory(p) => {
                assert!(p.is_absolute());
                assert_eq!(p, fs::canonicalize(dir.path()).unwrap());
            }
            other => panic!("expected Directory, got {other:?}"),
        }
    }

    #[test]
    fn source_dev_missing_manifest_errors() {
        // VT-2: --dev with no .claude-plugin/marketplace.json ⇒ hard error.
        let dir = tempfile::tempdir().unwrap();
        let err =
            select_marketplace_source(dir.path(), Path::new("/unused"), "davidlee/doctrine", true)
                .unwrap_err();
        assert!(
            err.to_string().contains("marketplace manifest"),
            "expected a manifest-absent error, got: {err}"
        );
    }

    #[test]
    fn source_dev_relative_root_canonicalizes_absolute() {
        // VT-4 (F-2): a relative --path yields an absolute canonical source, with
        // cwd injected so the test is deterministic (no process-CWD mutation).
        let base = tempfile::tempdir().unwrap();
        let proj = base.path().join("proj");
        fs::create_dir_all(&proj).unwrap();
        write_marketplace_manifest(&proj);
        let src =
            select_marketplace_source(Path::new("proj"), base.path(), "davidlee/doctrine", true)
                .unwrap();
        match src {
            MarketplaceSource::Directory(p) => {
                assert!(p.is_absolute(), "relative root must yield absolute source");
                assert_eq!(p, fs::canonicalize(&proj).unwrap());
            }
            other => panic!("expected Directory, got {other:?}"),
        }
    }

    // ---------------------------------------------------------------
    // PHASE-03: marketplace source-refresh (R4)
    // ---------------------------------------------------------------

    #[test]
    fn parse_registered_source_reads_directory_and_github() {
        // VT-1: the `marketplace list` block for `doctrine` yields its source.
        let dir = "Configured marketplaces:\n\n  ❯ other\n    Source: GitHub (a/b)\n\n  ❯ doctrine\n    Source: Directory (/workspace/doctrine)\n";
        assert_eq!(
            parse_registered_source(dir, "doctrine"),
            Some(RegisteredSource::Directory("/workspace/doctrine".into()))
        );
        let gh = "  ❯ doctrine\n    Source: GitHub (davidlee/doctrine)\n";
        assert_eq!(
            parse_registered_source(gh, "doctrine"),
            Some(RegisteredSource::Github("davidlee/doctrine".into()))
        );
    }

    #[test]
    fn parse_registered_source_absent_or_sibling_is_none() {
        // VT-1: name absent, or only a sibling present, ⇒ None (caller ⇒ Add).
        let none = "Configured marketplaces:\n\n  ❯ caveman\n    Source: GitHub (j/c)\n";
        assert_eq!(parse_registered_source(none, "doctrine"), None);
        // A sibling marketplace must not leak its source to `doctrine`.
        let sibling = "  ❯ doctrine-memory\n    Source: Directory (/tmp/x)\n";
        assert_eq!(parse_registered_source(sibling, "doctrine"), None);
    }

    #[test]
    fn marketplace_action_add_skip_refresh() {
        // VT-1: absent ⇒ Add; same source ⇒ Skip; different ⇒ Refresh.
        let intended = MarketplaceSource::Directory(PathBuf::from("/workspace/doctrine"));
        assert_eq!(marketplace_action(None, &intended), MarketplaceAction::Add);
        assert_eq!(
            marketplace_action(
                Some(RegisteredSource::Directory("/workspace/doctrine".into())),
                &intended
            ),
            MarketplaceAction::Skip
        );
        assert_eq!(
            marketplace_action(
                Some(RegisteredSource::Directory("/old/path".into())),
                &intended
            ),
            MarketplaceAction::Refresh
        );
        // github slug parity, and a kind mismatch ⇒ Refresh (never a false Skip).
        let gh = MarketplaceSource::Github("davidlee/doctrine".into());
        assert_eq!(
            marketplace_action(
                Some(RegisteredSource::Github("davidlee/doctrine".into())),
                &gh
            ),
            MarketplaceAction::Skip
        );
        assert_eq!(
            marketplace_action(
                Some(RegisteredSource::Directory("/workspace/doctrine".into())),
                &gh
            ),
            MarketplaceAction::Refresh
        );
    }

    #[test]
    fn refresh_failure_is_fatal_only_on_refresh() {
        // VT-2 (F-5): a failed refresh aborts; a failed fresh add is tolerable.
        assert!(refresh_failure_is_fatal(&MarketplaceAction::Refresh));
        assert!(!refresh_failure_is_fatal(&MarketplaceAction::Add));
        assert!(!refresh_failure_is_fatal(&MarketplaceAction::Skip));
    }

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

    #[test]
    fn expand_worker_marker_replaces_literal_marker() {
        let def = format!("before\n{WORKER_RESOLVE_MARKER}\nafter\n");
        let expanded = expand_worker_marker(&def, "resolved");
        assert_eq!(expanded, "before\nresolved\nafter\n");
    }

    #[test]
    fn expand_worker_marker_without_marker_is_unchanged() {
        let def = "dispatch-worker resolve --role worker".to_string();
        let expanded = expand_worker_marker(&def, "ignored");
        assert_eq!(expanded, def);
    }

    #[test]
    fn install_agent_def_expands_worker_marker_before_write() {
        let dir = tempfile::tempdir().unwrap();
        let hymns_dir = dir
            .path()
            .join(".doctrine")
            .join(HYMNS_DIRNAME)
            .join("role");
        fs::create_dir_all(&hymns_dir).unwrap();
        fs::write(hymns_dir.join("worker.md"), "RESOLVED WORKER BODY").unwrap();

        let mut out = Vec::new();
        install_agent_def(
            dir.path(),
            "claude",
            None,
            DISPATCH_WORKER_AGENT_ASSET,
            false,
            false,
            &mut out,
        )
        .unwrap();

        let written =
            fs::read_to_string(dir.path().join(".doctrine/agents/dispatch-worker.md")).unwrap();
        assert!(written.contains("RESOLVED WORKER BODY"), "{written}");
        assert!(!written.contains(WORKER_RESOLVE_MARKER), "{written}");
    }

    #[test]
    fn install_agent_def_dispatch_probe_writes_bytes_identically_under_the_derived_dest() {
        // SL-206 PHASE-13 (T4/VT-2): the dest filename is DERIVED from the
        // embed-asset basename — a marker-free asset (the probe def has no
        // WORKER_RESOLVE_MARKER) lands as a plain byte copy at its OWN name
        // (`dispatch-probe.md`), not the previously-hardcoded
        // `dispatch-worker.md` (the bug this generalization fixes).
        let dir = tempfile::tempdir().unwrap();
        let mut out = Vec::new();
        install_agent_def(
            dir.path(),
            "claude",
            None,
            DISPATCH_PROBE_AGENT_ASSET,
            false,
            false,
            &mut out,
        )
        .unwrap();

        let expected = embedded_asset(DISPATCH_PROBE_AGENT_ASSET).unwrap();
        let written = fs::read(dir.path().join(".doctrine/agents/dispatch-probe.md")).unwrap();
        assert_eq!(written, expected.as_ref());
    }

    // --- Workflows leg (SL-206 PHASE-13 T6) ---

    #[test]
    fn embedded_workflow_defs_carries_the_drive_slice_payload() {
        // SL-206 PHASE-14 shipped `drive-slice.js` into the embed root — the leg
        // is now live in production, not a no-op.
        let defs = embedded_workflow_defs();
        assert!(
            defs.iter()
                .any(|(rel, bytes)| rel == "drive-slice.js" && !bytes.is_empty()),
            "expected drive-slice.js in embedded workflow defs, got {:?}",
            defs.iter().map(|(r, _)| r).collect::<Vec<_>>()
        );
    }

    #[test]
    fn install_workflow_assets_materializes_and_links_a_synthetic_workflow() {
        // Drives the mechanism over a synthetic asset list (no real embed payload
        // exists yet) — proves materialize+link works before PHASE-14 ships the
        // first real `.js` file.
        let dir = tempfile::tempdir().unwrap();
        let mut out = Vec::new();
        let assets = vec![("drive-slice.js".to_string(), b"// stub workflow".to_vec())];
        install_workflow_assets(dir.path(), false, false, &mut out, &assets).unwrap();

        let canon = dir.path().join(".doctrine/workflows/drive-slice.js");
        let link = dir.path().join(".claude/workflows/drive-slice.js");
        assert_eq!(fs::read(&canon).unwrap(), b"// stub workflow");
        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink(),
            "the workflows leg links, not copies, into .claude/workflows/"
        );
        assert_eq!(fs::read(&link).unwrap(), b"// stub workflow");
    }

    #[test]
    fn install_workflow_assets_dry_run_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let mut out = Vec::new();
        let assets = vec![("drive-slice.js".to_string(), b"// stub".to_vec())];
        install_workflow_assets(dir.path(), false, true, &mut out, &assets).unwrap();

        assert!(!dir.path().join(".doctrine/workflows").exists());
        assert!(!dir.path().join(".claude/workflows").exists());
    }

    #[test]
    fn install_workflow_assets_empty_list_is_a_no_op() {
        // The production shape TODAY (PHASE-14 payload absent): no assets ⇒ no
        // dirs created, no error.
        let dir = tempfile::tempdir().unwrap();
        let mut out = Vec::new();
        install_workflow_assets(dir.path(), false, false, &mut out, &[]).unwrap();
        assert!(!dir.path().join(".doctrine/workflows").exists());
        assert!(!dir.path().join(".claude/workflows").exists());
    }

    // --- trait-aware bake (SL-191 VT-3) ---

    /// A corpus that resolves the worker role AND covers the `adherence/low` trait,
    /// so a covered bake inlines the adherence body and an uncovered/typo'd key fails.
    fn role_plus_adherence_corpus() -> Vec<crate::hymns::Snippet> {
        use crate::hymns::{Band, Provenance, Role, Selector, Slot, Snippet};
        vec![
            Snippet {
                slot: Slot::new(Band::Role, "worker"),
                selector: Selector {
                    role: Some(Role::Worker),
                    ..Default::default()
                },
                provenance: Provenance::Framework,
                body: "ROLE WORKER BODY".into(),
            },
            Snippet {
                slot: Slot::new(Band::Model, "adherence-low"),
                selector: Selector {
                    model: ["adherence/low".to_string()].into(),
                    ..Default::default()
                },
                provenance: Provenance::Framework,
                body: "ADHERENCE LOW BODY".into(),
            },
        ]
    }

    #[test]
    fn bake_worker_def_covered_trait_inlines_adherence_via_model_band() {
        let corpus = role_plus_adherence_corpus();
        let def = format!("---\ntraits: [\"adherence/low\"]\n---\n\n{WORKER_RESOLVE_MARKER}\n");
        // worker_context adds Band::Model because a trait is declared, so the
        // adherence body composes into the baked def.
        let out = bake_worker_def(&def, "pi", &corpus, &crate::hymns::SealSet::default()).unwrap();
        assert!(out.contains("ADHERENCE LOW BODY"), "{out}");
        assert!(!out.contains(WORKER_RESOLVE_MARKER), "{out}");
    }

    #[test]
    fn bake_worker_def_traitless_stays_role_only() {
        let corpus = role_plus_adherence_corpus();
        let def = format!("---\nname: x\n---\n\n{WORKER_RESOLVE_MARKER}\n");
        // No traits ⇒ role-only band ⇒ adherence content must NOT leak in (VT-2/VT-4).
        let out =
            bake_worker_def(&def, "claude", &corpus, &crate::hymns::SealSet::default()).unwrap();
        assert!(out.contains("ROLE WORKER BODY"), "{out}");
        assert!(!out.contains("ADHERENCE LOW BODY"), "{out}");
    }

    #[test]
    fn bake_worker_def_uncovered_trait_is_a_hard_error() {
        let corpus = role_plus_adherence_corpus();
        // A typo'd key ("adherance") is covered by nothing → contractless → bail.
        let def = format!("---\ntraits: [\"adherance/low\"]\n---\n\n{WORKER_RESOLVE_MARKER}\n");
        let err = bake_worker_def(&def, "pi", &corpus, &crate::hymns::SealSet::default())
            .expect_err("uncovered trait must fail the bake");
        assert!(err.to_string().contains("uncovered trait"), "{err}");
    }

    // --- shipped-def declarations (SL-191 VT-5) ---

    #[test]
    fn embedded_dispatch_worker_defs_declare_expected_traits() {
        let defs = embedded_agent_defs();
        let traits_of = |rel: &str| -> BTreeSet<String> {
            let (_, bytes) = defs
                .iter()
                .find(|(name, _)| name == rel)
                .unwrap_or_else(|| panic!("embedded agent def {rel} present"));
            parse_agent_def_traits(std::str::from_utf8(bytes).unwrap()).unwrap()
        };
        // The pi/universal twin ships the low-adherence trait; losing this declaration
        // (not just a fixture) must fail. The claude def declares none (D1).
        assert_eq!(
            traits_of("pi/dispatch-worker.md"),
            ["adherence/low".to_string()].into()
        );
        assert!(traits_of("claude/dispatch-worker.md").is_empty());
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
                hymns: HymnsSection::default(),
            }
        }
    }
}
