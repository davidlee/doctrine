// SPDX-License-Identifier: GPL-3.0-only
#![allow(
    clippy::same_name_method,
    reason = "rust-embed derive generates conflicting method names"
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

/// Source from which the delegated `npx skills` pulls non-Claude installs.
const DELEGATE_SOURCE: &str = "doctrine/doctrine";

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
    /// `review/skills/code-review/SKILL.md`.
    files: Vec<String>,
}

/// An install target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Agent {
    Claude,
    Other(String),
}

/// One planned action for the Claude direct path.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Step {
    Install { id: String, dest: PathBuf },
    Skip { id: String, dest: PathBuf },
}

/// Per-agent plan: Claude copies files; others delegate to `npx skills`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum AgentPlan {
    Claude(Vec<Step>),
    Delegate { agent: String, argv: Vec<String> },
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
fn discover() -> anyhow::Result<Vec<Entry>> {
    use std::collections::BTreeMap;

    let mut grouped: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for path in PluginAssets::iter() {
        let p = path.as_ref();
        let parts: Vec<&str> = p.split('/').collect();
        if let [domain, "skills", skill, ..] = parts.as_slice() {
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
fn select<'a>(all: &'a [Entry], ids: &[String], domains: &[String]) -> Vec<&'a Entry> {
    all.iter()
        .filter(|e| {
            let id_ok = ids.is_empty() || ids.iter().any(|i| i == &e.id);
            let dom_ok = domains.is_empty() || domains.iter().any(|d| d == &e.domain);
            id_ok && dom_ok
        })
        .collect()
}

/// Validate that every requested id/domain matches at least one skill.
fn validate_filters(all: &[Entry], ids: &[String], domains: &[String]) -> anyhow::Result<()> {
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

/// The Claude skills directory (project-local or, with `global`, user home).
fn claude_dir(root: &Path, global: bool) -> anyhow::Result<PathBuf> {
    if global {
        let home = std::env::var_os("HOME").context("HOME is not set; cannot resolve --global")?;
        Ok(PathBuf::from(home).join(".claude/skills"))
    } else {
        Ok(root.join(".claude/skills"))
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

/// The canonical skills tree (project-local, or under `$HOME` with `global`).
/// Mirrors `claude_dir`'s base so the relative link target is stable.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "consumed by build_plan in PHASE-04")
)]
fn canonical_dir(root: &Path, global: bool) -> anyhow::Result<PathBuf> {
    if global {
        let home = std::env::var_os("HOME").context("HOME is not set; cannot resolve --global")?;
        Ok(PathBuf::from(home).join(".doctrine/skills"))
    } else {
        Ok(root.join(".doctrine/skills"))
    }
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
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "consumed by build_plan in PHASE-04")
)]
fn relative_target(agent_skills_dir: &Path, canonical_dir: &Path, id: &str) -> PathBuf {
    relative_path(agent_skills_dir, &canonical_dir.join(id))
}

/// Why an agent skill path is foreign — left untouched and warned.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "consumed by execute in PHASE-04")
)]
enum ForeignReason {
    /// A real directory or file the user owns (e.g. a pinned copy override).
    RealDir,
    /// A symlink whose value is not our canonical target — points elsewhere.
    ForeignSymlink(PathBuf),
}

/// Reconciliation action for one agent skill link, by proven ownership.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "consumed by execute in PHASE-04")
)]
enum Link {
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
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "consumed by build_plan in PHASE-04")
)]
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

/// Build the Claude direct-install steps (skip existing skill dirs).
fn claude_steps(skills: &[&Entry], dir: &Path) -> Vec<Step> {
    skills
        .iter()
        .map(|e| {
            let dest = dir.join(&e.id);
            if dest.exists() {
                Step::Skip {
                    id: e.id.clone(),
                    dest,
                }
            } else {
                Step::Install {
                    id: e.id.clone(),
                    dest,
                }
            }
        })
        .collect()
}

/// Assemble the `npx skills add …` argv (program `npx` excluded).
fn delegate_argv(agent: &str, skills: &[&Entry], global: bool, subset: bool) -> Vec<String> {
    let mut argv = vec![
        "skills".to_string(),
        "add".to_string(),
        DELEGATE_SOURCE.to_string(),
        "--agent".to_string(),
        agent.to_string(),
    ];
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
) -> anyhow::Result<Plan> {
    let selected = select(all, ids, domains);
    let subset = !(ids.is_empty() && domains.is_empty());

    let mut items = Vec::new();
    for agent in agents {
        match agent {
            Agent::Claude => {
                let dir = claude_dir(root, global)?;
                items.push(AgentPlan::Claude(claude_steps(&selected, &dir)));
            }
            Agent::Other(name) => items.push(AgentPlan::Delegate {
                agent: name.clone(),
                argv: delegate_argv(name, &selected, global, subset),
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

/// Resolve target agents: explicit list, else auto-detect Claude, else error.
fn resolve_agents(explicit: &[String], root: &Path) -> anyhow::Result<Vec<Agent>> {
    if !explicit.is_empty() {
        return Ok(explicit.iter().map(|s| parse_agent(s)).collect());
    }
    if root.join(".claude").exists() {
        return Ok(vec![Agent::Claude]);
    }
    bail!(
        "No --agent given and no .claude/ found. Pass --agent <name> (e.g. claude, codex, cursor)."
    )
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
struct Npx;

impl Runner for Npx {
    fn run(&self, program: &str, args: &[String]) -> anyhow::Result<bool> {
        let status = std::process::Command::new(program)
            .args(args)
            .status()
            .with_context(|| format!("Failed to run '{program}' (is Node installed?)"))?;
        Ok(status.success())
    }
}

/// Materialise the canonical copy of `entry` at `canonical_dir/<id>`, staged via
/// a sibling `.tmp-<id>` then swapped in with a minimal-window remove+rename.
/// Always overwrites — the canonical tree is derived (owns no authored data).
///
/// Unix reality (design §5.1/§10 pass-2 F4): `rename` cannot replace a non-empty
/// directory and std has no `renameat2(RENAME_EXCHANGE)`, so the swap is
/// remove-then-rename — a one-syscall window where a crash leaves the agent link
/// dangling, healed by the next idempotent install. A partial stage lives only
/// under `.tmp-<id>`, never under `<id>`, so a live link never sees a half-tree.
// Used by tests now; wired into execute's Claude arm in PHASE-04. Gate the
// dead-code expectation to non-test builds (under `cfg(test)` it is exercised).
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "wired into execute in PHASE-04")
)]
fn materialise_canonical(entry: &Entry, canonical_dir: &Path) -> anyhow::Result<()> {
    let dest = canonical_dir.join(&entry.id);
    let tmp = canonical_dir.join(format!(".tmp-{}", entry.id));

    // Clear any crashed leftover from a prior interrupted stage.
    if tmp.exists() {
        fs::remove_dir_all(&tmp)
            .with_context(|| format!("Failed to clear stale {}", tmp.display()))?;
    }
    // Stage the embed into the temp (same filesystem → the rename below is valid).
    copy_skill(entry, &tmp)?;
    // Minimal-window swap: drop the prior canonical, then rename the temp in.
    if dest.exists() {
        fs::remove_dir_all(&dest)
            .with_context(|| format!("Failed to remove {}", dest.display()))?;
    }
    fs::rename(&tmp, &dest)
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
    out: &mut dyn Write,
) -> anyhow::Result<()> {
    let mut failed: Vec<String> = Vec::new();

    for item in &plan.items {
        match item {
            AgentPlan::Claude(steps) => {
                writeln!(out, "agent claude (direct):")?;
                for step in steps {
                    match step {
                        Step::Install { id, dest } => {
                            let entry = catalog
                                .iter()
                                .find(|e| &e.id == id)
                                .with_context(|| format!("Skill '{id}' vanished from catalog"))?;
                            copy_skill(entry, dest)?;
                            writeln!(out, "  installed {id}")?;
                        }
                        Step::Skip { id, .. } => writeln!(out, "  skip      {id} (exists)")?,
                    }
                }
            }
            AgentPlan::Delegate { agent, argv } => {
                writeln!(out, "agent {agent} (delegate): npx {}", argv.join(" "))?;
                if !runner.run("npx", argv)? {
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

// ---------------------------------------------------------------------------
// Imperative: printing
// ---------------------------------------------------------------------------

fn print_plan(plan: &Plan, out: &mut dyn Write) -> io::Result<()> {
    writeln!(out, "Project root: {}", plan.root.display())?;
    writeln!(out)?;
    for item in &plan.items {
        match item {
            AgentPlan::Claude(steps) => {
                writeln!(out, "agent claude (direct):")?;
                for step in steps {
                    match step {
                        Step::Install { id, dest } => {
                            writeln!(out, "  install   {id} → {}", dest.display())?;
                        }
                        Step::Skip { id, dest } => {
                            writeln!(out, "  skip      {id} → {} (exists)", dest.display())?;
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

// ---------------------------------------------------------------------------
// CLI entry points (thin)
// ---------------------------------------------------------------------------

/// `doctrine skills list`.
pub(crate) fn run_list(agent: Option<&str>, installed_only: bool) -> anyhow::Result<()> {
    let catalog = discover()?;
    let root = crate::root::find(None, &crate::root::default_markers())?;
    let claude_present = matches!(agent.map(parse_agent), None | Some(Agent::Claude));
    let dir = root.join(".claude/skills");

    let mut out = io::stdout();
    let mut domain = String::new();
    for entry in &catalog {
        let installed = dir.join(&entry.id).exists();
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

/// `doctrine skills install`.
pub(crate) fn run_install(
    path: Option<PathBuf>,
    agents: &[String],
    skills: &[String],
    domains: &[String],
    global: bool,
    dry_run: bool,
    yes: bool,
) -> anyhow::Result<()> {
    let catalog = discover()?;
    validate_filters(&catalog, skills, domains)?;
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let agents = resolve_agents(agents, &root)?;
    let plan = build_plan(&root, &agents, &catalog, skills, domains, global)?;

    let mut out = io::stdout();
    print_plan(&plan, &mut out)?;

    if dry_run {
        return Ok(());
    }
    if !yes && !crate::install::prompt_confirm("\nProceed? [y/N] ")? {
        writeln!(out, "Aborted.")?;
        return Ok(());
    }

    // Self-enforce the derived-tree ignore invariant (SL-010 F4): `skills install`
    // owns `.doctrine/skills/*` regardless of whether `doctrine install` ran first.
    crate::install::ensure_gitignored(&root, ".doctrine/skills/*")?;
    execute(&plan, &catalog, &Npx, &mut out)?;
    writeln!(out, "Done.")?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

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
        assert_eq!(cr.domain, "review");
        assert!(!cr.description.is_empty());
        assert!(cr.files.iter().any(|f| f.ends_with("SKILL.md")));
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

    // --- claude steps ---

    #[test]
    fn claude_steps_install_then_skip_existing() {
        let dir = tempfile::tempdir().unwrap();
        let e = entry("review", "code-review");
        let sel = vec![&e];

        let steps = claude_steps(&sel, dir.path());
        assert!(matches!(steps.as_slice(), [Step::Install { .. }]));

        fs::create_dir_all(dir.path().join("code-review")).unwrap();
        let steps = claude_steps(&sel, dir.path());
        assert!(matches!(steps.as_slice(), [Step::Skip { .. }]));
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

        materialise_canonical(&e, dir.path()).unwrap();

        // Stale file gone; the embed's SKILL.md present and byte-equal.
        assert!(!id_dir.join("STALE.md").exists(), "stale file must be gone");
        let embed = PluginAssets::get("review/skills/code-review/SKILL.md").unwrap();
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

        materialise_canonical(&e, dir.path()).unwrap();

        // Temp cleared; canonical coherent (embed content, no junk leaked in).
        assert!(!tmp.exists(), "leftover temp must be cleared");
        assert!(!id_dir.join("JUNK.md").exists());
        let embed = PluginAssets::get("review/skills/code-review/SKILL.md").unwrap();
        assert_eq!(
            fs::read(id_dir.join("SKILL.md")).unwrap(),
            embed.data.as_ref()
        );
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
        let argv = delegate_argv("codex", &[&e], false, false);
        assert_eq!(
            argv,
            vec![
                "skills",
                "add",
                "doctrine/doctrine",
                "--agent",
                "codex",
                "--yes"
            ]
        );
    }

    #[test]
    fn delegate_argv_subset_and_global() {
        let e = entry("review", "code-review");
        let argv = delegate_argv("cursor", &[&e], true, true);
        assert_eq!(
            argv,
            vec![
                "skills",
                "add",
                "doctrine/doctrine",
                "--agent",
                "cursor",
                "--global",
                "--skill",
                "code-review",
                "--yes",
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
    fn resolve_agents_errors_without_target() {
        let dir = tempfile::tempdir().unwrap();
        assert!(resolve_agents(&[], dir.path()).is_err());
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
        )
        .unwrap();

        assert!(matches!(plan.items.first(), Some(AgentPlan::Claude(_))));
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

    #[test]
    fn execute_copies_claude_skill_files() {
        let dir = tempfile::tempdir().unwrap();
        let catalog = discover().unwrap();
        let plan = build_plan(dir.path(), &[Agent::Claude], &catalog, &[], &[], false).unwrap();

        let runner = FakeRunner {
            ok: true,
            ..FakeRunner::default()
        };
        let mut out = Vec::new();
        execute(&plan, &catalog, &runner, &mut out).unwrap();

        let installed = dir.path().join(".claude/skills/code-review/SKILL.md");
        assert!(installed.is_file());
        assert!(runner.calls.borrow().is_empty());
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
        )
        .unwrap();

        let runner = FakeRunner {
            ok: true,
            ..FakeRunner::default()
        };
        let mut out = Vec::new();
        execute(&plan, &catalog, &runner, &mut out).unwrap();

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
            &["claude".into()],
            &["code-review".into()],
            &[],
            false,
            false,
            true,
        )
        .unwrap();

        let gi = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert!(
            gi.contains(".doctrine/skills/*"),
            "skills install must self-enforce the derived-tree ignore"
        );
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
        )
        .unwrap();

        let runner = FakeRunner {
            ok: false,
            ..FakeRunner::default()
        };
        let mut out = Vec::new();
        assert!(execute(&plan, &catalog, &runner, &mut out).is_err());
    }
}
