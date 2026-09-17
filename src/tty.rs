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
use std::io::{Read, Write};
use std::time::{Duration, Instant};

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

// ── SL-245 PHASE-03: the single verified render terminal (DEC-259) ──────────
//
// A second, unrelated capability on the same seam. `-X` must size, raw-mode,
// query and WRITE TO ONE DEVICE: otherwise a positive probe of one terminal
// could authorise output to another. `stdout_terminal_width` above cannot
// serve — it returns `None` for both a pipe and an unreadable size, and it
// probes through crossterm, which picks its own descriptor. So this half opens
// the controlling terminal once, proves it is stdout's device, and does
// everything else on that one file descriptor.

/// The controlling terminal's device node — opened read-write and BLOCKING.
/// POSIX lets `O_NONBLOCK` override `VTIME`, which would defeat the timed
/// reads the whole exchange is bounded by (DEC-259), so it is never passed.
const CONTROLLING_TERMINAL: &str = "/dev/tty";

/// `VMIN` for the query exchange: 0 ⇒ a `read` never waits for a minimum byte
/// count, it returns as soon as anything arrives or [`QUERY_VTIME`] expires.
const QUERY_VMIN: u8 = 0;

/// `VTIME` for the query exchange, in tenths of a second: 1 ⇒ ~100 ms. The
/// read itself is what bounds each wait, so no thread outlives the query and
/// no readiness call is needed — `poll` does not work on `/dev/tty` on macOS
/// and `select` is `unsafe` in rustix (DEC-259).
const QUERY_VTIME: u8 = 1;

/// Read granularity for the reply loop. Replies are tens of bytes; the buffer
/// only has to be larger than one arrival, never than the whole exchange.
const QUERY_READ_BUFFER: usize = 1024;

/// The `tcsetattr` action for BOTH the raw-mode entry and the restore: apply
/// immediately (`TCSANOW`). Not `Flush` — it discards unread input, and a
/// reply may already be sitting in the buffer when the restore runs. Not
/// `Drain` — it blocks on pending output, and the restore must be reliable on
/// every path, including the error and unwind paths.
const TERMIOS_ACTION: rustix::termios::OptionalActions = rustix::termios::OptionalActions::Now;

/// What `open_render_terminal` found: either the one verified terminal, or the
/// named reason there isn't one. Both refusals are typed outcomes, never a
/// swallowed error (STD-003).
#[expect(
    dead_code,
    reason = "the Terminal payload is not read until PHASE-05 (SL-245) matches on \
              RenderTarget in terminal_image::prepare"
)]
pub(crate) enum RenderTarget {
    NotTerminal,
    /// stdout is a terminal, but not the controlling one — or there is none.
    NotControllingTerminal,
    Terminal(RenderTerminal),
}

/// The controlling terminal, verified to be stdout's own device. Every later
/// `termios`, winsize and I/O call goes through this one handle.
#[expect(
    dead_code,
    reason = "`window` is not read until PHASE-05 (SL-245) feeds it to \
              kitty::cell_geometry"
)]
pub(crate) struct RenderTerminal {
    tty: std::fs::File,
    pub(crate) window: WindowGeometry,
}

/// `tcgetwinsize` as reported. `0` means the terminal did not report the
/// field — NOT judged here; `kitty::cell_geometry` is what refuses a
/// degenerate geometry, so this type stays a faithful record of the probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WindowGeometry {
    pub(crate) columns: u16,
    pub(crate) rows: u16,
    pub(crate) pixel_width: u16,
    pub(crate) pixel_height: u16,
}

/// How a query exchange failed. The two arms are ranked, not merely distinct —
/// see [`bracket`].
#[derive(Debug)]
#[expect(
    dead_code,
    reason = "the wrapped io::Errors are not read until PHASE-05 (SL-245) renders \
              them as a RenderRefusal; the tests match the variants, not the payloads"
)]
pub(crate) enum QueryError {
    /// Entering raw mode, writing, or reading failed; the terminal WAS restored.
    Io(std::io::Error),
    /// Restoring the saved settings failed. Takes precedence over whatever the
    /// exchange produced, because the user's shell is what is left wrong.
    Restore(std::io::Error),
}

/// The topology the probes describe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Endpoint {
    NotTerminal,
    NotControlling,
    Same,
}

/// The pure endpoint decision with every probe injected (device ids are
/// `st_rdev`).
///
/// `tty_device: None` carries "`/dev/tty` would not open" — EX-2's "a failed
/// `/dev/tty` open maps to `NotControlling`". Modelling it as an absent device
/// rather than a propagated error keeps the *decision* here, in the pure
/// layer, and keeps the shell free of a nested `Result`.
fn endpoint(stdout_is_tty: bool, stdout_device: u64, tty_device: Option<u64>) -> Endpoint {
    if !stdout_is_tty {
        return Endpoint::NotTerminal;
    }
    match tty_device {
        Some(device) if device == stdout_device => Endpoint::Same,
        // No controlling terminal, or a different device than stdout's.
        _ => Endpoint::NotControlling,
    }
}

/// Thin shell: isatty(stdout); open `/dev/tty` read-write and blocking;
/// `fstat` both; `tcgetwinsize` on the tty. The decision itself is the pure
/// [`endpoint`], so the interesting part is testable without a terminal.
#[expect(
    dead_code,
    reason = "no production caller until PHASE-05 (SL-245) wires terminal_image::prepare \
              to open_render_terminal; the pure decisions it roots (endpoint, bracket) \
              are driven by this module's own tests until then"
)]
pub(crate) fn open_render_terminal() -> std::io::Result<RenderTarget> {
    let stdout_is_tty = std::io::IsTerminal::is_terminal(&std::io::stdout());
    let stdout_device = rustix::fs::fstat(std::io::stdout())?.st_rdev;
    // A `/dev/tty` that will not open IS the "no controlling terminal" answer
    // (EX-2) — the error is not propagated, because it is not a failure of
    // this function, it is one of the outcomes it reports.
    let tty = rustix::fs::open(
        CONTROLLING_TERMINAL,
        rustix::fs::OFlags::RDWR,
        rustix::fs::Mode::empty(),
    )
    .ok()
    .map(std::fs::File::from);
    let tty_device = tty
        .as_ref()
        .map(rustix::fs::fstat)
        .transpose()?
        .map(|stat| stat.st_rdev);

    match (endpoint(stdout_is_tty, stdout_device, tty_device), tty) {
        (Endpoint::Same, Some(tty)) => {
            let size = rustix::termios::tcgetwinsize(&tty)?;
            Ok(RenderTarget::Terminal(RenderTerminal {
                window: WindowGeometry {
                    columns: size.ws_col,
                    rows: size.ws_row,
                    pixel_width: size.ws_xpixel,
                    pixel_height: size.ws_ypixel,
                },
                tty,
            }))
        }
        (Endpoint::NotTerminal, _) => Ok(RenderTarget::NotTerminal),
        // `Same` implies `tty_device` was `Some`, so the `(Same, None)` half of
        // this arm is unreachable; folding it in keeps the match total without
        // a panic, and reports the conservative answer if it ever were.
        (Endpoint::NotControlling | Endpoint::Same, _) => Ok(RenderTarget::NotControllingTerminal),
    }
}

impl RenderTerminal {
    /// Raw mode with timed reads on this one fd: write `request`, read until
    /// `complete(&bytes_so_far)` or `timeout`, then restore. `Ok(None)` is the
    /// deadline (EX-4).
    ///
    /// VA-1: there is exactly one way out of the exchange — [`bracket`], whose
    /// `exit` closure below is the explicit, CHECKED restore. Nothing here
    /// returns around it.
    #[expect(
        dead_code,
        reason = "no production caller until PHASE-05 (SL-245) wires terminal_image::prepare \
                  to RenderTerminal::query for the kitty support probe"
    )]
    pub(crate) fn query(
        &self,
        request: &[u8],
        complete: impl Fn(&[u8]) -> bool,
        timeout: Duration,
    ) -> Result<Option<Vec<u8>>, QueryError> {
        bracket(
            // enter: save the current settings, then switch to raw with timed
            // reads. The guard is armed only AFTER the mode actually changed —
            // before that there is nothing to restore.
            || {
                let saved = rustix::termios::tcgetattr(&self.tty)?;
                let mut raw = saved.clone();
                raw.make_raw();
                raw.special_codes[rustix::termios::SpecialCodeIndex::VMIN] = QUERY_VMIN;
                raw.special_codes[rustix::termios::SpecialCodeIndex::VTIME] = QUERY_VTIME;
                rustix::termios::tcsetattr(&self.tty, TERMIOS_ACTION, &raw)?;
                Ok(RestoreGuard {
                    tty: &self.tty,
                    saved,
                    armed: true,
                })
            },
            // body: the exchange itself. It never restores — `bracket`
            // guarantees `exit` runs whichever way this returns.
            |_guard| self.exchange(request, &complete, timeout),
            // exit: the explicit, checked restore — reached on EVERY path out
            // of `body`, success and error alike (VA-1) — and only then is the
            // unwind-only guard disarmed. The restore's own result is returned
            // unswallowed, so `bracket` can rank it above the body's (EX-3).
            |mut guard| {
                let restored = rustix::termios::tcsetattr(&self.tty, TERMIOS_ACTION, &guard.saved)
                    .map_err(std::io::Error::from);
                guard.disarm();
                restored
            },
        )
    }

    /// Write the request, then read until `complete` holds or the deadline
    /// passes. `Ok(None)` is the deadline, which is honoured to within one
    /// [`QUERY_VTIME`] read.
    fn exchange(
        &self,
        request: &[u8],
        complete: &impl Fn(&[u8]) -> bool,
        timeout: Duration,
    ) -> std::io::Result<Option<Vec<u8>>> {
        let deadline = Instant::now() + timeout;
        // `&File` is `Read`/`Write` on Unix, so the whole exchange rides the
        // single handle without a `&mut` borrow of `self`.
        (&self.tty).write_all(request)?;
        (&self.tty).flush()?;

        let mut received = Vec::new();
        let mut chunk = [0u8; QUERY_READ_BUFFER];
        loop {
            if complete(&received) {
                return Ok(Some(received));
            }
            if Instant::now() >= deadline {
                return Ok(None);
            }
            // Bounded by `VTIME`, not by a readiness call: 0 bytes back just
            // means "nothing arrived in ~100 ms", not end of input.
            let read = (&self.tty).read(&mut chunk)?;
            match chunk.get(..read) {
                Some(arrived) => received.extend_from_slice(arrived),
                // Unreachable: `read` counts bytes written INTO `chunk`, so it
                // cannot exceed its length. Disclosed rather than skipped
                // (STD-003) — a truncated read must never look like a clean one.
                None => {
                    return Err(std::io::Error::other(
                        "terminal read reported more bytes than the buffer holds",
                    ));
                }
            }
        }
    }
}

/// Best-effort restore WHILE UNWINDING FROM A PANIC, and only then.
///
/// This is a fallback, never the guarantee: a destructor cannot report a
/// failed restore, which is exactly why the explicit restore in `query`'s
/// `exit` exists and is checked (DEC-259). On every ordinary path — success,
/// `Io`, and `Restore` alike — `exit` has already run and called [`disarm`],
/// so this must not restore a second time.
///
/// [`disarm`]: RestoreGuard::disarm
struct RestoreGuard<'a> {
    tty: &'a std::fs::File,
    saved: rustix::termios::Termios,
    armed: bool,
}

impl RestoreGuard<'_> {
    /// Stand the guard down: the explicit, checked restore has run.
    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for RestoreGuard<'_> {
    fn drop(&mut self) {
        if self.armed && std::thread::panicking() {
            // Nothing to report to mid-unwind; the alternative is leaving the
            // user's shell in raw mode.
            let _restore = rustix::termios::tcsetattr(self.tty, TERMIOS_ACTION, &self.saved);
        }
    }
}

/// Enter, run, always exit — with the closures injected so every path is unit
/// tested without a terminal (VT-2).
///
/// The precedence is the whole point, and it is stated here once:
///
/// - `enter` fails ⇒ neither `body` nor `exit` runs. Nothing changed, so there
///   is nothing to restore; the failure is [`QueryError::Io`].
/// - `enter` succeeds ⇒ `exit` runs, whichever way `body` went.
/// - an `exit` error BEATS the body result — a body `Ok` *and* a body `Err` —
///   and surfaces as [`QueryError::Restore`], because a shell left in raw mode
///   outranks whatever the exchange did or didn't learn.
/// - otherwise a body error is [`QueryError::Io`], and a clean run returns the
///   body's value.
fn bracket<S, R>(
    enter: impl FnOnce() -> std::io::Result<S>,
    body: impl FnOnce(&S) -> std::io::Result<R>,
    exit: impl FnOnce(S) -> std::io::Result<()>,
) -> Result<R, QueryError> {
    let state = enter().map_err(QueryError::Io)?;
    // From here on, `exit` runs on every path out of `body`.
    let outcome = body(&state);
    match (exit(state), outcome) {
        // Restore failure wins over BOTH a body value and a body error.
        (Err(restore), _) => Err(QueryError::Restore(restore)),
        (Ok(()), Ok(value)) => Ok(value),
        (Ok(()), Err(io)) => Err(QueryError::Io(io)),
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

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

    // ── SL-245 PHASE-03: the render-terminal endpoint ────────────────────

    /// A distinct synthetic I/O failure for the `bracket` path tests.
    fn io_err() -> std::io::Error {
        std::io::Error::other("synthetic")
    }

    /// VT-1: the pure endpoint decision over injected probe results (device ids
    /// are `st_rdev`). A failed `/dev/tty` open reaches here as `None` — the
    /// shell never propagates that error (EX-2), so "no controlling terminal"
    /// is a typed outcome rather than a swallowed one (STD-003).
    #[test]
    fn endpoint_decides_from_injected_probe_results() {
        // stdout is not a terminal at all — device ids are irrelevant.
        assert_eq!(endpoint(false, 0x8801, Some(0x8801)), Endpoint::NotTerminal);
        // stdout is a terminal, but `/dev/tty` would not open: no controlling one.
        assert_eq!(endpoint(true, 0x8801, None), Endpoint::NotControlling);
        // Both are terminals, but different devices — a probe of one must never
        // authorise output to the other.
        assert_eq!(
            endpoint(true, 0x8801, Some(0x8802)),
            Endpoint::NotControlling
        );
        // One verified endpoint.
        assert_eq!(endpoint(true, 0x8801, Some(0x8801)), Endpoint::Same);
    }

    /// VT-2: `bracket`'s success path — every stage runs, the body's value is
    /// returned, and `exit` still runs.
    #[test]
    fn bracket_returns_the_body_result_when_every_stage_succeeds() {
        let ran_exit = Cell::new(false);
        let out = bracket(
            || Ok(7u8),
            |state| Ok(u32::from(*state) + 1),
            |_state| {
                ran_exit.set(true);
                Ok(())
            },
        );
        assert!(matches!(out, Ok(8)));
        assert!(ran_exit.get(), "exit runs on the success path");
    }

    /// VT-2: a body failure still restores, and surfaces as `QueryError::Io`.
    #[test]
    fn bracket_runs_exit_and_reports_io_when_the_body_fails() {
        let ran_exit = Cell::new(false);
        let out = bracket(
            || Ok(7u8),
            |_state| Err::<u32, _>(io_err()),
            |_state| {
                ran_exit.set(true);
                Ok(())
            },
        );
        assert!(matches!(out, Err(QueryError::Io(_))));
        assert!(ran_exit.get(), "exit runs whenever enter succeeded");
    }

    /// VT-2: an exit error beats a *successful* body — the body's value is
    /// discarded, because the user's shell is what is affected.
    #[test]
    fn bracket_lets_a_restore_error_beat_a_successful_body() {
        let out = bracket(|| Ok(7u8), |_state| Ok(1u32), |_state| Err(io_err()));
        assert!(
            matches!(out, Err(QueryError::Restore(_))),
            "a restore failure beats the body's Ok"
        );
    }

    /// VT-2: an exit error beats a *failed* body too — `Restore` wins over
    /// `Io`, not merely over success.
    #[test]
    fn bracket_lets_a_restore_error_beat_a_body_error() {
        let out = bracket(
            || Ok(7u8),
            |_state| Err::<u32, _>(io_err()),
            |_state| Err(io_err()),
        );
        assert!(
            matches!(out, Err(QueryError::Restore(_))),
            "a restore failure beats a body failure too"
        );
    }

    /// VT-2: an enter failure runs neither the body nor the exit — there is
    /// nothing to restore, so reporting it as `QueryError::Io` is honest.
    #[test]
    fn bracket_runs_neither_body_nor_exit_when_enter_fails() {
        let ran_body = Cell::new(false);
        let ran_exit = Cell::new(false);
        let out = bracket::<u8, u32>(
            || Err(io_err()),
            |_state| {
                ran_body.set(true);
                Ok(1)
            },
            |_state| {
                ran_exit.set(true);
                Ok(())
            },
        );
        assert!(matches!(out, Err(QueryError::Io(_))));
        assert!(
            !ran_body.get(),
            "the body must not run after a failed enter"
        );
        assert!(!ran_exit.get(), "there is nothing to restore");
    }
}
