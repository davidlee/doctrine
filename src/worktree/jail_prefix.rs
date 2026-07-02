// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine worktree jail-prefix` — SL-185 PHASE-02 (command tier, ADR-001).
//!
//! Emits a confinement wrap PREFIX (a NUL-delimited argv terminating in `--`) to
//! a `--out` file, so an orchestrator spawn script can do
//! `timeout "${PREFIX[@]}" <harness-cmd>` (design §1, G1). This is the subprocess
//! (pi) arm's analog of the claude `PreToolUse` hook: the orchestrator, not a
//! per-Bash hook, applies ONE whole-process wrap around the harness exec.
//!
//! ## Pure / impure split (ADR-001)
//! The argv is NOT re-authored here — `Jailer::wrap_argv` (leaf) already returns
//! the wrap prefix terminating in `FLAG_ARG_SEP` (`--`); this command tier only
//! resolves a backend, canonicalizes the impure inputs the leaf cannot touch, and
//! writes the emission. Its OWN backend resolution (NOT `probe_backend`, whose
//! macOS branch reads the per-arming disk policy this arm deliberately rejects —
//! AR-4).
//!
//! ## Fail-closed (AR-1)
//! Any resolve / validate / materialize / write error ⇒ nonzero exit + reason on
//! stderr + **no `--out` file** (the full argv is built first, then written once —
//! never a partial/empty file the caller could mistake for success). The `--out`
//! file seam is load-bearing: a `$(…)` capture would strip NUL and swallow the
//! child's exit status (the AR-1 hole).
//!
//! ## XR-1 — inline `--extra-rw` is raw shell input
//! Unlike the claude arm's pre-canonicalized disk policy, an inline `--extra-rw`
//! grant is raw orchestrator-shell input. `validate_policy` is a PURE lexical
//! ancestor test whose D-canon precondition is that its paths are already
//! symlink-resolved (`jail.rs`). So each grant is `env.realpath`'d (fail-closed if
//! it does not exist) BEFORE `validate_policy` runs — else a `..`/symlinked grant
//! would pass the lexical check, then resolve to `/` / a worktree ancestor / `.git`
//! and be bound rw (sandbox-widening). Canonicalize at the source; do NOT reorder
//! the behaviour-preserved resolver core.

use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use super::jail::{Backend, JailPolicy, RealEnv, ResolveEnv, select_jailer};
#[cfg(not(target_os = "macos"))]
use super::jail::validate_policy;
#[cfg(not(target_os = "macos"))]
use super::pretooluse::{REASON_NO_BWRAP, have_bwrap};
#[cfg(target_os = "macos")]
use super::jail::{resolve_with_policy, write_seatbelt_profile};
#[cfg(target_os = "macos")]
use super::pretooluse::REASON_PROFILE_WRITE_FAILED;

// ---- vocabulary (STD-001: single-sourced, no inline literals) ------------------
/// The command name, prefixed onto every fail-closed reason surfaced to stderr.
const CMD_JAIL_PREFIX: &str = "jail-prefix";
/// The argv record delimiter written to `--out` (AR-1 / EX-4): tokens are joined
/// by a NUL byte so non-UTF-8 paths survive and `mapfile -d ''` round-trips them.
const ARGV_NUL_DELIM: u8 = 0;
/// An inline `--extra-rw` that does not resolve on disk (XR-1 existence obligation).
const REASON_EXTRA_RW_UNRESOLVABLE: &str = "extra-rw-unresolvable";
/// The `--dir` worktree does not resolve on disk.
#[cfg(not(target_os = "macos"))]
const REASON_DIR_UNRESOLVABLE: &str = "dir-unresolvable";
/// An inline `--extra-rw` that resolves to `/`, a worktree ancestor, or `.git`
/// (XR-1 — `validate_policy` rejected it after canonicalization).
#[cfg(not(target_os = "macos"))]
const REASON_UNSAFE_EXTRA_RW: &str = "unsafe-extra-rw";
/// Structurally-unreachable defensive stem (`select_jailer` yields `None` only for
/// `Backend::Deny`, never constructed here). Fail-closed, never a panic.
const REASON_NO_BACKEND: &str = "no-jail-backend";
/// The macOS arm requires `--main-root` (validate + policy base); absent ⇒ fail-closed.
#[cfg(target_os = "macos")]
const REASON_MAIN_ROOT_REQUIRED: &str = "main-root-required";

/// `doctrine worktree jail-prefix` — resolve a confinement backend for `dir` under
/// the inline policy, emit its wrap prefix NUL-delimited to `out`. Impure command
/// tier; cfg-split per host arm (D2). Fail-closed: `Err` ⇒ nonzero exit + no `out`.
pub(crate) fn run_jail_prefix(
    dir: &Path,
    main_root: Option<&Path>,
    out: &Path,
    network: bool,
    extra_rw: &[PathBuf],
) -> anyhow::Result<()> {
    // The sole impurity seam (realpath / topology). `main_root` is consumed only by
    // the macOS arm; the Linux arm ignores it.
    let env = RealEnv {
        main_root: main_root.map(Path::to_path_buf).unwrap_or_default(),
    };

    // XR-1: realpath each inline grant BEFORE any validation (the D-canon precondition
    // validate_policy assumes). A non-existent grant fails closed rather than binding a
    // phantom path.
    let mut extra_rw_canon = Vec::with_capacity(extra_rw.len());
    for grant in extra_rw {
        let real = env.realpath(grant).map_err(|_e| {
            anyhow::anyhow!(
                "{CMD_JAIL_PREFIX}: {REASON_EXTRA_RW_UNRESOLVABLE}: {}",
                grant.display()
            )
        })?;
        extra_rw_canon.push(real);
    }
    let policy = JailPolicy {
        extra_rw: extra_rw_canon,
        network,
    };

    // Build the FULL wrap prefix first (its own backend resolution — NOT probe_backend);
    // only a fully-resolved success reaches the write.
    let prefix = resolve_prefix(dir, main_root, &policy, &env)?;
    write_prefix(out, &prefix)
}

/// Linux (bwrap) arm — THIS phase, fully compiled + tested here (D2 cfg-rot
/// mitigation). Canonicalize `dir`, reject a dangerous inline grant (XR-1), then —
/// iff bwrap is on `PATH` (AR-4: the factored presence helper, never a duplicate
/// scan) — hand the leaf's `bwrap_argv` prefix back. bwrap absent ⇒ fail-closed.
#[cfg(not(target_os = "macos"))]
fn resolve_prefix(
    dir: &Path,
    _main_root: Option<&Path>,
    policy: &JailPolicy,
    env: &RealEnv,
) -> anyhow::Result<Vec<OsString>> {
    let dir_real = env.realpath(dir).map_err(|_e| {
        anyhow::anyhow!("{CMD_JAIL_PREFIX}: {REASON_DIR_UNRESOLVABLE}: {}", dir.display())
    })?;
    // XR-1: now that the grant is canonical, the lexical ancestor test is sound — a
    // grant resolving to `/`, an ancestor of the worktree, or `.git` is rejected.
    validate_policy(policy, &dir_real).map_err(|e| {
        anyhow::anyhow!("{CMD_JAIL_PREFIX}: {REASON_UNSAFE_EXTRA_RW}: {e:?}")
    })?;
    if !have_bwrap() {
        anyhow::bail!("{CMD_JAIL_PREFIX}: {REASON_NO_BWRAP}");
    }
    let jailer = select_jailer(&Backend::Bwrap)
        .ok_or_else(|| anyhow::anyhow!("{CMD_JAIL_PREFIX}: {REASON_NO_BACKEND}"))?;
    Ok(jailer.wrap_argv(&dir_real, policy))
}

/// macOS (Seatbelt) arm — SL-185 PHASE-04, gated behind the RISK-1 probe (XR-4).
/// SHAPE ONLY on this Linux host: `#[cfg(target_os = "macos")]` strips it before
/// name resolution, so it references the PHASE-01 factored core
/// (`resolve_with_policy` / `write_seatbelt_profile`) without a local build seeing
/// them. Probe topology ONCE, resolve over the supplied inline policy (validate_policy
/// runs inside `resolve_with_policy` — the moved shared check), materialize the `.sb`
/// the `sandbox-exec -f` prefix references (io error ⇒ fail-closed), then emit.
#[cfg(target_os = "macos")]
fn resolve_prefix(
    dir: &Path,
    main_root: Option<&Path>,
    policy: &JailPolicy,
    env: &RealEnv,
) -> anyhow::Result<Vec<OsString>> {
    let main_root = main_root
        .ok_or_else(|| anyhow::anyhow!("{CMD_JAIL_PREFIX}: {REASON_MAIN_ROOT_REQUIRED}"))?;
    let topo = env
        .worktree_topology(dir)
        .map_err(|e| anyhow::anyhow!("{CMD_JAIL_PREFIX}: {}", e.reason()))?;
    let resolved = resolve_with_policy(policy, &topo, main_root, env)
        .map_err(|e| anyhow::anyhow!("{CMD_JAIL_PREFIX}: {}", e.reason()))?;
    write_seatbelt_profile(&resolved)
        .map_err(|_e| anyhow::anyhow!("{CMD_JAIL_PREFIX}: {REASON_PROFILE_WRITE_FAILED}"))?;
    let jailer = select_jailer(&Backend::Seatbelt(resolved.clone()))
        .ok_or_else(|| anyhow::anyhow!("{CMD_JAIL_PREFIX}: {REASON_NO_BACKEND}"))?;
    Ok(jailer.wrap_argv(&resolved.wt, policy))
}

/// Write the resolved prefix NUL-delimited to `out`, ONLY on full success (EX-3/EX-4).
/// Tokens are joined by `ARGV_NUL_DELIM` (no trailing delimiter — split-on-`\0`
/// round-trips exactly), bytes straight from `OsStr` so non-UTF-8 paths survive.
fn write_prefix(out: &Path, prefix: &[OsString]) -> anyhow::Result<()> {
    let mut buf: Vec<u8> = Vec::new();
    for (i, tok) in prefix.iter().enumerate() {
        if i > 0 {
            buf.push(ARGV_NUL_DELIM);
        }
        buf.extend_from_slice(tok.as_bytes());
    }
    // The `--out` argv sink is a runtime/derived artifact, not an authored entity —
    // a plain `fs::write` is the sanctioned form for runtime sites (clippy policy:
    // authored writes route through `fsutil::write_atomic`). Written once, only on the
    // success path, so `--out` is never partial/empty (AR-1).
    #[expect(clippy::disallowed_methods, reason = "runtime jail-prefix argv sink")]
    std::fs::write(out, &buf)
        .map_err(|e| anyhow::anyhow!("{CMD_JAIL_PREFIX}: write {}: {e}", out.display()))?;
    Ok(())
}
