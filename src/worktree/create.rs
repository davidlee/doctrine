// SPDX-License-Identifier: GPL-3.0-only
//! worktree create-fork — the claude `WorktreeCreate` hook verb (SL-152).
//!
//! Since SL-254 PHASE-06 this verb provisions BENIGN harness-created worktrees and
//! nothing else. The dispatch Fork arm (the arming dir, its `base`/`slice`/`phase`
//! slots, the per-arming jail declaration and the claim→bind→act spawn) is deleted
//! with the claude dispatch arm it existed for: `doctrine worktree fork --worker` is
//! now the sole worker-fork writer (`D1`). The hook entry STAYS — a `WorktreeCreate`
//! with no verb behind it would mint an unprovisioned worktree, and a surviving Fork
//! arm would mint an unmarked, unconfined one.
//!
//! Mirror of the old subagent stamp: a PURE classifier ([`classify_create`] +
//! [`sanitise_name`]) decides Passthrough-vs-Refuse from already-resolved facts (no
//! git / disk / env / clock in the classifier — ADR-001 leaf, CLAUDE.md
//! pure/imperative split), and an in-file impure SHELL ([`run_create_fork`]) gathers
//! those facts — the payload cwd realpath and the `git -C cwd --show-toplevel`
//! coord-tree root (NOT `primary_worktree`: create-fork fires in the PARENT before
//! the tree exists, G2/I5) — and ACTS ([`act_on_create`]). The shell reads
//! `{cwd, name}` JSON on stdin, prints the created absolute path ALONE on stdout
//! (D11/G1), routes everything else to stderr, and fails closed (non-zero exit, never
//! a panic) on any malformed input or failure.

use super::fork::remove_worktree_dir;
use super::provision::run_provision;
use crate::git;
use anyhow::{Context, bail};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

/// Verdict of the PURE create classifier. One action survives the SL-254 arm
/// collapse: a benign detached worktree at `<name>`, provisioned by the shared
/// copier and NOT worker-marked. Kept as an enum rather than folded away because the
/// classifier's contract is verdict-or-typed-refusal, and the refusal set is what the
/// goldens assert. The validated `name` slug is carried in the verdict (D-P2) so the
/// shell does not re-sanitise it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CreateAction {
    /// A benign detached worktree at `<name>`, provisioned by the same copier as
    /// every other tree, NOT worker-marked.
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
}

impl CreateRefusal {
    /// The distinct named token each refusal fails closed with.
    pub(crate) fn token(self) -> &'static str {
        match self {
            CreateRefusal::MissingCwd => "missing-cwd",
            CreateRefusal::BadName(_) => "bad-name",
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

/// PURE create classifier (no git / disk / env / clock — ADR-001 leaf). Takes the
/// gathered, already-resolved FACTS and returns the verdict; the shell resolves the
/// cwd realpath (impure), then calls this.
///
/// Since SL-254 PHASE-06 there is exactly ONE verdict — every harness-created
/// worktree is a benign Passthrough. The positional (arming-dir) and confined
/// (coord-root-on-`dispatch/<n>`) Fork triggers are deleted with the claude dispatch
/// arm, and with them the `missing-base` / `bad-base` refusals that only an armed
/// trigger could reach. What remains is the pair of refusals the benign path always
/// owed: an unresolvable cwd, and a ref/path-unsafe name.
///
/// * `cwd_resolved` — the payload carried a `cwd` that resolved (canonicalised) on disk.
/// * `name` — the payload `name`, validated here via [`sanitise_name`].
///
/// Precond order: cwd-resolution → name-validity, so a missing cwd names itself first.
pub(crate) fn classify_create(
    cwd_resolved: bool,
    name: &str,
) -> Result<CreateAction, CreateRefusal> {
    if !cwd_resolved {
        return Err(CreateRefusal::MissingCwd);
    }
    let slug = sanitise_name(name).map_err(CreateRefusal::BadName)?;
    Ok(CreateAction::Passthrough { name: slug })
}

// ---------------------------------------------------------------------------
// Imperative shell — gather → classify → act (impure: stdin, git, disk).
// ---------------------------------------------------------------------------

/// Where every created tree lives under the coord-tree root: `<root>/.worktrees/<name>`.
/// `pub(crate)` so the dispatch-record resolver recovers the coord root by stripping
/// this same layout — one owner of the `.worktrees/<name>` shape, no re-spell.
pub(crate) const WORKTREES_SUBDIR: &str = ".worktrees";

/// The `WorktreeCreate` payload subset we read (tolerate extra fields). JSON on
/// stdin: `{ "cwd": "<orchestrator cwd at spawn>", "name": "<unique slug>" }`. The
/// payload is THIN by construction (probe, design §10): no `agent_type`, no base, no
/// target path — the cwd is only ever used to resolve the source tree to provision from.
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
/// Returns the CANONICALISED created dir.
///
/// * `Passthrough` — a benign DETACHED tree at the coord tree's HEAD, provisioned by
///   the SAME copier ([`run_provision`], source = `root` not the fresh tree — the
///   ISS-011 trap), NOT worker-marked. Owns its own `dir`-collision refusal (no
///   branch to check), and COMPENSATES (G3) — removes the half-created tree before
///   the fail-closed bail so an abort leaks nothing.
fn act_on_create(root: &Path, action: CreateAction) -> anyhow::Result<PathBuf> {
    match action {
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

    match classify_create(cwd_resolved, &name) {
        Err(refusal) => {
            // Stable token (`bad-name`), plus the specific sanitiser reason in
            // parens for hook debugging (e.g. `create-refused: bad-name (whitespace)`).
            let line = match refusal {
                CreateRefusal::BadName(reason) => {
                    format!("create-refused: {} ({})", refusal.token(), reason.token())
                }
                CreateRefusal::MissingCwd => format!("create-refused: {}", refusal.token()),
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
            let created = act_on_create(&root, action)?;
            // stdout = EXACTLY the created path, one line, nothing else (G1/D11).
            writeln!(io::stdout(), "{}", created.display())?;
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
    //
    // Retargeted at SL-254 PHASE-06: the arming-dir / confined Fork triggers and the
    // `missing-base` / `bad-base` refusals they alone could reach are deleted with the
    // claude dispatch arm. What survives is the surviving contract — an unresolvable
    // cwd names itself first, a bad name names itself second, and everything else is
    // a benign Passthrough.

    #[test]
    fn missing_cwd_refuses_first_regardless_of_everything_else() {
        // cwd unresolved ⇒ missing-cwd even with a perfectly valid name.
        assert_eq!(
            classify_create(false, "agent-abc123"),
            Err(CreateRefusal::MissingCwd)
        );
        assert_eq!(classify_create(false, ""), Err(CreateRefusal::MissingCwd));
        assert_eq!(CreateRefusal::MissingCwd.token(), "missing-cwd");
    }

    #[test]
    fn bad_name_refuses_with_the_sanitiser_reason() {
        assert_eq!(
            classify_create(true, "a/b"),
            Err(CreateRefusal::BadName(NameRefusal::Slash))
        );
        assert_eq!(
            classify_create(true, ""),
            Err(CreateRefusal::BadName(NameRefusal::Empty))
        );
        assert_eq!(
            CreateRefusal::BadName(NameRefusal::Slash).token(),
            "bad-name"
        );
    }

    #[test]
    fn a_resolved_cwd_with_a_valid_name_passes_through() {
        // The ONE surviving verdict: every harness-created worktree is benign, and the
        // validated slug is carried in it (D-P2) so the shell never re-sanitises.
        assert_eq!(
            classify_create(true, "agent-abc123"),
            Ok(CreateAction::Passthrough {
                name: "agent-abc123".to_string(),
            })
        );
        assert_eq!(
            classify_create(true, "bold-oak-a3f2"),
            Ok(CreateAction::Passthrough {
                name: "bold-oak-a3f2".to_string(),
            })
        );
    }

    // ---- SL-198 PHASE-01: per-worktree dispatch record + resolver -------------
    //
    // These drive the real worker-fork path over a temp coord tree, so they stand up a
    // live worktree the resolver keys on. Retargeted at SL-254 PHASE-06 from
    // `act_on_create`'s deleted Fork arm onto `fork_core` + `bind_dispatch_record` —
    // the same claim→bind→act core, reached the way `worktree fork --worker` (now the
    // sole worker-fork writer) reaches it.

    use crate::worktree::dispatch_record::{
        DispatchRecord, ForkBinding, ForkExpect, RECORD_SUBPATH, ResolveFacts, ResolveRefusal,
        classify_resolve, resolve_agent,
    };

    /// Create a worker fork exactly as `worktree fork --worker` does: `fork_core` with
    /// a bind closure that writes the per-worktree dispatch record. Returns the fork dir.
    fn fork_worker(
        root: &Path,
        base: &str,
        name: &str,
        binding: Option<&ForkBinding>,
    ) -> anyhow::Result<PathBuf> {
        let dir = root.join(WORKTREES_SUBDIR).join(name);
        let branch = format!("dispatch/{name}");
        let mut bind = || -> anyhow::Result<()> {
            super::super::dispatch_record::bind_dispatch_record(
                root, name, base, &dir, &branch, binding,
            )
        };
        super::super::fork::fork_core(root, base, &branch, &dir, &mut bind)?;
        Ok(dir)
    }

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

    // VT-1: the worker fork writes the DispatchRecord with all five fields; `base` is
    // the fork-time snapshot; `coord` is the coordination root.
    #[test]
    fn worker_fork_writes_dispatch_record_with_all_five_fields() {
        let tmp = tempfile::tempdir().unwrap();
        let root = crate::worktree::test_helpers::init_repo(&tmp.path().join("coord"));
        let base = head_sha(&root);
        let name = "agent-abc123";

        fork_worker(&root, &base, name, None).expect("fork + record provision");

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
            resolve_agent(&root, "agent-never", ForkExpect::AtBase),
            Err(ResolveRefusal::UnknownAgent)
        );
        // The agent is sanitised via `sanitise_name` BEFORE any path join — a traversal
        // id is rejected up front and folds to unknown-agent (never joins a hostile path).
        assert!(sanitise_name("../evil").is_err());
        assert_eq!(
            resolve_agent(&root, "../evil", ForkExpect::AtBase),
            Err(ResolveRefusal::UnknownAgent)
        );

        // Happy path: fork a live worker ⇒ one consistent hit resolves to its record.
        let name = "agent-live";
        fork_worker(&root, &base, name, None).expect("fork the live worker");
        let record = resolve_agent(&root, name, ForkExpect::AtBase)
            .expect("a live, consistent worker resolves");
        assert_eq!(record.name, name);
        assert_eq!(record.branch, format!("dispatch/{name}"));
        assert_eq!(record.base, base);

        // stale-record: advance the worker HEAD past base ⇒ HEAD != record.base.
        let dir = root.join(WORKTREES_SUBDIR).join(name);
        commit_in(&dir, "work.txt", "c", "worker advances HEAD");
        assert_eq!(
            resolve_agent(&root, name, ForkExpect::AtBase),
            Err(ResolveRefusal::StaleRecord)
        );

        // ambiguous-agent: unreachable through `worktree_for_ref` (git ≤1 worktree per
        // branch), so pin it at the pure classifier — >1 live hits refuses defensively.
        assert_eq!(
            classify_resolve(ResolveFacts {
                worktree_hits: 2,
                record: None,
                dir_exists: false,
                branch_head: None,
                base_commit: None,
                head_parents: Vec::new(),
                expect: ForkExpect::AtBase,
            }),
            Err(ResolveRefusal::AmbiguousAgent)
        );

        // Distinct named tokens.
        assert_eq!(ResolveRefusal::UnknownAgent.token(), "unknown-agent");
        assert_eq!(ResolveRefusal::AmbiguousAgent.token(), "ambiguous-agent");
        assert_eq!(ResolveRefusal::StaleRecord.token(), "stale-record");
    }

    // ==========================================================================
    // SL-228 PHASE-04 — the branch-as-claim fork sequence (VT-1 / VT-5 / VT-6).
    // ==========================================================================

    /// `git -C <root> <args>`, returning success (never asserting) — for probes.
    fn git_ok(root: &Path, args: &[&str]) -> bool {
        std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .status()
            .expect("spawn git")
            .success()
    }

    fn branch_exists(root: &Path, branch: &str) -> bool {
        git_ok(root, &["rev-parse", "--verify", "--quiet", branch])
    }

    // VT-5: the ORDER is claim → bind → act. Proven from INSIDE the bind step, which is
    // the only place that can observe the intermediate state: when bind runs, the branch
    // claim must already exist and the worktree must NOT. That is what makes "a live
    // fork implies its binding" true by construction rather than by timing.
    #[test]
    fn bind_runs_after_the_claim_and_before_the_worktree_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let root = crate::worktree::test_helpers::init_repo(&tmp.path().join("coord"));
        let base = head_sha(&root);
        let dir = root.join(WORKTREES_SUBDIR).join("agent-order");
        let branch = "dispatch/agent-order";

        let mut observed = None;
        let mut bind = || -> anyhow::Result<()> {
            observed = Some((branch_exists(&root, branch), dir.exists()));
            Ok(())
        };
        super::super::fork::fork_core(&root, &base, branch, &dir, &mut bind)
            .expect("the fork completes");

        assert_eq!(
            observed,
            Some((true, false)),
            "at bind time the CLAIM is held and the worktree does not exist yet"
        );
        // ...and once the act completes, both exist.
        assert!(branch_exists(&root, branch) && dir.exists());
    }

    // VT-5: a failure AFTER the claim reverses the claim too — no branch residue, no
    // worktree, and no record left behind by a rolled-back fork.
    #[test]
    fn a_failed_bind_rolls_the_claim_back_leaving_no_residue() {
        let tmp = tempfile::tempdir().unwrap();
        let root = crate::worktree::test_helpers::init_repo(&tmp.path().join("coord"));
        let base = head_sha(&root);
        let name = "agent-rollback";
        let dir = root.join(WORKTREES_SUBDIR).join(name);
        let branch = format!("dispatch/{name}");

        // A bind that writes its record and THEN fails — the worst case for residue.
        let mut bind = || -> anyhow::Result<()> {
            super::super::dispatch_record::bind_dispatch_record(
                &root, name, &base, &dir, &branch, None,
            )?;
            anyhow::bail!("bind blew up")
        };
        let err = super::super::fork::fork_core(&root, &base, &branch, &dir, &mut bind)
            .expect_err("a failed bind fails the fork");
        assert!(
            format!("{err:#}").contains("bind blew up"),
            "the original cause survives: {err:#}"
        );

        assert!(
            !branch_exists(&root, &branch),
            "the CLAIM is reversed — no branch residue from a rolled-back fork"
        );
        assert!(!dir.exists(), "no worktree dir");
        assert!(
            !root
                .join(crate::worktree::dispatch_record::RECORD_SUBPATH)
                .join(format!("{name}.toml"))
                .exists(),
            "the bind is reversed too — no record survives a rolled-back fork"
        );
    }

    // VT-1: TWO CONCURRENT same-name spawns ⇒ exactly ONE wins. Real threads against a
    // real git repo — a mocked claim would prove nothing about the property the design
    // bought. The loser refuses AT THE CLAIM, and the winner's binding is exactly what
    // a solo spawn would have written (never cross-paired).
    //
    // Retargeted at SL-254 PHASE-06 from `act_on_create`'s deleted Fork arm onto the
    // worker-fork path that survives. NOTE the strength this loses and why: the deleted
    // arm ALSO held a per-name flock across the window (`claim_lock::acquire`), and
    // `worktree fork --worker` never did. What still makes exactly-one-winner true is
    // the atomic branch-ref claim inside `fork_core`, which is what this now pins.
    #[test]
    fn two_concurrent_same_name_spawns_leave_exactly_one_winner() {
        let tmp = tempfile::tempdir().unwrap();
        let root = crate::worktree::test_helpers::init_repo(&tmp.path().join("coord"));
        let base = head_sha(&root);
        let name = "agent-race";
        let binding = ForkBinding {
            slice: 228,
            phase: "PHASE-04".to_string(),
        };

        let outcomes: Vec<anyhow::Result<PathBuf>> = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..2)
                .map(|_| {
                    let root = root.clone();
                    let base = base.clone();
                    let binding = binding.clone();
                    scope.spawn(move || fork_worker(&root, &base, name, Some(&binding)))
                })
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });

        let winners = outcomes.iter().filter(|o| o.is_ok()).count();
        assert_eq!(winners, 1, "exactly one concurrent same-name spawn wins");
        let loser = outcomes
            .iter()
            .find_map(|o| o.as_ref().err())
            .expect("one loser");
        assert!(
            format!("{loser:#}").contains("fork-refused"),
            "the loser refuses AT THE CLAIM: {loser:#}"
        );

        // The winner's binding is the solo-spawn binding, whole and uncrossed.
        let record = crate::worktree::dispatch_record::resolve_agent(
            &root,
            name,
            crate::worktree::dispatch_record::ForkExpect::AtBase,
        )
        .expect("the winner resolves");
        assert_eq!(record.base, base);
        assert_eq!(record.branch, format!("dispatch/{name}"));
        assert_eq!(
            record.binding(),
            Some(crate::worktree::dispatch_record::ForkBinding {
                slice: 228,
                phase: "PHASE-04".to_string(),
            })
        );
    }

    // VT-1: a crash between the claim/bind and the act leaves INERT residue (a branch,
    // and maybe a record, with no worktree). A same-name retry refuses AT THE CLAIM, the
    // refusal prescribes the gc sweep, and after the sweep the name is re-claimable.
    #[test]
    fn crash_residue_is_inert_refuses_a_retry_and_sweeps_clean() {
        let tmp = tempfile::tempdir().unwrap();
        let root = crate::worktree::test_helpers::init_repo(&tmp.path().join("coord"));
        let base = head_sha(&root);
        let name = "agent-residue";
        let branch = format!("dispatch/{name}");
        let dir = root.join(WORKTREES_SUBDIR).join(name);

        // The residue a crash between bind and act leaves: branch ⊕ record, NO worktree.
        assert!(git_ok(&root, &["branch", &branch, &base]));
        super::super::dispatch_record::bind_dispatch_record(
            &root, name, &base, &dir, &branch, None,
        )
        .unwrap();
        assert!(!dir.exists(), "the residue has no worktree — it is inert");

        // A same-name retry refuses at the claim rather than adopting the branch.
        let err = fork_worker(&root, &base, name, None)
            .expect_err("a same-name retry refuses at the claim");
        assert!(
            format!("{err:#}").contains("fork-refused: branch"),
            "refused at the claim: {err:#}"
        );

        // gc sweeps the residue (--force: an unlanded residue is not a landed fork).
        super::super::gc::run_gc(Some(root.clone()), &branch, None, true, false)
            .expect("gc sweeps the residue");
        assert!(!branch_exists(&root, &branch), "branch residue swept");

        // ...and the name is re-claimable afterwards.
        fork_worker(&root, &base, name, None).expect("the swept name is re-claimable");
    }
}
