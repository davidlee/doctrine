// SPDX-License-Identifier: GPL-3.0-only
//! The kind-blind `show` router: resolve a canonical ref to its kind and
//! delegate to that kind's own `run_show`. It renders nothing; the bytes come
//! from the kind (design sec-2, SL-265).

use std::path::PathBuf;

use crate::listing::Format;

/// The resolved kind for a prefix — a closed enum over the 13 handler shapes
/// covering all 24 `kinds::KINDS` rows.
#[derive(Clone, Copy)]
pub(crate) enum Route {
    Slice,
    Adr,
    Policy,
    Standard,
    Rfc,
    Spec,
    Req,
    Knowledge,
    Backlog,
    Review,
    Rec,
    Revision,
    ConceptMap,
}

pub(crate) fn route(prefix: &str) -> Option<Route> {
    use crate::kinds as k;
    match prefix {
        k::SL => Some(Route::Slice),
        k::ADR => Some(Route::Adr),
        k::POL => Some(Route::Policy),
        k::STD => Some(Route::Standard),
        k::RFC => Some(Route::Rfc),
        k::PRD | k::SPEC => Some(Route::Spec),
        k::REQ => Some(Route::Req),
        k::RV => Some(Route::Review),
        k::REC => Some(Route::Rec),
        k::REV => Some(Route::Revision),
        k::CM => Some(Route::ConceptMap),
        _ => {
            // The two family kinds classify through their own single source and
            // the value is discarded — the Route variants carry no payload
            // (knowledge/backlog `run_show` re-resolve their own kind).
            if crate::knowledge::RecordKind::from_prefix(prefix).is_some() {
                Some(Route::Knowledge)
            } else if crate::backlog::kind_from_prefix(prefix).is_some() {
                Some(Route::Backlog)
            } else {
                None
            }
        }
    }
}

/// ASCII-uppercase the prefix of a `PREFIX-NNN` ref (design sec-2 case rule); a
/// bare id passes through unchanged.
fn uppercase_prefix(reference: &str) -> String {
    match reference.split_once('-') {
        Some((prefix, id)) => format!("{}-{id}", prefix.to_ascii_uppercase()),
        None => reference.to_string(),
    }
}

pub(crate) fn run_show(
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
) -> anyhow::Result<()> {
    let root = crate::root::find(path.clone(), &crate::root::default_markers())?;
    let normalised = uppercase_prefix(reference);
    let (kref, id) = crate::kinds::parse_resolvable_ref(&root, &normalised)?;
    let canonical = crate::kinds::canonical_id(kref.kind.prefix, id);
    match route(kref.kind.prefix) {
        Some(route) => dispatch(route, path, &canonical, format),
        // The unrouted-prefix refusal lives HERE, at the dispatch site, never
        // inside `route()` (RV-384 F-16).
        None => anyhow::bail!(
            "`{reference}` names kind prefix `{}`, which the `show` router does not route",
            kref.kind.prefix
        ),
    }
}

fn dispatch(
    route: Route,
    path: Option<PathBuf>,
    reference: &str,
    format: Format,
) -> anyhow::Result<()> {
    match route {
        Route::Slice => crate::slice::run_show(path, reference, format),
        Route::Adr => crate::adr::run_show(path, reference, format),
        Route::Policy => crate::policy::run_show(path, reference, format),
        Route::Standard => crate::standard::run_show(path, reference, format),
        Route::Rfc => crate::rfc::run_show(path, reference, format),
        Route::Spec => crate::spec::run_show(path, reference, format),
        Route::Req => crate::spec::run_req_show(path, reference, format),
        Route::Knowledge => crate::knowledge::run_show(path, reference, format),
        Route::Backlog => crate::backlog::run_show(path, reference, format),
        Route::Review => {
            // `review::run_show` returns a value; the kind's own dispatch prints
            // it via `print_review`. Mirror that exactly (design sec-2).
            use std::io::Write;
            let out = crate::review::run_show(path, reference, format)?;
            write!(std::io::stdout(), "{}", crate::review::print_review(&out))?;
            Ok(())
        }
        Route::Rec => crate::rec::run_show(path, reference, format),
        Route::Revision => crate::revision::run_show(path, reference, format),
        // `concept-map show` defaults edges=false, nodes=false.
        Route::ConceptMap => crate::concept_map::run_show(path, reference, format, false, false),
    }
}

#[cfg(test)]
mod tests {
    use super::route;

    /// RV-384 F-1/F-5: every row of the `KINDS` table (the set
    /// `parse_resolvable_ref`/`kind_by_prefix` resolve against) routes, and a
    /// synthetic unrouted prefix returns `None` (negative control).
    #[test]
    fn every_kinds_row_routes() {
        for kref in crate::kinds::KINDS {
            assert!(
                route(kref.kind.prefix).is_some(),
                "KINDS row `{}` has no `show` route",
                kref.kind.prefix
            );
        }
        assert!(
            route("ZZ").is_none(),
            "the negative control: an unrouted prefix returns None"
        );
    }
}
