// SPDX-License-Identifier: GPL-3.0-only
//! `doctrine estimate` / `doctrine value` — facet set/clear commands (SL-118 PHASE-03).
//! SL-129: uses `entity::id_path`

use std::path::PathBuf;

use anyhow::Context;
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

/// `doctrine risk set <ID> ...`
#[derive(Args)]
pub(crate) struct RiskSetArgs {
    /// Canonical entity ref (e.g. RSK-001)
    pub(crate) id: String,

    /// Likelihood axis level
    #[arg(long, value_enum)]
    pub(crate) likelihood: Option<crate::risk::RiskLevel>,

    /// Impact axis level
    #[arg(long, value_enum)]
    pub(crate) impact: Option<crate::risk::RiskLevel>,

    /// Risk origin (free-text label)
    #[arg(long)]
    pub(crate) origin: Option<String>,

    /// Controls — each occurrence replaces the entire list (not additive)
    #[arg(
        long,
        long_help = "Controls — each occurrence replaces the entire list (not additive)"
    )]
    pub(crate) controls: Vec<String>,

    /// Explicit project root (default: auto-detect)
    #[arg(short = 'p', long)]
    pub(crate) path: Option<PathBuf>,
}

/// `doctrine risk clear <ID>`
#[derive(Args)]
pub(crate) struct RiskClearArgs {
    /// Canonical entity ref (e.g. RSK-001)
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

/// Read the `kind` field from a backlog entity TOML.
fn read_kind(path: &std::path::Path) -> anyhow::Result<String> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("entity not found at {}", path.display()))?;
    let doc = text
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    match doc.get("kind").and_then(toml_edit::Item::as_str) {
        Some(s) => Ok(s.to_owned()),
        None => anyhow::bail!("no 'kind' field — not a backlog item"),
    }
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

pub(crate) fn run_risk_set(args: &RiskSetArgs) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(args.path.clone(), &crate::root::default_markers())?;
    let (path, canonical) = resolve_entity_path_and_canonical(&root, &args.id)?;

    // Kind gate: must be a risk item.
    let kind = read_kind(&path)?;
    if kind != "risk" {
        anyhow::bail!("{canonical}: risk set requires a risk item, got {kind}");
    }

    // At-least-one axis guard.
    if args.likelihood.is_none() && args.impact.is_none() {
        anyhow::bail!("risk set: must supply at least one of --likelihood or --impact");
    }

    // Build FacetField list.
    let mut fields: Vec<crate::facet_write::FacetField> = Vec::new();
    if let Some(ref level) = args.likelihood {
        fields.push(crate::facet_write::FacetField::Str {
            key: "likelihood",
            value: level.as_str().to_owned(),
        });
    }
    if let Some(ref level) = args.impact {
        fields.push(crate::facet_write::FacetField::Str {
            key: "impact",
            value: level.as_str().to_owned(),
        });
    }
    if let Some(ref origin) = args.origin {
        fields.push(crate::facet_write::FacetField::Str {
            key: "origin",
            value: origin.clone(),
        });
    }
    if !args.controls.is_empty() {
        fields.push(crate::facet_write::FacetField::Arr {
            key: "controls",
            values: args.controls.clone(),
        });
    }

    let changed = crate::facet_write::apply_set_mixed(&path, "facet", &fields)?;

    // Build echo parts (Vec<String> + join — house style).
    if changed {
        let mut parts: Vec<String> = Vec::new();
        if let Some(ref level) = args.likelihood {
            parts.push(format!("likelihood={}", level.as_str()));
        }
        if let Some(ref level) = args.impact {
            parts.push(format!("impact={}", level.as_str()));
        }
        if let Some(ref origin) = args.origin {
            parts.push(format!("origin={origin:?}"));
        }
        if !args.controls.is_empty() {
            let list: Vec<String> = args.controls.iter().map(|c| format!("{c:?}")).collect();
            parts.push(format!("controls=[{}]", list.join(", ")));
        }
        let detail = parts.join(" ");
        writeln!(std::io::stdout(), "risk set: {canonical} {detail}")?;
    } else {
        // Unchanged — same detail pattern.
        let mut parts: Vec<String> = Vec::new();
        if let Some(ref level) = args.likelihood {
            parts.push(format!("likelihood={}", level.as_str()));
        }
        if let Some(ref level) = args.impact {
            parts.push(format!("impact={}", level.as_str()));
        }
        if let Some(ref origin) = args.origin {
            parts.push(format!("origin={origin:?}"));
        }
        if !args.controls.is_empty() {
            let list: Vec<String> = args.controls.iter().map(|c| format!("{c:?}")).collect();
            parts.push(format!("controls=[{}]", list.join(", ")));
        }
        let detail = parts.join(" ");
        writeln!(std::io::stdout(), "risk unchanged: {canonical} {detail}")?;
    }
    Ok(())
}

pub(crate) fn run_risk_clear(args: &RiskClearArgs) -> anyhow::Result<()> {
    use std::io::Write;
    let root = crate::root::find(args.path.clone(), &crate::root::default_markers())?;
    let (path, canonical) = resolve_entity_path_and_canonical(&root, &args.id)?;

    // Kind gate.
    let kind = read_kind(&path)?;
    if kind != "risk" {
        anyhow::bail!("{canonical}: risk clear requires a risk item, got {kind}");
    }

    let cleared = crate::facet_write::apply_clear(&path, "facet")?;
    if cleared {
        writeln!(std::io::stdout(), "risk cleared: {canonical}")?;
    } else {
        writeln!(std::io::stdout(), "no risk facet to clear: {canonical}")?;
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

    // ---- VT-1: risk set --likelihood low --impact medium writes both to [facet] ----

    #[test]
    fn risk_set_writes_both_axes() {
        let (_tmp, root) = mk_project_root();
        let (toml_path, canonical) = seed_entity(&root, "RSK", 1);
        // Overwrite with kind = "risk" so read_kind passes.
        let body = std::fs::read_to_string(&toml_path).unwrap();
        let with_kind = format!("{body}kind = \"risk\"\n");
        std::fs::write(&toml_path, with_kind).unwrap();

        let args = RiskSetArgs {
            id: canonical,
            likelihood: Some(crate::risk::RiskLevel::Low),
            impact: Some(crate::risk::RiskLevel::Medium),
            origin: None,
            controls: vec![],
            path: Some(root.clone()),
        };
        run_risk_set(&args).unwrap();

        let after = std::fs::read_to_string(&toml_path).unwrap();
        assert!(
            after.contains("likelihood = \"low\""),
            "missing likelihood:\n{after}"
        );
        assert!(
            after.contains("impact = \"medium\""),
            "missing impact:\n{after}"
        );
    }

    // ---- VT-2: risk set --likelihood only — partial write ----

    #[test]
    fn risk_set_likelihood_only() {
        let (_tmp, root) = mk_project_root();
        let (toml_path, canonical) = seed_entity(&root, "RSK", 2);
        let body = std::fs::read_to_string(&toml_path).unwrap();
        let with_kind = format!("{body}kind = \"risk\"\n");
        std::fs::write(&toml_path, with_kind).unwrap();

        let args = RiskSetArgs {
            id: canonical,
            likelihood: Some(crate::risk::RiskLevel::High),
            impact: None,
            origin: None,
            controls: vec![],
            path: Some(root.clone()),
        };
        run_risk_set(&args).unwrap();

        let after = std::fs::read_to_string(&toml_path).unwrap();
        assert!(
            after.contains("likelihood = \"high\""),
            "missing likelihood:\n{after}"
        );
        assert!(
            !after.contains("impact"),
            "impact should be absent:\n{after}"
        );
    }

    // ---- VT-3: risk set with neither axis → error ----

    #[test]
    fn risk_set_no_axis_rejected() {
        let (_tmp, root) = mk_project_root();
        let (toml_path, canonical) = seed_entity(&root, "RSK", 3);
        let body = std::fs::read_to_string(&toml_path).unwrap();
        let with_kind = format!("{body}kind = \"risk\"\n");
        std::fs::write(&toml_path, with_kind).unwrap();

        let args = RiskSetArgs {
            id: canonical,
            likelihood: None,
            impact: None,
            origin: None,
            controls: vec![],
            path: Some(root),
        };
        let err = run_risk_set(&args).unwrap_err().to_string();
        assert!(
            err.contains("must supply at least one of --likelihood or --impact"),
            "got: {err}"
        );
    }

    // ---- VT-4: risk set on non-risk item → kind-gate error ----

    #[test]
    fn risk_set_on_non_risk_kind_rejected() {
        let (_tmp, root) = mk_project_root();
        let (toml_path, _) = seed_entity(&root, "ISS", 1);
        let body = std::fs::read_to_string(&toml_path).unwrap();
        let with_kind = format!("{body}kind = \"issue\"\n");
        std::fs::write(&toml_path, with_kind).unwrap();

        let args = RiskSetArgs {
            id: "ISS-001".into(),
            likelihood: Some(crate::risk::RiskLevel::Low),
            impact: None,
            origin: None,
            controls: vec![],
            path: Some(root),
        };
        let err = run_risk_set(&args).unwrap_err().to_string();
        assert!(err.contains("risk set requires a risk item"), "got: {err}");
    }

    // ---- VT-5: risk set on non-backlog entity → error ----

    #[test]
    fn risk_set_on_non_backlog_rejected() {
        let (_tmp, root) = mk_project_root();
        seed_entity(&root, "SL", 1);

        let args = RiskSetArgs {
            id: "SL-001".into(),
            likelihood: Some(crate::risk::RiskLevel::Low),
            impact: None,
            origin: None,
            controls: vec![],
            path: Some(root),
        };
        let err = run_risk_set(&args).unwrap_err().to_string();
        assert!(
            err.contains("no 'kind' field — not a backlog item"),
            "got: {err}"
        );
    }

    // ---- VT-6: risk clear removes [facet] table ----

    #[test]
    fn risk_clear_removes_facet() {
        let (_tmp, root) = mk_project_root();
        let (toml_path, canonical) = seed_entity(&root, "RSK", 6);
        let body = std::fs::read_to_string(&toml_path).unwrap();
        let with_kind_and_facet = format!("{body}kind = \"risk\"\n[facet]\nlikelihood = \"low\"\n");
        std::fs::write(&toml_path, with_kind_and_facet).unwrap();

        let args = RiskClearArgs {
            id: canonical,
            path: Some(root.clone()),
        };
        run_risk_clear(&args).unwrap();

        let after = std::fs::read_to_string(&toml_path).unwrap();
        assert!(
            !after.contains("[facet]"),
            "[facet] should be gone:\n{after}"
        );
    }

    // ---- VT-7: risk clear on absent facet → no-op echo ----

    #[test]
    fn risk_clear_absent_noop() {
        let (_tmp, root) = mk_project_root();
        let (toml_path, canonical) = seed_entity(&root, "RSK", 7);
        let body = std::fs::read_to_string(&toml_path).unwrap();
        let with_kind = format!("{body}kind = \"risk\"\n");
        std::fs::write(&toml_path, with_kind).unwrap();

        let args = RiskClearArgs {
            id: canonical,
            path: Some(root),
        };
        // No error — just no-op.
        run_risk_clear(&args).unwrap();
    }

    // ---- VT-8: risk set idempotent — same values → no-op echo ----

    #[test]
    fn risk_set_idempotent_noop() {
        let (_tmp, root) = mk_project_root();
        let (toml_path, canonical) = seed_entity(&root, "RSK", 8);
        let body = std::fs::read_to_string(&toml_path).unwrap();
        let with_kind_and_facet =
            format!("{body}kind = \"risk\"\n[facet]\nlikelihood = \"low\"\nimpact = \"medium\"\n");
        std::fs::write(&toml_path, with_kind_and_facet).unwrap();

        let args = RiskSetArgs {
            id: canonical,
            likelihood: Some(crate::risk::RiskLevel::Low),
            impact: Some(crate::risk::RiskLevel::Medium),
            origin: None,
            controls: vec![],
            path: Some(root),
        };
        // Should succeed (no error), and the file should be unchanged.
        run_risk_set(&args).unwrap();
    }

    // ---- VT-9: risk set --origin writes origin string ----

    #[test]
    fn risk_set_origin() {
        let (_tmp, root) = mk_project_root();
        let (toml_path, canonical) = seed_entity(&root, "RSK", 9);
        let body = std::fs::read_to_string(&toml_path).unwrap();
        let with_kind = format!("{body}kind = \"risk\"\n");
        std::fs::write(&toml_path, with_kind).unwrap();

        let args = RiskSetArgs {
            id: canonical,
            likelihood: Some(crate::risk::RiskLevel::Low),
            impact: None,
            origin: Some("supply-chain".into()),
            controls: vec![],
            path: Some(root.clone()),
        };
        run_risk_set(&args).unwrap();

        let after = std::fs::read_to_string(&toml_path).unwrap();
        assert!(
            after.contains("origin = \"supply-chain\""),
            "missing origin:\n{after}"
        );
    }

    // ---- VT-10: risk set --controls A --controls B writes ["A", "B"] ----

    #[test]
    fn risk_set_controls() {
        let (_tmp, root) = mk_project_root();
        let (toml_path, canonical) = seed_entity(&root, "RSK", 10);
        let body = std::fs::read_to_string(&toml_path).unwrap();
        let with_kind = format!("{body}kind = \"risk\"\n");
        std::fs::write(&toml_path, with_kind).unwrap();

        let args = RiskSetArgs {
            id: canonical,
            likelihood: Some(crate::risk::RiskLevel::Low),
            impact: None,
            origin: None,
            controls: vec!["A".into(), "B".into()],
            path: Some(root.clone()),
        };
        run_risk_set(&args).unwrap();

        let after = std::fs::read_to_string(&toml_path).unwrap();
        assert!(
            after.contains("controls = [\"A\", \"B\"]"),
            "missing controls array:\n{after}"
        );
    }

    // ---- VT-11: risk set preserves non-managed facet keys ----

    #[test]
    fn risk_set_preserves_unknown_sibling() {
        let (_tmp, root) = mk_project_root();
        let (toml_path, canonical) = seed_entity(&root, "RSK", 11);
        let body = std::fs::read_to_string(&toml_path).unwrap();
        let with_kind_and_facet =
            format!("{body}kind = \"risk\"\n[facet]\nlikelihood = \"low\"\nnotes = \"keep me\"\n");
        std::fs::write(&toml_path, with_kind_and_facet).unwrap();

        let args = RiskSetArgs {
            id: canonical,
            likelihood: Some(crate::risk::RiskLevel::High),
            impact: None,
            origin: None,
            controls: vec![],
            path: Some(root.clone()),
        };
        run_risk_set(&args).unwrap();

        let after = std::fs::read_to_string(&toml_path).unwrap();
        assert!(
            after.contains("likelihood = \"high\""),
            "likelihood not updated:\n{after}"
        );
        assert!(
            after.contains("notes = \"keep me\""),
            "non-managed sibling lost:\n{after}"
        );
    }

    // ---- VT-12 (VT-18 in design): risk set on risk item with absent [facet] table — allocates ----

    #[test]
    fn risk_set_allocates_absent_facet() {
        let (_tmp, root) = mk_project_root();
        let (toml_path, canonical) = seed_entity(&root, "RSK", 12);
        let body = std::fs::read_to_string(&toml_path).unwrap();
        let with_kind = format!("{body}kind = \"risk\"\n");
        std::fs::write(&toml_path, with_kind).unwrap();

        let args = RiskSetArgs {
            id: canonical,
            likelihood: Some(crate::risk::RiskLevel::Critical),
            impact: Some(crate::risk::RiskLevel::Critical),
            origin: None,
            controls: vec![],
            path: Some(root.clone()),
        };
        run_risk_set(&args).unwrap();

        let after = std::fs::read_to_string(&toml_path).unwrap();
        assert!(
            after.contains("[facet]"),
            "[facet] should be allocated:\n{after}"
        );
        assert!(
            after.contains("likelihood = \"critical\""),
            "missing likelihood:\n{after}"
        );
        assert!(
            after.contains("impact = \"critical\""),
            "missing impact:\n{after}"
        );
    }
}
