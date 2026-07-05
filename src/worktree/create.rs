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
use std::io::{self, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};

/// Verdict of the PURE create classifier: positional arming says fork-or-passthrough.
/// `Fork` when the payload cwd IS the arming dir and `base` parses to a plausible sha
/// (the dispatch-worker spawn); `Passthrough` for a benign spawn from anywhere else
/// (a plain detached worktree, same provisioning, no worker marker). The validated
/// `name` slug is carried in BOTH arms (D-P2) so the shell does not re-sanitise it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CreateAction {
    /// An armed trigger fired with a plausible `base` ⇒ fork off `base` on
    /// `dispatch/<name>`, provision + worker-mark inside the fork. Reached by EITHER
    /// the positional arming-dir trigger OR the confined coord-root trigger (SL-199) —
    /// both converge on this ONE verdict and this ONE act path.
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
    /// The POSITIONAL arming dir fired but the `base` file is absent/empty — armed but
    /// unloaded, no commit to fork. (The confined trigger treats an absent base as a
    /// benign un-armed spawn ⇒ Passthrough, never this refusal.)
    MissingBase,
    /// An armed trigger (positional or confined) fired with a `base` that is present but
    /// not a plausible sha (shape).
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

/// Whether a COORD-tree branch is a dispatch COORDINATION branch — `dispatch/<n>`
/// with an all-digit slice number (the confined orchestrator's branch, SL-199). A
/// worker FORK branch (`dispatch/agent-<hex>`) is deliberately NOT matched: its
/// suffix is never all-digits. PURE, and anchored on the whole `dispatch/` suffix
/// (a component/shape match, not a loose substring). Empty suffix ⇒ false.
fn is_dispatch_coord_branch(branch: &str) -> bool {
    branch
        .strip_prefix("dispatch/")
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

/// PURE create classifier (no git / disk / env / clock — ADR-001 leaf). Mirror of
/// [`super::subagent::classify_stamp`]: takes the gathered, already-resolved FACTS and
/// returns the verdict; the shell resolves the cwd realpath, the arming-dir realpath
/// compare, the coord-root compare, the coord branch, and the `base` file read (all
/// impure), then calls this.
///
/// Two ADDITIVE Fork triggers (SL-199), sharing ONE base-shape gate and ONE Fork verdict:
/// * POSITIONAL (main-thread, `cwd_is_arming_dir`) — the orchestrator `cd`s into the
///   arming dir and the payload cwd IS it (D3/D4, the original discriminator, UNCHANGED).
/// * CONFINED (`cwd_is_coord_root && coord_in_dispatch`) — a confined orchestrator, whose
///   Bash cwd is ALWAYS the coord root (never the arming dir), arming a nested worker from
///   a `dispatch/<n>` coordination branch.
///
/// They DIVERGE only on an ABSENT base: positional ⇒ `MissingBase` (armed-but-unloaded);
/// confined ⇒ `Passthrough` (a plain, un-armed `isolation:worktree` spawn — the normal
/// case, which must pass through, never refuse).
///
/// * `cwd_resolved` — the payload carried a `cwd` that resolved (canonicalised) on disk.
/// * `cwd_is_arming_dir` — resolved cwd IS `<coord>/.doctrine/state/dispatch/spawn`.
/// * `cwd_is_coord_root` — resolved cwd IS the coord-tree root itself.
/// * `coord_in_dispatch` — the coord tree is on a `dispatch/<n>` coordination branch.
/// * `base` — the arming dir's `base` file contents (`None` if absent/empty).
/// * `name` — the payload `name`, validated here via [`sanitise_name`].
///
/// Precond order (mirror `classify_stamp`): cwd-resolution → name-validity → (when a
/// trigger is live) base-shape. cwd before name so a missing cwd names itself first;
/// name before base so a bad name is caught on the benign path too.
#[expect(
    clippy::fn_params_excessive_bools,
    reason = "the four flags are the already-resolved trigger FACTS the pure classifier decides on \
              (two positional, two confined); folding them into an enum would move the trigger \
              logic into the shell, breaking the pure/imperative split this fn exists to keep"
)]
pub(crate) fn classify_create(
    cwd_resolved: bool,
    cwd_is_arming_dir: bool,
    cwd_is_coord_root: bool,
    coord_in_dispatch: bool,
    base: Option<&str>,
    name: &str,
) -> Result<CreateAction, CreateRefusal> {
    if !cwd_resolved {
        return Err(CreateRefusal::MissingCwd);
    }
    let slug = sanitise_name(name).map_err(CreateRefusal::BadName)?;
    let confined = cwd_is_coord_root && coord_in_dispatch;
    if cwd_is_arming_dir || confined {
        match base {
            Some(b) if !plausible_sha(b) => Err(CreateRefusal::BadBase),
            Some(b) => Ok(CreateAction::Fork {
                base: b.trim().to_string(),
                name: slug,
            }),
            // The ONLY divergence between the two triggers: a positional arm with no
            // base is armed-but-unloaded (refuse); a confined arm with no base is the
            // ordinary un-armed isolation spawn (benign Passthrough). Positional takes
            // precedence when both hold.
            None if cwd_is_arming_dir => Err(CreateRefusal::MissingBase),
            None => Ok(CreateAction::Passthrough { name: slug }),
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
/// The arming jail-policy DECLARATION, written beside `base` in [`ARMING_SUBPATH`] by
/// `dispatch arm-spawn` in the SAME arming step (F-4 pairing) and read here to
/// provision. Absent ⇒ no provision ⇒ the pretooluse Default floor (design §5.3).
pub(crate) const ARMING_JAIL_FILE: &str = "jail.toml";
/// Where the PROVISIONED per-worktree policy lives under the coord-tree root:
/// `<root>/`⟨`JAIL_SUBPATH`⟩`/<worktree-name>.toml` — OUTSIDE every worktree and ro to
/// the worker (design §5.3). Written by `create-fork`, read by `pretooluse` (keyed
/// `cwd → basename(worktree)`). Runtime state: gitignored, GC'd with teardown.
pub(crate) const JAIL_SUBPATH: &str = ".doctrine/state/dispatch/jail";
/// Where every created tree lives under the coord-tree root: `<root>/.worktrees/<name>`.
/// `pub(crate)` so the `pretooluse` reader recovers the coord root by stripping this
/// same layout (design §5.3 — one owner of the `.worktrees/<name>` shape, no re-spell).
pub(crate) const WORKTREES_SUBDIR: &str = ".worktrees";

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

/// Provision the arming jail-policy DECLARATION to the per-worktree PROVISIONED policy
/// (design §5.3, NET-NEW step beside `run_provision`/`write_marker`). Copies the arming
/// declaration ([`ARMING_SUBPATH`]/[`ARMING_JAIL_FILE`] under `root`) to
/// [`JAIL_SUBPATH`]/`<name>.toml` under `root`, keyed by the harness-assigned worktree
/// `name`. The destination is OUTSIDE the fork (under the coord root) and ro to the
/// worker under the bwrap ro-bind blanket. Declaration ABSENT ⇒ no-op ⇒ pretooluse
/// resolves the Default floor (never a deny; objective 3). Declaration present but the
/// copy fails ⇒ propagate: `create-fork` is fail-closed, so a failed provision aborts
/// the spawn loudly rather than silently under-confining or dropping the intent.
fn provision_jail_policy(root: &Path, name: &str) -> anyhow::Result<()> {
    let decl = root.join(ARMING_SUBPATH).join(ARMING_JAIL_FILE);
    let body = match fs::read(&decl) {
        Ok(body) => body,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(()),
        Err(e) => {
            return Err(e).with_context(|| format!("read jail declaration {}", decl.display()));
        }
    };
    let dir = root.join(JAIL_SUBPATH);
    fs::create_dir_all(&dir)
        .with_context(|| format!("create jail provision dir {}", dir.display()))?;
    let dest = dir.join(format!("{name}.toml"));
    crate::fsutil::write_atomic(&dest, &body)
        .with_context(|| format!("provision jail policy {}", dest.display()))?;
    Ok(())
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
            // One-shot consume (SL-199 EX-3): unlink the arming `base` slot at the fork
            // point, BEFORE `fork_core`, so a crash between fork and return cannot leave a
            // stale base that mis-forks the NEXT benign spawn. After consumption a second
            // WorktreeCreate with no re-arm reads no base ⇒ classifies Passthrough.
            consume_arming_base(root)?;
            // fork_core owns the dir/branch collision refusal (`fork-refused: …`) —
            // do NOT re-check here (no parallel impl against shared machinery).
            fork_core(root, &base, &branch, &dir, true)?;
            // NET-NEW (PHASE-04): provision the arming jail declaration to the
            // per-worktree policy BEFORE the worker's first tool call — outside the
            // fork, ro to the worker (design §5.3). Absent declaration ⇒ no-op.
            provision_jail_policy(root, &name)
                .with_context(|| format!("provision jail policy for {name}"))?;
            // NET-NEW (SL-198 PHASE-01): write the per-worktree dispatch record beside
            // the jail policy at the fork point — the trust anchor `worker_commit`
            // (PHASE-02) resolves. `base` is B snapshotted HERE (not re-read from the
            // mutable arming slot); `dir`/`branch`/`root(=coord)` are all in scope.
            // Fail-closed, exactly like the jail-policy write (design §5.2 EX-1).
            super::dispatch_record::provision_dispatch_record(root, &name, &base, &dir, &branch)
                .with_context(|| format!("provision dispatch record for {name}"))?;
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

/// One-shot consume of the arming `base` slot (SL-199 EX-3): unlink
/// `<root>/`⟨[`ARMING_SUBPATH`]⟩`/base` so exactly ONE Fork can fire per arm. Absent ⇒
/// no-op (a benign re-spawn with no re-arm, or a direct `act_on_create` fork whose
/// coord tree was never armed). Any OTHER io error propagates: leaving a stale base in
/// place would let the NEXT benign spawn mis-fork, so a failed consume aborts the spawn
/// fail-closed (called BEFORE `fork_core`, so the abort leaves no partial fork). Impure.
fn consume_arming_base(root: &Path) -> anyhow::Result<()> {
    let base = root.join(ARMING_SUBPATH).join("base");
    match fs::remove_file(&base) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e).with_context(|| format!("consume arming base {}", base.display())),
    }
}

/// The impure Fork-trigger facts the classifier consumes, gathered from disk + git.
#[derive(Debug, Default)]
struct ForkFacts {
    /// Resolved cwd IS the arming dir (positional trigger).
    cwd_is_arming: bool,
    /// Resolved cwd IS the coord-tree root (confined orchestrator's Bash cwd).
    cwd_is_coord_root: bool,
    /// The coord tree is on a `dispatch/<n>` coordination branch.
    coord_in_dispatch: bool,
    /// The arming dir's `base` slot contents, read only when a trigger could be live.
    base: Option<String>,
}

/// Whether the coord tree at `root` is on a `dispatch/<n>` coordination branch. Any git
/// error (detached HEAD, not a repo) folds to `false` (fail-closed ⇒ Passthrough). Impure.
fn coord_on_dispatch_branch(root: &Path) -> bool {
    git::git_opt(root, &["symbolic-ref", "--quiet", "--short", "HEAD"])
        .ok()
        .flatten()
        .is_some_and(|branch| is_dispatch_coord_branch(&branch))
}

/// Gather the impure Fork-trigger facts (design §5.2 step 2, SL-199): the arming-dir
/// realpath compare, the coord-root compare, the coord branch, and the `base` slot read
/// (only when EITHER trigger could be live — positional OR confined). Both `root` and
/// `cwd` are already canonicalised by the shell. Keeps the classifier git/disk-free.
fn gather_fork_facts(root: &Path, cwd: &Path) -> ForkFacts {
    // cwd IS the arming dir? Both realpath'd; a MISSING arming dir (the usual benign
    // case) canonicalises to Err ⇒ folds to false, never propagates.
    let cwd_is_arming =
        fs::canonicalize(root.join(ARMING_SUBPATH)).is_ok_and(|arming| arming == cwd);
    // cwd IS the coord root? For a confined coord-root spawn the git toplevel of the
    // payload cwd is itself, so `root == cwd` (both canonicalised).
    let cwd_is_coord_root = root == cwd;
    let coord_in_dispatch = coord_on_dispatch_branch(root);
    // Read the `base` slot when EITHER Fork condition could be live (arming OR confined).
    let base = if cwd_is_arming || (cwd_is_coord_root && coord_in_dispatch) {
        fs::read_to_string(root.join(ARMING_SUBPATH).join("base")).ok()
    } else {
        None
    };
    ForkFacts {
        cwd_is_arming,
        cwd_is_coord_root,
        coord_in_dispatch,
        base,
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

    // Gather the impure Fork-trigger facts (arming dir, coord root, dispatch branch,
    // base slot) — meaningful only when both cwd and root resolved; otherwise the
    // classifier refuses on cwd/root first regardless.
    let facts = match (root.as_deref(), cwd_canon.as_deref()) {
        (Some(root), Some(cwd)) => gather_fork_facts(root, cwd),
        _ => ForkFacts::default(),
    };

    match classify_create(
        cwd_resolved,
        facts.cwd_is_arming,
        facts.cwd_is_coord_root,
        facts.coord_in_dispatch,
        facts.base.as_deref(),
        &name,
    ) {
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
            classify_create(false, true, false, false, Some(SHA), "agent-abc123"),
            Err(CreateRefusal::MissingCwd)
        );
        assert_eq!(
            classify_create(false, false, false, false, None, ""),
            Err(CreateRefusal::MissingCwd)
        );
        assert_eq!(CreateRefusal::MissingCwd.token(), "missing-cwd");
    }

    #[test]
    fn bad_name_refuses_before_base_on_both_channels() {
        // Armed + valid base but a bad name ⇒ bad-name (name precedes base).
        assert_eq!(
            classify_create(true, true, false, false, Some(SHA), "a/b"),
            Err(CreateRefusal::BadName(NameRefusal::Slash))
        );
        // Benign channel with a bad name ⇒ bad-name too (name checked on both paths).
        assert_eq!(
            classify_create(true, false, false, false, None, ""),
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
            classify_create(true, true, false, false, None, "agent-abc123"),
            Err(CreateRefusal::MissingBase)
        );
        assert_eq!(CreateRefusal::MissingBase.token(), "missing-base");
    }

    #[test]
    fn armed_with_unparseable_base_refuses_bad_base() {
        // Non-hex garbage ⇒ bad-base.
        assert_eq!(
            classify_create(true, true, false, false, Some("zzz"), "agent-abc123"),
            Err(CreateRefusal::BadBase)
        );
        // Too short (< 4) ⇒ bad-base.
        assert_eq!(
            classify_create(true, true, false, false, Some("ab"), "agent-abc123"),
            Err(CreateRefusal::BadBase)
        );
        assert_eq!(CreateRefusal::BadBase.token(), "bad-base");
    }

    #[test]
    fn armed_with_plausible_base_and_valid_name_forks() {
        // cwd IS the arming dir ∧ base is a plausible sha ⇒ Fork; the trimmed sha and
        // validated slug are carried in the verdict (D-P2).
        assert_eq!(
            classify_create(
                true,
                true,
                false,
                false,
                Some("  68250bcd\n"),
                "bold-oak-a3f2"
            ),
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
            classify_create(true, false, false, false, None, "agent-abc123"),
            Ok(CreateAction::Passthrough {
                name: "agent-abc123".to_string(),
            })
        );
        // A stray base on the benign channel is ignored (positional discrimination).
        assert_eq!(
            classify_create(true, false, false, false, Some(SHA), "agent-abc123"),
            Ok(CreateAction::Passthrough {
                name: "agent-abc123".to_string(),
            })
        );
    }

    // --- VT-1: SL-199 confined Fork trigger — the SECOND, additive discriminator ---

    #[test]
    fn confined_coord_root_fork_trigger_matrix() {
        // confined = cwd_is_coord_root && coord_in_dispatch. With a plausible base ⇒ Fork.
        assert_eq!(
            classify_create(true, false, true, true, Some(SHA), "agent-x"),
            Ok(CreateAction::Fork {
                base: SHA.to_string(),
                name: "agent-x".to_string(),
            })
        );
        // BENIGN divergence: confined but NO base ⇒ Passthrough (an un-armed
        // isolation:worktree spawn from the coord root is the normal case, NOT a refusal).
        assert_eq!(
            classify_create(true, false, true, true, None, "agent-x"),
            Ok(CreateAction::Passthrough {
                name: "agent-x".to_string(),
            })
        );
        // coord root but NOT on a dispatch branch ⇒ not confined ⇒ Passthrough, base ignored.
        assert_eq!(
            classify_create(true, false, true, false, Some(SHA), "agent-x"),
            Ok(CreateAction::Passthrough {
                name: "agent-x".to_string(),
            })
        );
        // confined with a bad-sha base ⇒ BadBase (shares the positional shape gate).
        assert_eq!(
            classify_create(true, false, true, true, Some("zzz"), "agent-x"),
            Err(CreateRefusal::BadBase)
        );
        // REGRESSION: the positional arm still forks regardless of the coord flags, and
        // its absent-base divergence (MissingBase) is UNCHANGED.
        assert_eq!(
            classify_create(true, true, false, false, Some(SHA), "agent-x"),
            Ok(CreateAction::Fork {
                base: SHA.to_string(),
                name: "agent-x".to_string(),
            })
        );
        assert_eq!(
            classify_create(true, true, false, false, None, "agent-x"),
            Err(CreateRefusal::MissingBase)
        );
    }

    #[test]
    fn dispatch_coord_branch_matches_slice_number_not_worker_fork() {
        assert!(is_dispatch_coord_branch("dispatch/199"));
        assert!(is_dispatch_coord_branch("dispatch/1"));
        // A worker FORK branch is NOT a coordination branch (suffix not all-digits).
        assert!(!is_dispatch_coord_branch("dispatch/agent-a9c3"));
        assert!(!is_dispatch_coord_branch("dispatch/"));
        assert!(!is_dispatch_coord_branch("main"));
        assert!(!is_dispatch_coord_branch("edge"));
        // Anchored on the whole suffix, not a loose substring.
        assert!(!is_dispatch_coord_branch("feature/dispatch/199"));
    }

    // ---- VT-1: create-fork provision step (PHASE-04) ---------------------------

    #[test]
    fn provision_jail_policy_copies_declaration_to_named_file() {
        // A present arming declaration is copied verbatim to jail/<name>.toml under the
        // coord root — outside the fork, keyed by the harness worktree name.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let arming = root.join(ARMING_SUBPATH);
        fs::create_dir_all(&arming).unwrap();
        let body = b"extra_rw = [\"/nix/store\"]\nnetwork = false\n";
        fs::write(arming.join(ARMING_JAIL_FILE), body).unwrap();

        provision_jail_policy(root, "agent-abc123").unwrap();

        let dest = root.join(JAIL_SUBPATH).join("agent-abc123.toml");
        assert!(
            dest.exists(),
            "provisioned policy exists under jail/<name>.toml"
        );
        assert_eq!(
            fs::read(&dest).unwrap(),
            body,
            "verbatim copy of the declaration"
        );
    }

    #[test]
    fn provision_jail_policy_absent_declaration_is_noop() {
        // No declaration ⇒ no-op (⇒ pretooluse Default floor), NOT an error and no file.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        provision_jail_policy(root, "agent-abc123").expect("absent declaration is a no-op");

        assert!(
            !root.join(JAIL_SUBPATH).exists(),
            "no jail dir materialised when nothing was declared"
        );
    }

    // ---- SL-198 PHASE-01: per-worktree dispatch record + resolver -------------
    //
    // These drive the real fork path (`act_on_create`) over a temp coord tree, so
    // they stand up a live worktree the resolver keys on — the same fixture idiom the
    // e2e create-fork suite uses, in-process.

    use crate::worktree::dispatch_record::{
        DispatchRecord, RECORD_SUBPATH, ResolveFacts, ResolveRefusal, classify_resolve,
        resolve_agent,
    };

    /// `git rev-parse HEAD` at `root` (full oid). Impure test helper.
    fn head_sha(root: &Path) -> String {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("spawn git rev-parse");
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    }

    /// Commit a file inside worktree `dir` (advances that worktree's HEAD).
    fn commit_in(dir: &Path, rel: &str, contents: &str, msg: &str) {
        fs::write(dir.join(rel), contents).unwrap();
        let run = |args: &[&str]| {
            assert!(
                std::process::Command::new("git")
                    .arg("-C")
                    .arg(dir)
                    .args(args)
                    .status()
                    .expect("spawn git")
                    .success(),
                "git {args:?} failed"
            );
        };
        run(&["add", rel]);
        run(&["commit", "-q", "-m", msg]);
    }

    // VT-1: create-fork writes the DispatchRecord with all five fields; `base` is the
    // fork-time snapshot; `coord` is the coordination root.
    #[test]
    fn create_fork_writes_dispatch_record_with_all_five_fields() {
        let tmp = tempfile::tempdir().unwrap();
        let root = crate::worktree::test_helpers::init_repo(&tmp.path().join("coord"));
        let base = head_sha(&root);
        let name = "agent-abc123";

        act_on_create(
            &root,
            CreateAction::Fork {
                base: base.clone(),
                name: name.to_string(),
            },
        )
        .expect("fork + record provision");

        let raw =
            fs::read_to_string(root.join(RECORD_SUBPATH).join(format!("{name}.toml"))).unwrap();
        let record: DispatchRecord = toml::from_str(&raw).unwrap();
        assert_eq!(record.name, name, "field 1: name");
        assert_eq!(
            record.dir,
            root.join(WORKTREES_SUBDIR).join(name),
            "field 2: worker worktree dir"
        );
        assert_eq!(record.branch, format!("dispatch/{name}"), "field 3: branch");
        assert_eq!(record.base, base, "field 4: base snapshotted at fork");
        assert_eq!(
            record.coord, root,
            "field 5: coord is the coordination root"
        );
    }

    // VT-3: the resolver refusal table (unknown-agent / ambiguous-agent / stale-record)
    // PLUS a happy single-hit resolve; the agent is sanitised before any path join.
    #[test]
    fn resolve_agent_refusal_table_and_happy_single_hit() {
        let tmp = tempfile::tempdir().unwrap();
        let root = crate::worktree::test_helpers::init_repo(&tmp.path().join("coord"));
        let base = head_sha(&root);

        // unknown-agent: a name that was never forked ⇒ 0 live worktree hits.
        assert_eq!(
            resolve_agent(&root, "agent-never"),
            Err(ResolveRefusal::UnknownAgent)
        );
        // The agent is sanitised via `sanitise_name` BEFORE any path join — a traversal
        // id is rejected up front and folds to unknown-agent (never joins a hostile path).
        assert!(sanitise_name("../evil").is_err());
        assert_eq!(
            resolve_agent(&root, "../evil"),
            Err(ResolveRefusal::UnknownAgent)
        );

        // Happy path: fork a live worker ⇒ one consistent hit resolves to its record.
        let name = "agent-live";
        act_on_create(
            &root,
            CreateAction::Fork {
                base: base.clone(),
                name: name.to_string(),
            },
        )
        .expect("fork the live worker");
        let record = resolve_agent(&root, name).expect("a live, consistent worker resolves");
        assert_eq!(record.name, name);
        assert_eq!(record.branch, format!("dispatch/{name}"));
        assert_eq!(record.base, base);

        // stale-record: advance the worker HEAD past base ⇒ HEAD != record.base.
        let dir = root.join(WORKTREES_SUBDIR).join(name);
        commit_in(&dir, "work.txt", "c", "worker advances HEAD");
        assert_eq!(resolve_agent(&root, name), Err(ResolveRefusal::StaleRecord));

        // ambiguous-agent: unreachable through `worktree_for_ref` (git ≤1 worktree per
        // branch), so pin it at the pure classifier — >1 live hits refuses defensively.
        assert_eq!(
            classify_resolve(ResolveFacts {
                worktree_hits: 2,
                record: None,
                dir_exists: false,
                branch_head: None,
                base_commit: None,
            }),
            Err(ResolveRefusal::AmbiguousAgent)
        );

        // Distinct named tokens.
        assert_eq!(ResolveRefusal::UnknownAgent.token(), "unknown-agent");
        assert_eq!(ResolveRefusal::AmbiguousAgent.token(), "ambiguous-agent");
        assert_eq!(ResolveRefusal::StaleRecord.token(), "stale-record");
    }

    // ---- SL-199 PHASE-01: confined Fork trigger — consume + shared provision -----

    /// Put the coord tree at `root` on a `dispatch/<n>` coordination branch (the
    /// confined orchestrator's branch) and arm a `base` slot in the arming dir.
    fn arm_confined(root: &Path, base: &str) {
        assert!(
            std::process::Command::new("git")
                .arg("-C")
                .arg(root)
                .args(["checkout", "-q", "-b", "dispatch/199"])
                .status()
                .expect("spawn git checkout")
                .success(),
            "checkout dispatch/199"
        );
        let arming = root.join(ARMING_SUBPATH);
        fs::create_dir_all(&arming).unwrap();
        fs::write(arming.join("base"), base).unwrap();
    }

    // VT-2: a confined Fork consumes the arming base ONE-SHOT at the fork point; a
    // re-invocation with no re-arm reads no base ⇒ classifies Passthrough (no double
    // fork off a stale base). Driven through the gather→classify→act seam, not the pure fn.
    #[test]
    fn confined_fork_consumes_base_one_shot_then_passes_through() {
        let tmp = tempfile::tempdir().unwrap();
        let root = crate::worktree::test_helpers::init_repo(&tmp.path().join("coord"));
        let base = head_sha(&root);
        arm_confined(&root, &base);

        // Gather from the coord root (cwd == root): the confined trigger is live, base loaded.
        let facts = gather_fork_facts(&root, &root);
        assert!(
            facts.cwd_is_coord_root && facts.coord_in_dispatch,
            "confined trigger live at the coord root on a dispatch branch"
        );
        assert_eq!(facts.base.as_deref(), Some(base.as_str()), "base slot read");
        let action = classify_create(
            true,
            facts.cwd_is_arming,
            facts.cwd_is_coord_root,
            facts.coord_in_dispatch,
            facts.base.as_deref(),
            "agent-confined",
        )
        .expect("confined classify");
        assert_eq!(
            action,
            CreateAction::Fork {
                base: base.clone(),
                name: "agent-confined".to_string(),
            }
        );

        // Act: the fork CONSUMES the base slot at the fork point (EX-3).
        act_on_create(&root, action).expect("confined fork");
        assert!(
            !root.join(ARMING_SUBPATH).join("base").exists(),
            "base consumed one-shot at the fork point"
        );

        // Re-invoke with no re-arm: the consumed slot yields no base ⇒ benign Passthrough.
        let facts2 = gather_fork_facts(&root, &root);
        assert_eq!(facts2.base, None, "no base after consume, no re-arm");
        assert_eq!(
            classify_create(
                true,
                facts2.cwd_is_arming,
                facts2.cwd_is_coord_root,
                facts2.coord_in_dispatch,
                facts2.base.as_deref(),
                "agent-second",
            ),
            Ok(CreateAction::Passthrough {
                name: "agent-second".to_string(),
            }),
            "second spawn off a consumed base passes through — no double fork"
        );
    }

    // VT-3: BOTH Fork triggers converge on the identical CreateAction::Fork verdict and
    // therefore reach the SAME act_on_create::Fork → provision_jail_policy. The jail
    // record is written for the confined trigger exactly as for the positional.
    #[test]
    fn both_fork_triggers_reach_shared_provision_jail_policy() {
        // The confined trigger and the positional trigger classify to the SAME Fork verdict.
        let confined = classify_create(true, false, true, true, Some(SHA), "agent-x").unwrap();
        let positional = classify_create(true, true, false, false, Some(SHA), "agent-x").unwrap();
        assert_eq!(
            confined, positional,
            "both triggers converge on one Fork verdict"
        );

        // That shared Fork path provisions the jail policy — proven by acting on a coord
        // tree that declares one. The verdict is trigger-agnostic, so one act suffices.
        let decl = b"network = false\n";
        let tmp = tempfile::tempdir().unwrap();
        let root = crate::worktree::test_helpers::init_repo(&tmp.path().join("coord"));
        let base = head_sha(&root);
        let arming = root.join(ARMING_SUBPATH);
        fs::create_dir_all(&arming).unwrap();
        fs::write(arming.join(ARMING_JAIL_FILE), decl).unwrap();

        act_on_create(
            &root,
            CreateAction::Fork {
                base,
                name: "agent-x".to_string(),
            },
        )
        .expect("shared Fork path");

        let dest = root.join(JAIL_SUBPATH).join("agent-x.toml");
        assert!(dest.exists(), "shared Fork path provisions the jail policy");
        assert_eq!(
            fs::read(&dest).unwrap(),
            decl,
            "verbatim declaration for the confined trigger, exactly as positional"
        );
    }
}
