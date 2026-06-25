// SPDX-License-Identifier: GPL-3.0-only
//! worktree create-fork — the claude `WorktreeCreate` hook verb (SL-152).
//!
//! Mirror of [`super::subagent`]: a PURE classifier ([`classify_create`] +
//! [`sanitise_name`]) decides Fork-vs-Passthrough-vs-Refuse from already-resolved
//! facts (no git / disk / env / clock in the classifier — ADR-001 leaf, CLAUDE.md
//! pure/imperative split), and an in-file impure SHELL ([`run_create_fork`]) gathers
//! those facts — the payload cwd realpath, the `git -C cwd --show-toplevel` coord-tree
//! root (NOT `primary_worktree`: create-fork fires in the PARENT before the fork
//! exists, G2/I5), the arming-dir realpath compare, the `base` file read — and ACTS
//! ([`act_on_create`]), exactly as `run_stamp_subagent` resolves facts for
//! `classify_stamp`. The shell reads `{cwd, name}` JSON on stdin, prints the created
//! absolute path ALONE on stdout (D11/G1), routes everything else to stderr, and
//! fails closed (non-zero exit, never a panic) on any malformed input or failure.

use super::fork::{fork_core, remove_worktree_dir};
use super::provision::run_provision;
use crate::git;
use anyhow::{Context, bail};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

/// Verdict of the PURE create classifier: positional arming says fork-or-passthrough.
/// `Fork` when the payload cwd IS the arming dir and `base` parses to a plausible sha
/// (the dispatch-worker spawn); `Passthrough` for a benign spawn from anywhere else
/// (a plain detached worktree, same provisioning, no worker marker). The validated
/// `name` slug is carried in BOTH arms (D-P2) so the shell does not re-sanitise it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CreateAction {
    /// cwd IS the arming dir and `base` is a plausible sha ⇒ fork off `base` on
    /// `dispatch/<name>`, provision + worker-mark inside the fork.
    Fork { base: String, name: String },
    /// cwd is anything else ⇒ a benign detached worktree at `<name>`, provisioned by
    /// the same copier, NOT worker-marked.
    Passthrough { name: String },
}

/// Why `create-fork` refuses (design §5.2 step 3, §5.5; mirrors [`super::subagent::
/// StampRefusal`]). Each variant fails closed with a distinct named token — the
/// property the goldens assert, never a proxy boolean.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CreateRefusal {
    /// The payload `cwd` did not resolve (absent / unreadable) — nothing to position.
    MissingCwd,
    /// The payload `name` failed [`sanitise_name`]; carries the specific reason.
    BadName(NameRefusal),
    /// cwd IS the arming dir but the `base` file is absent/empty — no commit to fork.
    MissingBase,
    /// cwd IS the arming dir and `base` is present but not a plausible sha (shape).
    /// The authoritative "is it a commit" check is the shell's `rev-parse` (PHASE-02);
    /// this catches obvious garbage early.
    BadBase,
}

impl CreateRefusal {
    /// The distinct named token each refusal fails closed with.
    pub(crate) fn token(self) -> &'static str {
        match self {
            CreateRefusal::MissingCwd => "missing-cwd",
            CreateRefusal::BadName(_) => "bad-name",
            CreateRefusal::MissingBase => "missing-base",
            CreateRefusal::BadBase => "bad-base",
        }
    }
}

/// Why [`sanitise_name`] rejects a payload `name`. The sanitiser is validate-and-pass
/// (identity-or-refuse, D-P1): it NEVER rewrites — a lossy normalisation would break
/// the `basename(worktreePath)` round-trip the orchestrator derives (D8/I3). Each
/// variant carries a distinct named token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NameRefusal {
    /// Empty after nothing — the literal empty string.
    Empty,
    /// Contains ASCII/Unicode whitespace anywhere (leading, trailing, or internal).
    /// Rejected wholesale rather than trimmed, so a valid name round-trips unchanged.
    Whitespace,
    /// Contains `/` — a path separator, never a single ref component.
    Slash,
    /// Contains `..` — a path-traversal sequence (and a git ref ban).
    DotDot,
    /// Outside the conservative `[A-Za-z0-9._-]` envelope, OR a leading `.`, OR a
    /// trailing `.lock` — the catch-all ref/path-unsafe refusal.
    RefInvalid,
}

impl NameRefusal {
    /// The distinct named token each refusal fails closed with.
    pub(crate) fn token(self) -> &'static str {
        match self {
            NameRefusal::Empty => "empty",
            NameRefusal::Whitespace => "whitespace",
            NameRefusal::Slash => "slash",
            NameRefusal::DotDot => "dotdot",
            NameRefusal::RefInvalid => "ref-invalid",
        }
    }
}

/// Validate a payload `name` to a ref- and path-safe slug (I4, shape only — a live-ref
/// collision is imperative, deferred to PHASE-02). PURE and validate-and-pass: a valid
/// name returns UNCHANGED (identity, D-P1); anything outside the envelope is rejected
/// fail-closed with a named token, never silently rewritten.
///
/// The envelope is deliberately CONSERVATIVE (a strict allowlist) — it may reject some
/// git-legal names, which is fine (fail-closed; the harness names sit well inside it).
/// Accepts BOTH observed forms (G7): `agent-<hex>` (tool spawns, P3) and the moby
/// `word-word-hex` slug (user / `--worktree` spawns, hooks.md:2419).
///
/// Order (each gate names itself): empty → whitespace → `/` → `..` → charset+`.`-edges.
pub(crate) fn sanitise_name(name: &str) -> Result<String, NameRefusal> {
    if name.is_empty() {
        return Err(NameRefusal::Empty);
    }
    if name.chars().any(char::is_whitespace) {
        return Err(NameRefusal::Whitespace);
    }
    if name.contains('/') {
        return Err(NameRefusal::Slash);
    }
    if name.contains("..") {
        return Err(NameRefusal::DotDot);
    }
    let charset_ok = name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
    // git rejects a ref component ending in the LITERAL lowercase `.lock` (its
    // lockfile sentinel) — a case-sensitive suffix ban, NOT a file-extension match,
    // so the pedantic case-insensitive-extension lint does not apply.
    #[expect(
        clippy::case_sensitive_file_extension_comparisons,
        reason = "git's ref `.lock` ban is the literal lowercase suffix, not a file extension"
    )]
    let lock_suffixed = name.ends_with(".lock");
    if !charset_ok || name.starts_with('.') || lock_suffixed {
        return Err(NameRefusal::RefInvalid);
    }
    Ok(name.to_string())
}

/// Shape check on the `base` file contents: a trimmed, plausible sha (non-empty, length
/// 4..=64, all ASCII hex). PURE and pure-LIGHT only — the authoritative commit check is
/// the shell's `rev-parse` (PHASE-02). Catches obvious garbage so `bad-base` names
/// itself early.
fn plausible_sha(base: &str) -> bool {
    let trimmed = base.trim();
    (4..=64).contains(&trimmed.len()) && trimmed.chars().all(|c| c.is_ascii_hexdigit())
}

/// PURE create classifier (no git / disk / env / clock — ADR-001 leaf). Mirror of
/// [`super::subagent::classify_stamp`]: takes the gathered, already-resolved FACTS and
/// returns the verdict; the shell resolves the cwd realpath, the arming-dir realpath
/// compare, and the `base` file read (all impure), then calls this.
///
/// * `cwd_resolved` — the payload carried a `cwd` that resolved (canonicalised) on disk.
/// * `cwd_is_arming_dir` — that resolved cwd IS the arming dir
///   `<coord>/.doctrine/state/dispatch/spawn` (both realpath'd by the shell). Positional
///   arming (D3/D4): discrimination is cwd-as-channel, never a payload class tag.
/// * `base` — the arming dir's `base` file contents (`None` if absent/empty).
/// * `name` — the payload `name`, validated here via [`sanitise_name`].
///
/// Precond order (mirror `classify_stamp`): cwd-resolution → name-validity → (when
/// armed) base-presence → base-shape. cwd before name so a missing cwd names itself
/// first; name before base so a bad name is caught on the benign path too.
pub(crate) fn classify_create(
    cwd_resolved: bool,
    cwd_is_arming_dir: bool,
    base: Option<&str>,
    name: &str,
) -> Result<CreateAction, CreateRefusal> {
    if !cwd_resolved {
        return Err(CreateRefusal::MissingCwd);
    }
    let slug = sanitise_name(name).map_err(CreateRefusal::BadName)?;
    if cwd_is_arming_dir {
        match base {
            None => Err(CreateRefusal::MissingBase),
            Some(b) if !plausible_sha(b) => Err(CreateRefusal::BadBase),
            Some(b) => Ok(CreateAction::Fork {
                base: b.trim().to_string(),
                name: slug,
            }),
        }
    } else {
        Ok(CreateAction::Passthrough { name: slug })
    }
}

// ---------------------------------------------------------------------------
// Imperative shell — gather → classify → act (impure: stdin, git, disk).
// ---------------------------------------------------------------------------

/// The arming dir, relative to the coord-tree root: the orchestrator `cd`s HERE
/// before a dispatch-worker spawn, and writes `base` inside it (design §5.3). The
/// payload cwd BEING this dir is the positional Fork discriminator (D3/D4).
pub(crate) const ARMING_SUBPATH: &str = ".doctrine/state/dispatch/spawn";
/// Where every created tree lives under the coord-tree root: `<root>/.worktrees/<name>`.
const WORKTREES_SUBDIR: &str = ".worktrees";

/// The `WorktreeCreate` payload subset we read (tolerate extra fields). JSON on
/// stdin: `{ "cwd": "<orchestrator cwd at spawn>", "name": "<unique slug>" }`. The
/// payload is THIN by construction (probe, design §10): no `agent_type`, no base, no
/// target path — discrimination is positional (cwd) and the base rides the arming dir.
#[derive(Debug, Default, serde::Deserialize)]
struct CreatePayload {
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    name: Option<String>,
}

/// Resolve the coord-tree root from the payload `cwd` via `git -C <cwd>
/// --show-toplevel`, canonicalised (G2/I5). This is the source tree for provisioning
/// AND the git `-C` root for creation — NOT `primary_worktree` (the stamp's
/// inside-fork resolution) and NOT the process cwd. `None` ⇒ cwd is not inside a git
/// worktree ⇒ the shell fails closed (no root to fork in). Impure (the git read).
fn resolve_root(cwd: &Path) -> Option<PathBuf> {
    git::git_text(cwd, &["rev-parse", "--show-toplevel"])
        .ok()
        .and_then(|top| fs::canonicalize(top.trim()).ok())
}

/// Act on the pure verdict — the only worktree-mutating step (design §5.2 step 4).
/// Returns the CANONICALISED created dir (stable for the harness to adopt as the
/// session cwd and for PHASE-05's `basename(worktreePath)` derivation).
///
/// * `Fork` — delegate the live-`dir`/`branch` collision refusal AND the
///   add+provision+mark to [`fork_core`] (the byte-identical core, worker=true,
///   provision source = `root`); no parallel pre-check.
/// * `Passthrough` — a benign DETACHED tree at the coord tree's HEAD, provisioned by
///   the SAME copier ([`run_provision`], source = `root` not the fresh tree — the
///   ISS-011 trap), NOT worker-marked. Owns its own `dir`-collision refusal (no
///   branch to check), and COMPENSATES (G3) — removes the half-created tree before
///   the fail-closed bail so an abort leaks nothing.
fn act_on_create(root: &Path, action: CreateAction) -> anyhow::Result<PathBuf> {
    match action {
        CreateAction::Fork { base, name } => {
            let dir = root.join(WORKTREES_SUBDIR).join(&name);
            let branch = format!("dispatch/{name}");
            // fork_core owns the dir/branch collision refusal (`fork-refused: …`) —
            // do NOT re-check here (no parallel impl against shared machinery).
            fork_core(root, &base, &branch, &dir, true)?;
            fs::canonicalize(&dir)
                .with_context(|| format!("canonicalize fork dir {}", dir.display()))
        }
        CreateAction::Passthrough { name } => {
            let dir = root.join(WORKTREES_SUBDIR).join(&name);
            if dir.exists() {
                bail!(
                    "create-refused: name-collision (dir {} already exists)",
                    dir.display()
                );
            }
            // Detached tree at the coord tree's HEAD (replicates `baseRef:"head"`).
            git::git_text(
                root,
                &[
                    "worktree",
                    "add",
                    "--detach",
                    &dir.to_string_lossy(),
                    "HEAD",
                ],
            )
            .with_context(|| format!("git worktree add --detach {} HEAD", dir.display()))?;
            // Provision from the coord tree; compensate on any failure (G3).
            if let Err(cause) = run_provision(Some(root.to_path_buf()), &dir) {
                let debris = remove_worktree_dir(root, &dir);
                if debris.is_empty() {
                    return Err(cause.context(format!(
                        "passthrough provision failed; compensated cleanly (removed {})",
                        dir.display()
                    )));
                }
                bail!(
                    "passthrough-rollback-debris: {} (original cause: {cause:#})",
                    debris.join(", ")
                );
            }
            fs::canonicalize(&dir)
                .with_context(|| format!("canonicalize passthrough dir {}", dir.display()))
        }
    }
}

/// `doctrine worktree create-fork` — the claude `WorktreeCreate` hook verb. Reads the
/// `{cwd, name}` payload on stdin, gathers the impure facts, [`classify_create`]s, and
/// [`act_on_create`]s. stdout carries the created absolute path and NOTHING else
/// (D11/G1); refusals and diagnostics go to stderr; any failure exits non-zero
/// (fail-closed — a non-zero `WorktreeCreate` exit aborts the spawn, design §5).
///
/// No `-p` override: the root is ALWAYS the payload cwd's `--show-toplevel` (G2/I5),
/// never the process cwd. Malformed/empty stdin folds to a named refusal, never a panic.
pub(crate) fn run_create_fork() -> anyhow::Result<()> {
    let mut raw = String::new();
    io::stdin()
        .read_to_string(&mut raw)
        .context("read WorktreeCreate payload")?;
    // Malformed JSON folds to an empty payload ⇒ `missing-cwd` (fail-closed).
    let payload: CreatePayload = serde_json::from_str(&raw).unwrap_or_default();

    let cwd_str = payload.cwd.unwrap_or_default();
    let name = payload.name.unwrap_or_default();

    // Resolve cwd on disk; absent/unresolvable ⇒ cwd_resolved=false ⇒ missing-cwd.
    let cwd_canon = if cwd_str.is_empty() {
        None
    } else {
        fs::canonicalize(&cwd_str).ok()
    };
    let cwd_resolved = cwd_canon.is_some();

    // Root from the PAYLOAD cwd (G2/I5). None ⇒ cannot act (fail-closed below).
    let root = cwd_canon.as_deref().and_then(resolve_root);

    // cwd IS the arming dir? Both realpath'd; a MISSING arming dir (the usual benign
    // case) canonicalises to Err ⇒ folds to false (Passthrough), never propagates.
    let cwd_is_arming = match (root.as_deref(), cwd_canon.as_deref()) {
        (Some(root), Some(cwd)) => {
            fs::canonicalize(root.join(ARMING_SUBPATH)).is_ok_and(|arming| arming == cwd)
        }
        _ => false,
    };

    // The `base` file lives in the arming dir; read it ONLY when armed.
    let base = if cwd_is_arming {
        root.as_deref()
            .and_then(|r| fs::read_to_string(r.join(ARMING_SUBPATH).join("base")).ok())
    } else {
        None
    };

    match classify_create(cwd_resolved, cwd_is_arming, base.as_deref(), &name) {
        Err(refusal) => {
            // Stable token (`bad-name`), plus the specific sanitiser reason in
            // parens for hook debugging (e.g. `create-refused: bad-name (whitespace)`).
            let line = match refusal {
                CreateRefusal::BadName(reason) => {
                    format!("create-refused: {} ({})", refusal.token(), reason.token())
                }
                _ => format!("create-refused: {}", refusal.token()),
            };
            writeln!(io::stderr(), "{line}")?;
            bail!("{line}");
        }
        Ok(action) => {
            // cwd resolved + classified, but not inside a git worktree ⇒ no root to
            // fork in. Fail closed with a named token (never a panic on hook input).
            let Some(root) = root else {
                writeln!(io::stderr(), "create-refused: no-root")?;
                bail!("create-refused: no-root");
            };
            let dir = act_on_create(&root, action)?;
            // stdout = EXACTLY the created path, one line, nothing else (G1/D11).
            writeln!(io::stdout(), "{}", dir.display())?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- VT-2: name sanitiser accept/reject table (both forms accepted; identity) ---

    #[test]
    fn sanitise_accepts_both_observed_name_forms_unchanged() {
        // agent-<hex> (tool spawns, P3) AND moby word-word-hex (user/--worktree).
        assert_eq!(
            sanitise_name("agent-abc123"),
            Ok("agent-abc123".to_string())
        );
        assert_eq!(
            sanitise_name("bold-oak-a3f2"),
            Ok("bold-oak-a3f2".to_string())
        );
        // Dots, underscores, digits inside are fine; identity round-trip.
        assert_eq!(sanitise_name("a.b_c-1"), Ok("a.b_c-1".to_string()));
    }

    #[test]
    fn sanitise_rejects_each_unsafe_shape_with_its_named_token() {
        assert_eq!(sanitise_name(""), Err(NameRefusal::Empty));
        assert_eq!(sanitise_name("a b"), Err(NameRefusal::Whitespace));
        // Leading/trailing whitespace is rejected wholesale (not trimmed) — identity.
        assert_eq!(sanitise_name(" abc"), Err(NameRefusal::Whitespace));
        assert_eq!(sanitise_name("abc\t"), Err(NameRefusal::Whitespace));
        assert_eq!(sanitise_name("   "), Err(NameRefusal::Whitespace));
        assert_eq!(sanitise_name("a/b"), Err(NameRefusal::Slash));
        assert_eq!(sanitise_name("a..b"), Err(NameRefusal::DotDot));
        assert_eq!(sanitise_name(".."), Err(NameRefusal::DotDot));
        assert_eq!(sanitise_name("a~b"), Err(NameRefusal::RefInvalid));
        assert_eq!(sanitise_name("a:b"), Err(NameRefusal::RefInvalid));
        assert_eq!(sanitise_name(".hidden"), Err(NameRefusal::RefInvalid));
        assert_eq!(sanitise_name("x.lock"), Err(NameRefusal::RefInvalid));
    }

    #[test]
    fn name_refusal_tokens_are_distinct() {
        let tokens = [
            NameRefusal::Empty.token(),
            NameRefusal::Whitespace.token(),
            NameRefusal::Slash.token(),
            NameRefusal::DotDot.token(),
            NameRefusal::RefInvalid.token(),
        ];
        let unique: std::collections::BTreeSet<&str> = tokens.iter().copied().collect();
        assert_eq!(unique.len(), 5, "every NameRefusal token is distinct");
        assert_eq!(NameRefusal::Empty.token(), "empty");
        assert_eq!(NameRefusal::Whitespace.token(), "whitespace");
        assert_eq!(NameRefusal::Slash.token(), "slash");
        assert_eq!(NameRefusal::DotDot.token(), "dotdot");
        assert_eq!(NameRefusal::RefInvalid.token(), "ref-invalid");
    }

    // --- VT-1: classifier matrix — distinct tokens, not a proxy bool ---

    const SHA: &str = "68250bcd"; // a plausible short sha (the probe's base, design §1)

    #[test]
    fn missing_cwd_refuses_first_regardless_of_everything_else() {
        // cwd unresolved ⇒ missing-cwd even with a valid name, armed, valid base.
        assert_eq!(
            classify_create(false, true, Some(SHA), "agent-abc123"),
            Err(CreateRefusal::MissingCwd)
        );
        assert_eq!(
            classify_create(false, false, None, ""),
            Err(CreateRefusal::MissingCwd)
        );
        assert_eq!(CreateRefusal::MissingCwd.token(), "missing-cwd");
    }

    #[test]
    fn bad_name_refuses_before_base_on_both_channels() {
        // Armed + valid base but a bad name ⇒ bad-name (name precedes base).
        assert_eq!(
            classify_create(true, true, Some(SHA), "a/b"),
            Err(CreateRefusal::BadName(NameRefusal::Slash))
        );
        // Benign channel with a bad name ⇒ bad-name too (name checked on both paths).
        assert_eq!(
            classify_create(true, false, None, ""),
            Err(CreateRefusal::BadName(NameRefusal::Empty))
        );
        assert_eq!(
            CreateRefusal::BadName(NameRefusal::Slash).token(),
            "bad-name"
        );
    }

    #[test]
    fn armed_without_base_refuses_missing_base() {
        assert_eq!(
            classify_create(true, true, None, "agent-abc123"),
            Err(CreateRefusal::MissingBase)
        );
        assert_eq!(CreateRefusal::MissingBase.token(), "missing-base");
    }

    #[test]
    fn armed_with_unparseable_base_refuses_bad_base() {
        // Non-hex garbage ⇒ bad-base.
        assert_eq!(
            classify_create(true, true, Some("zzz"), "agent-abc123"),
            Err(CreateRefusal::BadBase)
        );
        // Too short (< 4) ⇒ bad-base.
        assert_eq!(
            classify_create(true, true, Some("ab"), "agent-abc123"),
            Err(CreateRefusal::BadBase)
        );
        assert_eq!(CreateRefusal::BadBase.token(), "bad-base");
    }

    #[test]
    fn armed_with_plausible_base_and_valid_name_forks() {
        // cwd IS the arming dir ∧ base is a plausible sha ⇒ Fork; the trimmed sha and
        // validated slug are carried in the verdict (D-P2).
        assert_eq!(
            classify_create(true, true, Some("  68250bcd\n"), "bold-oak-a3f2"),
            Ok(CreateAction::Fork {
                base: "68250bcd".to_string(),
                name: "bold-oak-a3f2".to_string(),
            })
        );
    }

    #[test]
    fn benign_cwd_with_valid_name_passes_through_ignoring_base() {
        // Not the arming dir ⇒ Passthrough regardless of base presence; only the name
        // is validated. Fork is NEVER reached off the arming dir.
        assert_eq!(
            classify_create(true, false, None, "agent-abc123"),
            Ok(CreateAction::Passthrough {
                name: "agent-abc123".to_string(),
            })
        );
        // A stray base on the benign channel is ignored (positional discrimination).
        assert_eq!(
            classify_create(true, false, Some(SHA), "agent-abc123"),
            Ok(CreateAction::Passthrough {
                name: "agent-abc123".to_string(),
            })
        );
    }
}
