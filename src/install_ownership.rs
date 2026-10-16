// SPDX-License-Identifier: GPL-3.0-only
//! `install_ownership` — the pure convergence core of `doctrine install`
//! (SL-281 design `sec-convergence`; DEC-374, DEC-377).
//!
//! From the declared framework roots, the [`Desired`] set and an on-disk
//! [`Snapshot`], [`plan`] decides every write, link, relink, unlink, prune and
//! report for framework-owned paths. The shell (`install.rs`) builds both
//! inputs and executes the actions, re-proving each before it mutates.
//!
//! A pure leaf (ADR-001): no clock, git, disk or env. Every path it is handed
//! is absolute and lexically normal; a link's value is the only path it
//! normalises itself.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::fsutil::{lexical_normalise, relative_path};

/// Name prefix of an in-flight staged write. An entry carrying it is never
/// removed — it may be a concurrent run's live staging — only reported.
pub(crate) const STAGING_PREFIX: &str = ".tmp-";

/// What a member's canon entry is — the executor bakes or copies by this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum MemberKind {
    AgentDef,
    Skill,
    Workflow,
}

/// One shipped canon entry and every registered link site that surfaces it.
/// A link is mapped to its member by site (dir + name), never by its value.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Member {
    pub(crate) kind: MemberKind,
    pub(crate) canon: PathBuf,
    pub(crate) links: BTreeSet<PathBuf>,
}

/// What is shipped across every registered harness (`members`), what this
/// run selected (`selected`), and the link sites it may newly create
/// (`new_links` — selected harnesses only).
#[derive(Debug, Default)]
pub(crate) struct Desired {
    pub(crate) members: BTreeSet<Member>,
    pub(crate) selected: BTreeSet<Member>,
    pub(crate) new_links: BTreeSet<PathBuf>,
}

/// What the shell observed: canon-root entries (at member depth, plus the
/// containers of members), every entry of every registered link dir, and the
/// directories reached through a symlinked ancestor (no removals there).
#[derive(Debug, Default)]
pub(crate) struct Snapshot {
    pub(crate) canon: Vec<PathBuf>,
    pub(crate) links: Vec<LinkSeen>,
    pub(crate) prune_blocked: BTreeSet<PathBuf>,
}

/// One entry of a link dir, as `lstat` saw it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LinkSeen {
    Symlink {
        dir: PathBuf,
        name: OsString,
        value: PathBuf,
    },
    Real {
        dir: PathBuf,
        name: OsString,
    },
}

/// A project install converges canon roots; a global one only reports
/// orphans (DEC-377).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Scope {
    Project,
    Global,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Action {
    WriteCanon(Member),
    Link { dest: PathBuf, target: PathBuf },
    Relink { dest: PathBuf, target: PathBuf },
    Unlink(PathBuf),
    PruneCanon(PathBuf),
    Report(Notice),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Notice {
    Orphan(PathBuf),
    ForeignSymlink(PathBuf),
    Shadows(PathBuf),
    PruneBlocked(PathBuf),
    StaleStaging(PathBuf),
}

/// Decide every action for framework-owned paths (design `sec-convergence`).
///
/// Order is fixed: canon writes, link writes, unlinks, canon prunes, then
/// reports — so a link is repaired before its old target is pruned, and a
/// new link never dangles. Each group is sorted, so the plan is deterministic.
pub(crate) fn plan(
    roots: &[PathBuf],
    scope: Scope,
    want: &Desired,
    seen: &Snapshot,
) -> Vec<Action> {
    let sites = link_sites(want);
    let mut out = Plan {
        writes: want.selected.iter().collect(),
        ..Plan::default()
    };
    for entry in &seen.links {
        out.observe_link(roots, &sites, entry);
    }
    out.create_links(want, seen);
    out.observe_canon(want, &seen.canon);
    out.into_actions(scope, &seen.prune_blocked)
}

/// Every registered link site, mapped to the member it surfaces.
fn link_sites(want: &Desired) -> BTreeMap<&Path, &Member> {
    want.members
        .iter()
        .flat_map(|m| m.links.iter().map(move |site| (site.as_path(), m)))
        .collect()
}

/// The relative value a link at `dest` takes to reach `member`'s canon entry.
fn link_target(dest: &Path, member: &Member) -> PathBuf {
    relative_path(dest.parent().unwrap_or(dest), &member.canon)
}

/// A doctrine link: its value, joined to its dir if relative and folded
/// lexically (a dangling old-layout link still counts), lies strictly under a
/// framework root, compared component-wise.
fn is_doctrine_link(roots: &[PathBuf], dir: &Path, value: &Path) -> bool {
    let resolved = lexical_normalise(&dir.join(value));
    roots
        .iter()
        .any(|root| resolved != *root && resolved.starts_with(root))
}

fn is_staging(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|n| n.to_string_lossy().starts_with(STAGING_PREFIX))
}

/// The plan under construction, one bucket per action group. Removals are
/// held as candidates until scope and the prune guard decide their fate.
#[derive(Default)]
struct Plan<'a> {
    writes: BTreeSet<&'a Member>,
    links: BTreeMap<PathBuf, Action>,
    unlinks: BTreeSet<PathBuf>,
    prunes: BTreeSet<PathBuf>,
    notices: BTreeSet<Notice>,
}

impl<'a> Plan<'a> {
    /// Classify one link-dir entry (design § Links flowchart).
    fn observe_link(
        &mut self,
        roots: &[PathBuf],
        sites: &BTreeMap<&Path, &'a Member>,
        entry: &LinkSeen,
    ) {
        let (dir, name, value) = match entry {
            LinkSeen::Symlink { dir, name, value } => (dir, name, Some(value)),
            LinkSeen::Real { dir, name } => (dir, name, None),
        };
        let dest = dir.join(name);
        let site = sites.get(dest.as_path()).copied();
        if is_staging(&dest) {
            self.notices.insert(Notice::StaleStaging(dest));
            return;
        }
        let Some(value) = value else {
            if site.is_some() {
                self.notices.insert(Notice::Shadows(dest));
            }
            return;
        };
        match (is_doctrine_link(roots, dir, value), site) {
            (true, Some(member)) => {
                self.writes.insert(member);
                let target = link_target(&dest, member);
                if *value != target {
                    self.links
                        .insert(dest.clone(), Action::Relink { dest, target });
                }
            }
            (true, None) => {
                self.unlinks.insert(dest);
            }
            (false, Some(_)) => {
                self.notices.insert(Notice::ForeignSymlink(dest));
            }
            (false, None) => {}
        }
    }

    /// Create the selected harnesses' links where nothing occupies the site.
    fn create_links(&mut self, want: &'a Desired, seen: &Snapshot) {
        let occupied: BTreeSet<PathBuf> = seen
            .links
            .iter()
            .map(|entry| match entry {
                LinkSeen::Symlink { dir, name, .. } | LinkSeen::Real { dir, name } => {
                    dir.join(name)
                }
            })
            .collect();
        for member in &want.members {
            for dest in &member.links {
                if want.new_links.contains(dest) && !occupied.contains(dest) {
                    self.writes.insert(member);
                    let target = link_target(dest, member);
                    self.links.insert(
                        dest.clone(),
                        Action::Link {
                            dest: dest.clone(),
                            target,
                        },
                    );
                }
            }
        }
    }

    /// Classify canon-root entries: a member, its container or its contents
    /// stay;
    /// staging is reported; anything else is a removal candidate, once —
    /// never again for an entry beneath one already taken.
    fn observe_canon(&mut self, want: &Desired, canon: &[PathBuf]) {
        let mut entries: Vec<&PathBuf> = canon.iter().collect();
        entries.sort();
        for entry in entries {
            if is_staging(entry) {
                self.notices.insert(Notice::StaleStaging(entry.clone()));
                continue;
            }
            let owned = want
                .members
                .iter()
                .any(|m| m.canon.starts_with(entry) || entry.starts_with(&m.canon));
            let beneath_prune = self.prunes.iter().any(|p| entry.starts_with(p));
            if !owned && !beneath_prune {
                self.prunes.insert(entry.clone());
            }
        }
    }

    /// Apply scope and the prune guard to the removal candidates, then
    /// flatten into the fixed action order.
    fn into_actions(self, scope: Scope, blocked: &BTreeSet<PathBuf>) -> Vec<Action> {
        let Plan {
            writes,
            links,
            unlinks,
            prunes,
            mut notices,
        } = self;
        let unlinks = admit(scope, blocked, unlinks, &mut notices);
        let prunes = admit(scope, blocked, prunes, &mut notices);
        writes
            .into_iter()
            .cloned()
            .map(Action::WriteCanon)
            .chain(links.into_values())
            .chain(unlinks.into_iter().map(Action::Unlink))
            .chain(prunes.into_iter().map(Action::PruneCanon))
            .chain(notices.into_iter().map(Action::Report))
            .collect()
    }
}

/// The removals that may proceed. A global install removes nothing and
/// reports each as an orphan; a removal under a blocked dir is withheld and
/// that dir reported once.
fn admit(
    scope: Scope,
    blocked: &BTreeSet<PathBuf>,
    candidates: BTreeSet<PathBuf>,
    notices: &mut BTreeSet<Notice>,
) -> BTreeSet<PathBuf> {
    let mut admitted = BTreeSet::new();
    for path in candidates {
        if scope == Scope::Global {
            notices.insert(Notice::Orphan(path));
        } else if let Some(dir) = blocked.iter().find(|d| path.starts_with(d)) {
            notices.insert(Notice::PruneBlocked(dir.clone()));
        } else {
            admitted.insert(path);
        }
    }
    admitted
}

// ── managed gitignore block (SL-281 design `sec-gitignore`) ──

/// The managed block's opening line. Detection uses [`BLOCK_START_PREFIX`],
/// so a later wording change still finds blocks written under this one.
pub(crate) const BLOCK_START: &str =
    "# >>> doctrine managed — rewritten by `doctrine install`; edit outside this block";
pub(crate) const BLOCK_END: &str = "# <<< doctrine managed";
const BLOCK_START_PREFIX: &str = "# >>> doctrine managed";
const BLOCK_END_PREFIX: &str = "# <<< doctrine managed";

/// Doctrine-owned link sites in `seen`: symlinks whose value lies strictly
/// under a framework root (see [`is_doctrine_link`]). Sorted, deduplicated.
pub(crate) fn doctrine_links(roots: &[PathBuf], seen: &[LinkSeen]) -> Vec<PathBuf> {
    seen.iter()
        .filter_map(|entry| match entry {
            LinkSeen::Symlink { dir, name, value } if is_doctrine_link(roots, dir, value) => {
                Some(dir.join(name))
            }
            _ => None,
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// The link sites that exist once `actions` have run: `links` (on disk now)
/// plus every `Link`/`Relink` destination, minus every `Unlink`. The dry-run
/// block uses it to predict what a live run's re-scan will find.
pub(crate) fn links_after(links: Vec<PathBuf>, actions: &[Action]) -> Vec<PathBuf> {
    let mut sites: BTreeSet<PathBuf> = links.into_iter().collect();
    for action in actions {
        match action {
            Action::Link { dest, .. } | Action::Relink { dest, .. } => {
                sites.insert(dest.clone());
            }
            Action::Unlink(path) => {
                sites.remove(path);
            }
            _ => {}
        }
    }
    sites.into_iter().collect()
}

/// The block, markers included: runtime `entries`, then `<root>/*` per root,
/// then one line per link (already `/`-anchored, project-relative; sorted and
/// deduplicated here so the output is byte-stable). Only the markers carry
/// `#` — gitignore has no inline comments.
pub(crate) fn render_block(entries: &[String], roots: &[String], links: &[String]) -> Vec<String> {
    let links: BTreeSet<&String> = links.iter().collect();
    std::iter::once(BLOCK_START.to_owned())
        .chain(entries.iter().cloned())
        .chain(roots.iter().map(|root| format!("{root}/*")))
        .chain(links.into_iter().cloned())
        .chain(std::iter::once(BLOCK_END.to_owned()))
        .collect()
}

/// Why [`splice_block`] left a file alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SpliceRefusal {
    /// Not exactly one start line followed by one end line, nor none of either.
    Malformed { starts: usize, ends: usize },
}

/// Replace the managed block in `existing` with `block`, or append it when
/// the file has none. Bytes outside the block are untouched, so a user's
/// later negation stays after it. The line ending is CRLF iff the first line
/// ends `\r\n`; every emitted block line carries it.
pub(crate) fn splice_block(existing: &str, block: &[String]) -> Result<String, SpliceRefusal> {
    let eol = if existing
        .split_inclusive('\n')
        .next()
        .is_some_and(|l| l.ends_with("\r\n"))
    {
        "\r\n"
    } else {
        "\n"
    };
    let lines: Vec<&str> = existing.split_inclusive('\n').collect();
    let marked = |prefix: &str| -> Vec<usize> {
        lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.trim_end_matches(['\n', '\r']).starts_with(prefix))
            .map(|(i, _)| i)
            .collect()
    };
    let (starts, ends) = (marked(BLOCK_START_PREFIX), marked(BLOCK_END_PREFIX));
    let rendered: String = block.iter().flat_map(|l| [l.as_str(), eol]).collect();
    match (starts.as_slice(), ends.as_slice()) {
        ([], []) => {
            let gap = if existing.is_empty() || existing.ends_with('\n') {
                ""
            } else {
                eol
            };
            Ok(format!("{existing}{gap}{rendered}"))
        }
        ([start], [end]) if start < end => {
            let before: String = lines.iter().take(*start).copied().collect();
            let after: String = lines.iter().skip(end + 1).copied().collect();
            Ok(format!("{before}{rendered}{after}"))
        }
        _ => Err(SpliceRefusal::Malformed {
            starts: starts.len(),
            ends: ends.len(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "/p";
    const ROOTS: [&str; 3] = [
        ".doctrine/agents",
        ".doctrine/skills",
        ".doctrine/workflows",
    ];

    fn p(rel: &str) -> PathBuf {
        Path::new(BASE).join(rel)
    }

    fn member(kind: MemberKind, canon: &str, link: &str) -> Member {
        Member {
            kind,
            canon: p(canon),
            links: BTreeSet::from([p(link)]),
        }
    }

    fn agent(harness: &str, def: &str) -> Member {
        member(
            MemberKind::AgentDef,
            &format!(".doctrine/agents/{harness}/{def}"),
            &format!(".{harness}/agents/{def}"),
        )
    }

    fn skill(id: &str) -> Member {
        member(
            MemberKind::Skill,
            &format!(".doctrine/skills/{id}"),
            &format!(".claude/skills/{id}"),
        )
    }

    fn workflow(file: &str) -> Member {
        member(
            MemberKind::Workflow,
            &format!(".doctrine/workflows/{file}"),
            &format!(".claude/workflows/{file}"),
        )
    }

    /// A desired set and a snapshot under `/p`, planted by relative path.
    #[derive(Default)]
    struct World {
        want: Desired,
        seen: Snapshot,
    }

    impl World {
        /// Shipped but not selected this run: a member, no new links.
        fn shipped(mut self, m: Member) -> Self {
            self.want.members.insert(m);
            self
        }

        /// Selected this run: a member whose sites are all new-link candidates.
        fn selected(mut self, m: Member) -> Self {
            self.want.new_links.extend(m.links.iter().cloned());
            self.want.selected.insert(m.clone());
            self.want.members.insert(m);
            self
        }

        fn canon(mut self, rel: &str) -> Self {
            self.seen.canon.push(p(rel));
            self
        }

        fn symlink(mut self, rel: &str, value: &str) -> Self {
            let dest = p(rel);
            self.seen.links.push(LinkSeen::Symlink {
                dir: dest.parent().unwrap().to_path_buf(),
                name: dest.file_name().unwrap().to_owned(),
                value: PathBuf::from(value),
            });
            self
        }

        fn real(mut self, rel: &str) -> Self {
            let dest = p(rel);
            self.seen.links.push(LinkSeen::Real {
                dir: dest.parent().unwrap().to_path_buf(),
                name: dest.file_name().unwrap().to_owned(),
            });
            self
        }

        fn blocked(mut self, rel: &str) -> Self {
            self.seen.prune_blocked.insert(p(rel));
            self
        }

        fn plan(&self, scope: Scope) -> Vec<Action> {
            let roots: Vec<_> = ROOTS.iter().map(|r| p(r)).collect();
            plan(&roots, scope, &self.want, &self.seen)
        }

        fn project(&self) -> Vec<Action> {
            self.plan(Scope::Project)
        }
    }

    fn writes(actions: &[Action]) -> Vec<PathBuf> {
        actions
            .iter()
            .filter_map(|a| match a {
                Action::WriteCanon(m) => Some(m.canon.clone()),
                _ => None,
            })
            .collect()
    }

    /// `(dest, target)` of every Link and Relink, tagged by which.
    fn links(actions: &[Action]) -> Vec<(&'static str, PathBuf, PathBuf)> {
        actions
            .iter()
            .filter_map(|a| match a {
                Action::Link { dest, target } => Some(("link", dest.clone(), target.clone())),
                Action::Relink { dest, target } => Some(("relink", dest.clone(), target.clone())),
                _ => None,
            })
            .collect()
    }

    fn removals(actions: &[Action]) -> Vec<Action> {
        actions
            .iter()
            .filter(|a| matches!(a, Action::Unlink(_) | Action::PruneCanon(_)))
            .cloned()
            .collect()
    }

    fn notices(actions: &[Action]) -> Vec<Notice> {
        actions
            .iter()
            .filter_map(|a| match a {
                Action::Report(n) => Some(n.clone()),
                _ => None,
            })
            .collect()
    }

    fn position(actions: &[Action], want: &Action) -> usize {
        actions
            .iter()
            .position(|a| a == want)
            .unwrap_or_else(|| panic!("{want:?} not planned in {actions:#?}"))
    }

    #[test]
    fn plan_writes_every_member_of_selected_set() {
        let w = World::default()
            .selected(agent("claude", "dispatch-worker.md"))
            .selected(skill("code-review"))
            .selected(workflow("ship.md"));
        let actions = w.project();
        assert_eq!(
            writes(&actions),
            [
                p(".doctrine/agents/claude/dispatch-worker.md"),
                p(".doctrine/skills/code-review"),
                p(".doctrine/workflows/ship.md"),
            ]
        );
        assert_eq!(
            links(&actions),
            [
                (
                    "link",
                    p(".claude/agents/dispatch-worker.md"),
                    PathBuf::from("../../.doctrine/agents/claude/dispatch-worker.md")
                ),
                (
                    "link",
                    p(".claude/skills/code-review"),
                    PathBuf::from("../../.doctrine/skills/code-review")
                ),
                (
                    "link",
                    p(".claude/workflows/ship.md"),
                    PathBuf::from("../../.doctrine/workflows/ship.md")
                ),
            ]
        );
        assert!(removals(&actions).is_empty());
        assert!(notices(&actions).is_empty());
    }

    #[test]
    fn plan_prunes_canon_entry_not_in_catalog() {
        let w = World::default()
            .selected(skill("a"))
            .canon(".doctrine/skills/a")
            .canon(".doctrine/skills/retired");
        assert_eq!(
            removals(&w.project()),
            [Action::PruneCanon(p(".doctrine/skills/retired"))]
        );
    }

    #[test]
    fn plan_keeps_unselected_member() {
        let w = World::default()
            .selected(skill("a"))
            .shipped(skill("b"))
            .canon(".doctrine/skills/b");
        let actions = w.project();
        assert_eq!(writes(&actions), [p(".doctrine/skills/a")]);
        assert!(removals(&actions).is_empty());
    }

    #[test]
    fn plan_relinks_stale_layout_symlink_into_framework_root() {
        // The ISS-216 case: pi's def linked into a dangling old `codex/` layout.
        let w = World::default()
            .selected(agent("pi", "dispatch-worker.md"))
            .symlink(
                ".pi/agents/dispatch-worker.md",
                "../../.doctrine/agents/codex/dispatch-worker.md",
            )
            .canon(".doctrine/agents/codex")
            .canon(".doctrine/agents/codex/dispatch-worker.md");
        let actions = w.project();
        assert_eq!(
            links(&actions),
            [(
                "relink",
                p(".pi/agents/dispatch-worker.md"),
                PathBuf::from("../../.doctrine/agents/pi/dispatch-worker.md")
            )]
        );
        assert_eq!(
            writes(&actions),
            [p(".doctrine/agents/pi/dispatch-worker.md")]
        );
        // The old layout dir goes as one prune, not again per child.
        assert_eq!(
            removals(&actions),
            [Action::PruneCanon(p(".doctrine/agents/codex"))]
        );
    }

    #[test]
    fn plan_unlinks_doctrine_symlink_whose_member_is_gone() {
        let w = World::default()
            .selected(skill("a"))
            .symlink(".claude/skills/a", "../../.doctrine/skills/a")
            .symlink(".claude/skills/retired", "../../.doctrine/skills/retired");
        let actions = w.project();
        assert_eq!(
            removals(&actions),
            [Action::Unlink(p(".claude/skills/retired"))]
        );
        // A link already at its current target needs no action.
        assert!(links(&actions).is_empty());
    }

    #[test]
    fn plan_keeps_regular_file_and_flags_shadowing() {
        let w = World::default()
            .selected(agent("claude", "dispatch-worker.md"))
            .real(".claude/agents/dispatch-worker.md")
            .real(".claude/agents/mine.md");
        let actions = w.project();
        assert!(links(&actions).is_empty());
        assert!(removals(&actions).is_empty());
        assert_eq!(
            notices(&actions),
            [Notice::Shadows(p(".claude/agents/dispatch-worker.md"))]
        );
        assert_eq!(
            writes(&actions),
            [p(".doctrine/agents/claude/dispatch-worker.md")]
        );
    }

    #[test]
    fn plan_keeps_foreign_symlink() {
        // Reported only at a member's link site (SL-281 OQ-1); a user's own link
        // elsewhere in the dir is silently theirs.
        let w = World::default()
            .selected(skill("a"))
            .symlink(".claude/skills/a", "/elsewhere/a")
            .symlink(".claude/skills/mine", "../../dotfiles/mine");
        let actions = w.project();
        assert!(links(&actions).is_empty());
        assert!(removals(&actions).is_empty());
        assert_eq!(
            notices(&actions),
            [Notice::ForeignSymlink(p(".claude/skills/a"))]
        );
    }

    #[test]
    fn plan_repairs_unselected_harness_link_before_pruning_its_old_target() {
        // RV-418 F-2: a pi-only run over a flat-layout claude install.
        let claude = agent("claude", "dispatch-worker.md");
        let w = World::default()
            .selected(agent("pi", "dispatch-worker.md"))
            .shipped(claude.clone())
            .symlink(
                ".claude/agents/dispatch-worker.md",
                "../../.doctrine/agents/dispatch-worker.md",
            )
            .canon(".doctrine/agents/dispatch-worker.md");
        let actions = w.project();
        let write = position(&actions, &Action::WriteCanon(claude));
        let relink = position(
            &actions,
            &Action::Relink {
                dest: p(".claude/agents/dispatch-worker.md"),
                target: PathBuf::from("../../.doctrine/agents/claude/dispatch-worker.md"),
            },
        );
        let prune = position(
            &actions,
            &Action::PruneCanon(p(".doctrine/agents/dispatch-worker.md")),
        );
        assert!(write < relink && relink < prune, "{actions:#?}");
        // The unselected harness gets its link repaired, not a new one elsewhere.
        assert_eq!(links(&actions).len(), 2);
    }

    #[test]
    fn plan_treats_absolute_link_under_root_as_doctrine_and_elsewhere_as_foreign() {
        let w = World::default()
            .selected(skill("a"))
            .selected(skill("b"))
            // A sibling dir sharing the root's string prefix is not under it.
            .symlink(".claude/skills/a", "/p/.doctrine/skills-old/a")
            .symlink(".claude/skills/b", "/p/.doctrine/skills/b")
            .symlink(".claude/skills/gone", "/p/.doctrine/skills/gone");
        let actions = w.project();
        assert_eq!(
            links(&actions),
            [(
                "relink",
                p(".claude/skills/b"),
                PathBuf::from("../../.doctrine/skills/b")
            )]
        );
        assert_eq!(
            removals(&actions),
            [Action::Unlink(p(".claude/skills/gone"))]
        );
        assert_eq!(
            notices(&actions),
            [Notice::ForeignSymlink(p(".claude/skills/a"))]
        );
    }

    #[test]
    fn plan_normalises_dotdot_link_values_lexically() {
        let w = World::default()
            .selected(skill("a"))
            .symlink(
                ".claude/skills/gone",
                "../../.claude/../.doctrine/skills/./gone",
            )
            .symlink(".claude/skills/a", "../../.doctrine/skills/../../outside/a");
        let actions = w.project();
        assert_eq!(
            removals(&actions),
            [Action::Unlink(p(".claude/skills/gone"))]
        );
        assert_eq!(
            notices(&actions),
            [Notice::ForeignSymlink(p(".claude/skills/a"))]
        );
    }

    #[test]
    fn plan_never_prunes_staging_entries_and_reports_them() {
        let w = World::default()
            .selected(skill("a"))
            .canon(".doctrine/skills/.tmp-a.41.0")
            .symlink(".claude/skills/.tmp-a", "../../.doctrine/skills/.tmp-a");
        let actions = w.project();
        assert!(removals(&actions).is_empty());
        assert_eq!(
            notices(&actions),
            [
                Notice::StaleStaging(p(".claude/skills/.tmp-a")),
                Notice::StaleStaging(p(".doctrine/skills/.tmp-a.41.0")),
            ]
        );
    }

    #[test]
    fn plan_blocks_removals_under_symlinked_dir() {
        let w = World::default()
            .selected(skill("a"))
            .canon(".doctrine/skills/a")
            .canon(".doctrine/skills/retired")
            .canon(".doctrine/skills/older")
            .symlink(".claude/skills/retired", "../../.doctrine/skills/retired")
            .blocked(".doctrine/skills");
        let actions = w.project();
        // Writes still follow the relocation, by member name only.
        assert_eq!(writes(&actions), [p(".doctrine/skills/a")]);
        // The unblocked link dir still converges.
        assert_eq!(
            removals(&actions),
            [Action::Unlink(p(".claude/skills/retired"))]
        );
        assert_eq!(
            notices(&actions),
            [Notice::PruneBlocked(p(".doctrine/skills"))]
        );
    }

    #[test]
    fn plan_global_scope_reports_orphans_and_prunes_nothing() {
        let w = World::default()
            .selected(skill("a"))
            .symlink(".claude/skills/a", "../../.doctrine/skills-v0/a")
            .symlink(".claude/skills/b", "../../.doctrine/agents/b")
            .symlink(".claude/skills/retired", "../../.doctrine/skills/retired")
            .canon(".doctrine/skills/retired");
        let actions = w.plan(Scope::Global);
        assert!(removals(&actions).is_empty());
        assert_eq!(writes(&actions), [p(".doctrine/skills/a")]);
        assert_eq!(
            notices(&actions),
            [
                Notice::Orphan(p(".claude/skills/b")),
                Notice::Orphan(p(".claude/skills/retired")),
                Notice::Orphan(p(".doctrine/skills/retired")),
                Notice::ForeignSymlink(p(".claude/skills/a")),
            ]
        );
    }

    #[test]
    fn plan_prunes_workflow_not_shipped() {
        let w = World::default()
            .shipped(workflow("ship.md"))
            .canon(".doctrine/workflows/ship.md")
            .canon(".doctrine/workflows/retired.md");
        assert_eq!(
            removals(&w.project()),
            [Action::PruneCanon(p(".doctrine/workflows/retired.md"))]
        );
    }

    #[test]
    fn plan_orders_canon_writes_before_prunes() {
        let w = World::default()
            .selected(skill("z"))
            .selected(agent("claude", "a.md"))
            .canon(".doctrine/skills/old")
            .canon(".doctrine/agents/old.md")
            .symlink(".claude/skills/old", "../../.doctrine/skills/old")
            .symlink(".claude/agents/a.md", "../../.doctrine/agents/a.md")
            .real(".claude/skills/z");
        let actions = w.project();
        let rank = |a: &Action| match a {
            Action::WriteCanon(_) => 0,
            Action::Link { .. } | Action::Relink { .. } => 1,
            Action::Unlink(_) => 2,
            Action::PruneCanon(_) => 3,
            Action::Report(_) => 4,
        };
        let ranks: Vec<_> = actions.iter().map(rank).collect();
        assert!(ranks.is_sorted(), "{actions:#?}");
        assert_eq!(
            ranks.iter().collect::<BTreeSet<_>>().len(),
            5,
            "{actions:#?}"
        );
    }

    // ── managed gitignore block (SL-281 PHASE-05) ──

    fn block(lines: &[&str]) -> Vec<String> {
        lines.iter().map(|l| (*l).to_owned()).collect()
    }

    /// The rendered block for a fixed, small input.
    fn sample_block() -> Vec<String> {
        render_block(
            &block(&[".doctrine/state/"]),
            &block(&[".doctrine/skills"]),
            &block(&["/.claude/skills/a"]),
        )
    }

    fn splice(existing: &str) -> String {
        splice_block(existing, &sample_block()).expect("splices")
    }

    fn joined(lines: &[String], eol: &str) -> String {
        lines.iter().map(|l| format!("{l}{eol}")).collect()
    }

    fn marker_counts(text: &str) -> (usize, usize) {
        let count = |prefix| text.lines().filter(|l| l.starts_with(prefix)).count();
        (count(BLOCK_START_PREFIX), count(BLOCK_END_PREFIX))
    }

    #[test]
    fn splice_block_appends_when_absent() {
        let out = splice("target/\n");
        assert_eq!(out, format!("target/\n{}", joined(&sample_block(), "\n")));
        assert_eq!(splice(""), joined(&sample_block(), "\n"));
    }

    #[test]
    fn splice_block_replaces_between_markers() {
        let old = block(&[BLOCK_START, "/stale", BLOCK_END]);
        let existing = format!("before\n{}after\n", joined(&old, "\n"));
        let out = splice(&existing);
        assert_eq!(
            out,
            format!("before\n{}after\n", joined(&sample_block(), "\n"))
        );
    }

    #[test]
    fn splice_block_finds_a_block_with_older_marker_wording() {
        let existing = "# >>> doctrine managed (old words)\n/stale\n# <<< doctrine managed\nuser\n";
        let out = splice(existing);
        assert_eq!(out, format!("{}user\n", joined(&sample_block(), "\n")));
    }

    #[test]
    fn splice_block_preserves_user_negations_after_block() {
        let existing = format!(
            "mine\n{}!/.claude/skills/a\n",
            joined(&sample_block(), "\n")
        );
        let out = splice(&existing);
        assert_eq!(out, existing);
        assert!(out.ends_with("!/.claude/skills/a\n"));
    }

    #[test]
    fn splice_block_refuses_lone_or_duplicate_markers() {
        let (s, e) = (BLOCK_START, BLOCK_END);
        let cases = [
            (format!("{s}\nx\n"), (1, 0)),
            (format!("x\n{e}\n"), (0, 1)),
            (format!("{s}\n{e}\n{s}\n{e}\n"), (2, 2)),
            (format!("{s}\n{s}\n{e}\n"), (2, 1)),
            (format!("{e}\n{s}\n"), (1, 1)),
        ];
        for (existing, (starts, ends)) in cases {
            assert_eq!(
                splice_block(&existing, &sample_block()),
                Err(SpliceRefusal::Malformed { starts, ends }),
                "{existing:?}"
            );
        }
    }

    #[test]
    fn splice_block_adds_newline_to_unterminated_file() {
        let out = splice("target/");
        assert_eq!(out, format!("target/\n{}", joined(&sample_block(), "\n")));
        // An unterminated end marker gains its newline, then is stable.
        let unterminated = joined(&sample_block(), "\n").trim_end().to_owned();
        let once = splice(&unterminated);
        assert_eq!(once, joined(&sample_block(), "\n"));
        assert_eq!(splice(&once), once);
    }

    #[test]
    fn splice_block_keeps_crlf() {
        let appended = splice("target/\r\n");
        assert_eq!(
            appended,
            format!("target/\r\n{}", joined(&sample_block(), "\r\n"))
        );
        let existing = format!(
            "a\r\n{}b\r\n",
            joined(&block(&[BLOCK_START, BLOCK_END]), "\r\n")
        );
        let out = splice(&existing);
        assert_eq!(
            out,
            format!("a\r\n{}b\r\n", joined(&sample_block(), "\r\n"))
        );
        assert_eq!(splice(&out), out);
    }

    #[test]
    fn splice_block_is_idempotent() {
        for existing in [
            "",
            "x",
            "x\n",
            "x\r\ny\r\n",
            "a\n# >>> doctrine managed\n# <<< doctrine managed",
        ] {
            let once = splice(existing);
            assert_eq!(splice(&once), once, "{existing:?}");
            assert_eq!(marker_counts(&once), (1, 1));
        }
    }

    #[test]
    fn render_block_lists_runtime_entries_roots_and_links() {
        let out = render_block(
            &block(&[".doctrine/state/", ".doctrine/memory/shipped/"]),
            &block(&[".doctrine/agents", ".doctrine/skills"]),
            &block(&["/.pi/agents/b.md", "/.claude/skills/a", "/.pi/agents/b.md"]),
        );
        assert_eq!(
            out,
            block(&[
                BLOCK_START,
                ".doctrine/state/",
                ".doctrine/memory/shipped/",
                ".doctrine/agents/*",
                ".doctrine/skills/*",
                "/.claude/skills/a",
                "/.pi/agents/b.md",
                BLOCK_END,
            ])
        );
        let inline = out[1..out.len() - 1].iter().any(|l| l.contains('#'));
        assert!(!inline, "only the markers carry `#`");
    }

    /// The scan of a project: (a) a doctrine link, (b) an npx-style link
    /// elsewhere, (c) a real entry.
    fn scanned() -> (Vec<PathBuf>, Vec<LinkSeen>) {
        let sym = |dir: &str, name: &str, value: &str| LinkSeen::Symlink {
            dir: p(dir),
            name: name.into(),
            value: value.into(),
        };
        let seen = vec![
            sym(".claude/skills", "a", "../../.doctrine/skills/a"),
            sym(".agents/skills", "x", "../../../elsewhere/x"),
            LinkSeen::Real {
                dir: p(".claude/skills"),
                name: "mine".into(),
            },
        ];
        (ROOTS.iter().map(|r| p(r)).collect(), seen)
    }

    fn rendered_links(roots: &[PathBuf], seen: &[LinkSeen]) -> Vec<String> {
        let links = doctrine_links(roots, seen)
            .iter()
            .map(|l| format!("/{}", l.strip_prefix(BASE).unwrap().display()))
            .collect::<Vec<_>>();
        render_block(&[], &[], &links)
    }

    #[test]
    fn links_after_applies_the_plan_to_the_links_on_disk() {
        let actions = [
            Action::Link {
                dest: p(".claude/skills/new"),
                target: "t".into(),
            },
            Action::Relink {
                dest: p(".claude/skills/a"),
                target: "t".into(),
            },
            Action::Unlink(p(".claude/skills/gone")),
            Action::PruneCanon(p(".doctrine/skills/gone")),
        ];
        let on_disk = vec![p(".claude/skills/a"), p(".claude/skills/gone")];
        assert_eq!(
            links_after(on_disk, &actions),
            vec![p(".claude/skills/a"), p(".claude/skills/new")]
        );
    }

    #[test]
    fn doctrine_links_returns_only_symlinks_into_a_root() {
        let (roots, seen) = scanned();
        assert_eq!(doctrine_links(&roots, &seen), vec![p(".claude/skills/a")]);
    }

    #[test]
    fn render_block_omits_npx_written_skill_copies() {
        let (roots, seen) = scanned();
        let out = rendered_links(&roots, &seen);
        assert!(out.contains(&"/.claude/skills/a".to_owned()));
        assert!(!out.iter().any(|l| l.contains(".agents/skills")), "{out:?}");
    }

    #[test]
    fn render_block_omits_user_owned_paths() {
        let (roots, seen) = scanned();
        let out = rendered_links(&roots, &seen);
        assert!(!out.iter().any(|l| l.contains("mine")), "{out:?}");
        assert_eq!(out.len(), 3, "markers + the one doctrine link: {out:?}");
    }
}
