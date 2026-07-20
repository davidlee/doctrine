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

/// Read the shipped publication manifest's bytes (`publication/manifest.toml`),
/// `None` if the embed is hollow. The `publication` engine's `load()` admits
/// these; the disk-source admission gate (VT-3) reads the same file from
/// `CARGO_MANIFEST_DIR` so embed staleness cannot mask a defect.
pub(crate) fn publication_manifest_bytes() -> Option<std::borrow::Cow<'static, [u8]>> {
    PublicationAssets::get("manifest.toml").map(|f| f.data)
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
