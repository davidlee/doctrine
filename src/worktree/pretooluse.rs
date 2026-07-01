// SPDX-License-Identifier: GPL-3.0-only
//! `PreToolUse` hook shell — SL-182 PHASE-03 (command tier, ADR-001).
//!
//! The thin impure shell over the pure jail core (`super::jail`). It reads the
//! `PreToolUse` payload on stdin, resolves the impure inputs the leaf cannot
//! touch — git topology, host capability, path canonicalization — passes them in
//! as data, renders the leaf's `Decision` to `hookSpecificOutput` JSON, and
//! **exits 0 always** (deny is data, never an exit code —
//! `mem.fact.claude.pretooluse-hook-fail-open`).
//!
//! Two walls, one subcommand (design §5.4): matcher `Bash` → the nested-bwrap
//! command rewrite (`updatedInput`); matcher `Edit|Write` → the `realpath ⊆ cwd`
//! pathcheck. Both discriminated here on `tool_name`.
//!
//! ## Pure / impure split
//! `decide` + `render` are PURE (unit-tested with synthetic stdin, VT-1/VT-3).
//! `run_pretooluse` is the imperative seam: stdin read, `CLAUDE_PROJECT_DIR`
//! anchor + `is_linked_worktree` topology, `realpath -m` canonicalization, and
//! the `bwrap`-presence capability probe. The orchestrator fast-path
//! (no `agent_id`, INV-1) short-circuits BEFORE any subprocess — this hook fires
//! on EVERY tool call, including the human's, and must leave that path free.

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context;
use serde::Deserialize;

use super::jail::{Backend, Decision, JailPolicy, decide_bash, decide_write, resolve_target};

// ---- stdin field / tool vocabulary (STD-001: single-sourced) -------------------
/// Harness-supplied project-root anchor (`docs/claude/hooks.md:462`). The topology
/// check confirms `cwd` shares this project's git-common-dir (A1). Absent ⇒
/// fail-closed (a subagent whose project cannot be confirmed is not jailed HERE).
const ENV_PROJECT_DIR: &str = "CLAUDE_PROJECT_DIR";
const TOOL_BASH: &str = "Bash";
const TOOL_EDIT: &str = "Edit";
const TOOL_WRITE: &str = "Write";

// ---- emitted-JSON vocabulary (STD-001; mirrors the probe wrap/pathcheck) --------
const HOOK_EVENT: &str = "PreToolUse";
const DECISION_DENY: &str = "deny";
const DECISION_ALLOW: &str = "allow";
/// Prepended to every deny reason (design §5.2 — one unified prefix; the probe's
/// split `worktree-jail:` / `worktree-pathwall:` is cosmetic and collapsed here).
const REASON_PREFIX: &str = "worktree-jail: ";

// ---- host capability probe vocabulary (STD-001) --------------------------------
const BWRAP_BIN: &str = "bwrap";
const REALPATH_BIN: &str = "realpath";
/// `realpath -m` — canonicalize a possibly-missing path (a write target need not
/// exist yet). Matches the proven probe (`pretooluse-pathcheck.sh`).
const REALPATH_MISSING_FLAG: &str = "-m";
const ENV_PATH: &str = "PATH";
/// The per-arm `Backend::Deny` reason when the Linux host has no `bwrap`.
const REASON_NO_BWRAP: &str = "bwrap-unavailable";

/// The `PreToolUse` stdin subset consumed (design §5.2). Every field is optional
/// so a malformed / partial payload folds to `Default` — fail-closed: a subagent
/// (`agent_id` present) with no resolvable `cwd`/topology cannot be confirmed a
/// project worktree and is rejected, never silently jailed-nowhere.
#[derive(Debug, Default, Deserialize)]
struct PreToolUseInput {
    #[serde(default)]
    agent_id: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    tool_name: Option<String>,
    #[serde(default)]
    tool_input: ToolInput,
}

#[derive(Debug, Default, Deserialize)]
struct ToolInput {
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    file_path: Option<String>,
}

/// Map the payload + resolved impure inputs to a leaf `Decision`. PURE. The shell
/// resolves `cwd` (canonicalized), `cwd_is_project_worktree` (topology), `real`
/// (the canonicalized write target), `backend` (host capability), and `policy`.
/// An unregistered `tool_name` ⇒ `PassThrough` (the matcher should not route it
/// here; guarding an unread tool would be a latent jail hole — design §5.2).
fn decide(
    input: &PreToolUseInput,
    cwd: &Path,
    cwd_is_project_worktree: bool,
    real: Option<&Path>,
    backend: &Backend,
    policy: &JailPolicy,
) -> Decision {
    let target = resolve_target(input.agent_id.as_deref(), cwd, cwd_is_project_worktree);
    match input.tool_name.as_deref() {
        Some(TOOL_BASH) => {
            let cmd = input.tool_input.command.as_deref().unwrap_or_default();
            let desc = input.tool_input.description.as_deref().unwrap_or_default();
            decide_bash(&target, cmd, desc, policy, backend)
        }
        Some(TOOL_EDIT | TOOL_WRITE) => decide_write(&target, real, policy),
        _ => Decision::PassThrough,
    }
}

/// Render a `Decision` to the `hookSpecificOutput` JSON line, or `None` for
/// pass-through (emit nothing). PURE. Shapes mirror the probe (design §5.2):
/// deny ⇒ `permissionDecision:"deny"` + prefixed reason; wrap ⇒
/// `permissionDecision:"allow"` + `updatedInput:{command,description}`.
fn render(decision: &Decision) -> Option<String> {
    match decision {
        Decision::PassThrough => None,
        Decision::Deny { reason } => Some(
            serde_json::json!({
                "hookSpecificOutput": {
                    "hookEventName": HOOK_EVENT,
                    "permissionDecision": DECISION_DENY,
                    "permissionDecisionReason": format!("{REASON_PREFIX}{reason}"),
                }
            })
            .to_string(),
        ),
        Decision::WrapBash {
            command,
            description,
        } => Some(
            serde_json::json!({
                "hookSpecificOutput": {
                    "hookEventName": HOOK_EVENT,
                    "permissionDecision": DECISION_ALLOW,
                    "updatedInput": { "command": command, "description": description },
                }
            })
            .to_string(),
        ),
    }
}

/// Is `cwd` a linked worktree of THIS project? (A1 — git topology, not a path
/// prefix.) Two gates: `is_linked_worktree(cwd)` AND `cwd`'s git-common-dir equals
/// the `CLAUDE_PROJECT_DIR` anchor's common-dir. The equality holds for any
/// same-repo worktree regardless of whether the anchor points at the main tree or
/// another worktree (both resolve to the one shared `.git`), while a sibling repo's
/// worktree — e.g. a ro-mounted `/workspace` repo — differs and is rejected.
/// Fail-closed: any git error, or an absent anchor, ⇒ `false` (⇒ Reject).
fn cwd_is_project_worktree(cwd: &Path) -> bool {
    if !matches!(super::shared::is_linked_worktree(cwd), Ok(true)) {
        return false;
    }
    let Some(anchor) = std::env::var_os(ENV_PROJECT_DIR) else {
        return false;
    };
    let anchor = PathBuf::from(anchor);
    match (
        super::shared::common_git_dir(cwd),
        super::shared::common_git_dir(&anchor),
    ) {
        (Ok(cwd_common), Ok(anchor_common)) => cwd_common == anchor_common,
        _ => false,
    }
}

/// Canonicalize a write target with `realpath -m` semantics (symlink-resolved,
/// absolute, missing-safe — the file need not exist yet). Relative paths join
/// `cwd` first, exactly as the probe (`pretooluse-pathcheck.sh`). This is the
/// R4-canon boundary the leaf trusts: an un-canonicalized `..`/symlink target
/// would bypass the worktree wall, so it MUST resolve here before `decide_write`.
/// `None` ⇒ the leaf denies (no usable path).
fn canonicalize_missing(cwd: &Path, file_path: &str) -> Option<PathBuf> {
    let raw = Path::new(file_path);
    let abs = if raw.is_absolute() {
        raw.to_path_buf()
    } else {
        cwd.join(raw)
    };
    let out = Command::new(REALPATH_BIN)
        .arg(REALPATH_MISSING_FLAG)
        .arg(&abs)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let resolved = String::from_utf8(out.stdout).ok()?;
    let trimmed = resolved.trim_end_matches('\n');
    (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
}

/// Whether `bwrap` resolves on `PATH` (the `command -v bwrap` the probe ran).
/// Capability is DATA: absence ⇒ `Backend::Deny` ⇒ the leaf denies with the
/// per-arm reason, never an unconfined pass-through (fail-closed).
fn have_bwrap() -> bool {
    let Some(path) = std::env::var_os(ENV_PATH) else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join(BWRAP_BIN).is_file())
}

/// Resolve the host capability descriptor (RV-202 — capability-as-data). Linux:
/// `bwrap` present ⇒ `Bwrap`, else `Deny{bwrap-unavailable}`.
fn probe_backend() -> Backend {
    if have_bwrap() {
        Backend::Bwrap
    } else {
        Backend::Deny {
            reason: REASON_NO_BWRAP.to_string(),
        }
    }
}

/// `doctrine worktree pretooluse` — the `PreToolUse` hook entry. Reads stdin,
/// resolves impure inputs, prints the decision, exits 0 always.
pub(crate) fn run_pretooluse() -> anyhow::Result<()> {
    let mut raw = String::new();
    // Fold an stdin read error into an empty payload ⇒ Default (fail-closed).
    let _read = io::stdin().read_to_string(&mut raw);
    let input: PreToolUseInput = serde_json::from_str(&raw).unwrap_or_default();

    // Orchestrator fast-path (INV-1): no `agent_id` ⇒ never jailed. Emit nothing
    // and skip ALL subprocess work — this hook fires on every tool call, the
    // human orchestrator's included; keep that path allocation-free.
    if input.agent_id.is_none() {
        return Ok(());
    }

    // Subagent: resolve the impure inputs the pure core consumes as data.
    let cwd = input
        .cwd
        .as_deref()
        .and_then(|c| fs::canonicalize(c).ok())
        .unwrap_or_default();
    let is_project_worktree = cwd_is_project_worktree(&cwd);
    let backend = probe_backend();
    // PHASE-04 wires the per-worktree `jail/<name>.toml` lookup + validate_policy;
    // here the strictest floor (rw worktree, ro rest) always applies.
    let policy = JailPolicy::default();
    let real = match input.tool_name.as_deref() {
        Some(TOOL_EDIT | TOOL_WRITE) => input
            .tool_input
            .file_path
            .as_deref()
            .and_then(|fp| canonicalize_missing(&cwd, fp)),
        _ => None,
    };

    let decision = decide(
        &input,
        &cwd,
        is_project_worktree,
        real.as_deref(),
        &backend,
        &policy,
    );
    if let Some(json) = render(&decision) {
        writeln!(io::stdout(), "{json}").context("emit hook decision")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const WT: &str = "/home/u/proj/.worktrees/agent-abc";

    fn input(agent: Option<&str>, tool: &str) -> PreToolUseInput {
        PreToolUseInput {
            agent_id: agent.map(str::to_string),
            cwd: Some(WT.to_string()),
            tool_name: Some(tool.to_string()),
            tool_input: ToolInput::default(),
        }
    }

    fn bash(agent: Option<&str>, cmd: &str) -> PreToolUseInput {
        let mut i = input(agent, TOOL_BASH);
        i.tool_input.command = Some(cmd.to_string());
        i.tool_input.description = Some("run it".to_string());
        i
    }

    fn parse(json: &str) -> serde_json::Value {
        serde_json::from_str(json).expect("emitted JSON parses")
    }

    // ── VT-1: synthetic stdin → emitted JSON, every §5.2 shape ──────────────────

    #[test]
    fn bash_in_project_worktree_wraps_via_updated_input() {
        let d = decide(
            &bash(Some("a1"), "echo hi"),
            Path::new(WT),
            true,
            None,
            &Backend::Bwrap,
            &JailPolicy::default(),
        );
        let json = render(&d).expect("wrap emits JSON");
        let v = parse(&json);
        let out = &v["hookSpecificOutput"];
        assert_eq!(out["hookEventName"], HOOK_EVENT);
        assert_eq!(out["permissionDecision"], DECISION_ALLOW);
        let wrapped = out["updatedInput"]["command"]
            .as_str()
            .expect("command string");
        // The original command is opaquely wrapped in a nested bwrap jail bound to
        // the worktree — never echoed verbatim (INV-5 / opaque_wrap).
        assert!(wrapped.contains(BWRAP_BIN), "wrapped in bwrap: {wrapped}");
        assert!(wrapped.contains(WT), "jail bound to the worktree: {wrapped}");
        assert!(
            !wrapped.contains("echo hi"),
            "original command is opaque (base64), not literal: {wrapped}"
        );
        assert_eq!(out["updatedInput"]["description"], "run it");
    }

    #[test]
    fn write_escaping_worktree_denies() {
        let d = decide(
            &input(Some("a1"), TOOL_WRITE),
            Path::new(WT),
            true,
            Some(Path::new("/etc/passwd")),
            &Backend::Bwrap,
            &JailPolicy::default(),
        );
        let v = parse(&render(&d).expect("deny emits JSON"));
        let out = &v["hookSpecificOutput"];
        assert_eq!(out["permissionDecision"], DECISION_DENY);
        let reason = out["permissionDecisionReason"].as_str().unwrap();
        assert!(reason.starts_with(REASON_PREFIX), "prefixed: {reason}");
        assert!(reason.contains("escapes-worktree"), "reason: {reason}");
    }

    #[test]
    fn write_inside_worktree_passes_through() {
        let d = decide(
            &input(Some("a1"), TOOL_WRITE),
            Path::new(WT),
            true,
            Some(&Path::new(WT).join("src/lib.rs")),
            &Backend::Bwrap,
            &JailPolicy::default(),
        );
        assert_eq!(d, Decision::PassThrough);
        assert_eq!(render(&d), None, "pass-through emits nothing");
    }

    #[test]
    fn orchestrator_no_agent_id_passes_through() {
        // No agent_id ⇒ Orchestrator regardless of tool/cwd — emit nothing.
        let d = decide(
            &bash(None, "rm -rf /"),
            Path::new(WT),
            false,
            None,
            &Backend::Bwrap,
            &JailPolicy::default(),
        );
        assert_eq!(d, Decision::PassThrough);
        assert_eq!(render(&d), None);
    }

    #[test]
    fn subagent_outside_a_project_worktree_denies() {
        // agent_id present but cwd is not a worktree of this project (isolation:none,
        // or a sibling repo) ⇒ Reject ⇒ deny.
        let d = decide(
            &bash(Some("a1"), "echo hi"),
            Path::new("/home/u/proj"),
            false,
            None,
            &Backend::Bwrap,
            &JailPolicy::default(),
        );
        let v = parse(&render(&d).expect("deny emits JSON"));
        assert_eq!(v["hookSpecificOutput"]["permissionDecision"], DECISION_DENY);
        let reason = v["hookSpecificOutput"]["permissionDecisionReason"]
            .as_str()
            .unwrap();
        assert!(reason.contains("cwd-not-a-worktree"), "reason: {reason}");
    }

    #[test]
    fn write_to_repo_root_ancestor_denies() {
        // INV-2: the repo root is an ANCESTOR of the worktree, so a write there
        // escapes the worktree wall (pathcheck is component-wise, not prefix).
        let d = decide(
            &input(Some("a1"), TOOL_EDIT),
            Path::new(WT),
            true,
            Some(Path::new("/home/u/proj/Cargo.toml")),
            &Backend::Bwrap,
            &JailPolicy::default(),
        );
        let v = parse(&render(&d).expect("deny emits JSON"));
        assert_eq!(v["hookSpecificOutput"]["permissionDecision"], DECISION_DENY);
    }

    #[test]
    fn unregistered_tool_passes_through() {
        // The matcher only routes Bash + Edit|Write; anything else is not a guarded
        // surface — pass through rather than deny (design §5.2).
        let d = decide(
            &input(Some("a1"), "Read"),
            Path::new(WT),
            true,
            None,
            &Backend::Bwrap,
            &JailPolicy::default(),
        );
        assert_eq!(d, Decision::PassThrough);
    }

    // ── VT-3: runtime fail-closed — a degraded backend denies, never passes ─────

    #[test]
    fn jailed_bash_with_no_bwrap_backend_denies_not_passthrough() {
        // Host probe found no bwrap ⇒ Backend::Deny. A jailed subagent's Bash must
        // DENY with the per-arm reason, never fall through unconfined (F-1).
        let d = decide(
            &bash(Some("a1"), "echo hi"),
            Path::new(WT),
            true,
            None,
            &Backend::Deny {
                reason: REASON_NO_BWRAP.to_string(),
            },
            &JailPolicy::default(),
        );
        let v = parse(&render(&d).expect("deny emits JSON"));
        assert_eq!(v["hookSpecificOutput"]["permissionDecision"], DECISION_DENY);
        let reason = v["hookSpecificOutput"]["permissionDecisionReason"]
            .as_str()
            .unwrap();
        assert!(reason.contains(REASON_NO_BWRAP), "per-arm reason: {reason}");
    }
}
