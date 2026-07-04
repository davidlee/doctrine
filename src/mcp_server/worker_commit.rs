// SPDX-License-Identifier: GPL-3.0-only
//! `worker_commit` — the gated, server-side self-commit for a jailed dispatch worker
//! (SL-198 PHASE-02, IMP-253; design §5.2/§5.4/§5.5).
//!
//! A dispatch worker runs in a linked worktree with the shared `.git` **read-only**
//! (SL-182 jail), so its raw `git commit` is walled. This MCP tool lets it self-commit
//! ONE gated commit on its own `dispatch/<agent>` branch entirely server-side: the
//! worker passes only an OPAQUE `agent` id (its worktree name) — never a path — and the
//! UNCONFINED MCP server resolves the target, runs the belts, and commits. This is the
//! deliberate bypass of the jail wall (RSK-225), so the belts ARE the security boundary
//! (INV-2: every write path is belt-gated server-side; a worker cannot skip a belt).
//!
//! **Belt order — cheap-first (design §10 X4, authoritative over §5.2's numbering):**
//! resolve → non-empty PRE-fmt delta → scope belt (two-tier) → HEAD==B → the mutating
//! `check commit` gate → stage the pre-fmt classified paths by name (post-fmt CONTENT,
//! PIN-3) → exactly one non-merge commit `C`, `C^==B`. Scoping the PRE-fmt delta before
//! the gate is why cheap-first is mandatory: the gate's `fmt` can widen the touched-path
//! set, so the belt must classify the intended delta first, and the commit then stages
//! exactly that classified set (INV-5 / F2 guard).
//!
//! **DRY (design §8 R3):** the scope hard tier reuses the SAME `.doctrine/`/`.claude/`
//! prefix consts as `classify_import` and the SAME `undeclared_paths` soft predicate, so
//! the two belt callers cannot diverge (VT-3). The resolver, gate, and delta-gather are
//! all reused PHASE-01 / existing seams — no forked copies.

use crate::dispatch_config::ForbiddenWrites;
use crate::git;
use crate::slice::SelectorIntent;
use crate::verify::{CheckKind, CheckPlan, VerificationConfig, resolve_check};
use crate::worktree::{
    CLAUDE_PREFIX, DOCTRINE_PREFIX, DispatchRecord, gather_worktree_delta_paths, resolve_agent,
};
use anyhow::{Context, bail};
use serde::Serialize;
use std::path::Path;

/// Refuse `worker_commit` with no in-scope change — never mint an empty commit (§5.5).
const EMPTY_DELTA: &str = "empty-delta";
/// Refuse: a delta path hit the HARD scope tier (`.doctrine/`/`.claude/` code floor OR
/// the `[dispatch].worker-forbidden-writes` config matcher). The offending path rides in
/// `detail` (§5.2 step 3, EX-6).
const FORBIDDEN_ZONE: &str = "forbidden-zone";
/// Refuse: the worktree `HEAD != B` (resumed / stacked) — no fast-forward past B (§5.5
/// INV-1). Normally subsumed by the resolver's `stale-record` consistency check; kept as
/// a distinct belt-and-suspenders assertion at the commit boundary (§5.2 step 4).
const NOT_AT_BASE: &str = "not-at-base";
/// Refuse: the `check commit` gate (fmt + lint/test/build) exited red; the captured
/// output rides in `detail` (§5.2 step 2).
const COMMIT_GATE_RED: &str = "commit-gate-red";

/// The tool result: a landed commit, or a typed refusal (design §5.2). Serialised
/// externally-tagged (`{"Committed": {…}}` / `{"Refused": {…}}`), matching the other MCP
/// verbs' `{"Verb": {…}}` shape. A refusal is a normal `Ok` result carrying its token —
/// NOT a JSON-RPC error — so the worker/orchestrator reads the reason structurally.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) enum WorkerCommitOutput {
    /// One non-merge commit `C` landed on `dispatch/<agent>` with `C^ == B`. `undeclared`
    /// carries the SOFT-tier src paths outside the design-target selectors (non-blocking —
    /// the orchestrator amends the selectors or rejects at import; §5.2 step 3).
    Committed {
        oid: String,
        base: String,
        undeclared: Vec<String>,
    },
    /// A belt refused; `reason` is the distinct token, `detail` the offending path /
    /// captured output (empty when the token is self-describing).
    Refused { reason: String, detail: String },
}

fn refused(reason: &str, detail: String) -> WorkerCommitOutput {
    WorkerCommitOutput::Refused {
        reason: reason.to_owned(),
        detail,
    }
}

/// The HARD scope tier: a path is a forbidden zone iff it hits the `.doctrine/` OR
/// `.claude/` code floor (the SAME consts `classify_import` rejects on — VT-3 agreement)
/// OR the `[dispatch].worker-forbidden-writes` config matcher (which additionally
/// re-applies the `.doctrine/` floor fail-closed, PIN-2). PURE.
fn is_forbidden_zone(path: &str, forbidden: &ForbiddenWrites) -> bool {
    path.starts_with(DOCTRINE_PREFIX)
        || path.starts_with(CLAUDE_PREFIX)
        || forbidden.is_forbidden(path)
}

/// PURE two-tier scope classification over the PRE-fmt delta (design §5.2 step 3, EX-6).
/// HARD tier: any forbidden-zone path fails closed with `Err(path)` → `forbidden-zone`.
/// SOFT tier: `undeclared_paths` (the SAME predicate `classify_import` uses) returns the
/// src paths outside the `design-target` selectors — these DO NOT block; they ride back
/// as `undeclared` for the orchestrator to amend/reject downstream (bounded by import
/// staying strict, PIN-4). Takes the gathered facts (delta, matcher, selectors) — no git
/// / disk here.
fn classify_scope(
    delta_paths: &[String],
    forbidden: &ForbiddenWrites,
    selectors: &[String],
) -> Result<Vec<String>, String> {
    if let Some(hit) = delta_paths.iter().find(|p| is_forbidden_zone(p, forbidden)) {
        return Err(hit.clone());
    }
    let paths: Vec<&str> = delta_paths.iter().map(String::as_str).collect();
    Ok(crate::conformance::undeclared_paths(selectors, &paths))
}

/// The commit-invariant base gate (§5.2 step 4, INV-1): the worktree `HEAD` must equal the
/// snapshotted base `B` pre-commit, so the one commit `C` satisfies `C^ == B`. PURE.
fn head_at_base(head: &str, base: &str) -> bool {
    head == base
}

/// The `check commit` gate outcome — green, or red with the captured child output.
enum GateOutcome {
    Green,
    Red(String),
}

/// Run the resolved `commit` tier (`resolve_check(Commit)`) in the worker worktree `dir`
/// (design §5.2 step 2). fmt mutates the tree FIRST, then lint/validate/test/build; a
/// red exit captures stdout+stderr for the refusal detail. The gate argv is resolved from
/// the tamper-proof PRIMARY config (`cfg`), never the worker-writable fork copy.
fn run_commit_gate(dir: &Path, cfg: &VerificationConfig) -> anyhow::Result<GateOutcome> {
    let argv = match resolve_check(cfg, CheckKind::Commit) {
        CheckPlan::Run(argv) => argv,
        CheckPlan::Empty(kind) => {
            bail!(
                "worker_commit: [verification].{} is empty — cannot run the commit gate",
                kind.key()
            )
        }
        CheckPlan::Noop(_) => {
            bail!("worker_commit: the commit gate resolved to a no-op — cannot gate the commit")
        }
    };
    let (program, rest) = argv
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("worker_commit: resolved commit-gate argv is empty"))?;
    // The gate runs in the fork worktree — a LINKED worktree, so the authored-write
    // guard (`is_linked_worktree`, marker.rs) trips on its own. DOCTRINE_WORKER-keyed
    // test skips (the `adr status` goldens) fire only if the gate child sees that env,
    // so export it here: the two worker-mode signals (guard vs test-skip) must agree.
    let output = std::process::Command::new(program)
        .args(rest)
        .current_dir(dir)
        .env("DOCTRINE_WORKER", "1")
        .output()
        .with_context(|| format!("spawning the worker commit gate: {}", argv.join(" ")))?;
    if output.status.success() {
        Ok(GateOutcome::Green)
    } else {
        let mut detail = String::from_utf8_lossy(&output.stdout).into_owned();
        detail.push_str(&String::from_utf8_lossy(&output.stderr));
        Ok(GateOutcome::Red(detail))
    }
}

/// Stage the PRE-fmt classified `paths` by name, then create exactly ONE non-merge
/// commit (design §5.2 step 6, INV-5 / PIN-3). Staging + committing BY PATH (not the
/// post-fmt working-tree diff) means a repo-wide `fmt` that normalised a pre-existing /
/// out-of-scope file cannot ride into the commit — yet the staged CONTENT is the
/// post-fmt content of these paths (the gate already ran), never a pre-fmt blob snapshot.
fn stage_and_commit(dir: &Path, message: &str, paths: &[String]) -> anyhow::Result<()> {
    let mut add_args: Vec<&str> = vec!["add", "--"];
    add_args.extend(paths.iter().map(String::as_str));
    git::git_text(dir, &add_args).context("stage the classified worker delta")?;

    let mut commit_args: Vec<&str> = vec!["commit", "-q", "-m", message, "--"];
    commit_args.extend(paths.iter().map(String::as_str));
    git::git_text(dir, &commit_args).context("create the gated worker commit")?;
    Ok(())
}

/// Resolve the slice's `design-target` selectors for the SOFT tier (design §5.2 step 3).
/// The coordination worktree (`record.coord`) is checked out on `dispatch/<slice>` and
/// carries the authored slice, so the slice id is recovered from its branch and the
/// selectors read from it. Best-effort by design: the SOFT tier never blocks, so a
/// resolution failure degrades to no selectors (every src path reported `undeclared`)
/// rather than refusing a legitimate commit.
fn design_target_selectors(record: &DispatchRecord) -> anyhow::Result<Vec<String>> {
    let branch = git::git_text(&record.coord, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let slice_id: u32 = branch
        .strip_prefix("dispatch/")
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| anyhow::anyhow!("coord HEAD `{branch}` is not a dispatch/<slice> branch"))?;
    crate::slice::selectors(&record.coord, slice_id, Some(SelectorIntent::DesignTarget))
}

/// Resolve a commit oid in `dir` (`rev-parse <rev>^{commit}`).
fn commit_oid(dir: &Path, rev: &str) -> anyhow::Result<String> {
    git::git_text(dir, &["rev-parse", &format!("{rev}^{{commit}}")])
        .with_context(|| format!("rev-parse {rev} in {}", dir.display()))
}

/// The gated server-side worker self-commit (design §5.2; belt order §10 X4). Resolves
/// the opaque `agent` to its per-worktree record, runs the cheap-first belts over the
/// worker's live PRE-fmt delta, gates with `check commit`, and lands exactly one
/// non-merge commit on `dispatch/<agent>`. `root` is the PRIMARY (unconfined) MCP root —
/// the tamper-proof source of the forbidden-write config + the gate argv (both under the
/// worker-unwritable `.doctrine/` floor). A belt refusal is a normal `Ok(Refused{…})`.
pub(crate) fn run_worker_commit(
    root: &Path,
    agent: &str,
    message: &str,
) -> anyhow::Result<WorkerCommitOutput> {
    // 1. Resolve the opaque agent-id → its per-worktree record (PHASE-01). No worker path
    //    enters resolution; a typed resolver refusal maps 1:1 to our refusal token.
    let record = match resolve_agent(root, agent) {
        Ok(record) => record,
        Err(refusal) => return Ok(refused(refusal.token(), String::new())),
    };

    // 2. Capture the PRE-fmt in-scope delta (tracked diff vs HEAD + untracked adds) — the
    //    worker cannot self-commit, so its change is the live working-tree delta at B.
    let delta = gather_worktree_delta_paths(&record.dir)?;
    if delta.is_empty() {
        return Ok(refused(EMPTY_DELTA, String::new()));
    }

    // Config from the PRIMARY root (tamper-proof; worker-unwritable — §5.3).
    let cfg = crate::dtoml::load_doctrine_toml(root)?;
    let forbidden = cfg.dispatch.forbidden_writes();
    let selectors = design_target_selectors(&record).unwrap_or_default();

    // 3. Scope belt (cheap-first, over the PRE-fmt delta): HARD refuses, SOFT rides back.
    let undeclared = match classify_scope(&delta, &forbidden, &selectors) {
        Ok(undeclared) => undeclared,
        Err(path) => return Ok(refused(FORBIDDEN_ZONE, path)),
    };

    // 4. Commit invariant — HEAD == B (pre-commit).
    let base = commit_oid(&record.dir, &record.base)?;
    let head = commit_oid(&record.dir, "HEAD")?;
    if !head_at_base(&head, &base) {
        return Ok(refused(NOT_AT_BASE, String::new()));
    }

    // 5. Gate — the mutating `check commit` (fmt first) in the worker worktree.
    match run_commit_gate(&record.dir, &cfg.verification)? {
        GateOutcome::Green => {}
        GateOutcome::Red(detail) => return Ok(refused(COMMIT_GATE_RED, detail)),
    }

    // 6. Stage the PRE-fmt classified paths (post-fmt content) + one non-merge commit.
    stage_and_commit(&record.dir, message, &delta)?;

    let oid = commit_oid(&record.dir, "HEAD")?;
    // Exactly one non-merge commit, C^ == B (INV-1) — no indexing (clippy floor).
    match git::parents(&record.dir, &oid)?.as_slice() {
        [parent] if *parent == base => {}
        [parent] => bail!("worker_commit parent {parent} != base {base} (C^ != B)"),
        other => bail!(
            "worker_commit produced a commit with {} parents (expected exactly 1)",
            other.len()
        ),
    }

    Ok(WorkerCommitOutput::Committed {
        oid,
        base,
        undeclared,
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "tests: fail-fast unwrap on fixture setup is idiomatic"
)]
mod tests {
    use super::*;
    use crate::dispatch_config::DispatchConfig;
    use crate::worktree::{Apply, Refusal, classify_import, provision_dispatch_record};
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    fn forbidden_from(lines: &[&str]) -> ForbiddenWrites {
        let cfg = DispatchConfig {
            worker_forbidden_writes: lines.iter().map(|s| (*s).to_string()).collect(),
            ..DispatchConfig::default()
        };
        cfg.forbidden_writes()
    }

    // --- VT-1 / VT-3 / VT-5: the PURE scope belt --------------------------------------

    #[test]
    fn classify_scope_hard_refuses_the_doctrine_and_claude_floors() {
        // The `.doctrine/`/`.claude/` code floor fails closed even with an EMPTY config —
        // it is not sourced from the matcher (VT-1 forbidden-zone).
        let fw = forbidden_from(&[]);
        assert_eq!(
            classify_scope(&[".doctrine/state/x".to_string()], &fw, &[]),
            Err(".doctrine/state/x".to_string())
        );
        assert_eq!(
            classify_scope(&[".claude/agents/w.md".to_string()], &fw, &[]),
            Err(".claude/agents/w.md".to_string())
        );
    }

    #[test]
    fn classify_scope_hard_refuses_a_config_forbidden_agent_def_or_flake() {
        // The config matcher hard-fences agent-defs + flake.nix (the template defaults).
        let fw = forbidden_from(&["flake.nix", "install/agents/**"]);
        assert_eq!(
            classify_scope(&["flake.nix".to_string()], &fw, &[]),
            Err("flake.nix".to_string())
        );
        assert_eq!(
            classify_scope(
                &["install/agents/claude/dispatch-worker.md".to_string()],
                &fw,
                &[]
            ),
            Err("install/agents/claude/dispatch-worker.md".to_string())
        );
    }

    #[test]
    fn classify_scope_soft_reports_undeclared_without_blocking() {
        // VT-5 (soft): a src write outside the design-target selectors COMMITS (Ok) and
        // rides back as `undeclared`; a declared path does not.
        let fw = forbidden_from(&[]);
        let selectors = vec!["src/allowed.rs".to_string()];
        let delta = vec!["src/allowed.rs".to_string(), "src/other.rs".to_string()];
        assert_eq!(
            classify_scope(&delta, &fw, &selectors),
            Ok(vec!["src/other.rs".to_string()])
        );
    }

    #[test]
    fn classify_scope_forbidden_wins_even_alongside_an_undeclared_path() {
        // VT-5 (hard still wins): a forbidden-zone write in the same call hard-refuses,
        // regardless of a co-present undeclared src path.
        let fw = forbidden_from(&[]);
        let selectors = vec!["src/allowed.rs".to_string()];
        let delta = vec!["src/other.rs".to_string(), ".doctrine/x".to_string()];
        assert_eq!(
            classify_scope(&delta, &fw, &selectors),
            Err(".doctrine/x".to_string())
        );
    }

    #[test]
    fn worker_commit_and_classify_import_agree_on_the_hard_verdict() {
        // VT-3: both belts REJECT the same `.doctrine/`/`.claude/` change-set and ACCEPT
        // the same benign src set — the shared consts + `undeclared_paths` predicate, not
        // a forked copy. `classify_import`'s stationary-head preconds are all true here so
        // only the scope verdict differs.
        let fw = forbidden_from(&[]);
        for hard in [".doctrine/state/x", ".claude/agents/w.md"] {
            let delta = vec![hard.to_string()];
            assert!(
                classify_scope(&delta, &fw, &[]).is_err(),
                "worker_commit rejects {hard}"
            );
            assert!(
                classify_import(true, true, true, &delta, &[]).is_err(),
                "classify_import rejects {hard}"
            );
        }
        // A benign src path: both accept.
        let benign = vec!["src/lib.rs".to_string()];
        assert!(classify_scope(&benign, &fw, &[]).is_ok());
        assert_eq!(
            classify_import(true, true, true, &benign, &[]),
            Ok(Apply::Ok)
        );
        // And the specific import token for a `.doctrine/` path is DoctrineTouch — the
        // hard verdict worker_commit mirrors as `forbidden-zone`.
        assert_eq!(
            classify_import(true, true, true, &[".doctrine/x".to_string()], &[]),
            Err(Refusal::DoctrineTouch)
        );
    }

    #[test]
    fn head_at_base_is_exact_equality() {
        assert!(head_at_base("abc123", "abc123"));
        assert!(!head_at_base("abc123", "def456"));
    }

    // --- integration: a live per-worktree record + a real linked worktree -------------

    fn git_run(dir: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// Stand up a primary repo at base B with a linked worker worktree on
    /// `dispatch/<name>` at `<coord>/.worktrees/<name>`, plus its per-worktree record.
    /// `commit_override` is written into the primary `[verification].commit` (the gate).
    /// Returns `(tmp, primary, wt, agent, base)`.
    fn worker_fixture(
        commit_override: &str,
        seed_files: &[(&str, &str)],
    ) -> (tempfile::TempDir, PathBuf, PathBuf, String, String) {
        let tmp = tempfile::tempdir().unwrap();
        let primary = fs::canonicalize(tmp.path()).unwrap().join("primary");
        fs::create_dir_all(&primary).unwrap();
        git_run(&primary, &["init", "-q", "-b", "main"]);
        git_run(&primary, &["config", "user.email", "t@t"]);
        git_run(&primary, &["config", "user.name", "t"]);
        // Base tree.
        for (rel, body) in seed_files {
            let path = primary.join(rel);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, body).unwrap();
        }
        fs::write(primary.join("seed"), "base\n").unwrap();
        git_run(&primary, &["add", "-A"]);
        git_run(&primary, &["commit", "-q", "-m", "base"]);
        let base = git_run(&primary, &["rev-parse", "HEAD^{commit}"]);

        // The tamper-proof gate config lives on the PRIMARY (the MCP root).
        fs::create_dir_all(primary.join(".doctrine")).unwrap();
        fs::write(
            primary.join(".doctrine/doctrine.toml"),
            format!("[verification]\ncommit = {commit_override}\n"),
        )
        .unwrap();

        // The worker worktree + its per-worktree record.
        let agent = "wk1".to_string();
        let coord = fs::canonicalize(tmp.path()).unwrap().join("coord");
        let wt = coord.join(".worktrees").join(&agent);
        fs::create_dir_all(coord.join(".worktrees")).unwrap();
        git_run(
            &primary,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                &format!("dispatch/{agent}"),
                wt.to_str().unwrap(),
                &base,
            ],
        );
        let wt = fs::canonicalize(&wt).unwrap();
        provision_dispatch_record(&coord, &agent, &base, &wt, &format!("dispatch/{agent}"))
            .unwrap();

        (tmp, primary, wt, agent, base)
    }

    #[test]
    fn worker_commit_unknown_agent_refuses() {
        // VT-1: the resolver refusal token surfaces as our refusal.
        let (_tmp, primary, _wt, _agent, _base) = worker_fixture("[\"true\"]", &[]);
        let out = run_worker_commit(&primary, "nosuchagent", "msg").unwrap();
        assert_eq!(
            out,
            refused("unknown-agent", String::new()),
            "an unresolvable agent refuses unknown-agent"
        );
    }

    #[test]
    fn worker_commit_empty_delta_refuses() {
        // VT-1: no in-scope change ⇒ empty-delta (no empty commit).
        let (_tmp, primary, _wt, agent, _base) = worker_fixture("[\"true\"]", &[]);
        let out = run_worker_commit(&primary, &agent, "msg").unwrap();
        assert_eq!(out, refused(EMPTY_DELTA, String::new()));
    }

    #[test]
    fn worker_commit_gate_red_refuses_and_leaves_head_at_base() {
        // VT-1: a red gate refuses commit-gate-red; the branch stays at B (no commit).
        let (_tmp, primary, wt, agent, base) = worker_fixture("[\"false\"]", &[]);
        fs::write(wt.join("seed"), "worker change\n").unwrap();
        let out = run_worker_commit(&primary, &agent, "msg").unwrap();
        match out {
            WorkerCommitOutput::Refused { reason, .. } => assert_eq!(reason, COMMIT_GATE_RED),
            other => panic!("expected commit-gate-red, got {other:?}"),
        }
        assert_eq!(git_run(&wt, &["rev-parse", "HEAD^{commit}"]), base);
    }

    #[test]
    fn worker_commit_gate_child_sees_doctrine_worker_env() {
        // The gate runs cargo test in a LINKED worktree; the authored-write guard trips
        // on `is_linked_worktree` alone, so DOCTRINE_WORKER-keyed test skips (the adr_status
        // goldens) only fire if the gate child inherits that env. A gate that passes iff
        // DOCTRINE_WORKER==1 lands the commit only when the env is exported.
        let gate = r#"["sh", "-c", "test \"$DOCTRINE_WORKER\" = 1"]"#;
        let (_tmp, primary, wt, agent, base) = worker_fixture(gate, &[]);
        fs::write(wt.join("seed"), "worker change\n").unwrap();
        let out = run_worker_commit(&primary, &agent, "msg").unwrap();
        match out {
            WorkerCommitOutput::Committed { base: out_base, .. } => assert_eq!(out_base, base),
            other => panic!("gate child must see DOCTRINE_WORKER=1 and pass; got {other:?}"),
        }
    }

    #[test]
    fn worker_commit_forbidden_zone_refuses() {
        // VT-1: a `.doctrine/` write hard-refuses forbidden-zone (before the gate runs).
        let (_tmp, primary, wt, agent, base) = worker_fixture("[\"true\"]", &[]);
        fs::create_dir_all(wt.join(".doctrine/state")).unwrap();
        fs::write(wt.join(".doctrine/state/x"), "sneaky\n").unwrap();
        let out = run_worker_commit(&primary, &agent, "msg").unwrap();
        match out {
            WorkerCommitOutput::Refused { reason, detail } => {
                assert_eq!(reason, FORBIDDEN_ZONE);
                assert!(
                    detail.contains(".doctrine/state/x"),
                    "names the path: {detail}"
                );
            }
            other => panic!("expected forbidden-zone, got {other:?}"),
        }
        assert_eq!(git_run(&wt, &["rev-parse", "HEAD^{commit}"]), base);
    }

    #[test]
    fn worker_commit_happy_path_lands_one_non_merge_commit() {
        // VT-2: an in-scope, gate-green tree → one non-merge commit, C^==B, parents==1.
        let (_tmp, primary, wt, agent, base) = worker_fixture("[\"true\"]", &[]);
        fs::write(wt.join("seed"), "worker change\n").unwrap();
        let out = run_worker_commit(&primary, &agent, "the message").unwrap();
        let (oid, out_base) = match out {
            WorkerCommitOutput::Committed { oid, base, .. } => (oid, base),
            other => panic!("expected Committed, got {other:?}"),
        };
        assert_eq!(out_base, base, "returned base == B");
        // The branch advanced B → C, exactly one non-merge commit, C^ == B.
        assert_eq!(git_run(&wt, &["rev-parse", "HEAD^{commit}"]), oid);
        let parents = git_run(&wt, &["rev-list", "--parents", "-n", "1", &oid]);
        let cols: Vec<&str> = parents.split_whitespace().collect();
        assert_eq!(cols.len(), 2, "exactly one parent: {parents}");
        assert_eq!(cols[1], base, "C^ == B");
        assert_eq!(git_run(&wt, &["log", "-1", "--format=%s"]), "the message");
    }

    #[test]
    fn worker_commit_stages_only_the_in_scope_path_after_the_gate_fmt() {
        // VT-4 (F2 guard): the gate reformats a pre-existing out-of-scope file; the
        // commit contains ONLY the in-scope path the worker touched, not the file the
        // gate rewrote. The "fmt" is simulated by a gate that mutates outofscope.txt.
        let (_tmp, primary, wt, agent, _base) = worker_fixture(
            "[\"sh\", \"-c\", \"printf reformatted > outofscope.txt\"]",
            &[
                ("outofscope.txt", "misformatted\n"),
                ("src/feature.rs", "fn f(){}\n"),
            ],
        );
        // Worker touches only the in-scope file.
        fs::write(wt.join("src/feature.rs"), "fn f() {}\n").unwrap();
        let out = run_worker_commit(&primary, &agent, "in-scope only").unwrap();
        let oid = match out {
            WorkerCommitOutput::Committed { oid, .. } => oid,
            other => panic!("expected Committed, got {other:?}"),
        };
        let touched = git_run(&wt, &["show", "--name-only", "--format=", &oid]);
        let names: Vec<&str> = touched.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(
            names,
            vec!["src/feature.rs"],
            "commit contains ONLY the in-scope path, not the gate-reformatted file"
        );
    }

    #[test]
    fn worker_commit_soft_undeclared_still_commits() {
        // VT-5 (shell): a src write outside the (here empty) design-target selectors
        // COMMITS and returns the path as `undeclared` — never dropped, never blocked.
        let (_tmp, primary, wt, agent, _base) = worker_fixture("[\"true\"]", &[]);
        fs::write(wt.join("seed"), "worker change\n").unwrap();
        let out = run_worker_commit(&primary, &agent, "msg").unwrap();
        match out {
            WorkerCommitOutput::Committed { undeclared, .. } => {
                assert!(
                    undeclared.contains(&"seed".to_string()),
                    "the out-of-selector src path is reported undeclared: {undeclared:?}"
                );
            }
            other => panic!("expected Committed, got {other:?}"),
        }
    }
}
