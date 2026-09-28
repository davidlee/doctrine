// SPDX-License-Identifier: GPL-3.0-only
//! `lib:` citation scanning and resolution (SL-273 design section 4).
//!
//! One pure core, two callers: the in-crate build-repo tests below walk the
//! shipped roots against the on-disk manifest; the doctor leg
//! (`doctor_checks::lib_citation_findings`) walks a client's `.doctrine/**`
//! against the embedded one. The core knows nothing about files — it takes text
//! and an admitted [`PublicationManifest`], and each caller owns its file set.
//!
//! Engine tier (ADR-001): depends only on `publication`.
//!
//! The bare-mention half (`bare_targets` / `bare_mentions`) is `#[cfg(test)]`:
//! its only caller is the build-repo test — clients get no bare report
//! (design 4.3), so it has no production consumer to compile for.

use crate::publication::{LIB_PREFIX, LogicalAddress, PublicationManifest};

#[cfg(test)]
use std::collections::BTreeSet;

/// The address prefix whose `*.md` basenames the bare report looks for.
#[cfg(test)]
const REFERENCE_PREFIX: &str = "reference/";
/// The extension a bare-report target carries.
#[cfg(test)]
const MARKDOWN_EXT: &str = ".md";
/// Library basenames never reported bare: `governance.md` is also the client's
/// own `.doctrine/governance.md` (DEC-343).
#[cfg(test)]
const BARE_EXEMPT: &[&str] = &["governance.md"];

/// One `lib:` citation found in text; `address` excludes the marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LibCitation<'t> {
    pub address: &'t str,
    pub line: usize,
}

/// A bare mention of a library doc outside any `lib:` citation.
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BareMention<'t> {
    pub name: &'t str,
    pub line: usize,
}

/// Why a citation does not resolve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Unresolved {
    /// The address is not a safe relative logical path.
    Malformed,
    /// The address is well formed but no manifest entry declares it.
    Undeclared,
}

impl Unresolved {
    /// The human-readable reason, for reports.
    pub(crate) const fn reason(self) -> &'static str {
        match self {
            Self::Malformed => "malformed address (not a safe relative logical path)",
            Self::Undeclared => "not declared in the publication manifest",
        }
    }
}

/// Characters that end a citation's run (design 3.1).
const fn ends_run(c: char) -> bool {
    c.is_whitespace() || matches!(c, '`' | '"' | '\'' | '<' | '>' | '(' | ')' | '[' | ']')
}

/// Trailing sentence punctuation and Markdown emphasis, trimmed from the END of
/// a run only — an inner `#section` / `?x` stays and fails resolution.
const fn trails_run(c: char) -> bool {
    matches!(c, '.' | ',' | ';' | ':' | '!' | '?' | '*' | '_' | '~')
}

/// The citations on one line as `(marker byte offset, address)`. The marker
/// must be preceded by start-of-line or a non-alphanumeric character; the run
/// is taken whole, then trailing punctuation is trimmed; an empty run (the
/// `lib:<address>` placeholder) is not a citation.
fn line_citations(line: &str) -> Vec<(usize, &str)> {
    let mut found = Vec::new();
    let mut resume = 0;
    for (start, _) in line.match_indices(LIB_PREFIX) {
        if start < resume {
            continue; // inside a previous citation's run
        }
        if line[..start]
            .chars()
            .next_back()
            .is_some_and(char::is_alphanumeric)
        {
            continue; // `xlib:` — the marker is part of a word
        }
        let body = &line[start + LIB_PREFIX.len()..];
        let run = body.find(ends_run).map_or(body, |end| &body[..end]);
        let address = run.trim_end_matches(trails_run);
        resume = start + LIB_PREFIX.len() + run.len();
        if !address.is_empty() {
            found.push((start, address));
        }
    }
    found
}

/// Every citation in `text`, per the section 3.1 grammar (code spans and
/// fences included).
pub(crate) fn scan(text: &str) -> Vec<LibCitation<'_>> {
    text.lines()
        .enumerate()
        .flat_map(|(i, line)| {
            line_citations(line)
                .into_iter()
                .map(move |(_, address)| LibCitation {
                    address,
                    line: i + 1,
                })
        })
        .collect()
}

/// Citations whose address is malformed or not declared in `manifest`.
pub(crate) fn unresolved<'t>(
    cites: &[LibCitation<'t>],
    manifest: &PublicationManifest,
) -> Vec<(LibCitation<'t>, Unresolved)> {
    cites
        .iter()
        .filter_map(|c| match LogicalAddress::parse(c.address) {
            Err(_) => Some((*c, Unresolved::Malformed)),
            Ok(addr) if !manifest.declares_address(&addr) => Some((*c, Unresolved::Undeclared)),
            Ok(_) => None,
        })
        .collect()
}

/// The library-doc basenames the bare report looks for: every
/// `reference/*.md` entry's basename, minus [`BARE_EXEMPT`].
#[cfg(test)]
pub(crate) fn bare_targets(manifest: &PublicationManifest) -> BTreeSet<&str> {
    manifest
        .entries()
        .iter()
        .filter_map(|e| e.address().as_str().strip_prefix(REFERENCE_PREFIX))
        .filter(|name| name.ends_with(MARKDOWN_EXT) && !name.contains('/'))
        .filter(|name| !BARE_EXEMPT.contains(name))
        .collect()
}

/// Whether `c` may continue a file name, so a match preceded by it is part of
/// a longer name (`my-glossary.md`), not a mention.
#[cfg(test)]
const fn continues_name(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-')
}

/// Occurrences of `targets` in `text` not inside a `lib:` citation.
#[cfg(test)]
pub(crate) fn bare_mentions<'t>(text: &'t str, targets: &BTreeSet<&str>) -> Vec<BareMention<'t>> {
    let mut found = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let spans: Vec<(usize, usize)> = line_citations(line)
            .into_iter()
            .map(|(start, address)| (start, start + LIB_PREFIX.len() + address.len()))
            .collect();
        let mut hits: Vec<(usize, &'t str)> = Vec::new();
        for target in targets {
            for (at, name) in line.match_indices(target) {
                let bounded = !line[..at].chars().next_back().is_some_and(continues_name);
                let cited = spans.iter().any(|&(s, e)| s <= at && at < e);
                if bounded && !cited {
                    hits.push((at, name));
                }
            }
        }
        hits.sort_unstable();
        found.extend(
            hits.into_iter()
                .map(|(_, name)| BareMention { name, line: i + 1 }),
        );
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// The shipped text roots the build-repo tests walk, relative to the repo
    /// root (design 4.2). Every file, not only `*.md`.
    const SHIPPED_TEXT_ROOTS: &[&str] = &["plugins/doctrine/skills", "install"];

    /// A manifest declaring `reference/glossary.md`, `reference/governance.md`
    /// and `templates/design.md`.
    fn manifest() -> PublicationManifest {
        let entry = |address: &str, kind: &str| {
            format!(
                "[[entry]]\n\
                 address = \"{address}\"\n\
                 backing = \"{address}\"\n\
                 kind = \"{kind}\"\n\
                 title = \"Fixture\"\n\
                 licence = \"MIT\"\n\
                 provenance = \"declared\"\n\
                 customization = \"customizable\"\n"
            )
        };
        let body = [
            entry("reference/glossary.md", "reference"),
            entry("reference/governance.md", "reference"),
            entry("reference/LICENSE", "reference"),
            entry("templates/design.md", "template"),
        ]
        .concat();
        PublicationManifest::admit(body.as_bytes()).expect("fixture admits")
    }

    fn addresses<'t>(cites: &[LibCitation<'t>]) -> Vec<&'t str> {
        cites.iter().map(|c| c.address).collect()
    }

    fn disk_manifest() -> PublicationManifest {
        PublicationManifest::admit(&crate::asset_source::publication_manifest_bytes_from_disk())
            .expect("shipped publication manifest admits from disk")
    }

    // ---- scan (design 4.4) ----

    #[test]
    fn scan_finds_citation_in_code_span() {
        let cites = scan("Read `lib:reference/glossary.md` first.");
        assert_eq!(
            cites,
            vec![LibCitation {
                address: "reference/glossary.md",
                line: 1
            }]
        );
    }

    #[test]
    fn scan_finds_citation_in_fence_with_line_number() {
        let text = "intro\n\n```text\nlib:templates/design.md\n```\n";
        let cites = scan(text);
        assert_eq!(
            cites,
            vec![LibCitation {
                address: "templates/design.md",
                line: 4
            }]
        );
    }

    #[test]
    fn scan_trims_sentence_end_period() {
        let cites = scan("See lib:reference/glossary.md.");
        assert_eq!(addresses(&cites), vec!["reference/glossary.md"]);
    }

    #[test]
    fn scan_finds_citation_in_markdown_link_target() {
        let cites = scan("the [glossary](lib:reference/glossary.md) says");
        assert_eq!(addresses(&cites), vec!["reference/glossary.md"]);
    }

    #[test]
    fn scan_ignores_placeholder_and_embedded_marker() {
        assert!(scan("write `lib:<address>` here").is_empty());
        assert!(scan("xlib:foo.md").is_empty());
        assert!(scan("lib:").is_empty());
    }

    #[test]
    fn scan_trims_trailing_bang() {
        let cites = scan("Read lib:reference/glossary.md!");
        assert_eq!(addresses(&cites), vec!["reference/glossary.md"]);
        assert!(
            unresolved(&cites, &manifest()).is_empty(),
            "trimmed citation resolves"
        );
    }

    #[test]
    fn scan_trims_trailing_question_and_emphasis() {
        assert_eq!(
            addresses(&scan("Did you read lib:reference/glossary.md?")),
            vec!["reference/glossary.md"]
        );
        assert_eq!(
            addresses(&scan("**lib:reference/glossary.md**")),
            vec!["reference/glossary.md"]
        );
        assert_eq!(
            addresses(&scan(
                "_lib:reference/glossary.md_, ~lib:templates/design.md~;"
            )),
            vec!["reference/glossary.md", "templates/design.md"]
        );
    }

    #[test]
    fn scan_keeps_inner_suffix_and_fails_it() {
        let cites = scan("lib:reference/glossary.md#x and lib:reference/glossary.md?x.");
        assert_eq!(
            addresses(&cites),
            vec!["reference/glossary.md#x", "reference/glossary.md?x"]
        );
        let bad = unresolved(&cites, &manifest());
        assert_eq!(bad.len(), 2, "both suffixed citations fail: {bad:?}");
        assert!(bad.iter().all(|(_, why)| *why == Unresolved::Undeclared));
    }

    #[test]
    fn scan_finds_several_per_line() {
        let cites = scan("lib:reference/glossary.md, lib:templates/design.md");
        assert_eq!(
            addresses(&cites),
            vec!["reference/glossary.md", "templates/design.md"]
        );
    }

    // ---- unresolved ----

    #[test]
    fn unresolved_classifies_malformed_undeclared_and_passes_declared() {
        let cites = scan("lib:../escape.md lib:reference/x.md lib:reference/glossary.md");
        let bad = unresolved(&cites, &manifest());
        let got: Vec<(&str, Unresolved)> = bad.iter().map(|(c, why)| (c.address, *why)).collect();
        assert_eq!(
            got,
            vec![
                ("../escape.md", Unresolved::Malformed),
                ("reference/x.md", Unresolved::Undeclared),
            ]
        );
    }

    // ---- bare_targets / bare_mentions ----

    #[test]
    fn bare_targets_are_reference_md_basenames_minus_exempt() {
        let m = manifest();
        let targets = bare_targets(&m);
        assert_eq!(targets.into_iter().collect::<Vec<_>>(), vec!["glossary.md"]);
    }

    #[test]
    fn bare_mentions_reports_path_forms_outside_citations() {
        let m = manifest();
        let targets = bare_targets(&m);
        let text = "glossary.md\ninstall/glossary.md\ndoctrine library show reference/glossary.md\n\
                    .doctrine/glossary.md\n";
        let got: Vec<(&str, usize)> = bare_mentions(text, &targets)
            .iter()
            .map(|b| (b.name, b.line))
            .collect();
        assert_eq!(
            got,
            vec![
                ("glossary.md", 1),
                ("glossary.md", 2),
                ("glossary.md", 3),
                ("glossary.md", 4)
            ]
        );
    }

    #[test]
    fn bare_mentions_ignores_citations_lookalikes_and_exempt() {
        let m = manifest();
        let targets = bare_targets(&m);
        let text = "`lib:reference/glossary.md` and lib:reference/glossary.md#x\n\
                    my-glossary.md my_glossary.md\ngovernance.md\n";
        assert!(bare_mentions(text, &targets).is_empty());
    }

    // ---- build-repo walk (design 4.2) ----

    /// What one walk of the shipped roots found.
    #[derive(Debug, Default)]
    struct Walk {
        files: usize,
        unresolved: Vec<String>,
        bare: Vec<String>,
    }

    /// Walk every file under `root`'s [`SHIPPED_TEXT_ROOTS`] against `manifest`.
    /// A missing root, an unreadable entry, or a non-UTF-8 file fails by name
    /// (STD-003) — nothing is skipped.
    fn walk_shipped(root: &Path, manifest: &PublicationManifest) -> Walk {
        let targets = bare_targets(manifest);
        let mut walk = Walk::default();
        for dir in SHIPPED_TEXT_ROOTS {
            for entry in walkdir::WalkDir::new(root.join(dir)).sort_by_file_name() {
                let entry = entry.unwrap_or_else(|e| panic!("cannot walk shipped root {dir}: {e}"));
                if !entry.file_type().is_file() {
                    continue;
                }
                let path = entry.path();
                let rel = path
                    .strip_prefix(root)
                    .unwrap_or(path)
                    .display()
                    .to_string();
                let bytes =
                    std::fs::read(path).unwrap_or_else(|e| panic!("cannot read {rel}: {e}"));
                let text = String::from_utf8(bytes)
                    .unwrap_or_else(|e| panic!("shipped file {rel} is not UTF-8: {e}"));
                walk.files += 1;
                for (c, why) in unresolved(&scan(&text), manifest) {
                    walk.unresolved.push(format!(
                        "{rel}:{}: {LIB_PREFIX}{} — {}",
                        c.line,
                        c.address,
                        why.reason()
                    ));
                }
                for b in bare_mentions(&text, &targets) {
                    walk.bare.push(format!("{rel}:{}: {}", b.line, b.name));
                }
            }
        }
        walk
    }

    /// A seeded fixture root carrying both shipped roots (empty) — callers add files.
    fn fixture_root() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().expect("tempdir");
        for dir in SHIPPED_TEXT_ROOTS {
            std::fs::create_dir_all(tmp.path().join(dir)).expect("create shipped root");
        }
        tmp
    }

    #[test]
    fn every_shipped_lib_citation_resolves() {
        let walk = walk_shipped(&crate::test_support::repo_root(), &disk_manifest());
        assert!(
            walk.files > 0,
            "the shipped walk read no files — misrooted?"
        );
        assert!(
            walk.unresolved.is_empty(),
            "{} unresolved lib: citation(s) in shipped text:\n{}",
            walk.unresolved.len(),
            walk.unresolved.join("\n")
        );
    }

    #[test]
    fn no_bare_library_mention_in_shipped_text() {
        let walk = walk_shipped(&crate::test_support::repo_root(), &disk_manifest());
        assert!(
            walk.files > 0,
            "the shipped walk read no files — misrooted?"
        );
        assert!(
            walk.bare.is_empty(),
            "{} bare library mention(s) in shipped text:\n{}",
            walk.bare.len(),
            walk.bare.join("\n")
        );
    }

    // RV-408 F-2 control: a shipped-style TOML file is walked (not only *.md),
    // and both checks fire on it.
    #[test]
    fn shipped_walk_fails_on_seeded_toml_fixture() {
        let tmp = fixture_root();
        let templates = tmp.path().join("install/templates");
        std::fs::create_dir_all(&templates).expect("mkdir");
        std::fs::write(
            templates.join("fixture.toml"),
            "# Terms are defined in glossary.md.\n# See lib:reference/not-declared.md\nkey = 1\n",
        )
        .expect("write fixture");
        let walk = walk_shipped(tmp.path(), &disk_manifest());
        assert_eq!(walk.files, 1);
        assert_eq!(
            walk.bare,
            vec!["install/templates/fixture.toml:1: glossary.md"]
        );
        assert_eq!(walk.unresolved.len(), 1, "{:?}", walk.unresolved);
        assert!(
            walk.unresolved[0]
                .starts_with("install/templates/fixture.toml:2: lib:reference/not-declared.md")
        );
    }

    // RV-408 F-4 control: the walk resolves against the manifest it is GIVEN.
    // Removing one entry from the on-disk bytes makes a citation to it fail,
    // which a walk reaching for the compiled embed could not show.
    #[test]
    fn shipped_walk_uses_given_manifest_not_embed() {
        let bytes = crate::asset_source::publication_manifest_bytes_from_disk();
        let full = PublicationManifest::admit(&bytes).expect("disk manifest admits");
        let removed = full.entries()[0].address().as_str().to_string();

        let mut doc: toml::Table =
            toml::from_str(std::str::from_utf8(&bytes).expect("utf-8")).expect("toml");
        let Some(toml::Value::Array(entries)) = doc.get_mut("entry") else {
            panic!("manifest has an [[entry]] array");
        };
        entries.retain(|e| e.get("address").and_then(toml::Value::as_str) != Some(&removed));
        let less = PublicationManifest::admit(toml::to_string(&doc).expect("ser").as_bytes())
            .expect("reduced manifest admits");
        assert_eq!(less.entries().len() + 1, full.entries().len());

        let tmp = fixture_root();
        std::fs::write(
            tmp.path().join("install/cite.md"),
            format!("{LIB_PREFIX}{removed}\n"),
        )
        .expect("write fixture");

        assert!(walk_shipped(tmp.path(), &full).unresolved.is_empty());
        let walk = walk_shipped(tmp.path(), &less);
        assert_eq!(walk.unresolved.len(), 1, "{:?}", walk.unresolved);
    }

    // STD-003 control: a non-UTF-8 shipped file fails the walk by name.
    #[test]
    #[should_panic(expected = "install/blob.bin is not UTF-8")]
    fn shipped_walk_fails_on_non_utf8_file_by_name() {
        let tmp = fixture_root();
        std::fs::write(tmp.path().join("install/blob.bin"), [0xff, 0xfe, 0x00]).expect("write");
        let _ = walk_shipped(tmp.path(), &manifest());
    }
}
