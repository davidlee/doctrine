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
// Unit tests — pure logic only (no git, no disk). The byte-identity proof for
// the remote table is copied verbatim from forgettable's reference (VT-1).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{
        AnchorKind, CHECKOUT_NORMALIZER, Confidence, Frame, REMOTE_NORMALIZER, RepoIdKind,
        RepoIdentity, canonical_bytes, checkout_state_id, normalize_remote_url, sha256,
    };

    /// Render canonical bytes as a `String` for readable assertions (canonical
    /// output is always valid UTF-8).
    fn canon(v: &Value) -> String {
        let bytes = canonical_bytes(v).unwrap_or_default();
        String::from_utf8(bytes).unwrap_or_default()
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
}
