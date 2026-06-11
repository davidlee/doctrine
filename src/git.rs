// SPDX-License-Identifier: GPL-3.0-only
//! Git seam — the born-frame producer for memory anchoring (SL-007, design §5.2).
//!
//! doctrine reproduces `forgettable`'s frozen `GitContextFrameV1` algorithm
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
//! verbatim from forgettable and, in PHASE-02, a shared conformance golden-vector.
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

use std::path::Path;
use std::process::Command;

use serde_json::{Number, Value};
use sha2::{Digest, Sha256};

/// Remote-URL normalizer tag (`forget.remote.v1`) — versions the algorithm so a
/// future change is detectable in persisted frames. Parity with forgettable.
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
// Canonical JSON bytes (DEC-009) + sha256 — mirrored from forgettable's
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
/// Byte-identical to forgettable's `hash::sha256`.
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
// forgettable's `git_context::capture`, projected down to doctrine's flat `Frame`.
// ---------------------------------------------------------------------------

/// Normative git config flags applied to **every** invocation (EX-1) so local
/// config (autocrlf/eol/fileMode) cannot perturb captured trees/diffs/hashes —
/// required for byte-identity with forgettable.
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
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(NORMATIVE_FLAGS)
        .args(args)
        .output()
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
fn git_opt(root: &Path, args: &[&str]) -> Result<Option<String>, CaptureError> {
    let output = run_git(root, args)?;
    if !output.status.success() {
        return Ok(None);
    }
    let text = String::from_utf8(output.stdout)
        .map_err(|_ignored| CaptureError::Git(format!("non-utf8 output: {}", args.join(" "))))?;
    Ok(Some(text.trim().to_string()))
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

/// The peeled trunk ladder with the explicit override injected (`explicit` is
/// `DOCTRINE_TRUNK_REF` when set). See [`trunk_tree_ish`] for the contract; the
/// asymmetry (F4/X6) lives here: an explicit ref that fails to peel is a hard
/// error, ladder candidates that fail simply fall through.
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
    for candidate in ["origin/HEAD", "main", "master"] {
        if let Some(sha) = peel(candidate)? {
            return Ok(Some(sha));
        }
    }
    Ok(None)
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

/// Capture the born frame for the working tree at `repo_root` (design §5.2).
///
/// Three Ok states: clean → [`AnchorKind::Commit`] (`commit`/`tree`/`base_commit`/
/// `ref_name`); dirty → [`AnchorKind::CheckoutState`] (`checkout_state_id`/`base_commit`,
/// `commit` empty); unborn or non-repo → [`AnchorKind::None`]. Detached HEAD is
/// still anchored with an empty `ref_name`. Submodule/multi-root/ambiguous-remote
/// trees error rather than emit an unstable anchor (D8). Symlinks are supported
/// (SL-012, mirrors forgettable DE-010): tracked symlinks ride `index_tree`/
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
    let index_tree = git_text(repo_root, &["write-tree"])?;
    let diff_bytes = git_bytes(
        repo_root,
        &["diff", "HEAD", "--binary", "--no-textconv", "--no-ext-diff"],
    )?;
    let untracked_fp = untracked_fingerprint(repo_root)?;
    let dirty = index_tree != tree || !diff_bytes.is_empty() || untracked_fp.is_some();

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
/// Symlinks (120000) are supported (SL-012, mirrors forgettable DE-010):
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
/// mirrors forgettable DE-010 §3.1). Regular-entry encoding is byte-identical to
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
/// target bytes `)`, never following the link (SL-012, mirrors forgettable DE-010
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
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "SL-042 P3 reconcile-reader seam: head_sha feeds coverage_scan's \
                  staleness resolution; no bins/lib consumer until the CLI reader \
                  slice wires it"
    )
)]
pub(crate) fn head_sha(root: &Path) -> Option<String> {
    git_opt(root, &["rev-parse", "--verify", "HEAD^{commit}"])
        .ok()
        .flatten()
}

// ---------------------------------------------------------------------------
// Unit tests — pure logic only (no git, no disk). The byte-identity proof for
// the remote table is copied verbatim from forgettable's reference (VT-1).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use std::path::{Path, PathBuf};
    use std::process::Command;

    use super::{
        AnchorKind, CHECKOUT_NORMALIZER, CaptureError, Confidence, Frame, REMOTE_NORMALIZER,
        RepoIdKind, RepoIdentity, canonical_bytes, capture, checkout_state_id, commits_touching,
        explicit_identity, normalize_remote_url, sha256,
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
    // forgettable's `normalize_remote_url_table` — this is the interop proof. ---

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
    // SHAs are deterministic. Mirrors forgettable's `tests/support` harness.
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

    // FR-001 (SL-012, mirrors forgettable DE-010) — a repo with a tracked symlink
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

    // NF-001 (SL-012, mirrors forgettable DE-010, RISK-03) — an untracked symlink
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
    // (forgettable IMPR-003 / `untracked_hashes`) silently reintroducing Finding A,
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
    // byte-copied from forgettable's frozen algorithm (VT-1 verbatim table) —
    // equals forgettable's value for the same tree. Drift in either impl breaks
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
}
