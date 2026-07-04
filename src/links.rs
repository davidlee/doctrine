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

/// Compile the memory-wikilink pattern (`[[mem.key]]` / `[[mem_uid]]`).
/// Single source for extraction and rewriting; returns `None` if the regex
/// fails to compile (unreachable in practice — the literal is constant).
fn wikilink_pattern() -> Option<Regex> {
    Regex::new(r"\[\[(mem\.[A-Za-z0-9][A-Za-z0-9._/-]*|mem_[A-Za-z0-9][A-Za-z0-9_-]*)\]\]").ok()
}

pub(crate) fn extract_wikilinks(body: &str) -> Vec<Wikilink> {
    let Some(pattern) = wikilink_pattern() else {
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

/// Rewrite *resolvable* memory wikilinks into markdown anchors that the map
/// router understands: `[[mem.key]]` → `[Title](#/focus/mem_<uid>)`.
///
/// Unresolvable links are left as the literal `[[target]]` (no dead link).
/// Fenced and inline code are skipped — a `[[mem.x]]` shown *inside* a code
/// sample stays literal, mirroring [`extract_wikilinks`]. The anchor label is
/// `title_by_uid[uid]`, falling back to the raw target when absent.
pub(crate) fn linkify_wikilinks(
    body: &str,
    known_uids: &BTreeSet<String>,
    key_to_uid: &BTreeMap<String, String>,
    title_by_uid: &BTreeMap<String, String>,
) -> String {
    let Some(pattern) = wikilink_pattern() else {
        return body.to_string();
    };
    let ctx = Linkifier {
        pattern,
        known_uids,
        key_to_uid,
        title_by_uid,
    };

    let mut out = String::with_capacity(body.len());
    let mut in_fence = false;
    for line in body.lines() {
        if line.contains("```") {
            in_fence = !in_fence;
            out.push_str(line);
        } else if in_fence {
            out.push_str(line);
        } else {
            ctx.rewrite_line(&mut out, line);
        }
        out.push('\n');
    }
    // `str::lines()` drops a trailing newline; only re-emit it if present.
    if !body.ends_with('\n') {
        out.pop();
    }
    out
}

/// Resolution context for a single linkify pass — the compiled pattern plus the
/// three lookup maps, so they aren't threaded through every helper.
struct Linkifier<'a> {
    pattern: Regex,
    known_uids: &'a BTreeSet<String>,
    key_to_uid: &'a BTreeMap<String, String>,
    title_by_uid: &'a BTreeMap<String, String>,
}

impl Linkifier<'_> {
    /// Rewrite one non-fenced line, replacing wikilinks only outside inline-code
    /// (backtick) spans. Code spans — including their backticks — are verbatim.
    fn rewrite_line(&self, out: &mut String, line: &str) {
        let mut in_code = false;
        let mut segment = String::new();
        for ch in line.chars() {
            if ch == '`' {
                self.flush_segment(out, &segment, in_code);
                segment.clear();
                out.push('`');
                in_code = !in_code;
            } else {
                segment.push(ch);
            }
        }
        self.flush_segment(out, &segment, in_code);
    }

    fn flush_segment(&self, out: &mut String, segment: &str, in_code: bool) {
        if in_code {
            out.push_str(segment);
            return;
        }
        let rewritten = self
            .pattern
            .replace_all(segment, |caps: &regex::Captures<'_>| {
                let whole = &caps[0];
                let target = &caps[1];
                let is_uid = target.starts_with("mem_");
                match resolve_wikilink(self.known_uids, self.key_to_uid, target, is_uid) {
                    Ok(uid) => {
                        let label = self.title_by_uid.get(&uid).map_or(target, String::as_str);
                        format!("[{}](#/focus/{uid})", escape_link_label(label))
                    }
                    Err(_) => whole.to_string(),
                }
            });
        out.push_str(&rewritten);
    }
}

/// Escape the two characters that would break a markdown link label so a
/// title containing brackets can't terminate the `[label]` early.
fn escape_link_label(label: &str) -> String {
    label.replace('[', "\\[").replace(']', "\\]")
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
    use super::{
        Wikilink, backlinks_index, extract_wikilinks, linkify_wikilinks, resolve_wikilink,
    };
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

    fn linkify_fixtures() -> (
        BTreeSet<String>,
        BTreeMap<String, String>,
        BTreeMap<String, String>,
    ) {
        let known_uids = BTreeSet::from(["mem_018f3a".to_string()]);
        let key_to_uid = BTreeMap::from([("mem.foo".to_string(), "mem_018f3a".to_string())]);
        let title_by_uid = BTreeMap::from([("mem_018f3a".to_string(), "Foo Memory".to_string())]);
        (known_uids, key_to_uid, title_by_uid)
    }

    #[test]
    fn vt_10_linkifies_resolvable_key_with_title_label() {
        let (uids, keys, titles) = linkify_fixtures();
        let out = linkify_wikilinks("see [[mem.foo]] here", &uids, &keys, &titles);
        assert_eq!(out, "see [Foo Memory](#/focus/mem_018f3a) here");
    }

    #[test]
    fn vt_11_linkifies_resolvable_uid_form() {
        let (uids, keys, titles) = linkify_fixtures();
        let out = linkify_wikilinks("see [[mem_018f3a]] here", &uids, &keys, &titles);
        assert_eq!(out, "see [Foo Memory](#/focus/mem_018f3a) here");
    }

    #[test]
    fn vt_12_leaves_unresolvable_link_literal() {
        let (uids, keys, titles) = linkify_fixtures();
        let out = linkify_wikilinks("see [[mem.nope]] here", &uids, &keys, &titles);
        assert_eq!(out, "see [[mem.nope]] here");
    }

    #[test]
    fn vt_13_falls_back_to_target_when_title_absent() {
        let uids = BTreeSet::new();
        let keys = BTreeMap::from([("mem.foo".to_string(), "mem_018f3a".to_string())]);
        let titles = BTreeMap::new();
        let out = linkify_wikilinks("see [[mem.foo]]", &uids, &keys, &titles);
        assert_eq!(out, "see [mem.foo](#/focus/mem_018f3a)");
    }

    #[test]
    fn vt_14_skips_fenced_code_block() {
        let (uids, keys, titles) = linkify_fixtures();
        let body = "before [[mem.foo]]\n```\ninside [[mem.foo]]\n```\nafter [[mem.foo]]";
        let out = linkify_wikilinks(body, &uids, &keys, &titles);
        let expected = "before [Foo Memory](#/focus/mem_018f3a)\n```\ninside [[mem.foo]]\n```\nafter [Foo Memory](#/focus/mem_018f3a)";
        assert_eq!(out, expected);
    }

    #[test]
    fn vt_15_skips_inline_code_only() {
        let (uids, keys, titles) = linkify_fixtures();
        let out = linkify_wikilinks("keep `[[mem.foo]]` link [[mem.foo]]", &uids, &keys, &titles);
        assert_eq!(
            out,
            "keep `[[mem.foo]]` link [Foo Memory](#/focus/mem_018f3a)"
        );
    }

    #[test]
    fn vt_16_escapes_brackets_in_title_label() {
        let uids = BTreeSet::new();
        let keys = BTreeMap::from([("mem.foo".to_string(), "mem_018f3a".to_string())]);
        let titles = BTreeMap::from([("mem_018f3a".to_string(), "Foo [bar]".to_string())]);
        let out = linkify_wikilinks("[[mem.foo]]", &uids, &keys, &titles);
        assert_eq!(out, "[Foo \\[bar\\]](#/focus/mem_018f3a)");
    }

    #[test]
    fn vt_17_preserves_trailing_newline() {
        let (uids, keys, titles) = linkify_fixtures();
        assert_eq!(linkify_wikilinks("a\n", &uids, &keys, &titles), "a\n");
        assert_eq!(linkify_wikilinks("a", &uids, &keys, &titles), "a");
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
