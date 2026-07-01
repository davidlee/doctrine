# search_columns type ByValue closure: as_str() already returns &str — extra & produces &&str compile error

When defining a ColumnPaint::ByValue closure that calls a function expecting &str, and the cell data comes from an as_str() that already returns &'static str, the extra & before as_str() produces &&str which Rust rejects. Follow the backlog precedent: backlog_kind_hue(i.kind.as_str()) — no extra &.
