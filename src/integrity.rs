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

use crate::adr::ADR_KIND;
use crate::backlog::{CHORE_KIND, IDEA_KIND, IMPROVEMENT_KIND, ISSUE_KIND, RISK_KIND};
use crate::concept_map::CONCEPT_MAP_KIND;
use crate::knowledge::{
    ASSUMPTION_KIND, CONSTRAINT_KIND, DECISION_KIND, EVIDENCE_KIND, HYPOTHESIS_KIND, QUESTION_KIND,
};
use crate::policy::POLICY_KIND;
use crate::rec::REC_KIND;
use crate::requirement::REQUIREMENT_KIND;
use crate::review::REVIEW_KIND;
use crate::revision::REV_KIND;
use crate::rfc::RFC_KIND;
use crate::slice::SLICE_KIND;
use crate::spec::{PRODUCT_SPEC_KIND, TECH_SPEC_KIND};
use crate::standard::STANDARD_KIND;
use crate::{entity, fsutil, git, listing, meta, root};

/// A numbered entity kind's identity for the id scan — a *referencing view* over
/// the engine [`entity::Kind`] each kind-owning module already declares (its
/// canonical `prefix` and tree `dir`), plus the two facts the engine leaf does
/// not carry: `stem` names the metadata file (`slice-007.toml`), and `state_dir`
/// is the gitignored runtime phase-state tree (`.doctrine/state/slice`) a kind
/// owns — `Some` only for slice today — which `reseat` refuses to strand (F3).
pub(crate) struct KindRef {
    pub(crate) kind: &'static entity::Kind,
    pub(crate) state_dir: Option<&'static str>,
}

/// When adding a numbered kind, add its `KindRef` row here and bump the count
/// in `kinds_table_covers_the_numbered_kinds`.
///
/// Every numbered kind, in canonical order. The one place this list lives; a new
/// numbered kind must be added here or it silently escapes `validate` (R-b — a
/// drift surface this table accepts in exchange for not threading a registry
/// through every kind-owning module).
pub(crate) const KINDS: &[KindRef] = &[
    KindRef {
        kind: &SLICE_KIND,
        state_dir: Some(".doctrine/state/slice"),
    },
    KindRef {
        kind: &ADR_KIND.kind,
        state_dir: None,
    },
    KindRef {
        kind: &POLICY_KIND.kind,
        state_dir: None,
    },
    KindRef {
        kind: &STANDARD_KIND.kind,
        state_dir: None,
    },
    KindRef {
        kind: &PRODUCT_SPEC_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &TECH_SPEC_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &REQUIREMENT_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &ISSUE_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &IMPROVEMENT_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &CHORE_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &RISK_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &IDEA_KIND,
        state_dir: None,
    },
    // Review (SL-040) — the 2nd kind with a runtime state tree (baton/lock/cache,
    // PHASE-03+), mirroring slice. Its authored toml is status-LESS (derived,
    // D-C8); the scan reads `.id` via the id-only reader (D2), so a status-less
    // ledger scans cleanly while the strict `Meta` stays untouched.
    KindRef {
        kind: &REVIEW_KIND,
        state_dir: Some(".doctrine/state/review"),
    },
    // REC (SL-042) — the reconciliation-record kind. Status-LESS like review
    // (D-Q3: one REC per act, no lifecycle), so the scan reads `.id` via the
    // id-only reader (meta::read_id). Owns no runtime state tree (state_dir None).
    KindRef {
        kind: &REC_KIND,
        state_dir: None,
    },
    // Knowledge records (SL-059) — six numbered kinds over one engine, each its
    // own tree + reservation namespace. Status-ful (scanned via the standard
    // `meta::Meta` path), one shared `record-NNN.{toml,md}` stem, no runtime state
    // tree. Their `outbound_for` arm (`relation_graph.rs`, L7) co-lands so the
    // KINDS-driven dispatch stays total (a row with no arm panics every debug-build
    // graph scan).
    KindRef {
        kind: &ASSUMPTION_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &DECISION_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &QUESTION_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &CONSTRAINT_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &EVIDENCE_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &HYPOTHESIS_KIND,
        state_dir: None,
    },
    KindRef {
        kind: &CONCEPT_MAP_KIND,
        state_dir: None,
    },
    // Revision (SL-066, ADR-013) — the REV change-axis kind. Status-ful (scanned via
    // the standard id-only reader; its `revision-NNN.toml` carries `status`), stem
    // `revision`, no runtime state tree (state_dir None). Its THREE corpus-walk arms
    // (G1 `priority::partition` REV row, G2 `relation_graph::dep_seq_for` REV arm,
    // G3 `relation_graph::outbound_for` REV arm) co-land with this row, or the
    // debug-build corpus scan panics/mis-classifies the moment a REV is minted.
    KindRef {
        kind: &REV_KIND,
        state_dir: None,
    },
    // RFC (SL-122) — the governance-neutral discussion kind. Status-ful (scanned via
    // the standard meta::Meta path; its `rfc-NNN.toml` carries `status`), stem `rfc`,
    // no runtime state tree. Its `outbound_for` arm co-lands with this row, or the
    // debug-build corpus scan panics the moment an RFC is minted.
    KindRef {
        kind: &RFC_KIND.kind,
        state_dir: None,
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
///
/// `diagnostics` collects schema-agnostic full-TOML parse errors (SL-151 D2):
/// parse as `toml::Value` catches non-contiguous sections and other
/// well-formedness failures that the typed id-only deserialize never sees.
fn scan_kind(
    root: &Path,
    kind: &'static KindRef,
    diagnostics: &mut Vec<String>,
) -> anyhow::Result<KindSnapshot> {
    let tree_root = root.join(kind.kind.dir);

    let mut entities = Vec::new();
    for dir_id in entity::scan_ids(&tree_root)? {
        // The scan path needs only the id (design §5 D2): read it via the id-only
        // reader so review's intentionally status-less toml scans cleanly, while
        // the strict `Meta` (status-bearing readers) is untouched.
        //
        // Schema-agnostic full-Toml parse first (SL-151 D2): catch
        // non-contiguous sections and other well-formedness errors the typed
        // deserialize won't see. The file text is read once; if the full parse
        // fails we push a canonical-id-tagged diagnostic and skip the entity —
        // the diagnostic is the hard error, and the entity is omitted from the
        // snapshot because its metadata is unreadable.
        let name = format!("{dir_id:03}");
        let path = tree_root
            .join(&name)
            .join(format!("{}-{name}.toml", kind.kind.stem));
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("read {stem} {name}", stem = kind.kind.stem))?;
        if let Err(e) = toml::from_str::<toml::Value>(&text) {
            diagnostics.push(format!(
                "{}-{dir_id:03}: TOML parse failed: {e}",
                kind.kind.prefix
            ));
            continue;
        }
        let toml_id = meta::read_id(&tree_root, kind.kind.stem, dir_id, kind.kind.prefix)?;
        entities.push(EntityFacts { dir_id, toml_id });
    }

    let aliases = scan_aliases(&tree_root, kind.kind.stem, kind.kind.prefix)?;
    Ok(KindSnapshot {
        prefix: kind.kind.prefix,
        entities,
        aliases,
    })
}

/// Collect the `NNN-slug` alias symlinks directly under `tree_root`. Each yields
/// the id its name encodes and the declared id of the dir it resolves to. A
/// symlink whose name does not lead with `NNN-` is not an entity alias and is
/// skipped (memory's `mem.*` aliases never appear under a numbered tree anyway).
fn scan_aliases(tree_root: &Path, stem: &str, prefix: &str) -> anyhow::Result<Vec<AliasFacts>> {
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
            .and_then(|target_dir_id| meta::read_id(tree_root, stem, target_dir_id, prefix).ok());

        aliases.push(AliasFacts {
            encoded_id,
            target_toml_id,
        });
    }
    Ok(aliases)
}

/// The id-integrity finding lines for `validate` (ADR-006 D3 detect-half) over an
/// ALREADY-resolved `root` — the per-kind `check_kind` rules as display strings. Split
/// from the printer so the command shell composes these with the SL-048 relation-edge
/// findings (`relation_graph::validate_relations`) WITHOUT this engine module depending
/// on `relation_graph` (which depends back on `integrity` — the cycle the split avoids).
pub(crate) fn id_integrity_findings(root: &Path) -> anyhow::Result<Vec<String>> {
    id_integrity_findings_native(root).map(|fs| fs.into_iter().map(|f| f.message).collect())
}

/// Native [#1 `IdIntegrity`] check — returns [`crate::finding::Finding`] directly (D12
/// re-point). The per-kind `check_kind` rules plus schema-agnostic TOML parse
/// diagnostics, each tagged with `Category::IdIntegrity` and best-effort entity
/// extraction.
pub(crate) fn id_integrity_findings_native(
    root: &Path,
) -> anyhow::Result<Vec<crate::finding::Finding>> {
    use crate::finding::{Category, Finding as DoctorFinding};
    let mut findings = Vec::new();
    let mut diagnostics = Vec::new();
    for kind in KINDS {
        let snap = scan_kind(root, kind, &mut diagnostics)?;
        for f in check_kind(&snap) {
            findings.push(DoctorFinding {
                category: Category::IdIntegrity,
                entity: extract_entity_id(&f.0, kind),
                message: f.0.clone(),
            });
        }
    }
    for diag in diagnostics {
        findings.push(DoctorFinding {
            category: Category::IdIntegrity,
            entity: None,
            message: diag,
        });
    }
    Ok(findings)
}

/// Try to extract a canonical entity id (`PREFIX-NNN`) from a finding message.
/// Best-effort: returns `None` when the message format does not carry the id
/// in a recognisable `PREFIX-NNN:` prefix.
fn extract_entity_id(msg: &str, kind: &KindRef) -> Option<String> {
    let prefix = format!("{}-", kind.kind.prefix);
    if let Some(rest) = msg.strip_prefix(&prefix)
        && let Some(end) = rest.find(':')
    {
        return Some(format!("{}{}", prefix, &rest[..end]));
    }
    None
}

/// The roster of kinds `validate` scanned, for the summary line (so the memory
/// omission is visible, D-A).
pub(crate) fn scanned_kinds() -> String {
    KINDS
        .iter()
        .map(|k| k.kind.prefix)
        .collect::<Vec<_>>()
        .join(", ")
}

// ---------------------------------------------------------------------------
// reseat — the D3 repair backstop (renumber an entity's canonical-id triple).
// ---------------------------------------------------------------------------

/// Resolve a numbered kind by its canonical prefix (`SL` → the slice [`KindRef`]).
pub(crate) fn kind_by_prefix(prefix: &str) -> Option<&'static KindRef> {
    KINDS.iter().find(|k| k.kind.prefix == prefix)
}

/// Validate that a canonical ref (`SL-024`) resolves to a real entity on disk —
/// the forward-edge guard a kind reuses at authoring time (SL-040 §7: `review
/// new` refuses a dangling / unknown-prefix `[target].ref` before minting an RV).
/// Two failure modes, both surfaced by [`parse_canonical_ref`] + a dir probe: an
/// unknown prefix / non-canonical shape, and a well-formed ref to an id with no
/// entity directory (dangling). Read-only.
pub(crate) fn ensure_ref_resolves(root: &Path, reference: &str) -> anyhow::Result<()> {
    let (kind, id) = parse_canonical_ref(reference)?;
    let name = format!("{id:03}");
    let dir = root.join(kind.kind.dir).join(&name);
    anyhow::ensure!(
        fsutil::is_real_dir(&dir),
        "`{reference}` does not resolve to an entity (no {} at {})",
        listing::canonical_id(kind.kind.prefix, id),
        dir.display()
    );
    Ok(())
}

/// Parse a canonical ref (`SL-031`) into its kind and numeric id. Reseat keys on
/// the canonical ref, never a bare number (X2/D7) — the kind disambiguates the
/// per-namespace id.
pub(crate) fn parse_canonical_ref(reference: &str) -> anyhow::Result<(&'static KindRef, u32)> {
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
/// CONTRACT (SL-032 review F-4): the dangler exit is **non-zero even on
/// a fully-completed reseat** — the mutation succeeded, the citations are the
/// human's to fix; `reseat && commit` is therefore wrong, drive it by hand.
/// The mutation is now staged in a sibling temp dir with a single atomic
/// rename as the commit point (IMP-010): a mid-sequence failure before the
/// rename leaves only an orphan `.MMM.tmp` the retry path cleans — never a
/// half-reseated entity at the canonical id.
pub(crate) fn run_reseat(
    path: Option<PathBuf>,
    reference: &str,
    to: Option<u32>,
) -> anyhow::Result<()> {
    let root = root::find(path, &root::default_markers())?;
    let (kind, src_id) = parse_canonical_ref(reference)?;
    let tree_root = root.join(kind.kind.dir);

    let src_name = format!("{src_id:03}");
    let src_dir = tree_root.join(&src_name);
    anyhow::ensure!(
        fsutil::is_real_dir(&src_dir),
        "no {} at {}",
        listing::canonical_id(kind.kind.prefix, src_id),
        src_dir.display()
    );
    // Slug from the authored metadata — the alias name component.
    let slug = meta::read_meta(&tree_root, kind.kind.stem, src_id, kind.kind.prefix)?.slug;

    // The free-id pick: explicit `--to`, else the trunk-aware default (PHASE-02).
    let dst_id = match to {
        Some(t) => t,
        None => entity::next_id(
            &entity::scan_ids(&tree_root)?,
            &git::trunk_entity_ids(&root, kind.kind.dir)?,
        ),
    };
    anyhow::ensure!(
        dst_id != src_id,
        "{} is already seated at {src_name}",
        listing::canonical_id(kind.kind.prefix, src_id)
    );

    let dst_name = format!("{dst_id:03}");
    let dst_dir = tree_root.join(&dst_name);

    // Guard 1 — occupied target (no clobber). `exists` resolves the numeric dir.
    anyhow::ensure!(
        !dst_dir.exists(),
        "id {dst_name} is occupied — refusing to clobber {}",
        dst_dir.display()
    );
    // Guard 2 — live runtime phase state (F3). Only kinds with a `state_dir`
    // (slice) key disposable state by id; reseat does not migrate that tier.
    if let Some(state_dir) = kind.state_dir {
        let state = root.join(state_dir).join(&src_name);
        anyhow::ensure!(
            !state.exists(),
            "{} has live runtime phase state at {} — clear it first (reseat does not own the disposable tier)",
            listing::canonical_id(kind.kind.prefix, src_id),
            state.display()
        );
    }

    // Staging dir — sibling `.MMM.tmp` on the same mount, invisible until commit.
    let tmp_dir = tree_root.join(format!(".{dst_name}.tmp"));
    if tmp_dir.exists() {
        std::fs::remove_dir_all(&tmp_dir)
            .with_context(|| format!("clean stale staging dir {}", tmp_dir.display()))?;
    }

    // --- Mutation (IMP-010: staged in tmp, atomic rename = commit point) ---
    // Step 1: copy src contents into staging dir (invisible).
    fsutil::copy_dir_all(&src_dir, &tmp_dir)
        .with_context(|| format!("copy {} → {}", src_dir.display(), tmp_dir.display()))?;

    // Step 2–3: transform staging dir in place.
    for ext in ["toml", "md"] {
        let from = tmp_dir.join(format!("{}-{src_name}.{ext}", kind.kind.stem));
        let onto = tmp_dir.join(format!("{}-{dst_name}.{ext}", kind.kind.stem));
        if from.exists() {
            std::fs::rename(&from, &onto)
                .with_context(|| format!("rename {} → {}", from.display(), onto.display()))?;
        }
    }
    let toml_path = tmp_dir.join(format!("{}-{dst_name}.toml", kind.kind.stem));
    let text = std::fs::read_to_string(&toml_path)
        .with_context(|| format!("read {}", toml_path.display()))?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("parse {}", toml_path.display()))?;
    doc.as_table_mut()
        .insert("id", toml_edit::value(i64::from(dst_id)));
    fsutil::write_atomic(&toml_path, doc.to_string().as_bytes())
        .with_context(|| format!("write {}", toml_path.display()))?;

    // Step 4: atomic commit — rename(tmp → dst_dir).
    std::fs::rename(&tmp_dir, &dst_dir).with_context(|| {
        format!(
            "commit rename {} → {}",
            tmp_dir.display(),
            dst_dir.display()
        )
    })?;

    // Step 5: swap aliases.
    let old_alias = tree_root.join(format!("{src_name}-{slug}"));
    if matches!(std::fs::symlink_metadata(&old_alias), Ok(m) if m.file_type().is_symlink()) {
        std::fs::remove_file(&old_alias)
            .with_context(|| format!("remove stale alias {}", old_alias.display()))?;
    }
    fsutil::set_symlink(
        &tree_root.join(format!("{dst_name}-{slug}")),
        Path::new(&dst_name),
    )?;

    // Step 6: cleanup src_dir.
    std::fs::remove_dir_all(&src_dir)
        .with_context(|| format!("remove old src dir {}", src_dir.display()))?;

    let old_ref = listing::canonical_id(kind.kind.prefix, src_id);
    let new_ref = listing::canonical_id(kind.kind.prefix, dst_id);
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
pub(crate) fn is_disposable_prose(path: &Path) -> bool {
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

    /// SL-040 D2 (VT-1, validate-path): the review kind's intentionally
    /// status-less toml scans cleanly through `scan_kind`'s id-only reader, so a
    /// review entity is visible to `validate` without seeding a derived status.
    #[test]
    fn scan_kind_reads_a_review_statusless_toml() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let dir = root.join(REVIEW_KIND.dir).join("001");
        std::fs::create_dir_all(&dir).unwrap();
        // No `status` key — review derives it (D-C8).
        std::fs::write(
            dir.join("review-001.toml"),
            "id = 1\nslug = \"s\"\ntitle = \"T\"\n\n[review]\nfacet = \"design\"\n",
        )
        .unwrap();
        let review_kind = kind_by_prefix("RV").expect("RV in KINDS");
        let mut diagnostics = Vec::new();
        let snap = scan_kind(root, review_kind, &mut diagnostics)
            .expect("status-less review scans cleanly");
        assert!(diagnostics.is_empty());
        assert_eq!(snap.entities.len(), 1);
        assert_eq!(snap.entities[0].toml_id, 1);
    }

    /// SL-040 §7: `ensure_ref_resolves` accepts a real target, refuses a dangling
    /// (well-formed but absent) ref and an unknown-prefix ref — the `review new`
    /// forward-edge guard.
    #[test]
    fn ensure_ref_resolves_guards_the_forward_edge() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let dir = root.join(".doctrine/slice/024");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("slice-024.toml"), "id = 24\n").unwrap();

        assert!(ensure_ref_resolves(root, "SL-024").is_ok());
        let dangling = ensure_ref_resolves(root, "SL-099").unwrap_err();
        assert!(
            dangling.to_string().contains("does not resolve"),
            "{dangling}"
        );
        let unknown = ensure_ref_resolves(root, "ZZ-001").unwrap_err();
        assert!(
            unknown.to_string().contains("unknown kind prefix"),
            "{unknown}"
        );
    }

    #[test]
    fn parse_canonical_ref_resolves_kind_and_id() {
        let (kind, id) = parse_canonical_ref("SL-031").expect("valid ref");
        assert_eq!(kind.kind.prefix, "SL");
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
    fn kinds_table_covers_the_numbered_kinds() {
        assert_eq!(KINDS.len(), 23, "add/remove a KindRef row? bump this count");
        let prefixes: Vec<_> = KINDS.iter().map(|k| k.kind.prefix).collect();
        assert_eq!(
            prefixes,
            [
                "SL", "ADR", "POL", "STD", "PRD", "SPEC", "REQ", "ISS", "IMP", "CHR", "RSK", "IDE",
                "RV", "REC", "ASM", "DEC", "QUE", "CON", "EVD", "HYP", "CM", "REV", "RFC"
            ]
        );
        // Slice and review (SL-040) own a runtime state tree (F3 guard surface).
        // REC (SL-042) is status-less but stateless — no runtime tree. The six
        // knowledge kinds (SL-059) are status-ful but stateless — no runtime tree.
        let stateful: Vec<_> = KINDS
            .iter()
            .filter(|k| k.state_dir.is_some())
            .map(|k| k.kind.prefix)
            .collect();
        assert_eq!(stateful, ["SL", "RV"]);
    }

    #[test]
    fn kinds_prefixes_are_corpus_wide_disjoint() {
        // NF-002 / F-A6: every numbered-kind prefix is distinct — the six SL-059
        // additions (ASM/DEC/QUE/CON/EVD/HYP) collide with NO existing corpus prefix. A
        // duplicate prefix here would route two kinds to one namespace.
        use std::collections::BTreeSet;
        let prefixes: Vec<_> = KINDS.iter().map(|k| k.kind.prefix).collect();
        let distinct: BTreeSet<_> = prefixes.iter().copied().collect();
        assert_eq!(
            distinct.len(),
            prefixes.len(),
            "all KINDS prefixes are distinct: {prefixes:?}"
        );
    }

    /// SL-151 D2 (VT-3): scan_kind flags a non-contiguous TOML (duplicate
    /// `[relationships]` header) via the schema-agnostic full parse.
    #[test]
    fn scan_kind_flags_non_contiguous_toml() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        // Use slice (SL) — any numbered kind works.
        let dir = root.join(".doctrine/slice/001");
        std::fs::create_dir_all(&dir).unwrap();
        // Non-contiguous: `[relationships]` appears twice, which toml::Value
        // rejects as a duplicate key.
        std::fs::write(
            dir.join("slice-001.toml"),
            "id = 1\n\
             slug = \"s\"\n\
             title = \"T\"\n\
             status = \"proposed\"\n\
             created = \"2026-01-01\"\n\
             updated = \"2026-01-01\"\n\
             \n\
             [relationships]\n\
             [relationships]\n",
        )
        .unwrap();
        let slice_kind = kind_by_prefix("SL").expect("SL in KINDS");
        let mut diagnostics = Vec::new();
        let snap = scan_kind(root, slice_kind, &mut diagnostics).expect("scan_kind succeeds");
        assert_eq!(
            snap.entities.len(),
            0,
            "unparseable entity is omitted from snapshot"
        );
        assert!(
            !diagnostics.is_empty(),
            "non-contiguous TOML must produce a diagnostic: {diagnostics:?}"
        );
        assert!(
            diagnostics[0].starts_with("SL-001: TOML parse failed:"),
            "diagnostic must be canonical-id tagged: {}",
            diagnostics[0]
        );
    }

    /// SL-151 D2 (VT-4): scan_kind produces no diagnostics on a valid TOML.
    #[test]
    fn scan_kind_no_diagnostics_on_valid_toml() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let dir = root.join(".doctrine/slice/001");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("slice-001.toml"),
            "id = 1\n\
             slug = \"s\"\n\
             title = \"T\"\n\
             status = \"proposed\"\n\
             created = \"2026-01-01\"\n\
             updated = \"2026-01-01\"\n\
             \n\
             [relationships]\n",
        )
        .unwrap();
        let slice_kind = kind_by_prefix("SL").expect("SL in KINDS");
        let mut diagnostics = Vec::new();
        let snap = scan_kind(root, slice_kind, &mut diagnostics).expect("scan_kind succeeds");
        assert_eq!(snap.entities.len(), 1);
        assert!(
            diagnostics.is_empty(),
            "valid TOML must produce no diagnostics: {diagnostics:?}"
        );
    }

    /// SL-151 D2 (VT-4 false-positive guard): scan_kind does NOT flag a TOML
    /// where `[section]` appears inside a string value (valid TOML, not a real
    /// duplicate key).
    #[test]
    fn scan_kind_no_false_positive_on_section_in_string() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let dir = root.join(".doctrine/slice/001");
        std::fs::create_dir_all(&dir).unwrap();
        // `[relationships]` inside a string value — valid TOML, not a duplicate key.
        std::fs::write(
            dir.join("slice-001.toml"),
            "id = 1\n\
             slug = \"s\"\n\
             title = \"T\"\n\
             status = \"proposed\"\n\
             created = \"2026-01-01\"\n\
             updated = \"2026-01-01\"\n\
             note = \"inner [relationships] key\"\n\
             \n\
             [relationships]\n",
        )
        .unwrap();
        let slice_kind = kind_by_prefix("SL").expect("SL in KINDS");
        let mut diagnostics = Vec::new();
        let snap = scan_kind(root, slice_kind, &mut diagnostics).expect("scan_kind succeeds");
        assert_eq!(snap.entities.len(), 1);
        assert!(
            diagnostics.is_empty(),
            "section inside a string value must not be reported: {diagnostics:?}"
        );
    }
}
