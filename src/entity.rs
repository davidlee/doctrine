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

/// The one impure-critical operation, behind a one-method trait so the future
/// `git-ref` backend drops in without a Kind-caller rewrite (reservation-spec).
pub(crate) trait Claim {
    /// Atomic, exclusive claim. `Won` if this caller created `claim`;
    /// `AlreadyHeld` if another agent won the race. Only this op arbitrates.
    fn claim(&self, claim: &Path) -> anyhow::Result<Acquired>;
}

/// The local-filesystem backend: the `mkdir` is the claim (D1 — the dir *is*
/// the claim). Lifted verbatim from the old `reserve_create`, so the slice-001
/// retry test stays green.
pub(crate) struct LocalFs;

impl Claim for LocalFs {
    fn claim(&self, claim: &Path) -> anyhow::Result<Acquired> {
        match fs::create_dir(claim) {
            Ok(()) => Ok(Acquired::Won),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(Acquired::AlreadyHeld),
            Err(e) => Err(e).with_context(|| format!("Failed to claim {}", claim.display())),
        }
    }
}

// ---------------------------------------------------------------------------
// The `Kind` descriptor
// ---------------------------------------------------------------------------

/// A `Kind` is *data*, not a trait: one dispatch site, no per-kind state (D2).
/// Placement is no longer a `Kind` field — it is a runtime `MaterialiseRequest`,
/// because a named entity carries its uid only at call time (D8).
pub(crate) struct Kind {
    /// Entity-tree root, relative to the project root, e.g. `.doctrine/slice`.
    /// Also the base every artifact path is joined to (H1).
    pub dir: &'static str,
    /// Canonical-id prefix, e.g. `SL` → `SL-003` (the `{{ref}}` token). Unused
    /// by named kinds, which have no numeric canonical id.
    pub prefix: &'static str,
    /// Fileset as a function — kind-supplied, never a frozen file count (D3).
    pub scaffold: fn(&ScaffoldCtx<'_>) -> anyhow::Result<Fileset>,
}

/// The rendered identity a scaffold sees: a numbered entity carries its id and
/// canonical `{{ref}}`; a named entity carries only its name (its uid). A closed
/// enum so a scaffold that assumes one shape fails loudly on the other (D1/D8).
#[derive(Clone, Copy)]
pub(crate) enum EntityId<'a> {
    Numbered { id: u32, canonical: &'a str },
    Named { name: &'a str },
}

/// The resolved context a `scaffold` renders from. Pure over these inputs plus
/// compile-time-embedded template text (M4): no disk, clock, git, or root.
pub(crate) struct ScaffoldCtx<'a> {
    pub eid: EntityId<'a>,
    pub slug: &'a str,
    pub title: &'a str,
    pub date: &'a str,
}

impl ScaffoldCtx<'_> {
    /// The numbered identity `(id, canonical)`, or an error for a named entity.
    /// The numeric scaffolds (slice/design/plan/notes) render `<id>` dirs and
    /// `<canonical>` refs and call this rather than re-match `eid` inline; the
    /// error (not a panic) keeps a future kind/placement mismatch loud (D8).
    pub(crate) fn numbered(&self) -> anyhow::Result<(u32, &str)> {
        match self.eid {
            EntityId::Numbered { id, canonical } => Ok((id, canonical)),
            EntityId::Named { name } => bail!("numbered scaffold over named entity {name}"),
        }
    }
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

/// Where to place an entity, resolved at call time — `Kind` is const, but a
/// named entity's uid is only known per-call, so placement cannot live on the
/// `Kind` (D8). A closed enum: a fourth placement is a compiler-forced variant.
pub(crate) enum MaterialiseRequest<'a> {
    /// Allocate a fresh reserved numeric id (slice, later spec).
    Fresh,
    /// Create file(s) under an existing numeric parent (design / plan / notes).
    InExisting { id: u32 },
    /// Claim a caller-named entity dir (memory; the name is its uid). No binary
    /// caller constructs this until `doctrine memory record` (SL-005 PHASE-04);
    /// the engine path and its tests already exercise it. The attribute comes off
    /// when `run_record` builds the request.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "constructed by `memory record` (SL-005 PHASE-04)")
    )]
    Named { name: &'a str },
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

    /// The canonical `{{ref}}` (numbered) or the name (named) — the on-disk dir
    /// name and the cross-entity reference token. Every entity has one.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "read by the memory verbs (SL-005 PHASE-04/05)")
    )]
    pub(crate) fn canonical_ref(&self) -> &str {
        match self {
            OwnedEntityId::Numbered { canonical, .. } => canonical,
            OwnedEntityId::Named { name } => name,
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
// Pure helpers: id, slug
// ---------------------------------------------------------------------------

/// Next id from a directory listing: `max + 1`, or `1` when empty. Gaps are
/// not back-filled — the id is monotonic (slices-spec § Id allocation).
pub(crate) fn candidate_id(existing: &[u32]) -> u32 {
    existing.iter().copied().max().map_or(1, |m| m + 1)
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
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "drives `memory list` (SL-005 PHASE-05)")
)]
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
/// `request`: allocate a fresh reserved id, create files under an existing
/// numeric parent, or claim a caller-named dir. Returns the owned identity and
/// entity dir.
pub(crate) fn materialise(
    kind: &Kind,
    claim: &dyn Claim,
    project_root: &Path,
    request: &MaterialiseRequest<'_>,
    inputs: &Inputs<'_>,
) -> anyhow::Result<Materialised> {
    let tree_root = project_root.join(kind.dir);
    // The entity-tree root; the non-recursive claim mkdir below needs it to
    // exist (the first-ever-entity case).
    fs::create_dir_all(&tree_root)
        .with_context(|| format!("Failed to create {}", tree_root.display()))?;

    match *request {
        MaterialiseRequest::Fresh => {
            allocate_fresh(kind, claim, &tree_root, inputs, || scan_ids(&tree_root))
        }
        MaterialiseRequest::InExisting { id } => create_in_existing(kind, &tree_root, id, inputs),
        MaterialiseRequest::Named { name } => allocate_named(kind, claim, &tree_root, name, inputs),
    }
}

/// Reserved top-level placement (slice, later spec): claim the next id with a
/// bounded retry loop, then scaffold. A `Won` claim means the dir is ours, so
/// any scaffold/write failure removes it — no ghost entity survives (H2).
fn allocate_fresh(
    kind: &Kind,
    claim: &dyn Claim,
    tree_root: &Path,
    inputs: &Inputs<'_>,
    mut scan: impl FnMut() -> anyhow::Result<Vec<u32>>,
) -> anyhow::Result<Materialised> {
    for _ in 0..MAX_CLAIM_RETRIES {
        let id = candidate_id(&scan()?);
        let name = format!("{id:03}");
        let dir = tree_root.join(&name);
        match claim.claim(&dir)? {
            Acquired::Won => {
                let canonical = format!("{}-{name}", kind.prefix);
                let ctx = ScaffoldCtx {
                    eid: EntityId::Numbered {
                        id,
                        canonical: &canonical,
                    },
                    slug: inputs.slug,
                    title: inputs.title,
                    date: inputs.date,
                };
                return match scaffold_and_write(kind, tree_root, &ctx) {
                    Ok(()) => Ok(Materialised {
                        eid: OwnedEntityId::Numbered { id, canonical },
                        dir,
                    }),
                    Err(e) => {
                        // Won ⟹ we created `dir` ⟹ a partial scaffold is our
                        // mess to clean (H2). git-ref will not need this — there
                        // an abandoned claim is a harmless gap (reservation-spec).
                        // Best-effort: the scaffold error is the one to surface.
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
        eid: EntityId::Numbered {
            id,
            canonical: &canonical,
        },
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

/// Named top-level placement (memory entity): the caller supplies the entity
/// name (its uid), already unique, so there is no id scan and no retry — claim
/// `tree_root/<name>` once. `Won` means the dir is ours, so a scaffold/write
/// failure removes it (H2, as in `allocate_fresh`); `AlreadyHeld` is a duplicate
/// uid and a hard error.
fn allocate_named(
    kind: &Kind,
    claim: &dyn Claim,
    tree_root: &Path,
    name: &str,
    inputs: &Inputs<'_>,
) -> anyhow::Result<Materialised> {
    let dir = tree_root.join(name);
    match claim.claim(&dir)? {
        Acquired::Won => {
            let ctx = ScaffoldCtx {
                eid: EntityId::Named { name },
                slug: inputs.slug,
                title: inputs.title,
                date: inputs.date,
            };
            match scaffold_and_write(kind, tree_root, &ctx) {
                Ok(()) => Ok(Materialised {
                    eid: OwnedEntityId::Named {
                        name: name.to_string(),
                    },
                    dir,
                }),
                // Won ⟹ we created `dir` ⟹ a partial scaffold is ours to clean.
                Err(e) => {
                    drop(fs::remove_dir_all(&dir));
                    Err(e)
                }
            }
        }
        Acquired::AlreadyHeld => bail!("entity {name} already exists"),
    }
}

/// Render `kind`'s fileset for `ctx` and write it under `tree_root`.
fn scaffold_and_write(kind: &Kind, tree_root: &Path, ctx: &ScaffoldCtx<'_>) -> anyhow::Result<()> {
    let fileset = (kind.scaffold)(ctx)?;
    write_fileset(tree_root, &fileset)
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
    fn local_fs_acquire_wins_then_already_held() {
        let dir = tempfile::tempdir().unwrap();
        let claim = dir.path().join("001");
        assert!(matches!(LocalFs.claim(&claim).unwrap(), Acquired::Won));
        assert!(matches!(
            LocalFs.claim(&claim).unwrap(),
            Acquired::AlreadyHeld
        ));
    }

    // --- a test Kind drives the kind-blind engine paths ---

    fn one_file(ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
        let (id, canonical) = ctx.numbered()?;
        let name = format!("{id:03}");
        Ok(vec![Artifact::File {
            rel_path: PathBuf::from(format!("{name}/body.md")),
            body: format!("{canonical} :: {}", ctx.title),
        }])
    }

    const TEST_KIND: Kind = Kind {
        dir: "tree",
        prefix: "TK",
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

        let out =
            allocate_fresh(&TEST_KIND, &LocalFs, &tree, &inputs(), || scan_ids(&tree)).unwrap();
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

        let out = allocate_fresh(&TEST_KIND, &LocalFs, &tree, &inputs(), scan).unwrap();
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
            fn claim(&self, _claim: &Path) -> anyhow::Result<Acquired> {
                Ok(Acquired::AlreadyHeld)
            }
        }
        let dir = tempfile::tempdir().unwrap();
        let tree = dir.path().join("tree");
        fs::create_dir_all(&tree).unwrap();

        let err =
            allocate_fresh(&TEST_KIND, &AlwaysHeld, &tree, &inputs(), || Ok(vec![])).unwrap_err();
        assert!(err.to_string().contains("Could not reserve an id"));
    }

    // --- H2: a write failure cleans up the won directory ---

    fn doomed_fileset(ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
        let (id, _) = ctx.numbered()?;
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
        scaffold: doomed_fileset,
    };

    #[test]
    fn reserved_materialise_write_failure_cleans_up_the_won_directory() {
        let dir = tempfile::tempdir().unwrap();
        let tree = dir.path().join("tree");
        fs::create_dir_all(&tree).unwrap();

        let err = allocate_fresh(&DOOMED_KIND, &LocalFs, &tree, &inputs(), || scan_ids(&tree))
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
        scaffold: escaping_fileset,
    };

    #[test]
    fn reserved_materialise_rejects_an_escaping_descriptor_and_cleans_up() {
        let dir = tempfile::tempdir().unwrap();
        let tree = dir.path().join("tree");
        fs::create_dir_all(&tree).unwrap();

        let err = allocate_fresh(&ESCAPING_KIND, &LocalFs, &tree, &inputs(), || {
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
        let (id, _) = ctx.numbered()?;
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
        let (id, _) = ctx.numbered()?;
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

    // --- Named placement (allocate_named) ---

    /// A named-entity scaffold: one file under the entity's own (named) dir.
    fn named_file(ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
        let EntityId::Named { name } = ctx.eid else {
            bail!("named scaffold over a numbered entity");
        };
        Ok(vec![Artifact::File {
            rel_path: PathBuf::from(format!("{name}/body.md")),
            body: format!("{name} :: {}", ctx.title),
        }])
    }

    const NAMED_KIND: Kind = Kind {
        dir: "tree",
        prefix: "",
        scaffold: named_file,
    };

    #[test]
    fn named_materialise_claims_and_writes_under_the_name() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        // Driven through the public `materialise` so the `Named` request arm and
        // its dispatch are exercised, not just `allocate_named` in isolation.
        let out = materialise(
            &NAMED_KIND,
            &LocalFs,
            root,
            &MaterialiseRequest::Named { name: "mem_abc" },
            &inputs(),
        )
        .unwrap();
        assert_eq!(
            out.eid,
            OwnedEntityId::Named {
                name: "mem_abc".to_string()
            }
        );
        // a named entity has no numeric id; its canonical ref is the name itself
        assert_eq!(out.eid.numeric_id(), None);
        assert_eq!(out.eid.canonical_ref(), "mem_abc");
        let body = fs::read_to_string(root.join("tree/mem_abc/body.md")).unwrap();
        assert_eq!(body, "mem_abc :: T");
    }

    #[test]
    fn allocate_named_errs_on_a_duplicate_name() {
        let dir = tempfile::tempdir().unwrap();
        let tree = dir.path().join("tree");
        // The name is pre-claimed on disk → the single mkdir claim loses.
        fs::create_dir_all(tree.join("mem_abc")).unwrap();

        let err = allocate_named(&NAMED_KIND, &LocalFs, &tree, "mem_abc", &inputs()).unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn allocate_named_write_failure_cleans_up_the_claimed_dir() {
        // A doomed named scaffold (the self-squat trick): the claim mkdir creates
        // the named dir, then the scaffold fails — H2 must remove the won dir.
        fn doomed(ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
            let EntityId::Named { name } = ctx.eid else {
                bail!("named scaffold over a numbered entity");
            };
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
        const DOOMED_NAMED_KIND: Kind = Kind {
            dir: "tree",
            prefix: "",
            scaffold: doomed,
        };

        let dir = tempfile::tempdir().unwrap();
        let tree = dir.path().join("tree");
        fs::create_dir_all(&tree).unwrap();

        let err =
            allocate_named(&DOOMED_NAMED_KIND, &LocalFs, &tree, "mem_abc", &inputs()).unwrap_err();
        assert!(err.to_string().contains("Failed to create"));
        assert!(
            !tree.join("mem_abc").exists(),
            "the claimed dir must be removed"
        );
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
