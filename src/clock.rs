// SPDX-License-Identifier: GPL-3.0-only
//! The shell time seam — the *only* home of wall-clock reads.
//!
//! Pure/imperative split (slices-spec § Architecture): the pure layers take the
//! date / timestamp as inputs; the CLI shell reads the clock here and passes the
//! value in. Shared by every scaffold verb so there is no parallel date formatter.

use anyhow::Context;

/// Format a `time::Date` as `YYYY-MM-DD` — the single date formatter (the module
/// contract above: no parallel date formatter).
fn fmt_date(d: time::Date) -> String {
    format!("{:04}-{:02}-{:02}", d.year(), u8::from(d.month()), d.day())
}

/// Today as `YYYY-MM-DD` (UTC) — the scaffold date stamp.
pub(crate) fn today() -> String {
    fmt_date(time::OffsetDateTime::now_utc().date())
}

/// `YYYY-MM-DD` (UTC) of a filesystem `SystemTime` — the lazyspec spec-date source
/// (a spec carries no authored date, so its toml mtime is the honest last-changed
/// signal). The mtime read stays in the impure shell; this only formats the value.
pub(crate) fn date_of_system_time(t: std::time::SystemTime) -> String {
    fmt_date(time::OffsetDateTime::from(t).date())
}

/// An RFC3339 UTC timestamp for the runtime progress log.
pub(crate) fn now_timestamp() -> anyhow::Result<String> {
    use time::format_description::well_known::Rfc3339;
    time::OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .context("Failed to format timestamp")
}
