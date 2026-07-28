// SPDX-License-Identifier: GPL-3.0-only
//! Pure filtering, shared lexical matching, deterministic ordering, and
//! keyset cursors (SL-231 PHASE-01, design §4).
//!
//! No clock, RNG, disk, environment, terminal, or MCP imports.

use crate::observation::resolve::Resolution;
use crate::observation::wire::{Envelope, ObservationKind, Payload};
use std::collections::BTreeSet;

// ── Query projection ──────────────────────────────────────────────────────

/// Which view to return from a query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Projection {
    /// Only active (non-corrected) primary observations.
    Active,
    /// All observations including superseded, retracted, and controls.
    History,
}

// ── Filter ────────────────────────────────────────────────────────────────

/// Typed query filter for observations.
#[derive(Debug, Clone, Default)]
pub(crate) struct Filter {
    /// Only return observations of this kind.
    pub(crate) kind: Option<ObservationKind>,
    /// Only return observations with `recorded_at >= from` (inclusive).
    pub(crate) time_from: Option<String>,
    /// Only return observations with `recorded_at < to` (exclusive).
    pub(crate) time_to: Option<String>,
}

impl Filter {
    /// Returns `true` when the envelope matches this filter.
    fn matches(&self, envelope: &Envelope) -> bool {
        if let Some(kind) = self.kind
            && envelope.kind() != kind
        {
            return false;
        }
        if let Some(ref from) = self.time_from
            && envelope.recorded_at.as_str() < from.as_str()
        {
            return false;
        }
        if let Some(ref to) = self.time_to
            && envelope.recorded_at.as_str() >= to.as_str()
        {
            return false;
        }
        true
    }
}

// ── Boolean lexical match ─────────────────────────────────────────────────

/// Returns `true` when every token from the query text appears somewhere
/// in the combined searchable text of the envelope.
///
/// Uses `crate::lexical::tokenize` (the shared tokenizer) for both the
/// query and the envelope text. Matching is Boolean: every query token
/// must be present in the envelope's combined text tokens. The match is
/// unranked.
pub(crate) fn lexical_match(query: &str, envelope: &Envelope) -> bool {
    let query_tokens: BTreeSet<String> = crate::lexical::tokenize(query).into_iter().collect();

    if query_tokens.is_empty() {
        return true; // empty query matches all
    }

    // Collect all searchable text from the envelope
    let mut combined = String::new();
    match &envelope.payload {
        Payload::Friction { summary, detail } => {
            combined.push_str(summary);
            combined.push(' ');
            if let Some(d) = detail {
                combined.push_str(d);
            }
        }
        Payload::Measurement {
            source,
            scope,
            units,
            completeness,
            ..
        } => {
            combined.push_str(source);
            combined.push(' ');
            if let Some(s) = scope {
                combined.push_str(s);
                combined.push(' ');
            }
            if let Some(u) = units {
                combined.push_str(u);
                combined.push(' ');
            }
            if let Some(c) = completeness {
                combined.push_str(c);
            }
        }
        Payload::Supersession { reason, .. } | Payload::Retraction { reason, .. } => {
            if let Some(r) = reason {
                combined.push_str(r);
            }
        }
    }

    // Add facet string values
    if let Some(facets) = envelope.facets.as_ref() {
        for val in facets.string_values() {
            combined.push(' ');
            combined.push_str(val);
        }
    }

    let doc_tokens: BTreeSet<String> = crate::lexical::tokenize(&combined).into_iter().collect();

    // Every query token must appear in the doc tokens
    query_tokens.iter().all(|qt| doc_tokens.contains(qt))
}

// ── Keyset cursor ─────────────────────────────────────────────────────────

/// An opaque keyset cursor for pagination over `(recorded_at desc, uid)`.
///
/// The cursor points to the last returned row; the next page resumes
/// strictly after that position in the total order.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "pagination: PHASE-04 / future use")
)]
pub(crate) struct KeysetCursor {
    /// The `recorded_at` of the last returned row.
    pub(crate) recorded_at: String,
    /// The `uid` of the last returned row.
    pub(crate) uid: String,
}

/// Order envelopes by `(recorded_at desc, uid desc)` — the canonical
/// query order.
fn sort_query_order(envelopes: &mut [&Envelope]) {
    envelopes.sort_by(|a, b| {
        b.recorded_at
            .cmp(&a.recorded_at)
            .then_with(|| b.uid.cmp(&a.uid))
    });
}

/// Return the page of results after `cursor` (exclusive), up to `limit`.
/// A `None` cursor returns the first page.
///
/// Returns the page and the cursor for the next page (if there are more
/// results). The next cursor is `None` when there are no more results.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "pagination: PHASE-04 / future use")
)]
pub(crate) fn paginate<'a>(
    results: &[&'a Envelope],
    cursor: Option<&KeysetCursor>,
    limit: usize,
) -> (Vec<&'a Envelope>, Option<KeysetCursor>) {
    if limit == 0 {
        return (Vec::new(), None);
    }

    // Find the start position: first entry strictly after the cursor
    let start_idx = match cursor {
        None => 0,
        Some(c) => {
            // results are in descending order by (recorded_at, uid)
            // We need the first entry where recorded_at < cursor.recorded_at
            // OR recorded_at == cursor.recorded_at AND uid < cursor.uid
            results
                .iter()
                .position(|e| {
                    (e.recorded_at.as_str() < c.recorded_at.as_str())
                        || (e.recorded_at == c.recorded_at && e.uid.as_str() < c.uid.as_str())
                })
                .unwrap_or(results.len())
        }
    };

    let page: Vec<&Envelope> = results
        .iter()
        .skip(start_idx)
        .take(limit)
        .copied()
        .collect();

    let next_cursor = if start_idx + page.len() < results.len() {
        page.last().map(|last| KeysetCursor {
            recorded_at: last.recorded_at.clone(),
            uid: last.uid.clone(),
        })
    } else {
        None
    };

    (page, next_cursor)
}

// ── Query ─────────────────────────────────────────────────────────────────

/// Run a query against a resolved corpus.
///
/// - `projection` selects between active-only and history views.
/// - `filter` applies optional kind and time-range filters.
/// - `search_text` applies Boolean lexical matching when `Some`.
/// - Results are ordered by `(recorded_at desc, uid desc)`.
///
/// Returns a vector of matched envelopes in canonical query order.
pub(crate) fn query<'a>(
    resolution: &'a Resolution,
    projection: Projection,
    filter: &Filter,
    search_text: Option<&str>,
) -> Vec<&'a Envelope> {
    let candidates: Vec<&Envelope> = match projection {
        Projection::Active => resolution.active(),
        Projection::History => resolution.all_envelopes().collect(),
    };

    let mut matched: Vec<&Envelope> = candidates
        .into_iter()
        .filter(|e| filter.matches(e))
        .filter(|e| {
            if let Some(text) = search_text {
                lexical_match(text, e)
            } else {
                true
            }
        })
        .collect();

    sort_query_order(&mut matched);
    matched
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test code is exempt from panic-family lints"
)]
mod tests {
    use super::*;
    use crate::observation::resolve::resolve;
    use crate::observation::wire::{Envelope, Facets, Payload, SCHEMA, SCHEMA_VERSION};

    fn friction_env(uid: &str, recorded_at: &str, summary: &str) -> Envelope {
        Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: uid.to_string(),
            recorded_at: recorded_at.to_string(),
            facets: None,
            payload: Payload::Friction {
                summary: summary.to_string(),
                detail: None,
            },
        }
    }

    // ── Default projection is active ──────────────────────────────────

    #[test]
    fn query_defaults_to_active_projection() {
        let a = friction_env("a", "2026-01-01T00:00:00Z", "active record");
        let b = friction_env("b", "2026-01-02T00:00:00Z", "superseded record");
        let c = friction_env("c", "2026-01-03T00:00:00Z", "replacement");
        // b → c
        let ss = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: "ss1".to_string(),
            recorded_at: "2026-01-04T00:00:00Z".to_string(),
            facets: None,
            payload: Payload::Supersession {
                old_uid: "b".to_string(),
                replacement_uid: "c".to_string(),
                reason: None,
            },
        };

        let (res, _outcomes) = resolve(vec![a, b, c, ss]);
        let filter = Filter::default();

        // Active projection should exclude superseded b
        let active_results = query(&res, Projection::Active, &filter, None);
        let active_uids: Vec<&str> = active_results.iter().map(|e| e.uid.as_str()).collect();
        assert!(active_uids.contains(&"a"));
        assert!(active_uids.contains(&"c"));
        assert!(
            !active_uids.contains(&"b"),
            "superseded b should be excluded from active"
        );

        // History projection should include b
        let history_results = query(&res, Projection::History, &filter, None);
        let history_uids: Vec<&str> = history_results.iter().map(|e| e.uid.as_str()).collect();
        assert!(
            history_uids.contains(&"b"),
            "history should include superseded b"
        );
    }

    // ── Lexical match uses shared tokenizer ───────────────────────────

    #[test]
    fn search_uses_shared_lexical_tokens() {
        let e = friction_env("a", "2026-01-01T00:00:00Z", "auth token bug encountered");

        // "auth token" both tokens present → match
        assert!(lexical_match("auth token", &e));
        // "Auth" case-folds → match
        assert!(lexical_match("auth", &e));
        // "auth bug" both present → match
        assert!(lexical_match("auth bug", &e));
        // "auth python" → "python" absent → no match
        assert!(!lexical_match("auth python", &e));
        // Empty query → matches all
        assert!(lexical_match("", &e));
        // Punctuation in query tokenizes the same
        assert!(lexical_match("auth.token", &e));
        // Nonexistent token → no match
        assert!(!lexical_match("nonexistent", &e));
    }

    // ── Search over detail and facets ─────────────────────────────────

    #[test]
    fn lexical_match_covers_detail_and_facets() {
        let mut facets = Facets::default();
        facets.execution = Some(crate::observation::wire::ExecutionFacet {
            schema_version: 1,
            interface: Some("cli".to_string()),
            ..Default::default()
        });

        let e = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: "a".to_string(),
            recorded_at: "2026-01-01T00:00:00Z".to_string(),
            facets: Some(facets),
            payload: Payload::Friction {
                summary: "summary text".to_string(),
                detail: Some("detail contains widget".to_string()),
            },
        };

        // "widget" in detail → match
        assert!(lexical_match("widget", &e));
        // "cli" in facets → match
        assert!(lexical_match("cli", &e));
        // "summary" in summary → match
        assert!(lexical_match("summary", &e));
    }

    // ── Kind filter ───────────────────────────────────────────────────

    #[test]
    fn query_kind_filter() {
        let a = friction_env("a", "2026-01-01T00:00:00Z", "friction");
        let b = friction_env("b", "2026-01-02T00:00:00Z", "another");

        let (res, _outcomes) = resolve(vec![a, b]);
        let filter = Filter {
            kind: Some(ObservationKind::Friction),
            ..Default::default()
        };
        let results = query(&res, Projection::Active, &filter, None);
        assert_eq!(results.len(), 2);

        let filter = Filter {
            kind: Some(ObservationKind::Measurement),
            ..Default::default()
        };
        let results = query(&res, Projection::Active, &filter, None);
        assert!(results.is_empty());
    }

    // ── Time range filter ─────────────────────────────────────────────

    #[test]
    fn query_time_range_filter() {
        let a = friction_env("a", "2026-01-01T00:00:00Z", "early");
        let b = friction_env("b", "2026-06-15T00:00:00Z", "middle");
        let c = friction_env("c", "2026-12-31T00:00:00Z", "late");

        let (res, _outcomes) = resolve(vec![a, b, c]);

        // Filter: from 2026-06-01 inclusive
        let filter = Filter {
            time_from: Some("2026-06-01T00:00:00Z".to_string()),
            ..Default::default()
        };
        let results = query(&res, Projection::Active, &filter, None);
        assert_eq!(results.len(), 2);

        // Filter: to 2026-06-01 exclusive
        let filter = Filter {
            time_to: Some("2026-06-01T00:00:00Z".to_string()),
            ..Default::default()
        };
        let results = query(&res, Projection::Active, &filter, None);
        // a (2026-01-01) is before 2026-06-01 → included
        assert_eq!(results.len(), 1);
    }

    // ── Keyset cursor pagination ──────────────────────────────────────

    #[test]
    fn keyset_pagination_no_cursor_returns_first_page() {
        let a = friction_env("a", "2026-01-03T00:00:00Z", "most recent");
        let b = friction_env("b", "2026-01-02T00:00:00Z", "middle");
        let c = friction_env("c", "2026-01-01T00:00:00Z", "oldest");

        let mut results: Vec<&Envelope> = vec![&a, &b, &c];
        sort_query_order(&mut results);
        // descending: a, b, c

        let (page, next) = paginate(&results, None, 2);
        assert_eq!(page.len(), 2);
        assert_eq!(page[0].uid, "a");
        assert_eq!(page[1].uid, "b");
        assert!(next.is_some());

        // Second page
        let (page2, next2) = paginate(&results, next.as_ref(), 2);
        assert_eq!(page2.len(), 1);
        assert_eq!(page2[0].uid, "c");
        assert!(next2.is_none());
    }

    // ── Keyset head insert does not duplicate ─────────────────────────

    #[test]
    fn keyset_head_insert_does_not_duplicate() {
        let a = friction_env("a", "2026-01-03T00:00:00Z", "third");
        let b = friction_env("b", "2026-01-02T00:00:00Z", "second");
        let c = friction_env("c", "2026-01-01T00:00:00Z", "first");

        let mut results: Vec<&Envelope> = vec![&a, &b, &c];
        sort_query_order(&mut results);
        // descending: a, b, c

        // First page of 1
        let (page1, cursor1) = paginate(&results, None, 1);
        assert_eq!(page1.len(), 1);
        assert_eq!(page1[0].uid, "a");
        assert!(cursor1.is_some());

        // Now "insert" a new observation at head: more recent than a
        let head = friction_env("head", "2026-01-04T00:00:00Z", "newest");
        let mut results2: Vec<&Envelope> = vec![&head, &a, &b, &c];
        sort_query_order(&mut results2);
        // descending: head, a, b, c

        // Resume from cursor1 (which was at 'a')
        // The cursor points to 'a', so resume strictly after 'a'
        // After 'a' in desc order: b, c (head is BEFORE a, not after)
        let (page2, _cursor2) = paginate(&results2, cursor1.as_ref(), 2);
        assert_eq!(page2.len(), 2);
        // Must receive b, c — NOT head and NOT a
        let uids: Vec<&str> = page2.iter().map(|e| e.uid.as_str()).collect();
        assert_eq!(uids, vec!["b", "c"]);
        // head should not have been duplicated (it was after cursor in the new results)
    }

    #[test]
    fn keyset_does_not_shift() {
        // Verify that a head insert doesn't shift traversed rows:
        // If we've already seen page 1 (a, b) and a new head is inserted,
        // the second page from the same cursor should still return the same
        // rows as before (c), not skip c.
        let a = friction_env("a", "2026-01-03T00:00:00Z", "third");
        let b = friction_env("b", "2026-01-02T00:00:00Z", "second");
        let c = friction_env("c", "2026-01-01T00:00:00Z", "first");

        let mut results: Vec<&Envelope> = vec![&a, &b, &c];
        sort_query_order(&mut results);

        // First page: a, b
        let (page1, cursor) = paginate(&results, None, 2);
        assert_eq!(page1[0].uid, "a");
        assert_eq!(page1[1].uid, "b");

        // Now insert head
        let head = friction_env("head", "2026-01-04T00:00:00Z", "newest");
        let mut results2: Vec<&Envelope> = vec![&head, &a, &b, &c];
        sort_query_order(&mut results2);

        // Resume from cursor (which was at 'b')
        // Should skip b and return c (one element)
        let (page2, next_cursor) = paginate(&results2, cursor.as_ref(), 2);
        assert_eq!(page2.len(), 1);
        assert_eq!(page2[0].uid, "c");
        assert!(next_cursor.is_none());
    }

    // ── Empty limit ───────────────────────────────────────────────────

    #[test]
    fn paginate_zero_limit_returns_empty() {
        let a = friction_env("a", "2026-01-01T00:00:00Z", "test");
        let results: Vec<&Envelope> = vec![&a];
        let (page, next) = paginate(&results, None, 0);
        assert!(page.is_empty());
        assert!(next.is_none());
    }

    // ── Search + filter combined ──────────────────────────────────────

    #[test]
    fn search_and_filter_combined() {
        let a = friction_env("a", "2026-01-01T00:00:00Z", "auth token bug");
        let b = friction_env("b", "2026-01-02T00:00:00Z", "unrelated thing");

        let (res, _outcomes) = resolve(vec![a, b]);

        let filter = Filter {
            time_from: Some("2026-01-02T00:00:00Z".to_string()),
            ..Default::default()
        };
        let results = query(&res, Projection::Active, &filter, Some("auth"));
        // "auth" only matches a, but a is excluded by time filter
        assert!(results.is_empty());

        let results = query(&res, Projection::Active, &filter, Some("thing"));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].uid, "b");
    }
}
