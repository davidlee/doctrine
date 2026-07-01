// SPDX-License-Identifier: GPL-3.0-only
#![allow(
    clippy::same_name_method,
    reason = "rust-embed derive generates conflicting method names"
)]
// run_install() and its call chain are pub(crate) but not currently called
// from main.rs (PHASE-01 consolidated the CLI surface). They are preserved
// for the standalone skills install path and are reachable via the extracted
// install_for_claude/install_for_other functions (SL-088 PHASE-02).
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "run_install call chain — preserved for standalone and ref paths"
    )
)]

//! `doctrine skills` — list and install agent skills.
//!
//! Skills are embedded from `plugins/<domain>/skills/<skill>/`. Claude is
//! installed **directly** (file copy); every other agent is **delegated** to
//! `npx skills`. The planner is pure; IO lives in the thin `run_*` shell and
//! behind the `Runner` seam.

use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use rust_embed::RustEmbed;
use serde::Deserialize;

/// Embedded skill plugins — everything under `plugins/`.
#[derive(RustEmbed)]
#[folder = "plugins/"]
struct PluginAssets;

/// The subset domain whose enumerated skills `--only-memory` resolves to — a
/// marketplace-only domain whose skills are symlinks into the canonical
/// `doctrine` domain, so it is excluded from the install catalog.
const MEMORY_SUBSET_DOMAIN: &str = "doctrine-memory";

/// The partner subset domain (`pair` + `walkthrough`), symlinked into the
/// canonical `doctrine` domain. Marketplace-only; no `--only-partner` analog.
const PARTNER_SUBSET_DOMAIN: &str = "doctrine-partner";

/// Marketplace-only domains the CLI does not install: their skills are symlinks
/// to a canonical domain (e.g. `doctrine-memory` → `doctrine`), so the embed
/// carries duplicates that would collide on skill id. Excluded at discovery.
const MARKETPLACE_ONLY_DOMAINS: &[&str] = &[MEMORY_SUBSET_DOMAIN, PARTNER_SUBSET_DOMAIN];

/// Runner program names — single-source per STD-001.
const RUNNER_BUNX: &str = "bunx";
const RUNNER_NPX: &str = "npx";

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

/// An install target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Agent {
    Claude,
    Other(String),
}

/// A canonical skill to (re)materialise — `dest` is `.doctrine/skills/<id>`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Canonical {
    id: String,
    dest: PathBuf,
}

/// Per-agent plan: Claude materialises a canonical tree and reconciles relative
/// symlinks into it (`Link` trichotomy); others delegate to `npx skills`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum AgentPlan {
    Claude {
        canonical: Vec<Canonical>,
        links: Vec<Link>,
    },
    Delegate {
        agent: String,
        argv: Vec<String>,
    },
}

/// A full install plan across the selected agents.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Plan {
    root: PathBuf,
    items: Vec<AgentPlan>,
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

/// Skill ids a marketplace subset domain enumerates, read from embedded paths.
/// `<domain>/skills/<id>/…` → {id}. Pure: the caller supplies the path iterator,
/// so it is unit-testable without the embed or disk.
fn subset_ids<'a>(paths: impl Iterator<Item = &'a str>, domain: &str) -> BTreeSet<String> {
    paths
        .filter_map(|p| match p.split('/').collect::<Vec<_>>().as_slice() {
            [d, "skills", id, ..] if *d == domain => Some((*id).to_string()),
            _ => None,
        })
        .collect()
}

/// Effective skill-id selection for `skills install`. When `only_memory`, derive
/// the subset from `paths` and bail loud if empty — the `select([]) == all` guard
/// (D3): an empty id set would otherwise install the entire catalog. Otherwise
/// pass `skills` through unchanged. clap guarantees `only_memory` is exclusive
/// with explicit `--skill`/`--domain`, so no exclusion check belongs here.
fn resolve_install_ids<'a>(
    only_memory: bool,
    skills: &[String],
    paths: impl Iterator<Item = &'a str>,
    subset_domain: &str,
) -> anyhow::Result<Vec<String>> {
    if !only_memory {
        return Ok(skills.to_vec());
    }
    let ids = subset_ids(paths, subset_domain);
    if ids.is_empty() {
        bail!("--only-memory: no skills enumerated under '{subset_domain}'");
    }
    Ok(ids.into_iter().collect())
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

/// The Claude skills directory (project-local or, with `global`, user home).
fn claude_dir(root: &Path, global: bool) -> anyhow::Result<PathBuf> {
    Ok(install_base(root, global)?.join(".claude/skills"))
}

// ---------------------------------------------------------------------------
// Pure: canonical tree + ownership-by-target-equality (SL-010 D3)
//
// A managed agent link is doctrine's *iff its value equals the relative target
// we would write* — type (is_symlink) is necessary but not sufficient. Anything
// else (a foreign symlink, or a real dir/file) is kept untouched. This is both
// the never-clobber guarantee and the override hatch.
// ---------------------------------------------------------------------------

/// The canonical skills tree (project-local, or under `$HOME` with `global`).
/// Mirrors `claude_dir`'s base so the relative link target is stable.
fn canonical_dir(root: &Path, global: bool) -> anyhow::Result<PathBuf> {
    Ok(install_base(root, global)?.join(".doctrine/skills"))
}

// ---------------------------------------------------------------------------
// Agents leg (SL-056 PHASE-11) — install the Claude dispatch-worker agent def
// the same way skills install: materialize a canonical copy from the embed,
// then symlink the agent dir at it (reusing classify_link/write_link/
// relative_target — no parallel symlink impl).
// ---------------------------------------------------------------------------

/// The dispatch-worker agent def's file name — the canonical-copy id and the
/// `.claude/agents/` link name.
const DISPATCH_WORKER_AGENT_FILE: &str = "dispatch-worker.md";

/// The embedded source of the agent def, relative to `install/`.
const DISPATCH_WORKER_AGENT_ASSET: &str = "agents/claude/dispatch-worker.md";

/// The embedded source of the pi agent def, relative to `install/`.
const DISPATCH_WORKER_AGENT_ASSET_PI: &str = "agents/pi/dispatch-worker.md";

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

/// Classify each selected skill's agent link against its canonical target.
fn claude_links(skills: &[&Entry], agent_dir: &Path, canon_dir: &Path) -> Vec<Link> {
    skills
        .iter()
        .map(|e| {
            let dest = agent_dir.join(&e.id);
            let target = relative_target(agent_dir, canon_dir, &e.id);
            classify_link(&e.id, &dest, &target)
        })
        .collect()
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

/// Build the cross-agent install plan.
fn build_plan(
    root: &Path,
    agents: &[Agent],
    all: &[Entry],
    ids: &[String],
    domains: &[String],
    global: bool,
    repo: &str,
) -> anyhow::Result<Plan> {
    let selected = select(all, ids, domains);
    let subset = !(ids.is_empty() && domains.is_empty());

    let mut items = Vec::new();
    for agent in agents {
        match agent {
            Agent::Claude => {
                let agent_dir = claude_dir(root, global)?;
                let canon_dir = canonical_dir(root, global)?;
                let canonical = selected
                    .iter()
                    .map(|e| Canonical {
                        id: e.id.clone(),
                        dest: canon_dir.join(&e.id),
                    })
                    .collect();
                let links = claude_links(&selected, &agent_dir, &canon_dir);
                items.push(AgentPlan::Claude { canonical, links });
            }
            Agent::Other(name) => items.push(AgentPlan::Delegate {
                agent: name.clone(),
                argv: delegate_argv(&[name.as_str()], &selected, global, subset, repo),
            }),
        }
    }

    Ok(Plan {
        root: root.to_path_buf(),
        items,
    })
}

// ---------------------------------------------------------------------------
// Pure: agent resolution
// ---------------------------------------------------------------------------

fn parse_agent(s: &str) -> Agent {
    if s.eq_ignore_ascii_case("claude") {
        Agent::Claude
    } else {
        Agent::Other(s.to_string())
    }
}

/// Resolve target agents: explicit list, else auto-detect by marker
/// (`.claude/` → Claude; `.codex/` → codex; `.pi/` → pi; `.agents/` → universal;
/// non-Claude-alias `AGENTS.md` → codex).
/// Mirrors `boot::resolve_harnesses` and `install::detect_agents` — all three must stay in sync.
fn resolve_agents(explicit: &[String], root: &Path) -> anyhow::Result<Vec<Agent>> {
    if !explicit.is_empty() {
        return Ok(explicit.iter().map(|s| parse_agent(s)).collect());
    }
    let claude = root.join(".claude").exists();
    let mut found = Vec::new();
    if claude {
        found.push(Agent::Claude);
    }
    // AGENTS.md is "merely Claude's alias" only when Claude is detected AND
    // AGENTS.md resolves to CLAUDE.md's inode (this repo's symlink). Otherwise a
    // present AGENTS.md is a real codex surface.
    let agents = root.join("AGENTS.md");
    let agents_is_claude_alias = claude
        && agents.exists()
        && fs::canonicalize(&agents).ok() == fs::canonicalize(root.join("CLAUDE.md")).ok();
    if root.join(".codex").exists() || (agents.exists() && !agents_is_claude_alias) {
        found.push(Agent::Other("codex".into()));
    }
    if root.join(".pi").exists() {
        found.push(Agent::Other("pi".into()));
    }
    if root.join(".agents").exists() {
        found.push(Agent::Other("universal".into()));
    }
    if found.is_empty() {
        bail!(
            "No --agent given and no .claude/, .codex/, .pi/, or .agents/ (or AGENTS.md) found. \
             Pass --agent <name> (e.g. claude, codex, pi, cursor)."
        );
    }
    Ok(found)
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

/// Materialise the canonical copy of `entry` at `dest` (`.doctrine/skills/<id>`),
/// staged via a `.tmp-<id>` sibling then swapped in with a minimal-window
/// remove+rename. Always overwrites — the canonical tree is derived (owns no
/// authored data).
///
/// Unix reality (design §5.1/§10 pass-2 F4): `rename` cannot replace a non-empty
/// directory and std has no `renameat2(RENAME_EXCHANGE)`, so the swap is
/// remove-then-rename — a one-syscall window where a crash leaves the agent link
/// dangling, healed by the next idempotent install. A partial stage lives only
/// under `.tmp-<id>`, never under `<id>`, so a live link never sees a half-tree.
fn materialise_canonical(entry: &Entry, dest: &Path) -> anyhow::Result<()> {
    let tmp = staging_path(dest)?;

    // Clear any crashed leftover from a prior interrupted stage. Use lexists
    // (symlink_metadata, not exists()): a leftover is normally a partial dir, but
    // an odd dangling symlink must also be cleared — exists() follows it and would
    // miss it, then `copy_skill`'s create_dir_all would fail on the stale link.
    match fs::symlink_metadata(&tmp) {
        Ok(m) if m.file_type().is_dir() => fs::remove_dir_all(&tmp)
            .with_context(|| format!("Failed to clear stale {}", tmp.display()))?,
        Ok(_) => {
            fs::remove_file(&tmp)
                .with_context(|| format!("Failed to clear stale {}", tmp.display()))?;
        }
        Err(_) => {}
    }
    // Stage the embed into the temp (same filesystem → the rename below is valid).
    copy_skill(entry, &tmp)?;
    // Minimal-window swap: drop the prior canonical, then rename the temp in.
    if dest.exists() {
        fs::remove_dir_all(dest).with_context(|| format!("Failed to remove {}", dest.display()))?;
    }
    fs::rename(&tmp, dest)
        .with_context(|| format!("Failed to swap {} → {}", tmp.display(), dest.display()))?;
    Ok(())
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

/// Execute a plan. `catalog` resolves Claude steps back to embedded files.
fn execute(
    plan: &Plan,
    catalog: &[Entry],
    runner: &dyn Runner,
    runner_name: &str,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let mut failed: Vec<String> = Vec::new();

    for item in &plan.items {
        match item {
            AgentPlan::Claude { canonical, links } => {
                writeln!(out, "agent claude (direct):")?;
                // 1. Refresh the canonical tree (always overwrite — derived).
                for c in canonical {
                    let entry = catalog
                        .iter()
                        .find(|e| e.id == c.id)
                        .with_context(|| format!("Skill '{}' vanished from catalog", c.id))?;
                    materialise_canonical(entry, &c.dest)?;
                    writeln!(out, "  refreshed {}", c.id)?;
                }
                // 2. Reconcile the agent links by proven ownership. Re-classify
                // at mutation time, not just from the plan: a foreign symlink/file
                // could appear at `dest` between build_plan and here (the confirm
                // window, or a concurrent install). `rename` would silently clobber
                // it — so we re-prove ownership and keep-foreign if it changed,
                // upholding the never-clobber invariant (design §5.5). A real dir
                // is already safe (rename cannot replace a directory).
                for link in links {
                    let (id, dest, target) = match link {
                        Link::Create { id, dest, target } | Link::Relink { id, dest, target } => {
                            (id, dest, target)
                        }
                        Link::KeepForeign { id, dest, reason } => {
                            let _ = dest;
                            writeln!(out, "  kept      {id} ({})", foreign_reason(reason))?;
                            continue;
                        }
                    };
                    match classify_link(id, dest, target) {
                        Link::Create { .. } => {
                            write_link(dest, target)?;
                            writeln!(out, "  linked    {id}")?;
                        }
                        Link::Relink { .. } => {
                            write_link(dest, target)?;
                            writeln!(out, "  relinked  {id}")?;
                        }
                        Link::KeepForeign { reason, .. } => {
                            writeln!(out, "  kept      {id} ({})", foreign_reason(&reason))?;
                        }
                    }
                }
            }
            AgentPlan::Delegate { agent, argv } => {
                writeln!(
                    out,
                    "agent {agent} (delegate): {runner_name} {}",
                    argv.join(" ")
                )?;
                if !runner.run(runner_name, argv)? {
                    failed.push(agent.clone());
                }
            }
        }
    }

    if !failed.is_empty() {
        bail!("npx skills failed for agent(s): {}", failed.join(", "));
    }
    Ok(())
}

/// Install skills for Claude: refresh canonical tree + reconcile agent symlinks.
/// Extracted from `execute()` for reuse from the consolidated `install::run()`
/// forward-step dispatch (SL-088 PHASE-02).
pub(crate) fn install_for_claude(
    root: &Path,
    catalog: &[Entry],
    selected: &[&Entry],
    global: bool,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let agent_dir = claude_dir(root, global)?;
    let canon_dir = canonical_dir(root, global)?;
    let canonical: Vec<Canonical> = selected
        .iter()
        .map(|e| Canonical {
            id: e.id.clone(),
            dest: canon_dir.join(&e.id),
        })
        .collect();
    let links = claude_links(selected, &agent_dir, &canon_dir);

    writeln!(out, "agent claude (direct):")?;
    // 1. Refresh the canonical tree (always overwrite — derived).
    for c in &canonical {
        let entry = catalog
            .iter()
            .find(|e| e.id == c.id)
            .with_context(|| format!("Skill '{}' vanished from catalog", c.id))?;
        materialise_canonical(entry, &c.dest)?;
        writeln!(out, "  refreshed {}", c.id)?;
    }
    // 2. Reconcile the agent links by proven ownership.
    for link in &links {
        let (id, dest, target) = match link {
            Link::Create { id, dest, target } | Link::Relink { id, dest, target } => {
                (id, dest, target)
            }
            Link::KeepForeign { id, dest, reason } => {
                let _ = dest;
                writeln!(out, "  kept      {id} ({})", foreign_reason(reason))?;
                continue;
            }
        };
        match classify_link(id, dest, target) {
            Link::Create { .. } => {
                write_link(dest, target)?;
                writeln!(out, "  linked    {id}")?;
            }
            Link::Relink { .. } => {
                write_link(dest, target)?;
                writeln!(out, "  relinked  {id}")?;
            }
            Link::KeepForeign { reason, .. } => {
                writeln!(out, "  kept      {id} ({})", foreign_reason(&reason))?;
            }
        }
    }
    Ok(())
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

fn print_plan(plan: &Plan, out: &mut dyn Write) -> io::Result<()> {
    writeln!(out, "Project root: {}", plan.root.display())?;
    writeln!(out)?;
    for item in &plan.items {
        match item {
            AgentPlan::Claude { canonical, links } => {
                writeln!(out, "agent claude (direct):")?;
                for c in canonical {
                    writeln!(out, "  refresh   {} → {}", c.id, c.dest.display())?;
                }
                for link in links {
                    match link {
                        Link::Create { id, dest, target } => {
                            writeln!(
                                out,
                                "  link      {id} → {} ⇒ {}",
                                dest.display(),
                                target.display()
                            )?;
                        }
                        Link::Relink { id, dest, target } => {
                            writeln!(
                                out,
                                "  relink    {id} → {} ⇒ {}",
                                dest.display(),
                                target.display()
                            )?;
                        }
                        Link::KeepForeign { id, dest, reason } => {
                            writeln!(
                                out,
                                "  keep      {id} → {} ({})",
                                dest.display(),
                                foreign_reason(reason)
                            )?;
                        }
                    }
                }
            }
            AgentPlan::Delegate { agent, argv } => {
                writeln!(out, "agent {agent} (delegate):")?;
                writeln!(out, "  npx {}", argv.join(" "))?;
            }
        }
    }
    Ok(())
}

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
    let data = crate::install::embedded_asset(embed_asset)
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

/// Install the doctrine hooks plugin directly into `.claude/skills/doctrine/`.
/// Claude auto-discovers skills-directory plugins — any folder under a skills
/// dir containing `.claude-plugin/plugin.json` loads as `<name>@skills-dir`
/// with all components (here: just hooks) active.
pub(crate) fn install_hooks_plugin_for_claude(
    root: &Path,
    global: bool,
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let skills_dir = claude_dir(root, global)?;
    let plugin_dir = skills_dir.join("doctrine");
    let manifest_dir = plugin_dir.join(".claude-plugin");
    let hooks_dir = plugin_dir.join("hooks");

    writeln!(out, "hooks (skills-dir plugin):")?;

    fs::create_dir_all(&manifest_dir)
        .with_context(|| format!("Failed to create {}", manifest_dir.display()))?;
    fs::create_dir_all(&hooks_dir)
        .with_context(|| format!("Failed to create {}", hooks_dir.display()))?;

    let manifest = PluginAssets::get("doctrine/.claude-plugin/plugin.json")
        .context("Embedded plugin manifest 'doctrine/.claude-plugin/plugin.json' not found")?;
    crate::fsutil::write_atomic(&manifest_dir.join("plugin.json"), &manifest.data)?;
    writeln!(out, "  refreshed .claude-plugin/plugin.json")?;

    let hooks = PluginAssets::get("doctrine/hooks/hooks.json")
        .context("Embedded hooks config 'doctrine/hooks/hooks.json' not found")?;
    // SL-182 F-1: bake fail-closed exec resolution at materialization, not a
    // verbatim byte-copy — see `template_hooks_commands`.
    let embedded =
        std::str::from_utf8(&hooks.data).context("embedded plugin hooks.json is not UTF-8")?;
    let exec = crate::boot::resolve_exec()?;
    let templated = template_hooks_commands(embedded, &exec)?;
    crate::fsutil::write_atomic(&hooks_dir.join("hooks.json"), templated.as_bytes())?;
    writeln!(
        out,
        "  refreshed hooks/hooks.json (exec baked: {})",
        exec.display()
    )?;

    Ok(())
}

// ---- SL-182 PHASE-03: fail-closed hook exec templating (F-1, design §5.4) -------
/// The bare program token in the embedded (checked-in) plugin hook commands.
const HOOK_DOCTRINE_TOKEN: &str = "doctrine";
/// The confinement subcommand — its command gets the fail-closed vanish-guard.
const HOOK_PRETOOLUSE_SUBCMD: &str = "worktree pretooluse";
/// Shell suffix converting a vanished/`127` exec into a blocking `exit 2` ⇒ deny
/// (`PreToolUse` fails OPEN otherwise — `mem.fact.claude.pretooluse-hook-fail-open`).
const HOOK_EXIT2_GUARD: &str = " || exit 2";

/// Bake fail-closed exec resolution into the embedded plugin hooks (SL-182 F-1,
/// design §5.4). `PreToolUse` hooks fail OPEN — only `exit 2` blocks — so a bare
/// `doctrine` on PATH that resolves to a stale/absent binary would let a guarded
/// tool run UNCONFINED (the RSK-014 hole, reopened by the installer). Rewrite each
/// command's leading `doctrine` token to the resolved absolute `exec`
/// (single-quote-escaped, INV-5), reaching parity with the settings `HookSpec` bake;
/// append `|| exit 2` to the confinement (`pretooluse`) command so a vanished
/// binary DENIES. Leading-token only — args untouched, so the checked-in asset
/// stays valid as authored (EX-2).
fn template_hooks_commands(embedded: &str, exec: &Path) -> anyhow::Result<String> {
    let mut doc: serde_json::Value =
        serde_json::from_str(embedded).context("parse embedded plugin hooks.json")?;
    let exec_quoted = crate::worktree::shell_single_quote(&exec.to_string_lossy());
    let events = doc
        .get_mut("hooks")
        .and_then(serde_json::Value::as_object_mut)
        .context("plugin hooks.json missing a `hooks` object")?;
    for entries in events.values_mut() {
        let Some(entries) = entries.as_array_mut() else {
            continue;
        };
        for entry in entries.iter_mut() {
            let Some(hooks) = entry
                .get_mut("hooks")
                .and_then(serde_json::Value::as_array_mut)
            else {
                continue;
            };
            for hook in hooks.iter_mut() {
                let Some(cmd) = hook.get("command").and_then(serde_json::Value::as_str) else {
                    continue;
                };
                let rewritten = template_command(cmd, &exec_quoted);
                hook["command"] = serde_json::Value::String(rewritten);
            }
        }
    }
    serde_json::to_string_pretty(&doc).context("serialize templated hooks.json")
}

/// Rewrite one hook command: leading `doctrine` → `exec_quoted`, plus the
/// `|| exit 2` vanish-guard on the `pretooluse` command. A command not led by a
/// bare `doctrine` token is left verbatim (defensive — no such command ships).
fn template_command(cmd: &str, exec_quoted: &str) -> String {
    let base = match cmd.strip_prefix(HOOK_DOCTRINE_TOKEN) {
        Some(rest) if rest.is_empty() || rest.starts_with(' ') => format!("{exec_quoted}{rest}"),
        _ => cmd.to_string(),
    };
    if cmd.contains(HOOK_PRETOOLUSE_SUBCMD) {
        format!("{base}{HOOK_EXIT2_GUARD}")
    } else {
        base
    }
}

// ---------------------------------------------------------------------------
// CLI entry points (thin)
// ---------------------------------------------------------------------------

/// Does a path exist *without following symlinks*? A managed agent link — even
/// momentarily dangling during a canonical refresh — counts as installed
/// (SL-010 F5); `Path::exists` follows the link and would hide it.
fn lexists(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

/// `doctrine skills list`.
pub(crate) fn run_list(agent: Option<&str>, installed_only: bool) -> anyhow::Result<()> {
    let catalog = discover()?;
    let root = crate::root::find(None, &crate::root::default_markers())?;
    let claude_present = matches!(agent.map(parse_agent), None | Some(Agent::Claude));
    let dir = root.join(".claude/skills");

    let mut out = io::stdout();
    let mut domain = String::new();
    for entry in &catalog {
        let installed = lexists(&dir.join(&entry.id));
        if installed_only && !installed {
            continue;
        }
        if entry.domain != domain {
            domain.clone_from(&entry.domain);
            writeln!(out, "{domain}")?;
        }
        let status = if !claude_present {
            "claude: n/a".to_string()
        } else if installed {
            "claude: installed".to_string()
        } else {
            "claude: —".to_string()
        };
        writeln!(
            out,
            "  {:<16} {:<48} [{status}]",
            entry.id, entry.description
        )?;
    }
    Ok(())
}

/// `doctrine claude install` arguments (selection + flags), shared by the hidden
/// deprecated `skills install` alias. Mirrors the `memory::RecordArgs` pattern so
/// the command handler stays under the bool/arg clippy ceilings; `path` stays a
/// separate param, as in `run_record`.
pub(crate) struct InstallArgs<'a> {
    pub(crate) agents: &'a [String],
    pub(crate) skills: &'a [String],
    pub(crate) domains: &'a [String],
    /// `--only-memory`: derive the subset from the `doctrine-memory` plugin.
    pub(crate) only_memory: bool,
    pub(crate) global: bool,
    pub(crate) dry_run: bool,
    pub(crate) yes: bool,
}

/// The shared `claude install` handler (SL-056): installs the Claude skills, the
/// dispatch-worker agent def (`install_agents`), and the `SubagentStart` stamp
/// hook (`boot::install_claude_hook`). Both the `claude install` verb and the
/// hidden deprecated `skills install` alias dispatch here (SR-3).
pub(crate) fn run_install(path: Option<PathBuf>, args: &InstallArgs<'_>) -> anyhow::Result<()> {
    let catalog = discover()?;
    // `--only-memory` derives the subset from the embed; otherwise pass `skills`
    // through. The thin shell supplies the live paths; the resolver is pure.
    let live: Vec<String> = PluginAssets::iter()
        .map(|p| p.as_ref().to_string())
        .collect();
    let skills = resolve_install_ids(
        args.only_memory,
        args.skills,
        live.iter().map(String::as_str),
        MEMORY_SUBSET_DOMAIN,
    )?;
    validate_filters(&catalog, &skills, args.domains)?;
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let agents = resolve_agents(args.agents, &root)?;
    let repo = crate::dtoml::load_doctrine_toml(&root)?.install.repo;
    let plan = build_plan(
        &root,
        &agents,
        &catalog,
        &skills,
        args.domains,
        args.global,
        &repo,
    )?;

    let mut out = io::stdout();
    print_plan(&plan, &mut out)?;

    if args.dry_run {
        return Ok(());
    }
    if !args.yes && !crate::install::prompt_confirm("\nProceed? [y/N] ")? {
        writeln!(out, "Aborted.")?;
        return Ok(());
    }

    // Self-enforce the derived-tree ignore invariant (SL-010 F4): `skills install`
    // owns `.doctrine/skills/*` regardless of whether `doctrine install` ran first.
    // Anchor the ignore at the same base the canonical tree is written to, so
    // `--global` ignores its $HOME tree rather than the project (SL-010 B1).
    crate::install::ensure_gitignored(&install_base(&root, args.global)?, ".doctrine/skills/*")?;
    let (runner_name, runner) = resolve_runner();
    execute(&plan, &catalog, &runner, runner_name, &mut out)?;

    // Agents leg + SubagentStart hook are Claude-surface-only: install them iff
    // Claude is a resolved target (skip a codex-only / global-npx install).
    if agents.iter().any(|a| matches!(a, Agent::Claude)) {
        // Ignore the derived agents (e.g. dispatch-worker.md) but re-include the
        // authored, tracked AGENTS.md — emit the whitelist pair in order (the
        // `*` exclude before its negation, so the re-include takes). ISS-012.
        let base = install_base(&root, args.global)?;
        crate::install::ensure_gitignored(&base, ".doctrine/agents/*")?;
        crate::install::ensure_gitignored(&base, "!.doctrine/agents/AGENTS.md")?;
        install_agents_for(&root, "claude", None, args.global, args.dry_run, &mut out)?;

        // Hooks install as a skills-directory plugin — Claude auto-discovers
        // `.claude/skills/doctrine/.claude-plugin/plugin.json` and loads hooks
        // with no marketplace install step.
        if let Err(e) = install_hooks_plugin_for_claude(&root, args.global, &mut out) {
            writeln!(out, "  hooks plugin install failed: {e:#}")?;
        }
    }

    writeln!(out, "Done.")?;
    Ok(())
}

// ── CLI dispatch ───────────────────────────────────────────────────────────

use clap::Subcommand;

#[derive(Subcommand)]
pub(crate) enum SkillsCommand {
    /// List available skills and their install status.
    List {
        /// Agent to report status for (default: claude).
        #[arg(short = 'a', long)]
        agent: Option<String>,

        /// Only show skills already installed.
        #[arg(long)]
        installed: bool,
    },
}

pub(crate) fn dispatch(cmd: SkillsCommand, _color: bool) -> anyhow::Result<()> {
    match cmd {
        SkillsCommand::List { agent, installed } => run_list(agent.as_deref(), installed),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    const TEST_REPO: &str = "davidlee/doctrine";

    // ── SL-182 PHASE-03 VT-2: fail-closed hook exec templating (F-1) ────────────

    /// A path with a space exercises the INV-5 single-quote discipline.
    const SPACED_EXEC: &str = "/opt/my tools/doctrine";

    #[test]
    fn templating_bakes_absolute_exec_into_every_command() {
        // Mirrors the embedded asset shape: a SessionStart + a PreToolUse entry.
        let embedded = r#"{"hooks":{
            "SessionStart":[{"matcher":"*","hooks":[{"type":"command","command":"doctrine boot --emit"}]}],
            "PreToolUse":[{"matcher":"Bash","hooks":[{"type":"command","command":"doctrine worktree pretooluse"}]}]
        }}"#;
        let out = template_hooks_commands(embedded, Path::new(SPACED_EXEC)).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();

        let session = v["hooks"]["SessionStart"][0]["hooks"][0]["command"]
            .as_str()
            .unwrap();
        assert_eq!(
            session, "'/opt/my tools/doctrine' boot --emit",
            "leading token → single-quoted absolute exec; args untouched"
        );
        assert!(
            !session.starts_with("doctrine "),
            "no bare-PATH `doctrine` leading token survives (fail-open closed)"
        );

        let ptu = v["hooks"]["PreToolUse"][0]["hooks"][0]["command"]
            .as_str()
            .unwrap();
        assert_eq!(
            ptu, "'/opt/my tools/doctrine' worktree pretooluse || exit 2",
            "the confinement command carries the fail-closed vanish-guard"
        );
    }

    #[test]
    fn templating_guards_only_pretooluse_not_other_hooks() {
        assert_eq!(
            template_command("doctrine worktree create-fork", "'/x/doctrine'"),
            "'/x/doctrine' worktree create-fork",
            "non-pretooluse command gets the exec bake but NO exit-2 guard"
        );
        assert_eq!(
            template_command("doctrine worktree pretooluse", "'/x/doctrine'"),
            "'/x/doctrine' worktree pretooluse || exit 2",
        );
        // Defensive: a command not led by a bare `doctrine` token is left verbatim.
        assert_eq!(
            template_command("/usr/bin/env doctrine boot", "'/x/doctrine'"),
            "/usr/bin/env doctrine boot",
        );
    }

    #[test]
    fn install_materializes_fail_closed_hooks_json() {
        // End-to-end EX-2/EX-3: the real install path bakes an ABSOLUTE exec (here
        // the test binary via resolve_exec) into every command — no bare-PATH
        // `doctrine` survives — and the pretooluse walls carry `|| exit 2`.
        let tmp = tempfile::tempdir().unwrap();
        let mut sink = Vec::new();
        install_hooks_plugin_for_claude(tmp.path(), false, &mut sink).unwrap();

        let materialized = tmp.path().join(".claude/skills/doctrine/hooks/hooks.json");
        let body = fs::read_to_string(&materialized).unwrap();
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();

        let mut commands = Vec::new();
        for entries in v["hooks"].as_object().unwrap().values() {
            for entry in entries.as_array().unwrap() {
                for hook in entry["hooks"].as_array().unwrap() {
                    commands.push(hook["command"].as_str().unwrap().to_string());
                }
            }
        }
        assert!(!commands.is_empty(), "some commands materialized");
        for cmd in &commands {
            assert!(
                cmd.starts_with('\''),
                "every command leads with a single-quoted absolute exec, not bare PATH: {cmd}"
            );
            assert!(
                !cmd.starts_with("doctrine "),
                "no fail-open bare-PATH doctrine survives: {cmd}"
            );
        }
        let guarded: Vec<&String> = commands
            .iter()
            .filter(|c| c.contains(HOOK_PRETOOLUSE_SUBCMD))
            .collect();
        assert_eq!(guarded.len(), 2, "both pretooluse walls materialized");
        for cmd in guarded {
            assert!(cmd.ends_with(HOOK_EXIT2_GUARD), "fail-closed guard: {cmd}");
        }
    }

    #[test]
    fn materialized_hooks_ship_no_worktree_remove_or_subagent_stop_entry() {
        // SL-182 PHASE-05 (symmetric live-import, RV-205 F-2 / AF-3): the funnel relies
        // on the worker worktree PERSISTING post-return so the orchestrator can import
        // its live delta. That persistence holds ONLY because doctrine ships `create-fork`
        // as the `WorktreeCreate` hook with NO paired `WorktreeRemove` hook — a
        // WorktreeRemove entry would reap the tree (and its uncommitted delta) before the
        // import runs (`docs/claude/hooks.md:2442`, `mem_019f1a5c…`). This install-time
        // boundary asserts no such hook — and no retired `SubagentStop` capture entry —
        // ever materializes. (The runtime boundary is `verify-worker --dir`'s
        // absence-catch; two boundaries per RV-205 F-2.)
        let tmp = tempfile::tempdir().unwrap();
        let mut sink = Vec::new();
        install_hooks_plugin_for_claude(tmp.path(), false, &mut sink).unwrap();
        let body = fs::read_to_string(tmp.path().join(".claude/skills/doctrine/hooks/hooks.json"))
            .unwrap();
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        let events = v["hooks"].as_object().expect("hooks object materialized");
        assert!(
            !events.contains_key("WorktreeRemove"),
            "no WorktreeRemove hook may ship — it would reap the worker tree pre-import (AF-3)"
        );
        assert!(
            !events.contains_key("SubagentStop"),
            "the retired SubagentStop capture entry must not materialize"
        );
    }

    #[test]
    fn embedded_hooks_asset_ships_bare_doctrine_and_both_pretooluse_walls() {
        // The checked-in asset stays authored-bare (EX-2); templating is install-time.
        // Also pins T7: both PreToolUse matchers (Bash + Edit|Write) are registered.
        let raw = PluginAssets::get("doctrine/hooks/hooks.json").unwrap();
        let v: serde_json::Value = serde_json::from_slice(&raw.data).unwrap();
        let ptu = v["hooks"]["PreToolUse"].as_array().unwrap();
        let matchers: Vec<&str> = ptu.iter().map(|e| e["matcher"].as_str().unwrap()).collect();
        assert!(matchers.contains(&"Bash"), "Bash wall registered");
        assert!(
            matchers.contains(&"Edit|Write"),
            "Edit|Write wall registered"
        );
        for e in ptu {
            assert_eq!(
                e["hooks"][0]["command"], "doctrine worktree pretooluse",
                "asset ships the bare command (templated at install)"
            );
        }
    }

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

    #[test]
    fn subset_ids_extracts_only_the_named_domain() {
        // VT-1: `<domain>/skills/<id>/…` → {id}; other domains and non-skill
        // paths (README, plugin.json) are ignored.
        let paths = [
            "doctrine-memory/skills/record-memory/SKILL.md",
            "doctrine-memory/skills/retrieve-memory/SKILL.md",
            "doctrine-memory/README.md",
            "doctrine-memory/.claude-plugin/plugin.json",
            "doctrine/skills/route/SKILL.md",
        ];
        let ids = subset_ids(paths.iter().copied(), "doctrine-memory");
        assert_eq!(
            ids,
            ["record-memory".to_string(), "retrieve-memory".to_string()]
                .into_iter()
                .collect()
        );
    }

    #[test]
    fn subset_ids_absent_domain_is_empty() {
        let paths = ["doctrine/skills/route/SKILL.md"];
        assert!(subset_ids(paths.iter().copied(), "doctrine-memory").is_empty());
    }

    #[test]
    fn resolve_install_ids_passes_skills_through_when_not_only_memory() {
        let got = resolve_install_ids(
            false,
            &["foo".into()],
            std::iter::empty(),
            MEMORY_SUBSET_DOMAIN,
        )
        .unwrap();
        assert_eq!(got, vec!["foo".to_string()]);
    }

    #[test]
    fn resolve_install_ids_derives_the_subset_when_only_memory() {
        let paths = [
            "doctrine-memory/skills/record-memory/SKILL.md",
            "doctrine-memory/skills/retrieve-memory/SKILL.md",
        ];
        let got = resolve_install_ids(true, &[], paths.iter().copied(), "doctrine-memory").unwrap();
        assert_eq!(
            got,
            vec!["record-memory".to_string(), "retrieve-memory".to_string()]
        );
    }

    #[test]
    fn resolve_install_ids_bails_on_empty_derivation() {
        // The select([]) == all guard (D3): an empty subset must fail loud, never
        // silently fall through to installing the entire catalog. Pure — no embed.
        let paths = ["doctrine/skills/route/SKILL.md"];
        assert!(resolve_install_ids(true, &[], paths.iter().copied(), "doctrine-memory").is_err());
    }

    #[test]
    fn resolve_install_ids_live_embed_yields_the_memory_pair() {
        // VT-2: pins embed-follows-symlinks. If rust-embed stops descending the
        // doctrine-memory symlinks this goes red — the flag is broken, not the test.
        let live: Vec<String> = PluginAssets::iter()
            .map(|p| p.as_ref().to_string())
            .collect();
        let got = resolve_install_ids(
            true,
            &[],
            live.iter().map(String::as_str),
            MEMORY_SUBSET_DOMAIN,
        )
        .unwrap();
        assert_eq!(
            got,
            vec!["record-memory".to_string(), "retrieve-memory".to_string()]
        );
    }

    #[test]
    fn only_memory_selects_exactly_the_two_canonical_skills() {
        // VT-3: cross-domain identity (§5.5). Ids derived from the discover-EXCLUDED
        // doctrine-memory domain validate against the catalog where they live under
        // the doctrine domain, and select exactly those two — no more, no less.
        let catalog = discover().unwrap();
        let live: Vec<String> = PluginAssets::iter()
            .map(|p| p.as_ref().to_string())
            .collect();
        let ids = resolve_install_ids(
            true,
            &[],
            live.iter().map(String::as_str),
            MEMORY_SUBSET_DOMAIN,
        )
        .unwrap();
        validate_filters(&catalog, &ids, &[]).unwrap();
        let selected = select(&catalog, &ids, &[]);
        let got: BTreeSet<&str> = selected.iter().map(|e| e.id.as_str()).collect();
        assert_eq!(
            got,
            ["record-memory", "retrieve-memory"].into_iter().collect()
        );
        assert!(selected.iter().all(|e| e.domain == "doctrine"));
    }

    // --- claude links (the plan builder) ---

    #[test]
    fn claude_links_creates_then_relinks_an_owned_link() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().unwrap();
        let agent_dir = dir.path().join(".claude/skills");
        let canon_dir = dir.path().join(".doctrine/skills");
        fs::create_dir_all(&agent_dir).unwrap();
        let e = entry("review", "code-review");
        let sel = vec![&e];

        // Nothing there → Create with the computed relative target.
        let links = claude_links(&sel, &agent_dir, &canon_dir);
        assert!(matches!(
            links.as_slice(),
            [Link::Create { target, .. }]
                if target == &PathBuf::from("../../.doctrine/skills/code-review")
        ));

        // An existing link with our target → Relink (ours).
        symlink(
            "../../.doctrine/skills/code-review",
            agent_dir.join("code-review"),
        )
        .unwrap();
        let links = claude_links(&sel, &agent_dir, &canon_dir);
        assert!(matches!(links.as_slice(), [Link::Relink { .. }]));
    }

    // --- canonical materialise ---

    fn code_review_entry() -> Entry {
        discover()
            .unwrap()
            .into_iter()
            .find(|e| e.id == "code-review")
            .unwrap()
    }

    #[test]
    fn materialise_overwrites_stale_canonical() {
        let dir = tempfile::tempdir().unwrap();
        let e = code_review_entry();
        let id_dir = dir.path().join(&e.id);

        // Pre-seed a stale canonical from a prior embed version.
        fs::create_dir_all(&id_dir).unwrap();
        fs::write(id_dir.join("STALE.md"), "old").unwrap();

        materialise_canonical(&e, &id_dir).unwrap();

        // Stale file gone; the embed's SKILL.md present and byte-equal.
        assert!(!id_dir.join("STALE.md").exists(), "stale file must be gone");
        let embed = PluginAssets::get("doctrine/skills/code-review/SKILL.md").unwrap();
        let got = fs::read(id_dir.join("SKILL.md")).unwrap();
        assert_eq!(got, embed.data.as_ref());
        // No temp left behind.
        assert!(!dir.path().join(format!(".tmp-{}", e.id)).exists());
    }

    #[test]
    fn materialise_heals_an_interrupted_stage() {
        let dir = tempfile::tempdir().unwrap();
        let e = code_review_entry();
        let id_dir = dir.path().join(&e.id);
        let tmp = dir.path().join(format!(".tmp-{}", e.id));

        // A prior crash: a leftover temp from an interrupted stage, plus an
        // intact prior canonical — the live state the next install must heal.
        fs::create_dir_all(&tmp).unwrap();
        fs::write(tmp.join("JUNK.md"), "partial").unwrap();
        fs::create_dir_all(&id_dir).unwrap();
        fs::write(id_dir.join("SKILL.md"), "prior").unwrap();

        materialise_canonical(&e, &id_dir).unwrap();

        // Temp cleared; canonical coherent (embed content, no junk leaked in).
        assert!(!tmp.exists(), "leftover temp must be cleared");
        assert!(!id_dir.join("JUNK.md").exists());
        let embed = PluginAssets::get("doctrine/skills/code-review/SKILL.md").unwrap();
        assert_eq!(
            fs::read(id_dir.join("SKILL.md")).unwrap(),
            embed.data.as_ref()
        );
    }

    #[test]
    fn materialise_clears_a_dangling_temp_leftover() {
        use std::os::unix::fs::symlink;
        let dir = tempfile::tempdir().unwrap();
        let e = code_review_entry();
        let id_dir = dir.path().join(&e.id);
        let tmp = dir.path().join(format!(".tmp-{}", e.id));
        // A crashed leftover that is a *dangling symlink*, not a partial dir —
        // exists() would miss it (A1).
        symlink("/no/such/target", &tmp).unwrap();

        materialise_canonical(&e, &id_dir).unwrap();

        assert!(
            fs::symlink_metadata(&tmp).is_err(),
            "dangling temp leftover must be cleared"
        );
        assert_eq!(fs::read(id_dir.join("SKILL.md")).unwrap(), embed_skill_md());
    }

    // --- canonical dir + relative target ---

    #[test]
    fn canonical_dir_is_project_local_or_home() {
        let root = Path::new("/proj");
        assert_eq!(
            canonical_dir(root, false).unwrap(),
            Path::new("/proj/.doctrine/skills")
        );
        let home = PathBuf::from(std::env::var_os("HOME").unwrap());
        assert_eq!(
            canonical_dir(root, true).unwrap(),
            home.join(".doctrine/skills")
        );
    }

    #[test]
    fn install_base_anchors_both_trees_and_the_ignore() {
        // F4's gitignore must land at the SAME base the canonical tree is written
        // to — `install_base` is that single source (SL-010 B1). Project-local: the
        // base IS the root; global: the base is $HOME, so the ignore follows the
        // tree to $HOME instead of polluting the project with an entry for a tree
        // that isn't there.
        let root = Path::new("/proj");
        assert_eq!(install_base(root, false).unwrap(), root);
        assert_eq!(
            canonical_dir(root, false).unwrap(),
            install_base(root, false).unwrap().join(".doctrine/skills")
        );
        assert_eq!(
            claude_dir(root, false).unwrap(),
            install_base(root, false).unwrap().join(".claude/skills")
        );

        let home = PathBuf::from(std::env::var_os("HOME").unwrap());
        assert_eq!(install_base(root, true).unwrap(), home);
        assert_eq!(
            canonical_dir(root, true).unwrap(),
            install_base(root, true).unwrap().join(".doctrine/skills")
        );
    }

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

    #[test]
    fn resolve_agents_explicit() {
        let dir = tempfile::tempdir().unwrap();
        let agents = resolve_agents(&["claude".into(), "codex".into()], dir.path()).unwrap();
        assert_eq!(agents, vec![Agent::Claude, Agent::Other("codex".into())]);
    }

    #[test]
    fn resolve_agents_detects_claude_dir() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".claude")).unwrap();
        assert_eq!(
            resolve_agents(&[], dir.path()).unwrap(),
            vec![Agent::Claude]
        );
    }

    #[test]
    fn resolve_agents_detects_codex_dir() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".codex")).unwrap();
        assert_eq!(
            resolve_agents(&[], dir.path()).unwrap(),
            vec![Agent::Other("codex".into())]
        );
    }

    #[test]
    fn resolve_agents_detects_agents_md() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("AGENTS.md"), "# Test").unwrap();
        assert_eq!(
            resolve_agents(&[], dir.path()).unwrap(),
            vec![Agent::Other("codex".into())]
        );
    }

    #[test]
    fn resolve_agents_detects_both_claude_and_codex() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".claude")).unwrap();
        fs::create_dir_all(dir.path().join(".codex")).unwrap();
        let agents = resolve_agents(&[], dir.path()).unwrap();
        assert!(agents.contains(&Agent::Claude));
        assert!(agents.contains(&Agent::Other("codex".into())));
    }

    #[test]
    fn resolve_agents_errors_without_target() {
        let dir = tempfile::tempdir().unwrap();
        assert!(resolve_agents(&[], dir.path()).is_err());
    }

    // --- plan ---

    #[test]
    fn resolve_agents_detects_pi_dir() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".pi")).unwrap();
        assert_eq!(
            resolve_agents(&[], dir.path()).unwrap(),
            vec![Agent::Other("pi".into())]
        );
    }

    #[test]
    fn resolve_agents_detects_universal_for_dot_agents_dir() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".agents")).unwrap();
        assert_eq!(
            resolve_agents(&[], dir.path()).unwrap(),
            vec![Agent::Other("universal".into())]
        );
    }

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

    #[test]
    fn build_plan_routes_claude_direct_and_others_delegate() {
        let dir = tempfile::tempdir().unwrap();
        let all = vec![entry("review", "code-review")];
        let plan = build_plan(
            dir.path(),
            &[Agent::Claude, Agent::Other("codex".into())],
            &all,
            &[],
            &[],
            false,
            TEST_REPO,
        )
        .unwrap();

        assert!(matches!(plan.items.first(), Some(AgentPlan::Claude { .. })));
        assert!(matches!(
            plan.items.get(1),
            Some(AgentPlan::Delegate { agent, .. }) if agent == "codex"
        ));
    }

    // --- execution ---

    #[derive(Debug, Default)]
    struct FakeRunner {
        calls: RefCell<Vec<Vec<String>>>,
        ok: bool,
    }

    impl Runner for FakeRunner {
        fn run(&self, _program: &str, args: &[String]) -> anyhow::Result<bool> {
            self.calls.borrow_mut().push(args.to_vec());
            Ok(self.ok)
        }
    }

    fn run_claude(root: &Path) -> String {
        let catalog = discover().unwrap();
        let plan = build_plan(
            root,
            &[Agent::Claude],
            &catalog,
            &["code-review".into()],
            &[],
            false,
            TEST_REPO,
        )
        .unwrap();
        let runner = FakeRunner {
            ok: true,
            ..FakeRunner::default()
        };
        let mut out = Vec::new();
        execute(&plan, &catalog, &runner, RUNNER_NPX, &mut out).unwrap();
        assert!(runner.calls.borrow().is_empty(), "no npx for Claude");
        String::from_utf8(out).unwrap()
    }

    fn embed_skill_md() -> Vec<u8> {
        PluginAssets::get("doctrine/skills/code-review/SKILL.md")
            .unwrap()
            .data
            .to_vec()
    }

    #[test]
    fn execute_creates_link_resolving_to_canonical() {
        let dir = tempfile::tempdir().unwrap();
        let log = run_claude(dir.path());

        let link = dir.path().join(".claude/skills/code-review");
        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        // Resolves through the symlink to the materialised canonical content.
        assert_eq!(fs::read(link.join("SKILL.md")).unwrap(), embed_skill_md());
        assert!(
            dir.path()
                .join(".doctrine/skills/code-review/SKILL.md")
                .is_file()
        );
        assert!(log.contains("refreshed code-review"));
        assert!(log.contains("linked    code-review"));
    }

    #[test]
    fn execute_relink_heals_a_dangling_owned_link() {
        use std::os::unix::fs::symlink;
        let dir = tempfile::tempdir().unwrap();
        let agent_dir = dir.path().join(".claude/skills");
        fs::create_dir_all(&agent_dir).unwrap();
        // An owned link that dangles because the canonical does not exist yet.
        symlink(
            "../../.doctrine/skills/code-review",
            agent_dir.join("code-review"),
        )
        .unwrap();
        assert!(
            !agent_dir.join("code-review").exists(),
            "dangling pre-state"
        );

        let log = run_claude(dir.path());

        // Materialise + relink heals it — it now resolves to canonical content.
        assert_eq!(
            fs::read(agent_dir.join("code-review/SKILL.md")).unwrap(),
            embed_skill_md()
        );
        assert!(log.contains("relinked  code-review"));
    }

    #[test]
    fn execute_keeps_a_foreign_real_dir() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join(".claude/skills/code-review");
        fs::create_dir_all(&real).unwrap();
        fs::write(real.join("MINE.md"), "pinned").unwrap();

        let log = run_claude(dir.path());

        // Untouched: still a real dir, the user's file intact.
        assert!(
            !fs::symlink_metadata(&real)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(fs::read_to_string(real.join("MINE.md")).unwrap(), "pinned");
        assert!(log.contains("kept      code-review (real dir)"));
    }

    #[test]
    fn lexists_reports_a_dangling_managed_link_as_installed() {
        use std::os::unix::fs::symlink;
        let dir = tempfile::tempdir().unwrap();
        let link = dir.path().join("code-review");
        // A managed link whose canonical target does not resolve.
        symlink("../../.doctrine/skills/code-review", &link).unwrap();

        assert!(!link.exists(), "exists() follows the link → hidden");
        assert!(lexists(&link), "lexists sees the link → installed (F5)");
    }

    #[test]
    fn execute_keeps_a_foreign_symlink() {
        use std::os::unix::fs::symlink;
        let dir = tempfile::tempdir().unwrap();
        let agent_dir = dir.path().join(".claude/skills");
        fs::create_dir_all(&agent_dir).unwrap();
        symlink("/some/other/place", agent_dir.join("code-review")).unwrap();

        let log = run_claude(dir.path());

        // Left byte/target-identical; never repointed.
        assert_eq!(
            fs::read_link(agent_dir.join("code-review")).unwrap(),
            PathBuf::from("/some/other/place")
        );
        assert!(log.contains("kept      code-review (foreign symlink → /some/other/place)"));
    }

    #[test]
    fn execute_re_keeps_a_dest_that_turned_foreign_after_planning() {
        use std::os::unix::fs::symlink;
        let dir = tempfile::tempdir().unwrap();
        let catalog = discover().unwrap();
        // Plan while the dest is missing → Link::Create.
        let plan = build_plan(
            dir.path(),
            &[Agent::Claude],
            &catalog,
            &["code-review".into()],
            &[],
            false,
            TEST_REPO,
        )
        .unwrap();

        // A foreign symlink appears at dest AFTER planning (the TOCTOU window:
        // confirm prompt, or a concurrent install).
        let agent_dir = dir.path().join(".claude/skills");
        fs::create_dir_all(&agent_dir).unwrap();
        symlink("/some/other/place", agent_dir.join("code-review")).unwrap();

        let runner = FakeRunner {
            ok: true,
            ..FakeRunner::default()
        };
        let mut out = Vec::new();
        execute(&plan, &catalog, &runner, RUNNER_NPX, &mut out).unwrap();

        // execute re-classifies at mutation time → keeps it, never clobbers (A2).
        let log = String::from_utf8(out).unwrap();
        assert!(
            log.contains("kept      code-review (foreign symlink → /some/other/place)"),
            "a dest that turned foreign after planning must be kept: {log}"
        );
        assert_eq!(
            fs::read_link(agent_dir.join("code-review")).unwrap(),
            PathBuf::from("/some/other/place"),
            "the foreign symlink is untouched"
        );
    }

    #[test]
    fn execute_delegates_with_expected_argv() {
        let dir = tempfile::tempdir().unwrap();
        let catalog = discover().unwrap();
        let plan = build_plan(
            dir.path(),
            &[Agent::Other("codex".into())],
            &catalog,
            &[],
            &[],
            false,
            TEST_REPO,
        )
        .unwrap();

        let runner = FakeRunner {
            ok: true,
            ..FakeRunner::default()
        };
        let mut out = Vec::new();
        execute(&plan, &catalog, &runner, RUNNER_NPX, &mut out).unwrap();

        let calls = runner.calls.borrow();
        assert_eq!(calls.len(), 1);
        let first = calls.first().unwrap();
        assert_eq!(first.first().map(String::as_str), Some("skills"));
        assert!(first.iter().any(|a| a == "codex"));
    }

    #[test]
    fn run_install_self_enforces_the_skills_gitignore() {
        let dir = tempfile::tempdir().unwrap();
        // No prior `doctrine install`: no .gitignore, no .doctrine tree.
        run_install(
            Some(dir.path().to_path_buf()),
            &InstallArgs {
                agents: &["claude".into()],
                skills: &["code-review".into()],
                domains: &[],
                only_memory: false,
                global: false,
                dry_run: false,
                yes: true,
            },
        )
        .unwrap();

        let gi = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert!(
            gi.contains(".doctrine/skills/*"),
            "skills install must self-enforce the derived-tree ignore"
        );
        // The agents leg ignores derived agents but must NOT swallow the
        // authored, tracked AGENTS.md — emit the whitelist pair, in order
        // (the `*` exclude before its negation), so re-include actually takes.
        let star = gi.find(".doctrine/agents/*");
        let keep = gi.find("!.doctrine/agents/AGENTS.md");
        assert!(star.is_some(), "agents install must ignore derived agents");
        assert!(
            keep.is_some(),
            "agents install must re-include the authored AGENTS.md"
        );
        assert!(star < keep, "the `*` exclude must precede its negation");
    }

    #[test]
    fn execute_reports_delegate_failure() {
        let dir = tempfile::tempdir().unwrap();
        let catalog = discover().unwrap();
        let plan = build_plan(
            dir.path(),
            &[Agent::Other("codex".into())],
            &catalog,
            &[],
            &[],
            false,
            TEST_REPO,
        )
        .unwrap();

        let runner = FakeRunner {
            ok: false,
            ..FakeRunner::default()
        };
        let mut out = Vec::new();
        assert!(execute(&plan, &catalog, &runner, "npx", &mut out).is_err());
    }
}
