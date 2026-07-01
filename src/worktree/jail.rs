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
use serde::{Deserialize, Serialize};

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

// ---- SL-183 PHASE-03 fail-closed derivation reason stems (STD-001, §5.5 F-B4).
//      Every `resolve_inputs` failure branch (a–f) renders one of these; the arm
//      emits `deny worktree-subagent Bash` carrying the stem. NEVER a fallback path
//      template, NEVER unwrapped pass-through (POL-002). -----------------------------
/// (a) `cwd` is not inside any git worktree — `git rev-parse --show-toplevel` failed.
const REASON_MAC_NOT_WORKTREE: &str = "seatbelt-cwd-not-a-worktree";
/// (b) toplevel IS the main checkout, not a linked subagent worktree — no per-arming
/// policy is provisioned for the primary tree.
const REASON_MAC_MAIN_CHECKOUT: &str = "seatbelt-cwd-is-main-checkout";
/// (d) ambiguous / multiple gitdirs — the derivation cannot pick one worktree.
const REASON_MAC_AMBIGUOUS_GITDIRS: &str = "seatbelt-ambiguous-gitdirs";
/// (c/e) no per-arming policy for the resolved basename — absent, unreadable, or a
/// nested-repo/submodule basename that was never provisioned.
const REASON_MAC_POLICY_MISSING: &str = "seatbelt-policy-missing";
/// (f) policy present but malformed / schema-invalid (covers the network ambiguity,
/// F-B6). A malformed policy DENIES — it never silently defaults network open.
const REASON_MAC_POLICY_MALFORMED: &str = "seatbelt-policy-malformed";

/// The `.git` directory name — an `extra_rw` touching it is rejected (INV-3).
const GIT_DIR: &str = ".git";
/// The per-worktree scratch subdir materialized under the worktree and pointed at by
/// `TMPDIR` (D-mac3). Realpath'd into `ResolvedMac.tmp`.
const WT_TMP_SUBDIR: &str = ".tmp";

// ---- SL-183 macOS Seatbelt vocabulary (STD-001: single-sourced, no inline
//      literals; §5.2 constant catalog). The base profile shape is probe-proven
//      (RSK-014 H2 pass 3); the require-all xcrun_db scoping is the F-P3-A
//      correction to design §5.1's illustrative bare regex. -----------------------
/// The `sandbox-exec` launcher binary.
const SANDBOX_EXEC: &str = "sandbox-exec";
/// `-D <name>=<value>` param flag (realpath'd values ride here, never the profile
/// body — F-A footgun mitigation, INV-M2).
const FLAG_D: &str = "-D";
/// `-f <profile>` — the materialized `.sb` profile file (PHASE-03 writes it).
const FLAG_F: &str = "-f";
/// The `env` launcher — sets `TMPDIR` for the wrapped command without touching the
/// shared `opaque_wrap` body (D2 seam-preserving choice; the proven
/// `seatbelt-jail.sh` exported TMPDIR inside the body, `env` is its argv analog).
const ENV_BIN: &str = "env";

/// `-D` param NAMES referenced by the profile (`(param "WT")` …). The profile emits
/// these names; `sandbox_exec_argv` binds `-D <name>=<realpath>`.
const PARAM_WT: &str = "WT";
const PARAM_TMP: &str = "TMP";
const PARAM_PTMP: &str = "PTMP";
const PARAM_DUTMP: &str = "DUTMP";
/// `extra_rw` params are `RW0`, `RW1`, … — the stem, indexed at emit.
const PARAM_RW_PREFIX: &str = "RW";
/// The `TMPDIR` env var name (`env TMPDIR=<tmp>` prefixes the wrapped command).
const ENV_TMPDIR: &str = "TMPDIR";
/// The materialized `.sb` profile filename, under `<wt>/.tmp` (`resolve_inputs` sets
/// `profile_path = <tmp>/jail.sb`; PHASE-04 provision writes the body there).
const SEATBELT_PROFILE_FILE: &str = "jail.sb";

/// `/private/tmp` — coarse-denied FIRST (F-A). A literal, not a resolved param:
/// macOS `/tmp` symlinks here, and the private-tmp deny collapses scratch into the
/// worktree rw scope (D-mac3). Named per STD-001.
const PTMP_LITERAL: &str = "/private/tmp";

/// SBPL profile tokens (STD-001). §5.1's profile shows these inline for readability
/// ONLY — none ship as inline literals.
const SB_VERSION: &str = "(version 1)";
const SB_ALLOW_DEFAULT: &str = "(allow default)";
const SB_DENY_WRITE_FLOOR: &str = "(deny file-write*)";

/// Device write sinks that MUST stay writable under the floor (F-B). Probe-proven
/// set (RSK-014 H2): `/dev/null`, `/dev/zero`, `/dev/random`, `/dev/urandom`, and
/// the tty family. Emitted as `(allow file-write* …)` lines between the coarse
/// denies and the WT/TMP allows.
const DEVICE_SINK_ALLOWS: &[&str] = &[
    r#"(allow file-write* (literal "/dev/null"))"#,
    r#"(allow file-write* (literal "/dev/zero"))"#,
    r#"(allow file-write* (literal "/dev/random"))"#,
    r#"(allow file-write* (literal "/dev/urandom"))"#,
    r#"(allow file-write* (regex #"^/dev/tty"))"#,
];

/// The `xcrun_db` cache-file re-allow regex (F-3 / F-E, anchored to one path segment).
///
/// Matches the OS-owned `xcrun_db` cache family under DUTMP: the committed cache is
/// plain `xcrun_db`, the atomic temps are `xcrun_db-<hash>`; `xcrun_db[^/]*$` (empty
/// or non-slash tail, anchored to `$`) covers both.
///
/// **OVER-MATCH CAVEAT (F-P3-3):** scoped to DUTMP this still allows ANY basename
/// beginning `xcrun_db` at ANY depth (`xcrun_db_x`, `xcrun_dbEVIL`, nested
/// `…/xcrun_db`). xcrun writes only at DUTMP top level so it is safe in practice;
/// documented, NOT tightened — a literal would break the `xcrun_db-<hash>` family.
const XCRUN_DB_REGEX: &str = r#"#"/xcrun_db[^/]*$""#;

/// `network == false` ⇒ this line is appended (coarse syscall-deny, not iface
/// removal — the stated coarseness caveat, §5.2 / scope objective 5). Default-open:
/// omitted on a valid network==true policy.
const DENY_NETWORK: &str = "(deny network*)";

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

/// SL-183 macOS Seatbelt inputs, shell-resolved. The ADDITIVE data channel on
/// `Backend::Seatbelt` — SL-183 slots its builders in with NO SL-182 signature
/// refactor (OQ-mac3 / D-mac2). PHASE-03's `resolve_inputs` populates these
/// (impure: `getconf DARWIN_USER_TEMP_DIR`, realpath, `<wt>/.tmp` creation, profile
/// materialization); the PHASE-02 pure builders consume them. **Every path is
/// already shell-canonicalized (realpath'd)** — the purity fence: no resolution in
/// this layer (INV-M2, D-canon). `#[derive(Default)]` is retained so SL-182's
/// `ResolvedMac {}` / `..Default::default()` test constructors compile unchanged
/// (behaviour-preservation).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct ResolvedMac {
    /// Realpath'd worktree → `-D WT`, `(subpath (param "WT"))`.
    pub wt: PathBuf,
    /// Realpath'd `<wt>/.tmp` → `-D TMP`, `(subpath (param "TMP"))`, and the
    /// `TMPDIR=` export for the wrapped command.
    pub tmp: PathBuf,
    /// Realpath'd `getconf DARWIN_USER_TEMP_DIR` (`/private/var/folders/…`) →
    /// `-D DUTMP`; coarse-denied, then require-all-scoped for the `xcrun_db` re-allow.
    pub dutmp: PathBuf,
    /// Validated (`validate_policy`) rw grants beyond the worktree → `-D RW0..RWn`,
    /// `(subpath (param "RWn"))` per index.
    pub extra_rw: Vec<PathBuf>,
    /// `false` ⇒ append `DENY_NETWORK`. Mirrors `JailPolicy.network` (bool, default
    /// open — reused as-is, never widened).
    pub network: bool,
    /// The materialized `.sb` profile file → `sandbox-exec -f <profile_path>`
    /// (PHASE-03 writes the `seatbelt_profile` output here).
    pub profile_path: PathBuf,
}

/// Per-arming jail policy (design §5.3). Parsed here (pure); the disk read is the
/// shell's. Default is the permissive floor that preserves current behaviour.
// `deny_unknown_fields` (MF-4): the policy is an orchestrator-authored SECURITY
// document; a typo'd key (`network`, `extra_rws`) must be a loud parse `Err`, not a
// silent fall-through to the *permissive* Default floor (network=true). Fail-closed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    // Read by `wrap_argv` → `sandbox_exec_argv` under `test`. Under the bin build the
    // whole macOS surface is dead (Seatbelt is never constructed on Linux) and the
    // module-level `not(test)` expectation absorbs it — so no field-level attr.
    resolved: ResolvedMac,
}

impl Jailer for Seatbelt {
    /// The `wt`/`policy` the trait passes are already folded into `self.resolved`
    /// (PHASE-03's `resolve_inputs` derives the realpath'd `ResolvedMac` FROM them),
    /// so the builder reads the canonical resolved set — not the raw trait args. The
    /// profile body is materialized separately (PHASE-03) at `resolved.profile_path`,
    /// which the argv references via `-f`.
    fn wrap_argv(&self, _wt: &Path, _policy: &JailPolicy) -> Vec<OsString> {
        sandbox_exec_argv(&self.resolved)
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

/// Build the macOS Seatbelt (SBPL) profile body (§5.1, SL-183 PHASE-02). PURE:
/// resolved paths in, `String` out — no realpath/getconf/exec/fs. Rules are ordered
/// **deny-coarse-first / allow-specific-last** (F-A, last-match-wins): allow-default
/// header → write floor → coarse PTMP+DUTMP denies → device sinks → WT/TMP allows →
/// the **require-all-scoped** `xcrun_db` re-allow (F-P3-A; a bare regex leaks outside
/// DUTMP) → one `RWn` allow per `extra_rw` → conditional `(deny network*)`. Paths are
/// NOT spliced here — the profile references `(param "…")`; `sandbox_exec_argv` binds
/// the realpaths via `-D`. VT-1.
pub(crate) fn seatbelt_profile(resolved: &ResolvedMac) -> String {
    let mut lines: Vec<String> = vec![
        SB_VERSION.to_string(),
        SB_ALLOW_DEFAULT.to_string(),
        SB_DENY_WRITE_FLOOR.to_string(),
        // coarse denies FIRST (F-A).
        format!(r#"(deny file-write* (subpath (param "{PARAM_PTMP}")))"#),
        format!(r#"(deny file-write* (subpath (param "{PARAM_DUTMP}")))"#),
    ];
    // device write sinks — must stay writable (F-B).
    lines.extend(DEVICE_SINK_ALLOWS.iter().map(|s| (*s).to_string()));
    // specific allows LAST.
    lines.push(format!(
        r#"(allow file-write* (subpath (param "{PARAM_WT}")))"#
    ));
    lines.push(format!(
        r#"(allow file-write* (subpath (param "{PARAM_TMP}")))"#
    ));
    // xcrun_db re-allow — require-all-scoped to DUTMP (F-P3-A).
    lines.push(format!(
        r#"(allow file-write* (require-all (subpath (param "{PARAM_DUTMP}")) (regex {XCRUN_DB_REGEX})))"#
    ));
    // one allow per validated extra_rw, indexed RW0..RWn.
    for (n, _) in resolved.extra_rw.iter().enumerate() {
        lines.push(format!(
            r#"(allow file-write* (subpath (param "{PARAM_RW_PREFIX}{n}")))"#
        ));
    }
    // network deny is opt-in (default open); emitted only on network==false.
    if !resolved.network {
        lines.push(DENY_NETWORK.to_string());
    }
    lines.push(String::new()); // trailing newline
    lines.join("\n")
}

/// Build the `sandbox-exec` launcher argv PREFIX (§5.1, SL-183 PHASE-02). PURE:
/// resolved paths in, `Vec<OsString>` out. Binds realpath'd `-D` params (paths ride
/// argv, NEVER the profile body — F-A footgun, INV-M2), points `-f` at the
/// materialized profile, and terminates with `--`. `opaque_wrap` appends the wrapped
/// command after; TMPDIR is set via a trailing `env TMPDIR=<tmp>` token (D2 — keeps
/// the shared `opaque_wrap` body unchanged, the argv analog of the proven shell's
/// in-body `export TMPDIR`). VT-2.
pub(crate) fn sandbox_exec_argv(resolved: &ResolvedMac) -> Vec<OsString> {
    let mut argv: Vec<OsString> = vec![SANDBOX_EXEC.into()];
    // -D <name>=<realpath> bindings — OsString to preserve non-UTF-8 paths.
    let mut bind = |name: &str, value: &Path| {
        argv.push(FLAG_D.into());
        let mut pair = OsString::from(name);
        pair.push("=");
        pair.push(value.as_os_str());
        argv.push(pair);
    };
    bind(PARAM_WT, &resolved.wt);
    bind(PARAM_TMP, &resolved.tmp);
    bind(PARAM_PTMP, Path::new(PTMP_LITERAL));
    bind(PARAM_DUTMP, &resolved.dutmp);
    for (n, rw) in resolved.extra_rw.iter().enumerate() {
        bind(&format!("{PARAM_RW_PREFIX}{n}"), rw);
    }
    // -f <profile> then the launcher terminator.
    argv.push(FLAG_F.into());
    argv.push(resolved.profile_path.as_os_str().to_os_string());
    argv.push(FLAG_ARG_SEP.into());
    // TMPDIR export for the wrapped command, via `env` (opaque_wrap appends `bash
    // -c <body>` after this tail).
    argv.push(ENV_BIN.into());
    let mut tmpdir = OsString::from(ENV_TMPDIR);
    tmpdir.push("=");
    tmpdir.push(resolved.tmp.as_os_str());
    argv.push(tmpdir);
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

// ─────────────────────────────────────────────────────────────────────────────────
// SL-183 PHASE-03 — impure `resolve_inputs` (fail-closed) + `Backend` mapper.
//
// The macOS twin of SL-182's fail-closed posture (§5.5 F-B4). `resolve_inputs` is the
// THIN IMPURE SHELL that derives a `ResolvedMac` from the PreToolUse `cwd`: git topology
// → realpath → basename → per-arming policy → getconf DUTMP → `<wt>/.tmp`. Every
// impurity is injected via `ResolveEnv` so the branch LOGIC is pure and unit-testable on
// any host (Linux CI included — no real `getconf`). `RealEnv` is the only site that
// actually touches git/fs/getconf (D-p3-1). Any ambiguity ⇒ `Err(ResolveDeny)` ⇒
// `Backend::Deny{reason}` ⇒ `Decision::Deny` — NEVER a fallback path template, NEVER
// unwrapped pass-through (POL-002, F-B4). SL-182's shared surface is reused UNCHANGED.
// ─────────────────────────────────────────────────────────────────────────────────

/// Why the macOS `cwd`→worktree derivation failed. Typed (not stringly) so tests assert
/// on VARIANTS and the security boundary is not a prose match (STD-001, mirrors
/// `PolicyError`). Each maps 1:1 to a §5.5 F-B4 branch; `reason()` renders the named-const
/// stem the arm surfaces to the user. There is NO permissive variant — the enum's very
/// shape is the fail-closed guarantee.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolveDeny {
    /// (a) `cwd` not inside a git worktree (`--show-toplevel` failed).
    NotAWorktree,
    /// (b) toplevel == the main checkout (no policy provisioned for the primary tree).
    IsMainCheckout,
    /// (d) ambiguous / multiple gitdirs.
    AmbiguousGitDirs,
    /// (c/e) policy for the resolved basename is missing, unreadable, or was never
    /// provisioned (nested-repo / submodule basename). The security outcome is identical.
    PolicyMissing,
    /// (f) policy present but malformed / schema-invalid — carries the parse detail.
    PolicyMalformed(String),
}

impl ResolveDeny {
    /// The reason string the arm surfaces (`deny worktree-subagent Bash: <reason>`).
    /// Single-sourced named-const stems (STD-001).
    pub(crate) fn reason(&self) -> String {
        match self {
            ResolveDeny::NotAWorktree => REASON_MAC_NOT_WORKTREE.to_string(),
            ResolveDeny::IsMainCheckout => REASON_MAC_MAIN_CHECKOUT.to_string(),
            ResolveDeny::AmbiguousGitDirs => REASON_MAC_AMBIGUOUS_GITDIRS.to_string(),
            ResolveDeny::PolicyMissing => REASON_MAC_POLICY_MISSING.to_string(),
            ResolveDeny::PolicyMalformed(detail) => {
                format!("{REASON_MAC_POLICY_MALFORMED}: {detail}")
            }
        }
    }
}

/// The injected impurity seam (D-p3-1). `resolve_inputs` takes a `&dyn ResolveEnv` so its
/// branch logic stays PURE and every fail-closed branch is unit-testable off-host. The
/// real implementation (`RealEnv`) is the ONLY place git / getconf / realpath / mkdir /
/// policy-file reads actually run — the thin shell the design names (§5.2). Each method's
/// `Result` failure is a distinct fail-closed branch; the shell never invents a default.
pub(crate) trait ResolveEnv {
    /// `git -C <cwd> rev-parse --show-toplevel`, realpath'd. `Err` ⇒ branch (a). An
    /// `Ok` with ambiguity signalled out-of-band is branch (d) — see `worktree_topology`.
    fn worktree_topology(&self, cwd: &Path) -> Result<Topology, ResolveDeny>;
    /// `getconf DARWIN_USER_TEMP_DIR`, realpath'd (`/private/var/folders/…`). Impure.
    fn getconf_dutmp(&self) -> Result<PathBuf, ResolveDeny>;
    /// `realpath -e` a path that must already exist (WT, `extra_rw`). Impure.
    fn realpath(&self, path: &Path) -> Result<PathBuf, ResolveDeny>;
    /// `mkdir -p <path>` then realpath it (the `<wt>/.tmp` scratch dir). Impure.
    fn ensure_dir(&self, path: &Path) -> Result<PathBuf, ResolveDeny>;
    /// Read the per-arming policy body for `basename`. `Ok(None)` ⇒ absent ⇒ branch (e);
    /// `Err` ⇒ unreadable ⇒ branch (e); `Ok(Some(body))` ⇒ parse it (branch (f) on
    /// malformed). Impure (disk read).
    fn read_policy(&self, basename: &std::ffi::OsStr) -> std::io::Result<Option<String>>;
}

/// The resolved git topology for a `cwd` (the pure output of the impure probe). Carries
/// the realpath'd linked-worktree root and whether it is the main checkout, so the
/// branch (b)/(d) decisions are made in the PURE `resolve_inputs`, not the env impl.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Topology {
    /// Realpath'd `--show-toplevel`.
    pub toplevel: PathBuf,
    /// `true` ⇒ toplevel is a linked worktree; `false` ⇒ it is the main checkout
    /// (`is_linked_worktree`). Branch (b) denies when `false`.
    pub is_linked: bool,
}

/// The thin impure shell (§5.2, D-p3-1). Derive a `ResolvedMac` from the `PreToolUse` `cwd`
/// and the per-arming policy, FAIL-CLOSED on every ambiguity (§5.5 F-B4 branches a–f).
/// `profile_dir` is where the materialized `.sb` lives (`<wt>/.tmp`); the actual profile
/// write is the caller's (PHASE-04 provision), so this returns the intended `profile_path`
/// without writing it — keeping the resolver free of the profile-body concern.
///
/// PURE branch logic over the injected `env`; the ONLY impurity is behind `ResolveEnv`.
/// `Ok` ⇒ a fully realpath'd `ResolvedMac` (INV-M2 fence honoured for the PHASE-02
/// builders); `Err(ResolveDeny)` ⇒ the arm denies. There is NO code path that returns a
/// permissive/template `ResolvedMac` on a failure — the `?` short-circuits every branch to
/// a Deny.
pub(crate) fn resolve_inputs(
    cwd: &Path,
    main_root: &Path,
    env: &dyn ResolveEnv,
) -> Result<ResolvedMac, ResolveDeny> {
    // (a)/(d): derive + realpath the worktree via git topology.
    let topo = env.worktree_topology(cwd)?;
    // (b): the main checkout has no per-arming policy — deny, never guess one.
    if !topo.is_linked {
        return Err(ResolveDeny::IsMainCheckout);
    }
    let wt = topo.toplevel;
    // (c)/(e): per-arming policy lookup by basename. Absent/unreadable ⇒ Deny.
    let basename = wt
        .file_name()
        .ok_or(ResolveDeny::PolicyMissing)?
        .to_os_string();
    // absent (`Ok(None)`) OR unreadable (`Err`) ⇒ branch (e) Deny — never a permissive
    // default (the PreToolUse fail-OPEN memory: an absent policy must DENY).
    let Ok(Some(body)) = env.read_policy(&basename) else {
        return Err(ResolveDeny::PolicyMissing);
    };
    // (f): malformed / unknown-key ⇒ Deny (never a silent permissive default). Reuses
    // SL-182's `from_toml_str` + `validate_policy` UNCHANGED.
    let policy = JailPolicy::from_toml_str(&body)
        .map_err(|e| ResolveDeny::PolicyMalformed(format!("{e:?}")))?;
    validate_policy(&policy, main_root)
        .map_err(|e| ResolveDeny::PolicyMalformed(format!("{e:?}")))?;

    // realpath every path the PHASE-02 builders will bind (INV-M2 fence).
    let dutmp = env.getconf_dutmp()?;
    let tmp = env.ensure_dir(&wt.join(WT_TMP_SUBDIR))?;
    let mut extra_rw = Vec::with_capacity(policy.extra_rw.len());
    for grant in &policy.extra_rw {
        extra_rw.push(env.realpath(grant)?);
    }
    let profile_path = tmp.join(SEATBELT_PROFILE_FILE);

    Ok(ResolvedMac {
        wt,
        tmp,
        dutmp,
        extra_rw,
        network: policy.network,
        profile_path,
    })
}

/// Map a `resolve_inputs` outcome onto SL-182's existing `Backend` so the funnel's Deny
/// path is reused UNCHANGED (D-p3-2): `Ok ⇒ Seatbelt(resolved)` (⇒ `select_jailer` yields
/// the Seatbelt jailer); `Err ⇒ Deny{reason}` (⇒ `select_jailer` yields `None` ⇒
/// `decide_bash` denies with the reason — fail-closed, never pass-through). No new decision
/// surface; the macOS routing IS this two-line map plus the untouched `select_jailer`.
pub(crate) fn seatbelt_backend(resolved: Result<ResolvedMac, ResolveDeny>) -> Backend {
    match resolved {
        Ok(mac) => Backend::Seatbelt(mac),
        Err(deny) => Backend::Deny {
            reason: deny.reason(),
        },
    }
}

/// The per-arming policy directory under the main checkout (design §5.3, SL-182
/// convention): `<main>/.doctrine/state/dispatch/jail/<worktree-name>.toml`. Segments are
/// single-sourced (STD-001); the provisioning WRITE is PHASE-04/SL-182's, this is only the
/// READ location.
const POLICY_DIR_SEGMENTS: &[&str] = &[".doctrine", "state", "dispatch", "jail"];
const POLICY_FILE_EXT: &str = "toml";

/// The real impure `ResolveEnv` — the ONLY site in this module that touches git / getconf /
/// realpath / mkdir / disk (D-p3-1, the "thin shell" the design names, §5.2). Everything
/// else in the PHASE-03 surface (`resolve_inputs`, `seatbelt_backend`, the two PHASE-02
/// builders) is PURE over its output. Holds the realpath'd `main_root` (to locate policy
/// and to feed `validate_policy`'s ancestor check upstream).
pub(crate) struct RealEnv {
    /// Realpath'd main checkout root — the base for the policy dir lookup.
    pub main_root: PathBuf,
}

impl ResolveEnv for RealEnv {
    /// `git -C <cwd> rev-parse --show-toplevel` (⇒ branch a on failure), realpath'd, then
    /// `is_linked_worktree` (⇒ branch b when it is the main checkout). A non-worktree cwd
    /// makes git exit non-zero ⇒ `CaptureError` ⇒ `NotAWorktree`. Genuine gitdir ambiguity
    /// (`$GIT_DIR` pollution) surfaces as an `is_linked_worktree` error ⇒ `AmbiguousGitDirs`.
    fn worktree_topology(&self, cwd: &Path) -> Result<Topology, ResolveDeny> {
        let toplevel_raw = crate::git::git_text(cwd, &["rev-parse", "--show-toplevel"])
            .map_err(|_e| ResolveDeny::NotAWorktree)?;
        let toplevel =
            std::fs::canonicalize(&toplevel_raw).map_err(|_e| ResolveDeny::NotAWorktree)?;
        let is_linked = crate::worktree::is_linked_worktree(&toplevel)
            .map_err(|_e| ResolveDeny::AmbiguousGitDirs)?;
        Ok(Topology {
            toplevel,
            is_linked,
        })
    }

    fn getconf_dutmp(&self) -> Result<PathBuf, ResolveDeny> {
        let out = std::process::Command::new("getconf")
            .arg("DARWIN_USER_TEMP_DIR")
            .output()
            .map_err(|_e| ResolveDeny::NotAWorktree)?;
        if !out.status.success() {
            return Err(ResolveDeny::NotAWorktree);
        }
        let raw = String::from_utf8(out.stdout)
            .map_err(|_e| ResolveDeny::NotAWorktree)?
            .trim()
            .to_string();
        std::fs::canonicalize(&raw).map_err(|_e| ResolveDeny::NotAWorktree)
    }

    fn realpath(&self, path: &Path) -> Result<PathBuf, ResolveDeny> {
        // extra_rw grants MUST already exist (bwrap-arm existence obligation, INV-4 twin):
        // a non-existent grant fails closed rather than binding a phantom path.
        std::fs::canonicalize(path).map_err(|_e| ResolveDeny::PolicyMissing)
    }

    fn ensure_dir(&self, path: &Path) -> Result<PathBuf, ResolveDeny> {
        std::fs::create_dir_all(path).map_err(|_e| ResolveDeny::PolicyMissing)?;
        std::fs::canonicalize(path).map_err(|_e| ResolveDeny::PolicyMissing)
    }

    fn read_policy(&self, basename: &std::ffi::OsStr) -> std::io::Result<Option<String>> {
        let mut path = self.main_root.clone();
        for seg in POLICY_DIR_SEGMENTS {
            path.push(seg);
        }
        path.push(basename);
        path.set_extension(POLICY_FILE_EXT);
        match std::fs::read_to_string(&path) {
            Ok(body) => Ok(Some(body)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
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

    #[test]
    fn to_toml_string_round_trips_through_from_toml_str() {
        // PHASE-04 T1: `arm-spawn` writes a declared policy via `toml::to_string`;
        // `create-fork` copies it; `pretooluse` reads it back through `from_toml_str`.
        // The written form MUST re-parse to the same value (one schema, both ends).
        let p = JailPolicy {
            extra_rw: vec![pb("/nix/store"), pb("/cache")],
            network: false,
        };
        let text = toml::to_string(&p).expect("serialize policy to TOML");
        assert_eq!(JailPolicy::from_toml_str(&text).expect("re-parse"), p);
    }

    // ---- VT-7 parity + VT-4: bwrap argv (T5) -----------------------------------

    /// Extract-at-test-time (D-parity-source, MF-5): the pi script INTERLEAVES the
    /// pi-specific `--bind "$HOME/.pi"` between core flags, so a line-slice would
    /// wrongly capture it. Filter by excluding pi-specific token groups instead — a
    /// script edit to the core flags then breaks this test loudly (R2).
    fn pi_spawn_core_tokens() -> Vec<String> {
        let path = crate::test_support::repo_root().join("scripts/pi-spawn-confined.sh");
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

    // ---- SL-183 PHASE-02 fixtures ----------------------------------------------

    /// A representative resolved macOS policy: realpath'd worktree + `.tmp`, the
    /// per-user temp, one extra rw grant, network open. `network`/`extra_rw` are
    /// varied per test.
    fn resolved_mac() -> ResolvedMac {
        ResolvedMac {
            wt: pb("/private/tmp/wt-abc"),
            tmp: pb("/private/tmp/wt-abc/.tmp"),
            dutmp: pb("/private/var/folders/xy/T"),
            extra_rw: vec![],
            network: true,
            profile_path: pb("/private/tmp/wt-abc/.tmp/jail.sb"),
        }
    }

    /// Byte offset of `needle` in `hay`, or a panic naming the missing token —
    /// keeps ordering assertions readable.
    fn at(hay: &str, needle: &str) -> usize {
        hay.find(needle)
            .unwrap_or_else(|| panic!("profile missing token: {needle:?}\n---\n{hay}"))
    }

    // ---- VT-1: seatbelt_profile (T1) -------------------------------------------

    #[test]
    fn seatbelt_profile_orders_deny_coarse_first_allow_specific_last() {
        // F-A last-match-wins: floor → coarse denies (PTMP, DUTMP) → device sinks
        // → specific allows (WT, TMP, xcrun_db) — strictly increasing offsets.
        let p = seatbelt_profile(&resolved_mac());
        let floor = at(&p, SB_DENY_WRITE_FLOOR);
        // PTMP is referenced by param in the body; the literal path rides -D (F-A).
        let ptmp = at(&p, r#"(deny file-write* (subpath (param "PTMP")))"#);
        let dutmp = at(&p, r#"(deny file-write* (subpath (param "DUTMP")))"#);
        let dev = at(&p, DEVICE_SINK_ALLOWS[0]);
        let wt = at(&p, r#"(subpath (param "WT"))"#);
        let tmp = at(&p, r#"(subpath (param "TMP"))"#);
        let xcrun = at(&p, XCRUN_DB_REGEX);
        assert!(
            floor < ptmp && ptmp < dutmp && dutmp < dev && dev < wt && wt < tmp && tmp < xcrun,
            "rule ordering violated (F-A): floor={floor} ptmp={ptmp} dutmp={dutmp} dev={dev} wt={wt} tmp={tmp} xcrun={xcrun}"
        );
        // allow-default header precedes the floor.
        assert!(at(&p, SB_ALLOW_DEFAULT) < floor);
        assert!(p.starts_with(SB_VERSION));
    }

    #[test]
    fn seatbelt_profile_scopes_xcrun_db_reallow_with_require_all() {
        // F-P3-A: the xcrun_db re-allow MUST be require-all-scoped to DUTMP; a bare
        // regex LEAKS outside DUTMP (proven RSK-014 H2 pass 3).
        let p = seatbelt_profile(&resolved_mac());
        assert!(
            p.contains(&format!(
                r#"(require-all (subpath (param "{PARAM_DUTMP}")) (regex {XCRUN_DB_REGEX}))"#
            )),
            "xcrun_db re-allow not require-all-scoped to DUTMP:\n{p}"
        );
    }

    #[test]
    fn seatbelt_profile_emits_device_sinks() {
        let p = seatbelt_profile(&resolved_mac());
        for sink in DEVICE_SINK_ALLOWS {
            assert!(p.contains(sink), "device sink missing: {sink}\n{p}");
        }
    }

    #[test]
    fn seatbelt_profile_network_line_is_conditional() {
        let mut open = resolved_mac();
        open.network = true;
        assert!(
            !seatbelt_profile(&open).contains(DENY_NETWORK),
            "network=true must NOT emit the network deny (default open)"
        );
        let mut closed = resolved_mac();
        closed.network = false;
        assert!(
            seatbelt_profile(&closed).contains(DENY_NETWORK),
            "network=false MUST emit {DENY_NETWORK}"
        );
    }

    #[test]
    fn seatbelt_profile_emits_one_rw_allow_per_extra_rw() {
        let mut none = resolved_mac();
        none.extra_rw = vec![];
        let p0 = seatbelt_profile(&none);
        assert!(!p0.contains(&format!(r#"(param "{PARAM_RW_PREFIX}0")"#)));

        let mut two = resolved_mac();
        two.extra_rw = vec![pb("/opt/a"), pb("/opt/b")];
        let p2 = seatbelt_profile(&two);
        assert!(p2.contains(&format!(
            r#"(allow file-write* (subpath (param "{PARAM_RW_PREFIX}0")))"#
        )));
        assert!(p2.contains(&format!(
            r#"(allow file-write* (subpath (param "{PARAM_RW_PREFIX}1")))"#
        )));
        assert!(!p2.contains(&format!(r#"(param "{PARAM_RW_PREFIX}2")"#)));
    }

    // ---- VT-2: sandbox_exec_argv (T3) ------------------------------------------

    /// Join an `OsString` argv into a lossy `String` for substring assertions.
    fn argv_str(argv: &[OsString]) -> String {
        argv.iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn sandbox_exec_argv_binds_realpathd_d_params() {
        let r = resolved_mac();
        let argv = sandbox_exec_argv(&r);
        let s = argv_str(&argv);
        // -D <name>=<realpath> for each of WT/TMP/PTMP/DUTMP — as OsString tokens.
        assert!(s.contains(&format!("{PARAM_WT}={}", r.wt.display())));
        assert!(s.contains(&format!("{PARAM_TMP}={}", r.tmp.display())));
        assert!(s.contains(&format!("{PARAM_PTMP}={PTMP_LITERAL}")));
        assert!(s.contains(&format!("{PARAM_DUTMP}={}", r.dutmp.display())));
        // the -D flag itself is present (paired with each binding).
        assert!(argv.iter().any(|a| a == FLAG_D));
    }

    #[test]
    fn sandbox_exec_argv_binds_one_rw_param_per_extra_rw() {
        let mut r = resolved_mac();
        r.extra_rw = vec![pb("/opt/a"), pb("/opt/b")];
        let s = argv_str(&sandbox_exec_argv(&r));
        assert!(s.contains(&format!("{PARAM_RW_PREFIX}0=/opt/a")));
        assert!(s.contains(&format!("{PARAM_RW_PREFIX}1=/opt/b")));
    }

    #[test]
    fn sandbox_exec_argv_references_profile_and_terminates() {
        let r = resolved_mac();
        let argv = sandbox_exec_argv(&r);
        // launcher head.
        assert_eq!(
            argv.first().map(|a| a.as_os_str()),
            Some(OsString::from(SANDBOX_EXEC).as_os_str())
        );
        // -f <profile_path>.
        let s = argv_str(&argv);
        assert!(s.contains(FLAG_F));
        assert!(argv.iter().any(|a| a == r.profile_path.as_os_str()));
        // TMPDIR export rides an `env` token (D2: opaque_wrap stays shared/unchanged).
        assert!(argv.iter().any(|a| a == ENV_BIN));
        assert!(s.contains(&format!("{ENV_TMPDIR}={}", r.tmp.display())));
        // the `--` separator precedes the env/body tail so opaque_wrap appends after.
        let sep = argv
            .iter()
            .position(|a| a == FLAG_ARG_SEP)
            .expect("no -- separator");
        let envpos = argv
            .iter()
            .position(|a| a == ENV_BIN)
            .expect("no env token");
        assert!(sep < envpos, "env/body tail must follow --");
    }

    #[test]
    fn seatbelt_wrap_argv_delegates_to_sandbox_exec_argv() {
        // The trait adapter reads self.resolved (not the raw wt/policy) and returns
        // the same launcher argv as the standalone builder.
        let r = resolved_mac();
        let jailer = Seatbelt {
            resolved: r.clone(),
        };
        assert_eq!(
            jailer.wrap_argv(&r.wt, &JailPolicy::default()),
            sandbox_exec_argv(&r)
        );
    }

    #[test]
    fn sandbox_exec_argv_never_splices_paths_into_profile_body() {
        // F-A footgun: paths ride -D params, NEVER string-spliced into SBPL. The
        // argv is the launcher; the profile body is a FILE (-f), not inline.
        let r = resolved_mac();
        let argv = sandbox_exec_argv(&r);
        let s = argv_str(&argv);
        assert!(
            !s.contains("file-write*"),
            "argv must not carry profile-body tokens"
        );
        assert!(
            !s.contains("subpath"),
            "argv must not carry profile-body tokens"
        );
    }

    // ---- SL-183 PHASE-03: resolve_inputs fail-closed (VT-1) + wiring (VT-2) ------
    //
    // Every branch is driven through an injected `FakeEnv` — no real git/getconf/fs,
    // so the whole fail-closed matrix runs on Linux CI (the design's pure/shell split).

    /// A programmable `ResolveEnv` double. Each field models one impure result; the
    /// default is the happy path (linked worktree, present valid policy, resolvable
    /// dutmp/tmp/realpath). Tests override ONE field to drive ONE branch — keeping each
    /// test's failure attributable to a single injected condition.
    struct FakeEnv {
        topology: Result<Topology, ResolveDeny>,
        dutmp: Result<PathBuf, ResolveDeny>,
        policy: std::io::Result<Option<String>>,
        realpath_ok: bool,
        ensure_ok: bool,
    }

    impl Default for FakeEnv {
        fn default() -> Self {
            FakeEnv {
                topology: Ok(Topology {
                    toplevel: pb("/private/tmp/wt-abc"),
                    is_linked: true,
                }),
                dutmp: Ok(pb("/private/var/folders/xy/T")),
                policy: Ok(Some(String::new())), // empty ⇒ JailPolicy::default (valid)
                realpath_ok: true,
                ensure_ok: true,
            }
        }
    }

    impl ResolveEnv for FakeEnv {
        fn worktree_topology(&self, _cwd: &Path) -> Result<Topology, ResolveDeny> {
            self.topology.clone()
        }
        fn getconf_dutmp(&self) -> Result<PathBuf, ResolveDeny> {
            self.dutmp.clone()
        }
        fn realpath(&self, path: &Path) -> Result<PathBuf, ResolveDeny> {
            if self.realpath_ok {
                Ok(path.to_path_buf())
            } else {
                Err(ResolveDeny::PolicyMissing)
            }
        }
        fn ensure_dir(&self, path: &Path) -> Result<PathBuf, ResolveDeny> {
            if self.ensure_ok {
                Ok(path.to_path_buf())
            } else {
                Err(ResolveDeny::PolicyMissing)
            }
        }
        fn read_policy(&self, _basename: &std::ffi::OsStr) -> std::io::Result<Option<String>> {
            match &self.policy {
                Ok(o) => Ok(o.clone()),
                Err(e) => Err(std::io::Error::new(e.kind(), e.to_string())),
            }
        }
    }

    const MAC_MAIN: &str = "/home/u/project";

    fn resolve(env: &FakeEnv) -> Result<ResolvedMac, ResolveDeny> {
        resolve_inputs(&pb("/private/tmp/wt-abc"), Path::new(MAC_MAIN), env)
    }

    // branch (a) — cwd not a worktree.
    #[test]
    fn resolve_inputs_branch_a_not_a_worktree_denies() {
        let env = FakeEnv {
            topology: Err(ResolveDeny::NotAWorktree),
            ..Default::default()
        };
        assert_eq!(resolve(&env), Err(ResolveDeny::NotAWorktree));
        assert_eq!(resolve(&env).unwrap_err().reason(), REASON_MAC_NOT_WORKTREE);
    }

    // branch (b) — toplevel is the main checkout.
    #[test]
    fn resolve_inputs_branch_b_main_checkout_denies() {
        let env = FakeEnv {
            topology: Ok(Topology {
                toplevel: pb("/private/tmp/wt-abc"),
                is_linked: false,
            }),
            ..Default::default()
        };
        assert_eq!(resolve(&env), Err(ResolveDeny::IsMainCheckout));
    }

    // branch (d) — ambiguous gitdirs (the env probe reports it).
    #[test]
    fn resolve_inputs_branch_d_ambiguous_gitdirs_denies() {
        let env = FakeEnv {
            topology: Err(ResolveDeny::AmbiguousGitDirs),
            ..Default::default()
        };
        assert_eq!(resolve(&env), Err(ResolveDeny::AmbiguousGitDirs));
    }

    // branch (c/e) — policy absent (None) OR a nested-repo basename never provisioned.
    #[test]
    fn resolve_inputs_branch_e_policy_absent_denies() {
        let env = FakeEnv {
            policy: Ok(None),
            ..Default::default()
        };
        assert_eq!(resolve(&env), Err(ResolveDeny::PolicyMissing));
    }

    // branch (c/e) — policy unreadable (io error) also denies (never a permissive default).
    #[test]
    fn resolve_inputs_policy_unreadable_denies() {
        let env = FakeEnv {
            policy: Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "boom",
            )),
            ..Default::default()
        };
        assert_eq!(resolve(&env), Err(ResolveDeny::PolicyMissing));
    }

    // branch (f) — malformed / unknown-key policy denies (covers the network ambiguity).
    #[test]
    fn resolve_inputs_branch_f_malformed_policy_denies() {
        let env = FakeEnv {
            policy: Ok(Some("network = \"maybe\"\n".to_string())), // wrong type
            ..Default::default()
        };
        match resolve(&env) {
            Err(ResolveDeny::PolicyMalformed(_)) => {}
            other => panic!("expected PolicyMalformed, got {other:?}"),
        }
        // an UNKNOWN key also denies (deny_unknown_fields, F-B6) — never a silent open.
        let env2 = FakeEnv {
            policy: Ok(Some("network_egress = true\n".to_string())),
            ..Default::default()
        };
        match resolve(&env2) {
            Err(ResolveDeny::PolicyMalformed(_)) => {}
            other => panic!("expected PolicyMalformed for unknown key, got {other:?}"),
        }
    }

    // branch (f-adjacent) — a dangerous extra_rw (root-ancestor) is rejected by the
    // reused validate_policy, still a Deny (behaviour-preservation of the shared check).
    #[test]
    fn resolve_inputs_dangerous_extra_rw_denies() {
        let env = FakeEnv {
            policy: Ok(Some("extra_rw = [\"/\"]\n".to_string())),
            ..Default::default()
        };
        match resolve(&env) {
            Err(ResolveDeny::PolicyMalformed(_)) => {}
            other => panic!("expected Deny for root extra_rw, got {other:?}"),
        }
    }

    // happy path (EX-1) — Ok ⇒ a fully realpath'd ResolvedMac; network defaults open.
    #[test]
    fn resolve_inputs_happy_path_builds_resolved_mac() {
        let env = FakeEnv::default();
        let mac = resolve(&env).expect("happy path resolves");
        assert_eq!(mac.wt, pb("/private/tmp/wt-abc"));
        assert_eq!(mac.tmp, pb("/private/tmp/wt-abc/.tmp"));
        assert_eq!(mac.dutmp, pb("/private/var/folders/xy/T"));
        assert!(mac.extra_rw.is_empty());
        assert!(mac.network, "a policy omitting network defaults OPEN");
        assert_eq!(mac.profile_path, pb("/private/tmp/wt-abc/.tmp/jail.sb"));
    }

    // happy path with extra_rw — each grant is realpath'd and carried through.
    #[test]
    fn resolve_inputs_happy_path_carries_validated_extra_rw() {
        let env = FakeEnv {
            policy: Ok(Some("extra_rw = [\"/opt/cache\"]\n".to_string())),
            ..Default::default()
        };
        let mac = resolve(&env).expect("valid extra_rw resolves");
        assert_eq!(mac.extra_rw, vec![pb("/opt/cache")]);
    }

    // ---- VT-2: seatbelt_backend map + select_jailer macOS routing ---------------

    // Ok(resolved) ⇒ Seatbelt ⇒ select_jailer Some ⇒ wrap_argv delegates to the builder.
    #[test]
    fn seatbelt_backend_ok_routes_to_seatbelt_jailer() {
        let mac = resolve(&FakeEnv::default()).unwrap();
        let backend = seatbelt_backend(Ok(mac.clone()));
        assert_eq!(backend, Backend::Seatbelt(mac.clone()));
        let jailer = select_jailer(&backend).expect("Seatbelt ⇒ Some");
        assert_eq!(
            jailer.wrap_argv(&mac.wt, &JailPolicy::default()),
            sandbox_exec_argv(&mac)
        );
    }

    // Err(deny) ⇒ Backend::Deny ⇒ select_jailer None ⇒ decide_bash DENIES, never wraps.
    #[test]
    fn seatbelt_backend_err_denies_never_passes_through() {
        let backend = seatbelt_backend(Err(ResolveDeny::PolicyMalformed("bad".into())));
        assert!(matches!(backend, Backend::Deny { .. }));
        assert!(select_jailer(&backend).is_none());
        let decision = decide_bash(
            &Target::Jail(pb("/private/tmp/wt-abc")),
            "rm -rf /",
            "danger",
            &JailPolicy::default(),
            &backend,
        );
        match decision {
            Decision::Deny { reason } => {
                assert!(reason.starts_with(REASON_MAC_POLICY_MALFORMED));
            }
            other => panic!("malformed policy MUST deny, got {other:?}"),
        }
    }

    // EX-4 / T10 — the network bool flows resolver → profile: false ⇒ deny line,
    // omitted ⇒ default open ⇒ no deny line. Malformed already covered (branch f).
    #[test]
    fn network_bool_flows_from_policy_to_profile() {
        let closed = FakeEnv {
            policy: Ok(Some("network = false\n".to_string())),
            ..Default::default()
        };
        let mac = resolve(&closed).expect("valid closed-network policy");
        assert!(!mac.network);
        assert!(
            seatbelt_profile(&mac).contains(DENY_NETWORK),
            "network=false MUST emit the deny line"
        );

        let open = resolve(&FakeEnv::default()).unwrap();
        assert!(open.network);
        assert!(
            !seatbelt_profile(&open).contains(DENY_NETWORK),
            "default-open policy MUST NOT emit the deny line"
        );
    }

    // ---- RealEnv host-agnostic legs (git topology + policy read) ----------------
    //
    // The getconf leg is macOS-only (PHASE-04 in-situ); these prove the git-topology
    // (branch a/b) and policy-read (branch e/f) legs against REAL repos on any host,
    // reusing the subsystem's init_repo/git helpers (no parallel test infra).
    use crate::worktree::test_helpers::{git, init_repo};

    // branch (b) real — the primary tree is NOT a linked worktree ⇒ IsMainCheckout.
    #[test]
    fn real_env_topology_main_checkout_is_branch_b() {
        let tmp = tempfile::tempdir().unwrap();
        let primary = init_repo(&tmp.path().join("src"));
        let env = RealEnv {
            main_root: primary.clone(),
        };
        let topo = env
            .worktree_topology(&primary)
            .expect("primary is a worktree");
        assert!(!topo.is_linked, "primary tree is the main checkout");
        // and the full resolver denies at branch b (no getconf/policy reached).
        assert_eq!(
            resolve_inputs(&primary, &primary, &env),
            Err(ResolveDeny::IsMainCheckout)
        );
    }

    // branch (a) real — a non-git dir ⇒ NotAWorktree.
    #[test]
    fn real_env_topology_non_git_is_branch_a() {
        let tmp = tempfile::tempdir().unwrap();
        let plain = tmp.path().join("not-a-repo");
        std::fs::create_dir_all(&plain).unwrap();
        let env = RealEnv {
            main_root: plain.clone(),
        };
        assert_eq!(
            env.worktree_topology(&plain),
            Err(ResolveDeny::NotAWorktree)
        );
    }

    // linked worktree real — is_linked true; then read_policy absent ⇒ branch e.
    #[test]
    fn real_env_linked_worktree_then_policy_absent_is_branch_e() {
        let tmp = tempfile::tempdir().unwrap();
        let primary = init_repo(&tmp.path().join("src"));
        let fork = tmp.path().join("wt-xyz");
        git(
            &primary,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                "feat",
                fork.to_str().unwrap(),
            ],
        );
        let fork = std::fs::canonicalize(&fork).unwrap();
        let env = RealEnv {
            main_root: primary.clone(),
        };
        let topo = env.worktree_topology(&fork).expect("fork is a worktree");
        assert!(topo.is_linked, "a linked worktree");
        // no policy provisioned under <main>/.doctrine/state/dispatch/jail/ ⇒ branch e.
        assert_eq!(
            resolve_inputs(&fork, &primary, &env),
            Err(ResolveDeny::PolicyMissing)
        );
    }

    // read_policy present+valid, malformed, and unknown-key — the disk read → parse legs.
    #[test]
    fn real_env_read_policy_present_absent_malformed() {
        let tmp = tempfile::tempdir().unwrap();
        let main = init_repo(&tmp.path().join("src"));
        let mut jail_dir = main.clone();
        for seg in POLICY_DIR_SEGMENTS {
            jail_dir.push(seg);
        }
        std::fs::create_dir_all(&jail_dir).unwrap();
        std::fs::write(jail_dir.join("wt-good.toml"), "network = false\n").unwrap();
        std::fs::write(jail_dir.join("wt-bad.toml"), "network = \"x\"\n").unwrap();
        let env = RealEnv {
            main_root: main.clone(),
        };
        assert_eq!(
            env.read_policy(std::ffi::OsStr::new("wt-good")).unwrap(),
            Some("network = false\n".to_string())
        );
        assert_eq!(
            env.read_policy(std::ffi::OsStr::new("wt-missing")).unwrap(),
            None
        );
        // the malformed body parses to a branch-f Deny through from_toml_str.
        let bad = env
            .read_policy(std::ffi::OsStr::new("wt-bad"))
            .unwrap()
            .unwrap();
        assert!(JailPolicy::from_toml_str(&bad).is_err());
    }
}
