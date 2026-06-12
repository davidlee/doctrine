// SPDX-License-Identifier: GPL-3.0-only
//! The impure colour-capability shell (SL-053 PHASE-02, D3).
//!
//! Colour *capability* is the one impurity the colour seam needs: it reads the
//! `NO_COLOR` environment variable and whether stdout is a terminal. Per the
//! pure/imperative split (slices-spec § Architecture, the date/uid injection
//! pattern) that read lives HERE, in the thin shell, and is injected as a plain
//! `bool` into the pure render layer ([`crate::listing`]) — which itself never
//! touches env, tty, clock, rng, git, or disk.
//!
//! The bool is the single authority: `owo_colors`' UNCONDITIONAL colorize methods
//! are gated on it in the leaf, never `if_supports_color` (which would re-read
//! env+tty at apply-time and smuggle impurity back into the pure layer).

use std::ffi::OsStr;

/// Whether colour should be emitted on stdout.
///
/// Thin shell: the env read (`NO_COLOR`) and the tty probe are the only impurities;
/// the decision itself is the pure, env-injected [`color_enabled`] so it is testable
/// without mutating the process environment (`set_var` is forbidden crate-wide —
/// CLAUDE.md pure/imperative split, mirroring `git::trunk_tree_ish`). `var_os` — the
/// repo bans `std::env::var` (`disallowed_methods`).
pub(crate) fn stdout_color_enabled() -> bool {
    color_enabled(
        std::env::var_os("NO_COLOR").as_deref(),
        std::io::IsTerminal::is_terminal(&std::io::stdout()),
    )
}

/// The pure colour-capability decision with both impurities injected.
///
/// `NO_COLOR` precedence: its mere *presence* (even empty, `Some("")`) disables
/// colour, per the `NO_COLOR` convention (<https://no-color.org>). Absent ⇒ colour
/// follows `is_tty`, so
/// piped/redirected output stays plain (the goldens run piped ⇒ colour-free, VT-4).
fn color_enabled(no_color: Option<&OsStr>, is_tty: bool) -> bool {
    if no_color.is_some() {
        return false;
    }
    is_tty
}

#[cfg(test)]
mod tests {
    use super::*;

    /// VT-3: `NO_COLOR` present (even empty) ⇒ colour disabled, regardless of the
    /// tty arm. Driven through the pure seam ([`color_enabled`]) — the process env
    /// is never mutated (`set_var` is forbidden crate-wide).
    ///
    /// The positive isatty arm (`None` + `true` ⇒ `true`) is asserted purely here;
    /// the *live* tty branch in [`stdout_color_enabled`] is exercised only
    /// indirectly (under `cargo test` stdout is not a terminal, so it returns
    /// `false`) — documented rather than driven, as a pty is out of scope.
    #[test]
    fn no_color_present_disables_colour_even_when_empty() {
        assert!(
            !color_enabled(Some(OsStr::new("")), true),
            "NO_COLOR present (empty) must disable colour even on a tty"
        );
        assert!(
            !color_enabled(Some(OsStr::new("1")), true),
            "NO_COLOR present (non-empty) must disable colour"
        );
    }

    #[test]
    fn absent_no_color_follows_the_tty() {
        assert!(
            color_enabled(None, true),
            "no NO_COLOR + tty ⇒ colour enabled"
        );
        assert!(
            !color_enabled(None, false),
            "no NO_COLOR + non-tty (pipe) ⇒ colour disabled"
        );
    }
}
