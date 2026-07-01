// SPDX-License-Identifier: GPL-3.0-only
//! Pure jail core — SL-182 PHASE-02 (leaf tier, ADR-001).
//!
//! Graduates the proven bwrap flag set + path logic (RSK-014 probe-h1) into a
//! PURE leaf: no clock / git / disk / rng — every impure input is passed in as
//! data by the PHASE-03 shell (`pretooluse`, command tier). Behavioural reference:
//! the harvested probe scripts at `.doctrine/slice/182/probe-evidence/scripts/`.
//!
//! ## Purity contract (leaf, ADR-001 — no clock/git/disk/rng)
//! Classified `"worktree::jail" = "leaf"`. `worktree::shared` (which owns
//! `is_linked_worktree`, a git read) is **engine** — a leaf cannot import engine,
//! so git-topology recognition CANNOT live here. The layering gate enforces the
//! pure/imperative split: the shell performs the git-topology check + the policy
//! disk read + the host capability probe + **all path canonicalization**, and
//! passes the *resolved* answers in as data:
//!   - `cwd_is_project_worktree: bool`  (was `worktrees_root` in design §5.2 — see R1)
//!   - `JailPolicy`                     (parsed from the ro policy file by the shell)
//!   - `Backend`                        (capability-as-data descriptor, RV-202)
//!   - **canonical paths**              (see D-canon below)
//!
//! ## D-canon — the canonicalization cut (twin of R1, MF-1/MF-2)
//! `realpath` resolves symlinks against the live filesystem — a **disk read**, so it
//! cannot live in this leaf, exactly as the git-topology read cannot (R1). Every path
//! the pure surface compares MUST arrive already **symlink-resolved and absolute**,
//! canonicalized by the shell with `realpath -m` semantics (non-existent-safe — a
//! write target or `extra_rw` entry need not exist yet), matching the proven probe
//! (`pretooluse-pathcheck.sh`: relative→`cwd`-join, then `realpath -m` on both `real`
//! and `wt`). Security-load-bearing: an un-canonicalized `file_path`/`extra_rw`
//! bypasses the INV-2 (repo-root) / INV-3 (`.git`) / INV-4 (allowlist) walls via
//! symlink/`..`/relative. The leaf therefore does PURE **component-wise**
//! `Path::starts_with` (never string prefix — the sibling-prefix guard: `/wt` must
//! not match `/wt-evil`). The canonicalization *impl* is the PHASE-03/04 shell's,
//! tested at that boundary (R4-canon). (`decide_write`'s param is `real`, not the raw
//! stdin `file_path`, to make the precondition load-bearing at the type site.)
//!
//! ## Adjudicated interface decisions (T0 codex pass — see the phase sheet)
//! - **R1 / D-resolve-purity (CONFIRMED).** `resolve_target` takes a shell-resolved
//!   `cwd_is_project_worktree: bool`, not a `worktrees_root` path-prefix (A1: topology,
//!   not prefix).
//! - **Typed `PolicyError` (ACCEPTED).** `validate_policy` / `from_toml_str` return a
//!   typed enum, not `Result<_, String>` — tests assert on VARIANTS, not prose (STD-001,
//!   security-boundary). Diverges from design §5.2's literal `String`; flagged for
//!   reconcile coherence. Deny *reasons* that ride to the user (`Backend::Deny{reason}`,
//!   `Target::Reject`) stay `String` — they ARE the JSON payload.
//! - **`base64` crate (ACCEPTED).** `opaque_wrap` uses the leaf-legal `base64` crate
//!   (cf. `worktree::allowlist` imports `glob`), not a hand-rolled encoder.
//! - **Seatbelt seam + SL-183 additive channel (HELD to locked design + reserved).**
//!   `select_jailer(Backend::Seatbelt) ⇒ Some` (VT-8 / D8 / EX-5). The macOS backend
//!   is never constructed on Linux, so its `wrap_argv` stub is unreachable in
//!   production. `Backend::Seatbelt` carries a `ResolvedMac` payload (same
//!   capability-as-data shape as `Deny{reason}`) so SL-183 slots its shell-resolved
//!   inputs in with NO SL-182 signature refactor (OQ-mac3 seam-gap; SL-183 coherence).

// The pure surface has no `not(test)` consumer until the PHASE-03 `pretooluse` shell
// lands, so under the bin build every item is legitimately dead. Under `test` the VT
// suite consumes the surface, so the expectation is scoped to `not(test)` — else it
// would go unfulfilled (and fire) once the tests reference every item.
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "SL-182 PHASE-02 pure jail core; consumed by the PHASE-03 pretooluse shell"
    )
)]

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use base64::Engine;
use serde::Deserialize;

// ---- bwrap flag vocabulary (STD-001: single-sourced, no inline literals) --------
const BWRAP: &str = "bwrap";
const FLAG_RO_BIND: &str = "--ro-bind";
const FLAG_DEV: &str = "--dev";
const FLAG_PROC: &str = "--proc";
const FLAG_TMPFS: &str = "--tmpfs";
const FLAG_BIND: &str = "--bind";
const FLAG_CHDIR: &str = "--chdir";
const FLAG_DIE_WITH_PARENT: &str = "--die-with-parent";
const FLAG_UNSHARE_NET: &str = "--unshare-net";
/// bwrap's command separator: flags before, the wrapped command after.
const FLAG_ARG_SEP: &str = "--";
const PATH_ROOT: &str = "/";
const PATH_DEV: &str = "/dev";
const PATH_PROC: &str = "/proc";
const PATH_TMP: &str = "/tmp";

// ---- opaque-wrap payload vocabulary --------------------------------------------
const SHELL_BIN: &str = "bash";
const SHELL_CMD_FLAG: &str = "-c";

// ---- deny/reject reason stems (STD-001) ----------------------------------------
const REASON_NOT_WORKTREE: &str = "cwd-not-a-worktree";
const REASON_NO_FILE_PATH: &str = "no-file-path";
const REASON_ESCAPES_WORKTREE: &str = "escapes-worktree";
/// Defensive stem for the structurally-unreachable `Jail + no-jailer + non-Deny`
/// arm (only `Backend::Deny` yields `None` from `select_jailer`, and that reason is
/// preferred). Never a panic — fail-closed to a deny.
const REASON_NO_BACKEND: &str = "no-jail-backend";

/// The `.git` directory name — an `extra_rw` touching it is rejected (INV-3).
const GIT_DIR: &str = ".git";

/// The hook's verdict for a single tool call. Deny is expressed as **data**, never
/// an exit code (the shell always exits 0 — `mem.fact.claude.pretooluse-hook-fail-open`).
/// - `PassThrough` → emit nothing (orchestrator / non-jailed).
/// - `Deny { reason }` → `permissionDecision:"deny"`, `"worktree-jail: <reason>"`.
/// - `WrapBash { command, description }` → `permissionDecision:"allow"` + `updatedInput`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Decision {
    PassThrough,
    Deny {
        reason: String,
    },
    WrapBash {
        command: String,
        description: String,
    },
}

/// Who the caller is, resolved from `agent_id` presence + shell-supplied topology.
/// - `Orchestrator` → no `agent_id` (INV-1): never jailed.
/// - `Jail(worktree)` → `agent_id` present AND `cwd` is a worktree of THIS project.
/// - `Reject(reason)` → `agent_id` present but `cwd` is not such a worktree
///   (the `isolation:none` arm — proven denied, RSK-014 Exp 3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Target {
    Orchestrator,
    Jail(PathBuf),
    Reject(String),
}

/// Capability descriptor — resolved by the shell's host probe (§5.1), passed in as
/// DATA (RV-202, `mem.pattern.design.capability-as-data-seam`). Three-valued, not a
/// bare `Option`: `Deny` also carries *present-but-degraded*, so SL-183 flips a `Deny`
/// reason to `Seatbelt` behind the same seam — a capability flip, not a control-flow
/// rewrite. `Seatbelt` carries the shell-resolved macOS inputs (`ResolvedMac`) so the
/// macOS builder slots in additively (SL-183 seam-gap). The `reason` rides per-arm
/// (`"bwrap-unavailable"` on Linux, `"seatbelt-unavailable"` on macOS-today).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Backend {
    Bwrap,
    Seatbelt(ResolvedMac),
    Deny { reason: String },
}

/// SL-183 macOS Seatbelt inputs, shell-resolved (`getconf DARWIN_USER_TEMP_DIR`,
/// `TMPDIR=<wt>/.tmp`, the materialized `sandbox-exec -f` profile path). Reserved
/// here as the ADDITIVE data channel on `Backend::Seatbelt` so SL-183 slots its
/// builder in with NO SL-182 signature refactor (OQ-mac3 / D-mac2; SL-183 coherence
/// seam-gap). Empty today; SL-183 populates the fields — `Default` keeps SL-182's
/// test constructor compiling unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ResolvedMac {}

/// Per-arming jail policy (design §5.3). Parsed here (pure); the disk read is the
/// shell's. Default is the permissive floor that preserves current behaviour.
// `deny_unknown_fields` (MF-4): the policy is an orchestrator-authored SECURITY
// document; a typo'd key (`network`, `extra_rws`) must be a loud parse `Err`, not a
// silent fall-through to the *permissive* Default floor (network=true). Fail-closed.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct JailPolicy {
    /// Absolute paths granted rw inside the jail (beyond the worktree). Default `[]`.
    #[serde(default)]
    pub extra_rw: Vec<PathBuf>,
    /// `false` ⇒ `--unshare-net`. Default `true` (preserves today's network access).
    #[serde(default = "default_true")]
    pub network: bool,
}

fn default_true() -> bool {
    true
}

impl Default for JailPolicy {
    fn default() -> Self {
        JailPolicy {
            extra_rw: vec![],
            network: true,
        }
    }
}

/// Why a policy (or its parse) was rejected. Typed (not stringly) so tests assert on
/// VARIANTS and the security boundary is not a prose match (T0 decision 3, STD-001).
/// The PHASE-03 shell renders the human string; SL-183's fail-closed branch matches
/// on these variants directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PolicyError {
    /// An `extra_rw` equal to `/`.
    IsRoot,
    /// An `extra_rw` that is an ancestor of (or equal to) `main_root`.
    AncestorOfMainRoot,
    /// An `extra_rw` whose path contains a `.git` component.
    TouchesGit,
    /// The policy body did not parse (syntax, wrong type, or unknown key).
    Malformed(String),
}

impl JailPolicy {
    /// PURE parse (VT-3). Default floor when the shell reports the file absent (the
    /// shell owns that branch); here: parse a present body, `Err(Malformed)` on
    /// malformed OR an unknown key (`deny_unknown_fields`). Never panics, never reads
    /// disk.
    pub(crate) fn from_toml_str(body: &str) -> Result<Self, PolicyError> {
        toml::from_str(body).map_err(|e| PolicyError::Malformed(e.to_string()))
    }
}

/// Map `(agent_id, cwd, shell-resolved topology)` → `Target` (VT-1). PURE.
/// `cwd_is_project_worktree` is computed by the shell via `is_linked_worktree` +
/// git-common-dir == this project's main `.git` (A1, git-topology not path-prefix);
/// see the module R1 note. A sibling repo's worktree ⇒ `false` ⇒ `Reject`.
/// **Precondition (D-canon, codex-blocker-1): `cwd` is already shell-canonicalized**
/// (symlink-resolved, absolute). `Target::Jail(cwd)` carries it forward AS `wt` into
/// both `pathcheck` and `bwrap_argv`, so a non-canonical `cwd` here poisons every
/// downstream wall — the canonicality obligation is the shell's, load-bearing at the
/// entry to the whole pure surface.
pub(crate) fn resolve_target(
    agent_id: Option<&str>,
    cwd: &Path,
    cwd_is_project_worktree: bool,
) -> Target {
    match agent_id {
        None => Target::Orchestrator,
        Some(_) if cwd_is_project_worktree => Target::Jail(cwd.to_path_buf()),
        Some(_) => Target::Reject(format!("{REASON_NOT_WORKTREE}: {}", cwd.display())),
    }
}

/// `real ∈ {wt} ∪ extra_rw` (VT-2). PURE **component-wise** prefix test
/// (`Path::starts_with`, never string prefix — sibling-prefix guard: `/wt` must not
/// match `/wt-evil`). **Precondition (D-canon): all args are shell-canonicalized**
/// (symlink-resolved, absolute); this leaf does not touch disk. INV-4: safe as the
/// Edit/Write allowlist ONLY because `validate_policy` has already rejected dangerous
/// `extra_rw` (root-ancestors / `.git`) — the pathcheck trusts a validated policy.
pub(crate) fn pathcheck(real: &Path, wt: &Path, extra_rw: &[PathBuf]) -> bool {
    real.starts_with(wt) || extra_rw.iter().any(|allowed| real.starts_with(allowed))
}

/// Reject an `extra_rw` equal to `/`, an ancestor of `main_root`, or touching `.git`
/// (INV-3, VT-6). STRICTLY platform-agnostic (D): zero bwrap/namespace assumptions —
/// this is the shared cross-arm contract SL-183 reuses UNCHANGED as its parity proof.
/// **Precondition (D-canon, MF-2): `policy.extra_rw` and `main_root` are already
/// shell-canonicalized** (symlink-resolved, absolute). This check is a PURE lexical
/// ancestor test; a relative / symlinked / `..`-bearing `extra_rw` reaching it
/// un-canonicalized would silently bypass the `.git` / root-ancestor rejection — that
/// canonicalization is the shell's job, asserted at the PHASE-03/04 boundary.
/// **Existence obligation (codex-blocker-2): each `extra_rw` is ALSO a bwrap `--bind`
/// SOURCE** (`bwrap_argv`), and bind of a non-existent source fails at runtime — so the
/// shell MUST materialize-then-canonicalize each `extra_rw` BEFORE both this validation
/// and argv construction, closing the create-after-validate TOCTOU. Leaf-side this
/// stays a pure lexical check; the materialization contract is pinned to PHASE-04
/// provision (see phase sheet R4-canon).
pub(crate) fn validate_policy(policy: &JailPolicy, main_root: &Path) -> Result<(), PolicyError> {
    for allowed in &policy.extra_rw {
        if allowed == Path::new(PATH_ROOT) {
            return Err(PolicyError::IsRoot);
        }
        if main_root.starts_with(allowed) {
            return Err(PolicyError::AncestorOfMainRoot);
        }
        if allowed.components().any(|c| c.as_os_str() == GIT_DIR) {
            return Err(PolicyError::TouchesGit);
        }
    }
    Ok(())
}

/// Assemble the `updatedInput.command` shell string (VT-5, INV-5). **Wrapper-agnostic
/// (B):** single-quote-escapes and assembles ANY given `argv` (not a bwrap-shaped one),
/// then appends the original command as charset-safe base64 (`bash -c 'printf %s <b64>
/// | base64 -d | bash'`, never re-parsed). Taking arbitrary `argv` is what lets SL-183
/// Seatbelt reuse this unchanged. All interpolated tokens are single-quote-escaped
/// (paths may carry spaces + quotes).
pub(crate) fn opaque_wrap(orig_cmd: &str, argv: &[OsString]) -> String {
    let b64 = base64::engine::general_purpose::STANDARD.encode(orig_cmd.as_bytes());
    let payload = format!("printf %s {b64} | base64 -d | bash");
    let mut parts: Vec<String> = argv
        .iter()
        .map(|a| shell_single_quote(a.to_string_lossy().as_ref()))
        .collect();
    parts.push(SHELL_BIN.to_string());
    parts.push(SHELL_CMD_FLAG.to_string());
    parts.push(shell_single_quote(&payload));
    parts.join(" ")
}

/// POSIX single-quote escaping: wrap in `'…'`, and render an embedded `'` as `'\''`.
/// Safe for arbitrary bytes (spaces, quotes, globs) — nothing inside `'…'` is special
/// except `'` itself. The canonical INV-5 escaper: `opaque_wrap` uses it for the
/// wrapper argv, and the PHASE-03 install templating reuses it (via the worktree
/// re-export) to single-quote the baked absolute exec into every plugin hook
/// command (design §5.4 — "same quoting discipline as INV-5").
pub(crate) fn shell_single_quote(s: &str) -> String {
    let escaped = s.replace('\'', "'\\''");
    format!("'{escaped}'")
}

/// The SINGLE fork point (D8): everything above is platform-agnostic; only the
/// wrapper argv builder differs per backend. `opaque_wrap` consumes whatever this
/// returns. Object-safe so `select_jailer` can hand back a `Box<dyn Jailer>`.
pub(crate) trait Jailer {
    fn wrap_argv(&self, wt: &Path, policy: &JailPolicy) -> Vec<OsString>;
}

/// Linux backend — THIS slice.
pub(crate) struct Bwrap;

impl Jailer for Bwrap {
    fn wrap_argv(&self, wt: &Path, policy: &JailPolicy) -> Vec<OsString> {
        bwrap_argv(wt, policy)
    }
}

/// macOS backend — SL-183 / IMP-045 (deferred). Present only to keep the seam real so
/// `select_jailer(Backend::Seatbelt(_)) == Some(_)` (VT-8); never built on Linux. Holds
/// the shell-resolved `ResolvedMac` so SL-183 fills `wrap_argv` with no signature change.
pub(crate) struct Seatbelt {
    // Unread today (macOS `wrap_argv` deferred to SL-183). Under the bin build the whole
    // surface is dead and the module-level `not(test)` expectation absorbs this; under
    // `test` the rest of the surface is live, so the field needs its own expectation.
    #[cfg_attr(
        test,
        expect(
            dead_code,
            reason = "SL-183 reads the shell-resolved macOS inputs; reserved additive channel"
        )
    )]
    resolved: ResolvedMac,
}

impl Jailer for Seatbelt {
    #[expect(
        clippy::unimplemented,
        reason = "SL-183 fills the Seatbelt sandbox-exec argv builder; unreachable on Linux"
    )]
    fn wrap_argv(&self, _wt: &Path, _policy: &JailPolicy) -> Vec<OsString> {
        unimplemented!("SL-183 — Seatbelt sandbox-exec argv, deferred")
    }
}

/// The pi-arm core flag set (D5 parity, VT-7, EX-2). Byte-equivalent to
/// `scripts/pi-spawn-confined.sh`'s core flags. Flag tokens are named constants
/// (STD-001), single-sourced. Excludes the program token and the `--` separator (those
/// ride `bwrap_argv`), so this is exactly the parity-checked confinement set. PURE.
pub(crate) fn bwrap_core_argv(wt: &Path) -> Vec<OsString> {
    let wt_os = wt.as_os_str().to_os_string();
    vec![
        FLAG_RO_BIND.into(),
        PATH_ROOT.into(),
        PATH_ROOT.into(),
        FLAG_DEV.into(),
        PATH_DEV.into(),
        FLAG_PROC.into(),
        PATH_PROC.into(),
        FLAG_TMPFS.into(),
        PATH_TMP.into(),
        FLAG_BIND.into(),
        wt_os.clone(),
        wt_os.clone(),
        FLAG_CHDIR.into(),
        wt_os,
        FLAG_DIE_WITH_PARENT.into(),
    ]
}

/// The full bwrap launcher argv: `bwrap` + `bwrap_core_argv` + one `--bind` per
/// validated `extra_rw` + `--unshare-net` when `!policy.network`, terminated by `--`
/// (so `opaque_wrap` appends the wrapped command after it). VT-4. PURE.
pub(crate) fn bwrap_argv(wt: &Path, policy: &JailPolicy) -> Vec<OsString> {
    let mut argv: Vec<OsString> = vec![BWRAP.into()];
    argv.extend(bwrap_core_argv(wt));
    for allowed in &policy.extra_rw {
        let allowed_os = allowed.as_os_str().to_os_string();
        argv.push(FLAG_BIND.into());
        argv.push(allowed_os.clone());
        argv.push(allowed_os);
    }
    if !policy.network {
        argv.push(FLAG_UNSHARE_NET.into());
    }
    argv.push(FLAG_ARG_SEP.into());
    argv
}

/// PURE map over the injected `Backend` (VT-8, RV-202) — NO host read, so the
/// "platform X ⇒ deny" arm is testable on a Linux CI host with no X present:
/// `Bwrap ⇒ Some(Bwrap)`; `Seatbelt(mac) ⇒ Some(Seatbelt)` (SL-183 stub, resolved
/// inputs threaded in); `Deny{..} ⇒ None` (⇒ the caller denies with the descriptor's
/// reason).
pub(crate) fn select_jailer(backend: &Backend) -> Option<Box<dyn Jailer>> {
    match backend {
        Backend::Bwrap => Some(Box::new(Bwrap)),
        Backend::Seatbelt(mac) => Some(Box::new(Seatbelt {
            resolved: mac.clone(),
        })),
        Backend::Deny { .. } => None,
    }
}

/// Compose a Bash-tool decision. PURE. Orchestrator ⇒ `PassThrough`;
/// `Reject(reason)` ⇒ `Deny{reason}`; `Jail(wt)` with `backend == Deny{reason}` ⇒
/// `Deny{reason}` (per-arm reason, NEVER pass-through — the capability-keyed deny, C);
/// `Jail(wt)` with `Some(jailer)` ⇒ `WrapBash(opaque_wrap(cmd, jailer.wrap_argv(wt, policy)))`.
pub(crate) fn decide_bash(
    target: &Target,
    cmd: &str,
    desc: &str,
    policy: &JailPolicy,
    backend: &Backend,
) -> Decision {
    match target {
        Target::Orchestrator => Decision::PassThrough,
        Target::Reject(reason) => Decision::Deny {
            reason: reason.clone(),
        },
        Target::Jail(wt) => match select_jailer(backend) {
            Some(jailer) => Decision::WrapBash {
                command: opaque_wrap(cmd, &jailer.wrap_argv(wt, policy)),
                description: desc.to_string(),
            },
            // Only `Backend::Deny` yields `None`; carry its reason (fail-closed, never
            // pass-through). The `_` arm is structurally unreachable — defensive deny,
            // not a panic.
            None => Decision::Deny {
                reason: match backend {
                    Backend::Deny { reason } => reason.clone(),
                    _ => REASON_NO_BACKEND.to_string(),
                },
            },
        },
    }
}

/// Compose an Edit/Write decision. PURE. `Jail(wt)` ⇒ `pathcheck(real, wt, extra_rw)`
/// ⇒ `PassThrough` / `Deny`. Edit/Write bypass the bwrap wrap entirely, so this is the
/// second wall (design §5.4). Orchestrator ⇒ `PassThrough`; `Reject` ⇒ `Deny`.
/// `real` is the write target **already canonicalized by the shell** (cwd-joined +
/// `realpath -m`, D-canon/MF-1) — NOT the raw stdin `file_path`; the param name pins
/// the precondition at the type site. `None` (no path in the tool input) ⇒ `Deny`.
pub(crate) fn decide_write(target: &Target, real: Option<&Path>, policy: &JailPolicy) -> Decision {
    match target {
        Target::Orchestrator => Decision::PassThrough,
        Target::Reject(reason) => Decision::Deny {
            reason: reason.clone(),
        },
        Target::Jail(wt) => match real {
            None => Decision::Deny {
                reason: REASON_NO_FILE_PATH.to_string(),
            },
            Some(real) if pathcheck(real, wt, &policy.extra_rw) => Decision::PassThrough,
            Some(real) => Decision::Deny {
                reason: format!("{REASON_ESCAPES_WORKTREE}: {}", real.display()),
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pb(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    // ---- VT-1: resolve_target (T2) ---------------------------------------------

    #[test]
    fn resolve_target_no_agent_is_orchestrator() {
        // No agent_id ⇒ orchestrator, regardless of topology (INV-1).
        assert_eq!(
            resolve_target(None, &pb("/anywhere"), false),
            Target::Orchestrator
        );
        assert_eq!(
            resolve_target(None, &pb("/anywhere"), true),
            Target::Orchestrator
        );
    }

    #[test]
    fn resolve_target_agent_in_project_worktree_is_jail() {
        let wt = pb("/root/.worktrees/agent-1");
        assert_eq!(
            resolve_target(Some("agent-1"), &wt, true),
            Target::Jail(wt.clone())
        );
    }

    #[test]
    fn resolve_target_agent_in_sibling_repo_worktree_is_reject() {
        // A1: a sibling repo's worktree resolves to `false` (git-topology, not
        // path-prefix) ⇒ Reject, never Jail.
        let sibling = pb("/other-repo/.worktrees/agent-9");
        match resolve_target(Some("agent-9"), &sibling, false) {
            Target::Reject(reason) => assert!(reason.contains(REASON_NOT_WORKTREE)),
            other => panic!("expected Reject, got {other:?}"),
        }
    }

    // ---- VT-2: pathcheck (T3) --------------------------------------------------

    #[test]
    fn pathcheck_inside_worktree_passes() {
        assert!(pathcheck(&pb("/wt/src/main.rs"), &pb("/wt"), &[]));
        assert!(pathcheck(&pb("/wt"), &pb("/wt"), &[])); // the wt itself
    }

    #[test]
    fn pathcheck_escape_denies() {
        assert!(!pathcheck(&pb("/etc/passwd"), &pb("/wt"), &[]));
        assert!(!pathcheck(&pb("/home/u/.ssh/id_rsa"), &pb("/wt"), &[]));
    }

    #[test]
    fn pathcheck_extra_rw_hit_passes() {
        let extra = vec![pb("/opt/cache")];
        assert!(pathcheck(&pb("/opt/cache/blob"), &pb("/wt"), &extra));
        assert!(!pathcheck(&pb("/opt/other/blob"), &pb("/wt"), &extra));
    }

    #[test]
    fn pathcheck_sibling_prefix_denies() {
        // MF-6 / probe trailing-slash guard: component-wise, `/wt` must NOT match
        // `/wt-evil` (a string-prefix test would wrongly pass this).
        assert!(!pathcheck(&pb("/wt-evil/x"), &pb("/wt"), &[]));
        assert!(!pathcheck(&pb("/wt-evil"), &pb("/wt"), &[]));
    }

    #[test]
    fn pathcheck_dotgit_under_worktree_passes() {
        // pathcheck does NOT reject `.git` — that is validate_policy's job (T7). A
        // path under the worktree is in-bounds here (design INV-4 division of labour).
        assert!(pathcheck(&pb("/wt/.git/config"), &pb("/wt"), &[]));
    }

    // ---- VT-3: JailPolicy parse + PolicyError (T4) -----------------------------

    #[test]
    fn jail_policy_default_is_permissive_floor() {
        let d = JailPolicy::default();
        assert!(d.extra_rw.is_empty());
        assert!(d.network); // preserves today's network access
    }

    #[test]
    fn from_toml_str_empty_is_default_floor() {
        let p = JailPolicy::from_toml_str("").expect("empty parses to default");
        assert_eq!(p, JailPolicy::default());
    }

    #[test]
    fn from_toml_str_present_values() {
        let p = JailPolicy::from_toml_str("network = false\nextra_rw = [\"/opt/x\"]")
            .expect("present body parses");
        assert!(!p.network);
        assert_eq!(p.extra_rw, vec![pb("/opt/x")]);
    }

    #[test]
    fn from_toml_str_malformed_is_err() {
        // Wrong type for `network`.
        assert!(matches!(
            JailPolicy::from_toml_str("network = 12"),
            Err(PolicyError::Malformed(_))
        ));
        // Syntactically broken.
        assert!(matches!(
            JailPolicy::from_toml_str("= = ="),
            Err(PolicyError::Malformed(_))
        ));
    }

    #[test]
    fn from_toml_str_unknown_field_is_err() {
        // D-serde-strict (MF-4): a typo'd key (`extra_rws`, not `extra_rw`) must be a
        // loud Err, not a silent fall-through to the permissive Default floor.
        assert!(matches!(
            JailPolicy::from_toml_str("extra_rws = []"),
            Err(PolicyError::Malformed(_))
        ));
    }

    // ---- VT-7 parity + VT-4: bwrap argv (T5) -----------------------------------

    /// Extract-at-test-time (D-parity-source, MF-5): the pi script INTERLEAVES the
    /// pi-specific `--bind "$HOME/.pi"` between core flags, so a line-slice would
    /// wrongly capture it. Filter by excluding pi-specific token groups instead — a
    /// script edit to the core flags then breaks this test loudly (R2).
    fn pi_spawn_core_tokens() -> Vec<String> {
        let path = format!(
            "{}/scripts/pi-spawn-confined.sh",
            env!("CARGO_MANIFEST_DIR")
        );
        let raw = std::fs::read_to_string(&path).expect("read pi-spawn-confined.sh");
        // Drop comment lines (they mention "bwrap"), then splice `\`-continuations.
        let code = raw
            .lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n")
            .replace("\\\n", " ");
        let toks: Vec<String> = code.split_whitespace().map(str::to_string).collect();
        let start = toks
            .iter()
            .position(|t| t == "bwrap")
            .expect("bwrap invocation token");
        // Tokens strictly between `bwrap` and the wrapped program `pi`, quotes stripped.
        let between: Vec<String> = toks[start + 1..]
            .iter()
            .take_while(|t| t.as_str() != "pi")
            .map(|t| t.trim_matches('"').to_string())
            .collect();
        // Remove pi-specific groups: `--bind <…/.pi> <…/.pi>` and `--setenv NAME VAL`.
        let mut out = Vec::new();
        let mut i = 0;
        while i < between.len() {
            let t = &between[i];
            if t == FLAG_BIND && i + 2 < between.len() && between[i + 1].contains(".pi") {
                i += 3;
                continue;
            }
            if t == "--setenv" && i + 2 < between.len() {
                i += 3;
                continue;
            }
            out.push(t.clone());
            i += 1;
        }
        out
    }

    fn to_strings(argv: &[OsString]) -> Vec<String> {
        argv.iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn bwrap_core_argv_matches_pi_spawn_core_flags() {
        // VT-7 / D5 / EX-2: byte-equivalence to pi-spawn-confined.sh's core flags. The
        // script writes the worktree as the shell var `$D`; build our core with that
        // placeholder so the comparison is over the flag structure, not a live path.
        let ours = to_strings(&bwrap_core_argv(Path::new("$D")));
        assert_eq!(ours, pi_spawn_core_tokens());
    }

    #[test]
    fn bwrap_argv_wraps_core_with_extra_rw_and_unshare_net() {
        // VT-4: core + one --bind per extra_rw + --unshare-net when network=false.
        let policy = JailPolicy {
            extra_rw: vec![pb("/opt/cache")],
            network: false,
        };
        let argv = to_strings(&bwrap_argv(Path::new("/wt"), &policy));
        assert_eq!(argv.first().map(String::as_str), Some(BWRAP));
        assert_eq!(argv.last().map(String::as_str), Some(FLAG_ARG_SEP));
        // Core is present (spot-check the anchoring flags).
        assert!(argv.iter().any(|t| t == FLAG_RO_BIND));
        assert!(argv.iter().any(|t| t == FLAG_CHDIR));
        // extra_rw bound rw.
        let bind_targets: Vec<&String> = argv
            .iter()
            .zip(argv.iter().skip(1))
            .filter(|(f, _)| *f == FLAG_BIND)
            .map(|(_, t)| t)
            .collect();
        assert!(bind_targets.iter().any(|t| t.as_str() == "/opt/cache"));
        // network=false ⇒ --unshare-net.
        assert!(argv.iter().any(|t| t == FLAG_UNSHARE_NET));
    }

    #[test]
    fn bwrap_argv_default_network_has_no_unshare_net() {
        let argv = to_strings(&bwrap_argv(Path::new("/wt"), &JailPolicy::default()));
        assert!(!argv.iter().any(|t| t == FLAG_UNSHARE_NET));
    }

    // ---- VT-5 / INV-5: opaque_wrap (T6) ----------------------------------------

    #[test]
    fn opaque_wrap_roundtrips_and_executes_space_and_quote_path() {
        // INV-5: a value carrying BOTH a space AND a single quote must survive the
        // single-quote escaping AND the wrapped command must execute. `env` sets P from
        // the tricky-valued argv token, then the decoded orig_cmd echoes it back —
        // stdout == the tricky value ⟺ argv round-trips AND orig_cmd ran. Hermetic:
        // needs sh/env/base64/bash (present on the coreutils CI host).
        let tricky = "/x/a b'c/wt";
        let argv = vec![OsString::from("env"), OsString::from(format!("P={tricky}"))];
        let orig = r#"printf %s "$P""#;
        let wrapped = opaque_wrap(orig, &argv);

        let out = std::process::Command::new("sh")
            .arg("-c")
            .arg(&wrapped)
            .output()
            .expect("run assembled shell string");
        assert!(
            out.status.success(),
            "wrapped command failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&out.stdout), tricky);
    }

    #[test]
    fn opaque_wrap_appends_base64_bash_payload() {
        // Structure check: the orig command never appears verbatim (opaque); it rides
        // as base64 in a `bash -c 'printf %s … | base64 -d | bash'` tail.
        let wrapped = opaque_wrap("rm -rf /", &[OsString::from(BWRAP)]);
        assert!(!wrapped.contains("rm -rf /"));
        assert!(wrapped.contains("base64 -d | bash"));
        let b64 = base64::engine::general_purpose::STANDARD.encode(b"rm -rf /");
        assert!(wrapped.contains(&b64));
    }

    // ---- VT-6 / INV-3: validate_policy (T7) ------------------------------------

    const MAIN_ROOT: &str = "/home/u/project";

    #[test]
    fn validate_policy_rejects_root() {
        let policy = JailPolicy {
            extra_rw: vec![pb("/")],
            network: true,
        };
        assert_eq!(
            validate_policy(&policy, Path::new(MAIN_ROOT)),
            Err(PolicyError::IsRoot)
        );
    }

    #[test]
    fn validate_policy_rejects_ancestor_of_main_root() {
        let policy = JailPolicy {
            extra_rw: vec![pb("/home/u")], // ancestor of /home/u/project
            network: true,
        };
        assert_eq!(
            validate_policy(&policy, Path::new(MAIN_ROOT)),
            Err(PolicyError::AncestorOfMainRoot)
        );
    }

    #[test]
    fn validate_policy_rejects_dotgit() {
        let policy = JailPolicy {
            extra_rw: vec![pb("/home/u/project/.git")],
            network: true,
        };
        assert_eq!(
            validate_policy(&policy, Path::new(MAIN_ROOT)),
            Err(PolicyError::TouchesGit)
        );
    }

    #[test]
    fn validate_policy_accepts_benign_extra_rw() {
        let policy = JailPolicy {
            extra_rw: vec![pb("/opt/cache"), pb("/tmp/scratch")],
            network: true,
        };
        assert_eq!(validate_policy(&policy, Path::new(MAIN_ROOT)), Ok(()));
    }

    #[test]
    fn validate_policy_empty_extra_rw_is_ok() {
        assert_eq!(
            validate_policy(&JailPolicy::default(), Path::new(MAIN_ROOT)),
            Ok(())
        );
    }

    // ---- VT-8: Jailer seam + select_jailer + decide_* (T8) ---------------------

    #[test]
    fn select_jailer_is_a_pure_map_over_backend() {
        assert!(select_jailer(&Backend::Bwrap).is_some());
        assert!(select_jailer(&Backend::Seatbelt(ResolvedMac::default())).is_some());
        assert!(
            select_jailer(&Backend::Deny {
                reason: "bwrap-unavailable".to_string()
            })
            .is_none()
        );
    }

    #[test]
    fn decide_bash_orchestrator_passes_through() {
        assert_eq!(
            decide_bash(
                &Target::Orchestrator,
                "ls",
                "list",
                &JailPolicy::default(),
                &Backend::Bwrap
            ),
            Decision::PassThrough
        );
    }

    #[test]
    fn decide_bash_reject_denies_with_reason() {
        let target = Target::Reject("cwd-not-a-worktree: /x".to_string());
        assert_eq!(
            decide_bash(
                &target,
                "ls",
                "list",
                &JailPolicy::default(),
                &Backend::Bwrap
            ),
            Decision::Deny {
                reason: "cwd-not-a-worktree: /x".to_string()
            }
        );
    }

    #[test]
    fn decide_bash_jail_with_bwrap_wraps() {
        let target = Target::Jail(pb("/wt"));
        match decide_bash(
            &target,
            "echo hi",
            "greet",
            &JailPolicy::default(),
            &Backend::Bwrap,
        ) {
            Decision::WrapBash {
                command,
                description,
            } => {
                assert_eq!(description, "greet");
                assert!(command.starts_with(&format!("'{BWRAP}'")));
                assert!(!command.contains("echo hi")); // opaque
            }
            other => panic!("expected WrapBash, got {other:?}"),
        }
    }

    #[test]
    fn decide_bash_jail_with_deny_backend_denies_never_passes_through() {
        // C: capability-keyed deny — a degraded backend in a jailed context must DENY
        // with the per-arm reason, never fall through to unwrapped execution.
        let target = Target::Jail(pb("/wt"));
        let backend = Backend::Deny {
            reason: "bwrap-unavailable".to_string(),
        };
        assert_eq!(
            decide_bash(
                &target,
                "echo hi",
                "greet",
                &JailPolicy::default(),
                &backend
            ),
            Decision::Deny {
                reason: "bwrap-unavailable".to_string()
            }
        );
    }

    #[test]
    fn decide_write_inside_worktree_passes_escape_denies() {
        let target = Target::Jail(pb("/wt"));
        let policy = JailPolicy::default();
        assert_eq!(
            decide_write(&target, Some(&pb("/wt/src/x.rs")), &policy),
            Decision::PassThrough
        );
        match decide_write(&target, Some(&pb("/etc/passwd")), &policy) {
            Decision::Deny { reason } => assert!(reason.contains(REASON_ESCAPES_WORKTREE)),
            other => panic!("expected Deny, got {other:?}"),
        }
    }

    #[test]
    fn decide_write_no_path_denies() {
        let target = Target::Jail(pb("/wt"));
        match decide_write(&target, None, &JailPolicy::default()) {
            Decision::Deny { reason } => assert!(reason.contains(REASON_NO_FILE_PATH)),
            other => panic!("expected Deny, got {other:?}"),
        }
    }

    #[test]
    fn decide_write_honours_extra_rw() {
        let target = Target::Jail(pb("/wt"));
        let policy = JailPolicy {
            extra_rw: vec![pb("/opt/cache")],
            network: true,
        };
        assert_eq!(
            decide_write(&target, Some(&pb("/opt/cache/blob")), &policy),
            Decision::PassThrough
        );
    }
}
