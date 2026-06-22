// SPDX-License-Identifier: GPL-3.0-only
//! Git seam — the born-frame producer for memory anchoring (SL-007, design §5.2).
//!
//! doctrine reproduces `the external decision register`'s frozen `GitContextFrameV1` algorithm
//! byte-for-byte so records and event-store claims derive the *same*
//! `repo_id`/`checkout_state_id` and dedup at the interop seam (design D2/D7).
//! This module is the **pure half** (PHASE-01): the frame data shapes plus the
//! config-independent derivations — canonical-bytes (DEC-009), sha256, the
//! `forget.remote.v1` remote normalizer, and the `forget.checkout.v1` checkout-id
//! composition. The impure `capture` (git subprocess) lands in PHASE-02.
//!
//! The data shape here is doctrine's own, flatter projection (design §5.2); only
//! the *derivation functions* are ported byte-identically. Byte-identity is the
//! contract — proven by the `normalize_remote_url` oracle table (VT-1) copied
//! verbatim from the external decision register and, in PHASE-02, a shared conformance golden-vector.
//!
//! No consumer is wired this phase — `record` (PHASE-04) and `verify` (PHASE-05)
//! pull it in. So the non-test build sees the module as dead; the tests exercise
//! it. The expectation lifts as each consumer lands (mirrors `memory.rs`).
#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "pure git seam; consumers wired by capture (PHASE-02), record (PHASE-04), verify (PHASE-05)"
    )
)]

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Number, Value};
use sha2::{Digest, Sha256};

/// Remote-URL normalizer tag (`forget.remote.v1`) — versions the algorithm so a
/// future change is detectable in persisted frames. Parity with the external decision register.
pub(crate) const REMOTE_NORMALIZER: &str = "forget.remote.v1";
/// Checkout-state hashing normalizer tag (`forget.checkout.v1`).
pub(crate) const CHECKOUT_NORMALIZER: &str = "forget.checkout.v1";

// ---------------------------------------------------------------------------
// Frame data shapes (design §5.2) — doctrine's flatter projection of the locked
// decision-6 frame. Names align with the persisted `[git]`/`[scope]` schema.
// ---------------------------------------------------------------------------

/// Which git coordinate a memory binds to (design §5.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AnchorKind {
    /// Clean checkout — anchored to an exact HEAD commit.
    Commit,
    /// Dirty checkout — anchored to a content-hashed `checkout_state_id`.
    CheckoutState,
    /// Unborn or non-repo — no stable anchor (a repo-scoped record here errors).
    None,
}

/// How a `repo_id` was derived, in precedence order (`forget.remote.v1`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RepoIdKind {
    /// Explicit override (`--repo` / pinned config).
    Explicit,
    /// Normalized remote URL.
    Remote,
    /// `repo:git-root:<root_sha>` fallback.
    LocalRoot,
}

/// Confidence that a `repo_id` converges across clones — the partition/security
/// trust signal (design §5.2: remote/explicit = high; local-root = medium/low).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Confidence {
    /// Globally shareable (explicit or remote).
    High,
    /// Shareable with caution (local root, born).
    Medium,
    /// Best-effort only (local root, unborn).
    Low,
}

// ---------------------------------------------------------------------------
// Persisted string forms (the `[git]`/`[scope]` snake_case tokens). These pin
// the frame's on-disk vocabulary — the read path (`memory.rs` validation) parses
// them; the write/render paths (PHASE-04/06) emit them. Mirrors the
// `MemoryType::parse`/`as_str` pattern in `memory.rs`; the persisted spelling is
// fixed here and both ends must agree. Empty→default normalization is NOT here —
// it is explicit in `memory.rs` validation (design D4/M1).
// ---------------------------------------------------------------------------

impl AnchorKind {
    /// Parse a persisted `anchor_kind` token. `""`→`None` is handled by the
    /// caller's explicit normalization, not here (a bare token only).
    pub(crate) fn parse(s: &str) -> Result<Self, String> {
        Ok(match s {
            "commit" => Self::Commit,
            "checkout_state" => Self::CheckoutState,
            "none" => Self::None,
            other => return Err(format!("unknown anchor_kind {other:?}")),
        })
    }

    /// The persisted token (inverse of `parse`).
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Commit => "commit",
            Self::CheckoutState => "checkout_state",
            Self::None => "none",
        }
    }
}

impl RepoIdKind {
    /// Parse a persisted `repo_id_kind` token.
    pub(crate) fn parse(s: &str) -> Result<Self, String> {
        Ok(match s {
            "explicit" => Self::Explicit,
            "remote" => Self::Remote,
            "local_root" => Self::LocalRoot,
            other => return Err(format!("unknown repo_id_kind {other:?}")),
        })
    }

    /// The persisted token (inverse of `parse`).
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Explicit => "explicit",
            Self::Remote => "remote",
            Self::LocalRoot => "local_root",
        }
    }
}

impl Confidence {
    /// Parse a persisted `confidence` token.
    pub(crate) fn parse(s: &str) -> Result<Self, String> {
        Ok(match s {
            "high" => Self::High,
            "medium" => Self::Medium,
            "low" => Self::Low,
            other => return Err(format!("unknown confidence {other:?}")),
        })
    }

    /// The persisted token (inverse of `parse`).
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

/// Stable repository identity (design §5.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RepoIdentity {
    /// Normalized `host[:port]/path`, `repo:git-root:<sha>`, or `""` when unscoped.
    pub repo_id: String,
    /// How `repo_id` was derived.
    pub kind: RepoIdKind,
    /// Convergence confidence.
    pub confidence: Confidence,
}

/// The full locked-decision-6 born frame (design §5.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Frame {
    /// The anchor coordinate kind.
    pub anchor_kind: AnchorKind,
    /// Repository identity.
    pub repo: RepoIdentity,
    /// HEAD commit SHA — set iff `anchor_kind == Commit` (clean).
    pub commit: String,
    /// HEAD `^{tree}` SHA.
    pub tree: String,
    /// Symbolic ref (`refs/heads/…`); `""` when detached (still anchored).
    pub ref_name: String,
    /// Content-bearing dirty-state id — set iff `anchor_kind == CheckoutState`.
    pub checkout_state_id: String,
    /// HEAD the memory sits on (always set when born; clean *and* dirty).
    pub base_commit: String,
}

// ---------------------------------------------------------------------------
// Canonical JSON bytes (DEC-009) + sha256 — mirrored from the external decision register's
// `canonical.rs` / `hash.rs` so the hashed bytes are byte-identical.
//
// A *protocol*, not "whatever serde_json emits": object keys sorted ascending
// bytewise, minimal string escaping, integer-only numbers, no insignificant
// whitespace. Floats are rejected (the integer-only interop constraint).
// ---------------------------------------------------------------------------

/// Lowercase hex digits for `\u00XX` control-character escapes.
const HEX: &[u8; 16] = b"0123456789abcdef";

/// A payload number was not an integer expressible as `i64`/`u64` — a fractional
/// or exponent form. Floats are out of scope for the v1 frame (integer/string only).
#[derive(Debug, thiserror::Error)]
#[error("non-integer number in canonical payload: {0}")]
pub(crate) struct NonIntegerNumber(pub String);

/// Encode `value` to canonical JSON bytes per DEC-009.
///
/// # Errors
///
/// Returns [`NonIntegerNumber`] if any number is not an exact `i64`/`u64`
/// integer; floats and exponent forms are rejected in v1.
pub(crate) fn canonical_bytes(value: &Value) -> Result<Vec<u8>, NonIntegerNumber> {
    let mut out = Vec::new();
    write_value(value, &mut out)?;
    Ok(out)
}

fn write_value(value: &Value, out: &mut Vec<u8>) -> Result<(), NonIntegerNumber> {
    match value {
        Value::Null => out.extend_from_slice(b"null"),
        Value::Bool(true) => out.extend_from_slice(b"true"),
        Value::Bool(false) => out.extend_from_slice(b"false"),
        Value::Number(n) => write_number(n, out)?,
        Value::String(s) => write_string(s, out),
        Value::Array(items) => {
            out.push(b'[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                write_value(item, out)?;
            }
            out.push(b']');
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
            out.push(b'{');
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                write_string(key, out);
                out.push(b':');
                // `key` is drawn from `map.keys()`, so the lookup always hits.
                if let Some(v) = map.get(key.as_str()) {
                    write_value(v, out)?;
                }
            }
            out.push(b'}');
        }
    }
    Ok(())
}

fn write_number(n: &Number, out: &mut Vec<u8>) -> Result<(), NonIntegerNumber> {
    if let Some(i) = n.as_i64() {
        out.extend_from_slice(i.to_string().as_bytes());
        Ok(())
    } else if let Some(u) = n.as_u64() {
        out.extend_from_slice(u.to_string().as_bytes());
        Ok(())
    } else {
        Err(NonIntegerNumber(n.to_string()))
    }
}

fn write_string(s: &str, out: &mut Vec<u8>) {
    out.push(b'"');
    for c in s.chars() {
        match c {
            '"' => out.extend_from_slice(b"\\\""),
            '\\' => out.extend_from_slice(b"\\\\"),
            '\u{08}' => out.extend_from_slice(b"\\b"),
            '\u{09}' => out.extend_from_slice(b"\\t"),
            '\u{0A}' => out.extend_from_slice(b"\\n"),
            '\u{0C}' => out.extend_from_slice(b"\\f"),
            '\u{0D}' => out.extend_from_slice(b"\\r"),
            c if u32::from(c) < 0x20 => write_control_escape(c, out),
            c => {
                let mut buf = [0_u8; 4];
                out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
            }
        }
    }
    out.push(b'"');
}

/// Emit a `\u00XX` escape (lowercase hex) for a control char below `U+0020`
/// with no short escape. `c` is `< 0x20`, so both nibbles are in range.
fn write_control_escape(c: char, out: &mut Vec<u8>) {
    let code = u32::from(c);
    let hi = usize::try_from((code >> 4) & 0xf).unwrap_or(0);
    let lo = usize::try_from(code & 0xf).unwrap_or(0);
    out.extend_from_slice(b"\\u00");
    if let Some(&h) = HEX.get(hi) {
        out.push(h);
    }
    if let Some(&l) = HEX.get(lo) {
        out.push(l);
    }
}

/// Lowercase-hex sha256 of `bytes` (git-context fingerprints, `checkout_state_id`).
/// Byte-identical to the external decision register's `hash::sha256`.
pub(crate) fn sha256(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

// ---------------------------------------------------------------------------
// Checkout-state id (`forget.checkout.v1`) — content-bearing dirty-tree hash.
// ---------------------------------------------------------------------------

/// Compute a `checkout_state_id` from the three content fingerprints.
///
/// Canonicalized + sha256'd under [`CHECKOUT_NORMALIZER`] so the id is
/// config-independent and reproducible. Distinct edits to the same fileset do
/// not collide (each fingerprint participates).
pub(crate) fn checkout_state_id(
    index_tree: &str,
    worktree_fingerprint: &str,
    untracked_fingerprint: &str,
) -> String {
    let value = serde_json::json!({
        "normalizer": CHECKOUT_NORMALIZER,
        "index_tree": index_tree,
        "worktree_fingerprint": worktree_fingerprint,
        "untracked_fingerprint": untracked_fingerprint,
    });
    // The composed value is all strings — canonicalization never errors.
    sha256(&canonical_bytes(&value).unwrap_or_default())
}

// ---------------------------------------------------------------------------
// Remote-URL normalization (`forget.remote.v1`).
// ---------------------------------------------------------------------------

/// A remote URL normalized to its routing identity (`forget.remote.v1`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NormalizedRemote {
    /// Lowercased host.
    pub host: String,
    /// Port, preserved only when non-default for the scheme.
    pub port: Option<u16>,
    /// Path with leading/trailing slashes and a single trailing `.git` stripped
    /// (case preserved).
    pub path: String,
    /// `host[:port]/path` — the derived `repo_id`.
    pub repo_id: String,
}

#[derive(Debug, Clone, Copy)]
struct SchemeInfo {
    default_port: u16,
    drop_default: bool,
}

/// Scheme routing info. `git://` keeps its default port (9418) so it never
/// coalesces with ssh/https on the same host/path (B3).
fn scheme_info(scheme: &str) -> Option<SchemeInfo> {
    match scheme {
        "ssh" => Some(SchemeInfo {
            default_port: 22,
            drop_default: true,
        }),
        "https" => Some(SchemeInfo {
            default_port: 443,
            drop_default: true,
        }),
        "http" => Some(SchemeInfo {
            default_port: 80,
            drop_default: true,
        }),
        "git" => Some(SchemeInfo {
            default_port: 9418,
            drop_default: false,
        }),
        _ => None,
    }
}

/// Strip leading/trailing `/` and a single trailing `.git`; preserve case.
fn clean_path(path: &str) -> String {
    let trimmed = path.trim_matches('/');
    let without_git = trimmed.strip_suffix(".git").unwrap_or(trimmed);
    without_git.trim_end_matches('/').to_string()
}

/// Split a `host[:port]` token, applying the scheme's default-port drop rule.
fn host_and_port(hostport: &str, scheme: SchemeInfo) -> (String, Option<u16>) {
    let (host, explicit_port) = match hostport.rsplit_once(':') {
        Some((h, p)) => (h, p.parse::<u16>().ok()),
        None => (hostport, None),
    };
    let port = explicit_port.unwrap_or(scheme.default_port);
    let rendered = if scheme.drop_default && port == scheme.default_port {
        None
    } else {
        Some(port)
    };
    (host.to_lowercase(), rendered)
}

/// Normalize a remote URL to a `repo_id` (`forget.remote.v1`).
///
/// Handles URL forms (`ssh://`, `https://`, `http://`, `git://`) and scp-short
/// (`git@host:org/repo`). Drops scheme + userinfo; preserves non-default ports;
/// lowercases the host; preserves path case. Returns `None` for unrecognized input.
pub(crate) fn normalize_remote_url(raw: &str) -> Option<NormalizedRemote> {
    let raw = raw.trim();
    let (host, port, path) = if let Some(idx) = raw.find("://") {
        let scheme = scheme_info(raw.get(..idx)?)?;
        let rest = raw.get(idx + 3..)?;
        let after_user = rest.rsplit_once('@').map_or(rest, |(_, h)| h);
        let (hostport, path) = after_user
            .split_once('/')
            .map_or((after_user, ""), |(h, p)| (h, p));
        let (host, port) = host_and_port(hostport, scheme);
        (host, port, clean_path(path))
    } else if let Some((hostpart, path)) = raw.split_once(':') {
        // scp-short form: [user@]host:path (no port; ssh defaults).
        let host = hostpart.rsplit_once('@').map_or(hostpart, |(_, h)| h);
        if host.is_empty() || path.is_empty() {
            return None;
        }
        (host.to_lowercase(), None, clean_path(path))
    } else {
        return None;
    };

    if host.is_empty() || path.is_empty() {
        return None;
    }
    let repo_id = match port {
        Some(p) => format!("{host}:{p}/{path}"),
        None => format!("{host}/{path}"),
    };
    Some(NormalizedRemote {
        host,
        port,
        path,
        repo_id,
    })
}

// ---------------------------------------------------------------------------
// Capture (git I/O) — the impure half (design §5.2). Shells `git` under the
// normative flags so machine-local config cannot perturb the frame. Ported from
// the external decision register's `git_context::capture`, projected down to doctrine's flat `Frame`.
// ---------------------------------------------------------------------------

/// Normative git config flags applied to **every** invocation (EX-1) so local
/// config (autocrlf/eol/fileMode) cannot perturb captured trees/diffs/hashes —
/// required for byte-identity with the external decision register.
const NORMATIVE_FLAGS: &[&str] = &[
    "-c",
    "core.autocrlf=false",
    "-c",
    "core.eol=lf",
    "-c",
    "core.fileMode=true",
];

/// Config key holding an explicit, user-pinned `repo_id` (precedence slot 1).
const CONFIG_EXPLICIT_REPO_ID: &str = "doctrine.repo.id";
/// Config key naming the preferred remote for `repo_id` derivation. (No `_`:
/// git config keys are alphanumeric/`-` only — an underscore is an invalid key.)
const CONFIG_PREFERRED_REMOTE: &str = "doctrine.repo.preferredremote";

/// Failures that abort a [`capture`] (design §5.2, F2 resolution).
///
/// Only the **unstable-frame guards** + git failures are errors. Unborn and
/// non-repo are *not* errors — they are `Ok(Frame{anchor_kind: None})` per design
/// §5.5; a repo-scoped `record` over a `None` frame is what errors, at the
/// `record` layer (PHASE-04, constraint 4). Spawn/UTF-8/non-zero-exit all fold
/// into [`CaptureError::Git`].
#[derive(Debug, thiserror::Error)]
pub(crate) enum CaptureError {
    /// More than one root commit reachable from HEAD — unstable to anchor.
    #[error("unsupported: multi-root repository ({0} root commits)")]
    MultiRoot(usize),
    /// A gitlink (submodule) index entry (mode 160000) — unstable to hash.
    #[error("unsupported: submodule entry (gitlink mode 160000)")]
    Submodule,
    /// Multiple remotes with no `origin`/preferred remote — no deterministic pick.
    #[error("ambiguous remote selection: multiple remotes without origin: {0:?}")]
    AmbiguousRemote(Vec<String>),
    /// A git invocation failed to spawn, exited non-zero, or returned non-UTF-8.
    #[error("git command failed: {0}")]
    Git(String),
    /// A filesystem operation (e.g. `readlink` on an untracked symlink) failed.
    #[error("io error during capture: {0}")]
    Io(String),
}

/// Run `git -C <root> <normative-flags> <args>`, capturing output. The single
/// chokepoint that applies [`NORMATIVE_FLAGS`] (EX-1).
fn run_git(root: &Path, args: &[&str]) -> Result<std::process::Output, CaptureError> {
    run_git_env(root, args, &[])
}

/// [`run_git`] with extra environment threaded onto the child — the seam for
/// plumbing that redirects `GIT_INDEX_FILE` to a throwaway index (`filter_tree`,
/// EX-2) so the live coordination index is never touched. The single
/// [`NORMATIVE_FLAGS`] chokepoint is preserved (EX-1 — no second runner; the
/// born-frame callers route through here unchanged via `run_git`).
fn run_git_env(
    root: &Path,
    args: &[&str],
    envs: &[(&str, &std::ffi::OsStr)],
) -> Result<std::process::Output, CaptureError> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(root).args(NORMATIVE_FLAGS).args(args);
    for (key, val) in envs {
        cmd.env(key, val);
    }
    cmd.output()
        .map_err(|e| CaptureError::Git(format!("spawn git {}: {e}", args.join(" "))))
}

/// Run a git command, erroring on non-zero exit; return raw stdout bytes.
/// `pub(crate)` so the worktree provisioner can drive `git ls-files -z` through
/// the one normative-flag chokepoint rather than forking a second runner
/// (SL-029 T6 / R-a — generic plumbing, not born-frame internals).
pub(crate) fn git_bytes(root: &Path, args: &[&str]) -> Result<Vec<u8>, CaptureError> {
    let output = run_git(root, args)?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(CaptureError::Git(format!(
            "{}: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

/// Run a git command expecting trimmed UTF-8 stdout (errors on non-zero exit).
/// `pub(crate)` for the worktree provisioner's `rev-parse --git-common-dir`
/// sibling-worktree check (SL-029 T6).
pub(crate) fn git_text(root: &Path, args: &[&str]) -> Result<String, CaptureError> {
    let bytes = git_bytes(root, args)?;
    let text = String::from_utf8(bytes)
        .map_err(|_ignored| CaptureError::Git(format!("non-utf8 output: {}", args.join(" "))))?;
    Ok(text.trim().to_string())
}

/// Run a git command that may legitimately fail; `None` on non-zero exit.
/// `pub(crate)` for the worktree fork verb's "is `<B>` a commit?" probe
/// (`rev-parse --verify --quiet <B>^{commit}`, SL-056 PHASE-06).
pub(crate) fn git_opt(root: &Path, args: &[&str]) -> Result<Option<String>, CaptureError> {
    let output = run_git(root, args)?;
    if !output.status.success() {
        return Ok(None);
    }
    let text = String::from_utf8(output.stdout)
        .map_err(|_ignored| CaptureError::Git(format!("non-utf8 output: {}", args.join(" "))))?;
    Ok(Some(text.trim().to_string()))
}

/// Apply a unified-diff `patch` into the index via `git apply --3way --index`,
/// NON-committing (SL-056 PHASE-07 import: the orchestrator commits separately,
/// ADR-006 D7). The patch is streamed on stdin as RAW BYTES — `git apply`
/// requires a newline-terminated stream, so the caller must hand the diff over
/// verbatim (via `git_bytes`, not `git_text`, whose `.trim()` would strip the
/// trailing newline and corrupt a hunk that ends at EOF — ISS-032). A non-zero
/// exit (a real conflict or malformed patch) errors. Invoked from the
/// coordination root so the index it writes is the coordination index. Impure
/// shell only.
pub(crate) fn git_apply_index(root: &Path, patch: &[u8]) -> Result<(), CaptureError> {
    use std::io::Write as _;
    use std::process::Stdio;

    let mut child = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(NORMATIVE_FLAGS)
        .args(["apply", "--3way", "--index"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| CaptureError::Git(format!("spawn git apply: {e}")))?;
    child
        .stdin
        .take()
        .ok_or_else(|| CaptureError::Git("git apply: no stdin pipe".to_owned()))?
        .write_all(patch)
        .map_err(|e| CaptureError::Git(format!("git apply: write stdin: {e}")))?;
    let output = child
        .wait_with_output()
        .map_err(|e| CaptureError::Git(format!("git apply: wait: {e}")))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(CaptureError::Git(format!(
            "apply --3way --index: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

/// Run `git cherry <upstream> <head>` and return its `± <sha>` lines verbatim
/// (SL-056 PHASE-09 gc oracle, design §8.1 patch-id leg). Each line is `- <sha>`
/// (an equivalent patch is already upstream) or `+ <sha>` (a commit whose patch is
/// NOT upstream). Errors on non-zero exit (e.g. an unresolvable ref). Impure shell;
/// the gc classifier reads the prefixes as facts, never the other way round.
pub(crate) fn git_cherry(
    root: &Path,
    upstream: &str,
    head: &str,
) -> Result<Vec<String>, CaptureError> {
    let text = git_text(root, &["cherry", upstream, head])?;
    Ok(text.lines().map(str::to_owned).collect())
}

/// True iff `git <args>` exits 0 (the exit-status-only seam — SL-056 PHASE-09 gc
/// oracle ancestry leg, `merge-base --is-ancestor <a> <b>`, which prints nothing
/// and signals purely via exit code). [`git_opt`] cannot distinguish exit-0-empty
/// from a real failure cleanly here, so this returns the boolean exit directly. A
/// spawn failure still errors (a missing git binary is not "false"). Impure shell.
pub(crate) fn git_status_ok(root: &Path, args: &[&str]) -> Result<bool, CaptureError> {
    Ok(run_git(root, args)?.status.success())
}

// --- SL-064 PHASE-03: projection plumbing (working-tree-free) ----------------

/// Run a git command with extra env, erroring on non-zero exit; trimmed UTF-8
/// stdout. The env-aware sibling of [`git_text`] — used by the tree-filter
/// primitive which must thread a throwaway `GIT_INDEX_FILE`.
fn git_env_text(
    root: &Path,
    args: &[&str],
    envs: &[(&str, &std::ffi::OsStr)],
) -> Result<String, CaptureError> {
    let output = run_git_env(root, args, envs)?;
    if output.status.success() {
        let text = String::from_utf8(output.stdout).map_err(|_ignored| {
            CaptureError::Git(format!("non-utf8 output: {}", args.join(" ")))
        })?;
        Ok(text.trim().to_string())
    } else {
        Err(CaptureError::Git(format!(
            "{}: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

/// A throwaway `GIT_INDEX_FILE` inside the repo's git dir, removed on drop. Lets
/// [`filter_tree`] stage into a scratch index that is **never** the live
/// coordination index (EX-2). Single-writer orchestrator ⇒ the pid-suffixed name
/// is collision-free. `new` sweeps **every** `doctrine-filter-index.*` sibling up
/// front, reclaiming the crash debris a hard-killed prior run leaves behind (its
/// `Drop` never fired, RV-030 F-5) — not just our same-pid path (which is fresh
/// anyway). An absent index file is an empty index to git.
struct ScratchIndex {
    path: std::path::PathBuf,
}

impl ScratchIndex {
    fn new(root: &Path) -> Result<Self, CaptureError> {
        let git_dir = git_text(root, &["rev-parse", "--absolute-git-dir"])?;
        let git_dir = Path::new(&git_dir);
        // Sweep cross-PID crash debris: the single-writer orchestrator owns the
        // `doctrine-filter-index.*` namespace, so every sibling is a leftover from
        // a prior run whose Drop never ran. Reclaim them all (our own same-pid
        // path included — remove_file's ENOENT is ignored).
        if let Ok(entries) = std::fs::read_dir(git_dir) {
            for entry in entries.flatten() {
                if entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("doctrine-filter-index.")
                {
                    drop(std::fs::remove_file(entry.path()));
                }
            }
        }
        let name = format!("doctrine-filter-index.{}", std::process::id());
        let path = git_dir.join(name);
        Ok(Self { path })
    }
}

impl Drop for ScratchIndex {
    fn drop(&mut self) {
        drop(std::fs::remove_file(&self.path));
    }
}

/// Build a filtered tree from `source_tree` with `exclude` pathspecs dropped,
/// staging through a throwaway `GIT_INDEX_FILE` so the live coordination index
/// and working tree are never touched (EX-2, design §4.1). Pure ref/object
/// plumbing — `read-tree` loads the scratch index, `rm --cached` drops the
/// excluded pathspecs, `write-tree` emits the new tree oid. No checkout. Returns
/// the filtered tree oid.
pub(crate) fn filter_tree(
    root: &Path,
    source_tree: &str,
    exclude: &[&str],
) -> Result<String, CaptureError> {
    let scratch = ScratchIndex::new(root)?;
    let env: [(&str, &std::ffi::OsStr); 1] = [("GIT_INDEX_FILE", scratch.path.as_os_str())];
    git_env_text(root, &["read-tree", source_tree], &env)?;
    if !exclude.is_empty() {
        let mut args = vec![
            "rm",
            "--cached",
            "-r",
            "-f",
            "--ignore-unmatch",
            "--quiet",
            "--",
        ];
        args.extend_from_slice(exclude);
        git_env_text(root, &args, &env)?;
    }
    git_env_text(root, &["write-tree"], &env)
}

/// Hash `content` into a blob object via `git hash-object -w --stdin`, returning
/// the blob oid. Streams the bytes on stdin (no temp file); writes the object to
/// the db without touching any index or working tree. The journal-commit
/// primitive's blob source ([`tree_with_file`]).
fn hash_object_stdin(root: &Path, content: &str) -> Result<String, CaptureError> {
    use std::io::Write as _;
    use std::process::Stdio;

    let mut child = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(NORMATIVE_FLAGS)
        .args(["hash-object", "-w", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| CaptureError::Git(format!("spawn git hash-object: {e}")))?;
    child
        .stdin
        .take()
        .ok_or_else(|| CaptureError::Git("git hash-object: no stdin pipe".to_owned()))?
        .write_all(content.as_bytes())
        .map_err(|e| CaptureError::Git(format!("git hash-object: write stdin: {e}")))?;
    let output = child
        .wait_with_output()
        .map_err(|e| CaptureError::Git(format!("git hash-object: wait: {e}")))?;
    if output.status.success() {
        let text = String::from_utf8(output.stdout)
            .map_err(|_ignored| CaptureError::Git("hash-object: non-utf8 oid".to_owned()))?;
        Ok(text.trim().to_string())
    } else {
        Err(CaptureError::Git(format!(
            "hash-object: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

/// Splice `content` into `base_tree` at `path`, returning the new tree oid. Stages
/// through a throwaway `GIT_INDEX_FILE` (never the live index, like
/// [`filter_tree`]): `read-tree <base_tree>`, hash the blob, `update-index --add
/// --cacheinfo` at `path`, `write-tree`. No checkout — the journal-commit
/// primitive (design §4.3: journal appends commit onto `dispatch/<slice>` via
/// `commit_tree`, no working tree). `path` is repo-relative.
pub(crate) fn tree_with_file(
    root: &Path,
    base_tree: &str,
    path: &str,
    content: &str,
) -> Result<String, CaptureError> {
    let scratch = ScratchIndex::new(root)?;
    let env: [(&str, &std::ffi::OsStr); 1] = [("GIT_INDEX_FILE", scratch.path.as_os_str())];
    git_env_text(root, &["read-tree", base_tree], &env)?;
    let blob = hash_object_stdin(root, content)?;
    let cacheinfo = format!("100644,{blob},{path}");
    git_env_text(
        root,
        &["update-index", "--add", "--cacheinfo", &cacheinfo],
        &env,
    )?;
    git_env_text(root, &["write-tree"], &env)
}

/// Read the blob at `path` from `refish`'s committed tree (`git cat-file -p
/// <refish>:<path>`), `None` when the path is absent from that tree. Working-tree-
/// free — reads the object db, so the sync verb sources the run ledger from the
/// `dispatch/<slice>` tip identically in stage-1 (worktree present) and stage-2
/// (worktree removed, no checkout — design §4.1).
pub(crate) fn read_path_at(
    root: &Path,
    refish: &str,
    path: &str,
) -> Result<Option<String>, CaptureError> {
    git_opt(root, &["cat-file", "-p", &format!("{refish}:{path}")])
}

/// Commit `tree` against `parent` with no working-tree touch (design §4.1) — the
/// B/C compose step's commit primitive. Returns the new commit oid.
pub(crate) fn commit_tree(
    root: &Path,
    tree: &str,
    parent: &str,
    msg: &str,
) -> Result<String, CaptureError> {
    git_text(root, &["commit-tree", tree, "-p", parent, "-m", msg])
}

/// Outcome of an explicit 3-way merge ([`merge_tree`]).
pub(crate) enum MergeTree {
    /// The merge applied cleanly; the union tree oid is carried.
    Clean { tree: String },
    /// The merge hit a content conflict — no tree is emitted (PHASE-03 lifecycle).
    Conflict,
}

/// Compute the 3-way union tree of `ours` and `theirs` against the common
/// ancestor `merge_base` via `git merge-tree --write-tree --merge-base=<mb>`
/// (git ≥ 2.38) — working-tree-free, object-db only. Exit 0 ⇒ a clean merge and
/// stdout is the written tree oid ([`MergeTree::Clean`]); a non-zero exit ⇒ a
/// content conflict ([`MergeTree::Conflict`]) with no tree written. A spawn /
/// usage failure still errors (a missing git is not "conflict"); a genuine
/// conflict (exit 1) is distinguished from a usage error (anything else) by the
/// exit code, mirroring [`is_ancestor`]. The candidate-create 3-way (design §5.3);
/// the no-ff merge commit is composed separately with [`commit_tree_merge`].
pub(crate) fn merge_tree(
    root: &Path,
    merge_base: &str,
    ours: &str,
    theirs: &str,
) -> Result<MergeTree, CaptureError> {
    let base_flag = format!("--merge-base={merge_base}");
    let output = run_git(
        root,
        &["merge-tree", "--write-tree", &base_flag, ours, theirs],
    )?;
    match output.status.code() {
        Some(0) => {
            let tree = String::from_utf8(output.stdout)
                .map_err(|_ignored| CaptureError::Git("merge-tree: non-utf8 oid".to_owned()))?;
            Ok(MergeTree::Clean {
                tree: tree.trim().to_string(),
            })
        }
        Some(1) => Ok(MergeTree::Conflict),
        _ => Err(CaptureError::Git(format!(
            "merge-tree --merge-base={merge_base} {ours} {theirs}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))),
    }
}

/// Commit `tree` as a 2-parent no-ff merge of `[first_parent, second_parent]`
/// (design §5.3 EX-3) — the candidate's merge commit, whose parents are the base
/// and the source so a diff against either is the genuine 3-way union (no phantom
/// `.doctrine` deletions). Working-tree-free; returns the merge commit oid.
pub(crate) fn commit_tree_merge(
    root: &Path,
    tree: &str,
    first_parent: &str,
    second_parent: &str,
    msg: &str,
) -> Result<String, CaptureError> {
    git_text(
        root,
        &[
            "commit-tree",
            tree,
            "-p",
            first_parent,
            "-p",
            second_parent,
            "-m",
            msg,
        ],
    )
}

/// Outcome of a compare-and-swap ref update ([`update_ref_cas`]).
pub(crate) enum RefCas {
    /// The ref equalled `expected_old` and was advanced to the new oid.
    Updated,
    /// The ref did **not** equal `expected_old`; nothing was written. `actual`
    /// is the ref's current value, or `None` if it does not exist.
    Moved { actual: Option<String> },
}

/// Compare-and-swap a ref via the native 3-arg `update-ref <ref> <new> <old>`
/// (design §4.1, ADR-012 D4): git advances the ref only if it currently equals
/// `expected_old`, otherwise refuses. For ref *creation*, pass the zero oid as
/// `expected_old` (git refuses if the ref already exists). On refusal the ref is
/// left untouched and the moved-target's actual value is reported — never forced,
/// never auto-resolved.
pub(crate) fn update_ref_cas(
    root: &Path,
    refname: &str,
    new_oid: &str,
    expected_old: &str,
) -> Result<RefCas, CaptureError> {
    let output = run_git(root, &["update-ref", refname, new_oid, expected_old])?;
    if output.status.success() {
        Ok(RefCas::Updated)
    } else {
        let actual = git_opt(root, &["rev-parse", "--verify", "--quiet", refname])?;
        Ok(RefCas::Moved { actual })
    }
}

/// The all-zero oid — the CAS `expected_old` sentinel for a ref *creation* (git's
/// `update-ref` refuses to create if the ref already exists). Shared by the
/// projection journal (stage-1 rows) and the replay (absent ref ↔ zero).
pub(crate) const ZERO_OID: &str = "0000000000000000000000000000000000000000";

/// Outcome of an idempotent 3-way replay ([`replay_ref`]).
#[derive(Debug)]
pub(crate) enum ReplayOutcome {
    /// `current == planned` already — the step is a verified no-op (the ref was
    /// applied on a prior run, or stage-1 created it). Nothing written.
    NoOp,
    /// `current == expected_old` — the CAS advanced the ref to `planned`.
    Applied,
    /// `current` matched neither `expected_old` nor `planned` — a moved target.
    /// Nothing written; `actual` is the ref's current value (`None` if absent).
    Moved { actual: Option<String> },
}

/// Idempotent compare-and-swap replay of one journal step (design §4.1, ADR-012
/// D4, EX-2). Resolves the ref's current oid (absent ↔ [`ZERO_OID`]) and:
/// `current == planned` ⇒ [`ReplayOutcome::NoOp`] (already applied — crash-safe
/// re-run); `current == expected_old` ⇒ [`update_ref_cas`] to `planned`
/// ([`ReplayOutcome::Applied`], or `Moved` on a TOCTOU race); divergence from
/// **both** ⇒ [`ReplayOutcome::Moved`] without writing. Never forces, never
/// auto-resolves — a moved target is reported, not clobbered.
pub(crate) fn replay_ref(
    root: &Path,
    refname: &str,
    expected_old: &str,
    planned: &str,
) -> Result<ReplayOutcome, CaptureError> {
    let actual = git_opt(root, &["rev-parse", "--verify", "--quiet", refname])?;
    let current = actual.as_deref().unwrap_or(ZERO_OID);
    if current == planned {
        Ok(ReplayOutcome::NoOp)
    } else if current == expected_old {
        match update_ref_cas(root, refname, planned, expected_old)? {
            RefCas::Updated => Ok(ReplayOutcome::Applied),
            // A concurrent writer raced between our resolve and the CAS.
            RefCas::Moved { actual: raced } => Ok(ReplayOutcome::Moved { actual: raced }),
        }
    } else {
        Ok(ReplayOutcome::Moved { actual })
    }
}

/// Resolve `refish` to its full commit oid via `rev-parse --verify --quiet
/// <refish>^{commit}` — `Ok(None)` when the ref/object is absent (the
/// `git_opt` success/None fold), distinct from a git/spawn failure. The
/// trunk-integration query (SL-126) uses it twice: an existence probe for
/// `dispatch/<slice>` (None ⇒ never dispatched) and to peel the trunk tip the
/// planned oid is ff-checked against. Peels to `^{commit}` so an annotated tag
/// or a ref-to-ref resolves to the underlying commit.
pub(crate) fn resolve_ref(root: &Path, refish: &str) -> Result<Option<String>, CaptureError> {
    let spec = format!("{refish}^{{commit}}");
    git_opt(root, &["rev-parse", "--verify", "--quiet", &spec])
}

/// True iff `ancestor` is an ancestor of (or equal to) `descendant`, via
/// `git merge-base --is-ancestor` (exit 0 ⇒ yes, exit 1 ⇒ no, anything else ⇒
/// error). Reads the raw exit code rather than [`git_opt`]'s success/None fold so
/// a legitimate "not an ancestor" is a clean `false`, never confused with a git
/// failure (the ff-only gate, EX-3; cousin of the masked-`cat-file -e` gotcha).
pub(crate) fn is_ancestor(
    root: &Path,
    ancestor: &str,
    descendant: &str,
) -> Result<bool, CaptureError> {
    let output = run_git(root, &["merge-base", "--is-ancestor", ancestor, descendant])?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(CaptureError::Git(format!(
            "merge-base --is-ancestor {ancestor} {descendant}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))),
    }
}

/// The parent oids of `commit`, in order, via `git rev-list --parents -n 1
/// <commit>` (the first whitespace token is the commit itself — dropped; the rest
/// are its parents). A root commit yields an empty vec; a 2-parent merge yields
/// `[first, second]`. The candidate-admit provenance check (SL-068 PHASE-05, I3)
/// compares this SET to `{base_oid, source_oid}` to prove `merge_oid` is the
/// Doctrine-created candidate merge.
pub(crate) fn parents(root: &Path, commit: &str) -> Result<Vec<String>, CaptureError> {
    let line = git_text(root, &["rev-list", "--parents", "-n", "1", commit])?;
    Ok(line.split_whitespace().skip(1).map(str::to_owned).collect())
}

/// The merge-base (best common ancestor) of `a` and `b` via `git merge-base <a>
/// <b>`. `Ok(None)` when the two share no common ancestor (exit 1 — unrelated
/// histories), kept distinct from a usage/spawn failure (anything else ⇒ error,
/// like [`is_ancestor`]). The dispatch projection's **pinned fork-point**:
/// stage-1 parents `review/<slice>` + `phase/<slice>-NN` on
/// merge-base(`dispatch/<slice>`, trunk), NOT the live trunk tip — so a foreign
/// commit landing on trunk mid-run cannot reparent the projection (design
/// §4.2/§4.3 `trunk_base_B`, RV-030 F-1). The live tip is used only at
/// integrate's trunk push under CAS.
pub(crate) fn merge_base(root: &Path, a: &str, b: &str) -> Result<Option<String>, CaptureError> {
    let output = run_git(root, &["merge-base", a, b])?;
    match output.status.code() {
        Some(0) => {
            let text = String::from_utf8(output.stdout)
                .map_err(|_ignored| CaptureError::Git("merge-base: non-utf8 oid".to_owned()))?;
            Ok(Some(text.trim().to_string()))
        }
        Some(1) => Ok(None),
        _ => Err(CaptureError::Git(format!(
            "merge-base {a} {b}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))),
    }
}

/// Resolve trunk's commit-ish via the peeled ladder (ADR-006 D3): an explicit
/// `DOCTRINE_TRUNK_REF`, else `origin/HEAD`, `main`, `master` in turn. Each
/// candidate is peeled with `rev-parse --verify --quiet <ref>^{commit}`; the
/// first that resolves wins. The whole ladder failing yields `Ok(None)` — a
/// repo with no trunk (fresh / no remote / detached) is a defined terminus, not
/// an error (R-2). **Asymmetry (F4/X6):** an *explicitly set* `DOCTRINE_TRUNK_REF`
/// that fails to peel is a hard error (the user pinned a bad ref — do not
/// silently fall through); only its *absence* descends to `origin/HEAD`.
fn trunk_tree_ish(root: &Path) -> anyhow::Result<Option<String>> {
    // Thin shell: the env read is the only impurity here. The ladder itself is
    // env-injected (`trunk_ladder`) so it is testable without mutating the
    // process environment — `set_var` is forbidden crate-wide (pure/imperative
    // split, CLAUDE.md: pass env in as an input).
    trunk_ladder(root, std::env::var_os("DOCTRINE_TRUNK_REF").as_deref())
}

/// Fold `candidates` (resolved shas, in ladder-preference order) toward the
/// freshest reachable base: start at the first, step to a later candidate
/// only when it is a *descendant* of the current pick; a diverged candidate
/// (not a descendant) is skipped, never regressed to. NOT a global maximum
/// (C2) — "preferred-order, advance-to-descendant". So a stale `origin/HEAD`
/// that is an ancestor of local `main` is overtaken by `main`, but a
/// `origin/HEAD` that has *diverged* from `main` is kept (preference wins,
/// never regress below the most-preferred resolvable ref).
fn freshest_descendant(root: &Path, candidates: &[String]) -> anyhow::Result<Option<String>> {
    candidates
        .iter()
        .try_fold(None::<String>, |acc, c| match acc {
            None => Ok(Some(c.clone())),
            Some(a) if is_ancestor(root, &a, c)? => Ok(Some(c.clone())), // c descends a ⇒ advance
            Some(a) => Ok(Some(a)),                                      // diverged/older ⇒ keep a
        })
}

/// The peeled trunk ladder with the explicit override injected (`explicit` is
/// `DOCTRINE_TRUNK_REF` when set). See [`trunk_tree_ish`] for the contract; the
/// asymmetry (F4/X6) lives here: an explicit ref that fails to peel is a hard
/// error, ladder candidates that fail simply fall through. The implicit arm
/// peels `origin/HEAD`, `main`, `master` IN THAT ORDER, then hands the resolved
/// shas (de-duplicated, first-seen order preserved) to [`freshest_descendant`]
/// — so a stale `origin/HEAD` that is an ancestor of local `main` is overtaken
/// by `main` rather than winning by first-resolves (design §2.2a, stance A, C2).
fn trunk_ladder(root: &Path, explicit: Option<&std::ffi::OsStr>) -> anyhow::Result<Option<String>> {
    let peel = |r: &str| -> anyhow::Result<Option<String>> {
        let spec = format!("{r}^{{commit}}");
        Ok(git_opt(root, &["rev-parse", "--verify", "--quiet", &spec])?)
    };
    if let Some(explicit) = explicit {
        let explicit = explicit.to_string_lossy();
        return match peel(&explicit)? {
            Some(sha) => Ok(Some(sha)),
            None => anyhow::bail!("DOCTRINE_TRUNK_REF={explicit} does not resolve to a commit"),
        };
    }
    let mut resolved: Vec<String> = Vec::new();
    for candidate in ["origin/HEAD", "main", "master"] {
        if let Some(sha) = peel(candidate)?
            && !resolved.contains(&sha)
        {
            resolved.push(sha);
        }
    }
    freshest_descendant(root, &resolved)
}

/// Resolve trunk's commit sha via the peeled ladder (ADR-006 D3) — public
/// wrapper over [`trunk_tree_ish`] for callers needing the integration base
/// (SL-064 `worktree coordinate`). `Ok(None)` when no trunk ref resolves.
pub(crate) fn trunk_commit(root: &Path) -> anyhow::Result<Option<String>> {
    trunk_tree_ish(root)
}

/// Numeric entity ids present under `kind_dir` on trunk's tree (ADR-006 D3).
/// `kind_dir` is ALREADY repo-relative including the `.doctrine/` prefix (X1) —
/// do NOT re-prepend. Lists trunk's tree with
/// `ls-tree -d --name-only <tree-ish> -- <kind_dir>/`; the trailing numeric
/// basename of each path is an id, non-numeric basenames ignored. No trunk
/// (`trunk_tree_ish` → None), an absent dir, or empty output all yield
/// `Ok(vec![])` — the local-only degradation (R-2).
pub(crate) fn trunk_entity_ids(root: &Path, kind_dir: &str) -> anyhow::Result<Vec<u32>> {
    let Some(tree_ish) = trunk_tree_ish(root)? else {
        return Ok(Vec::new());
    };
    let pathspec = format!("{kind_dir}/");
    let listing = git_opt(
        root,
        &["ls-tree", "-d", "--name-only", &tree_ish, "--", &pathspec],
    )?;
    let Some(listing) = listing else {
        return Ok(Vec::new());
    };
    let ids = listing
        .lines()
        .filter_map(|line| line.rsplit('/').next())
        .filter_map(|base| base.parse::<u32>().ok())
        .collect();
    Ok(ids)
}

/// PURE: scan `git worktree list --porcelain` text for the worktree path that has
/// `refname` (e.g. `refs/heads/main`) checked out, `None` if none does. The
/// porcelain stream is blank-line-separated blocks; each opens with a `worktree
/// <path>` line and, when a branch is checked out, carries a `branch
/// refs/heads/<name>` line. **Block-reset rule (M9):** a blank line clears the
/// pending path, so a `branch` line can only bind to the `worktree` line of its own
/// block — the more defensive of the two pre-extraction parses (a detached or
/// bare block leaves no stale path to mis-attribute). No I/O — fed by
/// [`worktree_for_ref`].
fn parse_worktree_for_ref(listing: &str, refname: &str) -> Option<PathBuf> {
    let mut current_path: Option<PathBuf> = None;
    for line in listing.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            current_path = Some(PathBuf::from(path));
        } else if let Some(branch) = line.strip_prefix("branch ") {
            if branch == refname {
                return current_path;
            }
        } else if line.is_empty() {
            current_path = None;
        }
    }
    None
}

/// The worktree path that has `refname` (e.g. `refs/heads/main`) checked out, or
/// `None` if no live worktree does. The single branch→worktree-path probe (SL-121
/// PHASE-01): one `git worktree list --porcelain` shell feeding the PURE
/// [`parse_worktree_for_ref`]. Thin impure half — a git failure is an `Err`
/// (callers fold it as they see fit), distinct from `Ok(None)` (the ref is simply
/// not checked out anywhere live).
///
/// # Errors
///
/// Returns [`CaptureError::Git`] if the `git worktree list` invocation fails.
pub(crate) fn worktree_for_ref(
    root: &Path,
    refname: &str,
) -> Result<Option<PathBuf>, CaptureError> {
    let listing = git_text(root, &["worktree", "list", "--porcelain"])?;
    Ok(parse_worktree_for_ref(&listing, refname))
}

/// True iff the *tracked* working tree at `root` is clean — `git status
/// --porcelain --untracked-files=no` empty (SL-121 §2.3). The single tracked-clean
/// predicate (leaf altitude, ADR-001): the integrate dirty pre-gate (dispatch.rs),
/// the §2.5 probe→merge re-check ([`ff_advance_in_worktree`]), and
/// `worktree::gather_tree_clean` all share it. Untracked scratch is deliberately
/// excluded — ephemeral files in a close session must not block a clean advance
/// (F1: an untracked *collision* is caught later by `merge --ff-only`'s own abort).
///
/// # Errors
///
/// Returns [`CaptureError::Git`] if the `git status` invocation fails.
pub(crate) fn tree_clean(root: &Path) -> Result<bool, CaptureError> {
    let status = git_text(root, &["status", "--porcelain", "--untracked-files=no"])?;
    Ok(status.is_empty())
}

/// Outcome of a guarded fast-forward advance of a checked-out ref
/// ([`ff_advance_in_worktree`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FfAdvance {
    /// The worktree's checked-out `target_ref` fast-forwarded to `planned`; ref +
    /// index + worktree all landed on `planned` with a clean tree.
    Advanced,
    /// A §2.5 race guard tripped — HEAD detached/switched off `target_ref`, the
    /// tree dirtied in the probe→merge window (M5), the merge aborted (e.g. an
    /// untracked collision, F1), or the post-merge ref did not land on `planned`.
    /// Nothing is claimed about the ref beyond "not a clean advance"; `token` is
    /// the caller's report fragment. Captured, never a bare `?`-`Err` (B3).
    Raced { token: String },
}

/// Fast-forward the checked-out `target_ref` in worktree `wt` to `planned` via
/// git's own `merge --ff-only` — the only primitive that advances ref + index +
/// worktree atomically (design §2.2). Guarded for the probe→merge TOCTOU
/// (§2.5/M6): before the merge HEAD must still be symbolically on `target_ref` and
/// the tracked tree clean (the M5 window `merge --ff-only` does NOT itself close);
/// after, `target_ref` must resolve to `planned`. Any guard failure or a merge
/// abort is a captured [`FfAdvance::Raced`] (the caller journals the row `Failed`),
/// NEVER a bare `?`-`Err` that would abort before status is durable (B3).
///
/// # Errors
///
/// Returns [`CaptureError::Git`] only for a genuine plumbing failure (a probe that
/// cannot run), never for a semantic race.
pub(crate) fn ff_advance_in_worktree(
    wt: &Path,
    target_ref: &str,
    planned: &str,
) -> Result<FfAdvance, CaptureError> {
    // Pre-guard: HEAD still symbolically attached to target_ref (not detached or
    // switched to a sibling branch — else the merge would advance the wrong ref).
    let head = git_opt(wt, &["symbolic-ref", "--quiet", "HEAD"])?;
    if head.as_deref() != Some(target_ref) {
        return Ok(FfAdvance::Raced {
            token: format!(
                "raced HEAD off {target_ref} (now {})",
                head.as_deref().unwrap_or("detached")
            ),
        });
    }
    // Pre-guard: tracked tree still clean (the M5 window merge --ff-only ignores).
    if !tree_clean(wt)? {
        return Ok(FfAdvance::Raced {
            token: format!("raced dirty worktree at {target_ref}"),
        });
    }
    // The atomic advance: ref + index + worktree together.
    let out = run_git(wt, &["merge", "--ff-only", planned])?;
    if !out.status.success() {
        return Ok(FfAdvance::Raced {
            token: format!(
                "ff-only merge aborted at {target_ref}: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ),
        });
    }
    // Post-condition: target landed exactly on planned (§2.2 post-assert).
    let now = git_opt(wt, &["rev-parse", "--verify", "--quiet", target_ref])?;
    if now.as_deref() != Some(planned) {
        return Ok(FfAdvance::Raced {
            token: format!(
                "post-merge {target_ref} at {} not planned {planned}",
                now.as_deref().unwrap_or("?")
            ),
        });
    }
    Ok(FfAdvance::Advanced)
}

/// Resync the checked-out worktree `wt`'s index + tracked tree onto `oid` via `git
/// reset --hard <oid>` — the None-leg post-CAS resync (design §2.2). When the race
/// fires, `update_ref_cas` has ALREADY advanced `target_ref` to `oid`, so the live
/// worktree's HEAD now resolves to `oid` while its index/tree are stale at the old
/// commit. `reset --keep`/`--merge` only touch files differing between `<oid>` and
/// HEAD — and HEAD == `oid` here (the ref moved under it), so they see no diff and
/// leave the desync UNFIXED (proven empirically). `reset --hard` sets index+tree to
/// `<oid>` unconditionally, immune to the HEAD-already-moved ordering. It is
/// content-safe ONLY because the caller invokes it after [`tree_clean`] confirmed
/// the tree clean (no tracked local modification to discard; untracked files
/// survive `reset --hard`); a failure here is a genuine plumbing fault,
/// `?`-propagated.
///
/// (Design §2.2 originally named `reset --keep`; that cannot fix the
/// already-advanced ref — corrected to `reset --hard` under the clean precondition,
/// folded back into the design at reconcile.)
///
/// # Errors
///
/// Returns [`CaptureError::Git`] if the `git reset --hard` invocation fails.
pub(crate) fn resync_worktree_hard(wt: &Path, oid: &str) -> Result<(), CaptureError> {
    git_text(wt, &["reset", "--hard", oid])?;
    Ok(())
}

/// Retry `git write-tree` with exponential backoff on transient index.lock
/// contention. Multiple concurrent doctrine processes (e.g. parallel `memory
/// verify` invocations) can race for the lock; this is harmless and resolves
/// as soon as the holder exits. 4 attempts, ~750 ms total.
fn write_tree_with_retry(repo_root: &Path) -> Result<String, CaptureError> {
    let mut delay_ms: u64 = 50;
    for _attempt in 0..4 {
        match git_text(repo_root, &["write-tree"]) {
            Ok(tree) => return Ok(tree),
            Err(CaptureError::Git(ref msg)) if msg.contains("index.lock") => {
                std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                delay_ms *= 2;
            }
            Err(e) => return Err(e),
        }
    }
    // Last attempt without catching — let the error surface.
    git_text(repo_root, &["write-tree"])
}

/// Capture the born frame for the working tree at `repo_root` (design §5.2).
///
/// Three Ok states: clean → [`AnchorKind::Commit`] (`commit`/`tree`/`base_commit`/
/// `ref_name`); dirty → [`AnchorKind::CheckoutState`] (`checkout_state_id`/`base_commit`,
/// `commit` empty); unborn or non-repo → [`AnchorKind::None`]. Detached HEAD is
/// still anchored with an empty `ref_name`. Submodule/multi-root/ambiguous-remote
/// trees error rather than emit an unstable anchor (D8). Symlinks are supported
/// (SL-012, mirrors the external decision register DE-010): tracked symlinks ride `index_tree`/
/// `worktree_fingerprint`, untracked symlinks hash by link text.
///
/// # Errors
///
/// Returns [`CaptureError`] for an unstable-frame guard or a failed git invocation.
pub(crate) fn capture(repo_root: &Path) -> Result<Frame, CaptureError> {
    // Non-repo → Ok None frame (design §5.5; `record` enforces the scope gate).
    match git_opt(repo_root, &["rev-parse", "--is-inside-work-tree"])? {
        Some(ref v) if v == "true" => {}
        _ => return Ok(none_frame()),
    }

    let head_commit = git_opt(repo_root, &["rev-parse", "--verify", "HEAD^{commit}"])?;
    let born = head_commit.is_some();
    let repo = derive_repo_identity(repo_root, born)?;

    // Unborn → Ok None frame (a repo, but no commit to anchor).
    let Some(commit) = head_commit else {
        return Ok(Frame {
            anchor_kind: AnchorKind::None,
            repo,
            commit: String::new(),
            tree: String::new(),
            ref_name: String::new(),
            checkout_state_id: String::new(),
            base_commit: String::new(),
        });
    };

    // Multi-root guard.
    let roots = git_text(repo_root, &["rev-list", "--max-parents=0", "HEAD"])?;
    let root_count = roots.lines().filter(|l| !l.is_empty()).count();
    if root_count > 1 {
        return Err(CaptureError::MultiRoot(root_count));
    }

    // HEAD anchor. Empty symbolic ref ⇒ detached, still anchored.
    let tree = git_text(repo_root, &["rev-parse", "HEAD^{tree}"])?;
    let ref_name = git_opt(repo_root, &["symbolic-ref", "--quiet", "HEAD"])?.unwrap_or_default();

    // Reject submodules before hashing (symlinks supported — SL-012/DE-010).
    reject_submodules(repo_root)?;

    // Content-based dirty detection (design §5.2).
    //
    // Use diff-index (lock-free) for the fast-path check — write-tree acquires
    // .git/index.lock, so we defer it to the rare dirty case where we need the
    // index tree for checkout_state_id. On a clean tree (>99% of invocations)
    // the lock is never touched, eliminating the dominant lock-contention source
    // when multiple doctrine processes run concurrently.
    let diff_bytes = git_bytes(
        repo_root,
        &["diff", "HEAD", "--binary", "--no-textconv", "--no-ext-diff"],
    )?;
    let untracked_fp = untracked_fingerprint(repo_root)?;
    let worktree_dirty = !diff_bytes.is_empty() || untracked_fp.is_some();

    // diff-index --quiet --cached HEAD: exit 0 = clean index (no staged changes).
    // This is a read-only operation — no lock acquired.
    let index_clean = git_opt(repo_root, &["diff-index", "--quiet", "--cached", "HEAD"])?.is_some();

    if !index_clean || worktree_dirty {
        // Tree is dirty — call write-tree (acquires lock) for the fingerprint.
        // Retry on transient index.lock contention (up to 4 attempts, ~750ms total).
        let index_tree = write_tree_with_retry(repo_root)?;
        let dirty = index_tree != tree || worktree_dirty;
        if dirty {
            let worktree_fp = sha256(&diff_bytes);
            let untracked = untracked_fp.unwrap_or_else(|| sha256(b""));
            Ok(Frame {
                anchor_kind: AnchorKind::CheckoutState,
                repo,
                commit: String::new(), // empty iff dirty
                tree,
                ref_name,
                checkout_state_id: checkout_state_id(&index_tree, &worktree_fp, &untracked),
                base_commit: commit, // HEAD always when born
            })
        } else {
            // write-tree showed clean but diff-index said dirty —
            // rare edge (e.g. file mode-only change invisible to diff-index).
            // Trust write-tree: the tree is clean.
            Ok(Frame {
                anchor_kind: AnchorKind::Commit,
                repo,
                commit: commit.clone(),
                tree,
                ref_name,
                checkout_state_id: String::new(),
                base_commit: commit,
            })
        }
    } else {
        // Index and worktree both clean — no lock acquired.
        Ok(Frame {
            anchor_kind: AnchorKind::Commit,
            repo,
            commit: commit.clone(),
            tree,
            ref_name,
            checkout_state_id: String::new(),
            base_commit: commit,
        })
    }
}

/// The unanchored, repo-empty frame a `record --global` master is minted from
/// (SL-018 PHASE-04): the global orientation class carries no repo coordinate and
/// asserts nothing about client git (design §5.3), so its born frame is suppressed
/// — identical to the unborn/non-repo frame (`repo_id=""`, anchor `none`). Riding
/// `none_frame` keeps a single construction site.
pub(crate) fn unanchored_frame() -> Frame {
    none_frame()
}

/// The `None`-anchor frame for an unborn/non-repo context: unscoped, lowest trust.
fn none_frame() -> Frame {
    Frame {
        anchor_kind: AnchorKind::None,
        repo: RepoIdentity {
            repo_id: String::new(),
            kind: RepoIdKind::LocalRoot,
            confidence: Confidence::Low,
        },
        commit: String::new(),
        tree: String::new(),
        ref_name: String::new(),
        checkout_state_id: String::new(),
        base_commit: String::new(),
    }
}

/// An explicit `repo_id` override (`--repo`, PHASE-04, or pinned config) → an
/// `Explicit`/`High` identity. Routed through [`normalize_remote_url`] so a
/// credentialed value is userinfo-stripped (design §5.2, R4); a non-URL value
/// (e.g. `org/project`) is kept verbatim.
pub(crate) fn explicit_identity(raw: &str) -> RepoIdentity {
    let raw = raw.trim();
    let repo_id = normalize_remote_url(raw).map_or_else(|| raw.to_string(), |n| n.repo_id);
    RepoIdentity {
        repo_id,
        kind: RepoIdKind::Explicit,
        confidence: Confidence::High,
    }
}

/// Derive [`RepoIdentity`] by precedence: explicit config → normalized remote →
/// local-root fallback (design §5.2 / EX-3).
fn derive_repo_identity(root: &Path, born: bool) -> Result<RepoIdentity, CaptureError> {
    let root_commit = if born {
        git_text(root, &["rev-list", "--max-parents=0", "HEAD"])?
            .lines()
            .next()
            .map(str::to_string)
    } else {
        None
    };

    // 1. Explicit config.
    if let Some(explicit) = git_opt(root, &["config", "--get", CONFIG_EXPLICIT_REPO_ID])?
        && !explicit.is_empty()
    {
        return Ok(explicit_identity(&explicit));
    }

    // 2. Normalized remote.
    let remotes = list_remotes(root)?;
    if let Some(selected) = select_remote(root, &remotes)? {
        let raw = git_text(root, &["remote", "get-url", &selected])?;
        if let Some(normalized) = normalize_remote_url(&raw) {
            return Ok(RepoIdentity {
                repo_id: normalized.repo_id,
                kind: RepoIdKind::Remote,
                confidence: Confidence::High,
            });
        }
    }

    // 3. Local-root fallback.
    let repo_id = match &root_commit {
        Some(sha) => format!("repo:git-root:{sha}"),
        None => "repo:git-root:unborn".to_string(),
    };
    Ok(RepoIdentity {
        repo_id,
        kind: RepoIdKind::LocalRoot,
        confidence: if born {
            Confidence::Medium
        } else {
            Confidence::Low
        },
    })
}

/// List configured remote names, sorted for a stable selection.
fn list_remotes(root: &Path) -> Result<Vec<String>, CaptureError> {
    let mut remotes: Vec<String> = git_text(root, &["remote"])?
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect();
    remotes.sort();
    Ok(remotes)
}

/// Select the remote for `repo_id`: preferred → `origin` → sole; `>1` with neither
/// is [`CaptureError::AmbiguousRemote`], not a guess (design §5.2).
fn select_remote(root: &Path, remotes: &[String]) -> Result<Option<String>, CaptureError> {
    if remotes.is_empty() {
        return Ok(None);
    }
    if let Some(preferred) = git_opt(root, &["config", "--get", CONFIG_PREFERRED_REMOTE])?
        && remotes.contains(&preferred)
    {
        return Ok(Some(preferred));
    }
    if remotes.iter().any(|r| r == "origin") {
        return Ok(Some("origin".to_string()));
    }
    match remotes.len() {
        1 => Ok(remotes.first().cloned()),
        _ => Err(CaptureError::AmbiguousRemote(remotes.to_vec())),
    }
}

/// Reject submodule (160000) index entries before hashing (D8).
///
/// Symlinks (120000) are supported (SL-012, mirrors the external decision register DE-010):
/// [`untracked_fingerprint`] encodes an untracked symlink by its link text, and
/// tracked symlinks ride `index_tree` / `worktree_fingerprint` as their `120000`
/// blob — so the up-front reject is unnecessary and over-rejected clean/tracked-only
/// trees. Submodules remain a distinct identity question (gitlink → nested-repo
/// commit) and stay deferred. Every `ls-files --stage` line is scanned, so a
/// `160000` entry at any merge stage is still caught.
fn reject_submodules(root: &Path) -> Result<(), CaptureError> {
    let staged = git_text(root, &["ls-files", "--stage"])?;
    for line in staged.lines() {
        // Format: "<mode> <sha> <stage>\t<path>".
        if let Some("160000") = line.split_whitespace().next() {
            return Err(CaptureError::Submodule);
        }
    }
    Ok(())
}

/// sha256 over sorted untracked-entry `path\0<hash>\n` records; `None` when there
/// are no untracked files. Each path is hashed by *identity*, never by following
/// links: a regular file via git's frozen blob hashing (`hash-object`), a symlink
/// via [`symlink_target_hash`] over its raw `readlink(2)` target bytes (SL-012,
/// mirrors the external decision register DE-010 §3.1). Regular-entry encoding is byte-identical to
/// before, so symlink-free csids do not move (DEC-010-06).
fn untracked_fingerprint(root: &Path) -> Result<Option<String>, CaptureError> {
    let raw = git_bytes(root, &["ls-files", "--others", "--exclude-standard", "-z"])?;
    let mut paths: Vec<&[u8]> = raw.split(|b| *b == 0).filter(|p| !p.is_empty()).collect();
    if paths.is_empty() {
        return Ok(None);
    }
    paths.sort_unstable();

    let mut acc: Vec<u8> = Vec::new();
    for path in paths {
        // Path key stays UTF-8 (pre-existing constraint); only a symlink *target*
        // is hashed as raw bytes.
        let path_str = std::str::from_utf8(path)
            .map_err(|_ignored| CaptureError::Git("non-utf8 untracked path".to_string()))?;
        let full = root.join(path_str);
        let is_symlink =
            std::fs::symlink_metadata(&full).is_ok_and(|meta| meta.file_type().is_symlink());
        let hash = if is_symlink {
            symlink_target_hash(&full)?
        } else {
            git_text(root, &["hash-object", "--", path_str])?
        };
        acc.extend_from_slice(path);
        acc.push(0);
        acc.extend_from_slice(hash.as_bytes());
        acc.push(b'\n');
    }
    Ok(Some(sha256(&acc)))
}

/// Hash an untracked symlink by its link-text identity: `sha256(` raw `readlink(2)`
/// target bytes `)`, never following the link (SL-012, mirrors the external decision register DE-010
/// §3.1) — robust to dangling/non-UTF-8 targets that `git hash-object` cannot read.
#[cfg(unix)]
fn symlink_target_hash(full: &Path) -> Result<String, CaptureError> {
    use std::os::unix::ffi::OsStrExt;
    let target = std::fs::read_link(full)
        .map_err(|e| CaptureError::Io(format!("readlink {}: {e}", full.display())))?;
    Ok(sha256(target.as_os_str().as_bytes()))
}

/// Non-Unix fallback. Symlink support is Unix-only in v0 (DEC-010-07); on
/// `core.symlinks=false` platforms a `120000` entry materializes as a regular
/// link-text file, so `is_symlink()` is false and this is not reached. Defined for
/// compilation parity, hashing the target's lossy bytes.
#[cfg(not(unix))]
fn symlink_target_hash(full: &Path) -> Result<String, CaptureError> {
    let target = std::fs::read_link(full)
        .map_err(|e| CaptureError::Io(format!("readlink {}: {e}", full.display())))?;
    Ok(sha256(target.to_string_lossy().as_bytes()))
}

/// Count commits in `since..target` that touch any of `paths` — the per-candidate
/// reachability fact PHASE-04 hands the pure ranker as
/// [`crate::retrieve::GitFacts::commits_since`]: `Some(0)` ⇒ Fresh, `Some(≥1)` ⇒
/// Stale, `None` ⇒ undecidable (design §5.2 / review B18).
///
/// Every failure folds to `None` so a single bad candidate degrades to
/// `Staleness::Unknown` — never aborting the whole query. `target` is ALWAYS a
/// frozen SHA resolved upstream, never the literal `HEAD` (codex F1): this seam
/// does not resolve HEAD.
pub(crate) fn commits_touching(
    root: &Path,
    paths: &[String],
    since: &str,
    target: &str,
) -> Option<u32> {
    // Cheap guards before any subprocess: empty paths (B17), and — defence in
    // depth — empty endpoints a caller slipped past the PHASE-04 gate.
    if paths.is_empty() || since.is_empty() || target.is_empty() {
        return None;
    }
    // F2 (mandatory, not an optimisation): `since..target` is a set difference
    // (`target`-reachable minus `since`-reachable), so a non-ancestor `since`
    // silently over-counts. Exit 1 (not an ancestor) and exit ≥2 (object absent /
    // shallow / error) both fail `success()` ⇒ None — no over-trust.
    let ancestry = run_git(root, &["merge-base", "--is-ancestor", since, target]).ok()?;
    if !ancestry.status.success() {
        return None;
    }
    let range = format!("{since}..{target}");
    let mut args = vec!["rev-list", "--count", &range, "--"];
    args.extend(paths.iter().map(String::as_str));
    git_opt(root, &args).ok().flatten()?.parse::<u32>().ok()
}

/// Resolve `HEAD` to its frozen commit SHA once, so the staleness seam
/// ([`commits_touching`], which REFUSES the literal `HEAD`) is fed a stable
/// `target`. Reuses the `rev-parse --verify HEAD^{commit}` form (the born-frame
/// capture seam, ~line 629). `None` on an unborn HEAD / non-repo / git failure —
/// the caller degrades every cell to `IsStale::Unknown`.
// SL-044 B·P2 wired the consumer (`reconcile` → `coverage_scan::scan_coverage` →
// `head_sha`), so this is live in the bins/lib build; the self-clearing `not(test)`
// dead_code expect retired itself as its reason foretold.
pub(crate) fn head_sha(root: &Path) -> Option<String> {
    git_opt(root, &["rev-parse", "--verify", "HEAD^{commit}"])
        .ok()
        .flatten()
}

// ---------------------------------------------------------------------------
// Unit tests — pure logic only (no git, no disk). The byte-identity proof for
// the remote table is copied verbatim from the external decision register's reference (VT-1).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use std::path::{Path, PathBuf};
    use std::process::Command;

    use super::{
        AnchorKind, CHECKOUT_NORMALIZER, CaptureError, Confidence, Frame, REMOTE_NORMALIZER,
        RepoIdKind, RepoIdentity, canonical_bytes, capture, checkout_state_id, commits_touching,
        explicit_identity, normalize_remote_url, parse_worktree_for_ref, sha256, worktree_for_ref,
    };

    /// Render canonical bytes as a `String` for readable assertions (canonical
    /// output is always valid UTF-8).
    fn canon(v: &Value) -> String {
        let bytes = canonical_bytes(v).unwrap_or_default();
        String::from_utf8(bytes).unwrap_or_default()
    }

    // --- PHASE-03: persisted enum tokens round-trip (pins the on-disk vocab) -

    #[test]
    fn anchor_kind_token_round_trips() {
        for k in [
            AnchorKind::Commit,
            AnchorKind::CheckoutState,
            AnchorKind::None,
        ] {
            assert_eq!(AnchorKind::parse(k.as_str()).unwrap(), k);
        }
        assert_eq!(
            AnchorKind::as_str(AnchorKind::CheckoutState),
            "checkout_state"
        );
        assert!(AnchorKind::parse("bogus").is_err());
    }

    #[test]
    fn repo_id_kind_token_round_trips() {
        for k in [
            RepoIdKind::Explicit,
            RepoIdKind::Remote,
            RepoIdKind::LocalRoot,
        ] {
            assert_eq!(RepoIdKind::parse(k.as_str()).unwrap(), k);
        }
        assert_eq!(RepoIdKind::as_str(RepoIdKind::LocalRoot), "local_root");
        assert!(RepoIdKind::parse("bogus").is_err());
    }

    #[test]
    fn confidence_token_round_trips() {
        for c in [Confidence::High, Confidence::Medium, Confidence::Low] {
            assert_eq!(Confidence::parse(c.as_str()).unwrap(), c);
        }
        assert!(Confidence::parse("bogus").is_err());
    }

    // --- EX-3: the frame types are constructible and field-addressable. -----

    fn sample_frame() -> Frame {
        Frame {
            anchor_kind: AnchorKind::Commit,
            repo: RepoIdentity {
                repo_id: "github.com/org/repo".to_string(),
                kind: RepoIdKind::Remote,
                confidence: Confidence::High,
            },
            commit: "abc123".to_string(),
            tree: "tree123".to_string(),
            ref_name: "refs/heads/main".to_string(),
            checkout_state_id: String::new(),
            base_commit: "abc123".to_string(),
        }
    }

    #[test]
    fn frame_carries_anchor_and_identity() {
        let f = sample_frame();
        assert_eq!(f.anchor_kind, AnchorKind::Commit);
        assert_eq!(f.repo.repo_id, "github.com/org/repo");
        assert_eq!(f.repo.kind, RepoIdKind::Remote);
        assert_eq!(f.repo.confidence, Confidence::High);
        assert_eq!(f.commit, f.base_commit);
        assert_eq!(f.tree, "tree123");
        assert_eq!(f.ref_name, "refs/heads/main");
        assert!(f.checkout_state_id.is_empty());
    }

    #[test]
    fn frame_variants_are_distinct() {
        assert_ne!(AnchorKind::Commit, AnchorKind::CheckoutState);
        assert_ne!(AnchorKind::CheckoutState, AnchorKind::None);
        assert_ne!(RepoIdKind::Explicit, RepoIdKind::LocalRoot);
        assert_ne!(Confidence::Medium, Confidence::Low);
    }

    // --- EX-2 / VT-2: canonical bytes (sorted keys, integer-only, float-reject). --

    #[test]
    fn canonical_primitives_encode_to_literals() {
        assert_eq!(canon(&json!(null)), "null");
        assert_eq!(canon(&json!(true)), "true");
        assert_eq!(canon(&json!(false)), "false");
        assert_eq!(canon(&json!(0)), "0");
        assert_eq!(canon(&json!(-1)), "-1");
        assert_eq!(canon(&json!(42)), "42");
    }

    #[test]
    fn canonical_sorts_object_keys_bytewise_and_keeps_array_order() {
        assert_eq!(canon(&json!({})), "{}");
        assert_eq!(canon(&json!([])), "[]");
        assert_eq!(canon(&json!({ "b": 1, "a": 2 })), "{\"a\":2,\"b\":1}");
        assert_eq!(canon(&json!([3, 1, 2])), "[3,1,2]");
        let nested = json!({ "z": [1, { "b": 2, "a": 3 }], "a": null });
        assert_eq!(canon(&nested), "{\"a\":null,\"z\":[1,{\"a\":3,\"b\":2}]}");
    }

    #[test]
    fn canonical_escapes_only_the_minimal_set() {
        assert_eq!(canon(&json!("a\"b\\c")), "\"a\\\"b\\\\c\"");
        assert_eq!(canon(&json!("\n\t")), "\"\\n\\t\"");
        assert_eq!(canon(&Value::String("\u{01}".to_owned())), "\"\\u0001\"");
        // Non-ASCII emitted raw, never escaped.
        assert_eq!(canon(&json!("é→")), "\"é→\"");
    }

    #[test]
    fn canonical_rejects_floats_and_exponent_forms() {
        assert!(canonical_bytes(&json!(1.5)).is_err(), "fractional rejected");
        let exp: Value = serde_json::from_str("1e3").unwrap_or(Value::Null);
        assert!(canonical_bytes(&exp).is_err(), "exponent form rejected");
        let dot_zero: Value = serde_json::from_str("1.0").unwrap_or(Value::Null);
        assert!(
            canonical_bytes(&dot_zero).is_err(),
            "1.0 float-form rejected"
        );
    }

    #[test]
    fn sha256_is_lowercase_hex_of_known_vector() {
        // The empty-string sha256 — a fixed, well-known vector.
        assert_eq!(
            sha256(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    // --- EX-5 / VT-2: checkout_state_id determinism + field sensitivity. -----

    #[test]
    fn checkout_state_id_is_deterministic() {
        assert_eq!(
            checkout_state_id("tree", "wf", "uf"),
            checkout_state_id("tree", "wf", "uf")
        );
    }

    #[test]
    fn checkout_state_id_changes_with_each_input() {
        let base = checkout_state_id("tree", "wf", "uf");
        assert_ne!(base, checkout_state_id("tree2", "wf", "uf"));
        assert_ne!(base, checkout_state_id("tree", "wf2", "uf"));
        assert_ne!(base, checkout_state_id("tree", "wf", "uf2"));
    }

    #[test]
    fn checkout_state_id_binds_the_normalizer_tag() {
        // The id is the sha256 of canonical bytes carrying CHECKOUT_NORMALIZER —
        // reconstruct it independently to pin the composition (byte-identity).
        let value = json!({
            "normalizer": CHECKOUT_NORMALIZER,
            "index_tree": "t",
            "worktree_fingerprint": "w",
            "untracked_fingerprint": "u",
        });
        let expected = sha256(&canonical_bytes(&value).unwrap_or_default());
        assert_eq!(checkout_state_id("t", "w", "u"), expected);
    }

    // --- EX-4 / VT-1: the repo_id byte-identity oracle. Copied VERBATIM from
    // the external decision register's `normalize_remote_url_table` — this is the interop proof. ---

    #[test]
    fn normalize_remote_url_table() {
        let cases = [
            ("https://github.com/org/repo.git", "github.com/org/repo"),
            ("https://github.com/org/repo", "github.com/org/repo"),
            ("git@github.com:org/repo.git", "github.com/org/repo"),
            ("ssh://git@github.com/org/repo.git", "github.com/org/repo"),
            ("ssh://git@github.com:22/org/repo", "github.com/org/repo"),
            ("https://github.com:443/org/repo", "github.com/org/repo"),
            // Non-default ports preserved (B3).
            (
                "ssh://git@git.example.com:2222/org/repo",
                "git.example.com:2222/org/repo",
            ),
            (
                "https://git.example.com:8443/org/repo",
                "git.example.com:8443/org/repo",
            ),
            // git:// keeps its default port so it never coalesces with ssh/https (B3).
            (
                "git://git.example.com/org/repo",
                "git.example.com:9418/org/repo",
            ),
            // Host lowercased, path case preserved.
            ("https://GitHub.com/Org/Repo.git", "github.com/Org/Repo"),
            // userinfo dropped.
            (
                "https://user:token@github.com/org/repo",
                "github.com/org/repo",
            ),
            // trailing slash.
            ("https://github.com/org/repo/", "github.com/org/repo"),
        ];
        for (raw, expected) in cases {
            assert_eq!(
                normalize_remote_url(raw).map(|g| g.repo_id).as_deref(),
                Some(expected),
                "input: {raw}"
            );
        }
    }

    #[test]
    fn normalize_remote_url_rejects_garbage() {
        assert!(normalize_remote_url("not a url").is_none());
        assert!(normalize_remote_url("").is_none());
        assert!(normalize_remote_url("https://").is_none());
    }

    #[test]
    fn normalize_remote_url_exposes_components() {
        let n = normalize_remote_url("ssh://git@git.example.com:2222/Org/Repo.git");
        assert!(n.is_some(), "should normalize");
        if let Some(n) = n {
            assert_eq!(n.host, "git.example.com");
            assert_eq!(n.port, Some(2222));
            assert_eq!(n.path, "Org/Repo");
            assert_eq!(n.repo_id, "git.example.com:2222/Org/Repo");
        }
    }

    #[test]
    fn remote_normalizer_tag_is_frozen() {
        assert_eq!(REMOTE_NORMALIZER, "forget.remote.v1");
    }

    // -----------------------------------------------------------------------
    // Impure capture (PHASE-02) — scratch-repo fixtures (VT-1/2/3).
    //
    // A throwaway git repo with pinned identity + commit dates so commit/tree
    // SHAs are deterministic. Mirrors the external decision register's `tests/support` harness.
    // -----------------------------------------------------------------------

    /// Fixed commit identity/time so captured SHAs are deterministic.
    const FIXED_DATE: &str = "2026-01-01T00:00:00 +0000";

    /// A throwaway git repository under a `tempfile` temp dir.
    struct ScratchRepo {
        _dir: tempfile::TempDir,
        path: PathBuf,
    }

    impl ScratchRepo {
        /// Create an unborn repo with `main` as the initial branch + pinned identity.
        fn new() -> Self {
            let dir = tempfile::tempdir().expect("tempdir");
            let path = dir.path().to_path_buf();
            let repo = Self { _dir: dir, path };
            repo.git(&["init", "-b", "main"]);
            repo.git(&["config", "user.name", "Doctrine Test"]);
            repo.git(&["config", "user.email", "test@doctrine.invalid"]);
            repo
        }

        fn path(&self) -> &Path {
            &self.path
        }

        /// Run a git command with pinned author/committer dates; panics on failure.
        fn git(&self, args: &[&str]) -> String {
            let output = Command::new("git")
                .arg("-C")
                .arg(&self.path)
                .args(args)
                .env("GIT_AUTHOR_DATE", FIXED_DATE)
                .env("GIT_COMMITTER_DATE", FIXED_DATE)
                .output()
                .expect("spawn git");
            assert!(
                output.status.success(),
                "git {args:?} failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            );
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }

        /// Write a file relative to the repo root (creating parents).
        fn write(&self, rel: &str, contents: &str) {
            let full = self.path.join(rel);
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent).expect("create parent");
            }
            std::fs::write(&full, contents).expect("write file");
        }

        /// Write, stage, and commit `rel`; return the commit SHA.
        fn commit(&self, rel: &str, contents: &str, message: &str) -> String {
            self.write(rel, contents);
            self.git(&["add", rel]);
            self.git(&["commit", "-m", message]);
            self.git(&["rev-parse", "HEAD"])
        }
    }

    // --- VT-1: frame fields per repo state. --------------------------------

    #[test]
    fn clean_checkout_anchors_to_head_commit() {
        let repo = ScratchRepo::new();
        let head = repo.commit("a.txt", "hello", "init");
        let tree = repo.git(&["rev-parse", "HEAD^{tree}"]);

        let frame = capture(repo.path()).expect("capture clean");
        assert_eq!(frame.anchor_kind, AnchorKind::Commit);
        assert_eq!(frame.commit, head);
        assert_eq!(frame.base_commit, head);
        assert_eq!(frame.tree, tree);
        assert_eq!(frame.ref_name, "refs/heads/main");
        assert!(
            frame.checkout_state_id.is_empty(),
            "clean tree carries no checkout_state_id"
        );
    }

    #[test]
    fn dirty_tracked_change_anchors_to_checkout_state() {
        let repo = ScratchRepo::new();
        let head = repo.commit("a.txt", "hello", "init");
        repo.write("a.txt", "hello world"); // unstaged modification

        let frame = capture(repo.path()).expect("capture dirty");
        assert_eq!(frame.anchor_kind, AnchorKind::CheckoutState);
        assert!(frame.commit.is_empty(), "commit empty iff dirty");
        assert!(!frame.checkout_state_id.is_empty());
        assert_eq!(
            frame.base_commit, head,
            "base_commit carries HEAD when dirty"
        );
    }

    #[test]
    fn untracked_only_is_dirty() {
        let repo = ScratchRepo::new();
        repo.commit("a.txt", "hello", "init");
        repo.write("untracked.txt", "new");

        let frame = capture(repo.path()).expect("capture untracked");
        assert_eq!(frame.anchor_kind, AnchorKind::CheckoutState);
        assert!(frame.commit.is_empty());
        assert!(!frame.checkout_state_id.is_empty());
    }

    #[test]
    fn detached_head_is_anchored_with_empty_ref() {
        let repo = ScratchRepo::new();
        let first = repo.commit("a.txt", "1", "first");
        repo.commit("b.txt", "2", "second");
        repo.git(&["checkout", &first]);

        let frame = capture(repo.path()).expect("capture detached");
        assert_eq!(frame.anchor_kind, AnchorKind::Commit, "still anchored");
        assert_eq!(frame.commit, first);
        assert!(
            frame.ref_name.is_empty(),
            "detached HEAD has empty ref_name"
        );
    }

    #[test]
    fn unborn_repo_is_none_anchor() {
        let repo = ScratchRepo::new(); // init, no commit
        let frame = capture(repo.path()).expect("capture unborn");
        assert_eq!(frame.anchor_kind, AnchorKind::None);
        assert!(frame.commit.is_empty());
        assert!(frame.base_commit.is_empty());
    }

    #[test]
    fn non_repo_is_none_anchor_not_error() {
        let dir = tempfile::tempdir().expect("tempdir"); // bare dir, not a repo
        let frame = capture(dir.path()).expect("non-repo must not error");
        assert_eq!(frame.anchor_kind, AnchorKind::None);
        assert_eq!(frame.repo.repo_id, "");
    }

    #[test]
    fn recapture_of_unchanged_dirty_tree_is_stable() {
        let repo = ScratchRepo::new();
        repo.commit("a.txt", "hello", "init");
        repo.write("a.txt", "changed");

        let a = capture(repo.path()).expect("capture a");
        let b = capture(repo.path()).expect("capture b");
        assert_eq!(a.checkout_state_id, b.checkout_state_id);
    }

    #[test]
    fn editing_worktree_changes_checkout_state_id() {
        let repo = ScratchRepo::new();
        repo.commit("a.txt", "hello", "init");

        repo.write("a.txt", "first edit");
        let first = capture(repo.path()).expect("capture first");
        repo.write("a.txt", "second edit");
        let second = capture(repo.path()).expect("capture second");

        assert_ne!(first.checkout_state_id, second.checkout_state_id);
    }

    // --- VT-2: repo-identity precedence. -----------------------------------

    #[test]
    fn origin_remote_drives_high_confidence_repo_id() {
        let repo = ScratchRepo::new();
        repo.commit("a.txt", "hello", "init");
        repo.git(&["remote", "add", "origin", "https://github.com/org/repo.git"]);

        let frame = capture(repo.path()).expect("capture remote");
        assert_eq!(frame.repo.kind, RepoIdKind::Remote);
        assert_eq!(frame.repo.confidence, Confidence::High);
        assert_eq!(frame.repo.repo_id, "github.com/org/repo");
    }

    #[test]
    fn two_remotes_without_origin_are_ambiguous() {
        let repo = ScratchRepo::new();
        repo.commit("a.txt", "hello", "init");
        repo.git(&["remote", "add", "alpha", "https://github.com/org/alpha.git"]);
        repo.git(&["remote", "add", "beta", "https://github.com/org/beta.git"]);

        let result = capture(repo.path());
        assert!(
            matches!(result, Err(CaptureError::AmbiguousRemote(_))),
            "got {result:?}"
        );
    }

    #[test]
    fn no_remote_falls_back_to_local_root_medium() {
        let repo = ScratchRepo::new();
        repo.commit("a.txt", "hello", "init");
        let root = repo.git(&["rev-list", "--max-parents=0", "HEAD"]);

        let frame = capture(repo.path()).expect("capture local-root");
        assert_eq!(frame.repo.kind, RepoIdKind::LocalRoot);
        assert_eq!(frame.repo.confidence, Confidence::Medium);
        assert_eq!(frame.repo.repo_id, format!("repo:git-root:{root}"));
    }

    #[test]
    fn explicit_config_repo_id_wins_over_remote() {
        let repo = ScratchRepo::new();
        repo.commit("a.txt", "hello", "init");
        repo.git(&["remote", "add", "origin", "https://github.com/org/repo.git"]);
        repo.git(&["config", "doctrine.repo.id", "custom/identity"]);

        let frame = capture(repo.path()).expect("capture explicit");
        assert_eq!(frame.repo.kind, RepoIdKind::Explicit);
        assert_eq!(frame.repo.confidence, Confidence::High);
        assert_eq!(frame.repo.repo_id, "custom/identity");
    }

    #[test]
    fn preferred_remote_config_overrides_origin() {
        let repo = ScratchRepo::new();
        repo.commit("a.txt", "hello", "init");
        repo.git(&[
            "remote",
            "add",
            "origin",
            "https://github.com/org/origin.git",
        ]);
        repo.git(&["remote", "add", "fork", "https://github.com/me/fork.git"]);
        repo.git(&["config", "doctrine.repo.preferredremote", "fork"]);

        let frame = capture(repo.path()).expect("capture preferred");
        assert_eq!(frame.repo.repo_id, "github.com/me/fork");
    }

    // The `--repo` override path (record's flag, PHASE-04) at the function level:
    // routed through the canonicalizer, so credentials are stripped (VT-2, R4).
    #[test]
    fn explicit_identity_strips_userinfo_from_credentialed_repo() {
        let id = explicit_identity("https://user:token@github.com/org/repo.git");
        assert_eq!(id.kind, RepoIdKind::Explicit);
        assert_eq!(id.confidence, Confidence::High);
        assert_eq!(id.repo_id, "github.com/org/repo", "userinfo dropped");
    }

    #[test]
    fn explicit_identity_keeps_non_url_value_verbatim() {
        let id = explicit_identity("org/project");
        assert_eq!(id.repo_id, "org/project");
        assert_eq!(id.kind, RepoIdKind::Explicit);
    }

    // --- VT-3: unstable-frame guards. --------------------------------------

    #[test]
    fn submodule_gitlink_entry_is_rejected() {
        let repo = ScratchRepo::new();
        let head = repo.commit("a.txt", "hello", "init");
        // Stage a gitlink (mode 160000) directly, no real submodule needed.
        repo.git(&[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("160000,{head},sub"),
        ]);

        let result = capture(repo.path());
        assert!(
            matches!(result, Err(CaptureError::Submodule)),
            "got {result:?}"
        );
    }

    // FR-001 (SL-012, mirrors the external decision register DE-010) — a repo with a tracked symlink
    // captures a frame instead of being rejected. Clean tree → Commit anchor.
    // (Was: symlink_entry_is_rejected, which asserted CaptureError::Symlink.)
    #[cfg(unix)]
    #[test]
    fn symlink_repo_captures_clean() {
        let repo = ScratchRepo::new();
        repo.commit("a.txt", "hello", "init");
        std::os::unix::fs::symlink("a.txt", repo.path().join("link")).expect("symlink");
        repo.git(&["add", "link"]);
        repo.git(&["commit", "-m", "add symlink"]);

        let frame = capture(repo.path()).expect("capture symlink repo");
        assert_eq!(
            frame.anchor_kind,
            AnchorKind::Commit,
            "clean symlink tree anchors on its commit"
        );
        assert!(frame.checkout_state_id.is_empty());
    }

    // A-3 (SL-012 audit) — proves design §3/§5's load-bearing claim that a *changed*
    // tracked symlink rides `worktree_fingerprint` (git's `diff --binary` of the
    // 120000 blob), not the untracked path. Commit a symlink, repoint it in the
    // worktree → CheckoutState with a non-empty csid, deterministic across captures.
    // Passes immediately: it is a characterization test pinning git's diff-based
    // worktree_fingerprint codepath (the repoint never touches `untracked_fingerprint`).
    #[cfg(unix)]
    #[test]
    fn tracked_symlink_repoint_is_dirty() {
        let repo = ScratchRepo::new();
        repo.commit("a.txt", "hello", "init");
        std::os::unix::fs::symlink("a.txt", repo.path().join("link")).expect("symlink");
        repo.git(&["add", "link"]);
        repo.git(&["commit", "-m", "add symlink"]);

        // Repoint the tracked symlink in the worktree (rm + re-symlink elsewhere).
        let link = repo.path().join("link");
        std::fs::remove_file(&link).expect("rm link");
        std::os::unix::fs::symlink("a.txt.other", &link).expect("re-symlink");

        let a = capture(repo.path()).expect("capture repointed");
        let b = capture(repo.path()).expect("recapture");
        assert_eq!(
            a.anchor_kind,
            AnchorKind::CheckoutState,
            "a changed tracked symlink makes the tree dirty"
        );
        assert!(
            !a.checkout_state_id.is_empty(),
            "the dirty tracked symlink carries a checkout_state_id"
        );
        assert_eq!(a, b, "tracked-symlink-repoint capture is deterministic");
    }

    // NF-001 (SL-012, mirrors the external decision register DE-010, RISK-03) — an untracked symlink
    // is encoded by its link text, never followed: mutating the *pointee's content*
    // leaves the csid unchanged. The pointee lives outside the repo, so the only
    // way its content could move the csid is a dereference.
    #[cfg(unix)]
    #[test]
    fn untracked_symlink_ignores_pointee_content() {
        let repo = ScratchRepo::new();
        repo.commit("a.txt", "hello", "init");

        let ext = tempfile::tempdir().expect("ext tempdir");
        let pointee = ext.path().join("pointee");
        std::fs::write(&pointee, "original").expect("write pointee");
        std::os::unix::fs::symlink(&pointee, repo.path().join("link")).expect("symlink");

        let csid1 = capture(repo.path()).expect("capture 1").checkout_state_id;
        std::fs::write(&pointee, "mutated content, a different length entirely")
            .expect("rewrite pointee");
        let csid2 = capture(repo.path()).expect("capture 2").checkout_state_id;

        assert!(!csid1.is_empty(), "untracked symlink makes the tree dirty");
        assert_eq!(
            csid1, csid2,
            "csid must be invariant to symlink target *content* (no-follow)"
        );
    }

    // NF-001 — repointing an untracked symlink to a different target changes the
    // csid (the link text *is* captured, not ignored).
    #[cfg(unix)]
    #[test]
    fn untracked_symlink_tracks_target_path() {
        let repo = ScratchRepo::new();
        repo.commit("a.txt", "hello", "init");
        let link = repo.path().join("link");
        std::os::unix::fs::symlink("first", &link).expect("symlink first");
        let csid1 = capture(repo.path())
            .expect("capture first")
            .checkout_state_id;

        std::fs::remove_file(&link).expect("rm link");
        std::os::unix::fs::symlink("second", &link).expect("symlink second");
        let csid2 = capture(repo.path())
            .expect("capture second")
            .checkout_state_id;

        assert_ne!(
            csid1, csid2,
            "repointing the symlink must change the csid (link text captured)"
        );
    }

    // NF-001 — a dangling untracked symlink captures cleanly and deterministically
    // (readlink succeeds even though the target is missing; the old
    // `git hash-object` path errored).
    #[cfg(unix)]
    #[test]
    fn dangling_untracked_symlink_ok() {
        let repo = ScratchRepo::new();
        repo.commit("a.txt", "hello", "init");
        std::os::unix::fs::symlink("does/not/exist", repo.path().join("link")).expect("symlink");

        let a = capture(repo.path()).expect("capture dangling symlink");
        let b = capture(repo.path()).expect("recapture");
        assert_eq!(
            a.anchor_kind,
            AnchorKind::CheckoutState,
            "untracked symlink makes the tree dirty"
        );
        assert_eq!(a, b, "dangling-symlink capture is deterministic");
    }

    // NF-001 / §3.1 — a symlink whose target is non-UTF-8 bytes captures and hashes
    // the raw readlink bytes (no `str` round-trip). Unix-only.
    #[cfg(unix)]
    #[test]
    fn untracked_symlink_non_utf8_target_bytes() {
        use std::os::unix::ffi::OsStrExt;
        let repo = ScratchRepo::new();
        repo.commit("a.txt", "hello", "init");
        // 0xFF 0xFE is not valid UTF-8; a legal symlink target on Unix.
        let target = std::ffi::OsStr::from_bytes(&[0xFF, 0xFE]);
        std::os::unix::fs::symlink(target, repo.path().join("link")).expect("symlink");

        let a = capture(repo.path()).expect("capture non-utf8 symlink target");
        let b = capture(repo.path()).expect("recapture");
        assert_eq!(a.anchor_kind, AnchorKind::CheckoutState);
        assert_eq!(a, b, "non-utf8 symlink target capture is deterministic");
    }

    // A-2 (SL-012 audit) — an untracked *regular* file whose name contains a `\n`
    // hashes correctly and deterministically. doctrine forks `git hash-object -- path`
    // once per path, which is newline-safe; this guards against a future batch-port
    // (the external decision register IMPR-003 / `untracked_hashes`) silently reintroducing Finding A,
    // where LF-separated `--stdin-paths` cannot carry a newline-bearing path.
    // A newline in a filename needs raw bytes — go through OsStr/std::fs directly.
    #[cfg(unix)]
    #[test]
    fn untracked_newline_in_name_is_deterministic() {
        use std::os::unix::ffi::OsStrExt;
        let repo = ScratchRepo::new();
        repo.commit("a.txt", "hello", "init");
        // "wei\nrd.txt" — a legal Unix filename with an embedded newline.
        let name = std::ffi::OsStr::from_bytes(b"wei\nrd.txt");
        std::fs::write(repo.path().join(name), "contents").expect("write newline file");

        let a = capture(repo.path()).expect("capture newline-name file");
        let b = capture(repo.path()).expect("recapture");
        assert_eq!(
            a.anchor_kind,
            AnchorKind::CheckoutState,
            "an untracked newline-named file makes the tree dirty"
        );
        assert!(!a.checkout_state_id.is_empty());
        assert_eq!(a, b, "newline-in-name capture is deterministic");
    }

    #[test]
    fn multi_root_repository_is_rejected() {
        let repo = ScratchRepo::new();
        repo.commit("a.txt", "hello", "init");
        // Build a second, unrelated root and merge it -> two roots under HEAD.
        repo.git(&["checkout", "--orphan", "other"]);
        let _ = Command::new("git")
            .arg("-C")
            .arg(repo.path())
            .args(["rm", "-rf", "."])
            .output();
        repo.commit("b.txt", "world", "second root");
        repo.git(&["checkout", "main"]);
        repo.git(&[
            "merge",
            "other",
            "--allow-unrelated-histories",
            "-m",
            "merge roots",
        ]);

        let result = capture(repo.path());
        assert!(
            matches!(result, Err(CaptureError::MultiRoot(2))),
            "got {result:?}"
        );
    }

    // --- VT-3: the conformance golden-vector (byte-identity proof, D7/R3). --
    //
    // A fixed fixture pinned to literal `repo_id` + `checkout_state_id`. The
    // fixture is **untracked-only dirty**, so every input to the csid is one of
    // git's frozen object hashes — `index_tree` = the HEAD tree SHA,
    // `worktree_fingerprint` = sha256 of an empty `diff HEAD` (untracked files do
    // not appear in the diff), `untracked_fingerprint` = sha256 over the
    // untracked path + its git blob SHA. None depend on commit dates or git
    // version, so the literal is reproducible and — because doctrine's
    // `normalize_remote_url`/`checkout_state_id`/`canonical_bytes`/`sha256` are
    // byte-copied from the external decision register's frozen algorithm (VT-1 verbatim table) —
    // equals the external decision register's value for the same tree. Drift in either impl breaks
    // this test.
    #[test]
    fn conformance_golden_vector() {
        let repo = ScratchRepo::new();
        repo.commit("a.txt", "hello", "init");
        repo.git(&["remote", "add", "origin", "https://github.com/org/repo.git"]);
        repo.write("untracked.txt", "world");

        let frame = capture(repo.path()).expect("capture golden");

        // repo_id from the remote URL — string-only, rock-solid across git versions.
        assert_eq!(frame.repo.repo_id, "github.com/org/repo");
        assert_eq!(frame.repo.kind, RepoIdKind::Remote);

        // checkout_state_id from git's frozen object hashes (see header comment).
        assert_eq!(frame.anchor_kind, AnchorKind::CheckoutState);
        assert_eq!(
            frame.checkout_state_id,
            "88d9489028e302700c2e6430e6df1d06539dccfd283d2ed99995258482ccf86c",
            "conformance golden checkout_state_id"
        );
    }

    // -----------------------------------------------------------------------
    // commits_touching (PHASE-03) — the per-candidate staleness count.
    // VT-1 counting · VT-2 guards/non-ancestor · VT-3 detached anchor survives.
    // -----------------------------------------------------------------------

    fn p(s: &str) -> Vec<String> {
        vec![s.to_string()]
    }

    // --- VT-2: cheap guards short-circuit before any subprocess. ------------

    #[test]
    fn empty_paths_returns_none_without_spawning() {
        // A bare (non-git) dir: a None here proves the guard fires before git.
        let dir = tempfile::tempdir().expect("tempdir");
        assert_eq!(
            commits_touching(dir.path(), &[], "deadbeef", "cafebabe"),
            None
        );
    }

    #[test]
    fn empty_endpoints_return_none() {
        let repo = ScratchRepo::new();
        let head = repo.commit("a.txt", "x", "init");
        assert_eq!(commits_touching(repo.path(), &p("a.txt"), "", &head), None);
        assert_eq!(commits_touching(repo.path(), &p("a.txt"), &head, ""), None);
    }

    // --- VT-1: the count, with the pathspec narrowing. ---------------------

    #[test]
    fn no_commits_since_anchor_is_zero() {
        let repo = ScratchRepo::new();
        let head = repo.commit("a.txt", "x", "init");
        // since == target ⇒ empty range ⇒ Some(0), not None.
        assert_eq!(
            commits_touching(repo.path(), &p("a.txt"), &head, &head),
            Some(0)
        );
    }

    #[test]
    fn counts_commits_touching_scoped_path() {
        let repo = ScratchRepo::new();
        let base = repo.commit("a.txt", "1", "init");
        repo.commit("a.txt", "2", "edit");
        let tip = repo.commit("a.txt", "3", "edit again");
        assert_eq!(
            commits_touching(repo.path(), &p("a.txt"), &base, &tip),
            Some(2)
        );
    }

    #[test]
    fn pathspec_narrows_out_other_paths() {
        let repo = ScratchRepo::new();
        let base = repo.commit("a.txt", "1", "init");
        let tip = repo.commit("b.txt", "1", "unrelated");
        // The commit since `base` touches only b.txt ⇒ a.txt scope sees Some(0).
        assert_eq!(
            commits_touching(repo.path(), &p("a.txt"), &base, &tip),
            Some(0)
        );
    }

    // --- VT-2: non-ancestor / missing object ⇒ None (no over-count). -------

    #[test]
    fn non_ancestor_since_returns_none_not_overcount() {
        let repo = ScratchRepo::new();
        let older = repo.commit("a.txt", "1", "init");
        let newer = repo.commit("a.txt", "2", "edit");
        // since=newer, target=older: newer is NOT an ancestor of older ⇒ None
        // (a bare `newer..older` would over-count via set difference).
        assert_eq!(
            commits_touching(repo.path(), &p("a.txt"), &newer, &older),
            None
        );
    }

    #[test]
    fn missing_object_returns_none() {
        let repo = ScratchRepo::new();
        let head = repo.commit("a.txt", "1", "init");
        let bogus = "0000000000000000000000000000000000000000";
        assert_eq!(
            commits_touching(repo.path(), &p("a.txt"), bogus, &head),
            None
        );
        assert_eq!(
            commits_touching(repo.path(), &p("a.txt"), &head, bogus),
            None
        );
    }

    // --- VT-3: anchoring survives a detached HEAD (frozen target SHA). ------

    #[test]
    fn detached_head_with_frozen_target_still_counts() {
        let repo = ScratchRepo::new();
        let base = repo.commit("a.txt", "1", "init");
        let tip = repo.commit("a.txt", "2", "edit");
        repo.git(&["checkout", &base]); // detach HEAD at base
        // Count is anchored on the passed SHAs, not HEAD ⇒ still Some(1).
        assert_eq!(
            commits_touching(repo.path(), &p("a.txt"), &base, &tip),
            Some(1)
        );
    }

    // --- SL-032 PHASE-02: trunk-ref id allocation (trunk read seam) --------
    //
    // The explicit-override ladder is exercised via `trunk_ladder` with the ref
    // injected — `set_var` is forbidden crate-wide, and the test process carries
    // no ambient `DOCTRINE_TRUNK_REF`, so the no-override path (`trunk_entity_ids`
    // → `trunk_tree_ish`) reads `None` naturally.

    use std::ffi::OsStr;

    /// Seed `.doctrine/slice/<NNN>/slice.toml` for each id and commit on `main`
    /// (git tracks a dir only via a contained file). A non-numeric sibling dir
    /// is committed too — it must be ignored by the numeric basename parse.
    fn commit_slice_dirs(repo: &ScratchRepo, ids: &[u32]) {
        for id in ids {
            repo.write(&format!(".doctrine/slice/{id:03}/slice.toml"), "x = 1\n");
        }
        repo.write(".doctrine/slice/scratch-notes/n.md", "ignore me\n");
        repo.git(&["add", "-A"]);
        repo.git(&["commit", "-m", "seed slices"]);
    }

    #[test]
    fn trunk_entity_ids_reads_committed_numeric_dirs() {
        // VT-2: trunk's tree carries slice dirs → their ids surface; the
        // non-numeric sibling dir is dropped.
        let repo = ScratchRepo::new();
        commit_slice_dirs(&repo, &[1, 2, 4]);
        let mut ids = super::trunk_entity_ids(repo.path(), ".doctrine/slice").unwrap();
        ids.sort_unstable();
        assert_eq!(ids, vec![1, 2, 4]);
    }

    #[test]
    fn trunk_entity_ids_does_not_reprepend_doctrine() {
        // VT-3 / X1: `kind_dir` is ALREADY repo-relative incl. `.doctrine/`. A
        // buggy re-prepend would query `.doctrine/.doctrine/slice/` → nothing.
        let repo = ScratchRepo::new();
        commit_slice_dirs(&repo, &[7]);
        let ids = super::trunk_entity_ids(repo.path(), ".doctrine/slice").unwrap();
        assert_eq!(ids, vec![7], "prefixed kind_dir must not be re-prepended");
    }

    #[test]
    fn trunk_entity_ids_empty_without_trunk() {
        // VT-4: an unborn repo (no commit ⇒ no main/master/origin peels) is a
        // defined terminus → None tree-ish → empty id set, not an error.
        let repo = ScratchRepo::new(); // init -b main, no commit
        assert_eq!(super::trunk_tree_ish(repo.path()).unwrap(), None);
        assert_eq!(
            super::trunk_entity_ids(repo.path(), ".doctrine/slice").unwrap(),
            Vec::<u32>::new()
        );
    }

    #[test]
    fn trunk_ladder_explicit_unpeelable_ref_is_hard_error() {
        // VT-5 / F4: an explicitly pinned ref that fails to peel must NOT fall
        // through to `main` — the user asked for a specific trunk.
        let repo = ScratchRepo::new();
        repo.commit("a.txt", "hello", "init"); // main DOES resolve…
        let bad = OsStr::new("refs/heads/does-not-exist");
        let err = super::trunk_ladder(repo.path(), Some(bad)).unwrap_err();
        assert!(
            err.to_string().contains("DOCTRINE_TRUNK_REF"),
            "error names the offending override: {err}"
        );
    }

    #[test]
    fn trunk_ladder_explicit_valid_ref_wins() {
        // The override resolves → it is used (peeled to a commit sha).
        let repo = ScratchRepo::new();
        let head = repo.commit("a.txt", "hello", "init");
        let sha = super::trunk_ladder(repo.path(), Some(OsStr::new("main"))).unwrap();
        assert_eq!(sha, Some(head));
    }

    // --- SL-127 PHASE-01: freshest-descendant trunk ladder (VT-1/2/3) -------
    //
    // The implicit ladder no longer takes the first ref that resolves: it folds
    // the resolved candidates (preference order) toward the freshest reachable
    // base — a stale `origin/HEAD` that is an *ancestor* of local `main` is
    // overtaken by `main`, but a diverged `origin/HEAD` is kept (no regression
    // below the most-preferred resolvable ref). Lagging/diverged `origin/HEAD`
    // is simulated by writing `refs/remotes/origin/HEAD` directly (the ladder
    // peels it as `origin/HEAD`) — no real remote, no `set_var`.

    /// Point `refs/remotes/origin/HEAD` at `sha` so the ladder peels it as the
    /// `origin/HEAD` candidate.
    fn set_origin_head(repo: &ScratchRepo, sha: &str) {
        repo.git(&["update-ref", "refs/remotes/origin/HEAD", sha]);
    }

    // VT-1: freshest_descendant exercised directly with explicit sha slices.

    #[test]
    fn freshest_descendant_advances_to_descendant() {
        // origin/HEAD < main: c1 is an ancestor of c2 ⇒ the later (fresher) c2 wins.
        let repo = ScratchRepo::new();
        let c1 = repo.commit("a.txt", "1", "c1");
        let c2 = repo.commit("a.txt", "2", "c2");
        let pick = super::freshest_descendant(repo.path(), &[c1, c2.clone()]).unwrap();
        assert_eq!(pick, Some(c2));
    }

    #[test]
    fn freshest_descendant_folds_full_chain() {
        // Chain c1 < c2 < c3 (origin/HEAD < main < master) ⇒ the tip c3 wins.
        let repo = ScratchRepo::new();
        let c1 = repo.commit("a.txt", "1", "c1");
        let c2 = repo.commit("a.txt", "2", "c2");
        let c3 = repo.commit("a.txt", "3", "c3");
        let pick = super::freshest_descendant(repo.path(), &[c1, c2, c3.clone()]).unwrap();
        assert_eq!(pick, Some(c3));
    }

    #[test]
    fn freshest_descendant_keeps_preferred_when_later_diverged() {
        // Most-preferred candidate diverges from the next ⇒ the preferred one is
        // kept (the later candidate is not a descendant — never regress to it).
        let repo = ScratchRepo::new();
        let base = repo.commit("a.txt", "base", "base");
        // Branch A (preferred) — the first candidate.
        repo.git(&["checkout", "-b", "branch-a"]);
        let a = repo.commit("a.txt", "branch-a", "a");
        // Branch B diverges from `base` (sibling of A, not a descendant of A).
        repo.git(&["checkout", "-b", "branch-b", &base]);
        let b = repo.commit("b.txt", "branch-b", "b");
        let pick = super::freshest_descendant(repo.path(), &[a.clone(), b]).unwrap();
        assert_eq!(
            pick,
            Some(a),
            "preferred-but-older kept; diverged sibling skipped"
        );
    }

    #[test]
    fn freshest_descendant_single_and_empty() {
        let repo = ScratchRepo::new();
        let c1 = repo.commit("a.txt", "1", "c1");
        assert_eq!(
            super::freshest_descendant(repo.path(), &[c1.clone()]).unwrap(),
            Some(c1)
        );
        assert_eq!(super::freshest_descendant(repo.path(), &[]).unwrap(), None);
    }

    // VT-1 (integration): the rewired implicit ladder arm via real refs.

    #[test]
    fn trunk_ladder_stale_origin_head_overtaken_by_main() {
        // origin/HEAD lags behind main (ancestor) ⇒ ladder picks the ahead `main`,
        // not the first-resolving `origin/HEAD`.
        let repo = ScratchRepo::new();
        let lag = repo.commit("a.txt", "1", "lag");
        let ahead = repo.commit("a.txt", "2", "ahead"); // main now at `ahead`
        set_origin_head(&repo, &lag);
        let pick = super::trunk_ladder(repo.path(), None).unwrap();
        assert_eq!(pick, Some(ahead), "stale origin/HEAD overtaken by main");
    }

    #[test]
    fn trunk_ladder_diverged_origin_head_kept_over_main() {
        // origin/HEAD diverges from main (most-preferred, not an ancestor of main)
        // ⇒ preference wins; main does NOT regress the pick below origin/HEAD.
        let repo = ScratchRepo::new();
        let base = repo.commit("a.txt", "base", "base");
        // main advances to its own tip.
        let _main_tip = repo.commit("a.txt", "main", "main-advance");
        // origin/HEAD points at a sibling commit off `base` (diverged from main).
        repo.git(&["checkout", "-b", "remote-sim", &base]);
        let origin = repo.commit("o.txt", "origin", "origin-advance");
        repo.git(&["checkout", "main"]);
        set_origin_head(&repo, &origin);
        let pick = super::trunk_ladder(repo.path(), None).unwrap();
        assert_eq!(
            pick,
            Some(origin),
            "diverged origin/HEAD kept (preference order)"
        );
    }

    // VT-2: explicit override still wins over a fresher implicit candidate.

    #[test]
    fn trunk_ladder_explicit_wins_over_fresher_implicit() {
        // main is ahead of origin/HEAD, but an explicit DOCTRINE_TRUNK_REF pinning
        // the lagging ref still short-circuits and wins (override unchanged).
        let repo = ScratchRepo::new();
        let lag = repo.commit("a.txt", "1", "lag");
        let _ahead = repo.commit("a.txt", "2", "ahead");
        set_origin_head(&repo, &lag);
        let pick = super::trunk_ladder(repo.path(), Some(OsStr::new("origin/HEAD"))).unwrap();
        assert_eq!(
            pick,
            Some(lag),
            "explicit override beats fresher implicit main"
        );
    }

    // VT-3: minting fallout — trunk_entity_ids reads off the ahead (`main`) tree.

    #[test]
    fn trunk_entity_ids_read_off_ahead_main_when_origin_head_behind() {
        // origin/HEAD behind, main ahead with an extra slice dir ⇒ the id-read
        // rides the ahead ladder pick and sees the newer id.
        let repo = ScratchRepo::new();
        repo.write(".doctrine/slice/001/slice.toml", "x = 1\n");
        repo.git(&["add", "-A"]);
        repo.git(&["commit", "-m", "seed 001"]);
        let behind = repo.git(&["rev-parse", "HEAD"]);
        // main advances, adding slice 002.
        repo.write(".doctrine/slice/002/slice.toml", "x = 1\n");
        repo.git(&["add", "-A"]);
        repo.git(&["commit", "-m", "seed 002"]);
        set_origin_head(&repo, &behind);
        let mut ids = super::trunk_entity_ids(repo.path(), ".doctrine/slice").unwrap();
        ids.sort_unstable();
        assert_eq!(
            ids,
            vec![1, 2],
            "ids read off the ahead main tree, not stale origin/HEAD"
        );
    }

    // --- SL-064 PHASE-03: projection plumbing (VT-1/VT-2/VT-3) --------------

    /// VT-1: filter-tree drops the excluded pathspecs and leaves the live index
    /// byte-for-byte unchanged (the throwaway-`GIT_INDEX_FILE` guarantee).
    #[test]
    fn filter_tree_excludes_paths_and_leaves_live_index_untouched() {
        let repo = ScratchRepo::new();
        repo.commit("keep.txt", "k", "init");
        repo.write(".doctrine/dispatch/64/journal.toml", "rows");
        repo.git(&["add", "."]);
        repo.git(&["commit", "-m", "add ledger"]);
        let tree = repo.git(&["rev-parse", "HEAD^{tree}"]);

        let index_before = std::fs::read(repo.path().join(".git/index")).expect("read index");

        let filtered =
            super::filter_tree(repo.path(), &tree, &[".doctrine/dispatch/64"]).expect("filter");

        let listing = repo.git(&["ls-tree", "-r", "--name-only", &filtered]);
        assert!(
            listing.contains("keep.txt"),
            "kept path survives: {listing}"
        );
        assert!(
            !listing.contains("journal.toml"),
            "excluded path dropped: {listing}"
        );

        let index_after = std::fs::read(repo.path().join(".git/index")).expect("read index");
        assert_eq!(
            index_before, index_after,
            "live index byte-for-byte unchanged"
        );
    }

    /// VT-1 (degenerate): an empty exclude set is the identity filter — the tree
    /// is re-emitted unchanged, still without touching the live index.
    #[test]
    fn filter_tree_empty_exclude_is_identity() {
        let repo = ScratchRepo::new();
        repo.commit("a.txt", "1", "init");
        let tree = repo.git(&["rev-parse", "HEAD^{tree}"]);
        let filtered = super::filter_tree(repo.path(), &tree, &[]).expect("filter");
        assert_eq!(filtered, tree, "identity filter re-emits the same tree");
    }

    /// VT-2: a filtered tree commits against a supplied parent with no checkout —
    /// HEAD and the working tree are untouched.
    #[test]
    fn commit_tree_against_parent_without_checkout() {
        let repo = ScratchRepo::new();
        let parent = repo.commit("a.txt", "1", "first");
        repo.commit("b.txt", "2", "second");
        let tip_tree = repo.git(&["rev-parse", "HEAD^{tree}"]);
        let head_before = repo.git(&["rev-parse", "HEAD"]);
        let a_before = std::fs::read_to_string(repo.path().join("a.txt")).expect("read a");

        let commit = super::commit_tree(repo.path(), &tip_tree, &parent, "synth").expect("commit");

        assert_eq!(
            repo.git(&["rev-parse", &format!("{commit}^")]),
            parent,
            "parent is as supplied"
        );
        assert_eq!(
            repo.git(&["diff", "--name-only", &parent, &commit]),
            "b.txt",
            "diff parent..commit is exactly the second-commit delta"
        );
        assert_eq!(
            repo.git(&["rev-parse", "HEAD"]),
            head_before,
            "HEAD unmoved"
        );
        assert_eq!(
            std::fs::read_to_string(repo.path().join("a.txt")).expect("read a"),
            a_before,
            "working tree untouched"
        );
    }

    /// VT-3: CAS succeeds only at the expected old oid; a mismatch reports the
    /// moved target's actual value and never clobbers the ref.
    #[test]
    fn update_ref_cas_succeeds_only_at_expected_old() {
        let repo = ScratchRepo::new();
        let c1 = repo.commit("a.txt", "1", "first");
        let c2 = repo.commit("b.txt", "2", "second");
        let zero = "0".repeat(40);
        let refname = "refs/review/x";

        // Create via zero-oid CAS (must-not-exist).
        assert!(matches!(
            super::update_ref_cas(repo.path(), refname, &c1, &zero).expect("create"),
            super::RefCas::Updated
        ));
        assert_eq!(repo.git(&["rev-parse", refname]), c1);

        // Wrong expected-old → Moved{actual: c1}, ref unchanged.
        match super::update_ref_cas(repo.path(), refname, &c2, &zero).expect("cas") {
            super::RefCas::Moved { actual } => assert_eq!(actual.as_deref(), Some(c1.as_str())),
            super::RefCas::Updated => panic!("expected Moved at wrong expected-old"),
        }
        assert_eq!(repo.git(&["rev-parse", refname]), c1, "ref not clobbered");

        // Correct expected-old → Updated to c2.
        assert!(matches!(
            super::update_ref_cas(repo.path(), refname, &c2, &c1).expect("cas2"),
            super::RefCas::Updated
        ));
        assert_eq!(repo.git(&["rev-parse", refname]), c2);
    }

    /// PHASE-04 journal-commit primitive: splice a file into a base tree without
    /// touching the live index; the new tree carries the content at the path and
    /// retains the base's other paths.
    #[test]
    fn tree_with_file_splices_blob_without_touching_index() {
        let repo = ScratchRepo::new();
        repo.commit("keep.txt", "k", "init");
        let base = repo.git(&["rev-parse", "HEAD^{tree}"]);
        let index_before = std::fs::read(repo.path().join(".git/index")).expect("read index");

        let tree = super::tree_with_file(
            repo.path(),
            &base,
            ".doctrine/dispatch/064/journal.toml",
            "rows = []\n",
        )
        .expect("splice");

        let listing = repo.git(&["ls-tree", "-r", "--name-only", &tree]);
        assert!(
            listing.contains("keep.txt"),
            "base path retained: {listing}"
        );
        assert!(
            listing.contains(".doctrine/dispatch/064/journal.toml"),
            "spliced path present: {listing}"
        );
        let blob = repo.git(&[
            "cat-file",
            "-p",
            &format!("{tree}:.doctrine/dispatch/064/journal.toml"),
        ]);
        assert_eq!(blob, "rows = []", "spliced content readable at path");

        let index_after = std::fs::read(repo.path().join(".git/index")).expect("read index");
        assert_eq!(index_before, index_after, "live index untouched");
    }

    /// PHASE-05 T2: the 3-way replay CAS (design §4.1, EX-2). `current==planned`
    /// is an idempotent no-op; `current==expected_old` (incl. absent↔zero for a
    /// creation) applies; divergence from BOTH refuses and reports the actual,
    /// never clobbering.
    #[test]
    fn replay_ref_no_op_apply_refuse() {
        let repo = ScratchRepo::new();
        let c1 = repo.commit("a.txt", "1", "first");
        let c2 = repo.commit("b.txt", "2", "second");
        let c3 = repo.commit("c.txt", "3", "third");
        let zero = "0".repeat(40);
        let refname = "refs/heads/trunk-x";

        // Creation: ref absent ↔ expected zero, planned c1 → Applied.
        assert!(matches!(
            super::replay_ref(repo.path(), refname, &zero, &c1).expect("create"),
            super::ReplayOutcome::Applied
        ));
        assert_eq!(repo.git(&["rev-parse", refname]), c1);

        // Idempotent: current==planned → NoOp, ref untouched (crash-after-apply).
        assert!(matches!(
            super::replay_ref(repo.path(), refname, &zero, &c1).expect("replay"),
            super::ReplayOutcome::NoOp
        ));
        assert_eq!(repo.git(&["rev-parse", refname]), c1);

        // Diverged: current c1 ∉ {expected c2, planned c3} → Moved, not clobbered.
        match super::replay_ref(repo.path(), refname, &c2, &c3).expect("diverge") {
            super::ReplayOutcome::Moved { actual } => {
                assert_eq!(actual.as_deref(), Some(c1.as_str()));
            }
            other => panic!("expected Moved, got {other:?}"),
        }
        assert_eq!(repo.git(&["rev-parse", refname]), c1, "not clobbered");

        // Apply at the correct expected: c1 → c2.
        assert!(matches!(
            super::replay_ref(repo.path(), refname, &c1, &c2).expect("apply"),
            super::ReplayOutcome::Applied
        ));
        assert_eq!(repo.git(&["rev-parse", refname]), c2);
    }

    /// PHASE-05 T3: ff-only ancestry reads the `merge-base --is-ancestor` exit code
    /// (exit 1 is a clean `false`, not an error). Reflexive on equality.
    #[test]
    fn is_ancestor_reads_exit_code() {
        let repo = ScratchRepo::new();
        let c1 = repo.commit("a.txt", "1", "first");
        let c2 = repo.commit("b.txt", "2", "second");

        assert!(super::is_ancestor(repo.path(), &c1, &c2).expect("c1<c2"));
        assert!(!super::is_ancestor(repo.path(), &c2, &c1).expect("c2!<c1"));
        assert!(super::is_ancestor(repo.path(), &c1, &c1).expect("reflexive"));
    }

    /// RV-030 F-1: the pinned fork-point. `merge_base` returns the common ancestor
    /// of two divergent branches; `Ok(None)` for unrelated histories (exit 1).
    #[test]
    fn merge_base_returns_fork_point_or_none() {
        let repo = ScratchRepo::new();
        let base = repo.commit("a.txt", "1", "base");
        // Diverge: a feature branch off `base`, then advance `main` past it.
        repo.git(&["branch", "feature"]);
        repo.commit("main.txt", "m", "main advances"); // main moves on
        repo.git(&["checkout", "feature"]);
        let feat = repo.commit("feat.txt", "f", "feature commit");

        assert_eq!(
            super::merge_base(repo.path(), &feat, "main").expect("merge-base"),
            Some(base.clone()),
            "fork-point is the shared base, not either tip"
        );

        // An orphan branch shares no history with `base` → no common ancestor.
        repo.git(&["checkout", "--orphan", "island"]);
        let island = repo.commit("island.txt", "i", "unrelated root");
        assert_eq!(
            super::merge_base(repo.path(), &island, &base).expect("merge-base unrelated"),
            None,
            "unrelated histories share no merge-base"
        );
    }

    /// RV-030 F-9: `read_path_at` reads a blob from a refish's committed tree
    /// (object db, no working tree) — `Some` when the path is present, `None` when
    /// absent — matching the discipline of every other new projection primitive.
    #[test]
    fn read_path_at_present_some_absent_none() {
        let repo = ScratchRepo::new();
        let head = repo.commit(
            ".doctrine/dispatch/064/journal.toml",
            "rows = []\n",
            "ledger",
        );

        assert_eq!(
            super::read_path_at(repo.path(), &head, ".doctrine/dispatch/064/journal.toml")
                .expect("present"),
            Some("rows = []".to_owned()),
            "present path yields its blob content"
        );
        assert_eq!(
            super::read_path_at(repo.path(), &head, ".doctrine/dispatch/064/absent.toml")
                .expect("absent"),
            None,
            "absent path yields None, not an error"
        );
    }

    // --- SL-121 PHASE-01: branch→worktree-path probe (VT-1) ----------------

    #[test]
    fn parse_worktree_for_ref_returns_path_of_matching_branch() {
        let listing = "\
worktree /repos/main
HEAD aaaa
branch refs/heads/main

worktree /repos/feature
HEAD bbbb
branch refs/heads/dispatch/121
";
        assert_eq!(
            parse_worktree_for_ref(listing, "refs/heads/dispatch/121"),
            Some(PathBuf::from("/repos/feature")),
        );
        assert_eq!(
            parse_worktree_for_ref(listing, "refs/heads/main"),
            Some(PathBuf::from("/repos/main")),
        );
    }

    #[test]
    fn parse_worktree_for_ref_absent_ref_is_none() {
        let listing = "\
worktree /repos/main
HEAD aaaa
branch refs/heads/main
";
        assert_eq!(
            parse_worktree_for_ref(listing, "refs/heads/nope"),
            None,
            "a ref no live worktree checks out yields None",
        );
    }

    #[test]
    fn parse_worktree_for_ref_skips_detached_block() {
        // A detached block (bare/no `branch` line) must not lend its `worktree`
        // path to a later block's branch match.
        let listing = "\
worktree /repos/detached
HEAD cccc
detached

worktree /repos/live
HEAD dddd
branch refs/heads/target
";
        assert_eq!(
            parse_worktree_for_ref(listing, "refs/heads/target"),
            Some(PathBuf::from("/repos/live")),
        );
        // And a refname that only the detached block could (wrongly) satisfy stays None.
        assert_eq!(parse_worktree_for_ref(listing, "refs/heads/detached"), None);
    }

    #[test]
    fn parse_worktree_for_ref_blank_line_resets_state() {
        // The block-reset rule (M9): the blank line clears the pending path, so a
        // stray `branch` line not preceded (in its block) by a `worktree` line
        // binds to nothing — proving the reset, not the carry-over.
        let listing = "\
worktree /repos/first
HEAD eeee

branch refs/heads/orphan
";
        assert_eq!(
            parse_worktree_for_ref(listing, "refs/heads/orphan"),
            None,
            "after a blank line the path is reset, so the orphan branch binds to no path",
        );
    }

    #[test]
    fn worktree_for_ref_finds_linked_worktree() {
        let repo = ScratchRepo::new();
        repo.commit("a.txt", "hello", "init");
        let linked = repo._dir.path().join("linked");
        repo.git(&[
            "worktree",
            "add",
            "-b",
            "feature",
            &linked.to_string_lossy(),
        ]);

        let found = worktree_for_ref(repo.path(), "refs/heads/feature")
            .expect("worktree list must succeed");
        // git may canonicalize the path (e.g. /private on macOS); compare by suffix.
        assert!(
            found.as_ref().is_some_and(|p| p.ends_with("linked")),
            "expected the linked worktree path, got {found:?}",
        );
        assert_eq!(
            worktree_for_ref(repo.path(), "refs/heads/absent").expect("list ok"),
            None,
            "a branch with no live worktree yields Ok(None)",
        );
    }

    #[test]
    fn worktree_for_ref_errors_when_not_a_repo() {
        let dir = tempfile::tempdir().expect("tempdir"); // bare dir, not a repo
        assert!(
            matches!(
                worktree_for_ref(dir.path(), "refs/heads/main"),
                Err(CaptureError::Git(_))
            ),
            "a git failure surfaces as Err, distinct from Ok(None)",
        );
    }

    // --- SL-121 PHASE-02: worktree-aware advance shells -----------------------

    /// Build `main @ c1` (checked out) with a descendant `c2` parked on `side`,
    /// leaving the working tree on `main` at `c1`. The shared fixture for the
    /// ff-advance / reset-keep shell tests.
    fn main_at_c1_with_descendant_c2(repo: &ScratchRepo) -> (String, String) {
        let c1 = repo.commit("a.txt", "1", "first");
        repo.git(&["branch", "side"]);
        repo.git(&["checkout", "side"]);
        let c2 = repo.commit("b.txt", "2", "second");
        repo.git(&["checkout", "main"]);
        assert_eq!(repo.git(&["rev-parse", "main"]), c1);
        (c1, c2)
    }

    /// `tree_clean` reports tracked dirt and ignores untracked scratch — the
    /// `--untracked-files=no` predicate shared by the dirty pre-gate and the §2.5
    /// race re-check.
    #[test]
    fn tree_clean_reports_tracked_dirt_ignores_untracked() {
        let repo = ScratchRepo::new();
        repo.commit("a.txt", "1", "first");
        assert!(super::tree_clean(repo.path()).expect("clean"));
        repo.write("untracked.txt", "x");
        assert!(
            super::tree_clean(repo.path()).expect("clean w/ untracked"),
            "untracked files are ignored (--untracked-files=no)",
        );
        repo.write("a.txt", "changed");
        assert!(
            !super::tree_clean(repo.path()).expect("dirty"),
            "tracked modification is dirt",
        );
    }

    /// Clean fast-forward of a checked-out ref: ref + index + worktree all land on
    /// `planned`, `git status` empty (the ISS-022/030 desync this shell kills).
    #[test]
    fn ff_advance_in_worktree_advances_checked_out_ref() {
        let repo = ScratchRepo::new();
        let (_c1, c2) = main_at_c1_with_descendant_c2(&repo);

        let out =
            super::ff_advance_in_worktree(repo.path(), "refs/heads/main", &c2).expect("probe ok");
        assert_eq!(out, super::FfAdvance::Advanced);
        assert_eq!(repo.git(&["rev-parse", "main"]), c2, "ref advanced");
        assert_eq!(
            repo.git(&["rev-parse", "HEAD"]),
            c2,
            "HEAD advanced with it"
        );
        assert!(
            repo.git(&["status", "--porcelain"]).is_empty(),
            "index + worktree at planned — no phantom reverse-diff",
        );
        assert_eq!(
            std::fs::read_to_string(repo.path().join("b.txt")).expect("read b"),
            "2",
            "worktree carries the advanced content",
        );
    }

    /// §2.5 guard: HEAD detached off `target_ref` between probe and merge → a
    /// captured `Raced`, never a wrong-ref advance.
    #[test]
    fn ff_advance_in_worktree_races_when_head_detached() {
        let repo = ScratchRepo::new();
        let (c1, c2) = main_at_c1_with_descendant_c2(&repo);
        repo.git(&["checkout", "--detach"]); // HEAD detached at c1

        match super::ff_advance_in_worktree(repo.path(), "refs/heads/main", &c2).expect("probe ok")
        {
            super::FfAdvance::Raced { token } => {
                assert!(token.contains("HEAD"), "token names the HEAD race: {token}");
            }
            super::FfAdvance::Advanced => panic!("must refuse: HEAD is not on target_ref"),
        }
        assert_eq!(
            repo.git(&["rev-parse", "main"]),
            c1,
            "main untouched under a raced HEAD",
        );
    }

    /// §2.5 guard: a dirty tracked tree in the probe→merge window → a captured
    /// `Raced` (merge --ff-only does NOT itself refuse arbitrary dirt, M5).
    #[test]
    fn ff_advance_in_worktree_races_when_tree_dirty() {
        let repo = ScratchRepo::new();
        let (c1, c2) = main_at_c1_with_descendant_c2(&repo);
        repo.write("a.txt", "locally modified"); // tracked dirt

        match super::ff_advance_in_worktree(repo.path(), "refs/heads/main", &c2).expect("probe ok")
        {
            super::FfAdvance::Raced { token } => {
                assert!(token.contains("dirty"), "token names the dirt: {token}");
            }
            super::FfAdvance::Advanced => panic!("must refuse: tree is dirty"),
        }
        assert_eq!(repo.git(&["rev-parse", "main"]), c1, "main untouched");
    }

    /// The None-leg clean resync: a pure `update-ref` under a live checkout
    /// desyncs the index/worktree; `resync_worktree_hard` restores them to the
    /// advanced ref with a clean tree. `reset --keep` cannot (HEAD already moved
    /// with the ref → no diff to apply); `reset --hard` is immune to that ordering.
    #[test]
    fn resync_worktree_hard_resyncs_stale_index_after_pure_ref_advance() {
        let repo = ScratchRepo::new();
        let (_c1, c2) = main_at_c1_with_descendant_c2(&repo);
        // Simulate the None-leg desync: advance the ref under the live checkout.
        repo.git(&["update-ref", "refs/heads/main", &c2]);
        assert!(
            !repo.git(&["status", "--porcelain"]).is_empty(),
            "pure update-ref desynced the live checkout",
        );

        super::resync_worktree_hard(repo.path(), &c2).expect("reset --hard");
        assert!(
            repo.git(&["status", "--porcelain"]).is_empty(),
            "resync restored index + worktree to the advanced ref",
        );
        assert_eq!(
            std::fs::read_to_string(repo.path().join("b.txt")).expect("read b"),
            "2",
        );
    }
}
