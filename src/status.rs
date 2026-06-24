// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine status` — project-orientation dashboard (SL-086 IMP-093).
//!
//! Pure/impure split (ADR-001): [`assemble_status`] plus [`render_human`] /
//! [`render_json`] are pure — they receive all data as plain structs, no clock,
//! rng, git, or disk. [`run`] is the impure shell: finds the root, calls existing
//! scan functions, runs `boot --check` and `git log`, then hands the collected
//! data to the pure layer.

use std::collections::BTreeMap;
use std::io::{self, Write as _};
use std::path::PathBuf;

use serde::Serialize;

use crate::listing::Format;

// ---------------------------------------------------------------------------
// Pure data types
// ---------------------------------------------------------------------------

/// Slice-count summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct SliceCounts {
    pub(crate) active: usize,
    pub(crate) blocked: usize,
    pub(crate) total: usize,
}

/// One next-up advisory item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct NextItem {
    pub(crate) id: String,
    pub(crate) status: String,
    pub(crate) title: String,
}

/// One RFC title entry for the status list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RfcTitle {
    pub(crate) id: String,
    pub(crate) title: String,
}

/// RFC summary for the status dashboard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RfcSummary {
    pub(crate) open: usize,
    pub(crate) total: usize,
    pub(crate) open_titles: Vec<RfcTitle>,
}

/// The "Work" section — slices, backlog, next-up, rfcs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct WorkSection {
    pub(crate) slices: SliceCounts,
    pub(crate) backlog: BTreeMap<String, usize>,
    pub(crate) next_up: Vec<NextItem>,
    pub(crate) rfcs: RfcSummary,
}

/// One blocked item — slice or backlog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BlockedItem {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) blocked_by: Vec<String>,
}

/// Boot snapshot staleness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BootSection {
    pub(crate) staleness: String,
    pub(crate) age_seconds: u64,
    pub(crate) commit: String,
}

/// One recent commit line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CommitLine {
    pub(crate) hash: String,
    pub(crate) subject: String,
    pub(crate) relative_time: String,
}

/// The assembled status dashboard.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct Status {
    pub(crate) work: WorkSection,
    pub(crate) blocked_slices: Vec<BlockedItem>,
    pub(crate) blocked_backlog: Vec<BlockedItem>,
    pub(crate) boot: BootSection,
    pub(crate) recent_commits: Vec<CommitLine>,
}

// ---------------------------------------------------------------------------
// Pure: assembly
// ---------------------------------------------------------------------------

/// Assemble the status dashboard from pre-gathered data. Pure — no disk, clock,
/// git, or rng.
#[expect(
    clippy::too_many_arguments,
    reason = "pure assembly fans gathered data 1:1"
)]
pub(crate) fn assemble_status(
    slice_counts: SliceCounts,
    backlog_counts: BTreeMap<String, usize>,
    next_up: Vec<NextItem>,
    rfcs: RfcSummary,
    blocked_slices: Vec<BlockedItem>,
    blocked_backlog: Vec<BlockedItem>,
    boot: BootSection,
    recent_commits: Vec<CommitLine>,
) -> Status {
    Status {
        work: WorkSection {
            slices: slice_counts,
            backlog: backlog_counts,
            next_up,
            rfcs,
        },
        blocked_slices,
        blocked_backlog,
        boot,
        recent_commits,
    }
}

// ---------------------------------------------------------------------------
// Pure: human render
// ---------------------------------------------------------------------------

/// Whether the status describes a repo with NO active work — no active slices
/// and no open backlog items.
fn is_empty(status: &Status) -> bool {
    status.work.slices.active == 0 && status.work.backlog.values().sum::<usize>() == 0
}

/// Render the status dashboard as human-readable text (10–20 lines).
pub(crate) fn render_human(status: &Status) -> String {
    // Design §4 output shape. Empty state → single line.
    if is_empty(status) {
        return "No active work.\n".to_string();
    }

    let mut parts: Vec<String> = Vec::new();

    // Work section.
    parts.push("Work\n".to_string());

    // Slices.
    let blocked_suffix = if status.work.slices.blocked > 0 {
        format!(" ({} blocked)", status.work.slices.blocked)
    } else {
        String::new()
    };
    parts.push(format!(
        "  slices: {} active{blocked_suffix}, {} total\n",
        status.work.slices.active, status.work.slices.total
    ));

    // Backlog by kind.
    if !status.work.backlog.is_empty() {
        let kinds: Vec<String> = status
            .work
            .backlog
            .iter()
            .map(|(k, v)| format!("{v} {k}{}", if *v == 1 { "" } else { "s" }))
            .collect();
        parts.push(format!("  backlog: {}\n", kinds.join(", ")));
    }

    // RFCs.
    if status.work.rfcs.total > 0 {
        parts.push(format!(
            "  rfcs: {} open, {} total\n",
            status.work.rfcs.open, status.work.rfcs.total
        ));
        for t in &status.work.rfcs.open_titles {
            parts.push(format!("  {} {}\n", t.id, t.title));
        }
        let overflow = status
            .work
            .rfcs
            .open
            .saturating_sub(status.work.rfcs.open_titles.len());
        if overflow > 0 {
            parts.push(format!("  +{overflow} more\n"));
        }
    }

    // Next up.
    if !status.work.next_up.is_empty() {
        let items: Vec<String> = status
            .work
            .next_up
            .iter()
            .map(|n| format!("{} ({})", n.id, n.status))
            .collect();
        parts.push(format!("  next up: {}\n", items.join(", ")));
    }

    // Blocked slices.
    if !status.blocked_slices.is_empty() {
        parts.push("\nBlocked slices\n".to_string());
        for item in &status.blocked_slices {
            parts.push(format!(
                "  {} blocked by {} — {}\n",
                item.id,
                item.blocked_by.join(", "),
                item.title
            ));
        }
    }

    // Blocked backlog.
    if !status.blocked_backlog.is_empty() {
        parts.push("\nBlocked backlog\n".to_string());
        for item in &status.blocked_backlog {
            parts.push(format!(
                "  {} blocked by {} — {}\n",
                item.id,
                item.blocked_by.join(", "),
                item.title
            ));
        }
    }

    // Boot.
    parts.push(format!("\nBoot\n  boot.md {}", boot_line(&status.boot)));

    // Recent commits.
    if !status.recent_commits.is_empty() {
        parts.push("\nRecent commits\n".to_string());
        for c in &status.recent_commits {
            parts.push(format!(
                "  {} {} — {}\n",
                c.hash, c.subject, c.relative_time
            ));
        }
    }

    parts.push("\n".to_string());
    parts.concat()
}

/// The human-readable boot line.
fn boot_line(boot: &BootSection) -> String {
    match boot.staleness.as_str() {
        "fresh" => {
            let mins = boot.age_seconds.div_ceil(60);
            format!("fresh ({mins} min ago) from commit {}\n", boot.commit)
        }
        "stale" => format!("stale from commit {}\n", boot.commit),
        _missing => "missing\n".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Pure: JSON render
// ---------------------------------------------------------------------------

/// JSON envelope for status, matching the `{ kind: "status", ... }` shape.
#[derive(Serialize)]
struct StatusEnvelope<'a> {
    kind: &'static str,
    #[serde(flatten)]
    status: &'a Status,
}

/// Render the status dashboard as JSON.
pub(crate) fn render_json(status: &Status) -> serde_json::Result<String> {
    let envelope = StatusEnvelope {
        kind: "status",
        status,
    };
    serde_json::to_string_pretty(&envelope)
}

// ---------------------------------------------------------------------------
// Impure shell: run
// ---------------------------------------------------------------------------

/// `doctrine status [--json]` — gather data, assemble, render.
pub(crate) fn run(path: Option<PathBuf>, format: Format, json: bool) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let resolved_format = if json { Format::Json } else { format };

    // --- Gather slice counts ---
    let slice_metas =
        crate::meta::read_metas(&root.join(".doctrine/slice"), "slice", "SL").unwrap_or_default();
    let slice_total = slice_metas.len();
    let slice_active: Vec<&crate::meta::Meta> = slice_metas
        .iter()
        .filter(|m| !matches!(m.status.as_str(), "done" | "abandoned"))
        .collect();
    let slice_active_count = slice_active.len();

    // --- Gather backlog counts (open only) ---
    let backlog_items = crate::backlog::read_all(&root).unwrap_or_default();
    let open_items: Vec<&crate::backlog::BacklogItem> = backlog_items
        .iter()
        .filter(|i| !i.status.is_terminal())
        .collect();
    let mut backlog_counts: BTreeMap<String, usize> = BTreeMap::new();
    for item in &open_items {
        *backlog_counts
            .entry(item.kind.as_str().to_string())
            .or_insert(0) += 1;
    }

    // --- Gather RFCs ---
    let rfc_metas =
        crate::meta::read_metas(&root.join(".doctrine/rfc"), "rfc", "RFC").unwrap_or_default();
    let rfc_total = rfc_metas.len();
    let mut open_rfc_ids: Vec<u32> = rfc_metas
        .iter()
        .filter(|m| m.status == "open")
        .map(|m| m.id)
        .collect();
    open_rfc_ids.sort_unstable_by(|a, b| b.cmp(a)); // most-recent first (id desc)
    let rfc_open = open_rfc_ids.len();
    let rfc_open_titles: Vec<RfcTitle> = open_rfc_ids
        .iter()
        .take(10)
        .filter_map(|id| {
            rfc_metas.iter().find(|m| m.id == *id).map(|m| RfcTitle {
                id: format!("RFC-{:03}", m.id),
                title: m.title.clone(),
            })
        })
        .collect();
    let rfcs = RfcSummary {
        open: rfc_open,
        total: rfc_total,
        open_titles: rfc_open_titles,
    };

    // --- Next up (top 5) ---
    let next_rows = crate::priority::surface::next(&root).unwrap_or_default();
    let next_up: Vec<NextItem> = next_rows
        .iter()
        .take(5)
        .map(|r| NextItem {
            id: r.id.clone(),
            status: r.status.clone(),
            title: r.title.clone(),
        })
        .collect();

    // --- Blocked detection (via priority graph) ---
    let (blocked_slices, blocked_backlog, slice_blocked_count) =
        if let Ok(graph) = crate::priority::graph::build(&root) {
            // Blocked slices: active + has unresolved needs edges.
            let mut bs: Vec<BlockedItem> = Vec::new();
            for m in &slice_active {
                let key = crate::relation_graph::EntityKey {
                    prefix: "SL",
                    id: m.id,
                };
                if crate::priority::channels::blocked(&graph, key) {
                    let blockers = crate::priority::channels::blocked_by(&graph, key);
                    bs.push(BlockedItem {
                        id: key.canonical(),
                        title: m.title.clone(),
                        blocked_by: blockers.iter().map(|k| k.canonical()).collect(),
                    });
                }
            }
            bs.sort_by(|a, b| a.id.cmp(&b.id));
            let cb = bs.len();
            bs.truncate(5);

            // Blocked backlog: open + has unresolved needs edges.
            let mut bb: Vec<BlockedItem> = Vec::new();
            for item in &open_items {
                let key = crate::relation_graph::EntityKey {
                    prefix: item.kind.prefix(),
                    id: item.id,
                };
                if crate::priority::channels::blocked(&graph, key) {
                    let blockers = crate::priority::channels::blocked_by(&graph, key);
                    bb.push(BlockedItem {
                        id: key.canonical(),
                        title: item.title.clone(),
                        blocked_by: blockers.iter().map(|k| k.canonical()).collect(),
                    });
                }
            }
            bb.sort_by(|a, b| a.id.cmp(&b.id));
            bb.truncate(5);
            (bs, bb, cb)
        } else {
            (Vec::new(), Vec::new(), 0)
        };

    let slice_counts = SliceCounts {
        active: slice_active_count,
        blocked: slice_blocked_count,
        total: slice_total,
    };

    // --- Boot staleness ---
    let exec = crate::boot::resolve_exec().unwrap_or_else(|_| PathBuf::from("doctrine"));
    let report = crate::boot::boot_check(&root, &exec, crate::commands::cli::render_boot_map);
    let boot_path = root.join(".doctrine/state/boot.md");
    let (staleness, age_seconds, commit) = if boot_path.exists() {
        let staleness_str = if report.stale { "stale" } else { "fresh" }.to_string();
        let age = std::fs::metadata(&boot_path)
            .ok()
            .and_then(|md| md.modified().ok())
            .and_then(|mtime| {
                std::time::SystemTime::now()
                    .duration_since(mtime)
                    .ok()
                    .map(|d| d.as_secs())
            })
            .unwrap_or(0);
        // Get the commit that wrote boot.md via git log; fall back to HEAD
        // when boot.md is gitignored (no tracked history).
        let commit_sha = crate::git::git_text(
            &root,
            &["log", "-1", "--format=%h", "--", ".doctrine/state/boot.md"],
        )
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| crate::git::git_text(&root, &["log", "-1", "--format=%h"]).ok())
        .unwrap_or_default();
        (staleness_str, age, commit_sha)
    } else {
        ("missing".to_string(), 0_u64, String::new())
    };

    let boot = BootSection {
        staleness,
        age_seconds,
        commit,
    };

    // --- Recent commits ---
    let recent_commits = parse_git_log(&root);

    // --- Assemble + render ---
    let status = assemble_status(
        slice_counts,
        backlog_counts,
        next_up,
        rfcs,
        blocked_slices,
        blocked_backlog,
        boot,
        recent_commits,
    );

    let out = match resolved_format {
        Format::Table => render_human(&status),
        Format::Json => render_json(&status)
            .map_err(|e| anyhow::anyhow!("failed to serialize status JSON: {e}"))?,
    };
    write!(io::stdout(), "{out}")?;
    Ok(())
}

/// Parse `git log -5 --format="%h %s — %ar"` into structured commit lines.
/// On any failure (no git, non-repo, etc.) returns an empty vec — never crashes.
fn parse_git_log(root: &std::path::Path) -> Vec<CommitLine> {
    let Ok(text) = crate::git::git_text(root, &["log", "-5", "--format=%h %s — %ar"]) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|line| {
            let (hash, rest) = line.split_once(' ')?;
            let (subject, relative_time) = rest.rsplit_once(" — ")?;
            Some(CommitLine {
                hash: hash.to_string(),
                subject: subject.to_string(),
                relative_time: relative_time.to_string(),
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn counts(active: usize, blocked: usize, total: usize) -> SliceCounts {
        SliceCounts {
            active,
            blocked,
            total,
        }
    }

    fn empty_counts() -> SliceCounts {
        counts(0, 0, 0)
    }

    fn empty_backlog() -> BTreeMap<String, usize> {
        BTreeMap::new()
    }

    fn empty_next() -> Vec<NextItem> {
        Vec::new()
    }

    fn empty_blocked() -> Vec<BlockedItem> {
        Vec::new()
    }

    fn fresh_boot() -> BootSection {
        BootSection {
            staleness: "fresh".to_string(),
            age_seconds: 120,
            commit: "a3f7b2c".to_string(),
        }
    }

    fn no_commits() -> Vec<CommitLine> {
        Vec::new()
    }

    fn empty_rfcs() -> RfcSummary {
        RfcSummary {
            open: 0,
            total: 0,
            open_titles: Vec::new(),
        }
    }

    // --- VT-3: empty corpus ---

    #[test]
    fn empty_corpus_shows_no_active_work() {
        let status = assemble_status(
            empty_counts(),
            empty_backlog(),
            empty_next(),
            empty_rfcs(),
            empty_blocked(),
            empty_blocked(),
            fresh_boot(),
            no_commits(),
        );
        assert_eq!(render_human(&status), "No active work.\n");
    }

    #[test]
    fn empty_corpus_json_has_expected_keys() {
        let status = assemble_status(
            empty_counts(),
            empty_backlog(),
            empty_next(),
            empty_rfcs(),
            empty_blocked(),
            empty_blocked(),
            fresh_boot(),
            no_commits(),
        );
        let json = render_json(&status).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["kind"], "status");
        assert_eq!(parsed["work"]["slices"]["active"], 0);
        assert_eq!(parsed["work"]["slices"]["blocked"], 0);
        assert_eq!(parsed["work"]["slices"]["total"], 0);
    }

    // --- VT-1: non-empty corpus → all sections present ---

    #[test]
    fn non_empty_status_shows_all_sections() {
        let mut backlog = BTreeMap::new();
        backlog.insert("issue".to_string(), 3_usize);
        backlog.insert("improvement".to_string(), 1_usize);

        let next = vec![NextItem {
            id: "SL-086".to_string(),
            status: "design".to_string(),
            title: "CLI UX".to_string(),
        }];

        let blocked_slices = vec![BlockedItem {
            id: "SL-082".to_string(),
            title: "reconcile engine".to_string(),
            blocked_by: vec!["SL-047".to_string()],
        }];

        let commits = vec![CommitLine {
            hash: "a3f7b2c".to_string(),
            subject: "plan(SL-086): phase sheets".to_string(),
            relative_time: "2 min ago".to_string(),
        }];

        let status = assemble_status(
            counts(2, 1, 4),
            backlog,
            next,
            empty_rfcs(),
            blocked_slices,
            empty_blocked(),
            fresh_boot(),
            commits,
        );

        let output = render_human(&status);
        assert!(output.contains("Work\n"));
        assert!(output.contains("slices: 2 active (1 blocked), 4 total\n"));
        // BTreeMap sorts alphabetically: "improvement" before "issue"
        assert!(output.contains("backlog: 1 improvement, 3 issues\n"));
        assert!(output.contains("next up: SL-086 (design)\n"));
        assert!(output.contains("Blocked slices\n"));
        assert!(output.contains("SL-082 blocked by SL-047 — reconcile engine\n"));
        assert!(output.contains("Boot\n"));
        assert!(output.contains("fresh (2 min ago) from commit a3f7b2c\n"));
        assert!(output.contains("Recent commits\n"));
        assert!(output.contains("a3f7b2c plan(SL-086): phase sheets — 2 min ago\n"));
    }

    // --- VT-2: JSON output shape ---

    #[test]
    fn json_output_has_expected_shape() {
        let status = assemble_status(
            counts(2, 1, 4),
            empty_backlog(),
            empty_next(),
            empty_rfcs(),
            empty_blocked(),
            empty_blocked(),
            fresh_boot(),
            no_commits(),
        );

        let json = render_json(&status).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["kind"], "status");
        assert_eq!(v["work"]["slices"]["active"], 2);
        assert_eq!(v["work"]["slices"]["blocked"], 1);
        assert_eq!(v["work"]["slices"]["total"], 4);
        assert_eq!(v["boot"]["staleness"], "fresh");
        assert_eq!(v["boot"]["age_seconds"], 120);
        assert_eq!(v["boot"]["commit"], "a3f7b2c");
    }

    // --- Suppressed sections ---

    #[test]
    fn blocked_sections_suppressed_when_empty() {
        let status = assemble_status(
            counts(1, 0, 1),
            empty_backlog(),
            empty_next(),
            empty_rfcs(),
            empty_blocked(),
            empty_blocked(),
            fresh_boot(),
            no_commits(),
        );
        let output = render_human(&status);
        assert!(!output.contains("Blocked slices"));
        assert!(!output.contains("Blocked backlog"));
    }

    #[test]
    fn recent_commits_suppressed_when_empty() {
        let status = assemble_status(
            counts(1, 0, 1),
            empty_backlog(),
            empty_next(),
            empty_rfcs(),
            empty_blocked(),
            empty_blocked(),
            fresh_boot(),
            no_commits(),
        );
        let output = render_human(&status);
        assert!(!output.contains("Recent commits"));
    }

    // --- Boot staleness variants ---

    #[test]
    fn boot_stale_output() {
        let boot = BootSection {
            staleness: "stale".to_string(),
            age_seconds: 3600,
            commit: "deadbee".to_string(),
        };
        let status = assemble_status(
            counts(1, 0, 1),
            empty_backlog(),
            empty_next(),
            empty_rfcs(),
            empty_blocked(),
            empty_blocked(),
            boot,
            no_commits(),
        );
        let output = render_human(&status);
        assert!(output.contains("boot.md stale from commit deadbee\n"));
    }

    #[test]
    fn boot_missing_output() {
        let boot = BootSection {
            staleness: "missing".to_string(),
            age_seconds: 0,
            commit: String::new(),
        };
        let status = assemble_status(
            counts(1, 0, 1),
            empty_backlog(),
            empty_next(),
            empty_rfcs(),
            empty_blocked(),
            empty_blocked(),
            boot,
            no_commits(),
        );
        let output = render_human(&status);
        assert!(output.contains("boot.md missing\n"));
    }

    // --- is_empty ---

    #[test]
    fn is_empty_true_when_no_active_slices_and_no_backlog() {
        let status = assemble_status(
            empty_counts(),
            empty_backlog(),
            empty_next(),
            empty_rfcs(),
            empty_blocked(),
            empty_blocked(),
            fresh_boot(),
            no_commits(),
        );
        assert!(is_empty(&status));
    }

    #[test]
    fn is_empty_false_when_has_active_slices() {
        let status = assemble_status(
            counts(1, 0, 1),
            empty_backlog(),
            empty_next(),
            empty_rfcs(),
            empty_blocked(),
            empty_blocked(),
            fresh_boot(),
            no_commits(),
        );
        assert!(!is_empty(&status));
    }

    #[test]
    fn is_empty_false_when_has_backlog_items() {
        let mut backlog = BTreeMap::new();
        backlog.insert("issue".to_string(), 1_usize);
        let status = assemble_status(
            empty_counts(),
            backlog,
            empty_next(),
            empty_rfcs(),
            empty_blocked(),
            empty_blocked(),
            fresh_boot(),
            no_commits(),
        );
        assert!(!is_empty(&status));
    }

    // --- VA-1: blocked items respect hard needs edges only ---
    // The blocked detection delegates to priority::channels::blocked / blocked_by,
    // which only walks the dep_overlay (needs edges), not the seq_overlay (after).
    // This is verified by the priority module's own tests (channels::blocked_by
    // exclusively walks dep_overlay). The status module's render test confirms
    // that blocked items passed into assemble_status render correctly.

    #[test]
    fn blocked_items_render_correctly() {
        let blocked = vec![BlockedItem {
            id: "SL-082".to_string(),
            title: "reconcile engine".to_string(),
            blocked_by: vec!["SL-047".to_string()],
        }];
        let status = assemble_status(
            counts(1, 1, 2),
            empty_backlog(),
            empty_next(),
            empty_rfcs(),
            blocked,
            empty_blocked(),
            fresh_boot(),
            no_commits(),
        );
        let output = render_human(&status);
        assert!(output.contains("Blocked slices"));
        assert!(output.contains("SL-082 blocked by SL-047"));
        assert!(!output.contains("Blocked backlog"));
    }

    // --- NextItem renders correctly (truncation happens in run, not assemble) ---

    #[test]
    fn next_up_shows_five_items() {
        let mut backlog = BTreeMap::new();
        backlog.insert("issue".to_string(), 1_usize);
        let next: Vec<NextItem> = (1..=5)
            .map(|i| NextItem {
                id: format!("SL-{i:03}"),
                status: "design".to_string(),
                title: format!("slice {i}"),
            })
            .collect();
        let status = assemble_status(
            counts(5, 0, 5),
            backlog,
            next,
            empty_rfcs(),
            empty_blocked(),
            empty_blocked(),
            fresh_boot(),
            no_commits(),
        );
        let output = render_human(&status);
        assert!(output.contains("SL-001 (design)"));
        assert!(output.contains("SL-005 (design)"));
    }

    // --- VT-1: RFC count line + title listing ---

    #[test]
    fn rfc_count_line_renders_in_work_section() {
        let rfcs = RfcSummary {
            open: 3,
            total: 5,
            open_titles: vec![
                RfcTitle {
                    id: "RFC-003".into(),
                    title: "Use async".into(),
                },
                RfcTitle {
                    id: "RFC-002".into(),
                    title: "Add linter".into(),
                },
                RfcTitle {
                    id: "RFC-001".into(),
                    title: "New format".into(),
                },
            ],
        };
        let status = assemble_status(
            counts(1, 0, 2),
            empty_backlog(),
            empty_next(),
            rfcs,
            empty_blocked(),
            empty_blocked(),
            fresh_boot(),
            no_commits(),
        );
        let output = render_human(&status);
        assert!(output.contains("rfcs: 3 open, 5 total\n"));
        assert!(output.contains("RFC-003 Use async\n"));
        assert!(output.contains("RFC-002 Add linter\n"));
        assert!(output.contains("RFC-001 New format\n"));
        assert!(!output.contains("+ more"));
    }

    #[test]
    fn rfc_overflow_shows_ten_titles_plus_k_more() {
        let titles: Vec<RfcTitle> = (1..=12)
            .map(|i| RfcTitle {
                id: format!("RFC-{i:03}"),
                title: format!("Title {i}"),
            })
            .collect();
        let rfcs = RfcSummary {
            open: 12,
            total: 15,
            open_titles: titles.into_iter().take(10).collect(),
        };
        let status = assemble_status(
            counts(1, 0, 2),
            empty_backlog(),
            empty_next(),
            rfcs,
            empty_blocked(),
            empty_blocked(),
            fresh_boot(),
            no_commits(),
        );
        let output = render_human(&status);
        assert!(output.contains("rfcs: 12 open, 15 total\n"));
        // Exactly 10 titles rendered, then +2 more overflow.
        for i in 1..=10 {
            assert!(output.contains(&format!("RFC-{i:03} Title {i}\n")));
        }
        assert!(!output.contains("RFC-011"));
        assert!(!output.contains("RFC-012"));
        assert!(output.contains("+2 more\n"));
    }

    #[test]
    fn rfc_section_suppressed_when_no_rfcs() {
        let status = assemble_status(
            counts(1, 0, 2),
            empty_backlog(),
            empty_next(),
            empty_rfcs(),
            empty_blocked(),
            empty_blocked(),
            fresh_boot(),
            no_commits(),
        );
        let output = render_human(&status);
        assert!(!output.contains("rfcs:"));
    }

    #[test]
    fn rfc_section_shows_count_but_no_titles_when_none_open() {
        let rfcs = RfcSummary {
            open: 0,
            total: 3,
            open_titles: vec![],
        };
        let status = assemble_status(
            counts(1, 0, 2),
            empty_backlog(),
            empty_next(),
            rfcs,
            empty_blocked(),
            empty_blocked(),
            fresh_boot(),
            no_commits(),
        );
        let output = render_human(&status);
        assert!(output.contains("rfcs: 0 open, 3 total\n"));
        // Should NOT list any titles.
        assert!(!output.contains("RFC-"));
    }

    // --- VT-2: empty-state — RFCs do NOT flip empty ---

    #[test]
    fn repo_with_only_open_rfcs_is_still_empty() {
        let rfcs = RfcSummary {
            open: 5,
            total: 5,
            open_titles: vec![RfcTitle {
                id: "RFC-001".into(),
                title: "Use Rust?".into(),
            }],
        };
        let status = assemble_status(
            empty_counts(),
            empty_backlog(),
            empty_next(),
            rfcs,
            empty_blocked(),
            empty_blocked(),
            fresh_boot(),
            no_commits(),
        );
        // is_empty must stay false on non-zero rfcs — only slices.active + backlog sum matter.
        assert!(is_empty(&status));
        assert_eq!(render_human(&status), "No active work.\n");
    }

    // --- VT-4: JSON envelope carries RFC data ---

    #[test]
    fn json_envelope_carries_rfc_counts_and_titles() {
        let rfcs = RfcSummary {
            open: 3,
            total: 5,
            open_titles: vec![
                RfcTitle {
                    id: "RFC-003".into(),
                    title: "Use async".into(),
                },
                RfcTitle {
                    id: "RFC-002".into(),
                    title: "Add linter".into(),
                },
            ],
        };
        let status = assemble_status(
            counts(1, 0, 2),
            empty_backlog(),
            empty_next(),
            rfcs,
            empty_blocked(),
            empty_blocked(),
            fresh_boot(),
            no_commits(),
        );
        let json = render_json(&status).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["kind"], "status");
        assert_eq!(v["work"]["rfcs"]["open"], 3);
        assert_eq!(v["work"]["rfcs"]["total"], 5);
        let titles = v["work"]["rfcs"]["open_titles"].as_array().unwrap();
        assert_eq!(titles.len(), 2);
        assert_eq!(titles[0]["id"], "RFC-003");
        assert_eq!(titles[0]["title"], "Use async");
    }

    // --- Parse git log ---

    #[test]
    fn parse_git_log_parses_standard_format() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // Set up a git repo so `git log` can run.
        let mut child = std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .arg("init")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        child.wait().unwrap();

        // Configure git user for commits.
        for (k, v) in [("user.name", "test"), ("user.email", "test@test")] {
            std::process::Command::new("git")
                .arg("-C")
                .arg(root)
                .args(["config", k, v])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .output()
                .unwrap();
        }

        // Create a file and commit.
        let f = root.join("test.txt");
        fs::write(&f, "hello").unwrap();
        std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["add", "test.txt"])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["commit", "-m", "test commit"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .unwrap();

        let commits = super::parse_git_log(root);
        assert!(
            !commits.is_empty(),
            "git log should produce at least one commit"
        );
        assert!(!commits[0].hash.is_empty());
        assert!(commits[0].subject.contains("test commit"));
    }

    use std::fs;
}
