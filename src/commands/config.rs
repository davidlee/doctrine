// SPDX-License-Identifier: GPL-3.0-only
//! "config" subcommand — inspect and modify doctrine.toml [priority] coefficients (SL-146).

use anyhow::Result;
use clap::{Args, Subcommand};
use serde::Serialize;
use std::fmt::Write as _;
use std::io::Write;
use std::path::Path;

use crate::fsutil;
use crate::priority::config::{
    PriorityConfig, clamp_dep, clamp_general, clamp_skew, load_from_table, read_priority_table,
};

#[derive(Debug, Args)]
pub(crate) struct ConfigShowArgs {
    /// Use [priority] coefficients from doctrine.toml
    #[arg(short = 'P', long, default_value_t = true)]
    pub(crate) priority: bool,

    /// Output as JSON
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct ConfigSetArgs {
    /// Update [priority] coefficients in doctrine.toml
    #[arg(short = 'P', long, default_value_t = true)]
    pub(crate) priority: bool,

    /// Coefficient key (e.g. "coefficients.value" or "`kind_weights.SL`")
    #[arg(required_unless_present = "tag")]
    pub(crate) key: String,

    /// New f64 value
    pub(crate) value: f64,

    /// Target a specific tag in `tag_coefficients`
    #[arg(short, long)]
    pub(crate) tag: Option<String>,
}

#[derive(Debug, Args)]
pub(crate) struct ConfigGetArgs {
    /// Get from [priority] coefficients in doctrine.toml
    #[arg(short = 'P', long, default_value_t = true)]
    pub(crate) priority: bool,

    /// Coefficient key
    #[arg(required_unless_present = "tag")]
    pub(crate) key: String,

    /// Target a specific tag in `tag_coefficients`
    #[arg(short, long)]
    pub(crate) tag: Option<String>,

    /// Output raw value only
    #[arg(long)]
    pub(crate) raw: bool,

    /// Output as JSON
    #[arg(long)]
    pub(crate) json: bool,
}

#[derive(Debug, Args)]
pub(crate) struct ConfigUnsetArgs {
    /// Unset in [priority] coefficients in doctrine.toml
    #[arg(short = 'P', long, default_value_t = true)]
    pub(crate) priority: bool,

    /// Coefficient key to remove
    #[arg(required_unless_present = "tag")]
    pub(crate) key: String,

    /// Target a specific tag in `tag_coefficients`
    #[arg(short, long)]
    pub(crate) tag: Option<String>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum ConfigCommand {
    /// Show current configuration
    Show(ConfigShowArgs),
    /// Set a configuration value
    Set(ConfigSetArgs),
    /// Get a configuration value
    Get(ConfigGetArgs),
    /// Remove a configuration value
    Unset(ConfigUnsetArgs),
    /// Validate the `[dispatch]` posture in doctrine.toml (SL-166).
    ///
    /// Refuses a buffered-trunk posture whose `authoring-branch` equals
    /// `deliver_to` (design §8 R4). Inert when the posture is off. The
    /// set-but-unresolvable-ref check (g2) arrives in a later phase.
    Validate,
}

pub(crate) struct ConfigPath {
    pub(crate) components: Vec<String>,
}

impl ConfigPath {
    pub(crate) fn parse(path: &str) -> Self {
        Self {
            components: path
                .split('.')
                .filter(|s| !s.is_empty())
                .map(std::string::ToString::to_string)
                .collect(),
        }
    }

    #[expect(dead_code, reason = "used by later phases")]
    pub(crate) fn join(&self) -> String {
        self.components.join(".")
    }
}

#[derive(Serialize)]
struct ConfigEntry {
    key: String,
    raw: Option<f64>,
    effective: f64,
    annotation: Option<String>,
}

pub(crate) fn run_config_show(root: &Path, args: &ConfigShowArgs) -> Result<()> {
    let raw_table = read_priority_table(root).unwrap_or_default();
    let effective = load_from_table(&raw_table);

    let entries = gather_entries(&raw_table, &effective);

    let mut stdout = std::io::stdout().lock();

    if args.json {
        writeln!(stdout, "{}", serde_json::to_string_pretty(&entries)?)?;
        return Ok(());
    }

    let mut current_prefix = String::new();
    for entry in entries {
        if let Some(pos) = entry.key.rfind('.') {
            let prefix = &entry.key[..pos];
            if prefix != current_prefix {
                writeln!(stdout, "\n# [priority.{prefix}]")?;
                current_prefix = prefix.to_string();
            }
        } else if !current_prefix.is_empty() {
            writeln!(stdout)?;
            current_prefix = String::new();
        }

        let annotation = entry
            .annotation
            .as_ref()
            .map(|a| format!("  # {a}"))
            .unwrap_or_default();
        writeln!(
            stdout,
            "{:<30} {:<10} {}",
            entry.key, entry.effective, annotation
        )?;
    }

    Ok(())
}

fn gather_entries(raw_table: &toml::Table, effective: &PriorityConfig) -> Vec<ConfigEntry> {
    let mut entries = Vec::new();

    // coefficients
    add_entry(
        &mut entries,
        "coefficients.value",
        get_raw(raw_table, &["coefficients", "value"]),
        effective.coefficients.value,
        1.0,
    );
    add_entry(
        &mut entries,
        "coefficients.risk",
        get_raw(raw_table, &["coefficients", "risk"]),
        effective.coefficients.risk,
        2.0,
    );

    // consequence
    add_entry(
        &mut entries,
        "consequence.dep_coeff",
        get_raw(raw_table, &["consequence", "dep_coeff"]),
        effective.consequence.dep_coeff,
        0.5,
    );
    add_entry(
        &mut entries,
        "consequence.ref_coeff",
        get_raw(raw_table, &["consequence", "ref_coeff"]),
        effective.consequence.ref_coeff,
        1.0,
    );

    // estimate
    add_entry(
        &mut entries,
        "estimate.skew",
        get_raw(raw_table, &["estimate", "skew"]),
        effective.estimate.skew,
        0.65,
    );
    add_entry(
        &mut entries,
        "estimate.margin",
        get_raw(raw_table, &["estimate", "margin"]),
        effective.estimate.margin,
        1.0,
    );

    // kind_weights
    let mut kinds: std::collections::BTreeSet<_> = effective.kind_weights.keys().collect();
    if let Some(raw_kinds) = raw_table
        .get("kind_weights")
        .and_then(toml::Value::as_table)
    {
        for k in raw_kinds.keys() {
            kinds.insert(k);
        }
    }
    for kind in kinds {
        let key = format!("kind_weights.{kind}");
        add_entry(
            &mut entries,
            &key,
            get_raw(raw_table, &["kind_weights", kind]),
            effective.kind_weight(kind),
            1.0,
        );
    }

    // tag_coefficients
    let mut tags: std::collections::BTreeSet<_> = effective.tag_coefficients.keys().collect();
    if let Some(raw_tags) = raw_table
        .get("tag_coefficients")
        .and_then(toml::Value::as_table)
    {
        for t in raw_tags.keys() {
            tags.insert(t);
        }
    }
    for tag in tags {
        let key = format!("tag_coefficients.{tag}");
        add_entry(
            &mut entries,
            &key,
            get_raw(raw_table, &["tag_coefficients", tag]),
            effective.tag_coeff(tag),
            1.0,
        );
    }

    entries
}

fn get_raw(table: &toml::Table, path: &[&str]) -> Option<f64> {
    let mut current = table;
    for (i, component) in path.iter().enumerate() {
        if i == path.len() - 1 {
            return current.get(*component).and_then(toml::Value::as_float);
        }
        current = current.get(*component).and_then(toml::Value::as_table)?;
    }
    None
}

fn add_entry(
    entries: &mut Vec<ConfigEntry>,
    key: &str,
    raw: Option<f64>,
    effective: f64,
    _default: f64,
) {
    let annotation = match raw {
        None => Some("default".to_string()),
        Some(r) if (r - effective).abs() > f64::EPSILON => Some(format!("clamped from {r}")),
        _ => None,
    };
    entries.push(ConfigEntry {
        key: key.to_string(),
        raw,
        effective,
        annotation,
    });
}

pub(crate) fn run_config_set(root: &Path, args: &ConfigSetArgs) -> Result<()> {
    let key = if let Some(tag) = &args.tag {
        format!("tag_coefficients.{tag}")
    } else {
        args.key.clone()
    };

    let path = ConfigPath::parse(&key);
    let (clamped, _original_fallback) = match path.components.as_slice() {
        [c, f] if c == "coefficients" && f == "value" => (clamp_general(args.value, 1.0), 1.0),
        [c, f] if c == "coefficients" && f == "risk" => (clamp_general(args.value, 2.0), 2.0),
        [c, f] if c == "consequence" && f == "dep_coeff" => (clamp_dep(args.value), 0.5),
        [c, f] if c == "consequence" && f == "ref_coeff" => (clamp_general(args.value, 1.0), 1.0),
        [c, f] if c == "estimate" && f == "skew" => (clamp_skew(args.value), 0.65),
        [c, f] if c == "estimate" && f == "margin" => (clamp_general(args.value, 1.0), 1.0),
        [c, _] if c == "kind_weights" => (clamp_general(args.value, 1.0), 1.0),
        [c, _] if c == "tag_coefficients" => (clamp_general(args.value, 1.0), 1.0),
        _ => anyhow::bail!("Unknown config key: {key}"),
    };

    let config_path = root.join(crate::dtoml::DOCTRINE_TOML);
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text = std::fs::read_to_string(&config_path).unwrap_or_default();
    let mut doc = text.parse::<toml_edit::DocumentMut>()?;

    // Navigate to [priority]
    let root_table = doc.as_table_mut();
    if root_table.get("priority").is_none() {
        root_table.insert("priority", toml_edit::Item::Table(toml_edit::Table::new()));
    }
    let priority = root_table
        .get_mut("priority")
        .and_then(toml_edit::Item::as_table_mut)
        .ok_or_else(|| anyhow::anyhow!("[priority] exists but is not a table"))?;

    // Build clamping annotation before the no-op check.
    let mut clamp_note = String::new();
    if !args.value.is_finite() {
        write!(
            clamp_note,
            " (clamped from non-finite to default {clamped})"
        )?;
    } else if (args.value - clamped).abs() > f64::EPSILON {
        write!(clamp_note, " (clamped from {})", args.value)?;
    }

    // Navigate/Create sub-tables
    let mut current_table = priority;
    for (i, component) in path.components.iter().enumerate() {
        if i == path.components.len() - 1 {
            // No-op guard: if existing == clamped and no clamping occurred, skip.
            if clamp_note.is_empty()
                && let Some(existing) = current_table
                    .get(component)
                    .and_then(toml_edit::Item::as_value)
                && let Some(existing_f) = existing.as_float()
                && (existing_f - clamped).abs() < f64::EPSILON
            {
                writeln!(
                    std::io::stdout().lock(),
                    "{key} is already set to {clamped}"
                )?;
                return Ok(());
            }

            current_table.insert(component, toml_edit::value(clamped));
        } else {
            if current_table.get(component).is_none() {
                current_table.insert(component, toml_edit::Item::Table(toml_edit::Table::new()));
            }
            current_table = current_table
                .get_mut(component)
                .and_then(toml_edit::Item::as_table_mut)
                .ok_or_else(|| {
                    anyhow::anyhow!("'priority.{component}' exists but is not a table")
                })?;
        }
    }

    fsutil::write_atomic(&config_path, doc.to_string().as_bytes())?;

    let mut msg = format!("Set {key} = {clamped}");
    msg.push_str(&clamp_note);
    writeln!(std::io::stdout().lock(), "{msg}")?;

    Ok(())
}

pub(crate) fn run_config_get(root: &Path, args: &ConfigGetArgs) -> Result<()> {
    let raw_table = read_priority_table(root).unwrap_or_default();
    let effective = load_from_table(&raw_table);

    let key = if let Some(tag) = &args.tag {
        format!("tag_coefficients.{tag}")
    } else {
        args.key.clone()
    };

    let path = ConfigPath::parse(&key);
    let value = match path.components.as_slice() {
        [c, f] if c == "coefficients" && f == "value" => Some(effective.coefficients.value),
        [c, f] if c == "coefficients" && f == "risk" => Some(effective.coefficients.risk),
        [c, f] if c == "consequence" && f == "dep_coeff" => Some(effective.consequence.dep_coeff),
        [c, f] if c == "consequence" && f == "ref_coeff" => Some(effective.consequence.ref_coeff),
        [c, f] if c == "estimate" && f == "skew" => Some(effective.estimate.skew),
        [c, f] if c == "estimate" && f == "margin" => Some(effective.estimate.margin),
        [c, k] if c == "kind_weights" => Some(effective.kind_weight(k)),
        [c, t] if c == "tag_coefficients" => Some(effective.tag_coeff(t)),
        _ => None,
    };

    let Some(val) = value else {
        anyhow::bail!("Unknown config key: {key}");
    };

    if args.json {
        writeln!(
            std::io::stdout().lock(),
            "{}",
            serde_json::json!({
                "key": key,
                "value": val
            })
        )?;
    } else if args.raw {
        writeln!(std::io::stdout().lock(), "{val}")?;
    } else {
        writeln!(std::io::stdout().lock(), "{key} = {val}")?;
    }

    Ok(())
}

pub(crate) fn run_config_unset(root: &Path, args: &ConfigUnsetArgs) -> Result<()> {
    let key = if let Some(tag) = &args.tag {
        format!("tag_coefficients.{tag}")
    } else {
        args.key.clone()
    };

    let path = ConfigPath::parse(&key);
    // Validate path
    match path.components.as_slice() {
        [c, f] if c == "coefficients" && (f == "value" || f == "risk") => {}
        [c, f] if c == "consequence" && (f == "dep_coeff" || f == "ref_coeff") => {}
        [c, f] if c == "estimate" && (f == "skew" || f == "margin") => {}
        [c, _] if c == "kind_weights" => {}
        [c, _] if c == "tag_coefficients" => {}
        _ => anyhow::bail!("Unknown or invalid config key: {key}"),
    }

    let config_path = root.join(crate::dtoml::DOCTRINE_TOML);
    if !config_path.exists() {
        writeln!(std::io::stdout().lock(), "{key} is not set")?;
        return Ok(());
    }

    let text = std::fs::read_to_string(&config_path)?;
    if text.trim().is_empty() {
        writeln!(std::io::stdout().lock(), "{key} is not set")?;
        return Ok(());
    }
    let mut doc = text.parse::<toml_edit::DocumentMut>()?;

    // Navigate to [priority]
    if doc
        .as_table_mut()
        .get_mut("priority")
        .and_then(toml_edit::Item::as_table_mut)
        .is_none()
    {
        writeln!(std::io::stdout().lock(), "{key} is not set")?;
        return Ok(());
    }

    // Re-navigating from root to be safe and avoid unsafe
    if let Some(p) = doc
        .as_table_mut()
        .get_mut("priority")
        .and_then(toml_edit::Item::as_table_mut)
    {
        let mut current = p;
        for (i, component) in path.components.iter().enumerate() {
            if i == path.components.len() - 1 {
                if current.remove(component).is_none() {
                    writeln!(std::io::stdout().lock(), "{key} is not set")?;
                    return Ok(());
                }
            } else if let Some(next) = current
                .get_mut(component)
                .and_then(toml_edit::Item::as_table_mut)
            {
                current = next;
            } else {
                writeln!(std::io::stdout().lock(), "{key} is not set")?;
                return Ok(());
            }
        }
    }

    // Optional: cleanup empty tables
    // We'll do a simple one-level cleanup for now: if the immediate sub-table is empty, remove it from [priority]
    if let Some(p) = doc
        .as_table_mut()
        .get_mut("priority")
        .and_then(toml_edit::Item::as_table_mut)
    {
        if let [sub_table_name, _leaf] = path.components.as_slice() {
            let is_empty = p
                .get(sub_table_name)
                .and_then(toml_edit::Item::as_table)
                .is_some_and(toml_edit::Table::is_empty);
            if is_empty {
                p.remove(sub_table_name);
            }
        }

        // If [priority] itself is now empty, remove it from doc
        if p.is_empty() {
            doc.as_table_mut().remove("priority");
        }
    }

    fsutil::write_atomic(&config_path, doc.to_string().as_bytes())?;
    writeln!(std::io::stdout().lock(), "Unset {key}")?;

    Ok(())
}

/// `doctrine config validate` — static `[dispatch]` posture coherence check
/// (SL-166 design §8 R4). Loads the resolved config and refuses a posture whose
/// `authoring-branch` equals `deliver_to`. Inert (Ok) when the posture is off.
pub(crate) fn run_config_validate(root: &Path) -> Result<()> {
    crate::dtoml::load_doctrine_toml(root)?
        .dispatch
        .validate_posture()?;
    writeln!(std::io::stdout().lock(), "config: posture ok")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[expect(dead_code, reason = "test helper")]
    fn setup_root(toml_content: &str) -> tempfile::TempDir {
        let dir = tempdir().unwrap();
        let config_dir = dir.path().join(".doctrine");
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(dir.path().join(crate::dtoml::DOCTRINE_TOML), toml_content).unwrap();
        dir
    }

    #[test]
    fn test_config_path_parse() {
        let path = ConfigPath::parse("coefficients.value");
        assert_eq!(path.components, vec!["coefficients", "value"]);

        let path2 = ConfigPath::parse(".kind_weights.SL.");
        assert_eq!(path2.components, vec!["kind_weights", "SL"]);
    }

    #[test]
    fn test_gather_entries_defaults() {
        let raw = toml::Table::new();
        let effective = PriorityConfig::default();
        let entries = gather_entries(&raw, &effective);

        let val = entries
            .iter()
            .find(|e| e.key == "coefficients.value")
            .unwrap();
        assert_eq!(val.effective, 1.0);
        assert_eq!(val.annotation, Some("default".to_string()));

        let risk = entries
            .iter()
            .find(|e| e.key == "coefficients.risk")
            .unwrap();
        assert_eq!(risk.effective, 2.0);
        assert_eq!(risk.annotation, Some("default".to_string()));
    }

    #[test]
    fn test_gather_entries_clamped() {
        let mut raw = toml::Table::new();
        let mut coeffs = toml::Table::new();
        coeffs.insert("value".to_string(), toml::Value::Float(1e12)); // over MAX
        raw.insert("coefficients".to_string(), toml::Value::Table(coeffs));

        let effective = load_from_table(&raw);
        let entries = gather_entries(&raw, &effective);

        let val = entries
            .iter()
            .find(|e| e.key == "coefficients.value")
            .unwrap();
        assert_eq!(val.effective, 1e9); // COEFF_MAX
        assert!(val.annotation.as_ref().unwrap().contains("clamped"));
    }

    #[test]
    fn test_run_config_set_integration() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // 1. Set a value in a new file
        let args = ConfigSetArgs {
            priority: true,
            key: "coefficients.value".to_string(),
            value: 5.5,
            tag: None,
        };
        run_config_set(root, &args).unwrap();

        let config_file = root.join(crate::dtoml::DOCTRINE_TOML);
        let content = fs::read_to_string(&config_file).unwrap();
        assert!(content.contains("[priority.coefficients]"));
        assert!(content.contains("value = 5.5"));

        // 2. Set another value in the same file
        let args2 = ConfigSetArgs {
            priority: true,
            key: "consequence.dep_coeff".to_string(),
            value: 0.8,
            tag: None,
        };
        run_config_set(root, &args2).unwrap();
        let content2 = fs::read_to_string(&config_file).unwrap();
        assert!(content2.contains("[priority.consequence]"));
        assert!(content2.contains("dep_coeff = 0.8"));
        assert!(content2.contains("value = 5.5")); // Still there

        // 3. Clamping
        let args3 = ConfigSetArgs {
            priority: true,
            key: "coefficients.risk".to_string(),
            value: -10.0,
            tag: None,
        };
        run_config_set(root, &args3).unwrap();
        let content3 = fs::read_to_string(&config_file).unwrap();
        assert!(content3.contains("risk = 0.0"));

        // 4. Tag shortcut
        let args4 = ConfigSetArgs {
            priority: true,
            key: "".to_string(), // Ignored when tag is Some
            value: 1.5,
            tag: Some("area:risk".to_string()),
        };
        run_config_set(root, &args4).unwrap();
        let content4 = fs::read_to_string(&config_file).unwrap();
        assert!(content4.contains("[priority.tag_coefficients]"));
        assert!(content4.contains("\"area:risk\" = 1.5"));
    }

    #[test]
    fn test_run_config_unset_integration() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // 1. Setup: set some values
        let set_args = ConfigSetArgs {
            priority: true,
            key: "coefficients.value".to_string(),
            value: 5.5,
            tag: None,
        };
        run_config_set(root, &set_args).unwrap();

        let set_args2 = ConfigSetArgs {
            priority: true,
            key: "kind_weights.SL".to_string(),
            value: 1.2,
            tag: None,
        };
        run_config_set(root, &set_args2).unwrap();

        // 2. Unset one value
        let unset_args = ConfigUnsetArgs {
            priority: true,
            key: "coefficients.value".to_string(),
            tag: None,
        };
        run_config_unset(root, &unset_args).unwrap();

        let config_file = root.join(crate::dtoml::DOCTRINE_TOML);
        let content = fs::read_to_string(&config_file).unwrap();
        assert!(!content.contains("value = 5.5"));
        assert!(!content.contains("[priority.coefficients]")); // Table should be cleaned up
        assert!(content.contains("[priority.kind_weights]"));
        assert!(content.contains("SL = 1.2"));

        // 3. Unset the other value
        let unset_args2 = ConfigUnsetArgs {
            priority: true,
            key: "kind_weights.SL".to_string(),
            tag: None,
        };
        run_config_unset(root, &unset_args2).unwrap();

        let content2 = fs::read_to_string(&config_file).unwrap();
        assert!(content2.trim().is_empty() || !content2.contains("[priority]"));

        // 4. Unset non-existent (idempotent)
        run_config_unset(root, &unset_args2).unwrap();
    }

    // --- config validate (SL-166 PHASE-01) ---

    fn write_doctrine_toml(body: &str) -> tempfile::TempDir {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".doctrine")).unwrap();
        fs::write(dir.path().join(crate::dtoml::DOCTRINE_TOML), body).unwrap();
        dir
    }

    #[test]
    fn config_validate_rejects_authoring_equals_deliver_to() {
        // deliver-to defaults to refs/heads/main; authoring-branch set equal.
        let dir = write_doctrine_toml("[dispatch]\nauthoring-branch = \"refs/heads/main\"\n");
        let err = run_config_validate(dir.path()).unwrap_err().to_string();
        assert!(
            err.contains("authoring-branch") && err.contains("deliver-to"),
            "{err}"
        );
    }

    #[test]
    fn config_validate_ok_when_posture_differs() {
        let dir = write_doctrine_toml("[dispatch]\nauthoring-branch = \"refs/heads/edge\"\n");
        run_config_validate(dir.path()).unwrap();
    }

    #[test]
    fn config_validate_ok_when_posture_unset() {
        // No doctrine.toml at all → defaults → posture off → inert.
        let dir = tempdir().unwrap();
        run_config_validate(dir.path()).unwrap();
    }

    // --- estimate.skew / estimate.margin (SL-172 PHASE-03) ---

    /// VT-1: on a default project, `get estimate.skew` returns the effective default 0.65.
    #[test]
    fn estimate_get_skew_default() {
        let dir = tempdir().unwrap();

        // The get logic resolves through load_from_table; test the effective value.
        let raw_table = read_priority_table(dir.path()).unwrap_or_default();
        let effective = load_from_table(&raw_table);
        assert!((effective.estimate.skew - 0.65).abs() < f64::EPSILON);

        // Also test via get's match arm logic: should yield Some(effective.estimate.skew)
        let path = ConfigPath::parse("estimate.skew");
        let val = match path.components.as_slice() {
            [c, f] if c == "estimate" && f == "skew" => Some(effective.estimate.skew),
            _ => None,
        };
        assert_eq!(val, Some(0.65));
    }

    /// VT-2: `set estimate.margin 2` then `get estimate.margin` returns 2.0;
    /// `show` output contains both an `estimate.skew` and an `estimate.margin` row.
    #[test]
    fn estimate_set_get_show() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // Set estimate.margin
        let set_args = ConfigSetArgs {
            priority: true,
            key: "estimate.margin".to_string(),
            value: 2.0,
            tag: None,
        };
        run_config_set(root, &set_args).unwrap();

        // Get estimate.margin → should be 2.0
        let raw_table = read_priority_table(root).unwrap_or_default();
        let effective = load_from_table(&raw_table);
        assert!((effective.estimate.margin - 2.0).abs() < f64::EPSILON);
        assert!((effective.estimate.skew - 0.65).abs() < f64::EPSILON);

        // show: entries should contain both estimate.skew and estimate.margin
        let entries = gather_entries(&raw_table, &effective);
        let skew_entry = entries.iter().find(|e| e.key == "estimate.skew").unwrap();
        assert_eq!(skew_entry.effective, 0.65);
        assert_eq!(skew_entry.annotation, Some("default".to_string()));
        let margin_entry = entries.iter().find(|e| e.key == "estimate.margin").unwrap();
        assert_eq!(margin_entry.effective, 2.0);
        // raw was set, so annotation should not be "default" (it's set explicitly)
        assert!(margin_entry.annotation.is_none());
    }

    /// VT-3: after setting it, `unset estimate.skew` reverts the effective value to 0.65.
    #[test]
    fn estimate_unset_reverts_default() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // Set estimate.skew
        run_config_set(
            root,
            &ConfigSetArgs {
                priority: true,
                key: "estimate.skew".to_string(),
                value: 0.9,
                tag: None,
            },
        )
        .unwrap();

        // Verify it's set
        let raw = read_priority_table(root).unwrap_or_default();
        let eff = load_from_table(&raw);
        assert!((eff.estimate.skew - 0.9).abs() < f64::EPSILON);

        // Unset
        run_config_unset(
            root,
            &ConfigUnsetArgs {
                priority: true,
                key: "estimate.skew".to_string(),
                tag: None,
            },
        )
        .unwrap();

        // Re-read: should be back to default
        let raw2 = read_priority_table(root).unwrap_or_default();
        let eff2 = load_from_table(&raw2);
        assert!((eff2.estimate.skew - 0.65).abs() < f64::EPSILON);
    }

    /// Guard: genuinely unknown keys still error "Unknown config key".
    #[test]
    fn estimate_unknown_keys_error() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // get
        let err = run_config_get(
            root,
            &ConfigGetArgs {
                priority: true,
                key: "estimate.bogus".to_string(),
                tag: None,
                raw: false,
                json: false,
            },
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("Unknown config key"), "{err}");

        // set
        let err2 = run_config_set(
            root,
            &ConfigSetArgs {
                priority: true,
                key: "nonsense.key".to_string(),
                value: 1.0,
                tag: None,
            },
        )
        .unwrap_err()
        .to_string();
        assert!(err2.contains("Unknown config key"), "{err2}");

        // unset
        let err3 = run_config_unset(
            root,
            &ConfigUnsetArgs {
                priority: true,
                key: "estimate.bogus".to_string(),
                tag: None,
            },
        )
        .unwrap_err()
        .to_string();
        assert!(err3.contains("Unknown"), "{err3}");
    }
}
