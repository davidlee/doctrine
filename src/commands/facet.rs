// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine estimate` / `doctrine value` — facet set/clear commands (SL-118 PHASE-03).
//! SL-129: uses `entity::id_path`

use std::path::PathBuf;

use clap::Args;

/// `doctrine estimate set <ID> ...`
#[derive(Args)]
pub(crate) struct EstimateSetArgs {
    /// Canonical entity ref (e.g. SL-118, ADR-001)
    pub(crate) id: String,
    /// Lower bound (>= 0, finite); omit with -x
    pub(crate) lower: Option<f64>,
    /// Upper bound (>= lower, finite); omit with -x
    #[arg(allow_hyphen_values = true)]
    pub(crate) upper: Option<f64>,
    /// Point estimate — sets lower == upper == N
    #[arg(long = "exact", short = 'x', conflicts_with_all = ["lower", "upper"])]
    pub(crate) exact: Option<f64>,
    /// Explicit project root (default: auto-detect)
    #[arg(short = 'p', long)]
    pub(crate) path: Option<PathBuf>,
}

#[derive(Args)]
pub(crate) struct EstimateClearArgs {
    /// Canonical entity ref (e.g. SL-118, ADR-001)
    pub(crate) id: String,
    /// Explicit project root (default: auto-detect)
    #[arg(short = 'p', long)]
    pub(crate) path: Option<PathBuf>,
}

/// `doctrine value set <ID> <magnitude>`
#[derive(Args)]
pub(crate) struct ValueSetArgs {
    /// Canonical entity ref (e.g. SL-118)
    pub(crate) id: String,
    /// Magnitude (any finite f64 — may be negative)
    #[arg(allow_hyphen_values = true)]
    pub(crate) magnitude: f64,
    /// Explicit project root (default: auto-detect)
    #[arg(short = 'p', long)]
    pub(crate) path: Option<PathBuf>,
}

#[derive(Args)]
pub(crate) struct ValueClearArgs {
    /// Canonical entity ref (e.g. SL-118)
    pub(crate) id: String,
    /// Explicit project root (default: auto-detect)
    #[arg(short = 'p', long)]
    pub(crate) path: Option<PathBuf>,
}

/// Resolve a canonical ref like `SL-118` / `ADR-003` to the entity TOML path.
/// Returns the `PathBuf` and the resolved canonical id string.
pub(crate) fn resolve_entity_path_and_canonical(
    root: &std::path::Path,
    raw: &str,
) -> anyhow::Result<(PathBuf, String)> {
    let (kref, id) = crate::integrity::parse_canonical_ref(raw)?;
    let path = crate::entity::id_path(root, kref.kind, id, crate::entity::Ext::Toml);
    if !path.exists() {
        anyhow::bail!("entity not found: {raw}");
    }
    let canonical = crate::listing::canonical_id(kref.kind.prefix, id);
    Ok((path, canonical))
}

pub(crate) fn run_estimate_set(args: &EstimateSetArgs) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(args.path.clone(), &crate::root::default_markers())?;
    let (path, canonical) = resolve_entity_path_and_canonical(&root, &args.id)?;

    // Determine bounds from -x or positionals.
    let (lower, upper) = match args.exact {
        Some(n) => (n, n),
        None => match (args.lower, args.upper) {
            (Some(l), Some(u)) => (l, u),
            (None | Some(_), None | Some(_)) => {
                anyhow::bail!("estimate set: must supply both lower and upper, or -x/--exact");
            }
        },
    };

    // Build facet & validate (the COMPLETE rule from PHASE-01).
    let facet = crate::estimate::EstimateFacet { lower, upper };
    crate::estimate::validate(&facet)?;

    // Write via the leaf.
    let fields: &[(&str, f64)] = &[("lower", lower), ("upper", upper)];
    let changed = crate::facet_write::apply_set(&path, "estimate", fields)?;

    if changed {
        writeln!(
            std::io::stdout(),
            "estimate set: {canonical} lower={lower} upper={upper}"
        )?;
    } else {
        writeln!(
            std::io::stdout(),
            "estimate unchanged: {canonical} lower={lower} upper={upper}"
        )?;
    }
    Ok(())
}

pub(crate) fn run_estimate_clear(args: &EstimateClearArgs) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(args.path.clone(), &crate::root::default_markers())?;
    let (path, canonical) = resolve_entity_path_and_canonical(&root, &args.id)?;
    let cleared = crate::facet_write::apply_clear(&path, "estimate")?;
    if cleared {
        writeln!(std::io::stdout(), "estimate cleared: {canonical}")?;
    } else {
        writeln!(std::io::stdout(), "no estimate to clear: {canonical}")?;
    }
    Ok(())
}

pub(crate) fn run_value_set(args: &ValueSetArgs) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(args.path.clone(), &crate::root::default_markers())?;
    let (path, canonical) = resolve_entity_path_and_canonical(&root, &args.id)?;

    // Build facet & validate.
    let facet = crate::value::ValueFacet {
        value: args.magnitude,
    };
    crate::value::validate(&facet)?;

    // Write via the leaf.
    let fields: &[(&str, f64)] = &[("value", args.magnitude)];
    let changed = crate::facet_write::apply_set(&path, "value", fields)?;

    if changed {
        writeln!(
            std::io::stdout(),
            "value set: {canonical} value={}",
            args.magnitude
        )?;
    } else {
        writeln!(
            std::io::stdout(),
            "value unchanged: {canonical} value={}",
            args.magnitude
        )?;
    }
    Ok(())
}

pub(crate) fn run_value_clear(args: &ValueClearArgs) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(args.path.clone(), &crate::root::default_markers())?;
    let (path, canonical) = resolve_entity_path_and_canonical(&root, &args.id)?;
    let cleared = crate::facet_write::apply_clear(&path, "value")?;
    if cleared {
        writeln!(std::io::stdout(), "value cleared: {canonical}")?;
    } else {
        writeln!(std::io::stdout(), "no value to clear: {canonical}")?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test code")]
mod tests {
    use super::*;

    /// Seed a minimal entity TOML for testing, returning (toml_path, canonical_id).
    fn seed_entity(root: &std::path::Path, prefix: &str, id: u32) -> (std::path::PathBuf, String) {
        let padded = format!("{id:03}");
        let kref = crate::integrity::kind_by_prefix(prefix).expect("valid prefix");
        let toml_path = crate::entity::id_path(&root, kref.kind, id, crate::entity::Ext::Toml);
        let dir = toml_path.parent().unwrap().to_path_buf();
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            &toml_path,
            format!(
                "id = {id}\nslug = \"t{padded}\"\ntitle = \"Test {prefix}-{padded}\"\nstatus = \"accepted\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n"
            ),
        )
        .unwrap();
        let canonical = crate::listing::canonical_id(prefix, id);
        (toml_path, canonical)
    }

    /// Create a tempdir that `root::find` can resolve as a project root.
    fn mk_project_root() -> (tempfile::TempDir, std::path::PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join(".project"), "").unwrap();
        std::fs::create_dir_all(tmp.path().join(".doctrine")).unwrap();
        std::fs::write(tmp.path().join("doctrine.toml"), "").unwrap();
        let root = tmp.path().to_path_buf();
        (tmp, root)
    }

    // ---- VT-8: invalid matrix rejected ----

    #[test]
    fn vt8_neither_mode_rejected() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        let args = EstimateSetArgs {
            id: "SL-118".into(),
            lower: None,
            upper: None,
            exact: None,
            path: Some(root),
        };
        let err = run_estimate_set(&args).unwrap_err().to_string();
        assert!(
            err.contains("must supply both lower and upper"),
            "got: {err}"
        );
    }

    #[test]
    fn vt8_one_lone_positional_rejected() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        let args = EstimateSetArgs {
            id: "SL-118".into(),
            lower: Some(1.0),
            upper: None,
            exact: None,
            path: Some(root),
        };
        let err = run_estimate_set(&args).unwrap_err().to_string();
        assert!(
            err.contains("must supply both lower and upper"),
            "got: {err}"
        );
    }

    #[test]
    fn vt8_negative_lower_rejected() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        let args = EstimateSetArgs {
            id: "SL-118".into(),
            lower: Some(-1.0),
            upper: Some(5.0),
            exact: None,
            path: Some(root),
        };
        let err = run_estimate_set(&args).unwrap_err().to_string();
        assert!(err.contains("lower must be >= 0"), "got: {err}");
    }

    #[test]
    fn vt8_upper_lt_lower_rejected() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        let args = EstimateSetArgs {
            id: "SL-118".into(),
            lower: Some(5.0),
            upper: Some(2.0),
            exact: None,
            path: Some(root),
        };
        let err = run_estimate_set(&args).unwrap_err().to_string();
        assert!(err.contains("upper must be >= lower"), "got: {err}");
    }

    #[test]
    fn vt8_inf_lower_rejected() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        let args = EstimateSetArgs {
            id: "SL-118".into(),
            lower: Some(f64::INFINITY),
            upper: Some(5.0),
            exact: None,
            path: Some(root),
        };
        let err = run_estimate_set(&args).unwrap_err().to_string();
        assert!(err.contains("finite"), "got: {err}");
    }

    #[test]
    fn vt8_nan_lower_rejected() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        let args = EstimateSetArgs {
            id: "SL-118".into(),
            lower: Some(f64::NAN),
            upper: Some(5.0),
            exact: None,
            path: Some(root),
        };
        let err = run_estimate_set(&args).unwrap_err().to_string();
        assert!(err.contains("finite"), "got: {err}");
    }

    #[test]
    fn vt8_entity_not_found_rejected() {
        let (_tmp, root) = mk_project_root();
        // No entity seeded; resolve fails.
        let err = resolve_entity_path_and_canonical(&root, "SL-999")
            .unwrap_err()
            .to_string();
        assert!(err.contains("entity not found"), "got: {err}");
    }

    // ---- VT-9: -x N sets lower == upper == N ----

    #[test]
    fn vt9_exact_sets_point_estimate() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        let args = EstimateSetArgs {
            id: "SL-118".into(),
            lower: None,
            upper: None,
            exact: Some(3.0),
            path: Some(root.clone()),
        };
        run_estimate_set(&args).unwrap();
        let (path, _) = resolve_entity_path_and_canonical(&root, "SL-118").unwrap();
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.contains("lower = 3.0"), "missing lower:\n{body}");
        assert!(body.contains("upper = 3.0"), "missing upper:\n{body}");
    }

    // ---- VT-10: value set / value clear round-trip ----

    #[test]
    fn vt10_value_set_then_clear() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        // Set value.
        run_value_set(&ValueSetArgs {
            id: "SL-118".into(),
            magnitude: 42.0,
            path: Some(root.clone()),
        })
        .unwrap();
        let (path, _) = resolve_entity_path_and_canonical(&root, "SL-118").unwrap();
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.contains("value = 42.0"), "missing value:\n{body}");
        // Clear value.
        run_value_clear(&ValueClearArgs {
            id: "SL-118".into(),
            path: Some(root),
        })
        .unwrap();
        let body2 = std::fs::read_to_string(&path).unwrap();
        assert!(
            !body2.contains("[value]"),
            "[value] should be gone:\n{body2}"
        );
    }

    #[test]
    fn vt10_value_set_negative() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        run_value_set(&ValueSetArgs {
            id: "SL-118".into(),
            magnitude: -5.0,
            path: Some(root.clone()),
        })
        .unwrap();
        let (path, _) = resolve_entity_path_and_canonical(&root, "SL-118").unwrap();
        let body = std::fs::read_to_string(&path).unwrap();
        assert!(body.contains("value = -5.0"), "missing value:\n{body}");
    }

    #[test]
    fn vt10_value_set_inf_rejected() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        let err = run_value_set(&ValueSetArgs {
            id: "SL-118".into(),
            magnitude: f64::INFINITY,
            path: Some(root),
        })
        .unwrap_err()
        .to_string();
        assert!(err.contains("finite"), "got: {err}");
    }

    #[test]
    fn vt10_value_set_nan_rejected() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 118);
        let err = run_value_set(&ValueSetArgs {
            id: "SL-118".into(),
            magnitude: f64::NAN,
            path: Some(root),
        })
        .unwrap_err()
        .to_string();
        assert!(err.contains("finite"), "got: {err}");
    }

    // ---- VT-11: catalog scan round-trip ----
    // The catalog scan reads facets from the toml; we seed the toml directly
    // and assert the catalog carries the data. Handler tests (VT-8/9/10)
    // already prove the write path.

    #[test]
    fn vt11_catalog_scan_estimate_readback() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".doctrine")).unwrap();
        std::fs::write(root.join("doctrine.toml"), "").unwrap();
        // Seed an entity with [estimate] present.
        let padded = "118";
        let dir = root.join(".doctrine/slice").join(padded);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("slice-{padded}.toml")),
            "id = 118\nslug = \"t118\"\ntitle = \"Test\"\nstatus = \"accepted\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n[estimate]\nlower = 2.0\nupper = 8.0\n",
        )
        .unwrap();
        std::fs::write(dir.join(format!("slice-{padded}.md")), "# Test body\n").unwrap();
        // Scan catalog and find the entity.
        let catalog = crate::catalog::hydrate::scan_catalog(root).unwrap();
        let entity = catalog
            .entities
            .iter()
            .find(|e| e.kind_label == "SL" && matches!(&e.key, crate::catalog::hydrate::CatalogKey::Numbered(k) if k.id == 118))
            .expect("SL-118 should be in the catalog");
        let est = entity
            .estimate
            .as_ref()
            .expect("estimate should be present");
        assert_eq!(est.lower, 2.0);
        assert_eq!(est.upper, 8.0);
    }

    #[test]
    fn vt11_catalog_scan_estimate_clear_readback() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".doctrine")).unwrap();
        std::fs::write(root.join("doctrine.toml"), "").unwrap();
        // Seed an entity WITHOUT [estimate] — simulating clear.
        let padded = "118";
        let dir = root.join(".doctrine/slice").join(padded);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("slice-{padded}.toml")),
            "id = 118\nslug = \"t118\"\ntitle = \"Test\"\nstatus = \"accepted\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n",
        )
        .unwrap();
        std::fs::write(dir.join(format!("slice-{padded}.md")), "# Test body\n").unwrap();
        // Scan catalog — estimate should be absent.
        let catalog = crate::catalog::hydrate::scan_catalog(root).unwrap();
        let entity = catalog
            .entities
            .iter()
            .find(|e| e.kind_label == "SL" && matches!(&e.key, crate::catalog::hydrate::CatalogKey::Numbered(k) if k.id == 118))
            .expect("SL-118 should be in the catalog");
        assert!(
            entity.estimate.is_none(),
            "estimate should be None after clear, got: {:?}",
            entity.estimate
        );
    }

    // ---- VT-12: slice typed-reader round-trip ----
    // Seed a toml with [estimate] / [value] present and read back via
    // parse_optional — the pure engine path.

    #[test]
    fn vt12_slice_typed_reader_roundtrip() {
        let toml_body = "id = 118\nslug = \"t118\"\ntitle = \"Test\"\nstatus = \"accepted\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n[estimate]\nlower = 3.0\nupper = 7.0\n";
        let val: toml::Table = toml_body.parse().unwrap();
        let parsed =
            crate::estimate::parse_optional(val.get("estimate").and_then(|v| v.as_table()))
                .unwrap()
                .expect("estimate should be present");
        assert_eq!(parsed.lower, 3.0);
        assert_eq!(parsed.upper, 7.0);
    }

    #[test]
    fn vt12_value_typed_reader_roundtrip() {
        let toml_body = "id = 118\nslug = \"t118\"\ntitle = \"Test\"\nstatus = \"accepted\"\ncreated = \"2026-01-01\"\nupdated = \"2026-01-01\"\n[value]\nvalue = 99.0\n";
        let val: toml::Table = toml_body.parse().unwrap();
        let parsed = crate::value::parse_optional(val.get("value").and_then(|v| v.as_table()))
            .unwrap()
            .expect("value should be present");
        assert_eq!(parsed.value, 99.0);
    }
}
