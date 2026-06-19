// SPDX-License-Identifier: GPL-3.0-only
//! The kind-blind read spine: the invariant axes of every `list` surface —
//! filtering, id form, output format, the generic table layout, and the JSON
//! envelope — lifted out of the five bespoke per-kind implementations (design
//! SL-025 §5.2).
//!
//! This is a **pure leaf** (ADR-001 module layering: leaf ← engine ← command).
//! It imports no `clap` and no `entity`: the clap-facing arg bundle lives
//! command-side and maps parsed args onto [`Filter`] / [`Format`] here, and the
//! id prefix is passed into [`canonical_id`] as a plain `&str` rather than read
//! from `entity::Kind`. Nothing here reads the clock, rng, git, or disk.
//!
//! What is shared (the *invariant* axis): the filter semantics ([`retain`] over
//! [`FilterFields`]), the status known-set check ([`validate_statuses`]), the
//! canonical id form ([`canonical_id`]), the table layout ([`render_table`],
//! relocated from `meta.rs`), and the JSON envelope ([`json_envelope`]). What
//! stays per-kind (the *variant* axis): the row type, the column projection, the
//! ordering, and the kind-specific flags — none of which live here.
//!
//! The kinds migrate onto this leaf one per phase (SL-025 PHASE-02+). A symbol
//! not yet consumed by a migrated kind carries a narrowed per-symbol
//! `#[expect(dead_code)]` that retires itself as each phase wires its consumer.

use std::fmt;
use std::str::FromStr;

use anyhow::Context;
use comfy_table::{ContentArrangement, Table, TableComponent};
use regex_lite::Regex;
use serde::{Deserialize, Serialize};

/// `SL` + `25` → `"SL-25"`; zero-padded to three digits like the citation
/// convention (`SL-025`). The single id-form authority for prefixed kinds.
/// Memory is conformant-by-exception — its uid *is* its canonical id, so it does
/// not route through here (design §5.3).
pub(crate) fn canonical_id(prefix: &str, id: u32) -> String {
    format!("{prefix}-{id:03}")
}

/// Output format for a `list`/`show` surface. A plain enum (NOT `clap::ValueEnum`,
/// which would drag clap into this leaf — A-3); the command layer wires it via
/// `#[arg(value_parser = Format::from_str)]`. `Display` is required by clap's
/// `default_value_t` (C-6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
pub(crate) enum Format {
    /// The default human-readable surface; `--format`/`--json` opt out of it.
    #[default]
    Table,
    Json,
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Format::Table => "table",
            Format::Json => "json",
        })
    }
}

impl FromStr for Format {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "table" => Ok(Format::Table),
            "json" => Ok(Format::Json),
            other => Err(anyhow::anyhow!(
                "unknown format `{other}` (expected `table` or `json`)"
            )),
        }
    }
}

/// One row projected to exactly its filterable fields — computed **once** per row
/// by the kind, then reused across both match domains (A-1). `substr` matches
/// `slug`+`title`; `regex` matches `canonical`+`slug`+`title` — distinct domains,
/// both derivable from this single projection. Filter-only; never a render type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FilterFields {
    /// The regex domain's leading field: `SL-025` / `ADR-001` / `mem_…`.
    pub(crate) canonical: String,
    pub(crate) slug: String,
    pub(crate) title: String,
    pub(crate) status: String,
    pub(crate) tags: Vec<String>,
}

/// The resolved, pre-compiled filter. Built once command-side from the parsed
/// flags via [`build`]; the leaf never sees clap. `regex_lite::Regex` is not
/// `Eq`, so this type intentionally does not derive `PartialEq` — compare by
/// behaviour in tests.
pub(crate) struct Filter {
    /// Lowercased once at build time (case-insensitive substring match).
    pub(crate) substr: Option<String>,
    /// Pre-compiled; the case flag is baked into the pattern at build time.
    pub(crate) regex: Option<Regex>,
    /// Empty = no status constraint; otherwise a row matches iff its status is in
    /// the set (OR within `--status`).
    pub(crate) status: Vec<String>,
    /// Empty = no tag constraint; otherwise OR within tags.
    pub(crate) tags: Vec<String>,
    /// Show every state, including the kind's hide-set.
    pub(crate) all: bool,
}

impl fmt::Debug for Filter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Filter")
            .field("substr", &self.substr)
            .field("regex", &self.regex.as_ref().map(Regex::as_str))
            .field("status", &self.status)
            .field("tags", &self.tags)
            .field("all", &self.all)
            .finish()
    }
}

/// The raw, parsed flag values the command layer hands to [`build`] — the
/// clap-free mirror of its `CommonListArgs` bundle (no clap types cross this
/// seam, A-3). One struct rather than a long positional argument list keeps the
/// call site self-documenting and the seam stable as flags accrete.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct ListArgs {
    /// Substring filter on slug+title (case-insensitive).
    pub(crate) substr: Option<String>,
    /// Regex over canonical-id + slug + title.
    pub(crate) regexp: Option<String>,
    /// Make the regex case-insensitive.
    pub(crate) case_insensitive: bool,
    /// Status filter, multi-value; any value reveals the hide-set.
    pub(crate) status: Vec<String>,
    /// Tag filter, OR within the axis.
    pub(crate) tags: Vec<String>,
    /// Show every state, including the hide-set.
    pub(crate) all: bool,
    /// Output format from `--format` (overridden by `json` below).
    pub(crate) format: Format,
    /// `--json` sugar — forces [`Format::Json`] regardless of `format` (A-9).
    pub(crate) json: bool,
    /// `--columns` table projection (SL-037 D3/D6): the verb pulls it via
    /// `args.columns.take()` *before* [`build`], which never reads it — the
    /// table is the only axis it touches (D2; no effect under `--json`, D7).
    pub(crate) columns: Option<Vec<String>>,
    /// The render-axis bundle handed to [`render_columns`] (SL-054 PHASE-01). The
    /// impure shell resolves its `color` ONCE ([`crate::tty::stdout_color_enabled`])
    /// and injects it here; the verb reads `args.render` before [`build`] and passes
    /// it to [`render_columns`]. `default = RenderOpts { color: false, term_width:
    /// None }` (piped/in-process output, and every `..Default::default()` test or
    /// `boot` call, stays plain and unwrapped — VT-4).
    pub(crate) render: RenderOpts,
}

/// The render-axis bundle threaded into [`render_columns`] (SL-054 PHASE-01). One
/// struct rather than a widening positional list keeps the render seam stable as
/// axes accrete. `Default` is the plain path: `{ color: false, term_width: None }`
/// (no colour, no terminal-width wrapping) — VT-2.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub(crate) struct RenderOpts {
    /// Whether the table render may emit ANSI colour (SL-053 D3). Resolved ONCE by
    /// the impure shell; `false` keeps piped/in-process output byte-clean.
    pub(crate) color: bool,
    /// Terminal width for wrapping (SL-054 PHASE-02). `None` ⇒ the unwrapped
    /// (`ContentArrangement::Disabled`) path; `Some(w)` clearing the per-grid
    /// structural floor ([`grid_min_width`]) ⇒ `Dynamic` wrap to `w` columns; below
    /// the floor it falls back to `Disabled` (clean overflow beats sliver wrapping).
    pub(crate) term_width: Option<u16>,
}

/// Build a [`Filter`] + resolved [`Format`] from the parsed [`ListArgs`].
/// Lowercases the substring once, pre-compiles the regex (a bad pattern is a
/// clean `anyhow` error, never a panic), and folds `--json` over `--format`:
/// `--json` forces [`Format::Json`] and wins over any `--format` value (A-9).
pub(crate) fn build(args: ListArgs) -> anyhow::Result<(Filter, Format)> {
    let regex = match args.regexp {
        None => None,
        Some(pattern) => {
            let pattern = if args.case_insensitive {
                format!("(?i){pattern}")
            } else {
                pattern
            };
            let compiled =
                Regex::new(&pattern).with_context(|| format!("invalid regex `{pattern}`"))?;
            Some(compiled)
        }
    };
    let filter = Filter {
        substr: args.substr.map(|s| s.to_lowercase()),
        regex,
        status: args.status,
        tags: args.tags,
        all: args.all,
    };
    let resolved = if args.json { Format::Json } else { args.format };
    Ok((filter, resolved))
}

/// Keep a row iff every active axis admits it. Axes AND across each other; within
/// `--status` and `--tag` the match is OR. Concretely a row is kept when:
/// not hidden (by the kind's LIST hide-set) AND substr-match (slug+title) AND
/// regex-match (canonical+slug+title) AND status-match AND tag-match.
///
/// The hide-set is suppressed when `--all` is set OR any explicit `--status` is
/// given (design §5.5 terminal-hide override). `is_hidden` is the kind's
/// presentation hide-set predicate — **distinct** from any divergence /
/// lifecycle-terminal predicate (design §5.3); `key` projects each row to its
/// [`FilterFields`].
///
/// FILTER-ONLY: ordering is the caller's (per-kind) concern (design §5.3) — this
/// preserves input order and never sorts.
pub(crate) fn retain<R>(
    rows: Vec<R>,
    f: &Filter,
    is_hidden: impl Fn(&str) -> bool,
    key: impl Fn(&R) -> FilterFields,
) -> Vec<R> {
    let reveal_hidden = f.all || !f.status.is_empty();
    rows.into_iter()
        .filter(|row| {
            let fields = key(row);
            if !reveal_hidden && is_hidden(&fields.status) {
                return false;
            }
            substr_admits(f, &fields)
                && regex_admits(f, &fields)
                && status_admits(f, &fields)
                && tags_admit(f, &fields)
        })
        .collect()
}

/// Substr match domain: lowercased slug+title (the substring was lowercased once
/// at build, so compare lowercased haystacks).
fn substr_admits(f: &Filter, fields: &FilterFields) -> bool {
    match &f.substr {
        None => true,
        Some(needle) => {
            fields.slug.to_lowercase().contains(needle)
                || fields.title.to_lowercase().contains(needle)
        }
    }
}

/// Regex match domain: canonical+slug+title (distinct from the substr domain —
/// the canonical id is searchable only here, A-1).
fn regex_admits(f: &Filter, fields: &FilterFields) -> bool {
    match &f.regex {
        None => true,
        Some(re) => {
            re.is_match(&fields.canonical)
                || re.is_match(&fields.slug)
                || re.is_match(&fields.title)
        }
    }
}

fn status_admits(f: &Filter, fields: &FilterFields) -> bool {
    f.status.is_empty() || f.status.contains(&fields.status)
}

fn tags_admit(f: &Filter, fields: &FilterFields) -> bool {
    f.tags.is_empty() || f.tags.iter().any(|t| fields.tags.contains(t))
}

/// Validate a stringly `--status` set against a kind's known statuses, with one
/// uniform error message (A-2 — recovers the correctness the shared
/// `Vec<String>` bundle loses versus a typed clap enum). Validates READ (filter)
/// input only — never a stored-status write/transition.
pub(crate) fn validate_statuses(given: &[String], known: &[&str]) -> anyhow::Result<()> {
    if let Some(bad) = given.iter().find(|s| !known.contains(&s.as_str())) {
        let known = known.join(", ");
        anyhow::bail!("unknown status `{bad}` (known: {known})");
    }
    Ok(())
}

/// One table column for a kind's row type `R`: a `name` (the `--columns`
/// selector token — shell-safe, lowercase), a `header` (display text; usually
/// == name), and a pure non-capturing cell extractor (SL-037 D5). Table-ONLY
/// (D2) — JSON rows stay typed per-kind.
/// NOT `#[derive(Copy)]` — derive would add a spurious `R: Copy` bound; columns
/// are only ever borrowed (`&[Column<R>]` / `Vec<&Column<R>>`), never moved.
pub(crate) struct Column<R> {
    pub(crate) name: &'static str,
    pub(crate) header: &'static str,
    pub(crate) cell: fn(&R) -> String,
    /// How this column's data cells are coloured when colour is enabled (SL-053
    /// PHASE-02). [`ColumnPaint::None`] for most columns; [`ColumnPaint::Fixed`]
    /// for id columns (a stable hue per kind); [`ColumnPaint::ByValue`] for status
    /// columns (the hue is a function of the ROW's raw status, NOT the emitted cell
    /// — F-4). Inert under `color = false` (every cell passes through unchanged).
    pub(crate) paint: ColumnPaint<R>,
}

/// How a [`Column`]'s data cells are coloured (SL-053 PHASE-02, D3). Headers are
/// painted uniformly (bold) by [`render_columns`]; this governs only the *data*
/// cells, and ONLY when the injected `color` bool is true.
pub(crate) enum ColumnPaint<R> {
    /// No colour — the cell string passes through byte-for-byte (the default; the
    /// only behaviour under `color = false`).
    None,
    /// A single fixed hue for every cell in the column (e.g. id columns).
    Fixed(owo_colors::DynColors),
    /// A hue derived from the ROW — reads the row's raw semantic status, never the
    /// emitted/decorated cell text (F-4: `slice list` decorates `done ⚠`, `review
    /// list` emits `open (await …)`; matching the cell would drop colour on exactly
    /// those surfaces). `None` ⇒ that row's cell is left uncoloured.
    ByValue(fn(&R) -> Option<owo_colors::DynColors>),
    /// Multi-valued cell: `split` yields the row's tokens; `render` paints ONE token
    /// (ANSI). Invoked ONLY under `color = true`; tokens joined by `", "`.
    PerToken {
        split: fn(&R) -> Vec<String>,
        render: fn(&str) -> String,
    },
    /// Alternating two-hue foreground per data-row index, for zebra-striping the
    /// title column. `[even_hue, odd_hue]`; data row 0 uses `even_hue`. The header
    /// row is excluded — [`render_columns`] builds it separately via [`paint_header`].
    Alternate([owo_colors::DynColors; 2]),
}

/// The single shared status→hue map for every coloured `list` surface (SL-053
/// PHASE-02) — no per-kind duplication. Greens are settled/accepted/required
/// states, yellows are in-flight lifecycle states, reds are stopped/adverse states.
///
/// A deliberately conservative SUBSET of the painted kinds' status vocabularies,
/// NOT their union: only tokens with an unambiguous semantic colour are mapped;
/// every other live token (`proposed`/`open`/`draft`/`resolved`/`closed`/
/// `superseded`/…) falls through to `None` (uncoloured) by design. Every mapped
/// token is one a painted surface can actually emit — slice lifecycle
/// (`proposed→design→plan→ready→started→audit→reconcile→done`/`abandoned`),
/// governance (`accepted`/`required`/`active`), review (`active`/`done`), memory
/// (`active`/…), spec/req (`active`/`draft`/…).
///
/// `in_progress` is intentionally absent: it is a *phase* status, rendered in the
/// `phases` column as a rollup count (`4/6`), never as a painted status cell — so
/// mapping it would be dead. `ready` is yellow alongside its in-flight lifecycle
/// siblings (`design`/`plan`/`started`/`audit`/`reconcile`); only the pre-engagement
/// `proposed` stays grey.
pub(crate) fn status_hue(s: &str) -> Option<owo_colors::DynColors> {
    use owo_colors::{
        AnsiColors::{Green, Red, Yellow},
        DynColors,
    };
    match s {
        "done" | "active" | "accepted" | "required" => Some(DynColors::Ansi(Green)),
        "design" | "plan" | "ready" | "started" | "audit" | "reconcile" => {
            Some(DynColors::Ansi(Yellow))
        }
        "blocked" | "abandoned" | "contested" => Some(DynColors::Ansi(Red)),
        _ => None,
    }
}

/// Wrap a status token in ANSI colour via the shared [`status_hue`] map, gated
/// on the injected `color` bool. Pure — both inputs injected. When `color` is
/// false, returns the status unchanged (zero ANSI). When `color` is true and a
/// status is unmapped in [`status_hue`], returns it unchanged (no colour).
pub(crate) fn status_colored(status: &str, color: bool) -> String {
    use owo_colors::OwoColorize;
    if !color {
        return status.to_string();
    }
    match status_hue(status) {
        Some(hue) => status.color(hue).to_string(),
        None => status.to_string(),
    }
}

/// Backlog item kind → hue: a distinct, stable ANSI colour per kind.
/// `issue`/`improvement`/`chore`/`risk`/`idea`; unknown kind → `None`.
pub(crate) fn backlog_kind_hue(kind: &str) -> Option<owo_colors::DynColors> {
    use owo_colors::{
        AnsiColors::{Blue, Green, Magenta, Red, Yellow},
        DynColors,
    };
    match kind {
        "issue" => Some(DynColors::Ansi(Red)),
        "improvement" => Some(DynColors::Ansi(Green)),
        "chore" => Some(DynColors::Ansi(Yellow)),
        "risk" => Some(DynColors::Ansi(Magenta)),
        "idea" => Some(DynColors::Ansi(Blue)),
        _ => None,
    }
}

/// Memory type → hue: a distinct, stable ANSI colour per memory kind.
/// `concept`/`fact`/`pattern`/`signpost`/`system`/`thread`; unknown → `None`.
pub(crate) fn memory_type_hue(kind: &str) -> Option<owo_colors::DynColors> {
    use owo_colors::{
        AnsiColors::{Blue, Cyan, Green, Magenta, Red, Yellow},
        DynColors,
    };
    match kind {
        "concept" => Some(DynColors::Ansi(Cyan)),
        "fact" => Some(DynColors::Ansi(Green)),
        "pattern" => Some(DynColors::Ansi(Magenta)),
        "signpost" => Some(DynColors::Ansi(Blue)),
        "system" => Some(DynColors::Ansi(Yellow)),
        "thread" => Some(DynColors::Ansi(Red)),
        _ => None,
    }
}

/// Memory trust level → hue: signals confidence/severity.
/// `high` → Green, `medium` → Yellow, `low` → Red; unknown → `None`.
pub(crate) fn trust_hue(trust: &str) -> Option<owo_colors::DynColors> {
    use owo_colors::{
        AnsiColors::{Green, Red, Yellow},
        DynColors,
    };
    match trust {
        "high" => Some(DynColors::Ansi(Green)),
        "medium" => Some(DynColors::Ansi(Yellow)),
        "low" => Some(DynColors::Ansi(Red)),
        _ => None,
    }
}

/// The tag-chip segment palette: a fixed set of distinguishable hues indexed by a
/// stable byte-fold of the segment. Gruvbox truecolour palette, 12 entries for
/// maximum distinguishability. EXCLUDES `Red` (reserved for adverse status in
/// [`status_hue`]). Deterministic — no RNG, no clock.
const TAG_PALETTE: [owo_colors::Rgb; 12] = [
    owo_colors::Rgb(204, 36, 29),   // red           #cc241d
    owo_colors::Rgb(152, 151, 26),  // green         #98971a
    owo_colors::Rgb(215, 153, 33),  // yellow        #d79921
    owo_colors::Rgb(69, 133, 136),  // blue          #458588
    owo_colors::Rgb(177, 98, 134),  // purple        #b16286
    owo_colors::Rgb(104, 157, 106), // aqua          #689d6a
    owo_colors::Rgb(214, 93, 14),   // orange        #d65d0e
    owo_colors::Rgb(250, 189, 47),  // bright yellow #fabd2f
    owo_colors::Rgb(131, 165, 152), // bright blue   #83a598
    owo_colors::Rgb(211, 134, 155), // bright purple #d3869b
    owo_colors::Rgb(142, 192, 124), // bright aqua   #8ec07c
    owo_colors::Rgb(254, 128, 25),  // bright orange #fe8019
];

/// Zebra-striping title-column hues: two subtle gruvbox-adjacent foreground colours
/// for alternating row visual separation. Even rows → `TITLE_EVEN`; odd rows →
/// `TITLE_ODD`. Wired via [`ColumnPaint::Alternate`] on the title column of each
/// list surface.
pub(crate) const TITLE_EVEN: owo_colors::DynColors = owo_colors::DynColors::Rgb(255, 230, 195); // #ebdbb2
pub(crate) const TITLE_ODD: owo_colors::DynColors = owo_colors::DynColors::Rgb(213, 196, 161); // #d5c4a1

/// A pure byte fold (FNV-1a, 32-bit) over a segment's bytes. No RNG, no clock —
/// deterministic across runs. Wrapping arithmetic keeps it cast-free (repo clippy
/// bans `as`).
fn stable_hash(seg: &str) -> u32 {
    let mut hash: u32 = 0x811c_9dc5;
    for byte in seg.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

/// Stable, pure hue for a colon-segment: byte-fold hash → fixed palette index. No
/// RNG, no clock — deterministic. Empty segment → `None` (no text, no colour).
fn segment_hue(seg: &str) -> Option<owo_colors::DynColors> {
    if seg.is_empty() {
        return None;
    }
    // Reduce in u32 space (the const palette length fits a u32, never 0), then narrow
    // the small remainder — keeps the fold cast-free and panic-free (repo clippy bans
    // `as` and `expect`).
    let len = u32::try_from(TAG_PALETTE.len()).unwrap_or(1);
    let index = usize::try_from(stable_hash(seg) % len).unwrap_or(0);
    TAG_PALETTE
        .get(index)
        .map(|c| owo_colors::DynColors::Rgb(c.0, c.1, c.2))
}

/// Render one tag as a colon-segment chip: segments hued by [`segment_hue`], colons
/// painted white. ANSI unconditional — only ever called under colour (the
/// [`ColumnPaint::PerToken`] gate). `cli:command` → hue(cli) + white `:` +
/// hue(command); `security` → one hue; empty segments (`:x`, `a::b`) contribute no
/// text but the white colon still renders.
pub(crate) fn paint_tag(tag: &str) -> String {
    use owo_colors::{AnsiColors::White, DynColors, OwoColorize};
    let white = DynColors::Ansi(White);
    let mut out = String::with_capacity(tag.len());
    for (index, seg) in tag.split(':').enumerate() {
        if index != 0 {
            out.push_str(&":".color(white).to_string());
        }
        match segment_hue(seg) {
            Some(hue) => out.push_str(&seg.color(hue).to_string()),
            None => out.push_str(seg),
        }
    }
    out
}

/// Resolve the visible, ordered selection. `requested` = parsed `--columns`
/// (None → `default`, taken verbatim). Each requested name is validated against
/// `available`; an unknown name is one uniform `anyhow` error listing the
/// available tokens (A-2 parity with [`validate_statuses`]). Requested order is
/// preserved; duplicates are permitted (the user asked for them) — SL-037 OQ-2:
/// subset+order, dups pass.
pub(crate) fn select_columns<'a, R>(
    available: &'a [Column<R>],
    default: &[&str],
    requested: Option<&[String]>,
) -> anyhow::Result<Vec<&'a Column<R>>> {
    debug_assert!(
        requested.is_some()
            || default
                .iter()
                .all(|d: &&str| available.iter().any(|c| c.name == *d)),
        "default column `{}` not in available set [{}]",
        default
            .iter()
            .find(|d| !available.iter().any(|c| c.name == **d))
            .unwrap_or(&"?"),
        available
            .iter()
            .map(|c| c.name)
            .collect::<Vec<_>>()
            .join(", ")
    );
    let pick = |name: &str| {
        available.iter().find(|c| c.name == name).ok_or_else(|| {
            let known = available
                .iter()
                .map(|c| c.name)
                .collect::<Vec<_>>()
                .join(", ");
            anyhow::anyhow!("unknown column `{name}` (available: {known})")
        })
    };
    match requested {
        None => default.iter().map(|n| pick(n)).collect(), // default names are curated-valid
        Some(names) => names.iter().map(|n| pick(n)).collect(),
    }
}

/// Header row + one cell-row per `R`, over [`render_table`]. Empty rows → `""`
/// (header suppressed, SL-025 §5.5). Replaces every kind's bespoke table
/// assembler.
pub(crate) fn render_columns<R>(rows: &[R], cols: &[&Column<R>], opts: RenderOpts) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let color = opts.color;
    let mut grid: Vec<Vec<String>> = Vec::with_capacity(rows.len() + 1);
    // Header row: bold when colour is on (D-EX3), raw otherwise.
    grid.push(cols.iter().map(|c| paint_header(c.header, color)).collect());
    grid.extend(rows.iter().enumerate().map(|(i, r)| {
        cols.iter()
            .map(|c| paint_cell(&(c.cell)(r), &c.paint, r, color, i))
            .collect()
    }));
    // comfy-table's custom_styling makes its width measurement ANSI-aware, so the
    // embedded escapes do not disturb the ` │ ` separator alignment (VT-2).
    render_table(&grid, opts.term_width)
}

/// Paint a header cell bold when `color`; otherwise return it raw. The bold escape
/// is emitted ONLY under colour, so piped output (`color = false`) is byte-clean.
fn paint_header(header: &str, color: bool) -> String {
    use owo_colors::OwoColorize;
    if color {
        header.bold().to_string()
    } else {
        header.to_string()
    }
}

/// Paint one data `cell` per its column's [`ColumnPaint`], reading the ROW for the
/// `ByValue` hue (F-4 — never the emitted cell). Under `color = false` or
/// [`ColumnPaint::None`] (or a `ByValue` returning `None`) the cell is returned
/// UNCHANGED — zero ANSI (EX-3). Uses `owo_colors`' UNCONDITIONAL `.color(..)`, gated
/// solely on the injected bool (never `if_supports_color`, D3).
fn paint_cell<R>(
    cell: &str,
    paint: &ColumnPaint<R>,
    row: &R,
    color: bool,
    row_index: usize,
) -> String {
    use owo_colors::OwoColorize;
    if !color {
        return cell.to_string();
    }
    // Multi-valued cell: paint each token via `render`, join by `", "`. Returns BEFORE
    // the hue match (whose return type is `String`, not `Option<DynColors>` — folding
    // this arm in would clash). Reached ONLY under `color == true`.
    if let ColumnPaint::PerToken { split, render } = paint {
        return split(row)
            .iter()
            .map(|t| render(t.as_str()))
            .collect::<Vec<_>>()
            .join(", ");
    }
    // Alternating zebra stripe: applies to the cell directly (String-returning),
    // so it needs its own early return before the `Option<DynColors>` hue match.
    if let ColumnPaint::Alternate([even, odd]) = paint {
        let hue = if row_index.is_multiple_of(2) {
            *even
        } else {
            *odd
        };
        return cell.color(hue).to_string();
    }
    let hue = match paint {
        ColumnPaint::Fixed(c) => Some(*c),
        ColumnPaint::ByValue(f) => f(row),
        // `PerToken` and `Alternate` are handled by the early returns above; folding
        // them in beside `None` keeps the match exhaustive.
        ColumnPaint::None | ColumnPaint::PerToken { .. } | ColumnPaint::Alternate(_) => None,
    };
    match hue {
        Some(c) => cell.color(c).to_string(),
        None => cell.to_string(),
    }
}

/// The interior column separator: a single box-drawing vertical, surrounded by the
/// `(1,1)` column padding into the ` │ ` inner gap. The single source of column
/// spacing for every `*list` surface (SL-053 D7).
const COLUMN_SEPARATOR: char = '│';

/// Render `rows` as a left-aligned text table with interior `│` column separators
/// and no outer frame, no horizontal/header rules: each column is padded to its
/// widest cell, the first column carries no leading space and the last no trailing
/// space, and a trailing newline terminates non-empty output (no rows → `""`, which
/// keeps the header suppressed on an empty list — design §5.5).
///
/// comfy-table is the **sole** layout + width-measurement authority (SL-053): all
/// hand-rolled width maths is gone. The minimalist style is built component-by-
/// component (D7) — every outer border / corner / horizontal / intersection style
/// is removed (so `should_draw_left/right_border` and `should_draw_horizontal_lines`
/// return false → clean edges, no rules) and only [`TableComponent::VerticalLines`]
/// is set, yielding the interior ` │ ` separator. The first column's left pad and
/// the last column's right pad are zeroed post-build so both outer edges are clean.
/// Determinism is bought by BOTH [`ContentArrangement::Disabled`] (never measure the
/// terminal) AND [`Table::force_no_tty`] (`custom_styling` transitively enables `tty`,
/// whose content formatter would otherwise consult `stdout().is_terminal()` at format
/// time — D6). comfy-table omits the final newline; we re-append it for the caller's
/// print-verbatim seam.
///
/// The single layout authority for every list surface (relocated from `meta.rs`
/// — it served only numeric kinds there, but the spine serves named (memory) and
/// own-struct (backlog) kinds too). It carries no per-kind knowledge and renders
/// nothing but a grid of strings (not a column framework). Callers bake any
/// markers — and the header row — into the cell strings, so it stays
/// presentation-neutral.
///
/// PRECONDITION: `rows` is rectangular (every row the same length). The shared
/// `render_columns` assembler only ever produces rectangular grids; a ragged grid
/// is a caller bug and fails loudly under `debug_assert` rather than mis-rendering
/// (SL-053 F-2) — the old hand-rolled renderer silently tolerated raggedness.
pub(crate) fn render_table(rows: &[Vec<String>], term_width: Option<u16>) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let width = rows.first().map_or(0, Vec::len);
    debug_assert!(
        rows.iter().all(|r| r.len() == width),
        "render_table requires a rectangular grid; got ragged rows"
    );

    let mut table = Table::new();
    // SL-054 PHASE-02: `Some(w)` that clears the structural floor switches to
    // terminal-width wrapping (`Dynamic` + `set_width(w)`); below the floor, and for
    // `None`, fall back to `Disabled` (clean overflow beats 1-char Dynamic slivers).
    // The arrangement is the ONLY wrap-conditional decision — everything below stays
    // unconditional (`None` renders byte-for-byte as before: the None-invariance proof).
    let fits = |w: u16| usize::from(w) >= grid_min_width(width);
    match term_width {
        Some(w) if fits(w) => {
            table.set_content_arrangement(ContentArrangement::Dynamic);
            table.set_width(w);
        }
        _ => {
            table.set_content_arrangement(ContentArrangement::Disabled);
        }
    }
    // Never let the content formatter consult a tty at format time — keeps output
    // byte-stable terminal-vs-pipe (D6). Orthogonal to the wrap arm above (a recorded
    // spike): `force_no_tty` governs comfy-table's styling tty-consult only, never the
    // Dynamic width budget, so it stays UNCONDITIONAL under both arms.
    table.force_no_tty();

    // Strip every outer/rule component, then set ONLY the interior vertical. The
    // header row is just the grid's first row (callers bake it in) — we never call
    // set_header, so no header line is ever drawn.
    for component in TableComponent::iter() {
        table.remove_style(component);
    }
    table.set_style(TableComponent::VerticalLines, COLUMN_SEPARATOR);

    for row in rows {
        table.add_row(row.clone());
    }

    // Zero the first column's left pad and the last column's right pad so the outer
    // edges carry no separator padding — interior cells keep `(1,1)` for the ` │ `
    // gap. (comfy-table still fills each cell to its column width, so the last
    // column's short cells acquire trailing fill; the per-line right-trim below
    // removes it, reproducing the old renderer's never-pad-the-last-column property
    // and guaranteeing NO trailing whitespace on any line — SL-053 D7.)
    let last = width.saturating_sub(1);
    for (index, column) in table.column_iter_mut().enumerate() {
        let left = u16::from(index != 0);
        let right = u16::from(index != last);
        column.set_padding((left, right));
    }

    let rendered = table.to_string();
    let mut out = String::with_capacity(rendered.len() + 1);
    for line in rendered.lines() {
        out.push_str(line.trim_end());
        out.push('\n');
    }
    out
}

/// Pure structural minimum (in columns) for a `cols`-wide minimalist `│` grid below
/// which `Dynamic` wrapping degrades to 1-char slivers, so `Disabled` (clean
/// overflow) is the better fallback. Derived to AGREE with comfy-table 7.2.2's
/// `available_content_width` accounting (`dynamic.rs`) for the EXACT style this body
/// sets — `set_width(w)` is the total table width, from which comfy subtracts:
///
/// - **borders** (`count_border_columns`): left + right outer borders are removed by
///   the component strip → 0; only the interior `VerticalLines` remain → `cols - 1`.
/// - **padding**: measured at Display time AFTER the outer-edge zeroing loop, so the
///   first column's left pad and the last column's right pad are already 0; every
///   other side is 1. Total = `2·cols` sides minus the two zeroed = `2·cols - 2`.
/// - **content**: ≥1 display char per visible column to render at all → `cols`.
///
/// Sum: `(cols - 1) + (2·cols - 2) + cols = 4·cols - 3`. At/above this, the Dynamic
/// budget seats ≥1 content char per column; below it, comfy saturating-subtracts
/// toward 0. (`cols == 0` cannot reach here — the `rows.is_empty()` guard returns
/// first, and a non-empty grid has `cols ≥ 1`.) Pinned by VT-2's boundary test.
fn grid_min_width(cols: usize) -> usize {
    (cols * 4).saturating_sub(3)
}

/// Wrap kind-faithful row values in the shared envelope: `{ "kind": …, "rows":
/// [ … ] }`. Each kind owns its row serde shape (a faithful mirror, D7); this
/// just supplies the uniform outer frame.
pub(crate) fn json_envelope<T: Serialize>(kind: &str, rows: &[T]) -> anyhow::Result<String> {
    let envelope = serde_json::json!({ "kind": kind, "rows": rows });
    serde_json::to_string_pretty(&envelope).context("failed to serialize list JSON envelope")
}

/// Strip SGR escape sequences (`ESC [ … m`) for the VT-2 alignment proof. Pure
/// test helper, crate-visible so the backlog tags-colour proof shares it — not a
/// render-path concern.
#[cfg(test)]
pub(crate) fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            // Consume up to and including the final `m` of an SGR sequence.
            for inner in chars.by_ref() {
                if inner == 'm' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- canonical_id ------------------------------------------------------

    #[test]
    fn canonical_id_prefixes_and_zero_pads_to_three() {
        assert_eq!(canonical_id("SL", 25), "SL-025");
        assert_eq!(canonical_id("ADR", 1), "ADR-001");
        assert_eq!(canonical_id("PRD", 123), "PRD-123");
        // four+ digits are not truncated — padding is a minimum, not a cap.
        assert_eq!(canonical_id("REQ", 1234), "REQ-1234");
    }

    // -- Format from_str / Display ----------------------------------------

    #[test]
    fn format_parses_table_and_json() {
        assert_eq!(Format::from_str("table").unwrap(), Format::Table);
        assert_eq!(Format::from_str("json").unwrap(), Format::Json);
    }

    #[test]
    fn format_rejects_unknown_value() {
        let err = Format::from_str("yaml").unwrap_err().to_string();
        assert!(err.contains("yaml"), "error names the bad value: {err}");
    }

    #[test]
    fn format_display_round_trips_from_str() {
        for f in [Format::Table, Format::Json] {
            assert_eq!(Format::from_str(&f.to_string()).unwrap(), f);
        }
    }

    // -- build: --json precedence + regex compile -------------------------

    /// Build with everything at its default — the no-constraint filter.
    fn no_filter() -> (Filter, Format) {
        build(ListArgs::default()).unwrap()
    }

    #[test]
    fn build_json_flag_forces_json_over_format_table() {
        // A-9: --json wins over --format table, no error.
        let (_f, fmt) = build(ListArgs {
            format: Format::Table,
            json: true,
            ..Default::default()
        })
        .unwrap();
        assert_eq!(fmt, Format::Json);
    }

    #[test]
    fn build_without_json_flag_honours_format() {
        let (_f, fmt) = build(ListArgs {
            format: Format::Json,
            ..Default::default()
        })
        .unwrap();
        assert_eq!(fmt, Format::Json);
    }

    #[test]
    fn build_lowercases_the_substring_once() {
        let (f, _) = build(ListArgs {
            substr: Some("HeLLo".into()),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(f.substr.as_deref(), Some("hello"));
    }

    #[test]
    fn build_compiles_a_valid_regex() {
        let (f, _) = build(ListArgs {
            regexp: Some("^SL-".into()),
            ..Default::default()
        })
        .unwrap();
        assert!(f.regex.is_some());
    }

    #[test]
    fn build_invalid_regex_is_a_clean_error_not_a_panic() {
        let err = build(ListArgs {
            regexp: Some("(unclosed".into()),
            ..Default::default()
        })
        .unwrap_err()
        .to_string();
        assert!(err.contains("invalid regex"), "got: {err}");
    }

    #[test]
    fn build_case_insensitive_bakes_the_flag_into_the_pattern() {
        let (f, _) = build(ListArgs {
            regexp: Some("sl".into()),
            case_insensitive: true,
            ..Default::default()
        })
        .unwrap();
        let re = f.regex.unwrap();
        assert!(
            re.is_match("SL-025"),
            "case-insensitive should match uppercase"
        );
    }

    // -- retain matrix -----------------------------------------------------

    /// A test row carrying everything `retain` projects.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Row {
        canonical: &'static str,
        slug: &'static str,
        title: &'static str,
        status: &'static str,
        tags: Vec<&'static str>,
    }

    fn row(
        canonical: &'static str,
        slug: &'static str,
        title: &'static str,
        status: &'static str,
        tags: &[&'static str],
    ) -> Row {
        Row {
            canonical,
            slug,
            title,
            status,
            tags: tags.to_vec(),
        }
    }

    fn fields(r: &Row) -> FilterFields {
        FilterFields {
            canonical: r.canonical.to_string(),
            slug: r.slug.to_string(),
            title: r.title.to_string(),
            status: r.status.to_string(),
            tags: r.tags.iter().map(|t| (*t).to_string()).collect(),
        }
    }

    /// The slice hide-set, for the hide-set arm of the matrix.
    fn hidden(status: &str) -> bool {
        matches!(status, "done" | "abandoned")
    }

    fn never_hidden(_: &str) -> bool {
        false
    }

    fn sample() -> Vec<Row> {
        vec![
            row("SL-001", "alpha-thing", "Alpha Thing", "proposed", &["x"]),
            row("SL-002", "beta-widget", "Beta Widget", "done", &["y"]),
            row(
                "SL-003",
                "gamma-gadget",
                "Gamma Gadget",
                "started",
                &["x", "z"],
            ),
            row(
                "SL-004",
                "delta-doohickey",
                "Delta Doohickey",
                "abandoned",
                &["y"],
            ),
        ]
    }

    fn canonicals(rows: &[Row]) -> Vec<&str> {
        rows.iter().map(|r| r.canonical).collect()
    }

    #[test]
    fn retain_default_hides_the_hide_set() {
        let (f, _) = no_filter();
        let kept = retain(sample(), &f, hidden, fields);
        // done (SL-002) + abandoned (SL-004) dropped by default.
        assert_eq!(canonicals(&kept), vec!["SL-001", "SL-003"]);
    }

    #[test]
    fn retain_all_reveals_the_hide_set() {
        let (f, _) = build(ListArgs {
            all: true,
            ..Default::default()
        })
        .unwrap();
        let kept = retain(sample(), &f, hidden, fields);
        assert_eq!(
            canonicals(&kept),
            vec!["SL-001", "SL-002", "SL-003", "SL-004"]
        );
    }

    #[test]
    fn retain_explicit_status_reveals_the_hide_set() {
        // --status done reveals the otherwise-hidden done row.
        let (f, _) = build(ListArgs {
            status: vec!["done".into()],
            ..Default::default()
        })
        .unwrap();
        let kept = retain(sample(), &f, hidden, fields);
        assert_eq!(canonicals(&kept), vec!["SL-002"]);
    }

    #[test]
    fn retain_multi_status_is_or_within_the_axis() {
        let (f, _) = build(ListArgs {
            status: vec!["proposed".into(), "started".into()],
            ..Default::default()
        })
        .unwrap();
        let kept = retain(sample(), &f, hidden, fields);
        assert_eq!(canonicals(&kept), vec!["SL-001", "SL-003"]);
    }

    #[test]
    fn retain_substr_matches_slug_or_title_case_insensitively() {
        let (f, _) = build(ListArgs {
            substr: Some("WIDGET".into()),
            all: true,
            ..Default::default()
        })
        .unwrap();
        let kept = retain(sample(), &f, hidden, fields);
        assert_eq!(canonicals(&kept), vec!["SL-002"]);
    }

    #[test]
    fn retain_tag_is_or_and_ands_with_other_axes() {
        // tag x (alpha + gamma); default hide-set drops nothing here.
        let (f, _) = build(ListArgs {
            tags: vec!["x".into()],
            ..Default::default()
        })
        .unwrap();
        let kept = retain(sample(), &f, hidden, fields);
        assert_eq!(canonicals(&kept), vec!["SL-001", "SL-003"]);
    }

    #[test]
    fn retain_axes_and_across_each_other() {
        // substr "a" matches all four slugs/titles; tag y narrows to beta+delta;
        // both are hide-set (done/abandoned) but tag isn't a status so the
        // hide-set still applies (no --all / --status) → empty.
        let (f, _) = build(ListArgs {
            substr: Some("a".into()),
            tags: vec!["y".into()],
            ..Default::default()
        })
        .unwrap();
        let kept = retain(sample(), &f, hidden, fields);
        assert!(
            kept.is_empty(),
            "hide-set still drops done+abandoned: {:?}",
            canonicals(&kept)
        );
    }

    #[test]
    fn retain_regex_over_canonical() {
        let (f, _) = build(ListArgs {
            regexp: Some("SL-00[13]".into()),
            all: true,
            ..Default::default()
        })
        .unwrap();
        let kept = retain(sample(), &f, hidden, fields);
        assert_eq!(canonicals(&kept), vec!["SL-001", "SL-003"]);
    }

    #[test]
    fn retain_preserves_input_order_never_sorts() {
        // Intentionally unsorted; retain must not reorder (ordering is per-kind).
        let rows = vec![
            row("SL-003", "c", "C", "proposed", &[]),
            row("SL-001", "a", "A", "proposed", &[]),
            row("SL-002", "b", "B", "proposed", &[]),
        ];
        let (f, _) = no_filter();
        let kept = retain(rows, &f, never_hidden, fields);
        assert_eq!(canonicals(&kept), vec!["SL-003", "SL-001", "SL-002"]);
    }

    // -- VT-2 domain distinction ------------------------------------------

    #[test]
    fn retain_regex_matches_canonical_only_keeps_the_row() {
        // canonical matches the regex; slug+title do NOT — proves regex searches
        // the canonical domain (A-1 guard).
        let rows = vec![row(
            "SL-042",
            "nomatch-slug",
            "Nomatch Title",
            "proposed",
            &[],
        )];
        let (f, _) = build(ListArgs {
            regexp: Some("SL-042".into()),
            all: true,
            ..Default::default()
        })
        .unwrap();
        let kept = retain(rows, &f, never_hidden, fields);
        assert_eq!(canonicals(&kept), vec!["SL-042"]);
    }

    #[test]
    fn retain_substr_does_not_search_canonical() {
        // substr "042" appears in canonical but NOT in slug/title → dropped.
        // Proves the substr domain excludes canonical (the other half of A-1).
        let rows = vec![row(
            "SL-042",
            "nomatch-slug",
            "Nomatch Title",
            "proposed",
            &[],
        )];
        let (f, _) = build(ListArgs {
            substr: Some("042".into()),
            all: true,
            ..Default::default()
        })
        .unwrap();
        let kept = retain(rows, &f, never_hidden, fields);
        assert!(
            kept.is_empty(),
            "substr must not see canonical: {:?}",
            canonicals(&kept)
        );
    }

    #[test]
    fn retain_substr_only_match_is_kept() {
        // slug matches the substr; canonical does not contain it → kept via the
        // substr domain (independent of regex, which is absent).
        let rows = vec![row("SL-042", "special-slug", "Title", "proposed", &[])];
        let (f, _) = build(ListArgs {
            substr: Some("special".into()),
            all: true,
            ..Default::default()
        })
        .unwrap();
        let kept = retain(rows, &f, never_hidden, fields);
        assert_eq!(canonicals(&kept), vec!["SL-042"]);
    }

    // -- validate_statuses -------------------------------------------------

    #[test]
    fn validate_statuses_accepts_known_values() {
        let known = ["proposed", "ready", "done"];
        assert!(validate_statuses(&["proposed".into(), "done".into()], &known).is_ok());
        assert!(validate_statuses(&[], &known).is_ok());
    }

    #[test]
    fn validate_statuses_rejects_an_unknown_value_naming_it_and_the_set() {
        let known = ["proposed", "ready", "done"];
        let err = validate_statuses(&["bogus".into()], &known)
            .unwrap_err()
            .to_string();
        assert!(err.contains("bogus"), "names the bad value: {err}");
        assert!(err.contains("proposed"), "lists the known set: {err}");
    }

    // -- render_table (relocated; behaviour preserved) --------------------

    fn cells(cells: &[&str]) -> Vec<String> {
        cells.iter().map(|s| (*s).to_string()).collect()
    }

    // VT-4: empty grid → "" (header suppressed on an empty list).
    #[test]
    fn render_table_empty_is_empty_string() {
        assert_eq!(render_table(&[], None), "");
    }

    // VT-1 (no-ANSI + idempotence). `force_no_tty()` is UNCONDITIONAL inside
    // render_table, so byte-stability terminal-vs-pipe (SL-053 D6) is a structural
    // property of the code, NOT something this in-process test can toggle or prove —
    // there is no tty arm to flip, and a pty is out of scope. What this test pins is
    // the two observable consequences: (a) the output carries no ANSI escape
    // (force_no_tty + Disabled suppress comfy-table's own styling), and (b) the pure
    // fn is idempotent. The live terminal-vs-pipe equivalence rests on force_no_tty
    // being unconditional, asserted by structure rather than exercised here.
    #[test]
    fn render_table_is_deterministic_and_carries_no_ansi() {
        let grid = [
            cells(&["id", "kind", "status", "title"]),
            cells(&["SL-001", "slice", "proposed", "Alpha"]),
        ];
        let first = render_table(&grid, None);
        let second = render_table(&grid, None);
        assert_eq!(first, second, "render_table must be byte-stable");
        assert!(
            !first.contains('\u{1b}'),
            "force_no_tty must suppress ANSI styling: {first:?}"
        );
    }

    // VT-2 (exact line shape): every line has NO leading space, NO trailing
    // whitespace, and interior separators are ` │ `.
    #[test]
    fn render_table_line_shape_minimalist_vertical_separators() {
        let out = render_table(
            &[
                cells(&["id", "kind", "status", "title"]),
                cells(&["SL-001", "slice", "proposed", "Alpha Thing"]),
            ],
            None,
        );
        assert_eq!(
            out,
            "id     │ kind  │ status   │ title\n\
             SL-001 │ slice │ proposed │ Alpha Thing\n"
        );
        for line in out.lines() {
            assert!(
                !line.starts_with(' '),
                "no leading space on a line: {line:?}"
            );
            assert_eq!(
                line.trim_end(),
                line,
                "no trailing whitespace on a line: {line:?}"
            );
            assert!(
                line.contains(" │ "),
                "interior separator is ` │ `: {line:?}"
            );
        }
    }

    // VT-2 (the slice-list middle-column case): a wider middle column pads cleanly
    // and the last column stays edge-clean.
    #[test]
    fn render_table_aligns_a_middle_column_the_slice_case() {
        let out = render_table(
            &[
                cells(&["001", "done", "4/6", "memory-entity-v1", "Memory entity v1"]),
                cells(&[
                    "009",
                    "proposed",
                    "—",
                    "slice-status-rollup",
                    "Slice status rollup",
                ]),
            ],
            None,
        );
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(
            lines[0],
            "001 │ done     │ 4/6 │ memory-entity-v1    │ Memory entity v1"
        );
        assert_eq!(
            lines[1],
            "009 │ proposed │ —   │ slice-status-rollup │ Slice status rollup"
        );
    }

    // -- SL-054 PHASE-02: terminal-width wrapping -------------------------

    /// VT-2 (the structural floor): `grid_min_width` is the EXACT comfy-table
    /// accounting — borders (`cols-1`) + zeroed-edge padding (`2·cols-2`) + ≥1
    /// content char per column (`cols`) = `4·cols - 3`. Pins the closed form against
    /// drift so the boundary test below rests on a derived value, not a magic number.
    #[test]
    fn grid_min_width_is_the_derived_comfy_accounting() {
        assert_eq!(grid_min_width(1), 1, "1 col: (0 borders)+(0 pad)+(1 char)");
        assert_eq!(grid_min_width(2), 5, "2 col: 1+2+2");
        assert_eq!(grid_min_width(4), 13, "4 col: 3+6+4");
        assert_eq!(grid_min_width(6), 21, "6 col: 5+10+6");
    }

    /// VT-1: a wide cell under a budget narrower than its content wraps to multiple
    /// lines; EVERY rendered line still carries the `│` separator, and no line exceeds
    /// the budget once `trim_end` has run.
    #[test]
    fn render_table_some_width_wraps_a_wide_cell_keeping_the_separator() {
        let budget = 40u16;
        let out = render_table(
            &[
                cells(&["id", "description"]),
                cells(&[
                    "SL-001",
                    "a deliberately long description that cannot fit inside forty columns and must wrap",
                ]),
            ],
            Some(budget),
        );
        let lines: Vec<&str> = out.lines().collect();
        assert!(
            lines.len() > 2,
            "the wide cell must wrap to extra lines, got {lines:?}"
        );
        for line in &lines {
            assert!(
                line.contains('\u{2502}'),
                "every wrapped line keeps the `│` separator: {line:?}"
            );
            assert!(
                line.chars().count() <= usize::from(budget),
                "no line exceeds the budget after trim_end: {line:?}"
            );
        }
    }

    /// VT-2 (the boundary): at `Some(w)` BELOW `grid_min_width` the render is exactly
    /// the `None`/Disabled byte-output (grid-floor fallback, no sliver wrapping); at
    /// `Some(w)` at/above the floor a too-wide cell DOES wrap (extra lines). Pins the
    /// structural boundary at the derived value.
    #[test]
    fn render_table_grid_floor_falls_back_below_and_wraps_at_or_above() {
        // A 6-column grid with one over-wide cell. grid_min_width(6) == 21.
        let grid = [
            cells(&["a", "b", "c", "d", "e", "f"]),
            cells(&[
                "1",
                "2",
                "3",
                "4",
                "5",
                "a wide trailing cell that would wrap given any real budget",
            ]),
        ];
        let floor = grid_min_width(6);
        assert_eq!(floor, 21, "the 6-col floor is the derived 4·6-3");

        // Below the floor: identical to None/Disabled, byte-for-byte (no wrapping).
        let disabled = render_table(&grid, None);
        let below = render_table(&grid, Some(20));
        assert_eq!(
            below, disabled,
            "Some(w) below the grid floor must equal the Disabled output"
        );
        assert!(
            u16::try_from(floor).is_ok_and(|f| f > 20),
            "20 is genuinely below the floor"
        );

        // At the floor: the over-wide cell wraps (more lines than the unwrapped two).
        let at_floor = render_table(&grid, Some(u16::try_from(floor).expect("floor fits u16")));
        assert!(
            at_floor.lines().count() > disabled.lines().count(),
            "at the floor the wide cell wraps: {at_floor:?}"
        );
    }

    /// VT-3 (first ByValue-under-wrap coverage, RSK-3): a coloured `ByValue`-painted
    /// wide cell rendered through `render_columns` with `term_width: Some(w)` wraps,
    /// and stripping the ANSI reproduces the EXACT plain wrapped layout — colour
    /// survives wrapping (trim_end strips spaces after owo resets, not the resets).
    #[test]
    fn render_columns_byvalue_wide_cell_wraps_and_colour_strips_to_plain() {
        struct WRow {
            status: &'static str,
            note: &'static str,
        }
        let columns: [Column<WRow>; 2] = [
            Column {
                name: "status",
                header: "status",
                cell: |r| r.status.to_string(),
                paint: ColumnPaint::ByValue(|r| status_hue(r.status)),
            },
            Column {
                name: "note",
                header: "note",
                cell: |r| r.note.to_string(),
                paint: ColumnPaint::None,
            },
        ];
        let rows = [WRow {
            status: "blocked",
            note: "a long trailing note that overflows a narrow budget and must wrap onto several lines",
        }];
        let sel = select_columns(&columns, &["status", "note"], None).unwrap();
        let width = Some(40u16);

        let plain = render_columns(
            &rows,
            &sel,
            RenderOpts {
                color: false,
                term_width: width,
            },
        );
        let coloured = render_columns(
            &rows,
            &sel,
            RenderOpts {
                color: true,
                term_width: width,
            },
        );

        assert!(
            plain.lines().count() > 2,
            "the wide note wraps under the budget: {plain:?}"
        );
        assert!(
            coloured.contains('\u{1b}'),
            "the ByValue-painted status cell carries ANSI under wrapping"
        );
        assert_eq!(
            strip_ansi(&coloured),
            plain,
            "stripping ANSI from the wrapped coloured render reproduces the plain layout"
        );
    }

    // VT-4 (trailing newline): non-empty output ends in EXACTLY one newline.
    #[test]
    fn render_table_non_empty_ends_in_exactly_one_newline() {
        let out = render_table(&[cells(&["a", "b"]), cells(&["c", "d"])], None);
        assert!(out.ends_with('\n'), "ends in a newline: {out:?}");
        assert!(
            !out.ends_with("\n\n"),
            "exactly one trailing newline: {out:?}"
        );
    }

    // VT-3 (rectangular-grid guard, F-2): a ragged grid must fail LOUDLY rather
    // than mis-render. The old hand-rolled renderer silently tolerated raggedness;
    // comfy-table is the sole authority now, so we pin the rectangularity invariant
    // with a debug_assert that panics on a ragged grid (debug builds — tests run
    // debug).
    #[test]
    #[should_panic(expected = "rectangular")]
    fn render_table_ragged_grid_panics_loudly() {
        let _ = render_table(&[cells(&["a", "b", "c"]), cells(&["short", "row"])], None);
    }

    // -- select_columns ----------------------------------------------------

    /// A minimal row for column-model tests.
    struct CRow {
        id: &'static str,
        slug: &'static str,
    }

    /// The available set for [`CRow`]: id, slug.
    const CROW_COLUMNS: [Column<CRow>; 2] = [
        Column {
            name: "id",
            header: "id",
            cell: |r| r.id.to_string(),
            paint: ColumnPaint::Fixed(owo_colors::DynColors::Ansi(owo_colors::AnsiColors::Cyan)),
        },
        Column {
            name: "slug",
            header: "slug",
            cell: |r| r.slug.to_string(),
            paint: ColumnPaint::None,
        },
    ];

    fn names<R>(sel: &[&Column<R>]) -> Vec<&'static str> {
        sel.iter().map(|c| c.name).collect()
    }

    fn req(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn select_columns_none_takes_the_default_verbatim() {
        let sel = select_columns(&CROW_COLUMNS, &["id"], None).unwrap();
        assert_eq!(names(&sel), ["id"]);
    }

    #[test]
    fn select_columns_requested_subset_and_order_win() {
        let sel = select_columns(&CROW_COLUMNS, &["id", "slug"], Some(&req(&["slug"]))).unwrap();
        assert_eq!(names(&sel), ["slug"]);
        let sel =
            select_columns(&CROW_COLUMNS, &["id", "slug"], Some(&req(&["slug", "id"]))).unwrap();
        assert_eq!(names(&sel), ["slug", "id"]);
    }

    #[test]
    fn select_columns_duplicates_are_permitted() {
        let sel = select_columns(&CROW_COLUMNS, &["id"], Some(&req(&["id", "id"]))).unwrap();
        assert_eq!(names(&sel), ["id", "id"]);
    }

    #[test]
    fn select_columns_unknown_name_is_one_uniform_error_listing_available() {
        // .err(): Result::unwrap_err would demand Debug on Vec<&Column<R>>,
        // and deriving Debug on Column would add a spurious R: Debug bound.
        let err = select_columns(&CROW_COLUMNS, &["id"], Some(&req(&["bogus"])))
            .err()
            .map(|e| e.to_string())
            .unwrap();
        assert!(err.contains("unknown column `bogus`"), "names it: {err}");
        assert!(err.contains("id, slug"), "lists the available set: {err}");
    }

    #[test]
    fn select_columns_empty_available_rejects_any_request() {
        let none: [Column<CRow>; 0] = [];
        let err = select_columns(&none, &[], Some(&req(&["id"])))
            .err()
            .map(|e| e.to_string())
            .unwrap();
        assert!(err.contains("unknown column `id`"), "got: {err}");
        // The empty default over an empty available set is the benign case.
        assert!(select_columns(&none, &[], None).unwrap().is_empty());
    }

    /// IMP-038: debug_assert! fires when a default column name is not in the
    /// available set. The assert is only active in debug builds (#[cfg(debug_assertions)]).
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "default column")]
    fn select_columns_panics_on_invalid_default_in_debug() {
        let _ = select_columns(&CROW_COLUMNS, &["bogus"], None);
    }

    /// IMP-038: debug_assert! does NOT fire when all defaults are valid.
    #[test]
    fn select_columns_valid_defaults_pass() {
        let result = select_columns(&CROW_COLUMNS, &["id"], None);
        assert!(result.is_ok());
    }

    // -- RenderOpts --------------------------------------------------------

    // VT-2: the default render-axis bundle is the plain, unwrapped path — no colour,
    // no terminal-width wrapping. Every `..Default::default()` caller rides this.
    #[test]
    fn render_opts_default_is_plain_unwrapped() {
        let opts = RenderOpts::default();
        assert!(!opts.color, "default is colourless");
        assert_eq!(
            opts.term_width, None,
            "default is unwrapped (no term width)"
        );
    }

    // -- render_columns ----------------------------------------------------

    #[test]
    fn render_columns_empty_rows_is_empty_string_header_suppressed() {
        let sel = select_columns(&CROW_COLUMNS, &["id", "slug"], None).unwrap();
        assert_eq!(render_columns::<CRow>(&[], &sel, RenderOpts::default()), "");
    }

    #[test]
    fn render_columns_emits_header_row_then_cells_via_render_table() {
        let rows = [
            CRow {
                id: "ADR-001",
                slug: "module-layering",
            },
            CRow {
                id: "ADR-004",
                slug: "relations",
            },
        ];
        let sel = select_columns(&CROW_COLUMNS, &["id", "slug"], None).unwrap();
        let out = render_columns(&rows, &sel, RenderOpts::default());
        assert_eq!(
            out,
            "id      │ slug\nADR-001 │ module-layering\nADR-004 │ relations\n"
        );
    }

    // -- VT-1 / VT-2: colour seam -----------------------------------------

    /// VT-1: `color = true` emits ANSI for painted columns AND a bold header;
    /// `color = false` emits ZERO ANSI (no `\x1b` anywhere) — the byte-clean piped
    /// path the goldens depend on.
    #[test]
    fn render_columns_colour_emits_ansi_only_when_enabled() {
        let rows = [
            CRow {
                id: "ADR-001",
                slug: "module-layering",
            },
            CRow {
                id: "ADR-004",
                slug: "relations",
            },
        ];
        let sel = select_columns(&CROW_COLUMNS, &["id", "slug"], None).unwrap();

        let plain = render_columns(&rows, &sel, RenderOpts::default());
        assert!(
            !plain.contains('\u{1b}'),
            "color=false must emit zero ANSI: {plain:?}"
        );

        let coloured = render_columns(
            &rows,
            &sel,
            RenderOpts {
                color: true,
                ..Default::default()
            },
        );
        assert!(
            coloured.contains('\u{1b}'),
            "color=true must emit ANSI for the painted id column + bold header"
        );
        // The bold-header SGR (ESC[1m) is present — the header is painted, not just
        // the data cells.
        assert!(
            coloured.contains("\u{1b}[1m"),
            "color=true bolds the header: {coloured:?}"
        );
    }

    /// VT-2: alignment survives colour. With ANSI embedded in a painted column,
    /// stripping the escapes must reproduce the EXACT plain layout — proving
    /// comfy-table's custom_styling measures display width, not byte length, so the
    /// ` │ ` separators stay aligned despite the escapes.
    #[test]
    fn render_columns_colour_keeps_separators_aligned() {
        let rows = [
            CRow {
                id: "ADR-001",
                slug: "module-layering",
            },
            CRow {
                id: "ADR-004",
                slug: "relations",
            },
        ];
        let sel = select_columns(&CROW_COLUMNS, &["id", "slug"], None).unwrap();
        let plain = render_columns(&rows, &sel, RenderOpts::default());
        let coloured = render_columns(
            &rows,
            &sel,
            RenderOpts {
                color: true,
                ..Default::default()
            },
        );
        assert_eq!(
            strip_ansi(&coloured),
            plain,
            "stripping ANSI from the coloured render must reproduce the plain layout"
        );
    }

    // -- status_hue + ByValue paint path ----------------------------------
    // The one branching decision in the colour seam. Previously unexercised:
    // CROW_COLUMNS paints only `id: Fixed` + `slug: None`, so no test ever called
    // a ByValue fn or status_hue. These pin the token→hue table against drift and
    // the `paint_cell` ByValue dispatch.

    /// Each hue class maps the tokens a painted surface can emit; everything else
    /// (incl. the deliberately-unmapped phase status `in_progress`) is `None`.
    #[test]
    fn status_hue_maps_each_class_and_leaves_the_rest_uncoloured() {
        use owo_colors::{
            AnsiColors::{Green, Red, Yellow},
            DynColors,
        };
        for green in ["done", "active", "accepted", "required"] {
            assert_eq!(
                status_hue(green),
                Some(DynColors::Ansi(Green)),
                "{green} is settled/green"
            );
        }
        for yellow in ["design", "plan", "ready", "started", "audit", "reconcile"] {
            assert_eq!(
                status_hue(yellow),
                Some(DynColors::Ansi(Yellow)),
                "{yellow} is in-flight/yellow"
            );
        }
        for red in ["blocked", "abandoned", "contested"] {
            assert_eq!(
                status_hue(red),
                Some(DynColors::Ansi(Red)),
                "{red} is stopped/red"
            );
        }
        // Conservative-subset tail + the pre-engagement state stay uncoloured.
        for none in [
            "proposed",
            "open",
            "draft",
            "resolved",
            "closed",
            "superseded",
        ] {
            assert_eq!(status_hue(none), None, "{none} falls through uncoloured");
        }
        // Regression guard: `in_progress` is a phase status (rollup count), never a
        // painted status cell — it must NOT be mapped (was a dead Yellow key).
        assert_eq!(
            status_hue("in_progress"),
            None,
            "in_progress is a phase status, not a painted status cell"
        );
    }

    // -- status_colored --------------------------------------------------

    /// VT-1: mapped status + color:true → ANSI present.
    #[test]
    fn status_colored_mapped_status_with_color_produces_ansi() {
        let s = status_colored("accepted", true);
        assert!(s.contains('\u{1b}'), "accepted+color → ANSI present: {s:?}");
        assert!(
            s.contains("accepted"),
            "the status word is still in there: {s:?}"
        );
    }

    /// VT-2: unmapped status + color:true → plain.
    #[test]
    fn status_colored_unmapped_status_produces_plain() {
        let s = status_colored("bogus", true);
        assert_eq!(s, "bogus");
        assert!(!s.contains('\u{1b}'));
    }

    /// VT-3: any status + color:false → plain.
    #[test]
    fn status_colored_without_color_produces_plain() {
        assert_eq!(status_colored("accepted", false), "accepted");
        assert_eq!(status_colored("bogus", false), "bogus");
        assert_eq!(status_colored("", false), "");
    }

    /// VT-4: the complete status_hue mapped set produces ANSI under color:true.
    #[test]
    fn status_colored_every_mapped_token_produces_ansi() {
        for green in ["done", "active", "accepted", "required"] {
            let s = status_colored(green, true);
            assert!(s.contains('\u{1b}'), "{green} + color → ANSI");
        }
        for yellow in ["design", "plan", "ready", "started", "audit", "reconcile"] {
            let s = status_colored(yellow, true);
            assert!(s.contains('\u{1b}'), "{yellow} + color → ANSI");
        }
        for red in ["blocked", "abandoned", "contested"] {
            let s = status_colored(red, true);
            assert!(s.contains('\u{1b}'), "{red} + color → ANSI");
        }
    }

    /// `paint_cell`'s `ByValue` arm reads the ROW and applies the returned hue;
    /// `None` from the fn (or `color = false`) leaves the cell byte-for-byte raw.
    #[test]
    fn paint_cell_byvalue_reads_row_and_respects_none() {
        let row = CRow {
            id: "SL-001",
            slug: "done",
        };
        // ByValue → Some(hue): coloured, but stripping ANSI reproduces the raw cell.
        let by_status: ColumnPaint<CRow> = ColumnPaint::ByValue(|r| status_hue(r.slug));
        let painted = paint_cell("done", &by_status, &row, true, 0);
        assert!(
            painted.contains('\u{1b}'),
            "ByValue Some hue emits ANSI: {painted:?}"
        );
        assert_eq!(
            strip_ansi(&painted),
            "done",
            "stripped ANSI is the raw cell"
        );

        // ByValue → None: raw, zero ANSI.
        let by_none: ColumnPaint<CRow> = ColumnPaint::ByValue(|_| None);
        assert_eq!(paint_cell("done", &by_none, &row, true, 0), "done");

        // color = false short-circuits before the hue lookup — raw even for Some.
        assert_eq!(paint_cell("done", &by_status, &row, false, 0), "done");
    }

    // -- PerToken paint + tag chips (SL-067 PHASE-02) ---------------------

    /// A row carrying a tag vector, for the PerToken paint path.
    struct TRow {
        tags: Vec<&'static str>,
    }

    fn trow(tags: &[&'static str]) -> TRow {
        TRow {
            tags: tags.iter().copied().collect(),
        }
    }

    /// The tags column wired exactly as `backlog list` wires it: plain `cell` joins by
    /// `", "`, the PerToken paint splits the same tokens and paints each via `paint_tag`.
    fn tags_column() -> Column<TRow> {
        Column {
            name: "tags",
            header: "tags",
            cell: |r| r.tags.join(", "),
            paint: ColumnPaint::PerToken {
                split: |r| r.tags.iter().map(|t| (*t).to_string()).collect(),
                render: paint_tag,
            },
        }
    }

    /// VT-5: `segment_hue` is deterministic and the palette (truecolour gruvbox)
    /// inherently excludes ANSI reserved colours. Palette size ≥ 8.
    #[test]
    fn segment_hue_is_deterministic_and_palette_excludes_reserved() {
        // Determinism: same segment → same hue across calls.
        for seg in ["cli", "command", "security", "a", "longer-segment"] {
            assert_eq!(segment_hue(seg), segment_hue(seg), "{seg} is deterministic");
        }
        // Empty segment → no colour (no text either).
        assert_eq!(segment_hue(""), None, "empty segment is uncoloured");
        // Every non-empty segment lands on a palette hue.
        for seg in ["x", "alpha", "cli", "命"] {
            let hue = segment_hue(seg).expect("non-empty segment is coloured");
            assert!(
                matches!(hue, owo_colors::DynColors::Rgb(..)),
                "{seg} hue {hue:?} is a truecolour palette entry"
            );
        }
        assert!(TAG_PALETTE.len() >= 8, "palette is sufficiently large");
    }

    /// VT-5 / VT-1: `paint_tag` paints each colon-segment with its hue and the `:`
    /// separators WHITE. `cli:command` → two DISTINCT segment hues with a white colon;
    /// `security` → one hue, no colon.
    #[test]
    fn paint_tag_colon_segments_hued_separators_white() {
        let white_colon = {
            use owo_colors::{AnsiColors::White, DynColors, OwoColorize};
            ":".color(DynColors::Ansi(White)).to_string()
        };

        // Single segment: a hue, no colon.
        let single = paint_tag("security");
        assert!(single.contains('\u{1b}'), "single segment is coloured");
        assert!(
            !single.contains(&white_colon),
            "no colon in a single segment"
        );
        assert_eq!(strip_ansi(&single), "security", "stripped is the raw tag");

        // Colon-namespaced: white colon present, two DISTINCT segment hues (cli vs
        // command land on different palette indices — guarded below).
        let chip = paint_tag("cli:command");
        assert_eq!(strip_ansi(&chip), "cli:command", "stripped is the raw tag");
        assert!(chip.contains(&white_colon), "the colon is painted white");
        assert_ne!(
            segment_hue("cli"),
            segment_hue("command"),
            "the fixture's two segments differ in hue (distinct chips)"
        );
    }

    /// VT-1: chip rendering is STABLE across two calls (no RNG, no clock).
    #[test]
    fn paint_tag_is_stable_across_runs() {
        for tag in ["cli:command", "security", "a::b", ":leading"] {
            assert_eq!(paint_tag(tag), paint_tag(tag), "{tag} renders identically");
        }
    }

    /// VT-1: empty segments (`:x`, `a::b`) contribute no text but the white colon still
    /// renders — stripped output is the raw tag including the literal colons.
    #[test]
    fn paint_tag_empty_segments_keep_the_colon() {
        for tag in [":x", "a::b", "trailing:"] {
            assert_eq!(strip_ansi(&paint_tag(tag)), tag, "{tag} stripped is raw");
        }
        // `a::b`: the middle empty segment is no text, but BOTH colons render.
        let chip = paint_tag("a::b");
        let colons = chip.matches('b').count(); // sanity the segment survives
        assert_eq!(colons, 1, "the painted segment text survives: {chip:?}");
    }

    /// VT-2 (the coupling guard — the property, not a proxy): for the tags column over
    /// a fixture with multi-tag, colon-namespaced AND empty-segment rows,
    /// `strip_ansi(paint_cell(color=true)) == paint_cell(color=false) == cell(r)`.
    #[test]
    fn pertoken_byte_clean_coupling_strip_equals_plain_equals_cell() {
        let col = tags_column();
        let rows = [
            trow(&["cli:command", "security"]),
            trow(&["a::b", ":lead", "trail:"]),
            trow(&[]),
            trow(&["solo"]),
        ];
        for r in &rows {
            let plain = paint_cell(&(col.cell)(r), &col.paint, r, false, 0);
            let coloured = paint_cell(&(col.cell)(r), &col.paint, r, true, 0);
            let raw_cell = (col.cell)(r);
            assert_eq!(plain, raw_cell, "color=false is the raw cell extractor");
            assert_eq!(
                strip_ansi(&coloured),
                plain,
                "stripping the coloured PerToken cell reproduces the plain cell"
            );
        }
    }

    /// VT-1: under `color = false` the PerToken arm emits ZERO ANSI and never calls
    /// `render` — the SL-053 plain-path invariant.
    #[test]
    fn pertoken_color_false_emits_zero_ansi() {
        let col = tags_column();
        let r = trow(&["cli:command", "security"]);
        let out = paint_cell(&(col.cell)(&r), &col.paint, &r, false, 0);
        assert!(
            !out.contains('\u{1b}'),
            "color=false PerToken is byte-clean: {out:?}"
        );
        assert_eq!(out, "cli:command, security", "joined by `, ` unchanged");
    }

    /// VT-3: a tags cell emitting MULTIPLE SGR sequences keeps column alignment, and
    /// with the tags column LAST the `render_table` trailing-fill `trim_end` strips only
    /// comfy-table padding, never a chip's trailing `\x1b[0m`. Strip-equals-plain proves
    /// alignment; the reset-survives assertion proves trim_end spared the SGR.
    #[test]
    fn pertoken_multi_sgr_keeps_alignment_and_spares_the_reset() {
        let columns: [Column<TRow>; 2] = [
            Column {
                name: "id",
                header: "id",
                cell: |_| "ITEM".to_string(),
                paint: ColumnPaint::None,
            },
            tags_column(),
        ];
        // tags LAST; the longer-tagged row sets the column width, a shorter cell gets
        // comfy-table trailing fill that trim_end must strip without touching the reset.
        let rows = [trow(&["cli:command", "security"]), trow(&["x"])];
        let sel = select_columns(&columns, &["id", "tags"], None).unwrap();
        let plain = render_columns(&rows, &sel, RenderOpts::default());
        let coloured = render_columns(
            &rows,
            &sel,
            RenderOpts {
                color: true,
                ..Default::default()
            },
        );
        assert!(
            coloured.matches('\u{1b}').count() > 2,
            "the tags cell emits multiple SGR sequences"
        );
        assert_eq!(
            strip_ansi(&coloured),
            plain,
            "multi-SGR tags cell stays column-aligned (display-width measured)"
        );
        // The last cell's chip resets survive trim_end (no trailing whitespace, but the
        // owo reset `\x1b[...m` after the final token is preserved).
        assert!(
            coloured.contains('\u{1b}'),
            "the chip ANSI survives the last-column trim_end"
        );
        for line in coloured.lines() {
            assert_eq!(line.trim_end(), line, "no trailing whitespace: {line:?}");
        }
    }

    // -- json_envelope -----------------------------------------------------

    #[derive(Serialize)]
    struct JsonRow {
        id: &'static str,
        status: &'static str,
    }

    #[test]
    fn json_envelope_wraps_rows_under_kind() {
        let rows = [
            JsonRow {
                id: "SL-001",
                status: "proposed",
            },
            JsonRow {
                id: "SL-002",
                status: "done",
            },
        ];
        let out = json_envelope("slice", &rows).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["kind"], "slice");
        assert_eq!(parsed["rows"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["rows"][0]["id"], "SL-001");
        assert_eq!(parsed["rows"][1]["status"], "done");
    }

    #[test]
    fn json_envelope_empty_rows_is_an_empty_array() {
        let rows: [JsonRow; 0] = [];
        let out = json_envelope("adr", &rows).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed["kind"], "adr");
        assert_eq!(parsed["rows"].as_array().unwrap().len(), 0);
    }

    // -- VT-1, VT-2: Alternate paint variant -------------------------------

    /// VT-1: `paint_cell` with `Alternate` returns the even hue at row 0 and
    /// the odd hue at row 1, both carrying ANSI escapes.
    #[test]
    fn paint_cell_alternate_even_odd_on_row_index() {
        use owo_colors::{
            AnsiColors::{Green, Yellow},
            DynColors,
        };
        let alt: ColumnPaint<CRow> =
            ColumnPaint::Alternate([DynColors::Ansi(Green), DynColors::Ansi(Yellow)]);
        let row = CRow { id: "X", slug: "x" };
        let even = paint_cell("title", &alt, &row, true, 0);
        let odd = paint_cell("title", &alt, &row, true, 1);
        assert!(even.contains('\u{1b}'), "even row 0 carries ANSI: {even:?}");
        assert!(odd.contains('\u{1b}'), "odd row 1 carries ANSI: {odd:?}");
        assert_ne!(even, odd, "even and odd hue differ");
        assert_eq!(strip_ansi(&even), "title", "strip even → raw cell");
        assert_eq!(strip_ansi(&odd), "title", "strip odd → raw cell");
    }

    /// VT-2: `paint_cell` with `Alternate` under `color = false` returns plain
    /// text — no ANSI, passing through the early `!color` return.
    #[test]
    fn paint_cell_alternate_color_false_is_plain() {
        use owo_colors::{AnsiColors::Green, DynColors};
        let alt: ColumnPaint<CRow> =
            ColumnPaint::Alternate([DynColors::Ansi(Green), DynColors::Ansi(Green)]);
        let row = CRow { id: "X", slug: "x" };
        let out = paint_cell("title", &alt, &row, false, 0);
        assert_eq!(out, "title", "color=false Alternate is raw");
        assert!(!out.contains('\u{1b}'), "zero ANSI: {out:?}");
    }

    // -- VT-3, VT-4, VT-5: hue maps --------------------------------------

    /// VT-3: `backlog_kind_hue` maps all 5 known kinds to distinct, stable
    /// `DynColors` per the design table; unknown kind returns `None`.
    #[test]
    fn backlog_kind_hue_maps_all_known_and_none_for_unknown() {
        use owo_colors::{
            AnsiColors::{Blue, Green, Magenta, Red, Yellow},
            DynColors,
        };
        let known: &[(&str, DynColors)] = &[
            ("issue", DynColors::Ansi(Red)),
            ("improvement", DynColors::Ansi(Green)),
            ("chore", DynColors::Ansi(Yellow)),
            ("risk", DynColors::Ansi(Magenta)),
            ("idea", DynColors::Ansi(Blue)),
        ];
        for &(kind, expected) in known {
            assert_eq!(
                backlog_kind_hue(kind),
                Some(expected),
                "{kind} hue mismatch"
            );
        }
        // All 5 kinds MUST map to distinct hues.
        let hues: Vec<_> = known.iter().map(|(_, h)| h).collect();
        for i in 0..hues.len() {
            for j in (i + 1)..hues.len() {
                assert_ne!(hues[i], hues[j], "kinds must have distinct hues");
            }
        }
        assert!(backlog_kind_hue("bogus").is_none(), "unknown kind → None");
    }

    /// VT-4: `memory_type_hue` maps all 6 known memory types to distinct,
    /// stable `DynColors` per the design table; unknown type returns `None`.
    #[test]
    fn memory_type_hue_maps_all_known_and_none_for_unknown() {
        use owo_colors::{
            AnsiColors::{Blue, Cyan, Green, Magenta, Red, Yellow},
            DynColors,
        };
        let known: &[(&str, DynColors)] = &[
            ("concept", DynColors::Ansi(Cyan)),
            ("fact", DynColors::Ansi(Green)),
            ("pattern", DynColors::Ansi(Magenta)),
            ("signpost", DynColors::Ansi(Blue)),
            ("system", DynColors::Ansi(Yellow)),
            ("thread", DynColors::Ansi(Red)),
        ];
        for &(kind, expected) in known {
            assert_eq!(memory_type_hue(kind), Some(expected), "{kind} hue mismatch");
        }
        // All 6 types MUST map to distinct hues.
        let hues: Vec<_> = known.iter().map(|(_, h)| h).collect();
        for i in 0..hues.len() {
            for j in (i + 1)..hues.len() {
                assert_ne!(hues[i], hues[j], "memory types must have distinct hues");
            }
        }
        assert!(memory_type_hue("bogus").is_none(), "unknown type → None");
    }

    /// VT-5: `trust_hue` maps high/medium/low → Green/Yellow/Red; unknown → None.
    #[test]
    fn trust_hue_maps_all_known_and_none_for_unknown() {
        use owo_colors::{
            AnsiColors::{Green, Red, Yellow},
            DynColors,
        };
        assert_eq!(trust_hue("high"), Some(DynColors::Ansi(Green)));
        assert_eq!(trust_hue("medium"), Some(DynColors::Ansi(Yellow)));
        assert_eq!(trust_hue("low"), Some(DynColors::Ansi(Red)));
        assert!(trust_hue("bogus").is_none(), "unknown trust → None");
    }

    // -- VT-7: render_columns with Alternate + Fixed  ---------------------

    /// VT-7b (zebra-pattern guard): 3 data rows with `Alternate` title column —
    /// even-indexed rows share one hue, odd-indexed rows share a different hue,
    /// and the header row carries bold but no alternate hue (it is built
    /// separately by `render_columns` and does not pass through the
    /// `enumerate()` data-row path).
    #[test]
    fn render_columns_alternate_zebra_pattern_on_data_rows_header_excluded() {
        use owo_colors::{
            AnsiColors::{Green, Red},
            DynColors,
        };
        struct ZRow {
            title: &'static str,
        }
        // Use a deliberately garish pair so the per-row ANSI tokens are distinct
        // and matchable — the real TITLE_EVEN/TITLE_ODD are subtle and hard to
        // fingerprint byte-for-byte.
        let columns: [Column<ZRow>; 1] = [Column {
            name: "title",
            header: "title",
            cell: |r| r.title.to_string(),
            paint: ColumnPaint::Alternate([DynColors::Ansi(Green), DynColors::Ansi(Red)]),
        }];
        let rows = [
            ZRow { title: "Row0" },
            ZRow { title: "Row1" },
            ZRow { title: "Row2" },
        ];
        let sel = select_columns(&columns, &["title"], None).unwrap();
        let out = render_columns(
            &rows,
            &sel,
            RenderOpts {
                color: true,
                ..Default::default()
            },
        );
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 4, "header + 3 data rows");

        // Header line: bold escape present, but NO alternate hue escape (the
        // Green SGR emitted by paint_cell for row 0 is `\x1b[32m` — the bold
        // header uses only `\x1b[1m`).
        let header = lines[0];
        assert!(header.contains("\u{1b}[1m"), "header is bold");
        assert!(
            !header.contains("\u{1b}[32m"),
            "header must not carry the Alternate even-hue (row 0) — header is excluded"
        );

        // Data rows: row 0 and row 2 share the even hue (Green → ESC[32m),
        // row 1 gets the odd hue (Red → ESC[31m).
        assert!(
            lines[1].contains("\u{1b}[32m"),
            "data row 0 carries even (Green) hue"
        );
        assert!(
            lines[2].contains("\u{1b}[31m"),
            "data row 1 carries odd (Red) hue"
        );
        assert!(
            lines[3].contains("\u{1b}[32m"),
            "data row 2 carries even (Green) hue — wraps back"
        );
    }

    /// VT-7: `render_columns` with an `Alternate`-painted title column and a
    /// `Fixed`-painted id column renders colour on both; stripping ANSI from
    /// the coloured output reproduces the plain layout.
    #[test]
    fn render_columns_alternate_and_fixed_colours_strip_to_plain() {
        use owo_colors::{AnsiColors::Cyan, DynColors};
        struct ARow {
            id: &'static str,
            title: &'static str,
        }
        let columns: [Column<ARow>; 2] = [
            Column {
                name: "id",
                header: "id",
                cell: |r| r.id.to_string(),
                paint: ColumnPaint::Fixed(DynColors::Ansi(Cyan)),
            },
            Column {
                name: "title",
                header: "title",
                cell: |r| r.title.to_string(),
                paint: ColumnPaint::Alternate([TITLE_EVEN, TITLE_ODD]),
            },
        ];
        let rows = [
            ARow {
                id: "001",
                title: "First",
            },
            ARow {
                id: "002",
                title: "Second",
            },
        ];
        let sel = select_columns(&columns, &["id", "title"], None).unwrap();
        let plain = render_columns(&rows, &sel, RenderOpts::default());
        let coloured = render_columns(
            &rows,
            &sel,
            RenderOpts {
                color: true,
                ..Default::default()
            },
        );
        assert!(coloured.contains('\u{1b}'), "coloured output carries ANSI");
        assert_eq!(
            strip_ansi(&coloured),
            plain,
            "stripping ANSI from coloured render reproduces the plain layout"
        );
    }
}
