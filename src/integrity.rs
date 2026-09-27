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

use crate::entity::{Acquired, Claim, ClaimCtx, Kind};
use crate::kinds::{KINDS, KindRef, parse_canonical_ref};
use crate::{entity, fsutil, git, listing, meta, reserve, root};

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

// ---------------------------------------------------------------------------
// reseat — the D3 repair backstop (renumber an entity's canonical-id triple).
// ---------------------------------------------------------------------------

/// `doctrine reseat <CANONICAL_REF> [--to <NNN>]` — renumber an entity's
/// canonical-id quad (dir name, the `<stem>-NNN.{toml,md}` filenames, the toml
/// `id` field, the `NNN-slug` alias) to the next free id, or to an explicit
/// `--to`. The slug is read leniently ([`meta::read_slug`]), so a status-less
/// kind (review) reseats (ISS-277). Refusals that do not depend on the
/// destination run BEFORE any claim, so a refused reseat leaves no claim behind:
/// an id with live gitignored runtime state is refused (F3 — reseat does not own
/// the disposable tier), and so is a `--to` equal to the source. Inbound prose
/// citations are reported as danglers and force a non-zero exit; prose is never
/// rewritten (ADR-004 outbound-only, D4/R-3).
///
/// CONTRACT (SL-032 review F-4): the dangler exit is **non-zero even on
/// a fully-completed reseat** — the mutation succeeded, the citations are the
/// human's to fix; `reseat && commit` is therefore wrong, drive it by hand.
///
/// The destination is **claimed** through the reservation backend
/// ([`reserve::backend`], SL-269), not merely picked: the default rides
/// [`entity::claim_next_id`]; an explicit `--to` is refused when the candidate set
/// (this tree, sibling trees, reservation refs, trunk) holds it, else claimed once
/// (no clobber, §5.3). The mutation is staged in a sibling `.MMM.tmp` and
/// committed by one `rename(2)` over the claimed **empty** dir (IMP-010), which
/// is the cleanup boundary (RV-406 F-5): before it, failure removes the claim
/// with `remove_dir` only — a populated claim is kept and named — and removes our
/// own staging dir; any reservation ref stays (its id is skipped from then on).
/// After it, the move is committed: nothing is rolled back, the destination is
/// never removed, and the error names what is seated and what remains.
pub(crate) fn run_reseat(
    path: Option<PathBuf>,
    reference: &str,
    to: Option<u32>,
    prompt: reserve::PromptFn,
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
    // Slug from the authored metadata — the alias name component. Lenient: a
    // status-less kind (review) must not trip the strict `Meta` (ISS-277).
    let slug = meta::read_slug(&tree_root, kind.kind.stem, src_id, kind.kind.prefix)?;

    // Guard — live runtime phase state (F3). Only kinds with a `state_dir` key
    // disposable state by id; reseat does not migrate that tier.
    if let Some(state_dir) = kind.state_dir {
        let state = root.join(state_dir).join(&src_name);
        anyhow::ensure!(
            !state.exists(),
            "{} has live runtime phase state at {} — clear it first (reseat does not own the disposable tier)",
            listing::canonical_id(kind.kind.prefix, src_id),
            state.display()
        );
    }
    anyhow::ensure!(
        to != Some(src_id),
        "{} is already seated at {src_name}",
        listing::canonical_id(kind.kind.prefix, src_id)
    );

    // Every destination-independent refusal has run: only now build the backend
    // (which may print the local-fallback signal or prompt) and claim.
    let trunk_ids = git::trunk_entity_ids(&root, kind.kind.dir)?;
    let (claim, mut scan) = reserve::backend(&root, kind.kind, prompt)?;
    let dst_id = reseat_onto(
        &tree_root, kind.kind, src_id, &slug, to, &*claim, &mut *scan, &trunk_ids,
    )?;

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

/// The reseat core below the shell's refusals: claim the destination, stage the
/// renumbered copy, commit it over the claimed empty dir, then swap the alias and
/// drop the source. Returns the seated id. `scan` maps this tree's numeric ids to
/// the full candidate set (a [`reserve::ScanSource`]); `trunk_ids` is constant.
#[expect(
    clippy::too_many_arguments,
    reason = "the shell/core seam: every input is resolved by the shell so the core runs git-free under test"
)]
fn reseat_onto(
    tree_root: &Path,
    kind: &Kind,
    src_id: u32,
    slug: &str,
    to: Option<u32>,
    claim: &dyn Claim,
    scan: &mut dyn FnMut(&[u32]) -> anyhow::Result<Vec<u32>>,
    trunk_ids: &[u32],
) -> anyhow::Result<u32> {
    let (dst_id, dst_dir) = match to {
        None => entity::claim_next_id(claim, tree_root, kind.prefix, trunk_ids, || {
            scan(&entity::scan_ids(tree_root)?)
        })?,
        Some(t) => claim_explicit(claim, tree_root, t, scan, trunk_ids)?,
    };
    let src_name = format!("{src_id:03}");
    let dst_name = format!("{dst_id:03}");
    let src_dir = tree_root.join(&src_name);
    let old_ref = listing::canonical_id(kind.prefix, src_id);
    let new_ref = listing::canonical_id(kind.prefix, dst_id);

    // --- Pre-commit: everything up to and including the commit rename. ---
    // Staging dir — sibling `.MMM.tmp` on the same mount, invisible until commit.
    let tmp_dir = tree_root.join(format!(".{dst_name}.tmp"));
    if let Err(e) = stage_and_commit(kind, src_id, dst_id, &src_dir, &tmp_dir, &dst_dir) {
        let fate = abandon_uncommitted(&dst_dir, &tmp_dir);
        return Err(e.context(format!(
            "reseat {old_ref} → {new_ref} not committed; {fate}; any reservation for \
             {new_ref} is kept (that id is skipped from now on)"
        )));
    }

    // --- Post-commit: the move is committed; never roll back, never touch dst. ---
    let old_alias = tree_root.join(format!("{src_name}-{slug}"));
    let new_alias = tree_root.join(format!("{dst_name}-{slug}"));
    if let Err(e) = finish_committed(&old_alias, &new_alias, &dst_name, &src_dir) {
        let remaining = remaining_after_commit(&old_alias, &new_alias, &dst_name, &src_dir);
        return Err(e.context(format!(
            "reseat committed {old_ref} → {new_ref} at {}, but a post-commit step \
             failed; remaining by hand: {remaining}",
            dst_dir.display()
        )));
    }
    Ok(dst_id)
}

/// Claim an explicit `--to` destination: refuse before any claim when the
/// candidate set holds it (a sibling tree's pre-slice dir is invisible to the ref
/// CAS), else claim once — `AlreadyHeld` refuses with the same wording.
fn claim_explicit(
    claim: &dyn Claim,
    tree_root: &Path,
    to: u32,
    scan: &mut dyn FnMut(&[u32]) -> anyhow::Result<Vec<u32>>,
    trunk_ids: &[u32],
) -> anyhow::Result<(u32, PathBuf)> {
    let dir = tree_root.join(format!("{to:03}"));
    let occupied = || {
        anyhow::anyhow!(
            "id {to:03} is occupied or reserved (this tree, a sibling tree, a reservation \
             ref, or trunk) — refusing to clobber {}",
            dir.display()
        )
    };
    let held = scan(&entity::scan_ids(tree_root)?)?;
    if held.contains(&to) || trunk_ids.contains(&to) {
        return Err(occupied());
    }
    match claim.claim(&ClaimCtx { dir: &dir, id: to })? {
        Acquired::Won => Ok((to, dir)),
        Acquired::AlreadyHeld => Err(occupied()),
    }
}

/// Stage the renumbered copy in `tmp_dir` and commit it by renaming over the
/// claimed empty `dst_dir` — the single commit point (IMP-010). `rename(2)`
/// replaces an empty dir atomically and fails on a populated one, so a claim
/// someone has filled is never overwritten.
fn stage_and_commit(
    kind: &Kind,
    src_id: u32,
    dst_id: u32,
    src_dir: &Path,
    tmp_dir: &Path,
    dst_dir: &Path,
) -> anyhow::Result<()> {
    let src_name = format!("{src_id:03}");
    let dst_name = format!("{dst_id:03}");
    if tmp_dir.exists() {
        std::fs::remove_dir_all(tmp_dir)
            .with_context(|| format!("clean stale staging dir {}", tmp_dir.display()))?;
    }
    // Step 1: copy src contents into staging dir (invisible).
    fsutil::copy_dir_all(src_dir, tmp_dir)
        .with_context(|| format!("copy {} → {}", src_dir.display(), tmp_dir.display()))?;

    // Step 2–3: transform staging dir in place.
    for ext in ["toml", "md"] {
        let from = tmp_dir.join(format!("{}-{src_name}.{ext}", kind.stem));
        let onto = tmp_dir.join(format!("{}-{dst_name}.{ext}", kind.stem));
        if from.exists() {
            std::fs::rename(&from, &onto)
                .with_context(|| format!("rename {} → {}", from.display(), onto.display()))?;
        }
    }
    let toml_path = tmp_dir.join(format!("{}-{dst_name}.toml", kind.stem));
    let text = std::fs::read_to_string(&toml_path)
        .with_context(|| format!("read {}", toml_path.display()))?;
    let mut doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("parse {}", toml_path.display()))?;
    doc.as_table_mut()
        .insert("id", toml_edit::value(i64::from(dst_id)));
    fsutil::write_atomic(&toml_path, doc.to_string().as_bytes())
        .with_context(|| format!("write {}", toml_path.display()))?;

    // Step 4: atomic commit — rename(tmp → the claimed empty dst_dir).
    std::fs::rename(tmp_dir, dst_dir).with_context(|| {
        format!(
            "commit rename {} → {}",
            tmp_dir.display(),
            dst_dir.display()
        )
    })
}

/// Pre-commit cleanup (RV-406 F-5): remove the claimed dir with `remove_dir`
/// ONLY — never `remove_dir_all`, a populated claim is someone's and is kept —
/// and best-effort remove our own staging dir. Returns the fate of each, naming
/// anything left behind (STD-003).
fn abandon_uncommitted(dst_dir: &Path, tmp_dir: &Path) -> String {
    let claim = match std::fs::remove_dir(dst_dir) {
        Ok(()) => format!("claimed dir {} removed", dst_dir.display()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            format!("claimed dir {} already gone", dst_dir.display())
        }
        Err(e) => format!("claimed dir {} kept ({e})", dst_dir.display()),
    };
    let staging = match std::fs::symlink_metadata(tmp_dir) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => format!("; staging dir {} not inspected ({e})", tmp_dir.display()),
        Ok(_) => match std::fs::remove_dir_all(tmp_dir) {
            Ok(()) => format!("; staging dir {} removed", tmp_dir.display()),
            Err(e) => format!("; staging dir {} left behind ({e})", tmp_dir.display()),
        },
    };
    format!("{claim}{staging}")
}

/// The post-commit steps: swap the alias, then drop the source dir.
fn finish_committed(
    old_alias: &Path,
    new_alias: &Path,
    dst_name: &str,
    src_dir: &Path,
) -> anyhow::Result<()> {
    if is_symlink(old_alias) {
        std::fs::remove_file(old_alias)
            .with_context(|| format!("remove stale alias {}", old_alias.display()))?;
    }
    fsutil::set_symlink(new_alias, Path::new(dst_name))?;
    std::fs::remove_dir_all(src_dir)
        .with_context(|| format!("remove old src dir {}", src_dir.display()))
}

/// What a failed post-commit sequence left for the human, read back from disk.
fn remaining_after_commit(
    old_alias: &Path,
    new_alias: &Path,
    dst_name: &str,
    src_dir: &Path,
) -> String {
    let mut left = Vec::new();
    if is_symlink(old_alias) {
        left.push(format!("remove stale alias {}", old_alias.display()));
    }
    if std::fs::read_link(new_alias).ok().as_deref() != Some(Path::new(dst_name)) {
        left.push(format!("point alias {} at {dst_name}", new_alias.display()));
    }
    if std::fs::symlink_metadata(src_dir).is_ok() {
        left.push(format!("remove source dir {}", src_dir.display()));
    }
    if left.is_empty() {
        "nothing".to_owned()
    } else {
        left.join("; ")
    }
}

fn is_symlink(path: &Path) -> bool {
    matches!(std::fs::symlink_metadata(path), Ok(m) if m.file_type().is_symlink())
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
    use crate::kinds::{REVIEW_KIND, kind_by_prefix};

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
        assert_eq!(KINDS.len(), 24, "add/remove a KindRef row? bump this count");
        let prefixes: Vec<_> = KINDS.iter().map(|k| k.kind.prefix).collect();
        assert_eq!(
            prefixes,
            [
                "SL", "ADR", "POL", "STD", "PRD", "SPEC", "REQ", "ISS", "IMP", "CHR", "RSK", "IDE",
                "RV", "REC", "ASM", "DEC", "QUE", "CON", "EVD", "HYP", "CPT", "CM", "REV", "RFC"
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
        // NF-002 / F-A6: every numbered-kind prefix is distinct — the seven SL-059
        // additions (ASM/DEC/QUE/CON/EVD/HYP/CPT) collide with NO existing corpus prefix. A
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

    // --- SL-269 PHASE-03: reseat claims its destination ---

    use crate::kinds::SLICE_KIND;
    use crate::test_support::LinkedTrees;

    /// The clone-local reservation namespace (`reserve`'s private constant; the
    /// ref shape is the contract under test here).
    const LOCAL_REF_NS: &str = "refs/doctrine/reservation-local";

    /// The injected fallback prompt: never opt in (no `set_var`, ISS-483).
    fn never(_: &str) -> anyhow::Result<bool> {
        Ok(false)
    }

    /// Seed a status-bearing slice `id` (slug `moved`) under `root`'s slice tree.
    fn seed_slice(root: &Path, id: u32) {
        let name = format!("{id:03}");
        let dir = root.join(SLICE_KIND.dir).join(&name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("slice-{name}.toml")),
            format!("id = {id}\nslug = \"moved\"\ntitle = \"T\"\nstatus = \"proposed\"\n"),
        )
        .unwrap();
        std::fs::write(dir.join(format!("slice-{name}.md")), "# body\n").unwrap();
    }

    fn git_in(dir: &Path, args: &[&str]) -> std::process::Output {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .expect("spawn git");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        out
    }

    /// ISS-281: a core-driven test needs a NON-git root; assert it rather than
    /// trusting `TMPDIR`.
    fn assert_outside_git(root: &Path) {
        assert!(
            git::toplevel_and_prefix(root).unwrap().is_none(),
            "precondition: {} must be outside any git worktree (ISS-281)",
            root.display()
        );
    }

    /// SL-269 VT-3: `--to` onto a sibling tree's dir, or onto an id a local
    /// reservation ref holds, is refused before any claim; the source is intact.
    #[test]
    fn reseat_to_a_sibling_held_id_refuses() {
        let lt = LinkedTrees::new("");
        let root_a = lt.root(&lt.a);
        seed_slice(&root_a, 31);
        let slices_a = root_a.join(SLICE_KIND.dir);

        // Case 1: tree b holds an uncommitted `045` dir.
        std::fs::create_dir_all(lt.root(&lt.b).join(SLICE_KIND.dir).join("045")).unwrap();
        let err = run_reseat(Some(root_a.clone()), "SL-031", Some(45), never).unwrap_err();
        assert!(err.to_string().contains("occupied"), "{err:#}");

        // Case 2: a clone-local reservation ref holds `046`.
        let head = String::from_utf8(git_in(&lt.main, &["rev-parse", "HEAD"]).stdout).unwrap();
        git_in(
            &lt.main,
            &["update-ref", &format!("{LOCAL_REF_NS}/SL/046"), head.trim()],
        );
        let err = run_reseat(Some(root_a.clone()), "SL-031", Some(46), never).unwrap_err();
        assert!(err.to_string().contains("occupied"), "{err:#}");

        assert!(slices_a.join("031/slice-031.toml").is_file());
        assert!(!slices_a.join("045").exists());
        assert!(!slices_a.join("046").exists());
    }

    /// SL-269 VT-3: the default destination skips a sibling tree's ids, and is
    /// claimed (a clone-local reservation ref now holds it).
    #[test]
    fn reseat_default_skips_sibling_ids() {
        let lt = LinkedTrees::new("");
        let root_a = lt.root(&lt.a);
        seed_slice(&root_a, 31);
        std::fs::create_dir_all(lt.root(&lt.b).join(SLICE_KIND.dir).join("050")).unwrap();

        run_reseat(Some(root_a.clone()), "SL-031", None, never).unwrap();

        let slices_a = root_a.join(SLICE_KIND.dir);
        assert!(slices_a.join("051/slice-051.toml").is_file());
        assert!(!slices_a.join("031").exists());
        git_in(
            &lt.main,
            &["rev-parse", "--verify", &format!("{LOCAL_REF_NS}/SL/051")],
        );
    }

    /// A claim double that wins by creating the dir AND populating it — someone
    /// else's bytes landed in the claim before the commit rename.
    struct PopulatingClaim;
    impl Claim for PopulatingClaim {
        fn claim(&self, ctx: &ClaimCtx<'_>) -> anyhow::Result<Acquired> {
            std::fs::create_dir(ctx.dir)?;
            std::fs::write(ctx.dir.join("foreign.txt"), "not ours")?;
            Ok(Acquired::Won)
        }
    }

    /// The identity scan: a non-git tree's candidate set is its own ids.
    fn identity(local: &[u32]) -> anyhow::Result<Vec<u32>> {
        Ok(local.to_vec())
    }

    /// SL-269 VT-5 (RV-406 F-5): the commit rename fails onto a populated claim;
    /// pre-commit cleanup uses `remove_dir` only, so the foreign bytes survive and
    /// the error names the kept dir. Our staging dir is removed; the source is intact.
    #[test]
    fn reseat_keeps_a_populated_claim_dir() {
        let dir = tempfile::tempdir().unwrap();
        assert_outside_git(dir.path());
        seed_slice(dir.path(), 31);
        let tree = dir.path().join(SLICE_KIND.dir);

        let err = reseat_onto(
            &tree,
            &SLICE_KIND,
            31,
            "moved",
            Some(45),
            &PopulatingClaim,
            &mut identity,
            &[],
        )
        .unwrap_err();
        let dst = tree.join("045");
        let msg = err.to_string();
        assert!(msg.contains("not committed"), "{err:#}");
        assert!(msg.contains(&dst.display().to_string()), "{err:#}");
        assert!(msg.contains("kept"), "{err:#}");
        assert!(dst.join("foreign.txt").is_file());
        assert!(!tree.join(".045.tmp").exists());
        assert!(tree.join("031/slice-031.toml").is_file());
    }

    /// SL-269 VT-5: a failure after the commit rename (a regular file squats the
    /// new alias) is a committed move — the destination stays seated, nothing is
    /// rolled back, and the error names what remains.
    #[test]
    fn reseat_post_commit_failure_leaves_destination() {
        let dir = tempfile::tempdir().unwrap();
        assert_outside_git(dir.path());
        seed_slice(dir.path(), 31);
        let tree = dir.path().join(SLICE_KIND.dir);
        std::fs::write(tree.join("045-moved"), "squatter").unwrap();

        let err = reseat_onto(
            &tree,
            &SLICE_KIND,
            31,
            "moved",
            Some(45),
            &entity::LocalFs,
            &mut identity,
            &[],
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("reseat committed SL-031 → SL-045"), "{err:#}");
        assert!(msg.contains("remove source dir"), "{err:#}");
        let toml = std::fs::read_to_string(tree.join("045/slice-045.toml")).unwrap();
        assert!(toml.contains("id = 45"), "{toml}");
        assert!(
            tree.join("031").is_dir(),
            "source removal follows the alias step"
        );
    }

    /// A staging failure with the claim still empty removes the claim: no
    /// destination dir, no staging dir, source intact.
    #[test]
    fn reseat_staging_failure_removes_an_empty_claim() {
        let dir = tempfile::tempdir().unwrap();
        assert_outside_git(dir.path());
        seed_slice(dir.path(), 31);
        let tree = dir.path().join(SLICE_KIND.dir);
        // Corrupt the source toml so the staged rewrite fails to parse.
        std::fs::write(tree.join("031/slice-031.toml"), "id = [").unwrap();

        let err = reseat_onto(
            &tree,
            &SLICE_KIND,
            31,
            "moved",
            Some(45),
            &entity::LocalFs,
            &mut identity,
            &[],
        )
        .unwrap_err();
        assert!(err.to_string().contains("removed"), "{err:#}");
        assert!(!tree.join("045").exists());
        assert!(!tree.join(".045.tmp").exists());
        assert!(tree.join("031/slice-031.toml").is_file());
    }
}
