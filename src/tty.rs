// SPDX-License-Identifier: GPL-3.0-only
//! The impure terminal-capability shell (SL-053 PHASE-02 colour; SL-054 PHASE-03 width).
//!
//! Two terminal capabilities are read HERE, in the thin shell, and injected as plain
//! values into the pure render layer ([`crate::listing`]) — which itself never touches
//! env, tty, clock, rng, git, or disk (the pure/imperative split, slices-spec
//! § Architecture, the date/uid injection pattern):
//!
//! - **colour** — reads `NO_COLOR` + isatty, injected as a `bool`. The bool is the
//!   single authority: `owo_colors`' UNCONDITIONAL colorize methods are gated on it in
//!   the leaf, never `if_supports_color` (which would re-read env+tty at apply-time and
//!   smuggle impurity back into the pure layer).
//! - **width** — reads isatty + the `crossterm::terminal::size()` ioctl, injected as an
//!   `Option<u16>`. `None` (a pipe / unreadable / degenerate size) ⇒ no wrapping, so
//!   piped output stays width-free and the SL-053 deterministic goldens stay frozen.
//!
//! Each capability follows the same shape: a thin `stdout_*` wrapper holding the
//! impurities and a pure both-injected decision fn, testable without a real tty.

use std::ffi::OsStr;

use clap::ColorChoice;

/// Resolve the effective colour bool from the CLI flag + auto-detection.
/// `Never` beats `NO_COLOR` beats isatty; `Always` beats non-TTY.
/// The single shell-side authority for colour capability.
pub(crate) fn resolve_color(mode: ColorChoice) -> bool {
    match mode {
        ColorChoice::Never => false,
        ColorChoice::Always => true,
        ColorChoice::Auto => stdout_color_enabled(),
    }
}

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

/// Terminal width for stdout, in columns — `None` ⇒ no wrapping.
///
/// Thin shell (mirrors [`stdout_color_enabled`]): the isatty probe and the
/// `crossterm::terminal::size()` ioctl are the only impurities; the decision is the
/// pure, both-injected [`terminal_width`], testable without a real tty. Wrapping
/// applies only on a tty — piped/redirected output gets `None` and stays width-free,
/// keeping the SL-053 deterministic goldens frozen. The live isatty branch is
/// documented-not-driven (mirrors [`stdout_color_enabled`]): under `cargo test`
/// stdout is not a terminal, so it returns `None`; a pty is out of scope.
pub(crate) fn stdout_terminal_width() -> Option<u16> {
    terminal_width(
        std::io::IsTerminal::is_terminal(&std::io::stdout()),
        crossterm::terminal::size().ok().map(|(cols, _rows)| cols),
    )
}

/// The pure width decision with both impurities injected (`is_tty`, `cols`).
///
/// `None` ⇒ no wrapping (the deterministic SL-053 path): a pipe (`!is_tty`), an
/// unreadable size (`cols == None`), or a degenerate width below [`MIN_WRAP_WIDTH`].
/// Otherwise the live column count flows to the pure render layer, which runs the
/// real grid-dependent fit test ([`crate::listing::render_table`]'s `grid_min_width`,
/// PHASE-02).
fn terminal_width(is_tty: bool, cols: Option<u16>) -> Option<u16> {
    if !is_tty {
        return None;
    }
    match cols {
        Some(w) if w >= MIN_WRAP_WIDTH => Some(w),
        // 0 / unreadably-narrow / unavailable ⇒ fall back to no-wrap.
        _ => None,
    }
}

/// Coarse shell-side pre-filter for degenerate sizes (`size() == 0`, headless /
/// unreadably-narrow terminals): below it, skip wrapping and emit clean overflow.
/// NOT the authoritative fit test — that is grid-dependent (`render_table`'s
/// `grid_min_width`, PHASE-02), which the pure layer applies to the real column
/// count and which already falls back to `Disabled` for any width it can't seat. So
/// this floor protects nothing the grid floor wouldn't; it is a cheap shell-side
/// cutoff (the shell has no grid) that also, as a side effect, suppresses the rare
/// legitimate few-column wrap on a sub-`16` terminal in favour of clean overflow.
const MIN_WRAP_WIDTH: u16 = 16;

#[cfg(test)]
mod tests {
    use super::*;

    /// VT-5: `resolve_color` modes — Never false, Always true, Auto delegates
    /// to stdout_color_enabled. Both tty arms are asserted through the pure
    /// [`color_enabled`] seam; the *live* tty branch is documented-not-driven
    /// (stdout may be a terminal under some harnesses).
    #[test]
    fn resolve_color_modes() {
        assert!(!resolve_color(ColorChoice::Never));
        assert!(resolve_color(ColorChoice::Always));
        // Auto delegates to stdout_color_enabled → color_enabled.
        // The live isatty probe is environment-dependent; both arms are proven
        // by the pure color_enabled tests below (absent_no_color_follows_the_tty +
        // no_color_present_disables_colour_even_when_empty).
    }

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

    /// VT-1: the pure width decision, both impurities injected. A pipe is always
    /// width-free; on a tty the live width passes through above the [`MIN_WRAP_WIDTH`]
    /// floor and collapses to `None` at/below it (incl. the degenerate `0`).
    ///
    /// The *live* isatty branch in [`stdout_terminal_width`] is documented-not-driven
    /// (mirrors `color_enabled`): under `cargo test` stdout is not a terminal, so it
    /// returns `None`; a pty is out of scope.
    #[test]
    fn terminal_width_decides_from_injected_tty_and_cols() {
        // Pipe ⇒ no wrapping, regardless of any reported size.
        assert_eq!(terminal_width(false, None), None);
        assert_eq!(terminal_width(false, Some(80)), None);
        // tty + readable width above the floor ⇒ that width flows through.
        assert_eq!(terminal_width(true, Some(80)), Some(80));
        // tty + degenerate / below-floor width ⇒ fall back to no-wrap.
        assert_eq!(terminal_width(true, Some(0)), None);
        assert_eq!(terminal_width(true, Some(8)), None);
        // tty but size() unreadable ⇒ no-wrap.
        assert_eq!(terminal_width(true, None), None);
        // Boundary: the floor itself is inclusive.
        assert_eq!(
            terminal_width(true, Some(MIN_WRAP_WIDTH)),
            Some(MIN_WRAP_WIDTH)
        );
    }
}
