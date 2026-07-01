// SPDX-License-Identifier: GPL-3.0-only
//! Pure jail core — SL-182 PHASE-02 (leaf tier, ADR-001).
//!
//! **INTERFACE SKELETON (PHASE-02 T0).** Signatures + types + doc-contracts only;
//! every body is `unimplemented!()`. This exists to be adversarially reviewed
//! (codex, purity/coupling/cohesion/SL-183-seam) BEFORE any TDD behaviour is sunk.
//! Bodies are filled in T1..T8 red/green/refactor. Behavioural reference: the
//! harvested probe scripts at `.doctrine/slice/182/probe-evidence/scripts/`.
//!
//! ## Purity contract (leaf, ADR-001 — no clock/git/disk/rng)
//! This module is classified `"worktree::jail" = "leaf"`. `worktree::shared`
//! (which owns `is_linked_worktree`, a git read) is **engine** — a leaf cannot
//! import engine, so the git-topology recognition CANNOT live here. The layering
//! gate enforces the pure/imperative split: the PHASE-03 shell (`pretooluse.rs`,
//! command tier) performs the git-topology check + the policy disk read + the host
//! capability probe, and passes the *resolved* answers in as data:
//!   - `cwd_is_project_worktree: bool`  (was `worktrees_root` in design §5.2 — see R1)
//!   - `JailPolicy`                     (parsed from the ro policy file by the shell)
//!   - `Backend`                        (capability-as-data descriptor, RV-202)
//!
//! ## Open interface decisions for the codex pass (recorded in the phase sheet)
//! - **R1 / D-resolve-purity.** `resolve_target` takes `cwd_is_project_worktree: bool`
//!   (shell-resolved) rather than computing topology from a `worktrees_root` prefix.
//!   This *diverges from the literal design §5.2 signature* — deliberately: the design
//!   also mandates git-topology-not-path-prefix (A1), and a path-prefix `worktrees_root`
//!   is exactly what A1 rejects. Confirm this is the right cut.
//! - **Seatbelt stub.** `select_jailer` must return `Some` for `Backend::Seatbelt`
//!   (VT-8), so a `Seatbelt` unit struct exists here with a `wrap_argv` that
//!   `unimplemented!()`s ("SL-183"). Is a stub struct the right way to keep the seam
//!   real today, or should VT-8 assert on the descriptor without a struct?
//! - **Error type.** `validate_policy` / `JailPolicy::from_toml_str` return
//!   `Result<_, String>`. Stringly-typed — sufficient, or a typed error enum?
//! - **base64.** `opaque_wrap` needs base64 encoding (T6). Add the `base64` external
//!   crate (leaf may import external, cf. `worktree::allowlist` imports `glob`), or
//!   hand-roll? Deferred to T6; irrelevant to the skeleton.
//! - **Dispatch shape.** `select_jailer -> Option<Box<dyn Jailer>>` (trait object) vs
//!   matching `Backend` inline. Design says trait (D8 single fork point); confirm the
//!   trait-object indirection earns its keep for a 3-variant enum.

// PHASE-02 skeleton: the pure surface is unconsumed until the PHASE-03 shell and the
// T2+ tests land. Held here (not on the `mod` decl) so it covers both `test` and
// `not(test)` cfg while items are dead; self-clears (unfulfilled → fires) once every
// item has a consumer, forcing removal (mem.pattern.lint.module-decl-expect-propagates).
#![expect(
    dead_code,
    reason = "SL-182 PHASE-02 pure jail core; consumed by PHASE-03 pretooluse shell + T2+ tests"
)]
// Skeleton bodies are `unimplemented!()` by design (behaviour lands T1..T8, TDD).
// The `expect` self-clears: replacing the last placeholder makes it unfulfilled and
// fires, forcing removal — the gate stays honest.
#![expect(
    clippy::unimplemented,
    reason = "SL-182 PHASE-02 T0 interface skeleton; each body filled red/green in T1..T8"
)]

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use serde::Deserialize;

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
/// bare `Option`: `Deny` also carries *present-but-degraded* (SL-183 Seatbelt nesting
/// refused), so SL-183 flips a `Deny` reason to `Seatbelt` behind the same seam — a
/// capability flip, not a control-flow rewrite. The `reason` rides per-arm
/// (`"bwrap-unavailable"` on Linux, `"seatbelt-unavailable"` on macOS-today).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Backend {
    Bwrap,
    Seatbelt,
    Deny { reason: String },
}

/// Per-arming jail policy (design §5.3). Parsed here (pure); the disk read is the
/// shell's. Default is the permissive floor that preserves current behaviour.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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

impl JailPolicy {
    /// PURE parse (VT-3). Default floor when the shell reports the file absent (the
    /// shell owns that branch); here: parse a present body, `Err` on malformed. Never
    /// panics, never reads disk.
    pub(crate) fn from_toml_str(_body: &str) -> Result<Self, String> {
        unimplemented!("PHASE-02 T4")
    }
}

/// Map `(agent_id, cwd, shell-resolved topology)` → `Target` (VT-1). PURE.
/// `cwd_is_project_worktree` is computed by the shell via `is_linked_worktree` +
/// git-common-dir == this project's main `.git` (A1, git-topology not path-prefix);
/// see the module R1 note. A sibling repo's worktree ⇒ `false` ⇒ `Reject`.
pub(crate) fn resolve_target(
    _agent_id: Option<&str>,
    _cwd: &Path,
    _cwd_is_project_worktree: bool,
) -> Target {
    unimplemented!("PHASE-02 T2")
}

/// `real ∈ {wt} ∪ extra_rw` (VT-2). PURE prefix test over already-canonical dirs.
/// INV-4: safe as the Edit/Write allowlist ONLY because `validate_policy` has already
/// rejected dangerous `extra_rw` (root-ancestors / `.git`) — the pathcheck trusts a
/// validated policy.
pub(crate) fn pathcheck(_real: &Path, _wt: &Path, _extra_rw: &[PathBuf]) -> bool {
    unimplemented!("PHASE-02 T3")
}

/// Reject an `extra_rw` equal to `/`, an ancestor of `main_root`, or touching `.git`
/// (INV-3, VT-6). STRICTLY platform-agnostic (D): zero bwrap/namespace assumptions —
/// this is the shared cross-arm contract SL-183 reuses UNCHANGED as its parity proof.
pub(crate) fn validate_policy(_policy: &JailPolicy, _main_root: &Path) -> Result<(), String> {
    unimplemented!("PHASE-02 T7")
}

/// Assemble the `updatedInput.command` shell string (VT-5, INV-5). **Wrapper-agnostic
/// (B):** single-quote-escapes and assembles ANY given `argv` (not a bwrap-shaped one),
/// then appends the original command as charset-safe base64 (`… | base64 -d | bash`,
/// never re-parsed). Taking arbitrary `argv` is what lets SL-183 Seatbelt reuse this
/// unchanged. All interpolated paths MUST be single-quote-escaped (spaces + quotes).
pub(crate) fn opaque_wrap(_orig_cmd: &str, _argv: &[OsString]) -> String {
    unimplemented!("PHASE-02 T6")
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

/// macOS backend — SL-183 / IMP-045 (deferred). Present only to keep the seam real
/// so `select_jailer(Backend::Seatbelt) == Some(_)` (VT-8); never built today.
pub(crate) struct Seatbelt;

impl Jailer for Seatbelt {
    fn wrap_argv(&self, _wt: &Path, _policy: &JailPolicy) -> Vec<OsString> {
        unimplemented!("SL-183 — Seatbelt sandbox-exec argv, deferred")
    }
}

/// The pi-arm core flag set (D5 parity, VT-7, EX-2). Byte-equivalent to
/// `scripts/pi-spawn-confined.sh:62-69` core flags. Flag tokens are named constants
/// (STD-001), single-sourced. PURE.
pub(crate) fn bwrap_core_argv(_wt: &Path) -> Vec<OsString> {
    unimplemented!("PHASE-02 T5")
}

/// `bwrap_core_argv` + one `--bind` per validated `extra_rw` + `--unshare-net` when
/// `!policy.network` (VT-4). PURE.
pub(crate) fn bwrap_argv(_wt: &Path, _policy: &JailPolicy) -> Vec<OsString> {
    unimplemented!("PHASE-02 T5")
}

/// PURE map over the injected `Backend` (VT-8, RV-202) — NO host read, so the
/// "platform X ⇒ deny" arm is testable on a Linux CI host with no X present:
/// `Bwrap ⇒ Some(Bwrap)`; `Seatbelt ⇒ Some(Seatbelt)` (SL-183 stub);
/// `Deny{..} ⇒ None` (⇒ the caller denies with the descriptor's reason).
pub(crate) fn select_jailer(_backend: &Backend) -> Option<Box<dyn Jailer>> {
    unimplemented!("PHASE-02 T8")
}

/// Compose a Bash-tool decision. PURE. Orchestrator ⇒ `PassThrough`;
/// `Reject(reason)` ⇒ `Deny{reason}`; `Jail(wt)` with `backend == Deny{reason}` ⇒
/// `Deny{reason}` (per-arm reason, NEVER pass-through — the capability-keyed deny, C);
/// `Jail(wt)` with `Some(jailer)` ⇒ `WrapBash(opaque_wrap(cmd, jailer.wrap_argv(wt, policy)))`.
pub(crate) fn decide_bash(
    _target: &Target,
    _cmd: &str,
    _desc: &str,
    _policy: &JailPolicy,
    _backend: &Backend,
) -> Decision {
    unimplemented!("PHASE-02 T8")
}

/// Compose an Edit/Write decision. PURE. `Jail(wt)` ⇒ `pathcheck(real, wt, extra_rw)`
/// ⇒ `PassThrough` / `Deny`. Edit/Write bypass the bwrap wrap entirely, so this is the
/// second wall (design §5.4). Orchestrator ⇒ `PassThrough`; `Reject` ⇒ `Deny`.
pub(crate) fn decide_write(
    _target: &Target,
    _file_path: Option<&Path>,
    _policy: &JailPolicy,
) -> Decision {
    unimplemented!("PHASE-02 T8")
}
