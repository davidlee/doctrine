// SPDX-License-Identifier: GPL-3.0-only
#![allow(
    clippy::same_name_method,
    reason = "rust-embed derive generates conflicting method names"
)]

//! Neutral leaf owner of the embedded-asset byte-read seam (D3, REQ-376).
//!
//! Owned by neither projection (`install`) nor publication — both read bytes
//! *through* this module so publication is not born coupled to install's
//! projection path. This phase relocates the `install/` embed here and exposes
//! a binary-safe byte read plus the UTF-8 convenience install already relied on.
//! The `PublicationAssets` embed + `publication_manifest_bytes` arrive in
//! PHASE-02; the embed types stay private so they never leak across the seam.

use rust_embed::RustEmbed;

/// Embedded install assets — everything under `install/`. Private: the embed
/// type does not cross the seam; callers use the free functions below.
#[derive(RustEmbed)]
#[folder = "install/"]
struct InstallAssets;

/// Embedded publication assets — everything under `publication/` (the manifest).
/// A SEPARATE root from `InstallAssets` so install's projection (`build_plan`,
/// which walks `InstallAssets` only) never touches it: publication is not
/// projection (SL-223 D-C / REQ-380). Private — the embed type stays behind the seam.
#[derive(RustEmbed)]
#[folder = "publication/"]
struct PublicationAssets;

/// Read one `install/`-relative asset's bytes (`None` if absent). Binary-safe —
/// the byte-level seam the whole extraction exists to provide (REQ-376).
pub(crate) fn read_bytes(key: &str) -> Option<std::borrow::Cow<'static, [u8]>> {
    InstallAssets::get(key).map(|f| f.data)
}

/// Existence of an `install/`-relative asset WITHOUT materializing its bytes —
/// probes the embed's name enumeration, so no `Cow` payload is read (the
/// availability probe a library veneer needs, distinct from `read_bytes`).
pub(crate) fn exists(key: &str) -> bool {
    iter().any(|name| name == key)
}

/// Read one `install/`-relative asset as UTF-8 text. Error text preserved from
/// install's prior `asset_text` so its behaviour gate stays green unchanged.
pub(crate) fn read_text(key: &str) -> anyhow::Result<String> {
    use anyhow::Context;
    let file =
        InstallAssets::get(key).with_context(|| format!("Embedded asset '{key}' is missing"))?;
    let text = std::str::from_utf8(&file.data)
        .with_context(|| format!("Embedded asset '{key}' is not valid UTF-8"))?;
    Ok(text.to_string())
}

/// Iterate every embedded `install/`-relative asset name.
pub(crate) fn iter() -> impl Iterator<Item = std::borrow::Cow<'static, str>> {
    InstallAssets::iter()
}

/// Read the shipped publication manifest's bytes (`publication/manifest.toml`)
/// from the compiled embed, `None` if hollow. The `publication` engine's `load()`
/// admits these in production. The reachability gates instead read disk-source via
/// [`publication_manifest_bytes_from_disk`] so embed staleness cannot mask a defect.
pub(crate) fn publication_manifest_bytes() -> Option<std::borrow::Cow<'static, [u8]>> {
    PublicationAssets::get("manifest.toml").map(|f| f.data)
}

// ---------------------------------------------------------------------------
// SL-227 F-6/F-7: disk-source authority for the reachability gates. RustEmbed
// runs with `debug-embed` on, so `iter()`/`PublicationAssets::get()` read the
// COMPILED embed — an incremental build over a stale `install/` edit can
// false-green a gate. These helpers read the SAME assets from RUNTIME disk-source
// (`repo_root()`, per CHR-014) so the pairing invariant is proven staleness-proof.
// ---------------------------------------------------------------------------

/// Disk-source enumeration of install asset keys (relative to `install/`),
/// excluding the manifest itself — the disk sibling of [`iter`].
#[cfg(test)]
pub(crate) fn install_asset_keys_from_disk() -> Vec<String> {
    let root = crate::test_support::repo_root().join("install");
    let mut names: Vec<String> = walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| {
            e.path()
                .strip_prefix(&root)
                .ok()
                .map(|p| p.to_string_lossy().replace('\\', "/"))
        })
        .filter(|n| n != "manifest.toml")
        .collect();
    names.sort();
    names
}

/// Disk-source bytes of the shipped `publication/manifest.toml` — the disk sibling
/// of [`publication_manifest_bytes`], authoritative regardless of embed staleness.
#[cfg(test)]
pub(crate) fn publication_manifest_bytes_from_disk() -> Vec<u8> {
    let path = crate::test_support::repo_root().join("publication/manifest.toml");
    std::fs::read(&path).expect("shipped publication/manifest.toml on disk")
}

/// Disk-source projection base backings (`install/manifest.toml [base].backings`) —
/// the single authority both reachability gates derive the base set from, so no
/// gate hardcodes the list (STD-001, dissolves the F-6 duplicate literal).
#[cfg(test)]
pub(crate) fn base_backings_from_disk() -> Vec<String> {
    #[derive(serde::Deserialize)]
    struct BaseOnly {
        base: Base,
    }
    #[derive(serde::Deserialize)]
    struct Base {
        backings: Vec<String>,
    }
    let path = crate::test_support::repo_root().join("install/manifest.toml");
    let bytes = std::fs::read(&path).expect("shipped install/manifest.toml on disk");
    let text = std::str::from_utf8(&bytes).expect("install manifest is UTF-8");
    let parsed: BaseOnly = toml::from_str(text).expect("install manifest [base] parses");
    parsed.base.backings
}

/// The shared reachability invariant, read entirely from disk-source: every install
/// asset that is NOT a projection base backing MUST be a published backing
/// (`{install} − {base} ⊆ {published}`) — else the minimal-projection flip would
/// strand it. Both the crux gate (`install.rs`) and its sibling (`publication.rs`)
/// delegate here so the two attest the SAME staleness-proof check (SL-227 F-6/F-7).
#[cfg(test)]
pub(crate) fn assert_unprojected_install_assets_are_published() {
    let published =
        crate::publication::PublicationManifest::admit(&publication_manifest_bytes_from_disk())
            .expect("shipped publication manifest admits from disk");
    let base: std::collections::BTreeSet<String> = base_backings_from_disk().into_iter().collect();
    let mut checked = 0usize;
    for key in install_asset_keys_from_disk() {
        if base.contains(&key) {
            continue;
        }
        assert!(
            published.declares_backing(&key),
            "install asset {key:?} is neither a base backing nor published — the \
             minimal-projection flip would strand it (disk-source reachability gate)"
        );
        checked += 1;
    }
    assert!(
        checked > 0,
        "the disk enumerator must yield non-base install assets to check"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// VT-1: `read_bytes` returns exact bytes for a known text asset and
    /// `read_text` agrees on the same asset — proves the seam is
    /// `Cow<[u8]>`-clean (byte read is not routed through `from_utf8`).
    #[test]
    fn read_bytes_and_read_text_agree_on_text_asset() {
        let key = "manifest.toml";
        let bytes = read_bytes(key).expect("install/manifest.toml is embedded");
        assert!(!bytes.is_empty(), "manifest.toml should not be empty");
        let text = read_text(key).expect("install/manifest.toml is valid UTF-8");
        assert_eq!(
            text.as_bytes(),
            bytes.as_ref(),
            "read_text must reproduce read_bytes byte-for-byte on a UTF-8 asset"
        );
    }

    /// PHASE-01 (SL-227): `exists` probes availability by name without
    /// materializing bytes — true for an embedded asset, false for an absent key.
    #[test]
    fn exists_true_for_embedded_asset_false_for_absent() {
        assert!(exists("manifest.toml"), "embedded install asset exists");
        assert!(
            !exists("no/such/asset.xyz"),
            "absent asset key does not exist"
        );
    }

    /// The `InstallAssets` embed is reachable through `iter()` and includes the
    /// root manifest — the enumeration `embedded_filenames`/hymn discovery rides.
    #[test]
    fn iter_enumerates_install_assets() {
        assert!(
            iter().any(|name| name == "manifest.toml"),
            "iter() should enumerate the InstallAssets embed"
        );
    }
}
