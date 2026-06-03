// SPDX-License-Identifier: GPL-3.0-only
//! Kind-agnostic directory-entity scaffolding engine.
//!
//! One engine materialises every directory entity (slice, design-doc sibling,
//! later drift/spec) from a `Kind` descriptor. The engine is kind-blind: the
//! claim is behind the `acquire` seam (reservation-spec § Code seam), the
//! fileset is a `Kind`-supplied function (not a frozen pair — slice-002 M3),
//! and placement is a closed `MaterialiseMode` enum (never a `reserve: bool`).
//!
//! Pure/imperative split (slices-spec § Architecture): id, slug and the fileset
//! are decided from inputs; only `acquire` and the writes touch disk, and the
//! writer is the *sole* joiner of descriptor paths to the filesystem (H1).

use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, bail};

/// Bounded retries for the reservation claim loop.
const MAX_CLAIM_RETRIES: u32 = 128;

// ---------------------------------------------------------------------------
// The `acquire` seam
// ---------------------------------------------------------------------------

/// Outcome of an atomic claim: this caller created it, or another agent already
/// holds it.
pub(crate) enum Acquired {
    Won,
    AlreadyHeld,
}

/// The one impure-critical operation, behind a one-method trait so the future
/// `git-ref` backend drops in without a Kind-caller rewrite (reservation-spec).
pub(crate) trait Reservation {
    /// Atomic, exclusive claim. `Won` if this caller created `claim`;
    /// `AlreadyHeld` if another agent won the race. Only this op arbitrates.
    fn acquire(&self, claim: &Path) -> anyhow::Result<Acquired>;
}

/// The local-filesystem backend: the `mkdir` is the claim (D1 — the dir *is*
/// the claim). Lifted verbatim from the old `reserve_create`, so the slice-001
/// retry test stays green.
pub(crate) struct LocalFs;

impl Reservation for LocalFs {
    fn acquire(&self, claim: &Path) -> anyhow::Result<Acquired> {
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

/// How an entity is placed — a closed enum, not a `bool`, so a third placement
/// is a compiler-forced new variant, never an overloaded `false` (M1/D4).
pub(crate) enum MaterialiseMode {
    /// Allocate a fresh reserved id under `dir` (slice, later spec).
    AllocateFreshEntity,
    /// Create file(s) in an existing parent entity (design doc, later phases).
    CreateInExistingEntity,
}

/// A `Kind` is *data*, not a trait: one dispatch site, no per-kind state (D2).
pub(crate) struct Kind {
    /// Entity-tree root, relative to the project root, e.g. `.doctrine/slice`.
    /// Also the base every artifact path is joined to (H1).
    pub dir: &'static str,
    /// Canonical-id prefix, e.g. `SL` → `SL-003` (the `{{ref}}` token).
    pub prefix: &'static str,
    /// How the entity is placed.
    pub mode: MaterialiseMode,
    /// Fileset as a function — kind-supplied, never a frozen file count (D3).
    pub scaffold: fn(&ScaffoldCtx<'_>) -> anyhow::Result<Fileset>,
}

/// The resolved context a `scaffold` renders from. Pure over these inputs plus
/// compile-time-embedded template text (M4): no disk, clock, git, or root.
pub(crate) struct ScaffoldCtx<'a> {
    pub id: u32,
    /// The `{{ref}}` token, e.g. `SL-003`.
    pub canonical_id: &'a str,
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

/// Caller-supplied scaffold inputs. The engine fills id / canonical-id / dir.
pub(crate) struct Inputs<'a> {
    /// Parent entity id for `CreateInExistingEntity`; ignored when allocating.
    pub existing_id: Option<u32>,
    pub slug: &'a str,
    pub title: &'a str,
    pub date: &'a str,
}

/// What a successful materialisation yields: the allocated/parent id and the
/// entity dir on disk.
#[derive(Debug)]
pub(crate) struct Materialised {
    pub id: u32,
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

// ---------------------------------------------------------------------------
// The materialise loop
// ---------------------------------------------------------------------------

/// Materialise `kind` under `project_root`. Dispatches on placement: allocate a
/// fresh reserved id, or create files in an existing parent. Returns the id and
/// entity dir.
pub(crate) fn materialise(
    kind: &Kind,
    reservation: &dyn Reservation,
    project_root: &Path,
    inputs: &Inputs<'_>,
) -> anyhow::Result<Materialised> {
    let tree_root = project_root.join(kind.dir);
    // The entity-tree root; the non-recursive claim mkdir below needs it to
    // exist (the first-ever-entity case).
    fs::create_dir_all(&tree_root)
        .with_context(|| format!("Failed to create {}", tree_root.display()))?;

    match kind.mode {
        MaterialiseMode::AllocateFreshEntity => {
            allocate_fresh(kind, reservation, &tree_root, inputs, || {
                scan_ids(&tree_root)
            })
        }
        MaterialiseMode::CreateInExistingEntity => create_in_existing(kind, &tree_root, inputs),
    }
}

/// Reserved top-level placement (slice, later spec): claim the next id with a
/// bounded retry loop, then scaffold. A `Won` claim means the dir is ours, so
/// any scaffold/write failure removes it — no ghost entity survives (H2).
fn allocate_fresh(
    kind: &Kind,
    reservation: &dyn Reservation,
    tree_root: &Path,
    inputs: &Inputs<'_>,
    mut scan: impl FnMut() -> anyhow::Result<Vec<u32>>,
) -> anyhow::Result<Materialised> {
    for _ in 0..MAX_CLAIM_RETRIES {
        let id = candidate_id(&scan()?);
        let name = format!("{id:03}");
        let dir = tree_root.join(&name);
        match reservation.acquire(&dir)? {
            Acquired::Won => {
                let canonical_id = format!("{}-{name}", kind.prefix);
                let ctx = ScaffoldCtx {
                    id,
                    canonical_id: &canonical_id,
                    slug: inputs.slug,
                    title: inputs.title,
                    date: inputs.date,
                };
                return match scaffold_and_write(kind, tree_root, &ctx) {
                    Ok(()) => Ok(Materialised { id, dir }),
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
/// Resolve the existing parent (err if absent), refuse to clobber, then write.
fn create_in_existing(
    kind: &Kind,
    tree_root: &Path,
    inputs: &Inputs<'_>,
) -> anyhow::Result<Materialised> {
    let id = inputs
        .existing_id
        .context("CreateInExistingEntity requires a parent id")?;
    let name = format!("{id:03}");
    let dir = tree_root.join(&name);
    if !dir.is_dir() {
        bail!("Parent entity {name} not found at {}", dir.display());
    }
    let canonical_id = format!("{}-{name}", kind.prefix);
    let ctx = ScaffoldCtx {
        id,
        canonical_id: &canonical_id,
        slug: inputs.slug,
        title: inputs.title,
        date: inputs.date,
    };
    let fileset = (kind.scaffold)(&ctx)?;
    refuse_clobber(tree_root, &fileset)?; // no silent clobber (D7)
    write_fileset(tree_root, &fileset)?;
    Ok(Materialised { id, dir })
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
        let abs = safe_join(tree_root, artifact_rel(art))?;
        if abs.exists() {
            bail!("Refusing to overwrite existing {}", abs.display());
        }
    }
    Ok(())
}

/// Write every artifact, joining each path under `tree_root` (the sole joiner).
fn write_fileset(tree_root: &Path, fileset: &Fileset) -> anyhow::Result<()> {
    for art in fileset {
        let abs = safe_join(tree_root, artifact_rel(art))?;
        match art {
            Artifact::File { body, .. } => {
                // create_dir_all so a future nested fileset needs no engine
                // change (the slice's numeric dir already exists).
                if let Some(parent) = abs.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create {}", parent.display()))?;
                }
                fs::write(&abs, body)
                    .with_context(|| format!("Failed to write {}", abs.display()))?;
            }
            Artifact::Symlink { target, .. } => {
                if let Err(e) = std::os::unix::fs::symlink(target, &abs)
                    && e.kind() != ErrorKind::AlreadyExists
                {
                    return Err(e).with_context(|| format!("Failed to symlink {}", abs.display()));
                }
            }
        }
    }
    Ok(())
}

fn artifact_rel(art: &Artifact) -> &Path {
    match art {
        Artifact::File { rel_path, .. } | Artifact::Symlink { rel_path, .. } => rel_path,
    }
}

/// Join a descriptor `rel` path under the entity-tree root, rejecting absolute
/// paths and any `..` that would escape the tree (H1). The single chokepoint
/// through which a Kind reaches the filesystem.
fn safe_join(tree_root: &Path, rel: &Path) -> anyhow::Result<PathBuf> {
    if rel.is_absolute() {
        bail!(
            "Artifact path {} must be relative to the entity tree",
            rel.display()
        );
    }
    if rel.components().any(|c| c == Component::ParentDir) {
        bail!(
            "Artifact path {} must not escape the entity tree",
            rel.display()
        );
    }
    Ok(tree_root.join(rel))
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

    // --- safe_join (H1 path containment) ---

    #[test]
    fn safe_join_accepts_a_tree_relative_path() {
        let joined = safe_join(Path::new("/tree"), Path::new("003/x.toml")).unwrap();
        assert_eq!(joined, Path::new("/tree/003/x.toml"));
    }

    #[test]
    fn safe_join_rejects_absolute_paths() {
        let err = safe_join(Path::new("/tree"), Path::new("/etc/passwd")).unwrap_err();
        assert!(err.to_string().contains("must be relative"));
    }

    #[test]
    fn safe_join_rejects_parent_escape() {
        let err = safe_join(Path::new("/tree"), Path::new("../../etc/passwd")).unwrap_err();
        assert!(err.to_string().contains("must not escape"));
    }

    // --- the acquire seam ---

    #[test]
    fn local_fs_acquire_wins_then_already_held() {
        let dir = tempfile::tempdir().unwrap();
        let claim = dir.path().join("001");
        assert!(matches!(LocalFs.acquire(&claim).unwrap(), Acquired::Won));
        assert!(matches!(
            LocalFs.acquire(&claim).unwrap(),
            Acquired::AlreadyHeld
        ));
    }

    // --- a test Kind drives the kind-blind engine paths ---

    fn one_file(ctx: &ScaffoldCtx<'_>) -> anyhow::Result<Fileset> {
        let name = format!("{:03}", ctx.id);
        Ok(vec![Artifact::File {
            rel_path: PathBuf::from(format!("{name}/body.md")),
            body: format!("{} :: {}", ctx.canonical_id, ctx.title),
        }])
    }

    const TEST_KIND: Kind = Kind {
        dir: "tree",
        prefix: "TK",
        mode: MaterialiseMode::AllocateFreshEntity,
        scaffold: one_file,
    };

    fn inputs() -> Inputs<'static> {
        Inputs {
            existing_id: None,
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
        assert_eq!(out.id, 1);
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
        assert_eq!(out.id, 2);
        assert!(tree.join("002/body.md").is_file());
        assert_eq!(calls.get(), 2, "expected one collision then success");
    }

    #[test]
    fn allocate_fresh_bails_after_bounded_retries() {
        // A backend that never yields a claim, with a listing that never grows,
        // exhausts the bounded loop rather than spinning forever.
        struct AlwaysHeld;
        impl Reservation for AlwaysHeld {
            fn acquire(&self, _claim: &Path) -> anyhow::Result<Acquired> {
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
        let name = format!("{:03}", ctx.id);
        // The second file's parent is the first file → create_dir_all fails.
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
        mode: MaterialiseMode::AllocateFreshEntity,
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
        mode: MaterialiseMode::AllocateFreshEntity,
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
        mode: MaterialiseMode::CreateInExistingEntity,
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
            &Inputs {
                existing_id: Some(3),
                slug: "",
                title: "Parent",
                date: "2026-06-04",
            },
        )
        .unwrap();
        assert_eq!(out.id, 3);
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
            &Inputs {
                existing_id: Some(9),
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
            &Inputs {
                existing_id: Some(3),
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
}
