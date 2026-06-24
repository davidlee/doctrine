// SPDX-License-Identifier: GPL-3.0-only
//! `globmatch` — the shared glob-match leaf (ADR-001). One neutral home for the
//! single `glob::Pattern` comparison policy, consumed by both the worktree
//! allowlist (`worktree::allowlist`) and conformance (`crate::conformance`, and
//! the staleness resolution that follows). Pure: the external `glob` crate +
//! std only — no disk, git, clock, or rng.

use glob::{MatchOptions, Pattern};

/// Match options shared by every glob comparison: `**` is the *only* way to cross
/// a path separator, so a single `*` matches one component (gitignore-ish, and
/// what keeps `*` from silently spanning `.doctrine/state/...`).
const MATCH_OPTS: MatchOptions = MatchOptions {
    case_sensitive: true,
    require_literal_separator: true,
    require_literal_leading_dot: false,
};

/// Does `pat` match `path` under the shared [`MATCH_OPTS`] policy? A literal path
/// is a degenerate glob — it matches itself and nothing else.
pub(crate) fn glob_matches(pat: &Pattern, path: &str) -> bool {
    pat.matches_with(path, MATCH_OPTS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_path_matches_itself_only() {
        let pat = Pattern::new("src/conformance.rs").unwrap();
        assert!(glob_matches(&pat, "src/conformance.rs"));
        assert!(!glob_matches(&pat, "src/globmatch.rs"));
    }

    #[test]
    fn single_star_does_not_cross_separators() {
        let pat = Pattern::new("src/*.rs").unwrap();
        assert!(glob_matches(&pat, "src/state.rs"));
        assert!(!glob_matches(&pat, "src/worktree/mod.rs"));
    }

    #[test]
    fn double_star_crosses_separators() {
        let pat = Pattern::new("src/**").unwrap();
        assert!(glob_matches(&pat, "src/state.rs"));
        assert!(glob_matches(&pat, "src/worktree/mod.rs"));
    }
}
