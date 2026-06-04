// SPDX-License-Identifier: GPL-3.0-only
//! The shell time seam — the *only* home of wall-clock reads.
//!
//! Pure/imperative split (slices-spec § Architecture): the pure layers take the
//! date / timestamp as inputs; the CLI shell reads the clock here and passes the
//! value in. Shared by every scaffold verb so there is no parallel date formatter.

use anyhow::Context;

/// Today as `YYYY-MM-DD` (UTC) — the scaffold date stamp.
pub(crate) fn today() -> String {
    let d = time::OffsetDateTime::now_utc().date();
    format!("{:04}-{:02}-{:02}", d.year(), u8::from(d.month()), d.day())
}

/// An RFC3339 UTC timestamp for the runtime progress log.
pub(crate) fn now_timestamp() -> anyhow::Result<String> {
    use time::format_description::well_known::Rfc3339;
    time::OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .context("Failed to format timestamp")
}
