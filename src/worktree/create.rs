// The pure items here have NO non-test consumer until PHASE-02 wires the shell
// (`run_create_fork`); under the gated bins/lib build they are genuinely unused, so
// the repo's `unused`/`dead_code` denies would fire. The lid is `not(test)`-gated:
// in the test cfg the in-file `tests` module consumes every item, so a plain
// `#![expect(unused)]` would be UNFULFILLED there (workspace `warnings = "deny"`
// captures `unfulfilled_lint_expectations`). PHASE-02 (EX-7) reconciles the lid.
#![cfg_attr(not(test), expect(unused, reason = "PHASE-02 wires the shell"))]
// SPDX-License-Identifier: GPL-3.0-only
//! worktree create-fork — the PURE core of the `WorktreeCreate` hook verb.
//!
//! Mirror of [`super::subagent::classify_stamp`]: the pure core decides, the shell
//! (PHASE-02's `run_create_fork`) gathers the impure facts and acts. No git / disk /
//! env / clock here (ADR-001 leaf, CLAUDE.md pure/imperative split) — every fact the
//! decision needs is passed in as a flat value/bool. The realpath compare
//! (`cwd IS the arming dir`) and the `base` file read are resolved by the shell and
//! folded into the inputs below, exactly as `run_stamp_subagent` resolves facts for
//! `classify_stamp`.

/// Verdict of the PURE create classifier: positional arming says fork-or-passthrough.
/// `Fork` when the payload cwd IS the arming dir and `base` parses to a plausible sha
/// (the dispatch-worker spawn); `Passthrough` for a benign spawn from anywhere else
/// (a plain detached worktree, same provisioning, no worker marker). The validated
/// `name` slug is carried in BOTH arms (D-P2) so the shell does not re-sanitise it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CreateAction {
    /// cwd IS the arming dir and `base` is a plausible sha ⇒ fork off `base` on
    /// `dispatch/<name>`, provision + worker-mark inside the fork.
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
    /// cwd IS the arming dir but the `base` file is absent/empty — no commit to fork.
    MissingBase,
    /// cwd IS the arming dir and `base` is present but not a plausible sha (shape).
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

/// PURE create classifier (no git / disk / env / clock — ADR-001 leaf). Mirror of
/// [`super::subagent::classify_stamp`]: takes the gathered, already-resolved FACTS and
/// returns the verdict; the shell resolves the cwd realpath, the arming-dir realpath
/// compare, and the `base` file read (all impure), then calls this.
///
/// * `cwd_resolved` — the payload carried a `cwd` that resolved (canonicalised) on disk.
/// * `cwd_is_arming_dir` — that resolved cwd IS the arming dir
///   `<coord>/.doctrine/state/dispatch/spawn` (both realpath'd by the shell). Positional
///   arming (D3/D4): discrimination is cwd-as-channel, never a payload class tag.
/// * `base` — the arming dir's `base` file contents (`None` if absent/empty).
/// * `name` — the payload `name`, validated here via [`sanitise_name`].
///
/// Precond order (mirror `classify_stamp`): cwd-resolution → name-validity → (when
/// armed) base-presence → base-shape. cwd before name so a missing cwd names itself
/// first; name before base so a bad name is caught on the benign path too.
pub(crate) fn classify_create(
    cwd_resolved: bool,
    cwd_is_arming_dir: bool,
    base: Option<&str>,
    name: &str,
) -> Result<CreateAction, CreateRefusal> {
    if !cwd_resolved {
        return Err(CreateRefusal::MissingCwd);
    }
    let slug = sanitise_name(name).map_err(CreateRefusal::BadName)?;
    if cwd_is_arming_dir {
        match base {
            None => Err(CreateRefusal::MissingBase),
            Some(b) if !plausible_sha(b) => Err(CreateRefusal::BadBase),
            Some(b) => Ok(CreateAction::Fork {
                base: b.trim().to_string(),
                name: slug,
            }),
        }
    } else {
        Ok(CreateAction::Passthrough { name: slug })
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
            classify_create(false, true, Some(SHA), "agent-abc123"),
            Err(CreateRefusal::MissingCwd)
        );
        assert_eq!(
            classify_create(false, false, None, ""),
            Err(CreateRefusal::MissingCwd)
        );
        assert_eq!(CreateRefusal::MissingCwd.token(), "missing-cwd");
    }

    #[test]
    fn bad_name_refuses_before_base_on_both_channels() {
        // Armed + valid base but a bad name ⇒ bad-name (name precedes base).
        assert_eq!(
            classify_create(true, true, Some(SHA), "a/b"),
            Err(CreateRefusal::BadName(NameRefusal::Slash))
        );
        // Benign channel with a bad name ⇒ bad-name too (name checked on both paths).
        assert_eq!(
            classify_create(true, false, None, ""),
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
            classify_create(true, true, None, "agent-abc123"),
            Err(CreateRefusal::MissingBase)
        );
        assert_eq!(CreateRefusal::MissingBase.token(), "missing-base");
    }

    #[test]
    fn armed_with_unparseable_base_refuses_bad_base() {
        // Non-hex garbage ⇒ bad-base.
        assert_eq!(
            classify_create(true, true, Some("zzz"), "agent-abc123"),
            Err(CreateRefusal::BadBase)
        );
        // Too short (< 4) ⇒ bad-base.
        assert_eq!(
            classify_create(true, true, Some("ab"), "agent-abc123"),
            Err(CreateRefusal::BadBase)
        );
        assert_eq!(CreateRefusal::BadBase.token(), "bad-base");
    }

    #[test]
    fn armed_with_plausible_base_and_valid_name_forks() {
        // cwd IS the arming dir ∧ base is a plausible sha ⇒ Fork; the trimmed sha and
        // validated slug are carried in the verdict (D-P2).
        assert_eq!(
            classify_create(true, true, Some("  68250bcd\n"), "bold-oak-a3f2"),
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
            classify_create(true, false, None, "agent-abc123"),
            Ok(CreateAction::Passthrough {
                name: "agent-abc123".to_string(),
            })
        );
        // A stray base on the benign channel is ignored (positional discrimination).
        assert_eq!(
            classify_create(true, false, Some(SHA), "agent-abc123"),
            Ok(CreateAction::Passthrough {
                name: "agent-abc123".to_string(),
            })
        );
    }
}
