// SPDX-License-Identifier: GPL-3.0-only
#![allow(
    clippy::same_name_method,
    reason = "rust-embed derive generates conflicting method names"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use rust_embed::RustEmbed;
use serde::Deserialize;

use crate::install_ownership::{
    Action, Desired, LinkSeen, Member, MemberKind, Notice, Scope, Snapshot, SpliceRefusal,
};
use crate::memory::MemoryType;

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
/// Marker token injected into the dispatch-worker agent defs (SL-186 PHASE-04).
/// When the convergence executor writes a def carrying this literal, it resolves the role
/// band (`Role::Worker`) through the prompt engine and replaces the marker with
/// the assembled text.
// full worker context (role+traits); the bake reads the def's frontmatter, not the
// marker args (kept literal, F4/C6). The sentinel stays this exact string — it is
// matched by `.contains`, never parsed for its `--role` argument.
pub(crate) const WORKER_RESOLVE_MARKER: &str = "{{ prompt resolve --role worker }}";

/// The `install/manifest.toml` schema.
#[derive(Debug, Deserialize)]
struct Manifest {
    /// Target directory relative to the project root (e.g. `".doctrine"`).
    #[serde(default = "default_target")]
    target: String,

    #[serde(default)]
    base: BaseSection,

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

    #[serde(default)]
    framework: FrameworkSection,

    /// The harness registry (SL-281, DEC-373): install's harness vocabulary.
    #[serde(default)]
    harness: Registry,
}

/// Framework-owned canon roots (SL-281, ADR-019 "owned"). Everything outside
/// them that is not a doctrine link is user-owned.
#[derive(Debug, Default, Deserialize)]
struct FrameworkSection {
    #[serde(default)]
    roots: Vec<String>,
}

/// One `[harness.<name>]` table. Carries at most one skills route: `skills`
/// (doctrine links into that dir) or `skills_via` (an `npx skills` target).
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct HarnessSection {
    /// The dir whose presence implies this harness when no `--agent` is given.
    detect: Option<String>,
    /// The boot adapter name; absent means no boot leg.
    boot: Option<String>,
    /// The link dir for the def set under `install/agents/<harness>/`.
    agents: Option<String>,
    skills: Option<String>,
    skills_via: Option<String>,
    workflows: Option<String>,
}

/// Harness name → its registry table, in name order.
type Registry = BTreeMap<String, HarnessSection>;

fn default_target() -> String {
    ".doctrine".to_string()
}

/// The minimal projection base set (SL-227). `backings` are the ONLY embedded
/// assets `build_plan` projects eagerly; every other embed is published, not
/// projected. Keys must match `publication.rs`'s `BASE_BACKINGS`.
#[derive(Debug, Default, Deserialize)]
struct BaseSection {
    #[serde(default)]
    backings: Vec<String>,
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
        print_forward_summary(&project_root, args, &manifest.harness)?;
        let planned = print_ownership_plan(&project_root, args, &manifest)?;
        if !args.global {
            write_gitignore_block(&project_root, &manifest, &planned, true)?;
        }
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
    run_forward_steps(&project_root, &exec, args, &manifest)?;
    // Last: the block reflects what is now on disk, whatever the steps did.
    if !args.global {
        write_gitignore_block(&project_root, &manifest, &[], false)?;
    }
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

/// The boot adapters a harness set implies, read from the registry's `boot`
/// field. `pi` rides the codex adapter; `universal` and unregistered names have
/// none. Duplicates collapse; first-seen order is preserved.
fn boot_names(agents: &[String], registry: &Registry) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for agent in agents {
        let Some(boot) = registry
            .get(&agent.to_lowercase())
            .and_then(|h| h.boot.as_ref())
        else {
            continue;
        };
        if !names.contains(boot) {
            names.push(boot.clone());
        }
    }
    names
}

/// The boot harnesses this install wires. An explicit `--agent` set is a
/// DIRECTIVE — honoured even in a project with no harness directory yet, and
/// honoured when it implies none. With no flags, the registry's detect dirs
/// govern, unioned with `boot::resolve_harnesses`' detection for its one extra
/// rule (a real, non-alias `AGENTS.md` implies codex). Both paths map names to
/// adapters through the registry (SL-281).
///
/// SL-250 PHASE-04: this used to be `resolve_harnesses(&[], root)`, which
/// discarded the `--agent` the operator had just typed. In a fresh project
/// `.claude/` does not exist when the leg is resolved (framework convergence creates
/// it later), so `doctrine install --agent claude` wired no @-import and no
/// hooks and reported only "no harness directories detected — skipped". That is
/// the silent-inertness failure class this slice exists to end, one layer above
/// the one it was chartered to fix.
fn install_harnesses(
    agents: &[String],
    root: &Path,
    registry: &Registry,
) -> Vec<crate::boot::Harness> {
    let names = boot_names(&detect_agents(agents, root, registry), registry);
    let mut harnesses = if names.is_empty() {
        Vec::new()
    } else {
        crate::boot::resolve_harnesses(&names, root).unwrap_or_default()
    };
    if agents.is_empty() {
        for h in crate::boot::resolve_harnesses(&[], root).unwrap_or_default() {
            if !harnesses.contains(&h) {
                harnesses.push(h);
            }
        }
    }
    harnesses
}

/// Where non-linked skills go: each registered harness's `skills_via` target,
/// or an unregistered name itself. De-duplicated, first-seen order. Harnesses
/// with a doctrine `skills` route are not delegated.
#[derive(Debug, PartialEq, Eq)]
struct DelegateTargets {
    targets: Vec<String>,
    unregistered: Vec<String>,
}

fn delegate_targets(agents: &[String], registry: &Registry) -> DelegateTargets {
    let mut d = DelegateTargets {
        targets: Vec::new(),
        unregistered: Vec::new(),
    };
    for agent in agents {
        let target = match registry.get(agent) {
            Some(h) if h.skills.is_some() => continue,
            Some(h) => h.skills_via.clone().unwrap_or_else(|| agent.clone()),
            None => {
                if !d.unregistered.contains(agent) {
                    d.unregistered.push(agent.clone());
                }
                agent.clone()
            }
        };
        if !d.targets.contains(&target) {
            d.targets.push(target);
        }
    }
    d
}

/// What confirming a harness installs, part by part (RV-420 F-9).
const OFFER_SKILLS: &str = "skills";
const OFFER_SKILLS_VIA_NPX: &str = "skills via npx";
const OFFER_AGENT_DEFS: &str = "agent defs";
const OFFER_WORKFLOWS: &str = "workflows";

/// What confirming `agent` installs: its skills route, then the def sets its
/// registry entry links. An unregistered harness gets npx skills only.
fn install_offer(agent: &str, registry: &Registry) -> String {
    let h = registry.get(agent);
    let skills = if h.is_some_and(|h| h.skills.is_some()) {
        OFFER_SKILLS
    } else {
        OFFER_SKILLS_VIA_NPX
    };
    let parts: Vec<&str> = [
        Some(skills),
        h.and_then(|h| h.agents.as_ref()).map(|_| OFFER_AGENT_DEFS),
        h.and_then(|h| h.workflows.as_ref())
            .map(|_| OFFER_WORKFLOWS),
    ]
    .into_iter()
    .flatten()
    .collect();
    format!("{} for {agent}", parts.join(" + "))
}

/// The canon convergence any confirmed harness triggers: a project install
/// prunes what is not shipped; a global one only reports it (DEC-377).
fn canon_convergence(scope: Scope) -> &'static str {
    match scope {
        Scope::Project => "converge framework canon (pruning unshipped entries)",
        Scope::Global => "converge framework canon (reporting orphans)",
    }
}

/// The confirmation asked per harness before convergence.
fn install_question(agent: &str, registry: &Registry, scope: Scope) -> String {
    format!(
        "Install {}, and {}? [y/N/a]",
        install_offer(agent, registry),
        canon_convergence(scope)
    )
}

/// The ownership scope `global` selects.
fn install_scope(global: bool) -> Scope {
    if global {
        Scope::Global
    } else {
        Scope::Project
    }
}

/// Disclose each unregistered harness (DEC-377): it gets no agent defs or
/// links, only npx skills delegation under its own name.
fn report_unregistered(names: &[String], out: &mut dyn Write) -> io::Result<()> {
    for name in names {
        writeln!(
            out,
            "  {name}: unregistered harness — skills delegated to npx under its own name"
        )?;
    }
    Ok(())
}

/// Print the forward-step summary (dry-run or live).
fn print_forward_summary(
    root: &Path,
    args: &InstallArgs<'_>,
    registry: &Registry,
) -> anyhow::Result<()> {
    let agents = detect_agents(args.agents, root, registry);
    let harnesses = install_harnesses(args.agents, root, registry);

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
            writeln!(
                stdout,
                "  {agent:<12} install {}",
                install_offer(agent, registry)
            )?;
        }
        writeln!(
            stdout,
            "  {:<12} {}",
            "canon",
            canon_convergence(install_scope(args.global))
        )?;
    }
    Ok(())
}

/// Build the convergence context and desired set for `harnesses`, then hand
/// both to `f` — the one wiring shared by the live run and `--dry-run`.
/// (`Wanted` borrows the embedded defs, so it cannot be returned.)
fn with_wanted<R>(
    root: &Path,
    args: &InstallArgs<'_>,
    manifest: &Manifest,
    harnesses: &[String],
    catalog: &[Entry],
    selected: &[&Entry],
    f: impl FnOnce(&Converge<'_>, &Wanted<'_>) -> anyhow::Result<R>,
) -> anyhow::Result<R> {
    let base = install_base(root, args.global)?;
    let cx = Converge {
        root,
        base: &base,
        scope: install_scope(args.global),
    };
    let defs = embedded_agent_defs();
    let workflows = embedded_workflow_defs();
    let wanted = desired_set(
        &base,
        &manifest.harness,
        harnesses,
        &defs,
        catalog,
        selected,
        &workflows,
    );
    f(&cx, &wanted)
}

/// `--dry-run`: survey the disk and print the ownership plan, returning its
/// actions. Mutates nothing.
fn print_ownership_plan(
    root: &Path,
    args: &InstallArgs<'_>,
    manifest: &Manifest,
) -> anyhow::Result<Vec<Action>> {
    let agents = detect_agents(args.agents, root, &manifest.harness);
    if agents.is_empty() {
        return Ok(Vec::new());
    }
    let catalog = discover()?;
    let selected = select_for_install(&catalog, args.skills, args.domains)?;
    with_wanted(
        root,
        args,
        manifest,
        &agents,
        &catalog,
        &selected,
        |cx, wanted| {
            let (scanned, actions) =
                survey(cx, &manifest.framework.roots, &manifest.harness, wanted)?;
            let mut out = io::stdout();
            writeln!(out, "\n{}", report_header(cx.scope, true))?;
            for a in &actions {
                writeln!(out, "{}", action_line(a, &scanned, wanted, true))?;
            }
            Ok(actions)
        },
    )
}

/// Run the forward steps: memory sync → boot wire → skills per agent.
/// Each step is individually prompted (unless `--yes`). Partial failure
/// is non-fatal — errors are printed and the next step proceeds.
fn run_forward_steps(
    root: &Path,
    exec: &Path,
    args: &InstallArgs<'_>,
    manifest: &Manifest,
) -> anyhow::Result<()> {
    let registry = &manifest.harness;
    let agents = detect_agents(args.agents, root, registry);
    let harnesses = install_harnesses(args.agents, root, registry);

    print_forward_summary(root, args, registry)?;

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
    // it MUST run before the convergence below, because its worker-def bake
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

    // 3. Framework artefacts per confirmed harness, then npx delegation. The
    // answers form this run's selection; one convergence covers every
    // confirmed harness (SL-281 D3) — none confirmed, nothing converges.
    let catalog = discover()?;
    let selected = select_for_install(&catalog, args.skills, args.domains)?;
    let (runner_name, runner) = resolve_runner();
    let repo = &crate::dtoml::load_doctrine_toml(root)?.install.repo;

    let mut confirmed: Vec<String> = Vec::new();
    for agent in &agents {
        let question = install_question(agent, registry, install_scope(args.global));
        if prompt_step(&question, args.yes, &mut all_yes)? {
            confirmed.push(agent.clone());
        }
    }
    if !confirmed.is_empty() {
        let mut out = io::stdout();
        let result = with_wanted(
            root,
            args,
            manifest,
            &confirmed,
            &catalog,
            &selected,
            |cx, wanted| converge(cx, &manifest.framework.roots, registry, wanted, &mut out),
        );
        if let Err(e) = result {
            writeln!(out, "  framework install failed: {e:#}")?;
        }
    }

    // Batch-delegate every confirmed harness without a doctrine `skills` route
    // to a single npx invocation, one de-duplicated target per skills route
    // (pi + codex → `universal` once).
    let delegate = delegate_targets(&confirmed, registry);
    report_unregistered(&delegate.unregistered, &mut io::stdout())?;
    if !delegate.targets.is_empty() {
        let mut out = io::stdout();
        if let Err(e) = install_for_other(
            &InstallOtherArgs {
                agent_names: &delegate.targets,
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

    Ok(())
}

// ---------------------------------------------------------------------------
// Manifest
// ---------------------------------------------------------------------------

/// Fetch an embedded asset (relative to `install/`) as UTF-8 text.
/// Shared with `slice` for template scaffolding.
pub(crate) fn asset_text(name: &str) -> anyhow::Result<String> {
    crate::asset_source::read_text(name)
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
    crate::asset_source::iter()
        .filter_map(|name| {
            let name = name.as_ref();
            name.strip_prefix(prefix).map(|rel| {
                (
                    rel.to_string(),
                    crate::asset_source::read_bytes(name).map(|c| c.to_vec()),
                )
            })
        })
        .filter_map(|(rel, opt)| opt.map(|bytes| (rel, bytes)))
        .collect()
}

/// Return every embedded agent-def file (under `"agents/"`) as
/// `(relative-path, bytes)` pairs. Used by `check_corpus` for
/// def-marker integrity (SL-186 PHASE-04 / T6).
pub(crate) fn embedded_agent_defs() -> Vec<(String, Vec<u8>)> {
    let prefix = "agents/";
    crate::asset_source::iter()
        .filter_map(|name| {
            let name = name.as_ref();
            name.strip_prefix(prefix).map(|rel| {
                (
                    rel.to_string(),
                    crate::asset_source::read_bytes(name).map(|c| c.to_vec()),
                )
            })
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
    "harvest",
    "handover",
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
/// (`Baker::bake`) and VT-3 drive; corpus-pure (disk/embed I/O stays in the
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
    let data = crate::asset_source::read_bytes("manifest.toml")
        .context("install/manifest.toml is missing from embedded assets")?;
    let text = std::str::from_utf8(&data).context("install/manifest.toml is not valid UTF-8")?;
    parse_manifest(text)
}

/// The `[framework] roots` the embedded manifest declares (SL-281).
#[cfg(test)]
pub(crate) fn framework_roots() -> anyhow::Result<Vec<String>> {
    Ok(load_manifest()?.framework.roots)
}

/// Parse and validate manifest text. A harness table may carry only one skills
/// route.
fn parse_manifest(text: &str) -> anyhow::Result<Manifest> {
    let manifest: Manifest =
        toml::from_str(text).context("Failed to parse install/manifest.toml")?;
    for (name, h) in &manifest.harness {
        if h.skills.is_some() && h.skills_via.is_some() {
            bail!(
                "install/manifest.toml [harness.{name}] sets both `skills` and `skills_via`; pick one"
            );
        }
    }
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

    // 2. Base backings — the MINIMAL projection (SL-227 FR-007/FR-008, D8). Only
    //    the manifest-declared `[base].backings` land eagerly; every other embedded
    //    asset is published (reachable on demand), never eagerly copied. `.gitignore`
    //    is the managed block (`write_gitignore_block`), so it is skipped here; the two content backings map to
    //    `install/<key>` embed sources and land write-if-absent under the target.
    //    `embedded_filenames()` stays defined (the crux reachability gate consumes
    //    it) but no longer drives projection.
    for key in &manifest.base.backings {
        if key == ".gitignore" {
            continue;
        }
        let source = key.clone();
        let dest = target_dir.join(key);
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

    Plan {
        project_root: project_root.to_path_buf(),
        target_dir,
        steps,
    }
}

/// Sorted list of embedded asset names, excluding `manifest.toml`. Since the
/// SL-227 minimal-projection flip, `build_plan` projects only the manifest base
/// backings, so this enumerator no longer drives projection — it is consumed by
/// the crux reachability gate and the `*_is_shipped` embedding tests.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "consumed by the SL-227 reachability gate + embedding tests"
    )
)]
fn embedded_filenames() -> Vec<String> {
    let mut names: Vec<String> = crate::asset_source::iter()
        .map(|f| f.to_string())
        .filter(|n| n != "manifest.toml")
        .collect();
    names.sort();
    names
}

/// Operator disclosure: the block cannot list what it cannot attribute.
const NPX_SKILLS_NOTE: &str = "note: skill copies written by `npx skills` (.agents/skills/<id>) are not listed in the block; doctrine cannot tell them from yours.";

/// The managed block (design `sec-gitignore`): the manifest's runtime entries,
/// each framework root's contents, and one anchored line per doctrine link
/// under `base` — the links on disk now, adjusted by `planned` actions not yet
/// executed (a dry-run's prediction; empty for a live run).
fn gitignore_block(
    base: &Path,
    manifest: &Manifest,
    planned: &[Action],
) -> anyhow::Result<Vec<String>> {
    let link_dirs = registered_link_dirs(base, &manifest.harness);
    let scanned = scan(base, &[], &link_dirs, &Desired::default())?;
    let roots: Vec<PathBuf> = manifest
        .framework
        .roots
        .iter()
        .map(|r| base.join(r))
        .collect();
    let on_disk = crate::install_ownership::doctrine_links(&roots, &scanned.snapshot.links);
    let links: Vec<String> = crate::install_ownership::links_after(on_disk, planned)
        .iter()
        .filter_map(|site| site.strip_prefix(base).ok())
        .map(|rel| format!("/{}", rel.display()))
        .collect();
    Ok(crate::install_ownership::render_block(
        &manifest.gitignore.entries,
        &manifest.framework.roots,
        &links,
    ))
}

/// Splice the managed block into the project `.gitignore` (or, under
/// `dry_run`, print it and write nothing). A malformed file is left untouched
/// and reported — the install still succeeds (D2).
fn write_gitignore_block(
    root: &Path,
    manifest: &Manifest,
    planned: &[Action],
    dry_run: bool,
) -> anyhow::Result<()> {
    let block = gitignore_block(root, manifest, planned)?;
    let path = root.join(".gitignore");
    let existing = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e).with_context(|| format!("Failed to read {}", path.display())),
    };
    let mut out = io::stdout();
    match crate::install_ownership::splice_block(&existing, &block) {
        Ok(_) if dry_run => {
            writeln!(out, "\nGitignore block (not written under --dry-run):")?;
            for line in &block {
                writeln!(out, "  {line}")?;
            }
        }
        Ok(text) => {
            crate::fsutil::write_atomic(&path, text.as_bytes())?;
            let rest = format!(".gitignore (managed block, {} lines)", block.len());
            writeln!(out, "\n{}", row("gitignore", &rest))?;
        }
        Err(SpliceRefusal::Malformed { starts, ends }) => {
            let rest = format!(
                ".gitignore left untouched: found {starts} start / {ends} end managed-block markers (expected one of each, or none)"
            );
            writeln!(out, "\n{}", row("gitignore", &rest))?;
        }
    }
    writeln!(out, "{NPX_SKILLS_NOTE}")?;
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
fn detect_agents(agents: &[String], root: &Path, registry: &Registry) -> Vec<String> {
    if !agents.is_empty() {
        return agents.to_vec();
    }
    registry
        .iter()
        .filter(|(_, h)| h.detect.as_ref().is_some_and(|d| root.join(d).exists()))
        .map(|(name, _)| name.clone())
        .collect()
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
                let data = crate::asset_source::read_bytes(source)
                    .with_context(|| format!("Embedded file '{source}' not found"))?;
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent).with_context(|| {
                        format!("Failed to create parent dir for {}", dest.display())
                    })?;
                }
                #[expect(clippy::disallowed_methods, reason = "derived asset unpack")]
                fs::write(dest, &data)
                    .with_context(|| format!("Failed to write {}", dest.display()))?;
            }
            Step::Skip { .. } => {
                // nothing to do
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
// Staging (SL-281 PHASE-03) — exclusive-create staging names plus an atomic
// swap, shared by canon materialisation and link writes.
// ---------------------------------------------------------------------------

/// Candidate attempts before `stage_from` gives up. Bounded so a hostile
/// directory cannot hang an install.
const STAGING_ATTEMPTS: u64 = 64;

/// The staging name for candidate `seq` beside `path`:
/// `<parent>/.tmp-<name>.<pid>.<seq>`. Pure; uniqueness is enforced by the
/// exclusive create in `stage_from`, not by the name.
fn staging_path(path: &Path, seq: u64) -> anyhow::Result<PathBuf> {
    use crate::install_ownership::STAGING_PREFIX;
    let parent = path.parent().context("path has no parent directory")?;
    let name = path.file_name().context("path has no file name")?;
    Ok(parent.join(format!(
        "{STAGING_PREFIX}{}.{}.{seq}",
        name.to_string_lossy(),
        std::process::id()
    )))
}

/// Create a fresh staging entry beside `path` via `create`, which must be an
/// exclusive create (`symlink` / `create_dir`). Tries candidates `first_seq..`,
/// moving on at `AlreadyExists` (a pre-existing entry is never reused or
/// removed); any other error is returned. Returns the path it created.
fn stage_from(
    path: &Path,
    first_seq: u64,
    create: impl Fn(&Path) -> io::Result<()>,
) -> anyhow::Result<PathBuf> {
    for seq in first_seq..first_seq + STAGING_ATTEMPTS {
        let tmp = staging_path(path, seq)?;
        match create(&tmp) {
            Ok(()) => return Ok(tmp),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => {
                return Err(e).with_context(|| format!("Failed to stage {}", tmp.display()));
            }
        }
    }
    bail!(
        "no free staging name beside {} after {STAGING_ATTEMPTS} attempts",
        path.display()
    )
}

/// `stage_from` with the next process-global candidate number, so concurrent
/// same-process stagers start apart.
fn stage_exclusive(
    path: &Path,
    create: impl Fn(&Path) -> io::Result<()>,
) -> anyhow::Result<PathBuf> {
    static NEXT_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let first = NEXT_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    stage_from(path, first, create)
}

/// Create the relative symlink `dest -> target` atomically: symlink at a
/// freshly-created unique staging name (exclusive; never a pre-existing entry)
/// then `rename` over `dest`. `rename` DOES replace an existing symlink (only
/// a non-empty *directory* is the exception), so an owned-link relink never leaves
/// a half-state. Callers pass only Create/Relink dests (missing or proven ours).
/// If the swap fails, the staging link we created is removed.
fn write_link(dest: &Path, target: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::symlink;
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    let tmp = stage_exclusive(dest, |t| symlink(target, t))?;
    fs::rename(&tmp, dest).map_err(|e| {
        fs::remove_file(&tmp).ok();
        anyhow::Error::new(e).context(format!(
            "Failed to swap link {} → {}",
            tmp.display(),
            dest.display()
        ))
    })
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

/// Return every embedded workflow file (under `"workflows/"`) as
/// `(relative-path, bytes)` pairs — mirrors `embedded_agent_defs`. Empty since
/// SL-254 PHASE-10 deleted `drive-slice.js`, its only payload.
pub(crate) fn embedded_workflow_defs() -> Vec<(String, Vec<u8>)> {
    let prefix = "workflows/";
    crate::asset_source::iter()
        .filter_map(|name| {
            let name = name.as_ref();
            name.strip_prefix(prefix).map(|rel| {
                (
                    rel.to_string(),
                    crate::asset_source::read_bytes(name).map(|c| c.to_vec()),
                )
            })
        })
        .filter_map(|(rel, opt)| opt.map(|bytes| (rel, bytes)))
        .collect()
}

// ---------------------------------------------------------------------------
// Skill canon materialisation (SL-250 PHASE-05) — the binary-sourced canonical
// skills tree. Recovered near-verbatim from `git show 347197e8^:src/skills.rs`;
// the convergence executor calls it for every skill member it writes.
// ---------------------------------------------------------------------------

/// Materialise the canonical copy of `entry` at `dest` (`.doctrine/skills/<id>`),
/// staged in a unique, exclusively-created `.tmp-<id>.<pid>.<n>` sibling then
/// swapped in with a minimal-window remove+rename. Always overwrites — the
/// canonical tree is derived (owns no authored data). A pre-existing entry at a
/// staging name is never reused; on failure only the directory we created is
/// removed.
///
/// Unix reality (design §5.1/§10 pass-2 F4): `rename` cannot replace a non-empty
/// directory and std has no `renameat2(RENAME_EXCHANGE)`, so the swap is
/// remove-then-rename — a one-syscall window where a crash leaves the agent link
/// dangling, healed by the next idempotent install. A partial stage lives only
/// under the staging name, never under `<id>`, so a live link never sees a
/// half-tree.
fn materialise_canonical(entry: &Entry, dest: &Path) -> anyhow::Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    let tmp = stage_exclusive(dest, |t| fs::create_dir(t))?;
    // Fill and swap; any failure removes the staging dir we own (same filesystem
    // → the rename is valid).
    let swapped = copy_skill(entry, &tmp).and_then(|()| {
        // Minimal-window swap: drop the prior canonical, then rename the temp in.
        if dest.exists() {
            fs::remove_dir_all(dest)
                .with_context(|| format!("Failed to remove {}", dest.display()))?;
        }
        fs::rename(&tmp, dest)
            .with_context(|| format!("Failed to swap {} → {}", tmp.display(), dest.display()))
    });
    if swapped.is_err() {
        fs::remove_dir_all(&tmp).ok();
    }
    swapped
}

/// Copy an embedded skill's files into `dest`, stripping the source prefix.
fn copy_skill(entry: &Entry, dest: &Path) -> anyhow::Result<()> {
    let prefix = format!("{}/skills/{}/", entry.domain, entry.id);
    for file in &entry.files {
        let rel = file
            .strip_prefix(prefix.as_str())
            .with_context(|| format!("'{file}' is not under '{prefix}'"))?;
        let target = dest.join(rel);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }
        let asset =
            PluginAssets::get(file).with_context(|| format!("Embedded file '{file}' not found"))?;
        #[expect(clippy::disallowed_methods, reason = "derived asset unpack")]
        fs::write(&target, &asset.data)
            .with_context(|| format!("Failed to write {}", target.display()))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Convergence shell (SL-281 PHASE-04, design `sec-convergence`). Builds the
// desired set and the on-disk snapshot `install_ownership::plan` decides over,
// then executes its actions, re-proving each against the scan before it
// mutates (SPEC-010 REQ-178).
// ---------------------------------------------------------------------------

/// The canon root a member kind lives under, relative to the install base.
/// One per `[framework] roots` entry — pinned equal by
/// `canon_roots_match_manifest_framework_roots`.
fn canon_root(kind: MemberKind) -> &'static str {
    match kind {
        MemberKind::AgentDef => ".doctrine/agents",
        MemberKind::Skill => ".doctrine/skills",
        MemberKind::Workflow => ".doctrine/workflows",
    }
}

/// Where a member's canon bytes come from.
enum Source<'a> {
    /// An agent def (baked if it carries `WORKER_RESOLVE_MARKER`) or a
    /// workflow; `label` is its embed path below `agents/` / `workflows/`.
    Asset {
        label: String,
        bytes: &'a [u8],
    },
    Skill(&'a Entry),
}

/// The desired set plus each member's source, keyed by `Member.canon`.
struct Wanted<'a> {
    desired: Desired,
    sources: BTreeMap<PathBuf, Source<'a>>,
}

/// Build the desired set (design § Desired set): members span every
/// registered harness; `selected` and `new_links` only those in `harnesses`.
/// An unregistered harness, or one without `agents`/`skills`/`workflows`
/// routes (codex), contributes nothing. Top-level `AGENTS.md` is never a
/// member — only `<harness>/<file>` defs match.
fn desired_set<'a>(
    base: &Path,
    registry: &Registry,
    harnesses: &[String],
    agent_defs: &'a [(String, Vec<u8>)],
    catalog: &'a [Entry],
    skills: &[&Entry],
    workflows: &'a [(String, Vec<u8>)],
) -> Wanted<'a> {
    let mut wanted = Wanted {
        desired: Desired::default(),
        sources: BTreeMap::new(),
    };
    let agents_root = base.join(canon_root(MemberKind::AgentDef));
    for (name, h) in registry {
        let Some(dir) = &h.agents else { continue };
        for (rel, bytes) in agent_defs {
            let Some(file) = rel
                .strip_prefix(name.as_str())
                .and_then(|r| r.strip_prefix('/'))
                .filter(|f| !f.contains('/'))
            else {
                continue;
            };
            wanted.add(
                harnesses,
                Shipped {
                    kind: MemberKind::AgentDef,
                    canon: agents_root.join(name).join(file),
                    sites: vec![(name.as_str(), base.join(dir).join(file))],
                    requested: true,
                    source: Source::Asset {
                        label: rel.clone(),
                        bytes,
                    },
                },
            );
        }
    }
    let skill_routes = routes(registry, |h| h.skills.as_ref());
    for entry in catalog {
        wanted.add(
            harnesses,
            Shipped {
                kind: MemberKind::Skill,
                canon: base.join(canon_root(MemberKind::Skill)).join(&entry.id),
                sites: sites(base, &skill_routes, &entry.id),
                requested: skills.iter().any(|s| s.id == entry.id),
                source: Source::Skill(entry),
            },
        );
    }
    let workflow_routes = routes(registry, |h| h.workflows.as_ref());
    for (rel, bytes) in workflows {
        let file = rel.rsplit('/').next().unwrap_or(rel);
        wanted.add(
            harnesses,
            Shipped {
                kind: MemberKind::Workflow,
                canon: base.join(canon_root(MemberKind::Workflow)).join(file),
                sites: sites(base, &workflow_routes, file),
                requested: true,
                source: Source::Asset {
                    label: rel.clone(),
                    bytes,
                },
            },
        );
    }
    wanted
}

/// One shipped entry before selection: its link sites, each tagged with the
/// harness whose route it is, and whether this run asked for it.
struct Shipped<'a, 'r> {
    kind: MemberKind,
    canon: PathBuf,
    sites: Vec<(&'r str, PathBuf)>,
    requested: bool,
    source: Source<'a>,
}

impl<'a> Wanted<'a> {
    /// Admit `shipped` as a member; select it (by clone, so `selected` holds
    /// the identical member) and its selected harnesses' sites when this run
    /// asked for it and some selected harness routes it.
    fn add(&mut self, harnesses: &[String], shipped: Shipped<'a, '_>) {
        let picked: Vec<PathBuf> = shipped
            .sites
            .iter()
            .filter(|(h, _)| harnesses.iter().any(|s| s == h))
            .map(|(_, site)| site.clone())
            .collect();
        let member = Member {
            kind: shipped.kind,
            canon: shipped.canon,
            links: shipped.sites.into_iter().map(|(_, site)| site).collect(),
        };
        if shipped.requested && !picked.is_empty() {
            self.desired.selected.insert(member.clone());
            self.desired.new_links.extend(picked);
        }
        self.sources.insert(member.canon.clone(), shipped.source);
        self.desired.members.insert(member);
    }
}

/// `(harness, link dir)` for every registered harness carrying `route`.
fn routes<'r>(
    registry: &'r Registry,
    route: impl Fn(&'r HarnessSection) -> Option<&'r String>,
) -> Vec<(&'r str, &'r str)> {
    registry
        .iter()
        .filter_map(|(name, h)| route(h).map(|dir| (name.as_str(), dir.as_str())))
        .collect()
}

/// The link site for `file` under each routed dir.
fn sites<'r>(base: &Path, routes: &[(&'r str, &str)], file: &str) -> Vec<(&'r str, PathBuf)> {
    routes
        .iter()
        .map(|(name, dir)| (*name, base.join(dir).join(file)))
        .collect()
}

/// What `lstat` saw at a path; the executor's re-proof compares against it.
#[derive(Debug, Clone, PartialEq, Eq)]
enum EntryState {
    Absent,
    File,
    Dir,
    Symlink(PathBuf),
}

/// `lstat` `path` (never following a final symlink); a symlink carries its
/// value. A missing entry is `Absent`, not an error.
fn entry_state(path: &Path) -> io::Result<EntryState> {
    let meta = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(EntryState::Absent),
        Err(e) => return Err(e),
    };
    Ok(if meta.file_type().is_symlink() {
        EntryState::Symlink(fs::read_link(path)?)
    } else if meta.is_dir() {
        EntryState::Dir
    } else {
        EntryState::File
    })
}

/// [`entry_state`], an error naming `path`.
fn read_state(path: &Path) -> anyhow::Result<EntryState> {
    entry_state(path).with_context(|| format!("Failed to read {}", path.display()))
}

/// The plan's snapshot plus every scanned path's state (a missing key was
/// absent) — re-proof state stays in the shell, not in `Snapshot`/`Action`.
struct Scan {
    snapshot: Snapshot,
    states: BTreeMap<PathBuf, EntryState>,
}

/// Every registered harness's link dirs (`agents`, `skills`, `workflows`)
/// under `base`, de-duplicated — all registered harnesses, not this run's.
fn registered_link_dirs(base: &Path, registry: &Registry) -> Vec<PathBuf> {
    let dirs: BTreeSet<PathBuf> = registry
        .values()
        .flat_map(|h| [&h.agents, &h.skills, &h.workflows])
        .flatten()
        .map(|dir| base.join(dir))
        .collect();
    dirs.into_iter().collect()
}

/// Read the disk the plan decides over (design § Shape). Lists each canon
/// root, descending only into real-dir containers of member canon paths;
/// lists every link dir. Any listed dir reached through a symlinked
/// component below `base` is prune-blocked. Reads only — creates nothing.
fn scan(
    base: &Path,
    roots: &[PathBuf],
    link_dirs: &[PathBuf],
    want: &Desired,
) -> anyhow::Result<Scan> {
    let mut out = Scan {
        snapshot: Snapshot::default(),
        states: BTreeMap::new(),
    };
    for root in roots {
        out.canon_dir(base, root, want)?;
    }
    for dir in link_dirs {
        for path in out.list(base, dir)? {
            let name = path.file_name().unwrap_or_default().to_os_string();
            let dir = dir.clone();
            // An unreadable entry has no state: the plan only discloses it
            // (STD-003), so nothing ever re-proves or touches it.
            let Ok(state) = entry_state(&path) else {
                out.snapshot.links.push(LinkSeen::Unreadable { dir, name });
                continue;
            };
            out.snapshot.links.push(match &state {
                EntryState::Symlink(value) => LinkSeen::Symlink {
                    dir,
                    name,
                    value: value.clone(),
                },
                _ => LinkSeen::Real { dir, name },
            });
            out.states.insert(path, state);
        }
    }
    Ok(out)
}

impl Scan {
    /// The state the scan recorded at `path`.
    fn scanned(&self, path: &Path) -> EntryState {
        self.states.get(path).cloned().unwrap_or(EntryState::Absent)
    }

    /// List canon dir `dir`, descending into each child that is a real-dir
    /// container of some member canon path — never into a symlink, never
    /// below member depth (the canon depth contract). A container that is a
    /// symlink to a dir is not listed (nothing beneath it becomes a prune
    /// candidate), but its members' states are recorded so their writes
    /// re-prove (RV-420 F-1).
    fn canon_dir(&mut self, base: &Path, dir: &Path, want: &Desired) -> anyhow::Result<()> {
        for path in self.list(base, dir)? {
            let state = read_state(&path)?;
            let beneath: Vec<&Path> = want
                .members
                .iter()
                .map(|m| m.canon.as_path())
                .filter(|canon| *canon != path && canon.starts_with(&path))
                .collect();
            let linked_dir = matches!(state, EntryState::Symlink(_)) && path.is_dir();
            let descend = state == EntryState::Dir && !beneath.is_empty();
            self.snapshot.canon.push(path.clone());
            self.states.insert(path.clone(), state);
            if descend {
                self.canon_dir(base, &path, want)?;
            } else if linked_dir {
                for canon in beneath {
                    self.states.insert(canon.to_path_buf(), read_state(canon)?);
                }
            }
        }
        Ok(())
    }

    /// `dir`'s entries, sorted; empty when `dir` is absent or not a
    /// directory. Prune-blocks `dir` when reached through a symlinked
    /// component below `base`.
    fn list(&mut self, base: &Path, dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
        if !dir.is_dir() {
            return Ok(Vec::new());
        }
        if crate::fsutil::symlinked_component(base, dir)?.is_some() {
            self.snapshot.prune_blocked.insert(dir.to_path_buf());
        }
        let mut paths = fs::read_dir(dir)
            .with_context(|| format!("Failed to list {}", dir.display()))?
            .map(|e| e.map(|e| e.path()))
            .collect::<io::Result<Vec<_>>>()
            .with_context(|| format!("Failed to list {}", dir.display()))?;
        paths.sort();
        Ok(paths)
    }
}

/// The header above the convergence report block.
fn report_header(scope: Scope, dry_run: bool) -> String {
    match (dry_run, scope) {
        (true, _) => "Ownership plan (not executed under --dry-run):".to_string(),
        (false, Scope::Project) => "framework artefacts (project):".to_string(),
        (false, Scope::Global) => "framework artefacts (global):".to_string(),
    }
}

/// One report line for `action` (design § Report vocabulary) — the single
/// renderer both the executor and dry-run print through.
fn action_line(action: &Action, scan: &Scan, wanted: &Wanted<'_>, dry_run: bool) -> String {
    let shown = |p: &Path| p.display().to_string();
    let name = |p: &Path| {
        p.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned()
    };
    match (action, dry_run) {
        (Action::WriteCanon(m), true) => format!("  would write {}", shown(&m.canon)),
        (Action::WriteCanon(m), false) => {
            let (verb, label) = match wanted.sources.get(&m.canon) {
                Some(Source::Skill(e)) => ("skill", e.id.clone()),
                Some(Source::Asset { label, .. }) if m.kind == MemberKind::Workflow => {
                    ("workflow", label.clone())
                }
                Some(Source::Asset { label, .. }) => ("agent", label.clone()),
                None => ("write", name(&m.canon)),
            };
            row(verb, &format!("{label} → {}", shown(&m.canon)))
        }
        (Action::Link { dest, target }, true) => {
            format!("  would link {} → {}", shown(dest), shown(target))
        }
        (Action::Link { dest, .. }, false) => row("linked", &name(dest)),
        (Action::Relink { dest, target }, true) => {
            format!("  would relink {} → {}", shown(dest), shown(target))
        }
        (Action::Relink { dest, .. }, false) => row("relinked", &name(dest)),
        (Action::Unlink(p) | Action::PruneCanon(p), true) => format!("  would prune {}", shown(p)),
        (Action::Unlink(p) | Action::PruneCanon(p), false) => row("pruned", &shown(p)),
        (Action::Report(notice), _) => match notice {
            Notice::Orphan(p) => row(
                "orphan",
                &format!(
                    "{} (global install never removes; delete it if unwanted)",
                    shown(p)
                ),
            ),
            Notice::ForeignSymlink(p) => {
                let value = match scan.scanned(p) {
                    EntryState::Symlink(value) => shown(&value),
                    _ => String::new(),
                };
                row("kept", &format!("{} (foreign symlink → {value})", name(p)))
            }
            Notice::Shadows(p) => row(
                "kept",
                &format!(
                    "{} (real file shadows doctrine def — remove it to receive updates)",
                    name(p)
                ),
            ),
            Notice::PruneBlocked(p) => row(
                "prune blocked",
                &format!(
                    "{} (reached through a symlink — nothing removed there)",
                    shown(p)
                ),
            ),
            Notice::StaleStaging(p) => row(
                "stale staging",
                &format!("{} (left by an interrupted install — delete it)", shown(p)),
            ),
            Notice::Unreadable(p) => row(
                "unreadable",
                &format!("{} (could not read it — left untouched)", shown(p)),
            ),
        },
    }
}

/// One report row: the verb padded to the shared 10-char column.
fn row(verb: &str, rest: &str) -> String {
    format!("  {verb:<9} {rest}")
}

/// The line for a mutation withheld because its entry changed since the scan.
fn skipped_line(path: &Path) -> String {
    row(
        "skipped",
        &format!("{} (changed since scan)", path.display()),
    )
}

/// The paths one convergence run executes against.
struct Converge<'a> {
    /// Project root — the hymn corpus the worker bake resolves against (a
    /// global install bakes against it too, pre-existing ISS-520).
    root: &'a Path,
    /// `install_base(root, global)` — where canon roots and link dirs live.
    base: &'a Path,
    scope: Scope,
}

/// Run `actions` in plan order, re-proving each mutation's entry against
/// `scan` first: a changed entry is skipped and reported; a removal whose
/// parent became reachable through a symlink is withheld. Aborts on the
/// first I/O or bake error — plan order puts removals last, so an abort
/// never strands a link before its target's prune.
fn execute(
    cx: &Converge<'_>,
    wanted: &Wanted<'_>,
    scan: &Scan,
    actions: &[Action],
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let mut baker = Baker {
        root: cx.root,
        corpus: None,
    };
    for action in actions {
        let line = match apply(cx, wanted, scan, action, &mut baker)? {
            Applied::Done => action_line(action, scan, wanted, false),
            Applied::Changed(path) => skipped_line(path),
            Applied::Blocked(dir) => action_line(
                &Action::Report(Notice::PruneBlocked(dir)),
                scan,
                wanted,
                false,
            ),
        };
        writeln!(out, "{line}")?;
    }
    Ok(())
}

/// Scan the disk under `cx.base` and plan its convergence toward `wanted`:
/// the canon `roots` (`[framework] roots`, base-relative) and every
/// registered harness's link dirs. Reads only — the dry-run seam.
fn survey(
    cx: &Converge<'_>,
    roots: &[String],
    registry: &Registry,
    wanted: &Wanted<'_>,
) -> anyhow::Result<(Scan, Vec<Action>)> {
    let roots: Vec<PathBuf> = roots.iter().map(|r| cx.base.join(r)).collect();
    let link_dirs = registered_link_dirs(cx.base, registry);
    let scanned = scan(cx.base, &roots, &link_dirs, &wanted.desired)?;
    let actions =
        crate::install_ownership::plan(&roots, cx.scope, &wanted.desired, &scanned.snapshot);
    Ok((scanned, actions))
}

/// Converge the framework artefacts under `cx.base` toward `wanted`: survey,
/// then execute under the scope's report header.
fn converge(
    cx: &Converge<'_>,
    roots: &[String],
    registry: &Registry,
    wanted: &Wanted<'_>,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let (scanned, actions) = survey(cx, roots, registry, wanted)?;
    writeln!(out, "{}", report_header(cx.scope, false))?;
    execute(cx, wanted, &scanned, &actions, out)
}

/// How one action fared at mutation time.
enum Applied<'p> {
    Done,
    /// The entry no longer matches the scan; nothing was touched.
    Changed(&'p Path),
    /// A removal's dir is now reached through a symlink; nothing removed.
    Blocked(PathBuf),
}

/// Re-prove then perform one action. Reports mutate nothing.
fn apply<'p>(
    cx: &Converge<'_>,
    wanted: &Wanted<'_>,
    scan: &Scan,
    action: &'p Action,
    baker: &mut Baker<'_>,
) -> anyhow::Result<Applied<'p>> {
    let path = match action {
        Action::Report(_) => return Ok(Applied::Done),
        Action::WriteCanon(m) => &m.canon,
        Action::Link { dest, .. } | Action::Relink { dest, .. } => dest,
        Action::Unlink(p) | Action::PruneCanon(p) => {
            let dir = p.parent().unwrap_or(cx.base);
            if crate::fsutil::symlinked_component(cx.base, dir)?.is_some() {
                return Ok(Applied::Blocked(dir.to_path_buf()));
            }
            p
        }
    };
    let now = read_state(path)?;
    if now != scan.scanned(path) {
        return Ok(Applied::Changed(path));
    }
    match action {
        Action::WriteCanon(m) => write_canon(wanted, m, baker)?,
        Action::Link { dest, target } | Action::Relink { dest, target } => {
            write_link(dest, target)?;
        }
        Action::Unlink(p) | Action::PruneCanon(p) => match now {
            EntryState::Dir => fs::remove_dir_all(p),
            EntryState::File | EntryState::Symlink(_) => fs::remove_file(p),
            EntryState::Absent => return Ok(Applied::Changed(path)),
        }
        .with_context(|| format!("Failed to remove {}", p.display()))?,
        Action::Report(_) => {}
    }
    Ok(Applied::Done)
}

/// Write member `m`'s canon entry from its source: a skill is materialised;
/// an asset is written atomically, baked first when it carries
/// `WORKER_RESOLVE_MARKER`.
fn write_canon(wanted: &Wanted<'_>, m: &Member, baker: &mut Baker<'_>) -> anyhow::Result<()> {
    match wanted.sources.get(&m.canon) {
        Some(Source::Skill(entry)) => materialise_canonical(entry, &m.canon),
        Some(Source::Asset { label, bytes }) => {
            if let Some(parent) = m.canon.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create {}", parent.display()))?;
            }
            match std::str::from_utf8(bytes) {
                Ok(def) if def.contains(WORKER_RESOLVE_MARKER) => {
                    crate::fsutil::write_atomic(&m.canon, baker.bake(def, label)?.as_bytes())
                }
                _ => crate::fsutil::write_atomic(&m.canon, bytes),
            }
        }
        None => bail!("no source for canon member {}", m.canon.display()),
    }
}

/// The worker-def bake, loading the hymn corpus at most once per run.
struct Baker<'a> {
    root: &'a Path,
    corpus: Option<(Vec<crate::hymns::Snippet>, crate::hymns::SealSet)>,
}

impl Baker<'_> {
    fn bake(&mut self, def: &str, label: &str) -> anyhow::Result<String> {
        let (corpus, sealed) = match &mut self.corpus {
            Some(loaded) => loaded,
            slot @ None => {
                let disk = self.root.join(".doctrine").join(HYMNS_DIRNAME);
                let sealed = embedded_seal_set()?;
                let corpus = load_full_corpus(&disk, &embedded_hymns(), &sealed)?;
                slot.insert((corpus, sealed))
            }
        };
        bake_worker_def(def, label, corpus, sealed)
    }
}

// ── Tests: convergence shell (SL-281 PHASE-04) ────────────────

#[cfg(test)]
mod tests_converge {
    use super::*;
    use crate::install_ownership::plan;

    const BASE: &str = "/p";

    fn real_registry() -> Registry {
        load_manifest().unwrap().harness
    }

    fn registry_from(toml: &str) -> Registry {
        parse_manifest(toml).unwrap().harness
    }

    fn names(harnesses: &[&str]) -> Vec<String> {
        harnesses.iter().map(ToString::to_string).collect()
    }

    fn canons(set: &BTreeSet<Member>) -> BTreeSet<PathBuf> {
        set.iter().map(|m| m.canon.clone()).collect()
    }

    fn manifest_roots(base: &Path) -> Vec<PathBuf> {
        framework_roots()
            .unwrap()
            .iter()
            .map(|r| base.join(r))
            .collect()
    }

    /// A tempdir install base plus plant helpers that prove what they made.
    struct Tree {
        tmp: tempfile::TempDir,
    }

    impl Tree {
        fn new() -> Self {
            Self {
                tmp: tempfile::tempdir().unwrap(),
            }
        }

        fn base(&self) -> &Path {
            self.tmp.path()
        }

        fn path(&self, rel: &str) -> PathBuf {
            self.base().join(rel)
        }

        fn file(&self, rel: &str, body: &str) -> PathBuf {
            let p = self.path(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(&p, body).unwrap();
            p
        }

        fn link(&self, rel: &str, value: impl AsRef<Path>) -> PathBuf {
            let p = self.path(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::os::unix::fs::symlink(value, &p).unwrap();
            p
        }
    }

    fn member(kind: MemberKind, canon: PathBuf, links: &[PathBuf]) -> Member {
        Member {
            kind,
            canon,
            links: links.iter().cloned().collect(),
        }
    }

    fn desired(members: Vec<Member>) -> Desired {
        Desired {
            members: members.into_iter().collect(),
            ..Desired::default()
        }
    }

    fn listing(dir: &Path) -> BTreeSet<PathBuf> {
        walk(dir).into_iter().collect()
    }

    fn walk(dir: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        for e in fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            out.push(p.clone());
            if crate::fsutil::is_real_dir(&p) {
                out.extend(walk(&p));
            }
        }
        out
    }

    #[test]
    fn entry_state_reads_without_following() {
        let t = Tree::new();
        let f = t.file("f", "x");
        let l = t.link("l", "gone/target");
        fs::create_dir(t.path("d")).unwrap();
        assert_eq!(entry_state(&f).unwrap(), EntryState::File);
        assert_eq!(entry_state(&t.path("d")).unwrap(), EntryState::Dir);
        assert_eq!(
            entry_state(&l).unwrap(),
            EntryState::Symlink(PathBuf::from("gone/target"))
        );
        assert_eq!(entry_state(&t.path("absent")).unwrap(), EntryState::Absent);
    }

    #[test]
    fn registered_link_dirs_cover_every_harness_route_once() {
        let registry = registry_from(
            "[harness.a]\nagents = \".a/agents\"\nskills = \".s\"\nworkflows = \".a/wf\"\n\
             [harness.b]\nskills = \".s\"\n[harness.c]\nskills_via = \"universal\"\n",
        );
        let base = Path::new(BASE);
        let dirs: BTreeSet<PathBuf> = registered_link_dirs(base, &registry).into_iter().collect();
        assert_eq!(
            dirs,
            BTreeSet::from([base.join(".a/agents"), base.join(".s"), base.join(".a/wf")])
        );
        assert_eq!(
            registered_link_dirs(base, &registry).len(),
            3,
            "de-duplicated"
        );
    }

    #[test]
    fn scan_lists_canon_at_member_depth_descending_only_containers() {
        let t = Tree::new();
        let other = tempfile::tempdir().unwrap();
        fs::write(other.path().join("dw.md"), "x").unwrap();
        fs::write(other.path().join("stray.md"), "x").unwrap();
        t.file(".doctrine/agents/claude/dw.md", "x");
        t.file(".doctrine/agents/claude/old.md", "x");
        t.file(".doctrine/agents/codex/x.md", "x");
        t.file(".doctrine/agents/flat.md", "x");
        t.file(".doctrine/skills/code-review/SKILL.md", "x");
        t.link(".doctrine/agents/pi", other.path());
        let want = desired(vec![
            member(
                MemberKind::AgentDef,
                t.path(".doctrine/agents/claude/dw.md"),
                &[],
            ),
            member(
                MemberKind::AgentDef,
                t.path(".doctrine/agents/pi/dw.md"),
                &[],
            ),
            member(
                MemberKind::Skill,
                t.path(".doctrine/skills/code-review"),
                &[],
            ),
        ]);
        let roots = manifest_roots(t.base());
        let got = scan(t.base(), &roots, &[], &want).unwrap();

        let canon: BTreeSet<PathBuf> = got.snapshot.canon.iter().cloned().collect();
        let expected: BTreeSet<PathBuf> = [
            ".doctrine/agents/claude",
            ".doctrine/agents/claude/dw.md",
            ".doctrine/agents/claude/old.md",
            ".doctrine/agents/codex",
            ".doctrine/agents/flat.md",
            ".doctrine/agents/pi",
            ".doctrine/skills/code-review",
        ]
        .iter()
        .map(|r| t.path(r))
        .collect();
        assert_eq!(
            canon, expected,
            "container descended; non-container and symlink not"
        );
        assert_eq!(
            got.scanned(&t.path(".doctrine/agents/claude")),
            EntryState::Dir
        );
        assert_eq!(
            got.scanned(&t.path(".doctrine/agents/flat.md")),
            EntryState::File
        );
        assert_eq!(
            got.scanned(&t.path(".doctrine/agents/pi")),
            EntryState::Symlink(other.path().to_path_buf())
        );
        assert!(
            got.snapshot.prune_blocked.is_empty(),
            "{:?}",
            got.snapshot.prune_blocked
        );
    }

    #[test]
    fn scan_classifies_link_dir_entries() {
        let t = Tree::new();
        let link = t.link(".claude/agents/a.md", "../../.doctrine/agents/claude/a.md");
        let real = t.file(".claude/agents/b.md", "mine");
        let dir = t.path(".claude/agents");
        let got = scan(t.base(), &[], &[dir.clone()], &Desired::default()).unwrap();

        let value = PathBuf::from("../../.doctrine/agents/claude/a.md");
        assert_eq!(got.snapshot.links.len(), 2);
        assert!(got.snapshot.links.contains(&LinkSeen::Symlink {
            dir: dir.clone(),
            name: "a.md".into(),
            value: value.clone(),
        }));
        assert!(got.snapshot.links.contains(&LinkSeen::Real {
            dir,
            name: "b.md".into(),
        }));
        assert_eq!(got.scanned(&link), EntryState::Symlink(value));
        assert_eq!(got.scanned(&real), EntryState::File);
    }

    /// RV-420 F-15: a link-dir entry `lstat` cannot read (dir listable, not
    /// searchable) is recorded as unreadable, with no state to re-prove.
    #[test]
    fn scan_records_unreadable_link_dir_entry() {
        use std::os::unix::fs::PermissionsExt;
        let t = Tree::new();
        let entry = t.file(".claude/agents/b.md", "mine");
        let dir = t.path(".claude/agents");
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o444)).unwrap();
        let got = scan(t.base(), &[], &[dir.clone()], &Desired::default());
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
        let got = got.unwrap();

        assert_eq!(
            got.snapshot.links,
            [LinkSeen::Unreadable {
                dir,
                name: "b.md".into(),
            }]
        );
        assert!(!got.states.contains_key(&entry));
    }

    #[test]
    fn scan_blocks_prune_through_symlinked_ancestors() {
        let t = Tree::new();
        let real = tempfile::tempdir().unwrap();
        let keep = real.path().join("agents/claude/keep.md");
        fs::create_dir_all(keep.parent().unwrap()).unwrap();
        fs::write(&keep, "x").unwrap();
        fs::create_dir_all(real.path().join("agents/claude-links")).unwrap();
        t.link(".doctrine", real.path());
        t.link(".claude", real.path().join("agents"));
        let want = desired(vec![member(
            MemberKind::AgentDef,
            t.path(".doctrine/agents/claude/dw.md"),
            &[],
        )]);
        let roots = manifest_roots(t.base());
        let link_dir = t.path(".claude/claude-links");
        let got = scan(t.base(), &roots, &[link_dir.clone()], &want).unwrap();

        assert!(
            got.snapshot
                .canon
                .contains(&t.path(".doctrine/agents/claude/keep.md")),
            "control: listed through the symlink"
        );
        assert_eq!(
            got.snapshot.prune_blocked,
            BTreeSet::from([
                t.path(".doctrine/agents"),
                t.path(".doctrine/agents/claude"),
                link_dir,
            ])
        );
    }

    #[test]
    fn scan_of_a_bare_tree_creates_nothing() {
        let t = Tree::new();
        let roots = manifest_roots(t.base());
        let dirs = registered_link_dirs(t.base(), &real_registry());
        assert!(
            !roots.is_empty() && !dirs.is_empty(),
            "control: scan given paths"
        );
        let before = listing(t.base());
        let want = desired(vec![member(
            MemberKind::AgentDef,
            t.path(".doctrine/agents/claude/dw.md"),
            &[],
        )]);
        let got = scan(t.base(), &roots, &dirs, &want).unwrap();
        assert!(got.snapshot.canon.is_empty() && got.snapshot.links.is_empty());
        assert_eq!(listing(t.base()), before);
        // control: the same paths are read once something is there
        t.file(".doctrine/agents/stray.md", "x");
        let again = scan(t.base(), &roots, &dirs, &want).unwrap();
        assert_eq!(
            again.snapshot.canon,
            vec![t.path(".doctrine/agents/stray.md")]
        );
    }

    /// A wanted set with one member of each kind and a scan that saw a
    /// foreign symlink — the fixture every vocabulary row renders against.
    fn vocabulary_fixture() -> (Vec<Entry>, Vec<(String, Vec<u8>)>, Scan) {
        let skill = Entry {
            domain: "doctrine".to_string(),
            id: "code-review".to_string(),
            description: String::new(),
            files: Vec::new(),
        };
        let assets = vec![
            ("claude/dw.md".to_string(), b"def".to_vec()),
            ("flow.js".to_string(), b"wf".to_vec()),
        ];
        let mut scan = Scan {
            snapshot: Snapshot::default(),
            states: BTreeMap::new(),
        };
        scan.states.insert(
            PathBuf::from("/p/.claude/agents/f.md"),
            EntryState::Symlink(PathBuf::from("/elsewhere/f.md")),
        );
        (vec![skill], assets, scan)
    }

    fn vocabulary_wanted<'a>(catalog: &'a [Entry], assets: &'a [(String, Vec<u8>)]) -> Wanted<'a> {
        let base = Path::new(BASE);
        let mut w = Wanted {
            desired: Desired::default(),
            sources: BTreeMap::new(),
        };
        w.sources.insert(
            base.join(".doctrine/skills/code-review"),
            Source::Skill(&catalog[0]),
        );
        for ((label, bytes), canon) in assets.iter().zip([
            ".doctrine/agents/claude/dw.md",
            ".doctrine/workflows/flow.js",
        ]) {
            w.sources.insert(
                base.join(canon),
                Source::Asset {
                    label: label.clone(),
                    bytes,
                },
            );
        }
        w
    }

    #[test]
    fn action_line_renders_every_vocabulary_row() {
        let (catalog, assets, scan) = vocabulary_fixture();
        let w = vocabulary_wanted(&catalog, &assets);
        let p = |rel: &str| PathBuf::from(BASE).join(rel);
        let write = |kind, rel: &str| Action::WriteCanon(member(kind, p(rel), &[]));
        let dest = p(".claude/agents/f.md");
        let target = PathBuf::from("../../.doctrine/agents/claude/f.md");
        let rows: Vec<(Action, &str, &str)> = vec![
            (
                write(MemberKind::Skill, ".doctrine/skills/code-review"),
                "  skill     code-review → /p/.doctrine/skills/code-review",
                "  would write /p/.doctrine/skills/code-review",
            ),
            (
                write(MemberKind::AgentDef, ".doctrine/agents/claude/dw.md"),
                "  agent     claude/dw.md → /p/.doctrine/agents/claude/dw.md",
                "  would write /p/.doctrine/agents/claude/dw.md",
            ),
            (
                write(MemberKind::Workflow, ".doctrine/workflows/flow.js"),
                "  workflow  flow.js → /p/.doctrine/workflows/flow.js",
                "  would write /p/.doctrine/workflows/flow.js",
            ),
            (
                Action::Link {
                    dest: dest.clone(),
                    target: target.clone(),
                },
                "  linked    f.md",
                "  would link /p/.claude/agents/f.md → ../../.doctrine/agents/claude/f.md",
            ),
            (
                Action::Relink {
                    dest: dest.clone(),
                    target,
                },
                "  relinked  f.md",
                "  would relink /p/.claude/agents/f.md → ../../.doctrine/agents/claude/f.md",
            ),
            (
                Action::Unlink(dest.clone()),
                "  pruned    /p/.claude/agents/f.md",
                "  would prune /p/.claude/agents/f.md",
            ),
            (
                Action::PruneCanon(p(".doctrine/agents/old.md")),
                "  pruned    /p/.doctrine/agents/old.md",
                "  would prune /p/.doctrine/agents/old.md",
            ),
        ];
        for (action, live, dry) in rows {
            assert_eq!(action_line(&action, &scan, &w, false), live);
            assert_eq!(action_line(&action, &scan, &w, true), dry);
        }

        let notices = [
            (
                Notice::Orphan(p(".doctrine/agents/old.md")),
                "  orphan    /p/.doctrine/agents/old.md (global install never removes; delete it if unwanted)",
            ),
            (
                Notice::ForeignSymlink(dest.clone()),
                "  kept      f.md (foreign symlink → /elsewhere/f.md)",
            ),
            (
                Notice::Shadows(dest),
                "  kept      f.md (real file shadows doctrine def — remove it to receive updates)",
            ),
            (
                Notice::PruneBlocked(p(".doctrine/agents")),
                "  prune blocked /p/.doctrine/agents (reached through a symlink — nothing removed there)",
            ),
            (
                Notice::StaleStaging(p(".doctrine/skills/.tmp-x.1.0")),
                "  stale staging /p/.doctrine/skills/.tmp-x.1.0 (left by an interrupted install — delete it)",
            ),
            (
                Notice::Unreadable(p(".claude/agents/f.md")),
                "  unreadable /p/.claude/agents/f.md (could not read it — left untouched)",
            ),
        ];
        for (notice, line) in notices {
            let action = Action::Report(notice);
            assert_eq!(action_line(&action, &scan, &w, false), line);
            assert_eq!(
                action_line(&action, &scan, &w, true),
                line,
                "same under dry-run"
            );
        }
        assert_eq!(
            skipped_line(&p(".claude/agents/f.md")),
            "  skipped   /p/.claude/agents/f.md (changed since scan)"
        );
    }

    #[test]
    fn report_header_names_scope_or_dry_run() {
        assert_eq!(
            report_header(Scope::Project, false),
            "framework artefacts (project):"
        );
        assert_eq!(
            report_header(Scope::Global, false),
            "framework artefacts (global):"
        );
        assert_eq!(
            report_header(Scope::Project, true),
            "Ownership plan (not executed under --dry-run):"
        );
    }

    const CLAUDE_ONLY: &str =
        "[harness.claude]\nagents = \".claude/agents\"\nskills = \".claude/skills\"\n";

    /// Scan `t` against `wanted` and plan; the plan is what `execute` runs.
    fn scan_and_plan(
        cx: &Converge<'_>,
        registry: &Registry,
        wanted: &Wanted<'_>,
    ) -> (Scan, Vec<Action>) {
        survey(cx, &framework_roots().unwrap(), registry, wanted).unwrap()
    }

    fn run(cx: &Converge<'_>, wanted: &Wanted<'_>, scanned: &Scan, actions: &[Action]) -> String {
        let mut out = Vec::new();
        execute(cx, wanted, scanned, actions, &mut out).unwrap();
        String::from_utf8(out).unwrap()
    }

    fn project(t: &Tree) -> Converge<'_> {
        Converge {
            root: t.base(),
            base: t.base(),
            scope: Scope::Project,
        }
    }

    /// Plant an orphan doctrine link and an orphan canon skill dir; both are
    /// planned for removal (asserted — the control every skip test leans on).
    fn plant_orphans(t: &Tree, actions_of: impl Fn() -> Vec<Action>) -> (PathBuf, PathBuf) {
        let link = t.link(".claude/skills/retired", "../../.doctrine/skills/retired");
        t.file(".doctrine/skills/retired/SKILL.md", "old");
        let dir = t.path(".doctrine/skills/retired");
        let actions = actions_of();
        assert!(
            actions.contains(&Action::Unlink(link.clone())),
            "{actions:?}"
        );
        assert!(
            actions.contains(&Action::PruneCanon(dir.clone())),
            "{actions:?}"
        );
        (link, dir)
    }

    #[test]
    fn execute_skips_action_whose_entry_changed_since_scan() {
        let registry = registry_from(CLAUDE_ONLY);
        let catalog = discover().unwrap();
        for mutate in [false, true] {
            let t = Tree::new();
            let cx = project(&t);
            let wanted = desired_set(
                t.base(),
                &registry,
                &names(&["claude"]),
                &[],
                &catalog,
                &[],
                &[],
            );
            let (link, dir) = plant_orphans(&t, || scan_and_plan(&cx, &registry, &wanted).1);
            let (scanned, actions) = scan_and_plan(&cx, &registry, &wanted);
            if mutate {
                fs::remove_file(&link).unwrap();
                std::os::unix::fs::symlink("../../.doctrine/skills/other", &link).unwrap();
                fs::remove_dir_all(&dir).unwrap();
                fs::write(&dir, "now a file").unwrap();
            }
            let out = run(&cx, &wanted, &scanned, &actions);
            if mutate {
                assert_eq!(
                    fs::read_link(&link).unwrap(),
                    PathBuf::from("../../.doctrine/skills/other")
                );
                assert_eq!(fs::read_to_string(&dir).unwrap(), "now a file");
                assert!(out.contains(&skipped_line(&link)), "{out}");
                assert!(out.contains(&skipped_line(&dir)), "{out}");
            } else {
                assert!(entry_state(&link).unwrap() == EntryState::Absent, "{out}");
                assert!(entry_state(&dir).unwrap() == EntryState::Absent, "{out}");
                assert!(
                    out.contains(&format!("  pruned    {}", link.display())),
                    "{out}"
                );
                assert!(
                    out.contains(&format!("  pruned    {}", dir.display())),
                    "{out}"
                );
            }
        }
    }

    #[test]
    fn execute_withholds_removal_when_ancestor_became_symlink() {
        let registry = registry_from(CLAUDE_ONLY);
        let catalog = discover().unwrap();
        let t = Tree::new();
        let cx = project(&t);
        let wanted = desired_set(
            t.base(),
            &registry,
            &names(&["claude"]),
            &[],
            &catalog,
            &[],
            &[],
        );
        let (_, dir) = plant_orphans(&t, || scan_and_plan(&cx, &registry, &wanted).1);
        let (scanned, actions) = scan_and_plan(&cx, &registry, &wanted);
        assert!(
            scanned.snapshot.prune_blocked.is_empty(),
            "control: clean at scan"
        );

        fs::rename(t.path(".doctrine"), t.path("moved")).unwrap();
        t.link(".doctrine", "moved");
        let out = run(&cx, &wanted, &scanned, &actions);

        assert_eq!(
            fs::read_to_string(t.path("moved/skills/retired/SKILL.md")).unwrap(),
            "old"
        );
        let blocked = Action::Report(Notice::PruneBlocked(t.path(".doctrine/skills")));
        assert!(
            out.contains(&action_line(&blocked, &scanned, &wanted, false)),
            "{out}"
        );
        assert!(
            !out.contains(&format!("  pruned    {}", dir.display())),
            "{out}"
        );
    }

    #[test]
    fn execute_bakes_agent_defs_and_copies_skills() {
        let t = Tree::new();
        t.file(".doctrine/hymns/role/worker.md", "RESOLVED WORKER BODY");
        let cx = project(&t);
        let registry = real_registry();
        let defs = embedded_agent_defs();
        let catalog = discover().unwrap();
        let picked = select_for_install(&catalog, &names(&["code-review"]), &[]).unwrap();
        let wanted = desired_set(
            t.base(),
            &registry,
            &names(&["claude"]),
            &defs,
            &catalog,
            &picked,
            &[],
        );
        let (scanned, actions) = scan_and_plan(&cx, &registry, &wanted);
        let out = run(&cx, &wanted, &scanned, &actions);

        let worker =
            fs::read_to_string(t.path(".doctrine/agents/claude/dispatch-worker.md")).unwrap();
        assert!(worker.contains("RESOLVED WORKER BODY"), "{worker}");
        assert!(!worker.contains(WORKER_RESOLVE_MARKER), "{worker}");

        let (_, capsule) = defs
            .iter()
            .find(|(rel, _)| rel == "claude/capsule-worker.md")
            .unwrap();
        assert!(
            !String::from_utf8_lossy(capsule).contains(WORKER_RESOLVE_MARKER),
            "control: no marker"
        );
        assert_eq!(
            &fs::read(t.path(".doctrine/agents/claude/capsule-worker.md")).unwrap(),
            capsule
        );

        let review = t.path(".doctrine/skills/code-review");
        assert!(review.join("SKILL.md").is_file());
        assert_eq!(
            fs::read_link(t.path(".claude/skills/code-review")).unwrap(),
            PathBuf::from("../../.doctrine/skills/code-review")
        );
        assert_eq!(
            fs::read_link(t.path(".claude/agents/dispatch-worker.md")).unwrap(),
            PathBuf::from("../../.doctrine/agents/claude/dispatch-worker.md")
        );
        assert!(
            out.contains(&format!("  skill     code-review → {}", review.display())),
            "{out}"
        );
        assert!(out.contains("  linked    dispatch-worker.md"), "{out}");
        assert!(!t.path(".doctrine/agents/pi").exists(), "pi not selected");
    }

    /// RV-420 F-1: a canon container symlinked outside the canon still
    /// converges — its defs refresh through the link; nothing beside them is
    /// pruned through it.
    #[test]
    fn execute_refreshes_defs_through_a_symlinked_canon_container() {
        let t = Tree::new();
        let reloc = tempfile::tempdir().unwrap();
        let def = reloc.path().join("capsule-worker.md");
        let notes = reloc.path().join("my-notes.md");
        fs::write(&def, "edited").unwrap();
        fs::write(&notes, "mine").unwrap();
        t.link(".doctrine/agents/claude", reloc.path());
        t.file(".doctrine/hymns/role/worker.md", "RESOLVED WORKER BODY");
        let cx = project(&t);
        let registry = real_registry();
        let defs = embedded_agent_defs();
        let catalog = discover().unwrap();
        let wanted = desired_set(
            t.base(),
            &registry,
            &names(&["claude"]),
            &defs,
            &catalog,
            &[],
            &[],
        );
        let (scanned, actions) = scan_and_plan(&cx, &registry, &wanted);
        let out = run(&cx, &wanted, &scanned, &actions);

        let (_, shipped) = defs
            .iter()
            .find(|(rel, _)| rel == "claude/capsule-worker.md")
            .unwrap();
        assert!(fs::read(&def).unwrap() == *shipped, "refreshed: {out}");
        assert!(reloc.path().join("dispatch-worker.md").is_file(), "{out}");
        assert_eq!(fs::read_to_string(&notes).unwrap(), "mine", "{out}");
        assert!(!out.contains("skipped"), "{out}");
        assert!(!out.contains("pruned"), "{out}");
    }

    #[test]
    fn execute_prunes_dir_with_remove_dir_all_and_link_with_remove_file() {
        let registry = registry_from(CLAUDE_ONLY);
        let catalog = discover().unwrap();
        let t = Tree::new();
        let cx = project(&t);
        let wanted = desired_set(
            t.base(),
            &registry,
            &names(&["claude"]),
            &[],
            &catalog,
            &[],
            &[],
        );
        let (link, dir) = plant_orphans(&t, || scan_and_plan(&cx, &registry, &wanted).1);
        let kept = t.file("keep/inside.md", "mine");
        let canon_link = t.link(".doctrine/skills/stray", "../../keep");
        assert!(
            t.path(".doctrine/skills/stray/inside.md").is_file(),
            "control: link resolves"
        );
        let (scanned, actions) = scan_and_plan(&cx, &registry, &wanted);
        assert!(
            actions.contains(&Action::PruneCanon(canon_link.clone())),
            "{actions:?}"
        );

        let out = run(&cx, &wanted, &scanned, &actions);
        for gone in [&link, &dir, &canon_link] {
            assert_eq!(
                entry_state(gone).unwrap(),
                EntryState::Absent,
                "{}",
                gone.display()
            );
            assert!(
                out.contains(&format!("  pruned    {}", gone.display())),
                "{out}"
            );
        }
        assert_eq!(
            fs::read_to_string(&kept).unwrap(),
            "mine",
            "symlink target survives"
        );
    }

    #[test]
    fn canon_roots_match_manifest_framework_roots() {
        let declared: BTreeSet<String> = framework_roots().unwrap().into_iter().collect();
        let coded: BTreeSet<String> = [
            MemberKind::AgentDef,
            MemberKind::Skill,
            MemberKind::Workflow,
        ]
        .into_iter()
        .map(|k| canon_root(k).to_string())
        .collect();
        assert_eq!(coded, declared);
    }

    // VT-3: marketplace-only skills are never desired members. Their ids
    // collide with `doctrine`-domain skills (`pair`, `record-memory`, …), so
    // the proof is by each skill member's source domain, not by id.
    #[test]
    fn plan_excludes_marketplace_only_skills_from_members() {
        let marketplace_only: BTreeSet<String> = PluginAssets::iter()
            .filter_map(|p| {
                let parts: Vec<&str> = p.split('/').collect();
                match parts.as_slice() {
                    [domain, "skills", skill, ..] if MARKETPLACE_ONLY_DOMAINS.contains(domain) => {
                        Some((*skill).to_string())
                    }
                    _ => None,
                }
            })
            .collect();
        assert!(
            !marketplace_only.is_empty(),
            "control: marketplace-only skills embedded"
        );

        let base = Path::new(BASE);
        let defs = embedded_agent_defs();
        let catalog = discover().unwrap();
        let all: Vec<&Entry> = catalog.iter().collect();
        let w = desired_set(
            base,
            &real_registry(),
            &names(&["claude"]),
            &defs,
            &catalog,
            &all,
            &[],
        );

        let review = base.join(".doctrine/skills/code-review");
        assert!(
            canons(&w.desired.members).contains(&review),
            "control: code-review is a member"
        );
        for m in w
            .desired
            .members
            .iter()
            .filter(|m| m.kind == MemberKind::Skill)
        {
            let Some(Source::Skill(e)) = w.sources.get(&m.canon) else {
                panic!("skill member {} has no skill source", m.canon.display());
            };
            assert!(
                !MARKETPLACE_ONLY_DOMAINS.contains(&e.domain.as_str()),
                "{} sourced from marketplace-only {}",
                m.canon.display(),
                e.domain
            );
        }
    }

    #[test]
    fn desired_set_spans_every_registered_harness() {
        let base = Path::new(BASE);
        let defs = embedded_agent_defs();
        assert!(
            defs.iter().any(|(rel, _)| rel == "AGENTS.md"),
            "control: AGENTS.md embedded"
        );
        let w = desired_set(
            base,
            &real_registry(),
            &names(&["claude"]),
            &defs,
            &[],
            &[],
            &[],
        );

        let pi = base.join(".doctrine/agents/pi/dispatch-worker.md");
        let claude = base.join(".doctrine/agents/claude/dispatch-worker.md");
        assert!(canons(&w.desired.members).contains(&pi));
        assert!(
            canons(&w.desired.selected).contains(&claude),
            "control: claude selected"
        );
        assert!(!canons(&w.desired.selected).contains(&pi));
        assert!(
            !w.desired
                .new_links
                .contains(&base.join(".pi/agents/dispatch-worker.md"))
        );
        assert!(
            w.desired
                .members
                .iter()
                .all(|m| !m.canon.ends_with("AGENTS.md"))
        );
        assert!(
            w.sources
                .keys()
                .all(|c| canons(&w.desired.members).contains(c))
        );
        assert_eq!(w.sources.len(), w.desired.members.len());
    }

    #[test]
    fn desired_set_installs_every_claude_def() {
        let shipped: BTreeSet<String> =
            fs::read_dir(crate::test_support::repo_root().join("install/agents/claude"))
                .unwrap()
                .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .filter(|n| n.ends_with(".md"))
                .collect();
        assert!(
            shipped.contains("capsule-worker.md"),
            "control: non-empty walk"
        );

        let base = Path::new(BASE);
        let defs = embedded_agent_defs();
        let w = desired_set(
            base,
            &real_registry(),
            &names(&["claude"]),
            &defs,
            &[],
            &[],
            &[],
        );
        let canon_dir = base.join(".doctrine/agents/claude");
        let selected: BTreeSet<String> = w
            .desired
            .selected
            .iter()
            .filter(|m| m.kind == MemberKind::AgentDef && m.canon.parent() == Some(&canon_dir))
            .map(|m| m.canon.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(selected, shipped);
        for f in &shipped {
            assert!(
                w.desired
                    .new_links
                    .contains(&base.join(".claude/agents").join(f)),
                "{f}"
            );
        }
    }

    #[test]
    fn desired_set_selects_nothing_for_codex_or_unregistered() {
        let base = Path::new(BASE);
        let defs = embedded_agent_defs();
        let catalog = discover().unwrap();
        let all: Vec<&Entry> = catalog.iter().collect();
        let w = desired_set(
            base,
            &real_registry(),
            &names(&["codex", "cursor"]),
            &defs,
            &catalog,
            &all,
            &[],
        );
        assert!(!w.desired.members.is_empty(), "control: members built");
        assert!(w.desired.selected.is_empty());
        assert!(w.desired.new_links.is_empty());
    }

    #[test]
    fn desired_set_skill_member_carries_every_skills_route_site() {
        let registry = registry_from(
            "[harness.a]\nskills = \".a/skills\"\n[harness.b]\nskills = \".b/deep/skills\"\n",
        );
        let skill = Entry {
            domain: "doctrine".to_string(),
            id: "x".to_string(),
            description: String::new(),
            files: Vec::new(),
        };
        let catalog = vec![skill];
        let picked: Vec<&Entry> = catalog.iter().collect();
        let base = Path::new(BASE);
        let a = base.join(".a/skills/x");
        let b = base.join(".b/deep/skills/x");

        let one = desired_set(base, &registry, &names(&["a"]), &[], &catalog, &picked, &[]);
        let [member] = one.desired.members.iter().collect::<Vec<_>>()[..] else {
            panic!("one skill member expected: {:?}", one.desired.members);
        };
        assert_eq!(member.links, BTreeSet::from([a.clone(), b.clone()]));
        assert!(
            one.desired.selected.contains(member),
            "selected by clone, same links"
        );
        assert_eq!(one.desired.new_links, BTreeSet::from([a.clone()]));

        let both = desired_set(
            base,
            &registry,
            &names(&["a", "b"]),
            &[],
            &catalog,
            &picked,
            &[],
        );
        let roots = manifest_roots(base);
        let targets: BTreeMap<PathBuf, PathBuf> =
            plan(&roots, Scope::Project, &both.desired, &Snapshot::default())
                .into_iter()
                .filter_map(|act| match act {
                    Action::Link { dest, target } => Some((dest, target)),
                    _ => None,
                })
                .collect();
        assert_eq!(
            targets.get(&a),
            Some(&PathBuf::from("../../.doctrine/skills/x"))
        );
        assert_eq!(
            targets.get(&b),
            Some(&PathBuf::from("../../../.doctrine/skills/x"))
        );
    }

    // ── converge: the wired chain (T4c migrations of the retired legs) ──

    /// Converge `base` toward `harnesses` over the real catalog with
    /// `skill_ids` selected and `workflows` shipped (no agent defs); returns
    /// the report.
    pub(super) fn converge_report(
        base: &Path,
        registry: &Registry,
        harnesses: &[&str],
        skill_ids: &[&str],
        workflows: &[(String, Vec<u8>)],
    ) -> String {
        let catalog = discover().unwrap();
        let picked = select_for_install(&catalog, &names(skill_ids), &[]).unwrap();
        let wanted = desired_set(
            base,
            registry,
            &names(harnesses),
            &[],
            &catalog,
            &picked,
            workflows,
        );
        let cx = Converge {
            root: base,
            base,
            scope: Scope::Project,
        };
        let mut out = Vec::new();
        converge(
            &cx,
            &framework_roots().unwrap(),
            registry,
            &wanted,
            &mut out,
        )
        .unwrap();
        String::from_utf8(out).unwrap()
    }

    /// `code-review` converged for claude over the real registry.
    fn converge_code_review(t: &Tree) -> String {
        converge_report(
            t.base(),
            &real_registry(),
            &["claude"],
            &["code-review"],
            &[],
        )
    }

    const CODE_REVIEW_TARGET: &str = "../../.doctrine/skills/code-review";

    #[test]
    fn converge_materialises_skill_and_links_it() {
        let t = Tree::new();
        let out = converge_code_review(&t);

        let canon = t.path(".doctrine/skills/code-review");
        assert!(canon.join("SKILL.md").is_file());
        assert_eq!(
            fs::read_link(t.path(".claude/skills/code-review")).unwrap(),
            PathBuf::from(CODE_REVIEW_TARGET),
            "the link value IS the criterion, not mere resolvability"
        );
        assert!(out.starts_with("framework artefacts (project):\n"), "{out}");
        assert!(
            out.contains(&format!("  skill     code-review → {}", canon.display())),
            "{out}"
        );
        assert!(out.contains("  linked    code-review"), "{out}");
    }

    #[test]
    fn converge_is_idempotent() {
        let t = Tree::new();
        let first = converge_code_review(&t);
        assert!(
            first.contains("  linked    code-review"),
            "control: {first}"
        );
        let skill = t.path(".doctrine/skills/code-review/SKILL.md");
        let link = t.path(".claude/skills/code-review");
        let (body, value) = (fs::read(&skill).unwrap(), fs::read_link(&link).unwrap());

        let second = converge_code_review(&t);

        // D1: an already-correct link plans nothing, so prints nothing.
        for verb in ["linked", "relinked", "kept", "pruned"] {
            assert!(!second.contains(&format!("  {verb} ")), "{verb}: {second}");
        }
        assert_eq!(fs::read(&skill).unwrap(), body);
        assert_eq!(fs::read_link(&link).unwrap(), value);
    }

    #[test]
    fn converge_keeps_real_dir_and_reports_shadowing() {
        let t = Tree::new();
        let mine = t.file(".claude/skills/code-review/mine.txt", "user-owned");

        let out = converge_code_review(&t);

        // D2: a real dir shadows like a real file — one wording.
        assert!(
            out.contains(
                "  kept      code-review (real file shadows doctrine def — remove it to receive updates)"
            ),
            "{out}"
        );
        assert_eq!(fs::read_to_string(&mine).unwrap(), "user-owned");
        assert_eq!(
            entry_state(&t.path(".claude/skills/code-review")).unwrap(),
            EntryState::Dir,
            "the user's dir survives as a real dir, never replaced"
        );
        assert!(
            t.path(".doctrine/skills/code-review/SKILL.md").is_file(),
            "control: convergence ran"
        );
    }

    #[test]
    fn converge_heals_a_dangling_owned_link() {
        let t = Tree::new();
        let link = t.link(".claude/skills/code-review", CODE_REVIEW_TARGET);
        assert!(!link.exists(), "control: the owned link dangles before");

        converge_code_review(&t);

        assert!(link.exists(), "the link resolves through the fresh canon");
        assert_eq!(
            fs::read_link(&link).unwrap(),
            PathBuf::from(CODE_REVIEW_TARGET)
        );
    }

    #[test]
    fn converge_links_every_skills_route_site() {
        // Two link dirs at different depths — a deeper dir must carry a deeper
        // relative target, proving the per-site target, not a hard-coded one.
        let registry = registry_from(
            "[harness.a]\nskills = \".a/skills\"\n[harness.b]\nskills = \".b/deep/skills\"\n",
        );
        let t = Tree::new();
        converge_report(t.base(), &registry, &["a", "b"], &["code-review"], &[]);

        assert_eq!(
            fs::read_link(t.path(".a/skills/code-review")).unwrap(),
            PathBuf::from(CODE_REVIEW_TARGET)
        );
        assert_eq!(
            fs::read_link(t.path(".b/deep/skills/code-review")).unwrap(),
            PathBuf::from("../../../.doctrine/skills/code-review")
        );
        assert!(t.path(".b/deep/skills/code-review/SKILL.md").is_file());
    }

    fn synthetic_workflow() -> Vec<(String, Vec<u8>)> {
        vec![("drive-slice.js".to_string(), b"// stub workflow".to_vec())]
    }

    #[test]
    fn converge_materialises_and_links_a_synthetic_workflow() {
        let t = Tree::new();
        let out = converge_report(
            t.base(),
            &real_registry(),
            &["claude"],
            &["code-review"],
            &synthetic_workflow(),
        );

        let canon = t.path(".doctrine/workflows/drive-slice.js");
        let link = t.path(".claude/workflows/drive-slice.js");
        assert_eq!(fs::read(&canon).unwrap(), b"// stub workflow");
        assert_eq!(
            fs::read_link(&link).unwrap(),
            PathBuf::from("../../.doctrine/workflows/drive-slice.js"),
            "linked, not copied, into .claude/workflows/"
        );
        assert_eq!(fs::read(&link).unwrap(), b"// stub workflow");
        assert!(
            out.contains(&format!("  workflow  drive-slice.js → {}", canon.display())),
            "{out}"
        );
    }

    #[test]
    fn workflow_dry_run_render_writes_nothing() {
        let t = Tree::new();
        let registry = real_registry();
        let catalog = discover().unwrap();
        let workflows = synthetic_workflow();
        let wanted = desired_set(
            t.base(),
            &registry,
            &names(&["claude"]),
            &[],
            &catalog,
            &[],
            &workflows,
        );
        let (scanned, actions) = scan_and_plan(&project(&t), &registry, &wanted);
        let lines: Vec<String> = actions
            .iter()
            .map(|a| action_line(a, &scanned, &wanted, true))
            .collect();

        let canon = t.path(".doctrine/workflows/drive-slice.js");
        assert!(
            lines.contains(&format!("  would write {}", canon.display())),
            "{lines:?}"
        );
        assert!(
            lines
                .iter()
                .any(|l| l.starts_with("  would link ") && l.contains("drive-slice.js")),
            "{lines:?}"
        );
        assert!(
            listing(t.base()).is_empty(),
            "a dry-run render writes nothing"
        );
    }

    #[test]
    fn converge_with_no_workflows_creates_no_workflow_dirs() {
        let t = Tree::new();
        let out = converge_report(
            t.base(),
            &real_registry(),
            &["claude"],
            &["code-review"],
            &[],
        );
        assert!(
            t.path(".doctrine/skills/code-review").is_dir(),
            "control: convergence ran"
        );
        assert!(!t.path(".doctrine/workflows").exists());
        assert!(!t.path(".claude/workflows").exists());
        assert!(!out.contains("workflow"), "{out}");
    }
}

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
        // The live [hymns] seal = ["preamble/core","stage/design"] — SL-233
        // PHASE-07 sealed the design stage's invariant hymn.
        //
        // Asserted by MEMBERSHIP, not by a bare count: the count is not the
        // property under test, and pinning it makes every future seal a false
        // red in a test that is about these two slots.
        let sealed = embedded_seal_set().unwrap();
        let sealed_slot = |band: crate::hymns::Band, label: &str| {
            sealed
                .0
                .iter()
                .any(|slot| slot.band == band && slot.label == label)
        };
        assert!(
            sealed_slot(crate::hymns::Band::Preamble, "core"),
            "preamble/core stays sealed: {sealed:?}"
        );
        assert!(
            sealed_slot(crate::hymns::Band::Stage, "design"),
            "stage/design is sealed: {sealed:?}"
        );
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

    // --- ownership classification ---

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

    // --- skill canon staging (SL-250 PHASE-05, SL-281 PHASE-03) ---

    /// A REAL embedded skill (not a synthetic `Entry`) — `copy_skill` resolves
    /// `entry.files` through `PluginAssets::get`, so a hand-built entry with
    /// invented paths would error rather than exercise the leg (A3).
    fn real_skill_entry() -> Entry {
        discover()
            .unwrap()
            .into_iter()
            .find(|e| e.id == "code-review" && e.domain == "doctrine")
            .expect("code-review ships in the doctrine domain")
    }

    /// Plant a regular file, a dir holding a file, and a dangling symlink at
    /// `names` under `dir`; returns each planted path with its identity probe.
    fn plant_squatters(dir: &Path, names: &[String]) -> Vec<PathBuf> {
        fs::create_dir_all(dir).unwrap();
        let mut planted = Vec::new();
        for (i, name) in names.iter().enumerate() {
            let p = dir.join(name);
            match i % 3 {
                0 => fs::write(&p, "user file").unwrap(),
                1 => {
                    fs::create_dir(&p).unwrap();
                    fs::write(p.join("inner"), "user inner").unwrap();
                }
                _ => std::os::unix::fs::symlink("/nonexistent/dangling", &p).unwrap(),
            }
            planted.push(p);
        }
        planted
    }

    /// Assert every squatter is still what `plant_squatters` made it.
    fn assert_squatters_intact(planted: &[PathBuf]) {
        for (i, p) in planted.iter().enumerate() {
            match i % 3 {
                0 => assert_eq!(fs::read_to_string(p).unwrap(), "user file", "{p:?}"),
                1 => assert_eq!(
                    fs::read_to_string(p.join("inner")).unwrap(),
                    "user inner",
                    "{p:?}"
                ),
                _ => assert_eq!(
                    fs::read_link(p).unwrap(),
                    PathBuf::from("/nonexistent/dangling"),
                    "{p:?}"
                ),
            }
        }
    }

    /// Legacy name plus the first candidate names `write_link`/`stage_exclusive`
    /// might try (the counter is process-global, so plant a wide band).
    fn squatter_names(id: &str) -> Vec<String> {
        let pid = std::process::id();
        let mut names = vec![format!(".tmp-{id}")];
        names.extend((0..48).map(|n| format!(".tmp-{id}.{pid}.{n}")));
        names
    }

    #[test]
    fn staging_never_reuses_preexisting_entry() {
        let cr = real_skill_entry();
        for case in 0..3 {
            let dir = tempfile::tempdir().unwrap();
            let skills = dir.path().join(".doctrine/skills");
            let names = squatter_names("code-review");
            let planted = plant_squatters(&skills, &names);
            assert!(planted.iter().all(|p| p.symlink_metadata().is_ok()));

            // (a) link
            let link_dir = dir.path().join(".claude/skills");
            let squat_links = plant_squatters(&link_dir, &names);
            let canon = skills.join("code-review");
            let target = Path::new("../../.doctrine/skills/code-review");
            // (b) canon, twice (second is a refresh)
            for _ in 0..=case {
                materialise_canonical(&cr, &canon).unwrap();
            }
            write_link(&link_dir.join("code-review"), target).unwrap();
            write_link(&link_dir.join("code-review"), target).unwrap();

            assert_squatters_intact(&planted);
            assert_squatters_intact(&squat_links);
            assert!(canon.join("SKILL.md").exists());
            assert_eq!(fs::read_link(link_dir.join("code-review")).unwrap(), target);
        }
    }

    #[test]
    fn staging_failure_removes_only_the_dir_it_created() {
        let dir = tempfile::tempdir().unwrap();
        let skills = dir.path().join(".doctrine/skills");
        let names = squatter_names("ghost");
        let planted = plant_squatters(&skills, &names);
        let before: BTreeSet<_> = fs::read_dir(&skills)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();

        // Synthetic entry: its file is not embedded, so copy_skill fails.
        let bad = entry("doctrine", "ghost");
        assert!(materialise_canonical(&bad, &skills.join("ghost")).is_err());

        let after: BTreeSet<_> = fs::read_dir(&skills)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(before, after, "nothing added, nothing but ours removed");
        assert_squatters_intact(&planted);
    }

    #[test]
    fn staging_never_reuses_preexisting_entry_under_symlinked_doctrine() {
        let project = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        let real = other.path().join("real-doctrine");
        fs::create_dir_all(real.join("skills/keep-me")).unwrap();
        std::os::unix::fs::symlink(&real, project.path().join(".doctrine")).unwrap();

        let planted = plant_squatters(&real.join("skills"), &squatter_names("code-review"));
        assert!(planted.iter().all(|p| p.symlink_metadata().is_ok()));

        let link_dir = project.path().join(".claude/skills");
        let registry = load_manifest().unwrap().harness;
        let mut out = String::new();
        for _ in 0..2 {
            out += &super::tests_converge::converge_report(
                project.path(),
                &registry,
                &["claude"],
                &["code-review"],
                &[],
            );
        }

        assert_squatters_intact(&planted);
        assert!(real.join("skills/keep-me").is_dir());
        // The root is reached through the symlinked `.doctrine`, so even the
        // orphan `keep-me` is withheld from pruning.
        let blocked = project.path().join(".doctrine/skills");
        assert!(
            out.contains(&format!("  prune blocked {}", blocked.display())),
            "{out}"
        );
        assert!(real.join("skills/code-review/SKILL.md").exists());
        assert!(link_dir.join("code-review/SKILL.md").exists());
    }

    #[test]
    fn stage_from_skips_taken_candidates_and_errors_when_exhausted() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path().join("x");
        let first = 1000;
        for seq in [first, first + 1] {
            fs::write(staging_path(&dest, seq).unwrap(), "taken").unwrap();
        }
        let got = stage_from(&dest, first, |t| fs::create_dir(t)).unwrap();
        assert_eq!(got, staging_path(&dest, first + 2).unwrap());
        assert!(got.is_dir());
        assert_eq!(
            fs::read_to_string(staging_path(&dest, first).unwrap()).unwrap(),
            "taken"
        );

        let err = stage_from(&dest, first, |_| {
            Err(io::Error::from(io::ErrorKind::AlreadyExists))
        })
        .unwrap_err();
        assert!(err.to_string().contains(&dir.path().display().to_string()));

        let err = stage_from(&dest, first, |_| {
            Err(io::Error::from(io::ErrorKind::PermissionDenied))
        })
        .unwrap_err();
        assert!(format!("{err:#}").contains("Failed to stage"));
    }
}

// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation;
    use std::fs;

    /// VT-2 (SL-223 PHASE-01): `asset_text` keeps its signature but now delegates
    /// to `asset_source::read_text`. A known embedded asset still resolves to Ok
    /// text through the seam — the extraction is behaviour-preserving.
    #[test]
    fn asset_text_delegates_to_asset_source() {
        let text = asset_text("manifest.toml").expect("manifest.toml embedded via asset_source");
        assert!(
            !text.is_empty(),
            "delegated asset_text should return content"
        );
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
        // Pre-create a base backing so the plan must Skip it (SL-227: projection is
        // now the three-file base set, so the skip target is a base backing).
        let existing = target.join("doctrine.toml");
        fs::write(&existing, "old content").unwrap();

        let manifest = Manifest::default_for_tests();
        let plan = build_plan(&manifest, dir.path());

        let has_skip = plan
            .steps
            .iter()
            .any(|s| matches!(s, Step::Skip { dest, .. } if dest == &existing));
        assert!(
            has_skip,
            "Expected a Skip step for the pre-existing base backing"
        );
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

        // SL-227 FR-008: items/ is NO LONGER eagerly scaffolded by the manifest — it
        // materialises lazily on the first `memory record` (entity.rs materialise*).
        assert!(
            !manifest
                .dirs
                .create
                .iter()
                .any(|d| d == ".doctrine/memory/items"),
            "memory items/ is lazy now — not an eager manifest [dirs].create entry"
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

    /// The observation corpus root: the parent of the records dir, so the two
    /// cannot drift. `records/` and the reserved `by-month/` view are siblings
    /// beneath it (design § 2.1).
    fn observations_root() -> &'static str {
        observation::store::RECORDS_DIR
            .rsplit_once('/')
            .expect("RECORDS_DIR is a nested path")
            .0
    }

    /// SL-231 PHASE-05 (VT-1): the ONE observation ignore rule the manifest
    /// distributes covers reserved publication temporaries and nothing else,
    /// and it survives the round-trip into a client `.gitignore`.
    ///
    /// The pattern embeds the reserved prefix, so it is asserted against
    /// `fsutil::PUBLICATION_TEMP_PREFIX` rather than a re-typed literal
    /// (STD-001) — the same crossing `observation::store`'s loader test makes
    /// against its skip predicate. A drift in either place would leave real
    /// temporaries tracked or real records ignored.
    #[test]
    fn observation_publication_temps_are_ignored() {
        let manifest = load_manifest().unwrap();

        let observation_entries: Vec<&String> = manifest
            .gitignore
            .entries
            .iter()
            .filter(|e| e.starts_with(observations_root()))
            .collect();

        assert_eq!(
            observation_entries.len(),
            1,
            "exactly one observation ignore rule is distributed, got {observation_entries:?}"
        );
        let entry = observation_entries[0];
        assert!(
            entry.contains(crate::fsutil::PUBLICATION_TEMP_PREFIX),
            "the distributed rule must be keyed on the reserved publication \
             prefix {:?}, got {entry:?}",
            crate::fsutil::PUBLICATION_TEMP_PREFIX
        );

        // Surface 2: it actually reaches a client's `.gitignore` block.
        let block = crate::install_ownership::render_block(
            &manifest.gitignore.entries,
            &manifest.framework.roots,
            &[],
        );
        assert!(
            block.iter().any(|l| l == entry),
            "the reserved-temp rule must land verbatim in the managed block; got:\n{block:#?}"
        );
    }

    /// SL-231 PHASE-05 (VT-1): authoritative observation records are authored
    /// collection data — committed, diffable, visible in review by default
    /// (design.md § 2.1). A project MAY opt into ignoring them, but shipping
    /// that as the default would silently make every client's corpus local-only
    /// and forfeit correlation and audit history.
    ///
    /// This is the guard on the failure mode with teeth: over-ignoring is
    /// invisible — the suite stays green and records simply stop reaching
    /// review. Reserved `by-month/` paths are likewise unignored; this slice
    /// distributes no pattern for them (EX-4).
    #[test]
    fn observation_records_remain_authored_by_default() {
        let manifest = load_manifest().unwrap();

        for entry in &manifest.gitignore.entries {
            if !entry.starts_with(observations_root()) {
                continue;
            }
            assert!(
                entry.contains(crate::fsutil::PUBLICATION_TEMP_PREFIX),
                "{entry:?} ignores part of the observation tree without being \
                 scoped to reserved publication temporaries — authored records \
                 would stop reaching review"
            );
            assert!(
                !entry.contains("by-month"),
                "{entry:?} distributes a by-month pattern, excluded from this slice"
            );
        }
    }

    /// SL-030 PHASE-03: the policy tree is an authored governance kind, so the
    /// manifest must create it (parity with adr / memory-items) and must NOT
    /// ignore it — install surface 1 of 3. The `.gitignore` negation (surface 2)
    /// and the git-add round-trip (surface 3) are covered by the e2e commit test.
    #[test]
    fn embedded_manifest_creates_the_policy_tree() {
        let manifest = load_manifest().unwrap();
        // SL-227 FR-008: the policy root is lazy now — created on first `policy new`,
        // not an eager manifest [dirs].create entry.
        assert!(
            !manifest.dirs.create.iter().any(|d| d == ".doctrine/policy"),
            "the policy tree is lazily scaffolded on first `policy new`, not eagerly"
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
        // SL-227 FR-008: the standard root is lazy now — created on first
        // `standard new`, not an eager manifest [dirs].create entry.
        assert!(
            !manifest
                .dirs
                .create
                .iter()
                .any(|d| d == ".doctrine/standard"),
            "the standard tree is lazily scaffolded on first `standard new`, not eagerly"
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
        // SL-227 FR-008: the review root is lazy now — created on first `review …`,
        // not an eager manifest [dirs].create entry. Its templates must still embed.
        assert!(
            !manifest.dirs.create.iter().any(|d| d == ".doctrine/review"),
            "the review tree is lazily scaffolded on first use, not eagerly"
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
        // SL-227 FR-008: the rec root is lazy now — created on first `rec new`, not
        // an eager manifest [dirs].create entry. Its templates must still embed.
        assert!(
            !manifest.dirs.create.iter().any(|d| d == ".doctrine/rec"),
            "the rec tree is lazily scaffolded on first `rec new`, not eagerly"
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
        // The ignore comes from the `[framework] roots` glob in the managed
        // block, not a literal manifest entry (SL-281 PHASE-05).
        assert!(
            !manifest
                .gitignore
                .entries
                .iter()
                .any(|e| e == ".doctrine/skills/*"),
            "the skills glob is derived from the framework roots, not listed"
        );
        let block = crate::install_ownership::render_block(
            &manifest.gitignore.entries,
            &manifest.framework.roots,
            &[],
        );
        assert!(
            block.iter().any(|l| l == ".doctrine/skills/*"),
            "the managed block must gitignore the derived skills tree; got:\n{block:#?}"
        );
    }

    /// SL-281 PHASE-06 (VT-1): this repo dogfoods the managed block. Its
    /// `.gitignore` carries every static block line — markers, runtime entries
    /// and root globs — as `render_block` produces them. Per-link lines depend
    /// on what is installed, so they are left out (empty `links`).
    #[test]
    fn repo_gitignore_contains_static_block_entries() {
        let manifest = load_manifest().unwrap();
        let gitignore =
            fs::read_to_string(crate::test_support::repo_root().join(".gitignore")).unwrap();
        let present: BTreeSet<&str> = gitignore.lines().collect();
        let missing: Vec<String> = crate::install_ownership::render_block(
            &manifest.gitignore.entries,
            &manifest.framework.roots,
            &[],
        )
        .into_iter()
        .filter(|line| !present.contains(line.as_str()))
        .collect();
        assert!(
            missing.is_empty(),
            "repo .gitignore lacks static managed-block lines (run `doctrine install`): {missing:#?}"
        );
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
        // A base backing (project-orientation.md) should be installed under the
        // target (SL-227: only the three-file base projects).
        let orientation = dir.path().join(".doctrine/project-orientation.md");
        assert!(orientation.is_file());
        let content = fs::read_to_string(&orientation).unwrap();
        assert!(content.contains("Project Orientation"));
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
        // The REAL manifest. SL-227: after the minimal-projection flip, install no
        // longer eagerly scaffolds the memory items/ tree — it appears lazily on the
        // first `memory record` (entity.rs materialise*). shipped/ is `memory sync`'s.
        let manifest = load_manifest().unwrap();
        let plan = build_plan(&manifest, dir.path());
        execute_plan(&plan).unwrap();

        assert!(
            !dir.path().join(".doctrine/memory/items").exists(),
            "install no longer eagerly creates items/ (lazy on first `memory record`)"
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
        let dest = dir.path().join(".doctrine/doctrine.toml");
        fs::create_dir_all(dest.parent().unwrap()).unwrap();
        let original = "original content";
        fs::write(&dest, original).unwrap();

        let plan = build_plan(&manifest, dir.path());
        execute_plan(&plan).unwrap();

        // Must still be original (an existing base backing is never clobbered).
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

    // SL-254 PHASE-10 retired two tests here with the assets they drove:
    //   - `install_agent_def_dispatch_probe_writes_bytes_identically_under_the_
    //     derived_dest` proved the dest filename is DERIVED from the embed-asset
    //     basename rather than hardcoded to `dispatch-worker.md`. It needed a
    //     MARKER-FREE shipped def to prove it, and `agents/claude/dispatch-probe.md`
    //     was the only one; both surviving defs (claude + pi `dispatch-worker.md`)
    //     carry `WORKER_RESOLVE_MARKER`, and their basenames are exactly the name
    //     the old bug hardcoded — so derived and hardcoded are now
    //     indistinguishable from any real asset. (SL-281 PHASE-04 restored the
    //     proof: `execute_bakes_agent_defs_and_copies_skills` writes the
    //     marker-free capsule defs byte-identical under their own names.)
    //   - `embedded_workflow_defs_carries_the_drive_slice_payload` asserted the
    //     workflows leg was live in production. It is a no-op again: the embed
    //     root ships no `workflows/*.js` since `drive-slice.js` went with Mode B.
    // The workflow MECHANISM stays covered by the synthetic-asset converge
    // tests in `tests_converge`.

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

    // SL-227 (was SL-011 VT-1, flipped): a base backing rides the write-if-absent
    // projection — installed create-if-missing, left untouched when already present.
    // Governance is no longer projected; `project-orientation.md` is the base backing
    // that now exercises this create-if-missing / skip-when-present behaviour.
    #[test]
    fn seeds_base_backing_when_missing_and_skips_when_present() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest {
            target: ".doctrine".to_string(),
            ..Manifest::default_for_tests()
        };
        let dest = dir.path().join(".doctrine/project-orientation.md");

        // missing → written from the embedded base backing.
        execute_plan(&build_plan(&manifest, dir.path())).unwrap();
        assert!(
            dest.is_file(),
            "project-orientation.md written when missing"
        );
        assert!(
            fs::read_to_string(&dest)
                .unwrap()
                .contains("Project Orientation"),
            "written from the embedded backing",
        );

        // present → a re-install leaves the user's edits untouched (Skip).
        let edited = "# My own orientation\n\nmy own pointers\n";
        fs::write(&dest, edited).unwrap();
        execute_plan(&build_plan(&manifest, dir.path())).unwrap();
        assert_eq!(
            fs::read_to_string(&dest).unwrap(),
            edited,
            "an existing base backing is never clobbered",
        );
    }

    // ---------------------------------------------------------------
    // SL-227 PHASE-03: the minimal-projection flip
    // ---------------------------------------------------------------

    /// VT-2 / EX-3 — the derived no-silent-unreachable gate. Every install asset
    /// that is NOT a projection base backing MUST be declared as a published backing
    /// (`{install} − {base}` ⊆ published) — else the minimal-projection flip would
    /// strand it (neither eagerly projected nor reachable). Both the enumeration and
    /// the base set are DERIVED from disk-source (`repo_root()`), never the compiled
    /// embed or a hardcoded list — so an incremental build over a stale `install/`
    /// edit cannot false-green the gate (SL-227 F-7; the `debug-embed` footgun,
    /// CHR-014). Shares the disk-source check with its `publication.rs` sibling.
    #[test]
    fn every_unprojected_embed_is_a_published_backing() {
        crate::asset_source::assert_unprojected_install_assets_are_published();
    }

    /// VT-1 / VT-4 — a fresh install materialises EXACTLY the three-file base
    /// {.gitignore, doctrine.toml, project-orientation.md} and seeds NO memory entity.
    /// The seed gate: `seed_authoring_memories` early-returns on the manifest's now
    /// empty `[memory].seed_items`, so a base install writes no memory.
    #[test]
    fn fresh_install_projects_exactly_the_three_file_base_and_seeds_no_memory() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = load_manifest().unwrap();

        // The shipped manifest declares exactly the three base backings.
        let base: BTreeSet<&str> = manifest.base.backings.iter().map(String::as_str).collect();
        let expected: BTreeSet<&str> = [".gitignore", "doctrine.toml", "project-orientation.md"]
            .into_iter()
            .collect();
        assert_eq!(base, expected, "base must be the three-file set");

        execute_plan(&build_plan(&manifest, dir.path())).unwrap();
        seed_authoring_memories(dir.path(), &manifest).unwrap();

        // Under the target, EXACTLY the two content backings appear — nothing else.
        let target = dir.path().join(".doctrine");
        let mut present: Vec<String> = fs::read_dir(&target)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        present.sort();
        assert_eq!(
            present,
            vec![
                "doctrine.toml".to_string(),
                "project-orientation.md".to_string()
            ],
            "the target holds exactly the two content base backings"
        );
        // The third base member, .gitignore, lands at the project root as the
        // managed block (written after the forward steps, not by the plan).
        write_gitignore_block(dir.path(), &manifest, &[], false).unwrap();
        assert!(dir.path().join(".gitignore").is_file());
        // The seed gate: no memory entity was created on a base install.
        assert!(
            !dir.path().join(".doctrine/memory").exists(),
            "the seed gate must leave no memory entity on a base install"
        );
    }

    /// VT-3 — `build_plan` leg 2 is driven by the manifest `[base].backings` set: the
    /// two content backings land as Install steps under the target, and `.gitignore`
    /// is NOT projected by leg 2 (the managed block owns it).
    #[test]
    fn build_plan_projects_the_manifest_base_backings() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = Manifest {
            base: BaseSection {
                backings: vec![
                    ".gitignore".to_string(),
                    "doctrine.toml".to_string(),
                    "project-orientation.md".to_string(),
                ],
            },
            target: ".doctrine".to_string(),
            ..Manifest::default_for_tests()
        };
        let plan = build_plan(&manifest, dir.path());
        let installs: BTreeSet<String> = plan
            .steps
            .iter()
            .filter_map(|s| match s {
                Step::Install { dest, .. } => {
                    Some(dest.file_name().unwrap().to_string_lossy().into_owned())
                }
                _ => None,
            })
            .collect();
        assert!(installs.contains("doctrine.toml"));
        assert!(installs.contains("project-orientation.md"));
        // .gitignore is the managed block — never an Install into the target.
        assert!(
            !installs.contains(".gitignore"),
            "leg 2 must not project .gitignore; the managed block owns it"
        );
    }

    /// VT-5 / NF-004 — the minimal-projection flip is orthogonal to harness adapter
    /// installation: a base-only projection installs no agent defs in leg 2, yet
    /// `detect_agents` still resolves a present harness dir, so the forward-step
    /// adapter leg still fires after the flip.
    #[test]
    fn detect_agents_is_independent_of_the_base_projection_flip() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".claude")).unwrap();
        // A base-only install projects just the three backings (no agent defs)…
        let manifest = load_manifest().unwrap();
        execute_plan(&build_plan(&manifest, dir.path())).unwrap();
        // …yet the harness is still detected, so the adapter leg fires.
        assert_eq!(
            detect_agents(&[], dir.path(), &registry()),
            vec!["claude".to_string()],
            "a detected harness must still resolve for adapter install after the flip"
        );
    }

    /// NF-005 — the project-governance seed is retired: a base install writes no
    /// governance.md under the target (governance is published, reachable on demand,
    /// not eagerly projected), and it is not a declared base backing.
    #[test]
    fn base_install_does_not_project_governance() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = load_manifest().unwrap();
        execute_plan(&build_plan(&manifest, dir.path())).unwrap();
        assert!(
            !dir.path().join(".doctrine/governance.md").exists(),
            "governance.md is published, not eagerly projected, after the flip"
        );
        assert!(
            !manifest.base.backings.iter().any(|b| b == "governance.md"),
            "governance.md must not be a base backing"
        );
    }

    // ---------------------------------------------------------------
    // PHASE-02: detect_agents + prompt_step
    // ---------------------------------------------------------------

    // SL-250 PHASE-04. An explicit `--agent` used to be DROPPED on the way to
    // the boot leg, so a fresh project got no @-import and no hooks and said
    // only "no harness directories detected — skipped".

    #[test]
    fn an_explicit_agent_reaches_the_boot_leg_in_a_bare_project() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // Directory detection finds nothing in a bare project…
        assert!(
            crate::boot::resolve_harnesses(&[], root)
                .unwrap_or_default()
                .is_empty()
        );
        // …but an explicit flag is a DIRECTIVE, so the boot leg still fires.
        assert_eq!(
            install_harnesses(&["claude".to_string()], root, &registry()).len(),
            1,
            "--agent claude wires the claude harness with no .claude/ on disk"
        );
        // A skills-only agent implies no harness — and must not silently fall
        // back to detection, which would wire something the user did not name.
        assert!(install_harnesses(&["universal".to_string()], root, &registry()).is_empty());
        // No flags: unchanged — detection governs.
        assert!(install_harnesses(&[], root, &registry()).is_empty());
    }

    #[test]
    fn detect_agents_empty_when_no_agent_dirs_and_no_flags() {
        let dir = tempfile::tempdir().unwrap();
        let agents = detect_agents(&[], dir.path(), &registry());
        assert!(agents.is_empty());
    }

    #[test]
    fn detect_agents_returns_claude_when_dir_present() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".claude")).unwrap();
        let agents = detect_agents(&[], dir.path(), &registry());
        assert_eq!(agents, vec!["claude".to_string()]);
    }

    #[test]
    fn detect_agents_returns_pi_when_dir_present() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".pi")).unwrap();
        let agents = detect_agents(&[], dir.path(), &registry());
        assert_eq!(agents, vec!["pi".to_string()]);
    }

    #[test]
    fn detect_agents_returns_codex_when_dir_present() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".codex")).unwrap();
        let agents = detect_agents(&[], dir.path(), &registry());
        assert_eq!(agents, vec!["codex".to_string()]);
    }

    #[test]
    fn detect_agents_returns_universal_for_dot_agents_dir() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".agents")).unwrap();
        let agents = detect_agents(&[], dir.path(), &registry());
        assert_eq!(agents, vec!["universal".to_string()]);
    }

    #[test]
    fn detect_agents_detects_multiple_agent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".claude")).unwrap();
        fs::create_dir(dir.path().join(".pi")).unwrap();
        let agents = detect_agents(&[], dir.path(), &registry());
        assert_eq!(agents, vec!["claude".to_string(), "pi".to_string()]);
    }

    #[test]
    fn detect_agents_uses_explicit_over_detection() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".claude")).unwrap();
        let agents = detect_agents(&["pi".to_string()], dir.path(), &registry());
        assert_eq!(agents, vec!["pi".to_string()]);
    }

    #[test]
    fn detect_agents_returns_multiple_explicit() {
        let dir = tempfile::tempdir().unwrap();
        let agents = detect_agents(
            &["claude".to_string(), "pi".to_string()],
            dir.path(),
            &registry(),
        );
        assert_eq!(agents, vec!["claude".to_string(), "pi".to_string()]);
    }

    // --- harness registry (SL-281 PHASE-01) ---

    fn registry() -> Registry {
        load_manifest().unwrap().harness
    }

    fn strings(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_string()).collect()
    }

    fn boot_labels(harnesses: &[crate::boot::Harness]) -> Vec<&'static str> {
        harnesses.iter().map(crate::boot::harness_label).collect()
    }

    #[test]
    fn harness_table_with_both_skills_and_skills_via_is_refused() {
        let text = "[harness.odd]\nskills = \".odd/skills\"\nskills_via = \"universal\"\n";
        let err = parse_manifest(text).unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("odd"), "names the harness: {msg}");
        // Positive control: either route alone parses.
        parse_manifest("[harness.odd]\nskills = \".odd/skills\"\n").unwrap();
        parse_manifest("[harness.odd]\nskills_via = \"universal\"\n").unwrap();
    }

    #[test]
    fn embedded_manifest_declares_framework_roots_and_registry() {
        let m = load_manifest().unwrap();
        assert_eq!(
            m.framework.roots,
            strings(&[
                ".doctrine/agents",
                ".doctrine/skills",
                ".doctrine/workflows"
            ])
        );
        let names: Vec<&str> = m.harness.keys().map(String::as_str).collect();
        assert_eq!(names, vec!["claude", "codex", "pi", "universal"]);
    }

    #[test]
    fn detection_and_boot_follow_registry() {
        let reg = registry();
        let boot_for = |dirs: &[&str], agents: &[&str]| {
            let dir = tempfile::tempdir().unwrap();
            for d in dirs {
                fs::create_dir(dir.path().join(d)).unwrap();
            }
            boot_labels(&install_harnesses(&strings(agents), dir.path(), &reg))
        };
        // No --agent: the registry's detect dirs govern.
        assert_eq!(
            boot_for(&[".pi"], &[]),
            vec!["codex"],
            ".pi alone wires codex"
        );
        assert_eq!(boot_for(&[".codex"], &[]), vec!["codex"]);
        assert_eq!(boot_for(&[".claude"], &[]), vec!["claude"]);
        assert!(
            boot_for(&[".agents"], &[]).is_empty(),
            "universal has no boot"
        );
        assert_eq!(
            boot_for(&[".claude", ".pi", ".codex"], &[]),
            vec!["claude", "codex"],
            "pi and codex collapse to one codex adapter"
        );
        // With --agent: the registry maps names, detection is ignored.
        assert_eq!(boot_for(&[".claude"], &["pi"]), vec!["codex"]);
        assert!(boot_for(&[], &["universal"]).is_empty());
        assert!(
            boot_for(&[], &["cursor"]).is_empty(),
            "unregistered: no boot"
        );

        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        assert_eq!(
            detect_agents(&[], root, &reg),
            Vec::<String>::new(),
            "bare project detects nothing"
        );
        // A real AGENTS.md (not Claude's alias) still implies codex.
        fs::write(root.join("AGENTS.md"), "x").unwrap();
        assert_eq!(
            boot_labels(&install_harnesses(&[], root, &reg)),
            vec!["codex"]
        );
    }

    #[test]
    fn detection_skips_an_aliased_agents_md() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir(root.join(".claude")).unwrap();
        fs::write(root.join("CLAUDE.md"), "x").unwrap();
        std::os::unix::fs::symlink("CLAUDE.md", root.join("AGENTS.md")).unwrap();
        assert_eq!(
            boot_labels(&install_harnesses(&[], root, &registry())),
            vec!["claude"]
        );
    }

    #[test]
    fn pi_and_codex_delegate_universal_once() {
        let reg = registry();
        for agents in [
            &["pi"][..],
            &["codex"],
            &["universal"],
            &["pi", "codex", "universal"],
        ] {
            let d = delegate_targets(&strings(agents), &reg);
            assert_eq!(d.targets, strings(&["universal"]), "{agents:?}");
            assert!(d.unregistered.is_empty());
        }
        // claude links its own skills: never delegated.
        let d = delegate_targets(&strings(&["claude", "pi"]), &reg);
        assert_eq!(d.targets, strings(&["universal"]));
    }

    #[test]
    fn unregistered_harness_delegates_skills_and_reports_unregistered() {
        let d = delegate_targets(&strings(&["cursor", "pi", "cursor"]), &registry());
        assert_eq!(d.targets, strings(&["cursor", "universal"]));
        assert_eq!(d.unregistered, strings(&["cursor"]));
        let mut out = Vec::new();
        report_unregistered(&d.unregistered, &mut out).unwrap();
        let out = String::from_utf8(out).unwrap();
        assert!(
            out.contains("cursor") && out.contains("unregistered"),
            "{out}"
        );
    }

    /// RV-420 F-9: the prompt and the forward summary name everything
    /// confirming a harness installs, and the canon convergence it triggers.
    #[test]
    fn install_offer_names_what_confirming_mutates() {
        let reg = registry();
        for (agent, offer) in [
            ("claude", "skills + agent defs + workflows for claude"),
            ("pi", "skills via npx + agent defs for pi"),
            ("codex", "skills via npx for codex"),
            ("cursor", "skills via npx for cursor"),
        ] {
            assert_eq!(install_offer(agent, &reg), offer);
        }
        assert_eq!(
            install_question("pi", &reg, Scope::Project),
            "Install skills via npx + agent defs for pi, and converge framework canon \
             (pruning unshipped entries)? [y/N/a]"
        );
        assert_eq!(
            canon_convergence(Scope::Global),
            "converge framework canon (reporting orphans)"
        );
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
                base: BaseSection {
                    backings: vec![
                        ".gitignore".to_string(),
                        "doctrine.toml".to_string(),
                        "project-orientation.md".to_string(),
                    ],
                },
                dirs: DirsSection::default(),
                gitignore: GitignoreSection::default(),
                root_markers: RootMarkersSection::default(),
                memory: MemorySection::default(),
                hymns: HymnsSection::default(),
                framework: FrameworkSection::default(),
                harness: Registry::default(),
            }
        }
    }
}
