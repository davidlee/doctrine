// SPDX-License-Identifier: GPL-3.0-only
//! Corpus id-integrity — `validate` (detect) + `reseat` (repair), the ADR-006 D3
//! backstop for fork-safe id allocation.
//!
//! Ids are **per-namespace** (X2): `SL-001`, `ADR-001`, `REQ-001` coexist
//! legitimately, so every check is *intra-kind*. The kind-owning modules each
//! declare their own `Kind`/`GovKind`, but the trio a generic id scan needs —
//! canonical prefix, tree dir, and the toml filename *stem* — travels together
//! nowhere. [`KINDS`] is that single table (design D-C). Memory is a *named*
//! kind (`mem_<uid>` dirs, key aliases) with no numeric id, so it is out of
//! scope here (D-A); its alias-integrity is a later key-based variant.

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};

use crate::{entity, fsutil, git, listing, meta, root};

/// A numbered entity kind's identity for the id scan — gathered where the
/// kind-owning modules otherwise scatter it. `stem` names the metadata file
/// (`slice-007.toml`); `prefix` the canonical id (`SL-007`); `has_runtime_state`
/// marks the kinds that own gitignored phase state under `.doctrine/state/`
/// (only slice today), which `reseat` refuses to strand (F3).
pub(crate) struct KindRef {
    pub(crate) prefix: &'static str,
    pub(crate) dir: &'static str,
    pub(crate) stem: &'static str,
    pub(crate) has_runtime_state: bool,
}

/// Every numbered kind, in canonical order. The one place this list lives; a new
/// numbered kind must be added here or it silently escapes `validate` (R-b — a
/// drift surface this table accepts in exchange for not threading a registry
/// through every kind-owning module).
pub(crate) const KINDS: &[KindRef] = &[
    KindRef {
        prefix: "SL",
        dir: ".doctrine/slice",
        stem: "slice",
        has_runtime_state: true,
    },
    KindRef {
        prefix: "ADR",
        dir: ".doctrine/adr",
        stem: "adr",
        has_runtime_state: false,
    },
    KindRef {
        prefix: "POL",
        dir: ".doctrine/policy",
        stem: "policy",
        has_runtime_state: false,
    },
    KindRef {
        prefix: "STD",
        dir: ".doctrine/standard",
        stem: "standard",
        has_runtime_state: false,
    },
    KindRef {
        prefix: "PRD",
        dir: ".doctrine/spec/product",
        stem: "spec",
        has_runtime_state: false,
    },
    KindRef {
        prefix: "SPEC",
        dir: ".doctrine/spec/tech",
        stem: "spec",
        has_runtime_state: false,
    },
    KindRef {
        prefix: "REQ",
        dir: ".doctrine/requirement",
        stem: "requirement",
        has_runtime_state: false,
    },
    KindRef {
        prefix: "ISS",
        dir: ".doctrine/backlog/issue",
        stem: "backlog",
        has_runtime_state: false,
    },
    KindRef {
        prefix: "IMP",
        dir: ".doctrine/backlog/improvement",
        stem: "backlog",
        has_runtime_state: false,
    },
    KindRef {
        prefix: "CHR",
        dir: ".doctrine/backlog/chore",
        stem: "backlog",
        has_runtime_state: false,
    },
    KindRef {
        prefix: "RSK",
        dir: ".doctrine/backlog/risk",
        stem: "backlog",
        has_runtime_state: false,
    },
    KindRef {
        prefix: "IDE",
        dir: ".doctrine/backlog/idea",
        stem: "backlog",
        has_runtime_state: false,
    },
];

// ---------------------------------------------------------------------------
// Pure check layer — facts in, findings out. No disk (design pure/impure split).
// ---------------------------------------------------------------------------

/// One numbered entity's scanned identity facts: its directory's id (the parsed
/// `NNN` basename) and the id its sister toml *declares*.
struct EntityFacts {
    dir_id: u32,
    toml_id: u32,
}

/// One `NNN-slug` alias symlink's facts: the id its name *encodes*, and the toml
/// id of the directory it actually *targets* (`None` if the target is missing or
/// non-numeric — an unverifiable, therefore failing, alias).
struct AliasFacts {
    encoded_id: u32,
    target_toml_id: Option<u32>,
}

/// One kind's scanned namespace — the pure-check input.
struct KindSnapshot {
    prefix: &'static str,
    entities: Vec<EntityFacts>,
    aliases: Vec<AliasFacts>,
}

/// A single integrity violation, pre-formatted with its kind for the report.
struct Finding(String);

impl std::fmt::Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// The three per-kind rules (design §5.2): (a) dir basename == toml `id`;
/// (b) no two dirs of a kind declare the same `id`; (c) every alias targets the
/// dir whose toml id equals the alias's encoded id — target equality, not mere
/// resolvability (X7).
fn check_kind(snap: &KindSnapshot) -> Vec<Finding> {
    let p = snap.prefix;
    let mut findings = Vec::new();

    // (a) dir basename vs declared id.
    for e in &snap.entities {
        if e.dir_id != e.toml_id {
            findings.push(Finding(format!(
                "{p}: dir {:03} declares id {:03} (basename ≠ toml id)",
                e.dir_id, e.toml_id
            )));
        }
    }

    // (b) duplicate declared id within the kind.
    let mut by_id: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for e in &snap.entities {
        by_id.entry(e.toml_id).or_default().push(e.dir_id);
    }
    for (id, mut dirs) in by_id {
        if dirs.len() > 1 {
            dirs.sort_unstable();
            let dirs = dirs
                .iter()
                .map(|d| format!("{d:03}"))
                .collect::<Vec<_>>()
                .join(", ");
            findings.push(Finding(format!(
                "{p}: id {id:03} declared by dirs {dirs} (intra-kind duplicate)"
            )));
        }
    }

    // (c) alias target equality.
    for a in &snap.aliases {
        if a.target_toml_id != Some(a.encoded_id) {
            let got = a.target_toml_id.map_or_else(
                || "no numbered target".to_string(),
                |t| format!("id {t:03}"),
            );
            findings.push(Finding(format!(
                "{p}: alias {:03}-* targets {got} (expected id {:03})",
                a.encoded_id, a.encoded_id
            )));
        }
    }

    findings
}

// ---------------------------------------------------------------------------
// Impure scan — the thin shell that reads the corpus into snapshots.
// ---------------------------------------------------------------------------

/// Read one kind's namespace under `root` into a [`KindSnapshot`]. A malformed
/// metadata toml is a hard error (propagated), distinct from an integrity
/// finding — `validate` reports inconsistency, it does not paper over corruption.
fn scan_kind(root: &Path, kind: &'static KindRef) -> anyhow::Result<KindSnapshot> {
    let tree_root = root.join(kind.dir);

    let mut entities = Vec::new();
    for dir_id in entity::scan_ids(&tree_root)? {
        let toml_id = meta::read_meta(&tree_root, kind.stem, dir_id)?.id;
        entities.push(EntityFacts { dir_id, toml_id });
    }

    let aliases = scan_aliases(&tree_root, kind.stem)?;
    Ok(KindSnapshot {
        prefix: kind.prefix,
        entities,
        aliases,
    })
}

/// Collect the `NNN-slug` alias symlinks directly under `tree_root`. Each yields
/// the id its name encodes and the declared id of the dir it resolves to. A
/// symlink whose name does not lead with `NNN-` is not an entity alias and is
/// skipped (memory's `mem.*` aliases never appear under a numbered tree anyway).
fn scan_aliases(tree_root: &Path, stem: &str) -> anyhow::Result<Vec<AliasFacts>> {
    let mut aliases = Vec::new();
    let entries = match std::fs::read_dir(tree_root) {
        Ok(e) => e,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(aliases),
        Err(e) => return Err(e).with_context(|| format!("read {}", tree_root.display())),
    };
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_symlink() {
            continue;
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        let Some((head, _)) = name.split_once('-') else {
            continue;
        };
        let Ok(encoded_id) = head.parse::<u32>() else {
            continue;
        };

        // Resolve the link's target dir basename → its declared toml id.
        let target_toml_id = std::fs::read_link(entry.path())
            .ok()
            .and_then(|t| t.file_name().and_then(|b| b.to_str()?.parse::<u32>().ok()))
            .and_then(|target_dir_id| meta::read_meta(tree_root, stem, target_dir_id).ok())
            .map(|m| m.id);

        aliases.push(AliasFacts {
            encoded_id,
            target_toml_id,
        });
    }
    Ok(aliases)
}

/// `doctrine validate` — scan every numbered kind for id-integrity violations
/// (ADR-006 D3 detect-half). Names the kinds scanned (so the memory omission is
/// visible, D-A), prints each finding, and exits non-zero if any; a clean corpus
/// prints an all-clear and exits zero. Read-only.
pub(crate) fn run_validate(path: Option<PathBuf>) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;

    let mut findings = Vec::new();
    for kind in KINDS {
        findings.extend(check_kind(&scan_kind(&root, kind)?));
    }

    let scanned = KINDS
        .iter()
        .map(|k| k.prefix)
        .collect::<Vec<_>>()
        .join(", ");
    writeln!(io::stdout(), "validate: scanned {scanned}")?;

    if findings.is_empty() {
        writeln!(io::stdout(), "validate: corpus clean")?;
        return Ok(());
    }
    for f in &findings {
        writeln!(io::stdout(), "  {f}")?;
    }
    bail!("validate: {} integrity finding(s)", findings.len())
}

// ---------------------------------------------------------------------------
// reseat — the D3 repair backstop (renumber an entity's canonical-id triple).
// ---------------------------------------------------------------------------

/// Resolve a numbered kind by its canonical prefix (`SL` → the slice [`KindRef`]).
pub(crate) fn kind_by_prefix(prefix: &str) -> Option<&'static KindRef> {
    KINDS.iter().find(|k| k.prefix == prefix)
}

/// Parse a canonical ref (`SL-031`) into its kind and numeric id. Reseat keys on
/// the canonical ref, never a bare number (X2/D7) — the kind disambiguates the
/// per-namespace id.
fn parse_canonical_ref(reference: &str) -> anyhow::Result<(&'static KindRef, u32)> {
    let (prefix, num) = reference
        .rsplit_once('-')
        .with_context(|| format!("`{reference}` is not a canonical ref (expected e.g. SL-031)"))?;
    let kind = kind_by_prefix(prefix)
        .with_context(|| format!("unknown kind prefix `{prefix}` in `{reference}`"))?;
    let id = num
        .parse::<u32>()
        .with_context(|| format!("`{num}` is not a numeric id in `{reference}`"))?;
    Ok((kind, id))
}

/// `doctrine reseat <CANONICAL_REF> [--to <NNN>]` — renumber an entity's
/// canonical-id quad (dir name, the `<stem>-NNN.{toml,md}` filenames, the toml
/// `id` field, the `NNN-slug` alias) to the next free trunk-aware id, or to an
/// explicit `--to`. Guards (checked BEFORE any mutation): an occupied target is
/// refused (no clobber, §5.3); an id with live gitignored runtime phase state is
/// refused (F3 — reseat does not own the disposable tier). Inbound prose
/// citations are reported as danglers and force a non-zero exit; prose is never
/// rewritten (ADR-004 outbound-only, D4/R-3).
///
/// CONTRACT (SL-032 review F-4, accepted): the dangler exit is **non-zero even on
/// a fully-completed reseat** — the mutation succeeded, the citations are the
/// human's to fix; `reseat && commit` is therefore wrong, drive it by hand. The
/// six post-guard fs ops are **not transactional**: a mid-sequence failure leaves
/// a half-reseated entity that `validate` will flag. Reseat targets freshly
/// minted, pre-execution collisions where that blast radius is acceptable;
/// hardening it to atomic is a tracked follow-up, not done here.
pub(crate) fn run_reseat(
    path: Option<PathBuf>,
    reference: &str,
    to: Option<u32>,
) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    let (kind, src_id) = parse_canonical_ref(reference)?;
    let tree_root = root.join(kind.dir);

    let src_name = format!("{src_id:03}");
    let src_dir = tree_root.join(&src_name);
    anyhow::ensure!(
        fsutil::is_real_dir(&src_dir),
        "no {} at {}",
        listing::canonical_id(kind.prefix, src_id),
        src_dir.display()
    );
    // Slug from the authored metadata — the alias name component.
    let slug = meta::read_meta(&tree_root, kind.stem, src_id)?.slug;

    // The free-id pick: explicit `--to`, else the trunk-aware default (PHASE-02).
    let dst_id = match to {
        Some(t) => t,
        None => entity::next_id(
            &entity::scan_ids(&tree_root)?,
            &git::trunk_entity_ids(&root, kind.dir)?,
        ),
    };
    anyhow::ensure!(
        dst_id != src_id,
        "{} is already seated at {src_name}",
        listing::canonical_id(kind.prefix, src_id)
    );

    let dst_name = format!("{dst_id:03}");
    let dst_dir = tree_root.join(&dst_name);

    // Guard 1 — occupied target (no clobber). `exists` resolves the numeric dir.
    anyhow::ensure!(
        !dst_dir.exists(),
        "id {dst_name} is occupied — refusing to clobber {}",
        dst_dir.display()
    );
    // Guard 2 — live runtime phase state (F3). Only `has_runtime_state` kinds
    // (slice) key disposable state by id; reseat does not migrate that tier.
    if kind.has_runtime_state {
        let state = root.join(".doctrine/state/slice").join(&src_name);
        anyhow::ensure!(
            !state.exists(),
            "{} has live runtime phase state at {} — clear it first (reseat does not own the disposable tier)",
            listing::canonical_id(kind.prefix, src_id),
            state.display()
        );
    }

    // --- Mutation (all guards passed) ---
    std::fs::rename(&src_dir, &dst_dir)
        .with_context(|| format!("rename {} → {}", src_dir.display(), dst_dir.display()))?;
    for ext in ["toml", "md"] {
        let from = dst_dir.join(format!("{}-{src_name}.{ext}", kind.stem));
        let onto = dst_dir.join(format!("{}-{dst_name}.{ext}", kind.stem));
        if from.exists() {
            std::fs::rename(&from, &onto)
                .with_context(|| format!("rename {} → {}", from.display(), onto.display()))?;
        }
    }
    // toml `id` field — edit-preserving (toml_edit keeps comments/sections).
    let toml_path = dst_dir.join(format!("{}-{dst_name}.toml", kind.stem));
    let text = std::fs::read_to_string(&toml_path)
        .with_context(|| format!("read {}", toml_path.display()))?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("parse {}", toml_path.display()))?;
    doc.as_table_mut()
        .insert("id", toml_edit::value(i64::from(dst_id)));
    std::fs::write(&toml_path, doc.to_string())
        .with_context(|| format!("write {}", toml_path.display()))?;
    // Alias — drop the old `NNN-slug`, plant `MMM-slug → MMM`.
    let old_alias = tree_root.join(format!("{src_name}-{slug}"));
    if matches!(std::fs::symlink_metadata(&old_alias), Ok(m) if m.file_type().is_symlink()) {
        std::fs::remove_file(&old_alias)
            .with_context(|| format!("remove stale alias {}", old_alias.display()))?;
    }
    fsutil::set_symlink(
        &tree_root.join(format!("{dst_name}-{slug}")),
        Path::new(&dst_name),
    )?;

    let old_ref = listing::canonical_id(kind.prefix, src_id);
    let new_ref = listing::canonical_id(kind.prefix, dst_id);
    writeln!(io::stdout(), "reseated {old_ref} → {new_ref}")?;

    // Inbound prose citations — report, never rewrite (D4/R-3).
    let danglers = scan_danglers(&root, &old_ref)?;
    if danglers.is_empty() {
        return Ok(());
    }
    writeln!(
        io::stdout(),
        "inbound citations to {old_ref} (rewrite by hand — prose relations are outbound-only):"
    )?;
    for d in &danglers {
        writeln!(io::stdout(), "  {d}")?;
    }
    bail!(
        "reseat: {} inbound citation(s) to {old_ref} remain",
        danglers.len()
    )
}

/// Scan authored `.doctrine/**/*.md` prose for inbound citations of `needle`
/// (a canonical ref), returning `file:line` locations. A whole-token match
/// (`SL-031` does not match inside `SL-0310`) keeps the report honest, and
/// disposable prose ([`is_disposable_prose`]) is skipped — a `rm -rf`-able
/// `handover.md` or runtime phase note is not a citation a human must rewrite.
fn scan_danglers(root: &Path, needle: &str) -> anyhow::Result<Vec<String>> {
    let pattern = root.join(".doctrine/**/*.md");
    let pattern = pattern
        .to_str()
        .with_context(|| format!("non-utf8 scan path {}", pattern.display()))?;

    let mut hits = Vec::new();
    for entry in glob::glob(pattern).context("bad glob pattern")? {
        let path = entry.context("glob walk")?;
        if is_disposable_prose(&path) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue; // non-utf8 / unreadable — not authored prose we cite
        };
        for (i, line) in text.lines().enumerate() {
            if line_cites(line, needle) {
                hits.push(format!("{}:{}", path.display(), i + 1));
            }
        }
    }
    Ok(hits)
}

/// True for prose in the disposable tiers a reseat must not nag about: any file
/// under the gitignored runtime state tree (`.doctrine/state/…`) and any
/// `handover.md` (per-agent scratch, GITIGNORED). Authored prose — slice/adr/spec
/// bodies, committed `memory.md` — is never disposable and stays in scope.
fn is_disposable_prose(path: &Path) -> bool {
    if path.file_name().and_then(|n| n.to_str()) == Some("handover.md") {
        return true;
    }
    let comps: Vec<_> = path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();
    comps.windows(2).any(|w| w == [".doctrine", "state"])
}

/// True when `line` cites `needle` as a whole canonical token — neither the char
/// before nor the char after is alphanumeric, so `SL-031` is not found inside
/// `ASL-031`, `SL-0310`, or `SL-031x` (a glued suffix is never a real ref).
fn line_cites(line: &str, needle: &str) -> bool {
    let mut base = 0;
    while let Some(rest) = line.get(base..)
        && let Some(pos) = rest.find(needle)
    {
        let i = base + pos;
        let before_ok = line
            .get(..i)
            .and_then(|s| s.chars().next_back())
            .is_none_or(|c| !c.is_ascii_alphanumeric());
        let after = i + needle.len();
        let after_ok = line
            .get(after..)
            .and_then(|s| s.chars().next())
            .is_none_or(|c| !c.is_ascii_alphanumeric());
        if before_ok && after_ok {
            return true;
        }
        base = i + 1;
    }
    false
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(entities: Vec<(u32, u32)>, aliases: Vec<(u32, Option<u32>)>) -> KindSnapshot {
        KindSnapshot {
            prefix: "SL",
            entities: entities
                .into_iter()
                .map(|(dir_id, toml_id)| EntityFacts { dir_id, toml_id })
                .collect(),
            aliases: aliases
                .into_iter()
                .map(|(encoded_id, target_toml_id)| AliasFacts {
                    encoded_id,
                    target_toml_id,
                })
                .collect(),
        }
    }

    #[test]
    fn clean_kind_yields_no_findings() {
        let s = snap(vec![(1, 1), (2, 2)], vec![(1, Some(1)), (2, Some(2))]);
        assert!(check_kind(&s).is_empty());
    }

    #[test]
    fn rule_a_flags_dir_id_mismatch() {
        // dir 003 declares id 045 — the planted VT-1 shape.
        let s = snap(vec![(3, 45)], vec![]);
        let f = check_kind(&s);
        assert_eq!(f.len(), 1);
        assert!(
            f[0].to_string().contains("dir 003 declares id 045"),
            "{}",
            f[0]
        );
    }

    #[test]
    fn rule_b_flags_intra_kind_duplicate_id() {
        // two dirs both declaring id 7 (VT-2). dir 008 also trips rule (a).
        let s = snap(vec![(7, 7), (8, 7)], vec![]);
        let f = check_kind(&s);
        let dup = f
            .iter()
            .find(|x| x.to_string().contains("intra-kind duplicate"));
        let dup = dup.expect("a duplicate finding");
        assert!(dup.to_string().contains("007, 008"), "{dup}");
    }

    #[test]
    fn rule_c_flags_mis_targeted_alias() {
        // alias encodes 031 but targets a dir declaring 045 (VT-3).
        let s = snap(vec![], vec![(31, Some(45))]);
        let f = check_kind(&s);
        assert_eq!(f.len(), 1);
        assert!(
            f[0].to_string().contains("alias 031-* targets id 045"),
            "{}",
            f[0]
        );
    }

    #[test]
    fn rule_c_flags_dangling_alias() {
        // alias resolves to no numbered target at all.
        let s = snap(vec![], vec![(31, None)]);
        let f = check_kind(&s);
        assert_eq!(f.len(), 1);
        assert!(f[0].to_string().contains("no numbered target"), "{}", f[0]);
    }

    #[test]
    fn parse_canonical_ref_resolves_kind_and_id() {
        let (kind, id) = parse_canonical_ref("SL-031").expect("valid ref");
        assert_eq!(kind.prefix, "SL");
        assert_eq!(id, 31);
        assert!(
            parse_canonical_ref("031").is_err(),
            "bare id is not canonical"
        );
        assert!(
            parse_canonical_ref("ZZ-001").is_err(),
            "unknown prefix rejected"
        );
        assert!(
            parse_canonical_ref("SL-x").is_err(),
            "non-numeric id rejected"
        );
    }

    #[test]
    fn line_cites_matches_whole_token_only() {
        assert!(line_cites("see SL-031 for detail", "SL-031"));
        assert!(line_cites("SL-031, ADR-004", "SL-031"));
        assert!(line_cites("SL-031", "SL-031"));
        // boundary guards: a longer id or a glued prefix/suffix must not match.
        assert!(!line_cites("SL-0310 is different", "SL-031"));
        assert!(!line_cites("XSL-031", "SL-031"));
        assert!(!line_cites("SL-031x is not a ref", "SL-031")); // glued alpha suffix
        assert!(!line_cites("nothing here", "SL-031"));
    }

    #[test]
    fn scan_danglers_skips_disposable_prose() {
        // F-7: a citation in authored prose is reported; the same citation in a
        // gitignored handover or runtime phase note is not.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let plant = |rel: &str| {
            let p = root.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, "cites SL-031 here\n").unwrap();
        };
        plant(".doctrine/notes/x.md"); // authored → reported
        plant(".doctrine/slice/001/handover.md"); // disposable → skipped
        plant(".doctrine/state/slice/001/phases/phase-01.md"); // runtime → skipped

        let hits = scan_danglers(root, "SL-031").unwrap();
        assert_eq!(hits.len(), 1, "only authored prose reported: {hits:?}");
        assert!(hits[0].ends_with("notes/x.md:1"), "{}", hits[0]);
    }

    #[test]
    fn kinds_table_covers_the_twelve_numbered_kinds() {
        let prefixes: Vec<_> = KINDS.iter().map(|k| k.prefix).collect();
        assert_eq!(
            prefixes,
            [
                "SL", "ADR", "POL", "STD", "PRD", "SPEC", "REQ", "ISS", "IMP", "CHR", "RSK", "IDE"
            ]
        );
        // Only slice owns runtime phase state (F3 guard surface).
        let stateful: Vec<_> = KINDS
            .iter()
            .filter(|k| k.has_runtime_state)
            .map(|k| k.prefix)
            .collect();
        assert_eq!(stateful, ["SL"]);
    }
}
