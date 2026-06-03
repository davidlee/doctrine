//! `heresy slice` — create and list slices, Heresiarch's unit of change.
//!
//! A slice is a numeric directory under `.doctrine/slice/` holding a sister
//! TOML (structured metadata) and a scaffolded markdown prose body, with a
//! `<id>-<slug>` symlink as a human alias (slices-spec).
//!
//! Same split as `install` / `skills`: pure functions decide everything from
//! data — candidate id, slug derivation, template render, list formatting —
//! and a thin IO shell performs the one impure-critical act, the atomic
//! `mkdir` claim that arbitrates the id race (reservation-spec § local backend).

use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use serde::Deserialize;

/// Bounded retries for the reservation claim loop.
const MAX_CLAIM_RETRIES: u32 = 128;

/// Relative dir of the slice tree inside the project root.
const SLICE_DIR: &str = ".doctrine/slice";

// ---------------------------------------------------------------------------
// Model
// ---------------------------------------------------------------------------

/// The fields a reader extracts from `slice-<id>.toml`. Unknown keys (the
/// `[relationships]` table, future sections) are ignored and preserved on disk.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct Meta {
    id: u32,
    slug: String,
    title: String,
    status: String,
}

/// A fully-resolved scaffold for one new slice: every path and byte payload,
/// decided purely from inputs so it is asserted without disk or clock.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Scaffold {
    id: u32,
    dir: PathBuf,
    toml_path: PathBuf,
    toml_body: String,
    md_path: PathBuf,
    md_body: String,
    symlink: PathBuf,
    symlink_target: String,
}

// ---------------------------------------------------------------------------
// Pure: id, slug, render, list
// ---------------------------------------------------------------------------

/// Next id from a directory listing: `max + 1`, or `1` when empty. Gaps are
/// not back-filled — the id is monotonic (slices-spec § Id allocation).
fn candidate_id(existing: &[u32]) -> u32 {
    existing.iter().copied().max().map_or(1, |m| m + 1)
}

/// Derive a slug from a title: lowercase, runs of whitespace/`-`/`_` collapse
/// to a single `-`, every other non-alphanumeric is stripped, no edge dashes.
fn derive_slug(title: &str) -> String {
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

/// Render `slice-<id>.toml` from the embedded template by token substitution.
fn render_toml(id: u32, slug: &str, title: &str, date: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/slice.toml")?
        .replace("{{id}}", &id.to_string())
        .replace("{{slug}}", slug)
        .replace("{{title}}", title)
        .replace("{{date}}", date))
}

/// Render `slice-<id>.md` from the embedded template by token substitution.
fn render_md(title: &str) -> anyhow::Result<String> {
    Ok(crate::install::asset_text("templates/slice.md")?.replace("{{title}}", title))
}

/// Build the scaffold for `id` under `slice_root`, purely from inputs + date.
fn build_scaffold(
    slice_root: &Path,
    id: u32,
    slug: &str,
    title: &str,
    date: &str,
) -> anyhow::Result<Scaffold> {
    let name = format!("{id:03}");
    let dir = slice_root.join(&name);
    Ok(Scaffold {
        id,
        toml_path: dir.join(format!("slice-{name}.toml")),
        toml_body: render_toml(id, slug, title, date)?,
        md_path: dir.join(format!("slice-{name}.md")),
        md_body: render_md(title)?,
        symlink: slice_root.join(format!("{name}-{slug}")),
        symlink_target: name,
        dir,
    })
}

/// Sort by id and, when a status is given, keep only matching rows.
fn sort_and_filter(mut rows: Vec<Meta>, status: Option<&str>) -> Vec<Meta> {
    rows.retain(|m| status.is_none_or(|s| m.status == s));
    rows.sort_by_key(|m| m.id);
    rows
}

/// Format slice rows as aligned `id  status  slug  title` lines.
fn format_list(rows: &[Meta]) -> String {
    let status_w = rows.iter().map(|m| m.status.len()).max().unwrap_or(0);
    let slug_w = rows.iter().map(|m| m.slug.len()).max().unwrap_or(0);
    let lines: Vec<String> = rows
        .iter()
        .map(|m| {
            format!(
                "{:03}  {:<status_w$}  {:<slug_w$}  {}",
                m.id, m.status, m.slug, m.title
            )
        })
        .collect();
    if lines.is_empty() {
        String::new()
    } else {
        lines.join("\n") + "\n"
    }
}

// ---------------------------------------------------------------------------
// Imperative: scan, the atomic claim, write
// ---------------------------------------------------------------------------

/// Today as `YYYY-MM-DD` (UTC). The clock lives only in the shell; the pure
/// layer takes the date as a parameter (slices-spec § Architecture).
fn today() -> String {
    let d = time::OffsetDateTime::now_utc().date();
    format!("{:04}-{:02}-{:02}", d.year(), u8::from(d.month()), d.day())
}

/// Numeric slice ids present under `slice_root` (symlinks and files ignored).
/// A missing directory yields an empty listing.
fn scan_ids(slice_root: &Path) -> anyhow::Result<Vec<u32>> {
    let mut ids = Vec::new();
    let entries = match fs::read_dir(slice_root) {
        Ok(entries) => entries,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(ids),
        Err(e) => {
            return Err(e).with_context(|| format!("Failed to read {}", slice_root.display()));
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

/// Reserve the next id and materialise the slice. `scan` supplies the current
/// listing; the atomic `mkdir` is the claim — on `AlreadyExists` (another agent
/// won the race) recompute and retry, bounded (reservation-spec § unification).
fn reserve_create(
    slice_root: &Path,
    slug: &str,
    title: &str,
    date: &str,
    mut scan: impl FnMut() -> anyhow::Result<Vec<u32>>,
) -> anyhow::Result<Scaffold> {
    fs::create_dir_all(slice_root)
        .with_context(|| format!("Failed to create {}", slice_root.display()))?;

    for _ in 0..MAX_CLAIM_RETRIES {
        let id = candidate_id(&scan()?);
        let scaffold = build_scaffold(slice_root, id, slug, title, date)?;
        match fs::create_dir(&scaffold.dir) {
            Ok(()) => {
                write_scaffold(&scaffold)?;
                return Ok(scaffold);
            }
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {} // lost the race; retry
            Err(e) => {
                return Err(e)
                    .with_context(|| format!("Failed to claim {}", scaffold.dir.display()));
            }
        }
    }
    bail!("Could not reserve a slice id after {MAX_CLAIM_RETRIES} attempts");
}

/// Write the sister TOML, prose body, and slug symlink for a claimed slice.
fn write_scaffold(s: &Scaffold) -> anyhow::Result<()> {
    fs::write(&s.toml_path, &s.toml_body)
        .with_context(|| format!("Failed to write {}", s.toml_path.display()))?;
    fs::write(&s.md_path, &s.md_body)
        .with_context(|| format!("Failed to write {}", s.md_path.display()))?;
    if let Err(e) = std::os::unix::fs::symlink(&s.symlink_target, &s.symlink)
        && e.kind() != ErrorKind::AlreadyExists
    {
        return Err(e).with_context(|| format!("Failed to symlink {}", s.symlink.display()));
    }
    Ok(())
}

/// Read and parse every `slice-<id>.toml` under `slice_root`.
fn read_metas(slice_root: &Path) -> anyhow::Result<Vec<Meta>> {
    let mut metas = Vec::new();
    for id in scan_ids(slice_root)? {
        let name = format!("{id:03}");
        let path = slice_root.join(&name).join(format!("slice-{name}.toml"));
        let text = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let meta: Meta =
            toml::from_str(&text).with_context(|| format!("Failed to parse {}", path.display()))?;
        metas.push(meta);
    }
    Ok(metas)
}

// ---------------------------------------------------------------------------
// CLI entry points (thin)
// ---------------------------------------------------------------------------

/// Resolve the title: use the argument, else prompt on stdin. Must be non-empty.
fn resolve_title(title: Option<String>) -> anyhow::Result<String> {
    if let Some(t) = title {
        let t = t.trim().to_string();
        if t.is_empty() {
            bail!("Title must not be empty");
        }
        return Ok(t);
    }
    let mut stdout = io::stdout();
    write!(stdout, "Title: ")?;
    stdout.flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let entered = line.trim().to_string();
    if entered.is_empty() {
        bail!("Title must not be empty");
    }
    Ok(entered)
}

/// `heresy slice new`.
pub(crate) fn run_new(
    path: Option<PathBuf>,
    title: Option<String>,
    slug: Option<String>,
) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let slice_root = root.join(SLICE_DIR);
    let title = resolve_title(title)?;
    let slug = match slug {
        Some(s) => s,
        None => derive_slug(&title),
    };
    if slug.is_empty() {
        bail!("Could not derive a slug from the title; pass --slug");
    }
    let date = today();
    let scaffold = reserve_create(&slice_root, &slug, &title, &date, || scan_ids(&slice_root))?;

    let mut out = io::stdout();
    writeln!(
        out,
        "Created slice {:03}: {}",
        scaffold.id,
        scaffold.dir.display()
    )?;
    Ok(())
}

/// `heresy slice list`.
pub(crate) fn run_list(path: Option<PathBuf>, status: Option<&str>) -> anyhow::Result<()> {
    let root = crate::root::find(path, &crate::root::default_markers())?;
    let slice_root = root.join(SLICE_DIR);
    let rows = sort_and_filter(read_metas(&slice_root)?, status);

    let mut out = io::stdout();
    write!(out, "{}", format_list(&rows))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn meta(id: u32, status: &str, slug: &str, title: &str) -> Meta {
        Meta {
            id,
            slug: slug.to_string(),
            title: title.to_string(),
            status: status.to_string(),
        }
    }

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

    // --- render / round-trip ---

    #[test]
    fn render_toml_round_trips_to_metadata() {
        let body = render_toml(7, "my-slug", "My Title", "2026-06-03").unwrap();
        let parsed: Meta = toml::from_str(&body).unwrap();
        assert_eq!(parsed, meta(7, "proposed", "my-slug", "My Title"));
        // injected date survives
        assert!(body.contains("created = \"2026-06-03\""));
    }

    #[test]
    fn render_md_substitutes_title() {
        let body = render_md("My Title").unwrap();
        assert!(body.contains("My Title"));
        assert!(!body.contains("{{title}}"));
    }

    // --- build_scaffold ---

    #[test]
    fn build_scaffold_lays_out_paths_and_symlink() {
        let root = Path::new("/tmp/proj/.doctrine/slice");
        let s = build_scaffold(root, 3, "vendor-skills", "Vendor skills", "2026-06-03").unwrap();

        assert_eq!(s.dir, root.join("003"));
        assert_eq!(s.toml_path, root.join("003/slice-003.toml"));
        assert_eq!(s.md_path, root.join("003/slice-003.md"));
        assert_eq!(s.symlink, root.join("003-vendor-skills"));
        assert_eq!(s.symlink_target, "003");
        assert!(s.toml_body.contains("2026-06-03"));
        assert!(s.md_body.contains("Vendor skills"));
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

    // --- reserve_create ---

    #[test]
    fn reserve_create_writes_well_formed_slice() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".doctrine/slice");

        let s = reserve_create(&root, "my-slug", "My Title", "2026-06-03", || {
            scan_ids(&root)
        })
        .unwrap();

        assert_eq!(s.id, 1);
        assert!(root.join("001").is_dir());
        assert!(root.join("001/slice-001.toml").is_file());
        assert!(root.join("001/slice-001.md").is_file());
        assert_eq!(
            fs::read_link(root.join("001-my-slug")).unwrap(),
            Path::new("001")
        );

        let toml_body = fs::read_to_string(root.join("001/slice-001.toml")).unwrap();
        assert!(toml_body.contains("id = 1"));
        assert!(toml_body.contains("2026-06-03"));
    }

    #[test]
    fn reserve_create_retries_on_collision_and_lands_next_id() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".doctrine/slice");
        fs::create_dir_all(&root).unwrap();
        // Pre-claim 001 on disk, but feed a stale (empty) listing first so the
        // candidate is 001 and the mkdir hits AlreadyExists → recompute.
        fs::create_dir(root.join("001")).unwrap();

        let calls = Cell::new(0u32);
        let scan = || {
            let n = calls.get();
            calls.set(n + 1);
            Ok(if n == 0 { vec![] } else { vec![1] })
        };

        let s = reserve_create(&root, "x", "T", "2026-06-03", scan).unwrap();
        assert_eq!(s.id, 2);
        assert!(root.join("002").is_dir());
        assert_eq!(calls.get(), 2, "expected one collision then success");
    }

    // --- list ---

    #[test]
    fn sort_and_filter_orders_by_id_and_filters_status() {
        let rows = vec![
            meta(2, "proposed", "b", "Two"),
            meta(1, "done", "a", "One"),
            meta(3, "proposed", "c", "Three"),
        ];

        let all = sort_and_filter(rows.clone(), None);
        assert_eq!(all.iter().map(|m| m.id).collect::<Vec<_>>(), vec![1, 2, 3]);

        let proposed = sort_and_filter(rows, Some("proposed"));
        assert_eq!(
            proposed.iter().map(|m| m.id).collect::<Vec<_>>(),
            vec![2, 3]
        );
    }

    #[test]
    fn format_list_renders_aligned_rows() {
        let rows = vec![
            meta(1, "in-progress", "add-skill-removal", "Add skill removal"),
            meta(2, "proposed", "vendor-skills", "Vendor skills"),
        ];
        let out = format_list(&rows);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("001  in-progress  add-skill-removal"));
        assert!(lines[0].ends_with("Add skill removal"));
        assert!(lines[1].starts_with("002  proposed   "));
    }

    #[test]
    fn read_metas_round_trips_a_created_slice() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".doctrine/slice");
        reserve_create(&root, "my-slug", "My Title", "2026-06-03", || {
            scan_ids(&root)
        })
        .unwrap();

        let metas = read_metas(&root).unwrap();
        assert_eq!(metas, vec![meta(1, "proposed", "my-slug", "My Title")]);
    }
}
