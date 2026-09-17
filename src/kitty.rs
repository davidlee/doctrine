// SPDX-License-Identifier: GPL-3.0-only
//! `kitty` — the graphics protocol, pure (ADR-001, SL-245 PHASE-03, sec-4).
//!
//! Bytes in, bytes out: the support query and its reply classifier, the PNG
//! header read, the `DEC-256` placement rule, and the transmit-and-display
//! encoding. No I/O, no clock, no terminal — everything the terminal knows
//! arrives as a plain value, so every rule here is unit-testable without one.
//!
//! The one crate-internal edge is [`crate::tty::WindowGeometry`] (leaf → leaf,
//! EX-11); nothing else in the crate is imported. Every protocol literal is a
//! named constant (STD-001), because the same bytes are written by the encoder
//! and read by the classifier and the two must not drift.

use base64::Engine as _;

use crate::tty::WindowGeometry;

// ── Framing ────────────────────────────────────────────────────────────────

/// Introducer for every kitty graphics escape, request or reply: `ESC _ G`.
const APC_START: &[u8] = b"\x1b_G";

/// String Terminator ending every graphics escape: `ESC \`.
const APC_END: &[u8] = b"\x1b\\";

/// Control Sequence Introducer: `ESC [`.
const CSI_START: &[u8] = b"\x1b[";

/// Parameter and intermediate bytes of a CSI sequence.
const CSI_PARAMETER_BYTES: std::ops::RangeInclusive<u8> = 0x20..=0x3F;

/// A CSI sequence ends at its first final byte.
const CSI_FINAL_BYTES: std::ops::RangeInclusive<u8> = 0x40..=0x7E;

/// The final byte identifying a primary-device-attributes (DA1) reply.
const DA1_FINAL: u8 = b'c';

/// The DA1 request: `ESC [ c`. Every VT-compatible terminal answers it, which
/// is what makes it a usable "the terminal has finished answering" marker.
///
/// Not read by production code: [`SUPPORT_QUERY`] is a single byte-string
/// literal (a `const` cannot be concatenated from slices on stable), so this
/// names the tail it must end with and is read only by the drift guard in this
/// module's tests. The suppression is per-symbol and `cfg_attr(not(test), …)`,
/// because under the test compilation the expectation WOULD be unfulfilled
/// (`mem.pattern.lint.expect-dead-code-at-item-level`). It surfaced when
/// SL-245 PHASE-05 removed this module's `#![allow(dead_code)]` blanket, which
/// had been covering it.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "drift guard for SUPPORT_QUERY's DA1 tail — read by this module's tests only"
    )
)]
const DA1_REQUEST: &[u8] = b"\x1b[c";

/// Separates control keys from each other.
const KEY_SEPARATOR: u8 = b',';

/// Separates the control keys from the payload or status text.
const PAYLOAD_SEPARATOR: u8 = b';';

// ── The support query ──────────────────────────────────────────────────────

/// The image id the support query uses. Single-sourced (STD-001): it is
/// written into [`SUPPORT_QUERY`] and matched by [`classify_support_reply`],
/// and a drift between the two would silently classify every terminal as
/// unsupported.
const SUPPORT_QUERY_ID: &str = "i=31";

/// The documented support probe: a graphics query, then DA1, sent as one write
/// (kitty's *Querying support*; `DEC-259`).
///
/// ```text
/// ESC_G i=31,s=1,v=1,a=q,t=d,f=24 ; AAAA ESC\     ESC[c
/// ```
///
/// | key | value | why |
/// |---|---|---|
/// | `i=31` | image id | [`SUPPORT_QUERY_ID`] — an arbitrary id the reply echoes |
/// | `s=1`, `v=1` | 1 × 1 | the smallest possible image |
/// | `a=q` | query only | never stored, never displayed |
/// | `t=d` | direct (in-band) | the payload is right here |
/// | `f=24` | 24-bit RGB | `AAAA` is base64 for the three zero bytes of one pixel |
///
/// DA1 follows because requests are answered in order: a terminal that speaks
/// the graphics protocol answers the query *before* DA1, so a DA1 reply
/// arriving first is proof the query went unanswered.
pub(crate) const SUPPORT_QUERY: &[u8] = b"\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\\x1b[c";

/// What the accumulated reply bytes say about graphics support so far.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SupportReply {
    /// No complete frame of interest yet — keep reading.
    Incomplete,
    Supported,
    Unsupported,
}

/// Walk COMPLETE frames in arrival order; the first frame of interest decides.
///
/// - a graphics reply carrying [`SUPPORT_QUERY_ID`], whatever its status text
///   (an error status is still a reply, and a reply proves the protocol is
///   spoken) ⇒ [`SupportReply::Supported`];
/// - a DA1 reply first ⇒ [`SupportReply::Unsupported`], even if a graphics
///   reply follows it — that is the tmux case, which answers DA1 itself and
///   does not pass the query through;
/// - graphics replies for other ids, other CSI sequences and stray bytes are
///   SKIPPED, never decisive;
/// - nothing decidable yet ⇒ [`SupportReply::Incomplete`].
///
/// A half-arrived frame stops the walk rather than being guessed at: because
/// the verdict is an ORDER over frames, no later frame may decide ahead of an
/// earlier one that has not finished arriving.
pub(crate) fn classify_support_reply(bytes: &[u8]) -> SupportReply {
    let mut rest = bytes;
    while !rest.is_empty() {
        let step = match head(rest) {
            Head::Graphics {
                carries_id: true, ..
            } => return SupportReply::Supported,
            Head::Csi { final_byte, .. } if final_byte == DA1_FINAL => {
                return SupportReply::Unsupported;
            }
            Head::Graphics { length, .. } | Head::Csi { length, .. } => length,
            Head::Noise => 1,
            Head::Partial => return SupportReply::Incomplete,
        };
        match rest.get(step..) {
            Some(tail) => rest = tail,
            None => return SupportReply::Incomplete,
        }
    }
    SupportReply::Incomplete
}

/// What sits at the head of the buffer.
enum Head {
    /// A complete graphics reply frame, and whether its key section names
    /// [`SUPPORT_QUERY_ID`].
    Graphics { carries_id: bool, length: usize },
    /// A complete CSI frame and its final byte.
    Csi { final_byte: u8, length: usize },
    /// A frame has started but has not terminated.
    Partial,
    /// One byte of anything else.
    Noise,
}

fn head(bytes: &[u8]) -> Head {
    if bytes.starts_with(APC_START) {
        let Some(body) = bytes.get(APC_START.len()..) else {
            return Head::Partial;
        };
        let Some(end) = find(body, APC_END) else {
            return Head::Partial;
        };
        // The id lives in the key section — the part BEFORE the status text —
        // so an error message quoting `i=31` as prose cannot false-positive.
        let keys = body
            .get(..end)
            .unwrap_or_default()
            .split(|byte| *byte == PAYLOAD_SEPARATOR)
            .next()
            .unwrap_or_default();
        Head::Graphics {
            carries_id: keys
                .split(|byte| *byte == KEY_SEPARATOR)
                .any(|field| field == SUPPORT_QUERY_ID.as_bytes()),
            length: APC_START.len() + end + APC_END.len(),
        }
    } else if bytes.starts_with(CSI_START) {
        let parameters = bytes.get(CSI_START.len()..).unwrap_or_default();
        for (offset, byte) in parameters.iter().enumerate() {
            if CSI_FINAL_BYTES.contains(byte) {
                return Head::Csi {
                    final_byte: *byte,
                    length: CSI_START.len() + offset + 1,
                };
            }
            if !CSI_PARAMETER_BYTES.contains(byte) {
                // Not a CSI sequence after all; the introducer is just bytes.
                return Head::Noise;
            }
        }
        Head::Partial
    } else {
        Head::Noise
    }
}

/// Offset of the first occurrence of `needle` in `haystack`.
fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

// ── The PNG header ─────────────────────────────────────────────────────────

/// The eight bytes every PNG starts with.
const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

/// The tag of a PNG's first chunk, which is always the image header.
const IHDR_TAG: &[u8] = b"IHDR";

/// Offset of the first chunk's type tag. Bytes 8..12 are that chunk's length,
/// which is deliberately NOT validated: the conditions below are exhaustive,
/// and an extra check could only reject PNGs doctrine would otherwise render.
const IHDR_TAG_OFFSET: usize = 12;

/// Offset of the big-endian `u32` image width, inside the IHDR chunk.
const IHDR_WIDTH_OFFSET: usize = 16;

/// Offset of the big-endian `u32` image height, inside the IHDR chunk.
const IHDR_HEIGHT_OFFSET: usize = 20;

/// Bytes of fixed PNG header that must be present before any dimension is read.
const PNG_HEADER_LENGTH: usize = 24;

/// A big-endian `u32` field is four bytes wide.
const BE_U32_LENGTH: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PngSize {
    pub(crate) width: u32,
    pub(crate) height: u32,
}

/// The image dimensions from the fixed layout every PNG starts with, or `None`
/// unless at least [`PNG_HEADER_LENGTH`] bytes are present AND the signature
/// AND the `IHDR` tag match. A "couldn't read this" typed outcome, never a
/// swallowed error (STD-003) — the caller refuses rather than forwarding
/// bytes the terminal would drop silently.
pub(crate) fn png_size(png: &[u8]) -> Option<PngSize> {
    let header = png.get(..PNG_HEADER_LENGTH)?;
    if header.get(..PNG_SIGNATURE.len())? != PNG_SIGNATURE {
        return None;
    }
    if header.get(IHDR_TAG_OFFSET..IHDR_TAG_OFFSET + IHDR_TAG.len())? != IHDR_TAG {
        return None;
    }
    Some(PngSize {
        width: big_endian_u32(header, IHDR_WIDTH_OFFSET)?,
        height: big_endian_u32(header, IHDR_HEIGHT_OFFSET)?,
    })
}

fn big_endian_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let field: [u8; BE_U32_LENGTH] = bytes.get(offset..offset + BE_U32_LENGTH)?.try_into().ok()?;
    Some(u32::from_be_bytes(field))
}

// ── Placement (DEC-256) ────────────────────────────────────────────────────

/// Cell size in pixels, with the column count that produced it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CellGeometry {
    pub(crate) columns: u16,
    pub(crate) cell_width: u16,
    pub(crate) cell_height: u16,
}

/// The cell size implied by the window, or `None` if the terminal did not
/// report a field (`0`) or reported fewer pixels than cells on either axis.
/// `DEC-256` refuses rather than guessing a cell size: a wrong one would
/// misplace the image, which is worse than declining to draw it.
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "design sec-2 (SL-245) pins this signature; `&WindowGeometry` and \
              `&CellGeometry` are threaded through terminal_image::prepare and \
              ::render_dot in PHASE-05 — an 8-byte copy does not justify diverging \
              from the locked design"
)]
#[expect(
    clippy::integer_division,
    reason = "DEC-256 specifies INTEGER division for the cell size; the truncation \
              is the rule, not a loss of precision, and a zero quotient is refused \
              two lines below rather than rounded away"
)]
pub(crate) fn cell_geometry(window: &WindowGeometry) -> Option<CellGeometry> {
    if window.columns == 0
        || window.rows == 0
        || window.pixel_width == 0
        || window.pixel_height == 0
    {
        return None;
    }
    let cell_width = window.pixel_width / window.columns;
    let cell_height = window.pixel_height / window.rows;
    if cell_width == 0 || cell_height == 0 {
        return None;
    }
    Some(CellGeometry {
        columns: window.columns,
        cell_width,
        cell_height,
    })
}

/// The cell rectangle an image occupies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Placement {
    pub(crate) columns: u16,
    pub(crate) rows: u16,
}

/// `DEC-256`: fix the rectangle here rather than letting the terminal move the
/// cursor, because the kitty protocol leaves the cursor position undefined if
/// a placement runs off the screen.
///
/// ```text
/// max_columns    = columns - 1                     never reach the right edge
/// native_columns = ceil(width / cell_width)
/// if native_columns <= max_columns:
///   placement = (native_columns, ceil(height / cell_height))
/// else:
///   scaled_height = height * (max_columns * cell_width) / width
///   placement     = (max_columns, ceil(scaled_height / cell_height))
/// rows = max(rows, 1)
/// ```
///
/// All arithmetic is `u64` and every division rounds UP; `max_columns` is
/// floored at 1 (a one-column terminal would otherwise get a zero-width
/// placement) and so is `rows`. The cast back to `u16` saturates. A tall image
/// is NOT bounded vertically — the trailing newlines scroll it like any other
/// output.
///
/// Provisional: `DEC-256` is judged by the human acceptance on PHASE-05.
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "design sec-2 (SL-245) pins this signature — see cell_geometry above"
)]
pub(crate) fn place(png: PngSize, cell: &CellGeometry) -> Placement {
    let width = u64::from(png.width);
    let height = u64::from(png.height);
    let cell_width = u64::from(cell.cell_width);
    let cell_height = u64::from(cell.cell_height);
    let max_columns = u64::from(cell.columns).saturating_sub(1).max(1);

    let native_columns = divide_rounding_up(width, cell_width);
    let (columns, pixel_height) = if native_columns <= max_columns {
        (native_columns, height)
    } else {
        // Clamp the width and carry the aspect ratio into the height.
        let scaled = height.saturating_mul(max_columns.saturating_mul(cell_width));
        (max_columns, divide_rounding_up(scaled, width))
    };

    Placement {
        columns: saturating_u16(columns),
        rows: saturating_u16(divide_rounding_up(pixel_height, cell_height).max(1)),
    }
}

/// Divide rounding UP. A zero divisor is treated as 1 so `place` is total:
/// [`cell_geometry`] already refuses a zero cell size upstream, and a panic
/// here would be a worse answer than a degenerate placement.
fn divide_rounding_up(numerator: u64, divisor: u64) -> u64 {
    numerator.div_ceil(divisor.max(1))
}

fn saturating_u16(value: u64) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}

// ── Encoding ───────────────────────────────────────────────────────────────

/// Transmit AND display in one command — no separate placement command.
const KEY_ACTION_TRANSMIT_DISPLAY: &str = "a=T";

/// Payload format: PNG. graphviz emits PNG and doctrine decodes no pixels.
const KEY_FORMAT_PNG: &str = "f=100";

/// Direct (in-band) transmission — the documented default, written out.
const KEY_TRANSMISSION_DIRECT: &str = "t=d";

/// Quiet. No image id is sent, so no reply is expected; `q=2` also suppresses
/// FAILURE replies, which would otherwise be typed into the user's shell.
const KEY_QUIET: &str = "q=2";

/// Do not move the cursor — doctrine moves it with the trailing newlines
/// instead, which is what makes the scroll deterministic (`DEC-256`).
const KEY_NO_CURSOR_MOVE: &str = "C=1";

/// Placement width, in cells.
const KEY_COLUMNS: &str = "c=";

/// Placement height, in cells.
const KEY_ROWS: &str = "r=";

/// More chunks follow this one.
const KEY_MORE_FOLLOWS: &str = "m=1";

/// The last chunk.
const KEY_LAST_CHUNK: &str = "m=0";

/// Maximum base64 payload bytes per escape. A multiple of 4, so fixed-size
/// slicing of the encoded stream never splits a base64 quantum — that single
/// fact satisfies both the size bound and the alignment rule, with no
/// special-casing.
const MAX_CHUNK: usize = 4096;

/// One newline per placement row, so the cursor lands on the line below the
/// image.
const CURSOR_ADVANCE: u8 = b'\n';

/// An upper bound on one escape's non-payload bytes: the introducer, the
/// longest control set (`a=T,f=100,t=d,q=2,C=1,c=65535,r=65535,m=0`), the
/// payload separator and the terminator. Used only to size the output buffer
/// up front, so it must never UNDER-estimate.
const ESCAPE_OVERHEAD: usize = 64;

/// The escape sequence(s) for one PNG at `placement`, followed by
/// `placement.rows` newlines.
///
/// The first chunk carries the full control set; continuations carry only `m`
/// and `q`. A payload that fits one chunk is a single escape with `m=0`. The
/// output buffer is allocated ONCE, sized from the encoded length plus the
/// per-escape overhead plus the newlines.
pub(crate) fn encode_png(png: &[u8], placement: Placement) -> Vec<u8> {
    let payload = base64::engine::general_purpose::STANDARD.encode(png);
    let encoded = payload.as_bytes();
    // At least one escape, even for an empty payload.
    let count = encoded.len().div_ceil(MAX_CHUNK).max(1);

    let mut out =
        Vec::with_capacity(encoded.len() + count * ESCAPE_OVERHEAD + usize::from(placement.rows));

    let separator = char::from(KEY_SEPARATOR);
    let first_keys = format!(
        "{KEY_ACTION_TRANSMIT_DISPLAY}{separator}{KEY_FORMAT_PNG}{separator}\
         {KEY_TRANSMISSION_DIRECT}{separator}{KEY_QUIET}{separator}{KEY_NO_CURSOR_MOVE}\
         {separator}{KEY_COLUMNS}{columns}{separator}{KEY_ROWS}{rows}",
        columns = placement.columns,
        rows = placement.rows,
    );

    for index in 0..count {
        let start = index * MAX_CHUNK;
        let chunk = encoded
            .get(start..start.saturating_add(MAX_CHUNK).min(encoded.len()))
            .unwrap_or_default();
        let more = if index + 1 == count {
            KEY_LAST_CHUNK
        } else {
            KEY_MORE_FOLLOWS
        };
        out.extend_from_slice(APC_START);
        // The first chunk carries the full control set; continuations carry
        // ONLY m and q. Written straight into the output buffer, so the
        // per-chunk work allocates nothing.
        if index == 0 {
            out.extend_from_slice(first_keys.as_bytes());
            out.push(KEY_SEPARATOR);
            out.extend_from_slice(more.as_bytes());
        } else {
            out.extend_from_slice(more.as_bytes());
            out.push(KEY_SEPARATOR);
            out.extend_from_slice(KEY_QUIET.as_bytes());
        }

        out.push(PAYLOAD_SEPARATOR);
        out.extend_from_slice(chunk);
        out.extend_from_slice(APC_END);
    }

    out.resize(out.len() + usize::from(placement.rows), CURSOR_ADVANCE);
    out
}

#[cfg(test)]
mod tests {

    use super::*;

    /// A graphics reply frame: `ESC _ G <keys> ; <status> ESC \`.
    fn graphics_reply(keys: &str, status: &str) -> Vec<u8> {
        let mut frame = Vec::new();
        frame.extend_from_slice(APC_START);
        frame.extend_from_slice(keys.as_bytes());
        frame.push(PAYLOAD_SEPARATOR);
        frame.extend_from_slice(status.as_bytes());
        frame.extend_from_slice(APC_END);
        frame
    }

    /// The standard primary-device-attributes reply, `CSI ? 62;1;6 c`.
    fn da1_reply() -> Vec<u8> {
        b"\x1b[?62;1;6c".to_vec()
    }

    /// A 24-byte PNG header: signature, IHDR chunk length, tag, width, height.
    fn ihdr(width: u32, height: u32) -> Vec<u8> {
        let mut png = Vec::new();
        png.extend_from_slice(PNG_SIGNATURE);
        png.extend_from_slice(&13u32.to_be_bytes());
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&width.to_be_bytes());
        png.extend_from_slice(&height.to_be_bytes());
        png
    }

    fn window(columns: u16, rows: u16, pixel_width: u16, pixel_height: u16) -> WindowGeometry {
        WindowGeometry {
            columns,
            rows,
            pixel_width,
            pixel_height,
        }
    }

    /// The request is the documented probe: a graphics query for the support
    /// id, then DA1. Asserted against the named parts so the literal and the
    /// constants the classifier reads cannot drift apart (STD-001).
    #[test]
    fn support_query_is_the_graphics_query_for_the_support_id_then_da1() {
        assert!(SUPPORT_QUERY.starts_with(APC_START));
        assert!(SUPPORT_QUERY.ends_with(DA1_REQUEST));
        let frame_end = SUPPORT_QUERY.len() - DA1_REQUEST.len();
        let frame = &SUPPORT_QUERY[..frame_end];
        assert!(
            frame.ends_with(APC_END),
            "the graphics query is one APC frame"
        );
        assert_eq!(
            classify_support_reply(frame),
            SupportReply::Supported,
            "the query carries the very id the classifier looks for"
        );
    }

    /// VT-9: a graphics reply for the support id, arriving before DA1, decides
    /// `Supported`.
    #[test]
    fn a_graphics_reply_before_da1_is_supported() {
        let mut stream = graphics_reply(SUPPORT_QUERY_ID, "OK");
        stream.extend_from_slice(&da1_reply());
        assert_eq!(classify_support_reply(&stream), SupportReply::Supported);
    }

    /// VT-9: ANY reply for the id proves the protocol is spoken — an error
    /// status is still a reply.
    #[test]
    fn a_graphics_reply_with_an_error_status_is_still_supported() {
        let stream = graphics_reply(SUPPORT_QUERY_ID, "EINVAL:bad transmission medium");
        assert_eq!(classify_support_reply(&stream), SupportReply::Supported);
    }

    /// VT-9: arrival order decides. DA1 first refuses even though a graphics
    /// reply follows — that is the tmux case (tmux answers DA1 itself and does
    /// not pass the query through).
    #[test]
    fn da1_first_is_unsupported_even_when_a_graphics_reply_follows() {
        let mut stream = da1_reply();
        stream.extend_from_slice(&graphics_reply(SUPPORT_QUERY_ID, "OK"));
        assert_eq!(classify_support_reply(&stream), SupportReply::Unsupported);
    }

    /// VT-9: DA1 alone — the terminal answered the standard request and not
    /// the graphics one.
    #[test]
    fn da1_alone_is_unsupported() {
        assert_eq!(
            classify_support_reply(&da1_reply()),
            SupportReply::Unsupported
        );
    }

    /// VT-9: a graphics reply for some OTHER id is skipped, not decisive, so
    /// the DA1 reply behind it still decides.
    #[test]
    fn a_graphics_reply_for_another_id_is_skipped() {
        let mut stream = graphics_reply("i=7", "OK");
        stream.extend_from_slice(&da1_reply());
        assert_eq!(classify_support_reply(&stream), SupportReply::Unsupported);
    }

    /// VT-9: stray bytes between frames are skipped, whichever verdict follows.
    #[test]
    fn noise_between_frames_is_skipped() {
        let mut supported = b"stray bytes".to_vec();
        supported.extend_from_slice(&graphics_reply(SUPPORT_QUERY_ID, "OK"));
        assert_eq!(classify_support_reply(&supported), SupportReply::Supported);

        let mut unsupported = b"\x1bstray\x1b".to_vec();
        unsupported.extend_from_slice(&da1_reply());
        assert_eq!(
            classify_support_reply(&unsupported),
            SupportReply::Unsupported
        );
    }

    /// VT-9: nothing to go on yet.
    #[test]
    fn empty_and_partial_input_is_incomplete() {
        assert_eq!(classify_support_reply(b""), SupportReply::Incomplete);
        // A graphics frame that has started but not terminated.
        assert_eq!(
            classify_support_reply(b"\x1b_Gi=31;OK"),
            SupportReply::Incomplete
        );
        // A CSI frame with no final byte yet.
        assert_eq!(
            classify_support_reply(b"\x1b[?62;1;6"),
            SupportReply::Incomplete
        );
        // Only noise.
        assert_eq!(
            classify_support_reply(b"nothing here"),
            SupportReply::Incomplete
        );
    }

    /// VT-9, the sharp one: split at EVERY byte boundary, a stream is
    /// `Incomplete` until the deciding frame is complete — never `Unsupported`
    /// or `Supported` on a truncated frame. A parser that guesses from a
    /// partial frame passes the hand-picked cases above and fails this.
    #[test]
    fn a_supported_stream_is_incomplete_at_every_split_before_its_frame_completes() {
        let decisive = graphics_reply(SUPPORT_QUERY_ID, "OK");
        let mut stream = decisive.clone();
        stream.extend_from_slice(&da1_reply());
        for prefix in 0..decisive.len() {
            assert_eq!(
                classify_support_reply(&stream[..prefix]),
                SupportReply::Incomplete,
                "{prefix} byte(s) cannot decide: the graphics frame is not complete"
            );
        }
        for prefix in decisive.len()..=stream.len() {
            assert_eq!(
                classify_support_reply(&stream[..prefix]),
                SupportReply::Supported,
                "the complete graphics frame decides at {prefix} byte(s)"
            );
        }
    }

    /// VT-9: the same property on the refusing side — a half-arrived DA1 reply
    /// must not read as `Unsupported` early.
    #[test]
    fn an_unsupported_stream_is_incomplete_at_every_split_before_its_frame_completes() {
        let stream = da1_reply();
        for prefix in 0..stream.len() {
            assert_eq!(
                classify_support_reply(&stream[..prefix]),
                SupportReply::Incomplete,
                "{prefix} byte(s) cannot decide: the DA1 frame is not complete"
            );
        }
        assert_eq!(classify_support_reply(&stream), SupportReply::Unsupported);
    }

    /// VT-6: width and height off a hand-built 24-byte IHDR header.
    #[test]
    fn png_size_reads_the_ihdr_dimensions() {
        assert_eq!(
            png_size(&ihdr(4000, 1000)),
            Some(PngSize {
                width: 4000,
                height: 1000
            })
        );
        // Trailing chunk data is irrelevant — only the fixed header is read.
        let mut with_body = ihdr(1, 2);
        with_body.extend_from_slice(b"...the rest of the file...");
        assert_eq!(
            png_size(&with_body),
            Some(PngSize {
                width: 1,
                height: 2
            })
        );
    }

    /// VT-6: the three refusals — short input, a bad signature, a first chunk
    /// that is not IHDR.
    #[test]
    fn png_size_refuses_anything_that_is_not_a_png_header() {
        let short = ihdr(10, 10);
        assert_eq!(png_size(&short[..23]), None, "fewer than 24 bytes");

        let mut bad_signature = ihdr(10, 10);
        bad_signature[0] = b'X';
        assert_eq!(png_size(&bad_signature), None, "signature mismatch");

        let mut not_ihdr = ihdr(10, 10);
        not_ihdr[12..16].copy_from_slice(b"sRGB");
        assert_eq!(png_size(&not_ihdr), None, "first chunk is not IHDR");
    }

    /// VT-7: integer cell size, or `None` for any zero field or zero quotient.
    #[test]
    fn cell_geometry_refuses_zero_fields_and_zero_quotients() {
        assert_eq!(
            cell_geometry(&window(200, 50, 2000, 1000)),
            Some(CellGeometry {
                columns: 200,
                cell_width: 10,
                cell_height: 20
            })
        );
        // Any unreported field (0) refuses.
        assert_eq!(cell_geometry(&window(0, 50, 2000, 1000)), None);
        assert_eq!(cell_geometry(&window(200, 0, 2000, 1000)), None);
        assert_eq!(cell_geometry(&window(200, 50, 0, 1000)), None);
        assert_eq!(cell_geometry(&window(200, 50, 2000, 0)), None);
        // Reported, but degenerate: fewer pixels than cells on either axis.
        assert_eq!(cell_geometry(&window(200, 50, 100, 1000)), None);
        assert_eq!(cell_geometry(&window(200, 50, 2000, 10)), None);
    }

    /// VT-8: every row of design sec-4's placement table (DEC-256).
    #[test]
    fn place_reproduces_the_design_placement_table() {
        // small graph: 200 × 50 cols/rows over 2000 × 1000 px ⇒ cell 10 × 20.
        let wide_window = CellGeometry {
            columns: 200,
            cell_width: 10,
            cell_height: 20,
        };
        assert_eq!(
            place(
                PngSize {
                    width: 300,
                    height: 200
                },
                &wide_window
            ),
            Placement {
                columns: 30,
                rows: 10
            },
            "small: native 30 ≤ max_columns 199"
        );
        // exactly fits: native 199 == max_columns 199, the boundary — native,
        // not scaled.
        assert_eq!(
            place(
                PngSize {
                    width: 1990,
                    height: 400
                },
                &wide_window
            ),
            Placement {
                columns: 199,
                rows: 20
            },
            "exactly fits: the bound is inclusive"
        );
        // wide graph: native 400 > 199 ⇒ clamp and scale the height.
        assert_eq!(
            place(
                PngSize {
                    width: 4000,
                    height: 1000
                },
                &wide_window
            ),
            Placement {
                columns: 199,
                rows: 25
            },
            "wide: scaled_height = ceil(1000 * 1990 / 4000) = 498 px ⇒ ceil(498/20) = 25"
        );
        // tall graph: 80 × 24 over 800 × 480 ⇒ cell 10 × 20. No vertical bound.
        let tall_window = CellGeometry {
            columns: 80,
            cell_width: 10,
            cell_height: 20,
        };
        assert_eq!(
            place(
                PngSize {
                    width: 400,
                    height: 3000
                },
                &tall_window
            ),
            Placement {
                columns: 40,
                rows: 150
            },
            "tall: native, scrolls"
        );
        // one-column terminal: columns - 1 would be 0, floored at 1.
        let one_column = CellGeometry {
            columns: 1,
            cell_width: 10,
            cell_height: 20,
        };
        assert_eq!(
            place(
                PngSize {
                    width: 100,
                    height: 100
                },
                &one_column
            ),
            Placement {
                columns: 1,
                rows: 1
            },
            "one column: max_columns floored at 1, rows floored at 1"
        );
    }

    /// VT-4: a payload that fits one chunk is a single escape carrying the full
    /// control set with `m=0`, then exactly `rows` newlines.
    #[test]
    fn a_single_chunk_payload_is_one_escape_with_the_full_control_set() {
        let png = b"a small png";
        let encoded = encode_png(
            png,
            Placement {
                columns: 30,
                rows: 10,
            },
        );
        let expected = format!(
            "\x1b_Ga=T,f=100,t=d,q=2,C=1,c=30,r=10,m=0;{}\x1b\\{}",
            base64::engine::general_purpose::STANDARD.encode(png),
            "\n".repeat(10),
        );
        assert_eq!(encoded, expected.as_bytes());
    }

    /// VT-5: `vec![0; 5000]` spills past one chunk — the non-final chunk is
    /// exactly 4096 bytes and carries `m=1`, the last carries `m=0`, and the
    /// concatenated payloads decode back to the input.
    #[test]
    fn a_multi_chunk_payload_slices_at_4096_and_round_trips() {
        let png = vec![0; 5000];
        let placement = Placement {
            columns: 199,
            rows: 25,
        };
        let encoded = encode_png(&png, placement);
        let frames = frames_of(&encoded, placement.rows);
        assert_eq!(frames.len(), 2, "6668 base64 bytes is two chunks");
        assert_eq!(
            frames[0].0, "a=T,f=100,t=d,q=2,C=1,c=199,r=25,m=1",
            "the first chunk carries the full control set and 'more follows'"
        );
        assert_eq!(
            frames[0].1.len(),
            4096,
            "every non-final chunk is exactly 4096 bytes"
        );
        assert_eq!(
            frames[1].0, "m=0,q=2",
            "the last chunk carries only m and q"
        );

        let payload: String = frames.iter().map(|(_, chunk)| *chunk).collect();
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(payload)
            .expect("the chunk boundaries preserve the base64 stream");
        assert_eq!(decoded, png, "the chunks reassemble into the original PNG");
    }

    /// VT-5: with three chunks there is a MIDDLE continuation, which is where
    /// `m=1,q=2` — only `m` and `q`, never the transmit keys — is observable.
    #[test]
    fn middle_continuations_carry_only_m_and_q() {
        let png = vec![0; 7000];
        let placement = Placement {
            columns: 10,
            rows: 3,
        };
        let encoded = encode_png(&png, placement);
        let frames = frames_of(&encoded, placement.rows);
        assert_eq!(frames.len(), 3, "9336 base64 bytes is three chunks");
        assert_eq!(frames[1].0, "m=1,q=2");
        assert_eq!(frames[1].1.len(), 4096);
        assert_eq!(frames[2].0, "m=0,q=2");
    }

    /// Split an encoded stream into `(control data, payload)` pairs, asserting
    /// the trailing newlines and the framing along the way.
    fn frames_of(encoded: &[u8], rows: u16) -> Vec<(&str, &str)> {
        let text = std::str::from_utf8(encoded).expect("the escape stream is ASCII");
        let body = text
            .strip_suffix(&"\n".repeat(usize::from(rows)))
            .expect("one newline per placement row, so the cursor lands below the image");
        assert!(!body.ends_with('\n'), "exactly `rows` newlines, no more");
        body.split("\x1b_G")
            .filter(|frame| !frame.is_empty())
            .map(|frame| {
                let frame = frame
                    .strip_suffix("\x1b\\")
                    .expect("every escape ends with the string terminator");
                frame
                    .split_once(';')
                    .expect("control data, then the payload")
            })
            .collect()
    }
}
