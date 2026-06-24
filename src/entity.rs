// SPDX-License-Identifier: GPL-3.0-only
//! Kind-agnostic directory-entity scaffolding engine.
//!
//! One engine materialises every directory entity (slice, design-doc sibling,
//! later drift/spec) from a `Kind` descriptor. The engine is kind-blind: the
//! claim is behind the `claim` seam (reservation-spec § Code seam), the
//! fileset is a `Kind`-supplied function (not a frozen pair — slice-002 M3),
//! and placement is a closed `MaterialiseRequest` enum (never a `reserve: bool`).
//!
//! Pure/imperative split (slices-spec § Architecture): id, slug and the fileset
//! are decided from inputs; only `claim` and the writes touch disk, and the
//! writer is the *sole* joiner of descriptor paths to the filesystem (H1).

use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};

use crate::fsutil;

/// Bounded retries for the reservation claim loop.
const MAX_CLAIM_RETRIES: u32 = 128;

// ---------------------------------------------------------------------------
// The `claim` seam
// ---------------------------------------------------------------------------

/// Outcome of an atomic claim: this caller created it, or another agent already
/// holds it.
pub(crate) enum Acquired {
    Won,
    AlreadyHeld,
}

/// What an atomic claim arbitrates over: the directory to create, plus the
/// numeric `id` the future `git-ref` backend reads as its ref segment. The
/// named path does not ride this — it has no `id` (D9).
pub(crate) struct ClaimCtx<'a> {
    pub(crate) dir: &'a Path,
    #[expect(
        dead_code,
        reason = "GitRef reads ctx.id as the ref segment in PHASE-03 (SL-148)"
    )]
    pub(crate) id: u32,
}

/// The one impure-critical operation, behind a one-method trait so the future
/// `git-ref` backend drops in without a Kind-caller rewrite (reservation-spec).
pub(crate) trait Claim {
    /// Atomic, exclusive claim. `Won` if this caller created `ctx.dir`;
    /// `AlreadyHeld` if another agent won the race. Only this op arbitrates.
    fn claim(&self, ctx: &ClaimCtx<'_>) -> anyhow::Result<Acquired>;
}

/// The local-filesystem backend: the `mkdir` is the claim (D1 — the dir *is*
/// the claim). Lifted verbatim from the old `reserve_create`, so the slice-001
/// retry test stays green.
pub(crate) struct LocalFs;

impl Claim for LocalFs {
    fn claim(&self, ctx: &ClaimCtx<'_>) -> anyhow::Result<Acquired> {
        match fs::create_dir(ctx.dir) {
            Ok(()) => Ok(Acquired::Won),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(Acquired::AlreadyHeld),
            Err(e) => Err(e).with_context(|| format!("Failed to claim {}", ctx.dir.display())),
        }
    }
}

// ---------------------------------------------------------------------------
// The `Kind` descriptor
// ---------------------------------------------------------------------------

/// A `Kind` is *data*, not a trait: one dispatch site, no per-kind state (D2).
/// Placement is no longer a `Kind` field — it is a runtime `MaterialiseRequest`,
/// because a named entity carries its uid only at call time (D8).
#[derive(serde::Serialize)]
pub(crate) struct Kind {
    /// Entity-tree root, relative to the project root, e.g. `.doctrine/slice`.
    /// Also the base every artifact path is joined to (H1).
    pub dir: &'static str,
    /// Canonical-id prefix, e.g. `SL` → `SL-003` (the `{{ref}}` token). Unused
    /// by named kinds, which have no numeric canonical id.
    pub prefix: &'static str,
    /// File stem for `<stem>-NNN.toml` / `<stem>-NNN.md` file names. Empty for
    /// sub-kinds that nest under a parent entity's numeric directory.
    #[serde(skip)]
    pub stem: &'static str,
    /// Fileset as a function — kind-supplied, never a frozen file count (D3).
    #[serde(skip)]
    pub scaffold: fn(&ScaffoldCtx<'_>) -> anyhow::Result<Fileset>,
}

/// The resolved context a `scaffold` renders from. Pure over these inputs plus
/// compile-time-embedded template text (M4): no disk, clock, git, or root. Only
/// numeric entities ride this context (named entities render eagerly via seam A —
/// `materialise_named`), so it carries the numeric `id` and canonical `{{ref}}`
/// directly: `<id>` dirs and `<canonical>` refs read them as fields.
pub(crate) struct ScaffoldCtx<'a> {
    pub id: u32,
    pub canonical: &'a str,
    pub slug: &'a str,
    pub title: &'a str,
    pub date: &'a str,
}

/// One file or symlink in a fileset. `rel_path` is *relative to the entity-tree
/// root* (`Kind.dir`) — the engine is the sole joiner and rejects absolute
/// paths and any `..` that escapes the tree before writing (H1).
pub(crate) enum Artifact {
    File { rel_path: PathBuf, body: String },
    Symlink { rel_path: PathBuf, target: String },
}

/// A `Kind`'s fileset — `Vec`, so the engine never hardcodes a count (D3).
pub(crate) type Fileset = Vec<Artifact>;

/// Caller-supplied scaffold inputs. The engine fills the identity (id /
/// canonical / name) and dir; placement (and any parent id / name) is the
/// `MaterialiseRequest`, not an input field.
pub(crate) struct Inputs<'a> {
    pub slug: &'a str,
    pub title: &'a str,
    pub date: &'a str,
}

/// Where to place a numeric entity, resolved at call time — `Kind` is const, but
/// placement is per-call (D8). A closed enum: a new placement is a compiler-forced
/// variant. (Named placement does not flow through here — see `materialise_named`.)
pub(crate) enum MaterialiseRequest {
    /// Allocate a fresh reserved numeric id (slice, later spec).
    Fresh,
    /// Create file(s) under an existing numeric parent (design / plan / notes).
    InExisting { id: u32 },
}

/// A materialised entity's owned identity: a numbered entity owns its canonical
/// `{{ref}}` string, a named entity owns its name. The shell keeps this past the
/// borrowed `Inputs`, so it is owned, not borrowed (D9).
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum OwnedEntityId {
    Numbered { id: u32, canonical: String },
    Named { name: String },
}

impl OwnedEntityId {
    /// The numeric id, or `None` for a named entity — for callers that still
    /// speak in ids (the slice CLI prints `out.eid.numeric_id()`).
    pub(crate) fn numeric_id(&self) -> Option<u32> {
        match self {
            OwnedEntityId::Numbered { id, .. } => Some(*id),
            OwnedEntityId::Named { .. } => None,
        }
    }
}

/// What a successful materialisation yields: the entity's owned identity and its
/// dir on disk.
#[derive(Debug)]
pub(crate) struct Materialised {
    pub eid: OwnedEntityId,
    pub dir: PathBuf,
}

// ---------------------------------------------------------------------------
// Ext enum + path helpers
// ---------------------------------------------------------------------------

#[derive(Copy, Clone)]
pub(crate) enum Ext {
    Toml,
    Md,
}

fn make_file_name(kind: &Kind, id: u32, ext: Ext) -> PathBuf {
    debug_assert!(
        !kind.stem.is_empty(),
        "{}: stem-less kind {}",
        module_path!(),
        kind.prefix
    );
    let n = format!("{id:03}");
    let file = match ext {
        Ext::Toml => format!("{}-{n}.toml", kind.stem),
        Ext::Md => format!("{}-{n}.md", kind.stem),
    };
    PathBuf::from(&n).join(file)
}

pub(crate) fn id_path(root: &Path, kind: &Kind, id: u32, ext: Ext) -> PathBuf {
    root.join(kind.dir).join(make_file_name(kind, id, ext))
}

pub(crate) fn rel_path(kind: &Kind, id: u32, ext: Ext) -> PathBuf {
    make_file_name(kind, id, ext)
}

// ---------------------------------------------------------------------------
// Pure helpers: id, slug
// ---------------------------------------------------------------------------

/// Next id from a directory listing: `max + 1`, or `1` when empty. Gaps are
/// not back-filled — the id is monotonic (slices-spec § Id allocation).
pub(crate) fn candidate_id(existing: &[u32]) -> u32 {
    existing.iter().copied().max().map_or(1, |m| m + 1)
}

/// Next id across the union of `local` (working-tree) and `trunk` ids:
/// `max(local ∪ trunk) + 1`, or `1` when both are empty. Pure — the trunk ids
/// are read once at the shell edge (`git::trunk_entity_ids`) and passed in, so
/// two divergent worktrees mint non-colliding ids (ADR-006 D3). Delegates to
/// [`candidate_id`] for the single `max+1` rule; `next_id(local, &[])` is
/// byte-identical to `candidate_id(local)` (INV-1 behaviour preservation).
pub(crate) fn next_id(local: &[u32], trunk: &[u32]) -> u32 {
    let union: Vec<u32> = local.iter().copied().chain(trunk.iter().copied()).collect();
    candidate_id(&union)
}

/// Derive a slug from a title: lowercase, runs of whitespace/`-`/`_` collapse
/// to a single `-`, every other non-alphanumeric is stripped, no edge dashes.
pub(crate) fn derive_slug(title: &str) -> String {
    let mut slug = String::new();
    let mut pending_dash = false;
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            pending_dash = false;
            slug.push(ch.to_ascii_lowercase());
        } else if ch.is_whitespace() || ch == '-' || ch == '_' {
            pending_dash = true;
        }
        // any other character is stripped
    }
    slug
}

/// Numeric entity ids present directly under `tree_root` (symlinks and files
/// ignored). A missing directory yields an empty listing.
pub(crate) fn scan_ids(tree_root: &Path) -> anyhow::Result<Vec<u32>> {
    let mut ids = Vec::new();
    let entries = match fs::read_dir(tree_root) {
        Ok(entries) => entries,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(ids),
        Err(e) => {
            return Err(e).with_context(|| format!("Failed to read {}", tree_root.display()));
        }
    };
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        if let Some(name) = entry.file_name().to_str()
            && let Ok(n) = name.parse::<u32>()
        {
            ids.push(n);
        }
    }
    Ok(ids)
}

/// Entity names present directly under `tree_root` — every real subdirectory,
/// whatever its name (numeric *or* `mem_…`); symlinks and files are ignored, and
/// a missing directory yields an empty listing. Sibling to `scan_ids`, which is
/// numeric-only and would skip a named (e.g. `mem_…`) dir (finding 1).
pub(crate) fn scan_named(tree_root: &Path) -> anyhow::Result<Vec<String>> {
    let mut names = Vec::new();
    let entries = match fs::read_dir(tree_root) {
        Ok(entries) => entries,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(names),
        Err(e) => {
            return Err(e).with_context(|| format!("Failed to read {}", tree_root.display()));
        }
    };
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        if let Some(name) = entry.file_name().to_str() {
            names.push(name.to_string());
        }
    }
    Ok(names)
}

// ---------------------------------------------------------------------------
// The materialise loop
// ---------------------------------------------------------------------------

/// Materialise `kind` under `project_root`. Dispatches on the placement
/// `request`: allocate a fresh reserved id, or create files under an existing
/// numeric parent. Returns the owned identity and entity dir. (Named placement
/// rides the separate `materialise_named` seam — memory's record fields exceed
/// `ScaffoldCtx`, so it renders eagerly and hands a pre-built fileset.)
pub(crate) fn materialise(
    kind: &Kind,
    claim: &dyn Claim,
    project_root: &Path,
    request: &MaterialiseRequest,
    inputs: &Inputs<'_>,
    trunk_ids: &[u32],
) -> anyhow::Result<Materialised> {
    let tree_root = project_root.join(kind.dir);
    // The entity-tree root; the non-recursive claim mkdir below needs it to
    // exist (the first-ever-entity case).
    fs::create_dir_all(&tree_root)
        .with_context(|| format!("Failed to create {}", tree_root.display()))?;

    match *request {
        MaterialiseRequest::Fresh => {
            allocate_fresh(kind, claim, &tree_root, inputs, trunk_ids, || {
                scan_ids(&tree_root)
            })
        }
        MaterialiseRequest::InExisting { id } => create_in_existing(kind, &tree_root, id, inputs),
    }
}

/// Reserved top-level placement (slice, adr, …): claim the next id with a
/// bounded retry loop, then scaffold from the `Kind`'s `ScaffoldCtx` template.
fn allocate_fresh(
    kind: &Kind,
    claim: &dyn Claim,
    tree_root: &Path,
    inputs: &Inputs<'_>,
    trunk_ids: &[u32],
    scan: impl FnMut() -> anyhow::Result<Vec<u32>>,
) -> anyhow::Result<Materialised> {
    claim_fresh_id(
        claim,
        tree_root,
        kind.prefix,
        trunk_ids,
        scan,
        |id, canonical| {
            let ctx = ScaffoldCtx {
                id,
                canonical,
                slug: inputs.slug,
                title: inputs.title,
                date: inputs.date,
            };
            (kind.scaffold)(&ctx)
        },
    )
}

/// Materialise a fresh-numbered entity from a **pre-built** fileset — the numbered
/// twin of [`materialise_named`] (seam A). For a kind whose fileset depends on more
/// than `ScaffoldCtx` carries (review: facet / target / phase), render the fileset
/// eagerly in the kind module and hand it here; the `build` closure receives the
/// claimed `id` and `canonical` ref so id-bearing paths/bodies resolve. Shares the
/// claim-retry + H2 cleanup core with [`allocate_fresh`].
pub(crate) fn materialise_fresh_prebuilt(
    claim: &dyn Claim,
    project_root: &Path,
    dir: &str,
    prefix: &str,
    trunk_ids: &[u32],
    build: impl FnMut(u32, &str) -> anyhow::Result<Fileset>,
) -> anyhow::Result<Materialised> {
    let tree_root = project_root.join(dir);
    fs::create_dir_all(&tree_root)
        .with_context(|| format!("Failed to create {}", tree_root.display()))?;
    claim_fresh_id(
        claim,
        &tree_root,
        prefix,
        trunk_ids,
        || scan_ids(&tree_root),
        build,
    )
}

/// The shared claim-retry + write + H2-cleanup core for fresh-numbered placement.
/// `scan` re-reads the local tree each retry (recovering a lost claim race);
/// `trunk_ids` is constant (read once at the shell edge, D-b). `build` renders the
/// fileset for the won `(id, canonical)`. A `Won` claim owns the dir, so any
/// build/write failure removes it — no ghost entity survives (H2).
fn claim_fresh_id(
    claim: &dyn Claim,
    tree_root: &Path,
    prefix: &str,
    trunk_ids: &[u32],
    mut scan: impl FnMut() -> anyhow::Result<Vec<u32>>,
    mut build: impl FnMut(u32, &str) -> anyhow::Result<Fileset>,
) -> anyhow::Result<Materialised> {
    for _ in 0..MAX_CLAIM_RETRIES {
        let id = next_id(&scan()?, trunk_ids);
        let name = format!("{id:03}");
        let dir = tree_root.join(&name);
        let ctx = ClaimCtx { dir: &dir, id };
        match claim.claim(&ctx)? {
            Acquired::Won => {
                let canonical = format!("{prefix}-{name}");
                let written = build(id, &canonical).and_then(|fs| write_fileset(tree_root, &fs));
                return match written {
                    Ok(()) => Ok(Materialised {
                        eid: OwnedEntityId::Numbered { id, canonical },
                        dir,
                    }),
                    Err(e) => {
                        // Won ⟹ we created `dir` ⟹ a partial scaffold is our mess
                        // to clean (H2). Best-effort; the build error is surfaced.
                        drop(fs::remove_dir_all(&dir));
                        Err(e)
                    }
                };
            }
            Acquired::AlreadyHeld => {} // lost the race; recompute and retry
        }
    }
    bail!("Could not reserve an id after {MAX_CLAIM_RETRIES} attempts");
}

/// Sub-artefact placement (design doc, later phases): no claim, no id alloc.
/// Resolve the existing numeric parent (err if absent), refuse to clobber,
/// then write. The parent `id` comes straight from the request.
fn create_in_existing(
    kind: &Kind,
    tree_root: &Path,
    id: u32,
    inputs: &Inputs<'_>,
) -> anyhow::Result<Materialised> {
    let name = format!("{id:03}");
    let dir = tree_root.join(&name);
    if !dir.is_dir() {
        bail!("Parent entity {name} not found at {}", dir.display());
    }
    let canonical = format!("{}-{name}", kind.prefix);
    let ctx = ScaffoldCtx {
        id,
        canonical: &canonical,
        slug: inputs.slug,
        title: inputs.title,
        date: inputs.date,
    };
    let fileset = (kind.scaffold)(&ctx)?;
    refuse_clobber(tree_root, &fileset)?; // no silent clobber (D7)
    write_fileset(tree_root, &fileset)?;
    Ok(Materialised {
        eid: OwnedEntityId::Numbered { id, canonical },
        dir,
    })
}

/// Materialise a named entity from a *pre-built* fileset (seam A — memory's
/// record fields exceed `ScaffoldCtx`, so it renders eagerly in `memory.rs` and
/// hands the fileset here rather than riding `Kind.scaffold`). Creates the entity
/// tree, then claims and writes through the shared `claim_and_write_named` core.
/// This is the *only* named placement path (the `MaterialiseRequest`/`ScaffoldCtx`
/// named arm was retired — seam A subsumed it).
pub(crate) fn materialise_named(
    project_root: &Path,
    dir: &str,
    name: &str,
    fileset: &Fileset,
) -> anyhow::Result<Materialised> {
    let tree_root = project_root.join(dir);
    fs::create_dir_all(&tree_root)
        .with_context(|| format!("Failed to create {}", tree_root.display()))?;
    let entity_dir = claim_and_write_named(&tree_root, name, fileset)?;
    Ok(Materialised {
        eid: OwnedEntityId::Named {
            name: name.to_string(),
        },
        dir: entity_dir,
    })
}

/// The claim+write+H2 core of the named path. The named entity has no numeric
/// `id`, so it claims `tree_root/<name>` with an inline `mkdir` rather than the
/// `Claim` seam (D9 — the seam carries `ClaimCtx{dir,id}` for the numeric path
/// only). A won dir writes the fileset transactionally and, on a write failure,
/// removes the won dir (Won ⟹ ours to clean, H2 — as in `allocate_fresh`); an
/// already-existing dir is a duplicate name and a hard error. Returns the entity dir.
fn claim_and_write_named(
    tree_root: &Path,
    name: &str,
    fileset: &Fileset,
) -> anyhow::Result<PathBuf> {
    let dir = tree_root.join(name);
    match fs::create_dir(&dir) {
        Ok(()) => match write_fileset(tree_root, fileset) {
            Ok(()) => Ok(dir),
            Err(e) => {
                drop(fs::remove_dir_all(&dir));
                Err(e)
            }
        },
        Err(e) if e.kind() == ErrorKind::AlreadyExists => bail!("entity {name} already exists"),
        Err(e) => Err(e).with_context(|| format!("Failed to claim {}", dir.display())),
    }
}

/// Refuse if any artifact target already exists (file-creating sub-artefacts
/// only — the engine materialises filesets, not row appends / table mutations).
fn refuse_clobber(tree_root: &Path, fileset: &Fileset) -> anyhow::Result<()> {
    for art in fileset {
        let abs = fsutil::safe_join(tree_root, artifact_rel(art))?;
        if abs.exists() {
            bail!("Refusing to overwrite existing {}", abs.display());
        }
    }
    Ok(())
}

/// Write every artifact under `tree_root` transactionally: on any failure,
/// every file/symlink and every directory component *this call* created is
/// undone, leaving the parent exactly as it was pre-call (D4 — discharges the
/// slice-003 `[M]` debt). The sub-artefact writer cannot `remove_dir_all` a
/// parent it does not own, so it tracks precisely what it made and unwinds
/// that. Pre-existing dirs and dirs a concurrent writer populated are left
/// intact. This is the sole joiner of descriptor paths to the filesystem (H1).
fn write_fileset(tree_root: &Path, fileset: &Fileset) -> anyhow::Result<()> {
    let mut created_paths: Vec<PathBuf> = Vec::new(); // files AND symlinks, in order
    let mut created_dirs: Vec<PathBuf> = Vec::new();
    match write_fileset_tracked(tree_root, fileset, &mut created_paths, &mut created_dirs) {
        Ok(()) => Ok(()),
        Err(e) => {
            rollback(&created_paths, &created_dirs);
            Err(e)
        }
    }
}

/// The forward pass: create dirs component-wise and write artifacts, recording
/// every path created so the caller can unwind on error. A created path is
/// tracked *before* its content is written, so a mid-write failure still
/// unlinks the just-created (empty/partial) file.
fn write_fileset_tracked(
    tree_root: &Path,
    fileset: &Fileset,
    created_paths: &mut Vec<PathBuf>,
    created_dirs: &mut Vec<PathBuf>,
) -> anyhow::Result<()> {
    for art in fileset {
        let rel = artifact_rel(art);
        let abs = fsutil::safe_join(tree_root, rel)?;
        ensure_parent_dirs(tree_root, rel, created_dirs)?;
        match art {
            Artifact::File { body, .. } => {
                // The atomic create-new IS the clobber refusal (one syscall,
                // no TOCTOU). Track before the body write.
                let mut f = fsutil::create_new_file(&abs)
                    .with_context(|| format!("Failed to create {}", abs.display()))?;
                created_paths.push(abs.clone());
                f.write_all(body.as_bytes())
                    .with_context(|| format!("Failed to write {}", abs.display()))?;
            }
            Artifact::Symlink { target, .. } => {
                // symlink(2) is atomic; an existing target is a clobber → fail.
                std::os::unix::fs::symlink(target, &abs)
                    .with_context(|| format!("Failed to symlink {}", abs.display()))?;
                created_paths.push(abs.clone());
            }
        }
    }
    Ok(())
}

/// Create each missing component of `rel`'s parent under `tree_root`, pushing
/// only the ones *this call* creates onto `created_dirs`. `create_dir_all`
/// cannot report which components it made, so the walk is component-wise
/// `create_dir` (finding 2). An `AlreadyExists` that is a real dir is a
/// pre-existing/concurrent parent (skip, do not track); anything else (a file
/// or symlink squatting the path) is an error.
fn ensure_parent_dirs(
    tree_root: &Path,
    rel: &Path,
    created_dirs: &mut Vec<PathBuf>,
) -> anyhow::Result<()> {
    let Some(parent) = rel.parent() else {
        return Ok(());
    };
    let mut cur = tree_root.to_path_buf();
    for comp in parent.components() {
        cur.push(comp);
        match fs::create_dir(&cur) {
            Ok(()) => created_dirs.push(cur.clone()),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                if !fsutil::is_real_dir(&cur) {
                    bail!(
                        "Failed to create {}: a non-directory squats that path",
                        cur.display()
                    );
                }
            }
            Err(e) => {
                return Err(e).with_context(|| format!("Failed to create {}", cur.display()));
            }
        }
    }
    Ok(())
}

/// Undo a partial fileset write: unlink created files/symlinks, then remove the
/// dirs *this call* created, both in reverse order. Runs while unwinding a prior
/// error, so it cannot itself fail — every error is ignored (the original error
/// is the one surfaced). The guarantee that carries weight is structural, not in
/// any error match: `remove_dir` (never `remove_dir_all`) means a dir a
/// concurrent writer populated fails with `DirectoryNotEmpty` and is left intact
/// — we never force. Never touches the parent.
fn rollback(created_paths: &[PathBuf], created_dirs: &[PathBuf]) {
    for path in created_paths.iter().rev() {
        drop(fs::remove_file(path)); // unlinks a file or a symlink
    }
    for dir in created_dirs.iter().rev() {
        drop(fs::remove_dir(dir));
    }
}

fn artifact_rel(art: &Artifact) -> &Path {
    match art {
        Artifact::File { rel_path, .. } | Artifact::Symlink { rel_path, .. } => rel_path,
    }
}

// ---------------------------------------------------------------------------
// Tests (kind-blind — driven by a test `Kind`)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    // --- candidate_id ---

    #[test]
    fn candidate_id_empty_is_one() {
        assert_eq!(candidate_id(&[]), 1);
    }

    #[test]
    fn candidate_id_is_max_plus_one_ignoring_gaps() {
        assert_eq!(candidate_id(&[1, 2, 3]), 4);
        assert_eq!(candidate_id(&[1, 3]), 4);
        assert_eq!(candidate_id(&[5]), 6);
    }

    // --- next_id (trunk union) ---

    #[test]
    fn next_id_empty_union_is_one() {
        assert_eq!(next_id(&[], &[]), 1);
    }

    #[test]
    fn next_id_local_only_equals_candidate_id() {
        // INV-1: next_id(local, &[]) is byte-identical to candidate_id(local).
        for local in [&[][..], &[1, 2, 3], &[5], &[1, 3]] {
            assert_eq!(next_id(local, &[]), candidate_id(local));
        }
    }

    #[test]
    fn next_id_is_max_of_union_plus_one() {
        assert_eq!(next_id(&[1, 2], &[5, 3]), 6); // trunk ahead
        assert_eq!(next_id(&[7], &[2, 3]), 8); // local ahead
        assert_eq!(next_id(&[], &[4]), 5); // trunk only
        assert_eq!(next_id(&[4], &[4]), 5); // overlap, not double-counted
    }

    // --- derive_slug ---

    #[test]
    fn derive_slug_normalises_title() {
        assert_eq!(derive_slug("Add skill removal"), "add-skill-removal");
        assert_eq!(derive_slug("Hello, World!"), "hello-world");
        assert_eq!(derive_slug("  trim  edges  "), "trim-edges");
        assert_eq!(derive_slug("snake_and-dash"), "snake-and-dash");
    }

    // --- scan_ids ---

    #[test]
    fn scan_ids_collects_numeric_dirs_only() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir(root.join("001")).unwrap();
        fs::create_dir(root.join("002")).unwrap();
        fs::create_dir(root.join("not-a-slice")).unwrap();
        fs::write(root.join("003"), "a file, not a dir").unwrap();
        std::os::unix::fs::symlink("001", root.join("001-some-slug")).unwrap();

        let mut ids = scan_ids(root).unwrap();
        ids.sort_unstable();
        assert_eq!(ids, vec![1, 2]);
    }

    #[test]
    fn scan_ids_missing_dir_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert!(scan_ids(&dir.path().join("nope")).unwrap().is_empty());
    }

    // --- the acquire seam ---

    #[test]
    fn local_fs_claim_creates_the_dir_indifferent_to_id() {
        let dir = tempfile::tempdir().unwrap();
        let claim = dir.path().join("001");
        // The first claim wins and creates the dir; `id` rides the ctx but LocalFs
        // arbitrates on `dir` alone (VT-2).
        let won = ClaimCtx { dir: &claim, id: 1 };
        assert!(matches!(LocalFs.claim(&won).unwrap(), Acquired::Won));
        assert!(claim.is_dir(), "a won claim creates the dir");
        // A second claim on the same dir loses the race — a different `id` does not
        // change the verdict.
        let again = ClaimCtx {
            dir: &claim,
            id: 999,
        };
        assert!(matches!(
            LocalFs.claim(&again).unwrap(),
            Acquired::AlreadyHeld
        ));
    }

    // --- a test Kind drives the kind-blind engine paths ---

    fn one_file(ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
        let (id, canonical) = (ctx.id, ctx.canonical);
        let name = format!("{id:03}");
        Ok(vec![Artifact::File {
            rel_path: PathBuf::from(format!("{name}/body.md")),
            body: format!("{canonical} :: {}", ctx.title),
        }])
    }

    const TEST_KIND: Kind = Kind {
        dir: "tree",
        prefix: "TK",
        stem: "",
        scaffold: one_file,
    };

    fn inputs() -> Inputs<'static> {
        Inputs {
            slug: "s",
            title: "T",
            date: "2026-06-04",
        }
    }

    #[test]
    fn allocate_fresh_writes_then_lands_first_id() {
        let dir = tempfile::tempdir().unwrap();
        let tree = dir.path().join("tree");
        fs::create_dir_all(&tree).unwrap();

        let out = allocate_fresh(&TEST_KIND, &LocalFs, &tree, &inputs(), &[], || {
            scan_ids(&tree)
        })
        .unwrap();
        assert_eq!(out.eid.numeric_id(), Some(1));
        let body = fs::read_to_string(tree.join("001/body.md")).unwrap();
        assert_eq!(body, "TK-001 :: T");
    }

    #[test]
    fn allocate_fresh_retries_on_collision_through_the_seam() {
        let dir = tempfile::tempdir().unwrap();
        let tree = dir.path().join("tree");
        fs::create_dir_all(&tree).unwrap();
        // Pre-claim 001 on disk, then feed a stale (empty) listing first so the
        // candidate is 001 and the mkdir claim hits AlreadyHeld → recompute.
        fs::create_dir(tree.join("001")).unwrap();

        let calls = Cell::new(0u32);
        let scan = || {
            let n = calls.get();
            calls.set(n + 1);
            Ok(if n == 0 { vec![] } else { vec![1] })
        };

        let out = allocate_fresh(&TEST_KIND, &LocalFs, &tree, &inputs(), &[], scan).unwrap();
        assert_eq!(out.eid.numeric_id(), Some(2));
        assert!(tree.join("002/body.md").is_file());
        assert_eq!(calls.get(), 2, "expected one collision then success");
    }

    #[test]
    fn allocate_fresh_bails_after_bounded_retries() {
        // A backend that never yields a claim, with a listing that never grows,
        // exhausts the bounded loop rather than spinning forever.
        struct AlwaysHeld;
        impl Claim for AlwaysHeld {
            fn claim(&self, _ctx: &ClaimCtx<'_>) -> anyhow::Result<Acquired> {
                Ok(Acquired::AlreadyHeld)
            }
        }
        let dir = tempfile::tempdir().unwrap();
        let tree = dir.path().join("tree");
        fs::create_dir_all(&tree).unwrap();

        let err = allocate_fresh(&TEST_KIND, &AlwaysHeld, &tree, &inputs(), &[], || {
            Ok(vec![])
        })
        .unwrap_err();
        assert!(err.to_string().contains("Could not reserve an id"));
    }

    // --- H2: a write failure cleans up the won directory ---

    fn doomed_fileset(ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
        let id = ctx.id;
        let name = format!("{id:03}");
        // The second file's parent is the first file → the component-wise dir
        // walk hits a non-directory squatting `<name>/a` and fails.
        Ok(vec![
            Artifact::File {
                rel_path: PathBuf::from(format!("{name}/a")),
                body: "x".to_string(),
            },
            Artifact::File {
                rel_path: PathBuf::from(format!("{name}/a/b")),
                body: "y".to_string(),
            },
        ])
    }

    const DOOMED_KIND: Kind = Kind {
        dir: "tree",
        prefix: "TK",
        stem: "",
        scaffold: doomed_fileset,
    };

    #[test]
    fn reserved_materialise_write_failure_cleans_up_the_won_directory() {
        let dir = tempfile::tempdir().unwrap();
        let tree = dir.path().join("tree");
        fs::create_dir_all(&tree).unwrap();

        let err = allocate_fresh(&DOOMED_KIND, &LocalFs, &tree, &inputs(), &[], || {
            scan_ids(&tree)
        })
        .unwrap_err();
        assert!(err.to_string().contains("Failed to create"));
        assert!(!tree.join("001").exists(), "the won dir must be removed");
    }

    // --- H1 through materialise: a bad descriptor never writes ---

    fn escaping_fileset(_ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
        Ok(vec![Artifact::File {
            rel_path: PathBuf::from("../escape.md"),
            body: "x".to_string(),
        }])
    }

    const ESCAPING_KIND: Kind = Kind {
        dir: "tree",
        prefix: "TK",
        stem: "",
        scaffold: escaping_fileset,
    };

    #[test]
    fn reserved_materialise_rejects_an_escaping_descriptor_and_cleans_up() {
        let dir = tempfile::tempdir().unwrap();
        let tree = dir.path().join("tree");
        fs::create_dir_all(&tree).unwrap();

        let err = allocate_fresh(&ESCAPING_KIND, &LocalFs, &tree, &inputs(), &[], || {
            scan_ids(&tree)
        })
        .unwrap_err();
        assert!(err.to_string().contains("must not escape"));
        assert!(!tree.join("001").exists());
        assert!(!dir.path().join("escape.md").exists());
    }

    // --- CreateInExistingEntity ---

    const SUB_KIND: Kind = Kind {
        dir: "tree",
        prefix: "TK",
        stem: "",
        scaffold: one_file,
    };

    #[test]
    fn create_in_existing_writes_under_the_parent_without_reserving() {
        let dir = tempfile::tempdir().unwrap();
        let tree = dir.path().join("tree");
        fs::create_dir_all(tree.join("003")).unwrap();

        let out = create_in_existing(
            &SUB_KIND,
            &tree,
            3,
            &Inputs {
                slug: "",
                title: "Parent",
                date: "2026-06-04",
            },
        )
        .unwrap();
        assert_eq!(out.eid.numeric_id(), Some(3));
        let body = fs::read_to_string(tree.join("003/body.md")).unwrap();
        assert_eq!(body, "TK-003 :: Parent");
    }

    #[test]
    fn create_in_existing_errors_when_parent_absent() {
        let dir = tempfile::tempdir().unwrap();
        let tree = dir.path().join("tree");
        fs::create_dir_all(&tree).unwrap();

        let err = create_in_existing(
            &SUB_KIND,
            &tree,
            9,
            &Inputs {
                slug: "",
                title: "T",
                date: "2026-06-04",
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn create_in_existing_refuses_to_clobber() {
        let dir = tempfile::tempdir().unwrap();
        let tree = dir.path().join("tree");
        fs::create_dir_all(tree.join("003")).unwrap();
        fs::write(tree.join("003/body.md"), "already here").unwrap();

        let err = create_in_existing(
            &SUB_KIND,
            &tree,
            3,
            &Inputs {
                slug: "",
                title: "T",
                date: "2026-06-04",
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("Refusing to overwrite"));
        // untouched
        assert_eq!(
            fs::read_to_string(tree.join("003/body.md")).unwrap(),
            "already here"
        );
    }

    // --- D4: the multi-file sub-artefact writer is transactional ---

    /// The real IP shape: two files under an existing parent, both succeed.
    fn two_files(ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
        let id = ctx.id;
        let name = format!("{id:03}");
        Ok(vec![
            Artifact::File {
                rel_path: PathBuf::from(format!("{name}/plan.toml")),
                body: "p".to_string(),
            },
            Artifact::File {
                rel_path: PathBuf::from(format!("{name}/plan.md")),
                body: "m".to_string(),
            },
        ])
    }

    const SUB_TWO_KIND: Kind = Kind {
        dir: "tree",
        prefix: "TK",
        stem: "",
        scaffold: two_files,
    };

    #[test]
    fn create_in_existing_writes_a_multi_file_fileset() {
        let dir = tempfile::tempdir().unwrap();
        let tree = dir.path().join("tree");
        fs::create_dir_all(tree.join("003")).unwrap();

        create_in_existing(
            &SUB_TWO_KIND,
            &tree,
            3,
            &Inputs {
                slug: "",
                title: "T",
                date: "2026-06-04",
            },
        )
        .unwrap();
        assert_eq!(fs::read_to_string(tree.join("003/plan.toml")).unwrap(), "p");
        assert_eq!(fs::read_to_string(tree.join("003/plan.md")).unwrap(), "m");
    }

    /// A sub-artefact that creates a dir, a file, and a symlink, then aborts on
    /// its last file (a non-dir squats a path component) — exercising rollback
    /// of files, symlinks, and the dir this call created, while a pre-existing
    /// sibling is left untouched.
    fn sub_doomed_fileset(ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
        let id = ctx.id;
        let name = format!("{id:03}");
        Ok(vec![
            Artifact::File {
                rel_path: PathBuf::from(format!("{name}/sub/a")),
                body: "x".to_string(),
            },
            Artifact::Symlink {
                rel_path: PathBuf::from(format!("{name}/link")),
                target: "sub".to_string(),
            },
            Artifact::File {
                rel_path: PathBuf::from(format!("{name}/sub/a/b")),
                body: "y".to_string(),
            },
        ])
    }

    const SUB_DOOMED_KIND: Kind = Kind {
        dir: "tree",
        prefix: "TK",
        stem: "",
        scaffold: sub_doomed_fileset,
    };

    #[test]
    fn create_in_existing_rolls_back_partial_fileset_leaving_parent_intact() {
        let dir = tempfile::tempdir().unwrap();
        let tree = dir.path().join("tree");
        fs::create_dir_all(tree.join("003")).unwrap();
        // A pre-existing sibling the rollback must never touch.
        fs::write(tree.join("003/keep.txt"), "keep").unwrap();

        let err = create_in_existing(
            &SUB_DOOMED_KIND,
            &tree,
            3,
            &Inputs {
                slug: "",
                title: "T",
                date: "2026-06-04",
            },
        )
        .unwrap_err();
        assert!(err.to_string().contains("Failed to create"));

        // Everything this call created is gone …
        assert!(!tree.join("003/sub").exists(), "created dir unwound");
        assert!(!tree.join("003/link").exists(), "created symlink unwound");
        // … the pre-existing parent + sibling are byte-identical.
        assert!(tree.join("003").is_dir());
        assert_eq!(
            fs::read_to_string(tree.join("003/keep.txt")).unwrap(),
            "keep"
        );
        // and no other detritus survives
        let mut left: Vec<String> = fs::read_dir(tree.join("003"))
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        left.sort();
        assert_eq!(left, vec!["keep.txt".to_string()]);
    }

    /// The promised invariant (design §5.5/§9): a dir a concurrent writer
    /// populated mid-call is left intact — `remove_dir` hits `DirectoryNotEmpty`
    /// and tolerates it; we never `remove_dir_all`. Driven directly against
    /// `rollback`, since the deterministic scaffold can't race a foreign write in.
    #[test]
    fn rollback_leaves_a_dir_a_concurrent_writer_populated_intact() {
        let tmp = tempfile::tempdir().unwrap();
        let created = tmp.path().join("created");
        fs::create_dir(&created).unwrap();
        // a concurrent writer dropped a file in after we created the dir but
        // before rollback — tracked as a created dir, but now non-empty.
        fs::write(created.join("intruder"), "x").unwrap();

        rollback(&[], std::slice::from_ref(&created));

        assert!(created.is_dir(), "populated dir survives rollback");
        assert_eq!(fs::read_to_string(created.join("intruder")).unwrap(), "x");
    }

    // --- materialise_named (seam A — pre-built fileset, no Kind) ---
    //
    // Seam A is the sole named placement path; its tests below cover the shared
    // `claim_and_write_named` core (duplicate name, H2 won-dir cleanup, pre-existing
    // alias rollback) that the retired `allocate_named` entry point used to re-prove.

    /// A memory-shaped fileset relative to the items tree: `<uid>/memory.toml`,
    /// `<uid>/memory.md`, and — iff `key` — a `<key> -> <uid>` symlink sibling.
    fn named_fileset(uid: &str, key: Option<&str>) -> Fileset {
        let mut fs = vec![
            Artifact::File {
                rel_path: PathBuf::from(format!("{uid}/memory.toml")),
                body: "toml".to_string(),
            },
            Artifact::File {
                rel_path: PathBuf::from(format!("{uid}/memory.md")),
                body: "md".to_string(),
            },
        ];
        if let Some(k) = key {
            fs.push(Artifact::Symlink {
                rel_path: PathBuf::from(k),
                target: uid.to_string(),
            });
        }
        fs
    }

    #[test]
    fn materialise_named_writes_a_prebuilt_fileset_under_dir_name() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        let out =
            materialise_named(root, "tree", "mem_abc", &named_fileset("mem_abc", None)).unwrap();
        assert_eq!(
            out.eid,
            OwnedEntityId::Named {
                name: "mem_abc".to_string()
            }
        );
        assert_eq!(out.dir, root.join("tree/mem_abc"));
        assert_eq!(
            fs::read_to_string(root.join("tree/mem_abc/memory.toml")).unwrap(),
            "toml"
        );
        assert_eq!(
            fs::read_to_string(root.join("tree/mem_abc/memory.md")).unwrap(),
            "md"
        );
    }

    #[test]
    fn materialise_named_with_a_key_writes_the_alias_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        materialise_named(
            root,
            "tree",
            "mem_abc",
            &named_fileset("mem_abc", Some("mem.a.b")),
        )
        .unwrap();
        // the alias resolves to the uid dir
        let link = root.join("tree/mem.a.b");
        assert_eq!(fs::read_link(&link).unwrap(), Path::new("mem_abc"));
        assert!(
            scan_named(&root.join("tree"))
                .unwrap()
                .contains(&"mem_abc".to_string())
        );
        // the symlink is not a real dir → excluded from the scan (VT-2 at engine level)
        assert!(
            !scan_named(&root.join("tree"))
                .unwrap()
                .contains(&"mem.a.b".to_string())
        );
    }

    #[test]
    fn materialise_named_errs_on_a_duplicate_name() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("tree/mem_abc")).unwrap();

        let err = materialise_named(root, "tree", "mem_abc", &named_fileset("mem_abc", None))
            .unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn materialise_named_write_failure_cleans_up_the_won_dir() {
        // A self-squatting fileset: a file `<uid>/a`, then `<uid>/a/b` whose parent
        // is that file → the dir walk fails; H2 removes the won uid dir.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let doomed = vec![
            Artifact::File {
                rel_path: PathBuf::from("mem_abc/a"),
                body: "x".to_string(),
            },
            Artifact::File {
                rel_path: PathBuf::from("mem_abc/a/b"),
                body: "y".to_string(),
            },
        ];

        let err = materialise_named(root, "tree", "mem_abc", &doomed).unwrap_err();
        assert!(err.to_string().contains("Failed to create"));
        assert!(
            !root.join("tree/mem_abc").exists(),
            "the won dir must be removed"
        );
    }

    #[test]
    fn materialise_named_rolls_back_the_uid_dir_on_a_pre_existing_key_alias() {
        // VT-3 at the engine level: the `<key>` path already exists, so the alias
        // symlink in the fileset fails → the whole record rolls back, the uid dir
        // included. No partial record survives.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let tree = root.join("tree");
        fs::create_dir_all(&tree).unwrap();
        // a stale alias squats the key name
        fs::write(tree.join("mem.a.b"), "stale").unwrap();

        let err = materialise_named(
            root,
            "tree",
            "mem_abc",
            &named_fileset("mem_abc", Some("mem.a.b")),
        )
        .unwrap_err();
        assert!(err.to_string().contains("Failed to symlink"));
        assert!(
            !tree.join("mem_abc").exists(),
            "the uid dir must be rolled back — no partial record"
        );
        // the pre-existing alias is untouched
        assert_eq!(fs::read_to_string(tree.join("mem.a.b")).unwrap(), "stale");
    }

    // --- scan_named ---

    #[test]
    fn scan_named_collects_every_real_subdir_skipping_files_and_symlinks() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir(root.join("001")).unwrap(); // numeric name is fine
        fs::create_dir(root.join("mem_abc")).unwrap(); // and so is a named one
        fs::write(root.join("a-file"), "x").unwrap();
        std::os::unix::fs::symlink("001", root.join("a-link")).unwrap();

        let mut names = scan_named(root).unwrap();
        names.sort();
        assert_eq!(names, vec!["001".to_string(), "mem_abc".to_string()]);
    }

    #[test]
    fn scan_named_missing_dir_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert!(scan_named(&dir.path().join("nope")).unwrap().is_empty());
    }
}
