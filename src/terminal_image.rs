// SPDX-License-Identifier: GPL-3.0-only
//! `terminal_image` — the `--render` (`-X`) guard and the DOT→terminal pipeline
//! (ADR-001, SL-245 PHASE-05, design sec-3).
//!
//! Two shells over three leaves ([`crate::graphviz`], [`crate::kitty`],
//! [`crate::tty`]), and nothing else:
//!
//! - [`prepare`] answers *"may I draw at all, and at what cell size?"* — the
//!   EX-2 check order, **cheapest first**, stopping at the first refusal:
//!   format → controlling terminal → pixel size → kitty support probe. The
//!   pixel-size stop sits BEFORE the probe deliberately: a terminal that cannot
//!   report pixels is refused without being talked to.
//! - [`render_dot`] rasterises, discloses anything `dot` said, refuses an image
//!   too large to send, and encodes into ONE buffer, so the caller has a single
//!   `write_all` and no half-written escape can survive a failure.
//!
//! **Nothing here writes to stdout.** Every stop is a refusal *value*, returned
//! to the command shell, which is what makes "no escape byte reaches stdout on a
//! refusal path" a structural property rather than a discipline (VA-1).
//!
//! Every user-facing message is a named constant (STD-001) carrying design
//! sec-3's wording verbatim, with `{…}` placeholders for the interpolations.
//! They are constants rather than `format!` literals because the same string is
//! asserted by the tests and read by the user, and a second copy would drift.
//! No deadline governs the render itself (IMP-452); the only clock here is the
//! support probe's own budget, [`SUPPORT_PROBE_TIMEOUT`].

use std::ffi::OsStr;
use std::time::Duration;

use crate::graphviz::RasterOutcome;
use crate::kitty::{self, CellGeometry, SupportReply};
use crate::tty::{QueryError, RenderTarget, RenderTerminal, WindowGeometry};

// ── The probe budget ───────────────────────────────────────────────────────

/// How long the kitty support probe waits for an answer before giving up and
/// refusing as [`RenderRefusal::Unconfirmed`]. This is the PROBE's budget, not
/// a render deadline — there is no render deadline (IMP-452).
///
/// Single-sourced (STD-001): [`MSG_UNCONFIRMED`] names the same duration to the
/// user, and gets it from here via [`support_probe_timeout_text`] rather than
/// repeating the number.
const SUPPORT_PROBE_TIMEOUT: Duration = Duration::from_secs(2);

/// The probe budget as the user reads it, e.g. `2s`.
fn support_probe_timeout_text() -> String {
    format!("{}s", SUPPORT_PROBE_TIMEOUT.as_secs())
}

// ── The image budget (RV-369 F-6) ──────────────────────────────────────────

/// Bytes a terminal spends holding one decoded pixel: 8-bit RGBA, which is
/// what `dot -Tpng` emits and what the kitty protocol stores.
const BYTES_PER_PIXEL: u64 = 4;

/// The kitty graphics protocol's documented default image storage quota.
const TERMINAL_IMAGE_QUOTA_BYTES: u64 = 320_000_000;

/// Pixels per megapixel — the unit the refusal speaks, because nobody converts
/// a raster's dimensions into bytes by eye.
const PIXELS_PER_MEGAPIXEL: u64 = 1_000_000;

/// The largest raster `-X` will hand the terminal, in PIXELS.
///
/// `DEC-256` bounds the placement in CELLS and nothing in resources, which is
/// not a bound at all: the whole corpus places into a perfectly reasonable
/// 199 x 237 rectangle on top of a raster ghostty silently declines, leaving
/// the user 237 blank rows (RV-369 F-6). `q=2` means the terminal's refusal is
/// unreportable (F-7), so an image that would be rejected has to be refused
/// HERE, before it is sent — the same shape as every other stop in [`prepare`],
/// and the one POL-002 facet (3) asks for.
///
/// **Pixels, not bytes.** An earlier revision bounded the PNG's compressed
/// size. That is the wrong unit twice over: the terminal budgets DECODED
/// pixels, and PNG compresses a sparse line drawing by two orders of magnitude
/// at a ratio that swings with the graph's density — so bytes and pixels are
/// not even monotonically related, and no byte limit can be tuned into
/// correctness. Measured: a focused `--depth 2` graph is a 3.09 MiB PNG that
/// passed an 8 MiB byte budget comfortably and still drew nothing, because
/// decoded it is 376 MB; a `--depth 1` graph in the same corpus decodes to
/// 71 MB and draws.
///
/// This is a TRANSMISSION bound and nothing else. An earlier revision also put
/// it below graphs it judged too dense to read; that judgement was not this
/// guard's to make — `--depth` and a focus id are the user's, and a graph they
/// choose to scroll is a graph they get. The only question here is whether the
/// terminal will take it.
///
/// SECONDARY to [`MAX_IMAGE_DIMENSION`], which is what actually refuses the
/// graphs seen in practice: a raster inside the per-side cap is at most
/// 100 Mpx, so this only bites the square corner case the cap alone would let
/// through. Both are backstops — [`fit_box`] scales every drawing inside both
/// by construction — against a raster that arrives oversized anyway: a `dot`
/// that ignores `-Gsize`, a future caller that does not fit.
#[expect(
    clippy::integer_division,
    reason = "a whole number of pixels is the only meaningful budget, and truncation \
              rounds it DOWN — the safe direction for a bound"
)]
const MAX_IMAGE_PIXELS: u64 = TERMINAL_IMAGE_QUOTA_BYTES / BYTES_PER_PIXEL;

/// The largest raster either terminal accepts on a SINGLE side.
///
/// Measured, not inferred: `q=0` probes of stepped rasters in both terminals
/// (SL-245 human acceptance). Same area, different shapes, identical verdicts:
/// 8000 x 4000 stores; 2000 x 16000, 32000 x 1000 and 1000 x 32000 do not. So
/// the bound is per-side, and area does not predict it. The tallest accepted
/// were 6298 x 7621 (ghostty) and 1760 x 9090 (kitty); the shortest refused
/// were 6298 x 10161 and 1760 x 13636, bracketing the cap in `[9090, 10161)`
/// — kitty's documented `MAX_IMAGE_DIMENSION`, which ghostty mirrors.
///
/// Stricter than cairo's own 32767-px bitmap limit on both axes, so [`fit_box`]
/// staying under this keeps graphviz's downscale warning ([`MSG_DOT_NOTES`])
/// meaning what it says.
///
/// **kitty answers `ENOMEM:PNG image is too large`; ghostty does not answer at
/// all** — not even with replies enabled. There is no runtime signal to react
/// to in either case, which is why this is a stop before the write rather than
/// an error path after it.
const MAX_IMAGE_DIMENSION: u32 = 10_000;

/// Whole megapixels, rounded UP — so a refused raster never reads as smaller
/// than the limit it exceeded.
fn megapixels(pixels: u64) -> u64 {
    pixels.div_ceil(PIXELS_PER_MEGAPIXEL)
}

// ── Message placeholders ───────────────────────────────────────────────────

/// The requested `--format` value.
const PLACEHOLDER_FORMAT: &str = "{format}";

/// An underlying `io::Error`, rendered with `Display`.
const PLACEHOLDER_ERROR: &str = "{error}";

/// The probe budget, from [`support_probe_timeout_text`].
const PLACEHOLDER_TIMEOUT: &str = "{timeout}";

/// The raster's own dimensions, `<width>x<height>`.
const PLACEHOLDER_RASTER: &str = "{raster}";

/// [`MAX_IMAGE_DIMENSION`], the per-side cap.
const PLACEHOLDER_SIDE: &str = "{side}";

/// [`MAX_IMAGE_PIXELS`] in whole megapixels.
const PLACEHOLDER_LIMIT: &str = "{limit}";

/// How `dot` terminated — [`DOT_STATUS_EXIT`] or [`DOT_STATUS_SIGNAL`].
const PLACEHOLDER_STATUS: &str = "{status}";

/// `dot`'s own captured stderr, trimmed.
const PLACEHOLDER_STDERR: &str = "{stderr}";

/// A process exit code.
const PLACEHOLDER_CODE: &str = "{code}";

/// Fill one of the four messages whose only interpolation is an underlying
/// `io::Error`. One seam, so a fifth cannot drift into a different rendering.
fn io_message(template: &str, error: &std::io::Error) -> String {
    template.replace(PLACEHOLDER_ERROR, &error.to_string())
}

// ── Messages (design sec-3's table, verbatim) ──────────────────────────────

const MSG_FORMAT_NOT_DOT: &str =
    "--render needs --format dot, got '{format}'; drop -X or the --format";

const MSG_NOT_TERMINAL: &str = "--render needs stdout to be a terminal; drop -X to emit DOT";

const MSG_NOT_CONTROLLING_TERMINAL: &str = "--render needs stdout to be the terminal you are running in; it is another terminal, \
     or there is none; drop -X to emit DOT";

const MSG_NO_PIXEL_SIZE: &str = "--render needs the terminal to report its size in pixels, \
     and it did not; drop -X to emit DOT";

const MSG_UNSUPPORTED: &str = "--render needs a terminal that supports the kitty graphics protocol (kitty, ghostty); \
     this one does not, and under tmux or screen it never will; drop -X to emit DOT";

const MSG_UNCONFIRMED: &str = "--render could not confirm kitty graphics support: \
     the terminal did not answer within {timeout}; drop -X to emit DOT";

/// `open_render_terminal` failing outright (`tcgetwinsize`).
///
/// Design sec-3's table gained a row for this path, and for
/// [`MSG_IMAGE_TOO_LARGE`], at reconcile (RV-369 F-3, F-6). It deliberately
/// does NOT reuse [`MSG_TTY_QUERY_IO`]:
/// that row means "the terminal was opened and the exchange failed", whereas this
/// one means "the terminal could not be inspected at all", and POL-002 facet (3)
/// requires the message to name what was actually missing.
const MSG_TERMINAL_INSPECT: &str =
    "--render could not inspect the terminal: {error}; drop -X to emit DOT";

const MSG_TTY_QUERY_IO: &str = "--render could not query the terminal: {error}";

const MSG_TERMINAL_RESTORE: &str =
    "--render could not restore the terminal's settings: {error}; run 'reset'";

const MSG_DOT_NOT_FOUND: &str =
    "--render needs graphviz: 'dot' was not found on PATH; install graphviz or drop -X";

const MSG_DOT_FAILED: &str = "'dot' failed ({status}): {stderr}";

const MSG_DOT_IO: &str = "could not run 'dot': {error}";

const MSG_NOT_A_PNG: &str = "'dot -Tpng' produced output that is not a PNG";

/// RV-369 F-6. Names the size, the limit, and BOTH flags that narrow a graph —
/// the refusal has to be actionable, because it is the answer to the most
/// obvious thing a user types (`doctrine graph -X`, no focus).
const MSG_IMAGE_TOO_LARGE: &str = "--render needs a smaller graph: 'dot' produced a {raster} image and the terminal takes \
     at most {side} px on a side and {limit} Mpx in total; narrow it with a focus id or --depth, \
     or drop -X to emit DOT";

/// RV-369 F-8. `dot` exited zero and still had something to say — most usefully
/// that it downscaled the drawing to fit cairo's bitmap limit, which means the
/// image about to be sent is not the image that was asked for.
/// No tool name of our own: graphviz already prefixes its stderr with `dot: `,
/// so naming it here produced `'dot' reported: dot: graph is too large …`
/// (observed at the VH-2 re-run). Let the tool speak for itself.
const MSG_DOT_NOTES: &str = "warning: {stderr}";

/// `dot` exited on its own, with a code.
const DOT_STATUS_EXIT: &str = "exit {code}";

/// `dot` was killed by a signal, so there is no exit code to report
/// (`RasterOutcome::CommandFailed { status: None, .. }`). Design sec-3's row is
/// written for the `Some(code)` case only; this is the other half of it, kept in
/// the constant rather than spelled inline at the call site.
const DOT_STATUS_SIGNAL: &str = "killed by signal";

// ── Refusals ───────────────────────────────────────────────────────────────

/// Why `--render` declined to draw. Exactly the six EX-1 arms: each one is a
/// *guard stop*, a condition the user can act on. Probe and spawn I/O failures
/// are NOT arms here — they are plain `anyhow` errors built from their own
/// named constants (D1), because they describe a broken exchange rather than an
/// unmet precondition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RenderRefusal {
    FormatNotDot {
        format: String,
    },
    NotTerminal,
    NotControllingTerminal,
    Unsupported,
    Unconfirmed,
    NoPixelSize,
    /// The raster exceeds [`MAX_IMAGE_PIXELS`] (RV-369 F-6). A guard stop like
    /// the others — the user acts on it by narrowing the graph — but the only
    /// one that cannot be decided until after `dot` has run.
    ImageTooLarge {
        width: u32,
        height: u32,
    },
}

impl std::fmt::Display for RenderRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FormatNotDot { format } => {
                f.write_str(&MSG_FORMAT_NOT_DOT.replace(PLACEHOLDER_FORMAT, format))
            }
            Self::NotTerminal => f.write_str(MSG_NOT_TERMINAL),
            Self::NotControllingTerminal => f.write_str(MSG_NOT_CONTROLLING_TERMINAL),
            Self::Unsupported => f.write_str(MSG_UNSUPPORTED),
            Self::Unconfirmed => f.write_str(
                &MSG_UNCONFIRMED.replace(PLACEHOLDER_TIMEOUT, &support_probe_timeout_text()),
            ),
            Self::NoPixelSize => f.write_str(MSG_NO_PIXEL_SIZE),
            Self::ImageTooLarge { width, height } => f.write_str(
                &MSG_IMAGE_TOO_LARGE
                    .replace(PLACEHOLDER_RASTER, &format!("{width}x{height}"))
                    .replace(PLACEHOLDER_SIDE, &MAX_IMAGE_DIMENSION.to_string())
                    .replace(PLACEHOLDER_LIMIT, &megapixels(MAX_IMAGE_PIXELS).to_string()),
            ),
        }
    }
}

/// So the shells can propagate a refusal with `?` into `anyhow::Result`.
impl std::error::Error for RenderRefusal {}

// ── The pure checks (EX-2's order, testable without a terminal) ────────────

/// Stop 1 — the requested output format. Cheapest of all: no syscall.
fn check_format(format: &str, is_dot: bool) -> Result<(), RenderRefusal> {
    if is_dot {
        return Ok(());
    }
    Err(RenderRefusal::FormatNotDot {
        format: format.to_owned(),
    })
}

/// Stop 3 — the cell size the window implies, or the refusal for a terminal
/// that did not report usable pixels. `kitty::cell_geometry`'s `None` covers
/// BOTH a zero reported field and a zero quotient; either way the placement
/// would be a guess, and `DEC-256` refuses rather than guessing.
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "mirrors kitty::cell_geometry, whose `&WindowGeometry` design sec-2 (SL-245) \
              pins — this is the thin wrapper that calls it, and diverging here would put \
              a `&`/value mismatch between two functions that are read as one step. Same \
              grounds as the two expects on kitty.rs's own signatures; pending human \
              ratification at audit alongside them"
)]
fn check_geometry(window: &WindowGeometry) -> Result<CellGeometry, RenderRefusal> {
    kitty::cell_geometry(window).ok_or(RenderRefusal::NoPixelSize)
}

/// Stops 2 and 3 over an injected probe result: which terminal (if any) was
/// found, and the geometry it reports.
///
/// The composition is the point — it is what pins stop 2 AHEAD of stop 3, so a
/// non-terminal is refused as such and never as [`RenderRefusal::NoPixelSize`]
/// (VT-1). Pure, so the order is testable with no terminal in sight.
fn check_terminal(target: &RenderTarget) -> Result<(&RenderTerminal, CellGeometry), RenderRefusal> {
    match target {
        RenderTarget::NotTerminal => Err(RenderRefusal::NotTerminal),
        RenderTarget::NotControllingTerminal => Err(RenderRefusal::NotControllingTerminal),
        RenderTarget::Terminal(terminal) => {
            let cell = check_geometry(&terminal.window)?;
            Ok((terminal, cell))
        }
    }
}

/// Stop 4 — the verdict of the kitty support probe, over the exchange's own
/// result.
///
/// `Ok(None)` is the deadline, NOT an error: the terminal simply did not answer,
/// which is [`RenderRefusal::Unconfirmed`]. `Incomplete` cannot arrive through
/// [`prepare`] (its `complete` predicate is exactly "not `Incomplete`"), but the
/// match stays total and conservative rather than panicking — `clippy::unreachable`
/// is denied, and an unconfirmed answer is the honest reading of a partial one.
fn check_support(reply: Result<Option<Vec<u8>>, QueryError>) -> anyhow::Result<()> {
    match reply {
        Err(QueryError::Io(error)) => Err(anyhow::anyhow!(io_message(MSG_TTY_QUERY_IO, &error))),
        Err(QueryError::Restore(error)) => {
            Err(anyhow::anyhow!(io_message(MSG_TERMINAL_RESTORE, &error)))
        }
        Ok(None) => Err(RenderRefusal::Unconfirmed.into()),
        Ok(Some(bytes)) => match kitty::classify_support_reply(&bytes) {
            SupportReply::Supported => Ok(()),
            SupportReply::Unsupported => Err(RenderRefusal::Unsupported.into()),
            SupportReply::Incomplete => Err(RenderRefusal::Unconfirmed.into()),
        },
    }
}

/// Stop 5 — the rasterised image against [`MAX_IMAGE_PIXELS`] (RV-369 F-6).
///
/// Last of the stops, and the only one that cannot run in [`prepare`]: nothing
/// knows the size until `dot` has produced it. Pure, so the rule is testable
/// without spawning graphviz.
fn check_image_size(size: kitty::PngSize) -> Result<(), RenderRefusal> {
    let within_sides = size.width <= MAX_IMAGE_DIMENSION && size.height <= MAX_IMAGE_DIMENSION;
    let within_total = u64::from(size.width) * u64::from(size.height) <= MAX_IMAGE_PIXELS;
    if within_sides && within_total {
        return Ok(());
    }
    Err(RenderRefusal::ImageTooLarge {
        width: size.width,
        height: size.height,
    })
}

/// Points per inch — the fixed conversion in `dpi = pt * 72 / px`.
const POINTS_PER_INCH: u32 = 72;

/// Graphviz's default node/edge label size, in points. The emitter does not
/// set `fontsize`, so this is what every label is drawn at.
const LABEL_POINT_SIZE: u32 = 14;

/// The resolution a graph is drawn at: the one that renders a
/// [`LABEL_POINT_SIZE`] label about as tall as a terminal row, so graph text
/// reads at the size of the text around it (ISS-459).
///
/// Apparent size is this, and only this. The fit box below never enlarges a
/// drawing — it only shrinks one that would overflow the window.
#[expect(
    clippy::integer_division,
    reason = "truncation costs at most one dpi, which moves a label by well under a \
              pixel — the test asserts the label lands within 1px of a row's height"
)]
fn raster_dpi(cell: CellGeometry) -> u32 {
    (POINTS_PER_INCH * u32::from(cell.cell_height)) / LABEL_POINT_SIZE
}

/// The box `dot` must fit the drawing INSIDE: as wide as the placement will
/// ever be, and as tall as the remaining budget allows.
///
/// Width is `DEC-256`'s `max_columns` in pixels — the widest rectangle
/// [`kitty::place`] will ever ask for — so a raster that binds on this arrives
/// at the size it will be displayed at, rather than having its surplus pixels
/// decoded, counted against the terminal's quota, and thrown away.
///
/// Height is whatever [`MAX_IMAGE_PIXELS`] has left once that width is spent,
/// which makes the box's AREA the budget: any drawing fitted into it is within
/// budget by construction, whatever its aspect ratio.
///
/// BOTH sides are then clamped to [`MAX_IMAGE_DIMENSION`], which is the bound
/// that actually bites. A wide window makes the area budget yield a TALLER box
/// — width is its divisor — so on a 6270-px-wide high-DPI window the area
/// rule alone asked for 12300 px of height and the terminal silently dropped
/// it.
/// Clamping is safe for the area invariant in a way that widening would not
/// be: a `min` only ever shrinks the box.
///
/// A tall graph therefore binds on HEIGHT and comes out narrower than the
/// window. That is the budget binding, and the alternative is a blank screen.
#[expect(
    clippy::integer_division,
    reason = "truncation spends the remainder on nothing, keeping the box's area at or \
              BELOW the budget — which is what makes `anything that fits is in budget` hold"
)]
fn fit_box(cell: CellGeometry) -> crate::graphviz::FitBox {
    let width_px = u32::from(cell.columns.saturating_sub(1).max(1)) * u32::from(cell.cell_width);
    let height_px = MAX_IMAGE_PIXELS / u64::from(width_px.max(1));
    crate::graphviz::FitBox {
        width_px: width_px.min(MAX_IMAGE_DIMENSION),
        height_px: u32::try_from(height_px)
            .unwrap_or(MAX_IMAGE_DIMENSION)
            .min(MAX_IMAGE_DIMENSION),
        dpi: raster_dpi(cell),
    }
}

/// The support probe's "stop reading" predicate: anything that is not
/// [`SupportReply::Incomplete`] is a verdict.
fn support_reply_complete(bytes: &[u8]) -> bool {
    kitty::classify_support_reply(bytes) != SupportReply::Incomplete
}

// ── prepare — the guard shell (EX-1, EX-2) ─────────────────────────────────

/// May `--render` draw, and at what cell size? The thin impure shell over the
/// checks above: it supplies the two probe results (`open_render_terminal` and
/// the support query) and threads them through the pure stops, in EX-2's order,
/// stopping at the first refusal.
///
/// Called BEFORE the project-root lookup in `run_graph`, so a refusal is
/// attributed to `-X` rather than masked by "no project root" (EX-4, VT-3/VT-4).
/// Writes nothing, anywhere.
pub(crate) fn prepare(format: &str, is_dot: bool) -> anyhow::Result<CellGeometry> {
    check_format(format, is_dot)?;
    let target = crate::tty::open_render_terminal()
        .map_err(|error| anyhow::anyhow!(io_message(MSG_TERMINAL_INSPECT, &error)))?;
    let (terminal, cell) = check_terminal(&target)?;
    check_support(terminal.query(
        kitty::SUPPORT_QUERY,
        support_reply_complete,
        SUPPORT_PROBE_TIMEOUT,
    ))?;
    Ok(cell)
}

// ── render_dot — the pipeline (EX-3) ───────────────────────────────────────

/// How `dot` terminated, as the message spells it.
fn dot_status_text(status: Option<i32>) -> String {
    match status {
        Some(code) => DOT_STATUS_EXIT.replace(PLACEHOLDER_CODE, &code.to_string()),
        None => DOT_STATUS_SIGNAL.to_owned(),
    }
}

/// DOT source → the exact bytes to write: rasterise, read the PNG's size, fix
/// the placement, encode.
///
/// **One buffer.** `kitty::encode_png` already appends `placement.rows`
/// newlines, so its return value IS the whole write — the image escapes and the
/// cursor advance together. Appending newlines again here would double-advance
/// the cursor on a real terminal. Every failure returns before any byte exists
/// for the caller to write (EX-3, VA-1).
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "design sec-2 / EX-1 (SL-245) pin this signature as `&kitty::CellGeometry`, \
              and the value is handed straight to kitty::place, which takes `&CellGeometry` \
              on the same pinned grounds. Pending human ratification at audit alongside \
              kitty.rs's two expects"
)]
pub(crate) fn render_dot(dot: &str, cell: &CellGeometry) -> anyhow::Result<Vec<u8>> {
    match crate::graphviz::rasterise_png(
        dot.as_bytes(),
        OsStr::new(crate::graphviz::DOT_PROGRAM),
        fit_box(*cell),
    ) {
        RasterOutcome::ToolUnavailable => Err(anyhow::anyhow!(MSG_DOT_NOT_FOUND)),
        RasterOutcome::CommandFailed { status, stderr } => Err(anyhow::anyhow!(
            MSG_DOT_FAILED
                .replace(PLACEHOLDER_STATUS, &dot_status_text(status))
                .replace(PLACEHOLDER_STDERR, stderr.trim())
        )),
        RasterOutcome::Io(error) => Err(anyhow::anyhow!(io_message(MSG_DOT_IO, &error))),
        RasterOutcome::Png { png, notes } => {
            // F-8: `dot` succeeded and still had something to say. Disclosed
            // before the size check, so a downscale warning reaches the user
            // whether or not the refusal below fires.
            if !notes.is_empty() {
                warn(&MSG_DOT_NOTES.replace(PLACEHOLDER_STDERR, &notes));
            }
            let size = kitty::png_size(&png).ok_or_else(|| anyhow::anyhow!(MSG_NOT_A_PNG))?;
            check_image_size(size)?;
            Ok(kitty::encode_png(&png, kitty::place(size, cell)))
        }
    }
}

/// The one place this module writes anything, and it writes to STDERR.
///
/// The module's invariant is that no byte it produces reaches STDOUT on a
/// refusal path — a diagnostic on stderr does not touch that, and the
/// alternative (threading a note back through `render_dot`'s return type into
/// `run_graph`) would buy nothing but a wider signature. Shape follows
/// `state::warn_capture`; a failed warning is not worth failing a render over.
fn warn(message: &str) {
    use std::io::Write as _;
    let _ignored = writeln!(std::io::stderr(), "{message}");
}

// ── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test code")]
mod tests {
    use super::*;

    /// A window a terminal that reports pixels would produce: 80×24 cells of
    /// 10×20 pixels each.
    fn healthy_window() -> WindowGeometry {
        WindowGeometry {
            columns: 80,
            rows: 24,
            pixel_width: 800,
            pixel_height: 480,
        }
    }

    // ── VT-1: the check order and the arms ─────────────────────────────────

    /// Stop 1 beats every later stop — and it beats them through the REAL
    /// `fn prepare`, not only through a pure helper. The test process's stdout
    /// is a pipe, so a `prepare` that checked the terminal first would answer
    /// `NotTerminal`; answering `FormatNotDot` is what convicts the order.
    ///
    /// Deliberately the ONLY `prepare` call in this module's tests: the dot-format
    /// path would talk to whatever terminal a `--nocapture` run happens to own.
    #[test]
    fn prepare_refuses_a_non_dot_format_before_it_looks_for_a_terminal() {
        let error = prepare("json", false).unwrap_err();
        let refusal = error.downcast_ref::<RenderRefusal>().expect("a refusal");
        assert_eq!(
            *refusal,
            RenderRefusal::FormatNotDot {
                format: "json".to_owned()
            },
            "the format stop must precede the terminal probe"
        );
    }

    #[test]
    fn a_dot_format_clears_the_format_stop() {
        assert_eq!(check_format("dot", true), Ok(()));
    }

    #[test]
    fn a_non_dot_format_names_the_format_it_got() {
        assert_eq!(
            check_format("json", false),
            Err(RenderRefusal::FormatNotDot {
                format: "json".to_owned()
            })
        );
    }

    /// Stop 2 beats stop 3: a missing terminal is refused as a missing terminal,
    /// never as `RenderRefusal::NoPixelSize`. (The `Terminal` arm cannot be
    /// synthesised here — `RenderTerminal.tty` is a private `File` — so the
    /// geometry stop is proved directly, below.)
    #[test]
    fn not_a_terminal_is_refused_before_pixel_size() {
        assert_eq!(
            check_terminal(&RenderTarget::NotTerminal).err(),
            Some(RenderRefusal::NotTerminal)
        );
    }

    #[test]
    fn another_terminal_is_refused_before_pixel_size() {
        assert_eq!(
            check_terminal(&RenderTarget::NotControllingTerminal).err(),
            Some(RenderRefusal::NotControllingTerminal)
        );
    }

    #[test]
    fn a_terminal_reporting_no_pixels_is_refused() {
        let window = WindowGeometry {
            pixel_width: 0,
            pixel_height: 0,
            ..healthy_window()
        };
        assert_eq!(
            check_geometry(&window).err(),
            Some(RenderRefusal::NoPixelSize)
        );
    }

    /// Fewer pixels than cells is a ZERO QUOTIENT, not a zero field — and it is
    /// refused just the same, because a zero cell size cannot place an image.
    #[test]
    fn a_degenerate_cell_size_is_refused_as_no_pixel_size() {
        let window = WindowGeometry {
            pixel_width: 40,
            ..healthy_window()
        };
        assert_eq!(
            check_geometry(&window).err(),
            Some(RenderRefusal::NoPixelSize)
        );
    }

    #[test]
    fn a_healthy_window_yields_its_cell_geometry() {
        let cell = check_geometry(&healthy_window()).ok().expect("geometry");
        assert_eq!(cell.columns, 80);
        assert_eq!(cell.cell_width, 10);
        assert_eq!(cell.cell_height, 20);
    }

    /// A graphics reply carrying the query's own id is the one answer that lets
    /// the render proceed.
    #[test]
    fn a_graphics_reply_confirms_support() {
        let reply = b"\x1b_Gi=31;OK\x1b\\".to_vec();
        assert!(check_support(Ok(Some(reply))).is_ok());
    }

    #[test]
    fn a_da1_reply_first_refuses_as_unsupported() {
        let reply = b"\x1b[?62;c".to_vec();
        let error = check_support(Ok(Some(reply))).unwrap_err();
        assert_eq!(
            error.downcast_ref::<RenderRefusal>(),
            Some(&RenderRefusal::Unsupported)
        );
    }

    /// The deadline is `Ok(None)` — a non-answer, not a failure.
    #[test]
    fn a_silent_terminal_is_unconfirmed_not_an_error() {
        let error = check_support(Ok(None)).unwrap_err();
        assert_eq!(
            error.downcast_ref::<RenderRefusal>(),
            Some(&RenderRefusal::Unconfirmed)
        );
    }

    /// Defensive: `prepare`'s `complete` predicate precludes a partial reply, so
    /// this arm exists to keep the match total. It reads as unconfirmed.
    #[test]
    fn a_partial_reply_is_unconfirmed() {
        let error = check_support(Ok(Some(b"\x1b_Gi=31".to_vec()))).unwrap_err();
        assert_eq!(
            error.downcast_ref::<RenderRefusal>(),
            Some(&RenderRefusal::Unconfirmed)
        );
    }

    /// `QueryError`'s two arms are ranked, not interchangeable — a failed
    /// restore says the user's shell is left wrong and tells them how to fix it.
    #[test]
    fn a_failed_exchange_reports_the_query_io_message() {
        let error =
            check_support(Err(QueryError::Io(std::io::Error::other("no read")))).unwrap_err();
        assert!(
            error.downcast_ref::<RenderRefusal>().is_none(),
            "a broken exchange is not one of the six guard stops"
        );
        let message = error.to_string();
        assert!(
            message.contains("could not query the terminal"),
            "{message}"
        );
        assert!(message.contains("no read"), "{message}");
    }

    #[test]
    fn a_failed_restore_outranks_and_tells_the_user_to_reset() {
        let error = check_support(Err(QueryError::Restore(std::io::Error::other("stuck"))))
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("could not restore the terminal's settings"),
            "{error}"
        );
        assert!(error.contains("stuck"), "{error}");
        assert!(error.contains("run 'reset'"), "{error}");
    }

    /// The support predicate is what makes `Incomplete` unreachable in `prepare`.
    #[test]
    fn the_probe_keeps_reading_until_a_verdict_arrives() {
        assert!(!support_reply_complete(b"\x1b_Gi=31"));
        assert!(support_reply_complete(b"\x1b_Gi=31;OK\x1b\\"));
        assert!(support_reply_complete(b"\x1b[?62;c"));
    }

    // ── VT-2: every message names the flag and its remedy ──────────────────

    /// Substring, not exact text: the wording is design sec-3's to revise, but
    /// a message that does not say `--render` and does not say what to do
    /// instead is a bug whatever the wording.
    #[test]
    fn every_refusal_names_the_flag_and_a_remedy() {
        let refusals = [
            RenderRefusal::FormatNotDot {
                format: "json".to_owned(),
            },
            RenderRefusal::NotTerminal,
            RenderRefusal::NotControllingTerminal,
            RenderRefusal::Unsupported,
            RenderRefusal::Unconfirmed,
            RenderRefusal::NoPixelSize,
            RenderRefusal::ImageTooLarge {
                width: MAX_IMAGE_DIMENSION + 1,
                height: MAX_IMAGE_DIMENSION + 1,
            },
        ];
        for refusal in &refusals {
            let message = refusal.to_string();
            assert!(message.contains("--render"), "no flag in: {message}");
            assert!(message.contains("drop -X"), "no remedy in: {message}");
        }
    }

    // ── RV-369 F-6: the image budget ───────────────────────────────────────

    /// A raster of `width` x `height`, as `kitty::png_size` reports one.
    fn raster(width: u32, height: u32) -> kitty::PngSize {
        kitty::PngSize { width, height }
    }

    /// The bound is on PIXELS, and it is inclusive at the limit. The corpus
    /// case that convicted this (VH-2) was a placement of 199 x 237 cells — a
    /// cell rectangle nothing would object to, which is why the cell bound
    /// could not catch it.
    #[test]
    fn the_image_budget_admits_the_limit_and_refuses_past_it() {
        assert!(
            check_image_size(raster(0, 0)).is_ok(),
            "an empty render is not too big"
        );

        let square = u32::try_from(MAX_IMAGE_PIXELS.isqrt()).expect("a plausible budget");
        assert!(
            check_image_size(raster(square, square)).is_ok(),
            "the limit itself is admitted"
        );
        assert!(
            check_image_size(raster(square + 1, square + 1)).is_err(),
            "one pixel past the limit is refused"
        );
    }

    /// The unit is the point, not the number: the measured `--depth 2` case is
    /// a 3.09 MiB PNG that a byte budget of 8 MiB waved through and ghostty
    /// then declined, because decoded it is 94 Mpx. A byte bound cannot order
    /// these two correctly; a pixel bound does.
    #[test]
    fn the_budget_refuses_the_raster_a_byte_budget_admitted() {
        // `--depth 1`, which VH-1 confirmed draws.
        assert!(check_image_size(raster(3125, 5692)).is_ok());
        // `--depth 2`, which drew nothing.
        assert!(check_image_size(raster(7280, 12917)).is_err());
    }

    /// The refusal has to be actionable: it is the answer to `doctrine graph -X`
    /// with no focus, which is the most obvious thing to type.
    #[test]
    fn the_size_refusal_names_the_size_the_limit_and_both_ways_to_narrow() {
        let message = RenderRefusal::ImageTooLarge {
            width: 2000,
            height: 16000,
        }
        .to_string();
        assert!(message.contains("2000x16000"), "{message}");
        assert!(
            message.contains(&MAX_IMAGE_DIMENSION.to_string()),
            "{message}"
        );
        assert!(
            message.contains(&format!("{} Mpx", megapixels(MAX_IMAGE_PIXELS))),
            "{message}"
        );
        assert!(message.contains("focus"), "{message}");
        assert!(message.contains("--depth"), "{message}");
    }

    /// Rounded UP, so a refused raster never reads as smaller than the limit it
    /// just exceeded — one pixel over must not print as the limit.
    #[test]
    fn a_size_just_over_the_limit_does_not_print_as_the_limit() {
        let limit = megapixels(MAX_IMAGE_PIXELS);
        assert_eq!(megapixels(MAX_IMAGE_PIXELS + 1), limit + 1);
        assert_eq!(megapixels(0), 0);
    }

    /// The budget is a whole number of megapixels, so the refusal never names a
    /// limit the guard does not actually enforce.
    #[test]
    fn the_budget_is_a_whole_number_of_megapixels() {
        assert_eq!(
            megapixels(MAX_IMAGE_PIXELS) * PIXELS_PER_MEGAPIXEL,
            MAX_IMAGE_PIXELS
        );
    }

    // ── The fit box ────────────────────────────────────────────────────────

    /// Apparent size is the dpi's job: a label is drawn at about the height of
    /// a terminal row, whatever the terminal's cell geometry (ISS-459).
    #[test]
    fn raster_dpi_draws_a_label_about_one_row_tall() {
        for cell_height in [14_u16, 20, 24, 40, 64] {
            let cell = CellGeometry {
                columns: 120,
                cell_width: cell_height / 2,
                cell_height,
            };
            let label_px = (LABEL_POINT_SIZE * raster_dpi(cell)) / POINTS_PER_INCH;
            let delta = i64::from(label_px).abs_diff(i64::from(cell_height));
            assert!(
                delta <= 1,
                "cell height {cell_height}: label drawn at {label_px}px"
            );
        }
    }

    /// The box bounds a drawing; it does not set its size. Two windows of the
    /// same cell geometry but different widths draw text identically — only
    /// the point at which a large graph starts shrinking moves.
    #[test]
    fn the_fit_box_width_does_not_change_apparent_size() {
        let narrow = fit_box(CellGeometry {
            columns: 40,
            cell_width: 10,
            cell_height: 20,
        });
        let wide = fit_box(CellGeometry {
            columns: 400,
            cell_width: 10,
            cell_height: 20,
        });
        assert_eq!(narrow.dpi, wide.dpi);
        assert!(wide.width_px > narrow.width_px);
    }

    /// The width `dot` is given is the width `kitty::place` will ask for, so
    /// the raster is neither upscaled by the terminal nor decoded and thrown
    /// away. `place` never reaches the right edge, hence `columns - 1`.
    #[test]
    fn the_fit_box_is_as_wide_as_the_widest_placement() {
        let cell = check_geometry(&healthy_window()).ok().expect("geometry");
        let fit = fit_box(cell);
        let placement = kitty::place(raster(fit.width_px, fit.height_px), &cell);
        assert_eq!(
            u32::from(placement.columns) * u32::from(cell.cell_width),
            fit.width_px,
            "a raster fitted to the box places at exactly its own width"
        );
    }

    /// The box's AREA is the budget, so anything scaled into it passes
    /// [`check_image_size`] whatever its aspect ratio — which is what demotes
    /// that check from the working limit to a backstop.
    #[test]
    fn anything_that_fits_the_box_is_within_budget() {
        // 286 x 22 is the measured HiDPI ghostty window whose fitted box the
        // area-only rule sent 12300 px tall, and which drew nothing.
        for columns in [1_u16, 2, 80, 286, 400, u16::MAX] {
            for cell_width in [1_u16, 8, 20, 22] {
                let cell = CellGeometry {
                    columns,
                    cell_width,
                    cell_height: 20,
                };
                let fit = fit_box(cell);
                assert!(
                    check_image_size(raster(fit.width_px, fit.height_px)).is_ok(),
                    "{columns} columns x {cell_width} px: {fit:?} exceeds the budget"
                );
                assert!(
                    fit.width_px <= MAX_IMAGE_DIMENSION && fit.height_px <= MAX_IMAGE_DIMENSION,
                    "{fit:?} exceeds the per-side cap the terminals enforce"
                );
            }
        }
    }

    #[test]
    fn the_format_refusal_quotes_the_format_that_was_asked_for() {
        let message = RenderRefusal::FormatNotDot {
            format: "json".to_owned(),
        }
        .to_string();
        assert!(message.contains("'json'"), "{message}");
        assert!(message.contains("--format dot"), "{message}");
    }

    /// No placeholder may survive into a rendered message — an unfilled `{…}`
    /// is the failure mode this constant-plus-placeholder scheme can have.
    #[test]
    fn no_rendered_message_leaks_a_placeholder() {
        let messages = [
            RenderRefusal::FormatNotDot {
                format: "json".to_owned(),
            }
            .to_string(),
            RenderRefusal::Unconfirmed.to_string(),
            MSG_TTY_QUERY_IO.replace(PLACEHOLDER_ERROR, "boom"),
            MSG_TERMINAL_RESTORE.replace(PLACEHOLDER_ERROR, "boom"),
            MSG_TERMINAL_INSPECT.replace(PLACEHOLDER_ERROR, "boom"),
            MSG_DOT_IO.replace(PLACEHOLDER_ERROR, "boom"),
            MSG_DOT_FAILED
                .replace(PLACEHOLDER_STATUS, &dot_status_text(Some(1)))
                .replace(PLACEHOLDER_STDERR, "syntax error"),
            MSG_DOT_NOT_FOUND.to_owned(),
            MSG_NOT_A_PNG.to_owned(),
        ];
        for message in &messages {
            assert!(!message.contains('{'), "unfilled placeholder in: {message}");
        }
    }

    /// The probe budget is named ONCE: the message reads it from the constant,
    /// so the two cannot drift.
    #[test]
    fn the_unconfirmed_message_quotes_the_probe_budget() {
        let message = RenderRefusal::Unconfirmed.to_string();
        assert!(
            message.contains(&support_probe_timeout_text()),
            "{message} does not quote {SUPPORT_PROBE_TIMEOUT:?}"
        );
    }

    /// Both halves of the `dot` failure row: an exit code, and the signal case
    /// design sec-3 does not spell.
    #[test]
    fn a_dot_failure_reports_either_an_exit_code_or_a_signal() {
        assert_eq!(dot_status_text(Some(1)), "exit 1");
        assert_eq!(dot_status_text(None), "killed by signal");
        let killed = MSG_DOT_FAILED
            .replace(PLACEHOLDER_STATUS, &dot_status_text(None))
            .replace(PLACEHOLDER_STDERR, "");
        assert!(
            killed.contains("'dot' failed (killed by signal)"),
            "{killed}"
        );
    }

    /// `render_dot`'s non-PNG stop, and the `-X`-less remedy on the missing-tool
    /// stop, are the two messages the pipeline owns.
    #[test]
    fn the_pipeline_messages_name_their_cause() {
        assert!(MSG_NOT_A_PNG.contains("not a PNG"));
        assert!(MSG_DOT_NOT_FOUND.contains("graphviz"));
        assert!(MSG_DOT_NOT_FOUND.contains("drop -X"));
    }
}
