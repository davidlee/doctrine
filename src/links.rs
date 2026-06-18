// SPDX-License-Identifier: GPL-3.0-only
//! Pure wikilink extraction/resolution leaf for memory-style references.
//!
//! This module is string-keyed by design: it imports nothing from `memory`,
//! `retrieve`, or any command/engine surface.
use std::collections::{BTreeMap, BTreeSet};

use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Wikilink {
    pub(crate) target: String,
    pub(crate) is_uid: bool,
}

pub(crate) fn extract_wikilinks(body: &str) -> Vec<Wikilink> {
    let Ok(pattern) =
        Regex::new(r"\[\[(mem\.[A-Za-z0-9][A-Za-z0-9._/-]*|mem_[A-Za-z0-9][A-Za-z0-9_-]*)\]\]")
    else {
        return Vec::new();
    };
    let mut in_fence = false;
    let mut links = Vec::new();

    for line in body.lines() {
        if line.contains("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }

        let visible = strip_inline_code(line);
        for captures in pattern.captures_iter(&visible) {
            let target = captures[1].to_string();
            links.push(Wikilink {
                is_uid: target.starts_with("mem_"),
                target,
            });
        }
    }

    links
}

pub(crate) fn resolve_wikilink(
    known_uids: &BTreeSet<String>,
    key_to_uid: &BTreeMap<String, String>,
    target: &str,
    is_uid: bool,
) -> Result<String, String> {
    if is_uid {
        if known_uids.contains(target) {
            Ok(target.to_string())
        } else {
            Err(target.to_string())
        }
    } else if let Some(uid) = key_to_uid.get(target) {
        Ok(uid.clone())
    } else {
        Err(target.to_string())
    }
}

pub(crate) fn backlinks_index<'a>(
    wikilinks_by_uid: BTreeMap<&'a str, Vec<&'a Wikilink>>,
    relations_by_uid: BTreeMap<&'a str, Vec<&'a str>>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut backlinks = BTreeMap::new();

    for (source, links) in wikilinks_by_uid {
        for link in links {
            backlinks
                .entry(link.target.clone())
                .or_insert_with(BTreeSet::new)
                .insert(source.to_string());
        }
    }

    for (source, targets) in relations_by_uid {
        for target in targets {
            backlinks
                .entry((*target).to_string())
                .or_insert_with(BTreeSet::new)
                .insert(source.to_string());
        }
    }

    backlinks
}

fn strip_inline_code(line: &str) -> String {
    let mut out = String::new();
    let mut in_code = false;
    let chars = line.chars();

    for ch in chars {
        if ch == '`' {
            in_code = !in_code;
            continue;
        }
        if !in_code {
            out.push(ch);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::{Wikilink, backlinks_index, extract_wikilinks, resolve_wikilink};
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn vt_1_extracts_mem_key_wikilink() {
        let links = extract_wikilinks("see [[mem.foo]] for details");

        assert_eq!(
            links,
            vec![Wikilink {
                target: "mem.foo".to_string(),
                is_uid: false,
            }]
        );
    }

    #[test]
    fn vt_2_extracts_mem_uid_wikilink() {
        let links = extract_wikilinks("see [[mem_018f3a]] for details");

        assert_eq!(
            links,
            vec![Wikilink {
                target: "mem_018f3a".to_string(),
                is_uid: true,
            }]
        );
    }

    #[test]
    fn vt_3_skips_fenced_code_blocks() {
        let body = "outside [[mem.foo]]\n```\ninside [[mem.bar]]\n```\nafter [[mem.baz]]";

        let links = extract_wikilinks(body);

        assert_eq!(
            links,
            vec![
                Wikilink {
                    target: "mem.foo".to_string(),
                    is_uid: false,
                },
                Wikilink {
                    target: "mem.baz".to_string(),
                    is_uid: false,
                },
            ]
        );
    }

    #[test]
    fn vt_4_skips_inline_code() {
        let links = extract_wikilinks("use `[[mem.foo]]` but keep [[mem.bar]]");

        assert_eq!(
            links,
            vec![Wikilink {
                target: "mem.bar".to_string(),
                is_uid: false,
            }]
        );
    }

    #[test]
    fn vt_5_ignores_non_mem_wikilinks() {
        let links = extract_wikilinks("see [[SL-099]] and [[ADR-004]]");

        assert!(links.is_empty());
    }

    #[test]
    fn vt_6_resolves_known_uid_and_rejects_unknown_uid() {
        let known_uids = BTreeSet::from(["mem_018f3a".to_string()]);
        let key_to_uid = BTreeMap::new();

        assert_eq!(
            resolve_wikilink(&known_uids, &key_to_uid, "mem_018f3a", true),
            Ok("mem_018f3a".to_string())
        );
        assert_eq!(
            resolve_wikilink(&known_uids, &key_to_uid, "mem_deadbeef", true),
            Err("mem_deadbeef".to_string())
        );
    }

    #[test]
    fn vt_7_resolves_key_via_map() {
        let known_uids = BTreeSet::new();
        let key_to_uid = BTreeMap::from([("mem.foo".to_string(), "mem_018f3a".to_string())]);

        assert_eq!(
            resolve_wikilink(&known_uids, &key_to_uid, "mem.foo", false),
            Ok("mem_018f3a".to_string())
        );
    }

    #[test]
    fn vt_8_builds_backlinks_from_wikilinks_and_relations() {
        let link = Wikilink {
            target: "B".to_string(),
            is_uid: true,
        };
        let wikilinks_by_uid = BTreeMap::from([("A", vec![&link])]);
        let relations_by_uid = BTreeMap::from([("C", vec!["B"])]);

        let backlinks = backlinks_index(wikilinks_by_uid, relations_by_uid);

        assert_eq!(
            backlinks.get("B"),
            Some(&BTreeSet::from(["A".to_string(), "C".to_string()]))
        );
    }

    #[test]
    fn vt_9_dedupes_duplicate_backlinks() {
        let link = Wikilink {
            target: "B".to_string(),
            is_uid: true,
        };
        let wikilinks_by_uid = BTreeMap::from([("A", vec![&link])]);
        let relations_by_uid = BTreeMap::from([("A", vec!["B"])]);

        let backlinks = backlinks_index(wikilinks_by_uid, relations_by_uid);

        assert_eq!(backlinks.get("B"), Some(&BTreeSet::from(["A".to_string()])));
    }
}
