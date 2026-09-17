// SPDX-License-Identifier: GPL-3.0-only
//! `graphviz` — the `dot -Tpng` render leaf (ADR-001, DEC-143, SL-245 PHASE-02).
//! One neutral home for the CLI's synchronous raster spawn and the shared
//! `dot` program name; `map_server` (`shell.rs`, `routes.rs`, `error.rs`)
//! consumes [`DOT_PROGRAM`] for its own, separately-timed, async spawns
//! (DEC-143 — the program name single-sources, each timeout stays where it's
//! enforced). Pure `classify` + a thin impure shell (`rasterise_png`): std
//! only (`out=0`, no crate imports, no domain knowledge). No deadline —
//! IMP-452 defers the bounded render; Ctrl-C is the interactive user's
//! timeout for this CLI spawn.

use std::ffi::OsStr;
use std::io::Write;
use std::process::{Command, Output, Stdio};

/// The `dot` program name — single-sourced (STD-001) across `graphviz` and
/// every `map_server` spawn site (DEC-143).
pub(crate) const DOT_PROGRAM: &str = "dot";

/// Output format: a PNG raster. `kitty::encode_png` transmits `f=100`.
const ARG_FORMAT_PNG: &str = "-Tpng";

/// Graph-level resolution override, `-Gdpi=<n>`.
const ARG_DPI_PREFIX: &str = "-Gdpi=";

/// Graph-level drawing size, `-Gsize=<w>,<h>` — inches, not pixels. A plain
/// `size` is a MAXIMUM only: a drawing larger than the box is scaled down to
/// it, a smaller one is left alone. Apparent size is set by the dpi the
/// caller asks for (ISS-459), not by stretching the drawing to the window —
/// the trailing `!` that did that blew a one-node graph up to fill the
/// terminal.
const ARG_SIZE_PREFIX: &str = "-Gsize=";

/// Decimal places on the inch conversion — enough that a box stated in whole
/// pixels round-trips at any plausible dpi.
const SIZE_PRECISION: usize = 4;

/// Floor on a caller-supplied dpi, so a terminal reporting an implausibly
/// small cell cannot ask for an unreadable raster.
const MIN_RASTER_DPI: u32 = 48;

/// The scaling brief for one raster: the pixel box the drawing must fit
/// INSIDE — the terminal's usable width, and whatever height the caller's
/// image budget leaves — and the resolution its text is drawn at.
///
/// The two are independent. `dpi` sets apparent size (how big the text comes
/// out); the box only ever shrinks a drawing that would overflow it. A small
/// graph therefore stays small instead of being stretched to the window.
///
/// Pixels rather than inches because the caller reasons in cells and window
/// geometry; converting to graphviz's units is this leaf's business.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FitBox {
    pub(crate) width_px: u32,
    pub(crate) height_px: u32,
    /// Raster resolution in dots per inch, which is what decides apparent
    /// size. The caller picks it from the terminal's cell geometry so graph
    /// text lands at about the size of terminal text.
    pub(crate) dpi: u32,
}

/// Outcome of a `dot -Tpng` raster spawn.
#[derive(Debug)]
pub(crate) enum RasterOutcome {
    /// The rendered PNG bytes (`dot`'s stdout on a clean exit), and whatever
    /// `dot` said on its stderr while succeeding.
    ///
    /// `notes` is carried rather than dropped because a zero exit does not mean
    /// an undegraded render: graphviz reports a forced downscale this way —
    /// `graph is too large for cairo-renderer bitmaps. Scaling by 0.668933 to
    /// fit` — and discarding it hid exactly that from the caller (RV-369 F-8;
    /// STD-003, a degraded read is disclosed). Usually empty.
    Png { png: Vec<u8>, notes: String },
    /// `program` was not found (`io::ErrorKind::NotFound`).
    ToolUnavailable,
    /// `dot` exited non-zero; `status` is its exit code (`None` if killed by
    /// a signal) and `stderr` is its captured error output.
    CommandFailed { status: Option<i32>, stderr: String },
    /// Any other I/O failure spawning or waiting on the child. Kept distinct
    /// from `ToolUnavailable` rather than folded in (STD-003: a degraded read
    /// is disclosed, not misreported as a different, more specific one).
    Io(std::io::Error),
}

/// Render `dot` source to PNG bytes via `program -Tpng`, synchronously and
/// with no deadline (IMP-452; Ctrl-C is the interactive user's timeout).
///
/// `RealDotRenderer` (`map_server::shell`) is the async counterpart of this
/// sync shell, for map-server's own request lifecycle (DEC-143).
pub(crate) fn rasterise_png(dot: &[u8], program: &OsStr, fit: FitBox) -> RasterOutcome {
    classify(run(dot, program, fit))
}

/// Spawn `program -Tpng`, write `dot` on its own thread, and collect the
/// result with `wait_with_output` (the stdlib drains stdout AND stderr
/// concurrently on the caller's side, avoiding the two-pipe deadlock). Only
/// the stdin write needs its own thread: a DOT payload larger than the stdin
/// pipe buffer would otherwise block the caller before `dot` starts reading,
/// and nothing else here drains stdin's other end concurrently with the
/// write (EX-7). The writer thread is never joined — nothing needs its
/// result, and joining would buy nothing but a chance to block on a wedged
/// child that `wait_with_output` is already waiting on.
fn run(dot: &[u8], program: &OsStr, fit: FitBox) -> std::io::Result<Output> {
    let mut child = Command::new(program)
        .args(raster_args(fit))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    #[expect(
        clippy::expect_used,
        reason = "stdin configured as Stdio::piped() so take() always returns Some"
    )]
    let mut stdin = child
        .stdin
        .take()
        .expect("stdin configured as Stdio::piped()");
    let dot_owned = dot.to_vec();
    std::thread::spawn(move || {
        // A child that exits early (e.g. dot's own syntax-error path) closes
        // its read end first; the write then fails with `BrokenPipe`. That is
        // not this thread's error to report — the exit status and stderr
        // already carry it (EX-7). Swallow it (repo idiom: a discarded
        // must-use `Result` goes through `drop`, per `coverage_verify::reap`):
        // no unwrap, no propagation.
        drop(stdin.write_all(&dot_owned));
    });
    child.wait_with_output()
}

/// The arguments `dot` is spawned with: PNG out, at `fit`'s resolution,
/// bounded by `fit`'s box.
fn raster_args(fit: FitBox) -> [String; 3] {
    let dpi = fit.dpi.max(MIN_RASTER_DPI);
    let inches = |px: u32| f64::from(px) / f64::from(dpi);
    [
        ARG_FORMAT_PNG.to_owned(),
        format!("{ARG_DPI_PREFIX}{dpi}"),
        format!(
            "{ARG_SIZE_PREFIX}{:.*},{:.*}",
            SIZE_PRECISION,
            inches(fit.width_px),
            SIZE_PRECISION,
            inches(fit.height_px)
        ),
    ]
}

/// Pure classification of a raw spawn result into a [`RasterOutcome`].
fn classify(run: std::io::Result<Output>) -> RasterOutcome {
    match run {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => RasterOutcome::ToolUnavailable,
        Err(e) => RasterOutcome::Io(e),
        Ok(output) if output.status.success() => RasterOutcome::Png {
            png: output.stdout,
            notes: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        },
        Ok(output) => RasterOutcome::CommandFailed {
            status: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a synthetic `Output` from a raw POSIX wait-status (`0` normal
    /// exit, `code << 8` for a nonzero exit) without spawning anything —
    /// the same idiom `src/git.rs`'s `CannedPush` test double uses.
    #[cfg(unix)]
    fn synthetic_output(raw_status: i32, stdout: &[u8], stderr: &[u8]) -> Output {
        use std::os::unix::process::ExitStatusExt;
        Output {
            status: ExitStatusExt::from_raw(raw_status),
            stdout: stdout.to_vec(),
            stderr: stderr.to_vec(),
        }
    }

    #[test]
    fn classify_not_found_is_tool_unavailable() {
        let err = std::io::Error::from(std::io::ErrorKind::NotFound);
        assert!(matches!(classify(Err(err)), RasterOutcome::ToolUnavailable));
    }

    #[test]
    fn classify_other_io_error_is_io() {
        let err = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
        assert!(matches!(classify(Err(err)), RasterOutcome::Io(_)));
    }

    #[test]
    #[cfg(unix)]
    fn classify_success_is_png_with_stdout() {
        let output = synthetic_output(0, b"\x89PNG", b"");
        match classify(Ok(output)) {
            RasterOutcome::Png { png, notes } => {
                assert_eq!(png, b"\x89PNG");
                assert!(notes.is_empty(), "a clean render says nothing: {notes:?}");
            }
            other => panic!("expected Png, got {other:?}"),
        }
    }

    /// RV-369 F-8: a zero exit does not mean an undegraded render. graphviz
    /// announces a forced downscale on stderr and still succeeds, and the
    /// caller must be able to see it (STD-003).
    #[test]
    #[cfg(unix)]
    fn classify_success_carries_what_dot_said_while_succeeding() {
        let warning =
            b"graph is too large for cairo-renderer bitmaps. Scaling by 0.668933 to fit\n";
        let output = synthetic_output(0, b"\x89PNG", warning);
        match classify(Ok(output)) {
            RasterOutcome::Png { notes, .. } => {
                assert!(notes.contains("too large"), "{notes}");
                assert!(
                    !notes.ends_with('\n'),
                    "trimmed for a one-line warning: {notes:?}"
                );
            }
            other => panic!("expected Png, got {other:?}"),
        }
    }

    #[test]
    #[cfg(unix)]
    fn classify_exit_1_is_command_failed_with_status_and_stderr() {
        let output = synthetic_output(1 << 8, b"", b"syntax error");
        match classify(Ok(output)) {
            RasterOutcome::CommandFailed { status, stderr } => {
                assert_eq!(status, Some(1));
                assert_eq!(stderr, "syntax error");
            }
            other => panic!("expected CommandFailed, got {other:?}"),
        }
    }

    /// A box in inches is what graphviz speaks; the caller thinks in the
    /// terminal's pixels, so the conversion is this leaf's job.
    #[test]
    fn raster_args_state_the_fit_box_in_inches_at_the_named_resolution() {
        let args = raster_args(FitBox {
            width_px: 960,
            height_px: 480,
            dpi: 96,
        });
        assert_eq!(args[0], "-Tpng");
        assert_eq!(args[1], "-Gdpi=96");
        assert_eq!(args[2], "-Gsize=10.0000,5.0000");
    }

    /// The box is a MAXIMUM only — no trailing `!`. With it, graphviz stretches
    /// any smaller drawing up to meet the box, which blew a one-node graph up
    /// to fill the terminal (ISS-459). Apparent size is `dpi`'s job.
    #[test]
    fn the_fit_box_never_enlarges_a_drawing() {
        let args = raster_args(FitBox {
            width_px: 100,
            height_px: 100,
            dpi: 96,
        });
        assert!(!args[2].ends_with('!'), "{}", args[2]);
    }

    /// A dpi below the floor would ask `dot` for an unreadable raster.
    #[test]
    fn raster_args_floor_an_implausibly_low_dpi() {
        let args = raster_args(FitBox {
            width_px: 960,
            height_px: 480,
            dpi: 1,
        });
        assert_eq!(args[1], format!("-Gdpi={MIN_RASTER_DPI}"));
    }

    #[test]
    fn rasterise_png_nonexistent_program_is_tool_unavailable() {
        let result = rasterise_png(
            b"digraph { a -> b }",
            OsStr::new("/nonexistent/definitely-not-a-dot-binary"),
            FitBox {
                width_px: 800,
                height_px: 600,
                dpi: 96,
            },
        );
        assert!(matches!(result, RasterOutcome::ToolUnavailable));
    }
}
