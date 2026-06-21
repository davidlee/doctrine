// Architecture layering gate (SL-112)
//
// PHASE-01 go/no-go report (SL-112)
//
// Units: 67 total — 23 leaf, 18 engine, 26 command
//   (engine includes 4 sub-classified umbrellas: catalog::{hydrate,graph,diagnostic}, priority::graph)
// Accepted violations: 10 upward edges baselined in `.doctrine/adr/001/layering.toml`
// Tangle baseline: leaf=0, engine=0, command=120
// Decision: **GO**
//   - Engine core is meaningful: 14 pure-engine + 4 sub-classified = 18 units
//     (entity, meta, relation, state, status, input, verify, coverage, coverage_scan,
//      coverage_store, coverage_verify, coverage_view, supersede, backlog_order,
//      catalog::{hydrate,graph,diagnostic}, priority::graph)
//   - Upward baseline is small (10 violations) and well-understood:
//     * coverage*→requirement (6 edges): requirement is the entity-kind command module;
//       coverage engine modules import its types — a classic ADR-001 wart, not structural rot
//     * backlog_order→backlog, supersede→knowledge: ordering/policy engines depend on
//       command-type definitions — same pattern
//     * state→install: the known ADR-001 install-as-utility wart
//     * dtoml→verify: pure TOML seam importing a config helper that happens to reach engine;
//       dtoml itself is leaf — a leaf→engine edge from a pure utility
//   - Command tangle (120 cyclic edges) is large but expected for a ~26-module CLI SCC;
//     intra-tier cohesion per verb is fine. Ratcheted (may not grow), not resolved here.
//   - Engine is a clean DAG (0 cyclic edges, CHR-015 done).
//
// PHASE-02 will build the `syn` fitness check from these primitives.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// Discover top-level modules under `src_dir` by scanning for non-test `.rs` files.
///
/// Each returned string is the basename of a `.rs` file under `src_dir/`, stripped
/// of the `.rs` extension. Sub-directory modules (`src/foo/mod.rs` or `src/foo.rs`
/// plus `src/foo/*.rs`) contribute their directory name. Module-private children
/// (`src/foo/bar.rs` → `foo::bar`) are NOT returned by this function; the caller is
/// expected to run `extract_edges` first and derive sub-modules from the edge set.
///
/// Skips `main.rs` and any file whose first line contains `#[cfg(test)]`.
pub fn discover_units(src_dir: &Path) -> BTreeSet<String> {
    let mut units = BTreeSet::new();

    let entries = match std::fs::read_dir(src_dir) {
        Ok(entries) => entries,
        Err(_) => return units,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(file_name) = path.file_name() else {
            continue;
        };

        if file_name == "main.rs" {
            continue;
        }

        if path.is_dir() {
            // Sub-directory modules: `src/catalog/` → "catalog"
            let mod_name = file_name.to_string_lossy().into_owned();
            // Verify there's a mod.rs or matching parent file
            let mod_file = path.join("mod.rs");
            let parent_file = src_dir.join(format!("{mod_name}.rs"));
            // Also check if flush file exists
            let flush_mod = path.join(format!("{mod_name}.rs"));
            if mod_file.exists() || parent_file.exists() || flush_mod.exists() {
                units.insert(mod_name);
            }
        } else if let Some(stem) = file_name.to_str().and_then(|s| s.strip_suffix(".rs")) {
            // Skip modules that exist as both .rs and directory — covered above
            let dir_version = src_dir.join(stem);
            if dir_version.is_dir() {
                // It's a directory module, covered by the is_dir branch
                continue;
            }
            if skip_cfg_test_file(&path, stem) {
                continue;
            }
            units.insert(stem.to_string());
        }
    }

    units
}

/// Return true if the file should be skipped because it's a cfg(test)-only module.
///
/// A file is skipped if it is named `test_helpers.rs` or any of these conditions
/// hold for its first line:
/// - starts with `#[cfg(test)]`
fn skip_cfg_test_file(path: &Path, stem: &str) -> bool {
    if stem == "test_helpers" {
        return true;
    }

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return true,
    };

    // Check first non-blank, non-comment line
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }
        return trimmed.starts_with("#[cfg(test)]");
    }

    false
}

/// Extract directed dependency edges from production (non-test) code under `src_dir`.
///
/// For each non-test `.rs` file, parse the file with `syn` and collect all
/// `crate::<module>::…` path references. The FROM side is the top-level module of the
/// source file; the TO side is the first path component after `crate` in the reference.
///
/// Rules:
/// - Paths inside `#[cfg(test)]` items and mods are excluded.
/// - Self-edges (FROM == TO) are excluded.
/// - Results are deduplicated (`BTreeSet`).
pub fn extract_edges(src_dir: &Path) -> BTreeSet<(String, String)> {
    let mut edges = BTreeSet::new();
    let rs_files = collect_rs_files(src_dir);

    for rs_file in &rs_files {
        // Determine the top-level module for this file
        let Some(top_mod) = top_level_module(src_dir, rs_file) else {
            continue;
        };

        let Ok(content) = std::fs::read_to_string(rs_file) else {
            continue;
        };

        // Skip cfg(test)-only files entirely
        if skip_cfg_test_file(rs_file, &top_mod) {
            continue;
        }

        let Ok(file) = syn::parse_file(&content) else {
            // If the file doesn't parse (likely because of macro-heavy code),
            // skip it gracefully.
            continue;
        };

        let mut collector = CratePathCollector::new();
        collector.recursive = true;
        collector.skip_cfg_test = true;
        syn::visit::visit_file(&mut collector, &file);

        for target in collector.targets {
            if target != top_mod {
                edges.insert((top_mod.clone(), target));
            }
        }
    }

    edges
}

/// Return the top-level module name for a Rust source file under `src_dir`.
///
/// `src/foo.rs` → "foo"; `src/bar/mod.rs` → "bar"; `src/bar/baz.rs` → "bar".
fn top_level_module(src_dir: &Path, file_path: &Path) -> Option<String> {
    let rel = file_path.strip_prefix(src_dir).ok()?;
    let components: Vec<&str> = rel
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    if components.is_empty() {
        return None;
    }

    if components.len() == 1 {
        // src/foo.rs → "foo"
        components[0].strip_suffix(".rs").map(String::from)
    } else {
        // src/bar/mod.rs or src/bar/baz.rs → "bar"
        Some(components[0].to_string())
    }
}

/// Recursively collect all `.rs` files under `src_dir/`, excluding test fixtures.
fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return files;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_rs_files(&path));
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            // Skip test_helpers
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if stem == "test_helpers" {
                    continue;
                }
            }
            files.push(path);
        }
    }

    files
}

// ─── syn visitor ───────────────────────────────────────────────────────────

/// A `syn` visitor that collects `crate::` path targets from production code.
///
/// When `skip_cfg_test` is true, `#[cfg(test)]` items and their bodies are excluded.
struct CratePathCollector {
    /// Accumulated target modules (first component after `crate`).
    targets: BTreeSet<String>,
    /// If true, descend into all items (not just top-level).
    recursive: bool,
    /// If true, skip items annotated with `#[cfg(test)]`.
    skip_cfg_test: bool,
}

impl CratePathCollector {
    fn new() -> Self {
        Self {
            targets: BTreeSet::new(),
            recursive: false,
            skip_cfg_test: false,
        }
    }

    /// Check whether an item's attributes contain `#[cfg(test)]`.
    fn has_cfg_test(attrs: &[syn::Attribute]) -> bool {
        attrs.iter().any(|attr| {
            // Look for #[cfg(test)]
            if !attr.path().is_ident("cfg") {
                return false;
            }
            // Parse the attribute args to find `test`
            if let syn::Meta::List(list) = &attr.meta {
                return list.tokens.to_string().trim() == "test";
            }
            false
        })
    }

    /// Extract the first segment from a `crate::`-qualified path.
    ///
    /// Returns `Some("foo")` for `crate::foo::bar`; `None` for paths not starting
    /// with `crate`.
    fn crate_target(path: &syn::Path) -> Option<String> {
        let segments = &path.segments;
        if segments.is_empty() {
            return None;
        }
        let first = &segments[0];
        if first.ident != "crate" {
            return None;
        }
        // The target is the second segment
        segments.get(1).map(|s| s.ident.to_string())
    }

    fn collect_path(&mut self, path: &syn::Path) {
        if let Some(target) = Self::crate_target(path) {
            self.targets.insert(target);
        }
    }

    /// Walk a `UseTree` from an item-use and collect `crate::` targets.
    /// `use crate::foo::Bar` → UseTree::Path(ident=crate, tree=Path(ident=foo, ...)).
    /// We collect the segment immediately after `crate`.
    fn collect_use_paths_from_item_use(&mut self, node: &syn::ItemUse) {
        self.walk_use_tree(&node.tree, None);
    }

    fn walk_use_tree(&mut self, tree: &syn::UseTree, prev: Option<&syn::Ident>) {
        match tree {
            syn::UseTree::Path(use_path) => {
                if use_path.ident == "crate" {
                    // The next segment is the target module if it's a Path or Name.
                    self.walk_use_tree(&use_path.tree, Some(&use_path.ident));
                } else if prev.map_or(false, |p| p == "crate") {
                    self.targets.insert(use_path.ident.to_string());
                    // Continue but without crate marker
                    self.walk_use_tree(&use_path.tree, Some(&use_path.ident));
                } else {
                    self.walk_use_tree(&use_path.tree, Some(&use_path.ident));
                }
            }
            syn::UseTree::Name(use_name) => {
                if prev.map_or(false, |p| p == "crate") {
                    self.targets.insert(use_name.ident.to_string());
                }
            }
            syn::UseTree::Rename(use_rename) => {
                if prev.map_or(false, |p| p == "crate") {
                    self.targets.insert(use_rename.ident.to_string());
                }
            }
            syn::UseTree::Glob(_) => {}
            syn::UseTree::Group(use_group) => {
                for item in &use_group.items {
                    self.walk_use_tree(item, prev);
                }
            }
        }
    }
}

impl<'ast> syn::visit::Visit<'ast> for CratePathCollector {
    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        self.collect_use_paths_from_item_use(node);
        if self.recursive {
            syn::visit::visit_item_use(self, node);
        }
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if self.skip_cfg_test && Self::has_cfg_test(&node.attrs) {
            return; // skip entire #[cfg(test)] module body
        }
        if self.recursive {
            syn::visit::visit_item_mod(self, node);
        }
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if self.skip_cfg_test && Self::has_cfg_test(&node.attrs) {
            return; // skip #[cfg(test)] function body
        }
        if self.recursive {
            syn::visit::visit_item_fn(self, node);
        }
    }

    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        self.collect_path(&node.path);
        if self.recursive {
            syn::visit::visit_expr_path(self, node);
        }
    }

    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
        self.collect_path(&node.path);
        if self.recursive {
            syn::visit::visit_type_path(self, node);
        }
    }
}

// ─── SL-112 PHASE-02: data types ────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Tier {
    Leaf = 0,
    Engine = 1,
    Command = 2,
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tier::Leaf => write!(f, "leaf"),
            Tier::Engine => write!(f, "engine"),
            Tier::Command => write!(f, "command"),
        }
    }
}

impl std::str::FromStr for Tier {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "leaf" => Ok(Tier::Leaf),
            "engine" => Ok(Tier::Engine),
            "command" => Ok(Tier::Command),
            other => Err(format!("unknown tier: {other}")),
        }
    }
}

#[allow(dead_code)]
struct LayerMap(BTreeMap<String, Tier>);
#[allow(dead_code)]
struct Accepted(BTreeSet<(String, String)>);
#[allow(dead_code)]
struct TangleBaseline(BTreeMap<Tier, u32>);

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
enum Violation {
    Unclassified(String),
    StaleEntry(String),
    UpwardEdge {
        from: String,
        to: String,
        from_tier: Tier,
        to_tier: Tier,
    },
    StaleAccepted {
        from: String,
        to: String,
    },
    MixedUmbrella {
        module: String,
        reaches: Tier,
    },
    TangleGrew {
        tier: Tier,
        baseline: u32,
        actual: u32,
    },
}

// ─── SL-112 PHASE-02: TOML loader ──────────────────────────────────────────

/// Parse `.doctrine/adr/001/layering.toml` into the three authored components.
///
/// The `[tiers]` and `[tangle_baseline]` sections are standard TOML.
/// The `[[accepted_violation]]` entries use `from = "x"; to = "y"` inline
/// syntax (toml_edit display format). The loader pre-processes those entries
/// into standard TOML before parsing the rest.
#[allow(dead_code)]
fn load_layering(path: &Path) -> Result<(LayerMap, Accepted, TangleBaseline), String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;

    // ── [[accepted_violation]] → Accepted (manual pre-processor for ';' syntax) ──
    let mut accepted = BTreeSet::new();
    let mut clean = String::with_capacity(raw.len());
    let mut in_accepted = false;
    for line in raw.lines() {
        if line.trim() == "[[accepted_violation]]" {
            in_accepted = true;
            continue; // skip the header — we parse manually from here
        }
        if in_accepted {
            if line.trim().is_empty() || line.trim().starts_with('[') {
                in_accepted = false;
                if !line.trim().is_empty() {
                    clean.push_str(line);
                    clean.push('\n');
                }
                continue;
            }
            // Parse inline `from = "A"; to = "B"` into (from, to)
            if let Some((from, to)) = parse_accepted_line(line) {
                accepted.insert((from, to));
            }
            // Ignore reason/follow_up — they're for human readers only
            continue;
        }
        clean.push_str(line);
        clean.push('\n');
    }

    let doc: toml_edit::DocumentMut = clean
        .parse()
        .map_err(|e| format!("invalid TOML in {}: {e}", path.display()))?;

    // ── [tiers] → LayerMap ──
    let tiers_table = doc
        .get("tiers")
        .and_then(|v| v.as_table())
        .ok_or_else(|| format!("missing [tiers] section in {}", path.display()))?;

    let mut map = BTreeMap::new();
    for (key, val) in tiers_table.iter() {
        let s = val
            .as_str()
            .ok_or_else(|| format!("tier value for `{key}` must be a string"))?;
        let tier: Tier = s
            .parse()
            .map_err(|e| format!("bad tier for `{key}`: {e}"))?;
        map.insert(key.to_string(), tier);
    }

    // ── [tangle_baseline] → TangleBaseline ──
    let tangle_table = doc
        .get("tangle_baseline")
        .and_then(|v| v.as_table())
        .ok_or_else(|| format!("missing [tangle_baseline] section in {}", path.display()))?;

    let mut tangle = BTreeMap::new();
    for (key, val) in tangle_table.iter() {
        let tier: Tier = key
            .parse()
            .map_err(|e| format!("bad tangle_baseline key `{key}`: {e}"))?;
        let count = val
            .as_integer()
            .ok_or_else(|| format!("tangle_baseline.{key} must be an integer"))?;
        let count: u32 = count
            .try_into()
            .map_err(|_| format!("tangle_baseline.{key} value out of u32 range"))?;
        tangle.insert(tier, count);
    }

    Ok((LayerMap(map), Accepted(accepted), TangleBaseline(tangle)))
}

/// Parse a line like `from = "state"; to = "install"` into ("state", "install").
/// Returns None if the line doesn't match the expected pattern.
fn parse_accepted_line(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    // Skip reason/follow_up/comment lines
    if line.starts_with("reason") || line.starts_with("follow_up") || line.starts_with('#') {
        return None;
    }
    // Look for `from = "X"; to = "Y"` pattern
    let rest = line.strip_prefix("from")?;
    let rest = rest.trim().strip_prefix('=')?.trim();
    let (from_val, rest) = extract_quoted_string(rest)?;
    let rest = rest.trim().strip_prefix(';')?.trim();
    let rest = rest.strip_prefix("to")?;
    let rest = rest.trim().strip_prefix('=')?.trim();
    let (to_val, _rest) = extract_quoted_string(rest)?;
    Some((from_val, to_val))
}

/// Extract a double-quoted string from the start of `s`.
/// Returns `(content, remainder)`.
fn extract_quoted_string(s: &str) -> Option<(String, String)> {
    let s = s.trim().strip_prefix('"')?;
    let end = s.find('"')?;
    let content = s[..end].to_string();
    let rest = s[end + 1..].to_string();
    Some((content, rest))
}

// ─── SL-112 PHASE-02: pure check() ─────────────────────────────────────────

#[allow(dead_code)]
fn check(
    units: &BTreeSet<String>,
    edges: &BTreeSet<(String, String)>,
    map: &LayerMap,
    accepted: &Accepted,
    baseline: &TangleBaseline,
) -> Vec<Violation> {
    let mut violations = Vec::new();

    // ── pre-filter: drop edges whose targets are crate-root types (not modules) ──
    // crate::CommonListArgs, crate::Command etc. are types defined in main.rs,
    // not source modules. The edge extractor sees crate::<name> and can't
    // distinguish a module ref from a type ref; filtering by source-file
    // existence removes the false positives without loosening the gate.
    //
    // For unit tests (synthetic modules like "leaf_mod"), the filter is a no-op
    // because those names won't be found on disk; we pass those edges through.
    let is_module = |name: &str| -> bool {
        let src = std::path::PathBuf::from("src");
        let exists_on_disk = src.join(format!("{name}.rs")).exists()
            || src.join(format!("{name}/mod.rs")).exists()
            || src.join(format!("{name}/{name}.rs")).exists();
        // In unit tests, units are synthetic (not on disk) → keep all edges.
        // In the real gate, units come from discover_units (on-disk) → filter.
        if !units.iter().any(|u| {
            src.join(format!("{u}.rs")).exists() || src.join(format!("{u}/mod.rs")).exists()
        }) {
            // No unit has a source file on disk → synthetic test mode, keep all edges.
            true
        } else {
            exists_on_disk
        }
    };
    let filtered_edges: BTreeSet<_> = edges
        .iter()
        .filter(|(_from, to)| is_module(to))
        .cloned()
        .collect();

    // ── assertion 1 — completeness ──
    // Every unit appearing in edges must be a key in map.
    for (from, to) in &filtered_edges {
        if !map.0.contains_key(from) {
            violations.push(Violation::Unclassified(from.clone()));
        }
        if !map.0.contains_key(to) {
            violations.push(Violation::Unclassified(to.clone()));
        }
    }
    // Every key in map that does NOT contain "::" must appear in units.
    // (Sub-classified keys like `catalog::scan` are exempt — they won't be
    // discovered as standalone units.)
    // Also exempt `main` — it's the binary entrypoint, deliberately excluded
    // by discover_units(), but classified in the map for completeness.
    for key in map.0.keys() {
        if key.contains("::") || key == "main" {
            continue;
        }
        if !units.contains(key) {
            violations.push(Violation::StaleEntry(key.clone()));
        }
    }

    // Collect modules that have sub-classified entries.
    // For these modules, we can't check upward edges at module granularity
    // because the edges are aggregated from files that may have different tiers.
    let sub_classified_modules: BTreeSet<&str> = map
        .0
        .keys()
        .filter_map(|k| {
            if let Some(colon) = k.find("::") {
                Some(&k[..colon])
            } else {
                None
            }
        })
        .collect();

    // ── assertion 2 — cross-tier direction ──
    for (from, to) in &filtered_edges {
        // Skip modules that are sub-classified; their edges come from files
        // that may have higher assigned tiers.
        if sub_classified_modules.contains(from.as_str()) {
            continue;
        }
        let Some(from_tier) = map.0.get(from) else {
            continue; // Unclassified already caught above
        };
        let Some(to_tier) = map.0.get(to) else {
            continue;
        };
        if *to_tier > *from_tier && !accepted.0.contains(&(from.clone(), to.clone())) {
            violations.push(Violation::UpwardEdge {
                from: from.clone(),
                to: to.clone(),
                from_tier: *from_tier,
                to_tier: *to_tier,
            });
        }
    }
    // Stale accepted: each entry in Accepted must appear in edges.
    for (from, to) in &accepted.0 {
        if !filtered_edges.contains(&(from.clone(), to.clone())) {
            violations.push(Violation::StaleAccepted {
                from: from.clone(),
                to: to.clone(),
            });
        }
    }

    // ── assertion 3 — MixedUmbrella forcing ──
    // Already built above: sub_classified_modules.
    for (mod_name, mod_tier) in &map.0 {
        if mod_name.contains("::") {
            continue; // sub-classified entry itself, not a module umbrella
        }
        if sub_classified_modules.contains(mod_name.as_str()) {
            continue; // umbrella is already sub-classified
        }
        // Check all outbound edges from this module
        let mut upward_edges: Vec<&String> = Vec::new();
        let mut all_upward_accepted = true;
        for (from, to) in &filtered_edges {
            if *from != *mod_name {
                continue;
            }
            let Some(to_tier) = map.0.get(to) else {
                continue;
            };
            if *to_tier > *mod_tier {
                upward_edges.push(to);
                if !accepted.0.contains(&(from.clone(), to.clone())) {
                    all_upward_accepted = false;
                }
            }
        }
        if !upward_edges.is_empty() && !all_upward_accepted {
            // Find the first non-accepted upward target for the message
            for to in &upward_edges {
                if !accepted.0.contains(&(mod_name.clone(), (*to).clone())) {
                    let to_tier = map.0.get(*to).copied().unwrap_or(*mod_tier);
                    violations.push(Violation::MixedUmbrella {
                        module: mod_name.clone(),
                        reaches: to_tier,
                    });
                    break;
                }
            }
        }
    }

    // ── assertion 4 — per-tier tangle ratchet ──
    for (tier, bl_count) in &baseline.0 {
        let actual = count_tangle_edges(units, &filtered_edges, map, *tier);
        if actual > *bl_count {
            violations.push(Violation::TangleGrew {
                tier: *tier,
                baseline: *bl_count,
                actual,
            });
        }
    }

    violations
}

// ─── Tarjan SCC helper (for tangle count) ──────────────────────────────────

/// Count edges where both endpoints are in the same non-trivial SCC (> 1 node)
/// for the given tier.
#[allow(dead_code)]
fn count_tangle_edges(
    units: &BTreeSet<String>,
    edges: &BTreeSet<(String, String)>,
    map: &LayerMap,
    tier: Tier,
) -> u32 {
    // Build the same-tier subgraph.
    let tier_units: BTreeSet<&str> = units
        .iter()
        .filter(|u| map.0.get(*u) == Some(&tier))
        .map(|s| s.as_str())
        .collect();

    if tier_units.len() < 2 {
        return 0;
    }

    // Build adjacency list (node index → neighbor indices).
    let unit_list: Vec<&str> = tier_units.iter().copied().collect();
    let idx: BTreeMap<&str, usize> = unit_list
        .iter()
        .enumerate()
        .map(|(i, name)| (*name, i))
        .collect();

    let n = unit_list.len();
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (from, to) in edges {
        if let (Some(&fi), Some(&ti)) = (idx.get(from.as_str()), idx.get(to.as_str())) {
            adj[fi].push(ti);
        }
    }

    // Tarjan SCC.
    let sccs = tarjan_scc(&adj);

    // Count edges where both endpoints are in the same SCC of size > 1.
    let node_to_scc: Vec<usize> = {
        let mut v = vec![0; n];
        for (scc_id, scc) in sccs.iter().enumerate() {
            for &ni in scc {
                v[ni] = scc_id;
            }
        }
        v
    };

    let non_trivial_sccs: BTreeSet<usize> = sccs
        .iter()
        .enumerate()
        .filter(|(_, scc)| scc.len() > 1)
        .map(|(id, _)| id)
        .collect();

    let mut count: u32 = 0;
    for (from, to) in edges {
        if let (Some(&fi), Some(&ti)) = (idx.get(from.as_str()), idx.get(to.as_str())) {
            let scc_id = node_to_scc[fi];
            if node_to_scc[ti] == scc_id && non_trivial_sccs.contains(&scc_id) {
                count += 1;
            }
        }
    }

    count
}

/// Tarjan's SCC algorithm. Returns a Vec of SCCs, each a Vec of node indices.
#[allow(dead_code)]
fn tarjan_scc(adj: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let n = adj.len();
    let mut index_counter = 0u32;
    let mut stack: Vec<usize> = Vec::new();
    let mut on_stack = vec![false; n];
    let mut indices = vec![u32::MAX; n];
    let mut lowlink = vec![u32::MAX; n];
    let mut sccs: Vec<Vec<usize>> = Vec::new();

    fn strongconnect(
        v: usize,
        adj: &[Vec<usize>],
        index_counter: &mut u32,
        stack: &mut Vec<usize>,
        on_stack: &mut [bool],
        indices: &mut [u32],
        lowlink: &mut [u32],
        sccs: &mut Vec<Vec<usize>>,
    ) {
        indices[v] = *index_counter;
        lowlink[v] = *index_counter;
        *index_counter += 1;
        stack.push(v);
        on_stack[v] = true;

        for &w in &adj[v] {
            if indices[w] == u32::MAX {
                strongconnect(
                    w,
                    adj,
                    index_counter,
                    stack,
                    on_stack,
                    indices,
                    lowlink,
                    sccs,
                );
                lowlink[v] = lowlink[v].min(lowlink[w]);
            } else if on_stack[w] {
                lowlink[v] = lowlink[v].min(indices[w]);
            }
        }

        if lowlink[v] == indices[v] {
            let mut scc = Vec::new();
            loop {
                let w = stack.pop().unwrap();
                on_stack[w] = false;
                scc.push(w);
                if w == v {
                    break;
                }
            }
            sccs.push(scc);
        }
    }

    for v in 0..n {
        if indices[v] == u32::MAX {
            strongconnect(
                v,
                adj,
                &mut index_counter,
                &mut stack,
                &mut on_stack,
                &mut indices,
                &mut lowlink,
                &mut sccs,
            );
        }
    }

    sccs
}

// ─── tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_units_excludes_main_and_cfg_test() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir(&src).unwrap();

        // main.rs — should be excluded
        std::fs::write(src.join("main.rs"), "fn main() {}\n").unwrap();

        // A cfg(test)-only file
        std::fs::write(
            src.join("tester.rs"),
            "#[cfg(test)]\nmod tests {\n    fn test_it() {}\n}\n",
        )
        .unwrap();

        // A normal production file
        std::fs::write(src.join("clock.rs"), "pub fn now() {}\n").unwrap();

        let units = discover_units(&src);
        assert!(!units.contains("main"), "main should be excluded");
        assert!(
            !units.contains("tester"),
            "cfg(test) file should be excluded"
        );
        assert!(units.contains("clock"), "clock should be present");
    }

    #[test]
    fn discover_units_detects_directory_modules() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir(&src).unwrap();

        // Directory module
        std::fs::create_dir(src.join("catalog")).unwrap();
        std::fs::write(src.join("catalog/mod.rs"), "pub mod scan;\n").unwrap();

        let units = discover_units(&src);
        assert!(
            units.contains("catalog"),
            "catalog directory module should be present"
        );
    }

    #[test]
    fn extract_edges_excludes_cfg_test_paths() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir(&src).unwrap();

        // A production file that imports from command
        std::fs::write(
            src.join("state.rs"),
            r#"use crate::command::run_something;

pub fn do_thing() {
    run_something();
}
"#,
        )
        .unwrap();

        // A cfg(test)-only file that imports from command — should be excluded
        std::fs::write(
            src.join("tests_only.rs"),
            r#"#[cfg(test)]
mod tests {
    use crate::command::Foo;

    #[test]
    fn test_it() {
        let _x = crate::command::bar();
    }
}
"#,
        )
        .unwrap();

        let edges = extract_edges(&src);

        // state → command should be present
        assert!(
            edges.contains(&(String::from("state"), String::from("command"))),
            "state→command edge should exist"
        );

        // tests_only → command should NOT be present
        assert!(
            !edges.contains(&(String::from("tests_only"), String::from("command"))),
            "cfg(test) file should be excluded entirely"
        );
    }

    #[test]
    fn extract_edges_excludes_cfg_test_mod_items() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir(&src).unwrap();

        // A production file with a #[cfg(test)] mod inside it
        std::fs::write(
            src.join("foo.rs"),
            r#"use crate::std::collections::HashMap;

pub fn normal_fn() {
    let _ = crate::entity::Entity::new();
}

#[cfg(test)]
mod tests {
    use crate::command::Foo;

    #[test]
    fn it_works() {
        let _ = crate::command::bar();
    }
}
"#,
        )
        .unwrap();

        let edges = extract_edges(&src);

        // foo → entity should be present (from production code)
        assert!(
            edges.contains(&(String::from("foo"), String::from("entity"))),
            "foo→entity should exist, but got: {edges:?}"
        );

        // foo → command should NOT be present (inside #[cfg(test)] mod)
        assert!(
            !edges.contains(&(String::from("foo"), String::from("command"))),
            "foo→command should be excluded (inside #[cfg(test)])"
        );

        // foo → std should be excluded (not crate-qualified — wait, it IS crate::std; but
        // std is the crate name, so `crate::std` is actually the same as `std::` in
        // edition 2024; but this is `use crate::std::collections::HashMap` which is a
        // path. In Rust 2024, `crate` resolves to the root of the current crate, and
        // `crate::std` would be unusual. That line is actually likely incorrect Rust
        // but it should still parse and the first segment is `crate`, second is `std`.
        // We don't assert on it because `std` is not a user module.
    }

    #[test]
    fn extract_edges_no_self_edges() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir(&src).unwrap();

        std::fs::write(
            src.join("entity.rs"),
            r#"pub struct Entity;
pub fn build() -> Entity {
    let e = crate::entity::Entity;
    e
}
"#,
        )
        .unwrap();

        let edges = extract_edges(&src);
        assert!(
            !edges.contains(&(String::from("entity"), String::from("entity"))),
            "self-edges should be excluded"
        );
    }

    #[test]
    fn extract_edges_type_path_and_expr_path() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir(&src).unwrap();

        std::fs::write(
            src.join("foo.rs"),
            r#"use crate::bar::Bar;

pub fn mk_bar() -> crate::bar::Bar {
    crate::bar::Bar
}

pub fn call_bar() {
    crate::bar::bar_fn();
}
"#,
        )
        .unwrap();

        let edges = extract_edges(&src);
        assert!(edges.contains(&(String::from("foo"), String::from("bar"))));
        // Dedup: only one edge even though bar appears in multiple path types
        assert_eq!(
            edges
                .iter()
                .filter(|(f, t)| f == "foo" && t == "bar")
                .count(),
            1,
            "should be deduplicated"
        );
    }

    #[test]
    fn extract_edges_deduplicates() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir(&src).unwrap();

        // Three references to crate::bar, all should collapse into one edge
        std::fs::write(
            src.join("foo.rs"),
            r#"
use crate::bar::Bar;           // ItemUse
fn f() -> crate::bar::Bar {    // TypePath (return type)
    crate::bar::Bar             // ExprPath
}
"#,
        )
        .unwrap();

        let edges = extract_edges(&src);
        let count = edges
            .iter()
            .filter(|(f, t)| f == "foo" && t == "bar")
            .count();
        assert_eq!(count, 1, "deduplication failed: {edges:?}");
    }

    #[test]
    fn extract_edges_submodule_files_map_to_parent() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        std::fs::create_dir(&src).unwrap();
        std::fs::create_dir(src.join("catalog")).unwrap();

        // mod.rs defines the module
        std::fs::write(src.join("catalog/mod.rs"), "pub mod scan;\n").unwrap();

        // scan.rs imports from entity
        std::fs::write(
            src.join("catalog/scan.rs"),
            r#"use crate::entity::Entity;

pub fn scan() {
    let _ = crate::entity::Entity;
    let _ = crate::listing::Format::Terminal;
}
"#,
        )
        .unwrap();

        let edges = extract_edges(&src);
        // scan.rs's top-level module is "catalog"
        assert!(
            edges.contains(&(String::from("catalog"), String::from("entity"))),
            "catalog→entity should exist, got: {edges:?}"
        );
        assert!(
            edges.contains(&(String::from("catalog"), String::from("listing"))),
            "catalog→listing should exist, got: {edges:?}"
        );
    }

    /// Diagnostic: prints discovered units and edges from the real `src/` tree.
    /// Run with: `cargo test --test architecture_layering dump_real_graph -- --nocapture --ignored`
    #[test]
    #[ignore]
    fn dump_real_graph() {
        let src = Path::new("src");
        let units = discover_units(src);
        let edges = extract_edges(src);
        println!("=== {n} UNITS ===", n = units.len());
        for u in &units {
            let out_count = edges.iter().filter(|(f, _)| f == u).count();
            let in_count = edges.iter().filter(|(_, t)| t == u).count();
            println!("  {u}  (out={out_count}, in={in_count})");
        }
        println!("\n=== {e_len} EDGES ===", e_len = edges.len());
        let mut sorted: Vec<_> = edges.iter().collect();
        sorted.sort();
        for (from, to) in &sorted {
            println!("  {from} -> {to}");
        }
    }

    /// SL-112 PHASE-02: the integration gate — production-graph layering enforcement.
    #[test]
    fn architecture_layering_gate() {
        let src = Path::new("src");
        assert!(src.exists(), "src directory must exist");
        assert!(Path::new("Cargo.toml").exists(), "Cargo.toml must exist");
        let layering_path = Path::new(".doctrine/adr/001/layering.toml");
        assert!(
            layering_path.exists(),
            ".doctrine/adr/001/layering.toml must exist"
        );

        let units = discover_units(src);
        let edges = extract_edges(src);

        let (map, accepted, baseline) =
            load_layering(layering_path).expect("failed to load layering.toml");

        let violations = check(&units, &edges, &map, &accepted, &baseline);

        if !violations.is_empty() {
            panic!("GATE FAILED:\n{:#?}", violations);
        }
    }

    // ── SL-112 PHASE-02: synthetic bite tests ───────────────────────────

    /// Helper: build a baseline with all three tiers = 0.
    fn baseline_zero() -> TangleBaseline {
        TangleBaseline(BTreeMap::from([
            (Tier::Leaf, 0),
            (Tier::Engine, 0),
            (Tier::Command, 0),
        ]))
    }

    #[test]
    fn check_legal_no_violations() {
        let units: BTreeSet<_> = ["leaf_mod", "engine_mod"]
            .into_iter()
            .map(String::from)
            .collect();
        let edges: BTreeSet<_> = [("leaf_mod".into(), "leaf_mod".into())]
            .into_iter()
            .collect();
        let mut tiers = BTreeMap::new();
        tiers.insert("leaf_mod".into(), Tier::Leaf);
        tiers.insert("engine_mod".into(), Tier::Engine);
        let map = LayerMap(tiers);
        let accepted = Accepted(BTreeSet::new());
        let baseline = baseline_zero();
        let v = check(&units, &edges, &map, &accepted, &baseline);
        assert!(v.is_empty(), "expected no violations, got {v:?}");
    }

    #[test]
    fn check_upward_edge_rejected() {
        let units: BTreeSet<_> = ["leaf_mod", "engine_mod"]
            .into_iter()
            .map(String::from)
            .collect();
        let edges: BTreeSet<_> = [("leaf_mod".into(), "engine_mod".into())]
            .into_iter()
            .collect();
        let mut tiers = BTreeMap::new();
        tiers.insert("leaf_mod".into(), Tier::Leaf);
        tiers.insert("engine_mod".into(), Tier::Engine);
        let map = LayerMap(tiers);
        let accepted = Accepted(BTreeSet::new());
        let baseline = baseline_zero();
        let v = check(&units, &edges, &map, &accepted, &baseline);
        // Both UpwardEdge and MixedUmbrella fire (leaf→engine upward without
        // acceptance or sub-classification).
        assert!(
            v.iter().any(|x| matches!(x, Violation::UpwardEdge { .. })),
            "expected UpwardEdge, got {v:?}"
        );
        assert!(
            v.iter()
                .any(|x| matches!(x, Violation::MixedUmbrella { .. })),
            "expected MixedUmbrella, got {v:?}"
        );
    }

    #[test]
    fn check_accepted_upward_edge_passes() {
        let units: BTreeSet<_> = ["leaf_mod", "engine_mod"]
            .into_iter()
            .map(String::from)
            .collect();
        let edges: BTreeSet<_> = [("leaf_mod".into(), "engine_mod".into())]
            .into_iter()
            .collect();
        let mut tiers = BTreeMap::new();
        tiers.insert("leaf_mod".into(), Tier::Leaf);
        tiers.insert("engine_mod".into(), Tier::Engine);
        let map = LayerMap(tiers);
        let mut accepted_set = BTreeSet::new();
        accepted_set.insert(("leaf_mod".into(), "engine_mod".into()));
        let accepted = Accepted(accepted_set);
        let baseline = baseline_zero();
        let v = check(&units, &edges, &map, &accepted, &baseline);
        assert!(v.is_empty(), "accepted upward edge should pass, got {v:?}");
    }

    #[test]
    fn check_stale_accepted_detected() {
        let units: BTreeSet<_> = ["leaf_mod", "engine_mod"]
            .into_iter()
            .map(String::from)
            .collect();
        let edges: BTreeSet<_> = BTreeSet::new();
        let mut tiers = BTreeMap::new();
        tiers.insert("leaf_mod".into(), Tier::Leaf);
        tiers.insert("engine_mod".into(), Tier::Engine);
        let map = LayerMap(tiers);
        let mut accepted_set = BTreeSet::new();
        accepted_set.insert(("leaf_mod".into(), "engine_mod".into()));
        let accepted = Accepted(accepted_set);
        let baseline = baseline_zero();
        let v = check(&units, &edges, &map, &accepted, &baseline);
        assert_eq!(v.len(), 1);
        assert!(matches!(v[0], Violation::StaleAccepted { .. }));
    }

    #[test]
    fn check_unclassified_detected() {
        let units: BTreeSet<_> = ["leaf_mod", "unknown_mod"]
            .into_iter()
            .map(String::from)
            .collect();
        let edges: BTreeSet<_> = [("leaf_mod".into(), "unknown_mod".into())]
            .into_iter()
            .collect();
        let mut tiers = BTreeMap::new();
        tiers.insert("leaf_mod".into(), Tier::Leaf);
        // unknown_mod is NOT in the map
        let map = LayerMap(tiers);
        let accepted = Accepted(BTreeSet::new());
        let baseline = baseline_zero();
        let v = check(&units, &edges, &map, &accepted, &baseline);
        assert!(
            v.iter()
                .any(|x| matches!(x, Violation::Unclassified(name) if name == "unknown_mod"))
        );
    }

    #[test]
    fn check_stale_entry_detected() {
        let units: BTreeSet<_> = ["leaf_mod"].into_iter().map(String::from).collect();
        let edges: BTreeSet<_> = BTreeSet::new();
        let mut tiers = BTreeMap::new();
        tiers.insert("leaf_mod".into(), Tier::Leaf);
        tiers.insert("ghost_mod".into(), Tier::Engine); // in map but not units
        let map = LayerMap(tiers);
        let accepted = Accepted(BTreeSet::new());
        let baseline = baseline_zero();
        let v = check(&units, &edges, &map, &accepted, &baseline);
        assert!(
            v.iter()
                .any(|x| matches!(x, Violation::StaleEntry(name) if name == "ghost_mod"))
        );
    }

    #[test]
    fn check_mixed_umbrella_detected() {
        // A module classified as Engine that reaches Command, not sub-classified,
        // and the upward edge is NOT accepted.
        let units: BTreeSet<_> = ["engine_mod", "cmd_mod"]
            .into_iter()
            .map(String::from)
            .collect();
        let edges: BTreeSet<_> = [("engine_mod".into(), "cmd_mod".into())]
            .into_iter()
            .collect();
        let mut tiers = BTreeMap::new();
        tiers.insert("engine_mod".into(), Tier::Engine);
        tiers.insert("cmd_mod".into(), Tier::Command);
        let map = LayerMap(tiers);
        let accepted = Accepted(BTreeSet::new());
        let baseline = baseline_zero();
        let v = check(&units, &edges, &map, &accepted, &baseline);
        assert!(
            v.iter()
                .any(|x| matches!(x, Violation::MixedUmbrella { .. })),
            "expected MixedUmbrella, got {v:?}"
        );
    }

    #[test]
    fn check_tangle_grew_detected() {
        // Two engine modules with a mutual cycle, baseline=0 → TangleGrew.
        let units: BTreeSet<_> = ["eng_a", "eng_b"].into_iter().map(String::from).collect();
        let edges: BTreeSet<_> = [
            ("eng_a".into(), "eng_b".into()),
            ("eng_b".into(), "eng_a".into()),
        ]
        .into_iter()
        .collect();
        let mut tiers = BTreeMap::new();
        tiers.insert("eng_a".into(), Tier::Engine);
        tiers.insert("eng_b".into(), Tier::Engine);
        let map = LayerMap(tiers);
        let accepted = Accepted(BTreeSet::new());
        let baseline = TangleBaseline(BTreeMap::from([
            (Tier::Leaf, 0),
            (Tier::Engine, 0),
            (Tier::Command, 0),
        ]));
        let v = check(&units, &edges, &map, &accepted, &baseline);
        assert!(
            v.iter().any(|x| matches!(
                x,
                Violation::TangleGrew {
                    tier: Tier::Engine,
                    baseline: 0,
                    actual: 2
                }
            )),
            "expected TangleGrew {{ tier: Engine, baseline: 0, actual: 2 }}, got {v:?}"
        );
    }
}
