// SPDX-License-Identifier: GPL-3.0-only
//! Publication declaration + fail-closed admission (SL-223, ADR-019 / PRD-017 /
//! SPEC-026 — D1/D6/D9/D-E).
//!
//! The publication manifest is the sole authority for what framework-owned
//! material is public, how it is addressed, and under what licence (REQ-373 /
//! REQ-359). This module owns the declaration schema, the source-agnostic
//! admission rules, the logical-address namespace, and the closed licence /
//! provenance / customization vocabularies (STD-001). It is engine-tier
//! (ADR-001): pure-first — the only IO is the leaf `asset_source` seam behind
//! [`PublicationManifest::load`]. The storage-independent resolver + framing-free
//! emit arrive in PHASE-03; this phase is admission only.
//!
//! **Fail-closed** (D6): an entry whose licence is outside the allowed set, or
//! any missing/unknown field, or a duplicate logical address, fails admission —
//! nothing is silently defaulted.

use serde::Deserialize;
use std::collections::BTreeSet;

/// Closed licence set (D6, STD-001). A value outside the set fails admission —
/// there is no runtime default and no guessed licence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Licence {
    Mit,
    Gpl,
}

impl Licence {
    const MIT: &'static str = "MIT";
    const GPL: &'static str = "GPL";

    /// Parse a declared licence string into the closed set (constructs every
    /// variant — the manifest vocabulary's single source, no scattered literals).
    fn parse(s: &str) -> Option<Self> {
        match s {
            Self::MIT => Some(Self::Mit),
            Self::GPL => Some(Self::Gpl),
            _ => None,
        }
    }

    /// The canonical declared spelling (reads every variant).
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Mit => Self::MIT,
            Self::Gpl => Self::GPL,
        }
    }
}

/// How an entry's licence was established (recorded per entry for auditability,
/// D-E / codex F7). Closed set (STD-001).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LicenceProvenance {
    Declared,
    RuleSuggestedConfirmed,
}

impl LicenceProvenance {
    const DECLARED: &'static str = "declared";
    const RULE_SUGGESTED_CONFIRMED: &'static str = "rule-suggested-confirmed";

    fn parse(s: &str) -> Option<Self> {
        match s {
            Self::DECLARED => Some(Self::Declared),
            Self::RULE_SUGGESTED_CONFIRMED => Some(Self::RuleSuggestedConfirmed),
            _ => None,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Declared => Self::DECLARED,
            Self::RuleSuggestedConfirmed => Self::RULE_SUGGESTED_CONFIRMED,
        }
    }
}

/// Whether a published asset is intended to be customized downstream (OQ-5
/// provisional, [[ASM-003]]). Closed set (STD-001); its stable vocabulary is C2's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CustomizationStatus {
    Customizable,
    Fixed,
}

impl CustomizationStatus {
    const CUSTOMIZABLE: &'static str = "customizable";
    const FIXED: &'static str = "fixed";

    fn parse(s: &str) -> Option<Self> {
        match s {
            Self::CUSTOMIZABLE => Some(Self::Customizable),
            Self::FIXED => Some(Self::Fixed),
            _ => None,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Customizable => Self::CUSTOMIZABLE,
            Self::Fixed => Self::FIXED,
        }
    }
}

/// The kind of published content. Templates-only this slice (D-D); the enum is
/// the closed vocabulary (STD-001) and widens additively when a later slice
/// declares non-template collections (OQ-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContentKind {
    Template,
}

impl ContentKind {
    const TEMPLATE: &'static str = "template";

    fn parse(s: &str) -> Option<Self> {
        match s {
            Self::TEMPLATE => Some(Self::Template),
            _ => None,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Template => Self::TEMPLATE,
        }
    }
}

/// A stable POSIX-like logical path — the durable addressing contract (REQ-374),
/// decoupled from the physical `backing` key. Traversal-safe *by type*: absolute,
/// empty, `.`/`..`, and backslash-bearing paths are rejected at construction, so
/// no resolve/emit path can be handed an unsafe address.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LogicalAddress(String);

impl LogicalAddress {
    /// Construct a validated logical address, or [`AdmissionError::TraversalRejected`]
    /// if it is absolute, empty, backslash-bearing, or contains a `.`/`..`/empty
    /// segment.
    pub(crate) fn parse(raw: &str) -> Result<Self, AdmissionError> {
        let rejected = || AdmissionError::TraversalRejected(raw.to_string());
        if raw.is_empty() || raw.starts_with('/') || raw.contains('\\') {
            return Err(rejected());
        }
        for segment in raw.split('/') {
            if segment.is_empty() || segment == "." || segment == ".." {
                return Err(rejected());
            }
        }
        Ok(Self(raw.to_string()))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// One declared publication entry. `address` is the stable contract; `backing`
/// is the physical embed key (they may diverge — storage independence, D-F).
/// Every field is read on a production path (admission validates it and/or the
/// `publication validate` report prints it via [`PublicationEntry::report_line`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PublicationEntry {
    address: LogicalAddress,
    backing: String,
    kind: ContentKind,
    title: String,
    licence: Licence,
    provenance: LicenceProvenance,
    customization: CustomizationStatus,
}

impl PublicationEntry {
    /// A single human-readable validation line that reads EVERY field — the
    /// production reader that keeps each parsed field live under `deny(unused)`
    /// (dead-field discipline, F-4 corollary) and gives the report per-entry
    /// identity: address, backing, kind, title, licence, provenance, customization.
    pub(crate) fn report_line(&self) -> String {
        format!(
            "{addr} -> {backing} [{kind}] \"{title}\" licence={licence} provenance={provenance} customization={customization}",
            addr = self.address.as_str(),
            backing = self.backing,
            kind = self.kind.as_str(),
            title = self.title,
            licence = self.licence.as_str(),
            provenance = self.provenance.as_str(),
            customization = self.customization.as_str(),
        )
    }
}

/// The admitted manifest — the validated public set. Held immutable; the
/// resolver (P03) borrows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PublicationManifest {
    entries: Vec<PublicationEntry>,
}

/// Why admission failed — one variant per reason so callers (and the fail-closed
/// tests) assert the REASON, not merely `is_err()`. Nothing is defaulted.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum AdmissionError {
    /// Bytes were not UTF-8, or not well-formed TOML for the manifest schema.
    #[error("malformed publication manifest: {0}")]
    MalformedManifest(String),
    /// A required field was absent, empty, or carried an unknown enum value
    /// (unknown provenance/customization/kind fail closed here, per §5.5).
    #[error("entry {index}: missing, empty, or unknown field '{field}'")]
    MissingField { index: usize, field: &'static str },
    /// The declared licence is outside the allowed set {MIT, GPL} (D6).
    #[error("entry {index}: licence '{value}' is missing or outside the allowed set {{MIT, GPL}}")]
    LicenceMissingOrOutOfSet { index: usize, value: String },
    /// Two entries declare the same logical address (D9) — rejected at admission,
    /// never resolved by precedence.
    #[error("duplicate logical address '{0}'")]
    DuplicateAddress(String),
    /// A logical address is absolute, empty, or traversal-bearing.
    #[error("address '{0}' is not a safe relative logical path")]
    TraversalRejected(String),
    /// The shipped publication manifest is absent from the embed (a hollow build).
    #[error("publication manifest asset is missing from the embed")]
    ManifestAssetMissing,
}

/// Raw TOML shape — every field optional so an absent key becomes a precise
/// [`AdmissionError::MissingField`] rather than a generic parse error, and
/// unknown keys are rejected (`deny_unknown_fields`). Validation into the typed
/// [`PublicationEntry`] happens in [`PublicationManifest::admit`].
#[derive(Deserialize)]
struct RawManifest {
    #[serde(default)]
    entry: Vec<RawEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEntry {
    address: Option<String>,
    backing: Option<String>,
    kind: Option<String>,
    title: Option<String>,
    licence: Option<String>,
    provenance: Option<String>,
    customization: Option<String>,
}

/// Extract a required, non-empty string field, else [`AdmissionError::MissingField`].
fn require<'a>(
    value: Option<&'a String>,
    index: usize,
    field: &'static str,
) -> Result<&'a str, AdmissionError> {
    match value {
        Some(s) if !s.trim().is_empty() => Ok(s.as_str()),
        _ => Err(AdmissionError::MissingField { index, field }),
    }
}

impl PublicationManifest {
    /// Source-AGNOSTIC admission — the sole-authority contract (REQ-373) is a
    /// property of these bytes, not of where they came from. Parses TOML,
    /// validates every field of every entry (all present, `title`/`backing`
    /// non-empty, `licence ∈ {MIT, GPL}`, known provenance/customization/kind,
    /// well-formed [`LogicalAddress`]), and rejects duplicate logical addresses
    /// (D9). Fail-closed: any failure returns a typed [`AdmissionError`].
    pub(crate) fn admit(bytes: &[u8]) -> Result<Self, AdmissionError> {
        let text = std::str::from_utf8(bytes)
            .map_err(|e| AdmissionError::MalformedManifest(e.to_string()))?;
        let raw: RawManifest =
            toml::from_str(text).map_err(|e| AdmissionError::MalformedManifest(e.to_string()))?;

        let mut entries = Vec::with_capacity(raw.entry.len());
        let mut seen: BTreeSet<String> = BTreeSet::new();
        for (index, raw_entry) in raw.entry.iter().enumerate() {
            let entry = admit_entry(raw_entry, index)?;
            // Duplicate logical addresses are rejected at admission (D9), not
            // resolved by precedence — the transient `seen` set is the dup gate;
            // the stored address→entry lookup index lands in P03 with its consumer.
            if !seen.insert(entry.address.as_str().to_string()) {
                return Err(AdmissionError::DuplicateAddress(
                    entry.address.as_str().to_string(),
                ));
            }
            entries.push(entry);
        }
        Ok(Self { entries })
    }

    /// Embedded convenience: admit the shipped `publication/manifest.toml`. The
    /// authoritative admission gate reads the manifest from disk source (VT-3);
    /// this proves the *embedded* copy admits too (VT-4).
    pub(crate) fn load() -> Result<Self, AdmissionError> {
        let bytes = crate::asset_source::publication_manifest_bytes()
            .ok_or(AdmissionError::ManifestAssetMissing)?;
        Self::admit(&bytes)
    }

    /// The admitted public set, in declaration order.
    pub(crate) fn entries(&self) -> &[PublicationEntry] {
        &self.entries
    }
}

/// Validate one raw entry into a typed [`PublicationEntry`], reading every field.
fn admit_entry(raw: &RawEntry, index: usize) -> Result<PublicationEntry, AdmissionError> {
    let address = LogicalAddress::parse(require(raw.address.as_ref(), index, "address")?)?;
    let backing = require(raw.backing.as_ref(), index, "backing")?.to_string();
    let kind = ContentKind::parse(require(raw.kind.as_ref(), index, "kind")?).ok_or(
        AdmissionError::MissingField {
            index,
            field: "kind",
        },
    )?;
    let title = require(raw.title.as_ref(), index, "title")?.to_string();
    let licence_raw = require(raw.licence.as_ref(), index, "licence")?;
    let licence =
        Licence::parse(licence_raw).ok_or_else(|| AdmissionError::LicenceMissingOrOutOfSet {
            index,
            value: licence_raw.to_string(),
        })?;
    let provenance =
        LicenceProvenance::parse(require(raw.provenance.as_ref(), index, "provenance")?).ok_or(
            AdmissionError::MissingField {
                index,
                field: "provenance",
            },
        )?;
    let customization =
        CustomizationStatus::parse(require(raw.customization.as_ref(), index, "customization")?)
            .ok_or(AdmissionError::MissingField {
                index,
                field: "customization",
            })?;
    Ok(PublicationEntry {
        address,
        backing,
        kind,
        title,
        licence,
        provenance,
        customization,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A well-formed single-entry manifest body; callers mutate one line to
    /// drive a specific fail-closed case.
    fn valid_entry(licence: &str, provenance: &str, customization: &str) -> String {
        format!(
            "[[entry]]\n\
             address = \"templates/slice.toml\"\n\
             backing = \"templates/slice.toml\"\n\
             kind = \"template\"\n\
             title = \"Slice metadata template\"\n\
             licence = \"{licence}\"\n\
             provenance = \"{provenance}\"\n\
             customization = \"{customization}\"\n"
        )
    }

    // VT-1: fail-closed admission — out-of-set licence, missing field, malformed
    // TOML, unknown provenance each -> a typed AdmissionError; nothing defaulted.
    // MIT and GPL both admit (the whole allowed set is live).
    #[test]
    fn mit_and_gpl_both_admit() {
        for licence in ["MIT", "GPL"] {
            let body = valid_entry(licence, "declared", "customizable");
            PublicationManifest::admit(body.as_bytes())
                .unwrap_or_else(|e| panic!("{licence} entry should admit: {e}"));
        }
    }

    #[test]
    fn out_of_set_licence_fails_closed() {
        let body = valid_entry("BSD", "declared", "customizable");
        let err = PublicationManifest::admit(body.as_bytes()).expect_err("BSD is out of set");
        assert_eq!(
            err,
            AdmissionError::LicenceMissingOrOutOfSet {
                index: 0,
                value: "BSD".to_string()
            }
        );
    }

    #[test]
    fn missing_field_fails_closed() {
        // Omit `title` entirely.
        let body = "[[entry]]\n\
             address = \"templates/slice.toml\"\n\
             backing = \"templates/slice.toml\"\n\
             kind = \"template\"\n\
             licence = \"MIT\"\n\
             provenance = \"declared\"\n\
             customization = \"customizable\"\n";
        let err = PublicationManifest::admit(body.as_bytes()).expect_err("missing title");
        assert_eq!(
            err,
            AdmissionError::MissingField {
                index: 0,
                field: "title"
            }
        );
    }

    #[test]
    fn malformed_toml_fails_closed() {
        let err = PublicationManifest::admit(b"this is [[[ not valid").expect_err("malformed");
        assert!(matches!(err, AdmissionError::MalformedManifest(_)));
    }

    #[test]
    fn unknown_provenance_fails_closed() {
        let body = valid_entry("MIT", "hearsay", "customizable");
        let err = PublicationManifest::admit(body.as_bytes()).expect_err("unknown provenance");
        assert_eq!(
            err,
            AdmissionError::MissingField {
                index: 0,
                field: "provenance"
            }
        );
    }

    #[test]
    fn unknown_customization_fails_closed() {
        let body = valid_entry("MIT", "declared", "whatever");
        let err = PublicationManifest::admit(body.as_bytes()).expect_err("unknown customization");
        assert_eq!(
            err,
            AdmissionError::MissingField {
                index: 0,
                field: "customization"
            }
        );
    }

    // VT-2: duplicate logical address rejected at admission (D9), not precedence.
    #[test]
    fn duplicate_address_rejected_at_admission() {
        let one = valid_entry("MIT", "declared", "customizable");
        let body = format!("{one}{one}"); // same address twice
        let err = PublicationManifest::admit(body.as_bytes()).expect_err("dup address");
        assert_eq!(
            err,
            AdmissionError::DuplicateAddress("templates/slice.toml".to_string())
        );
    }

    #[test]
    fn traversal_address_rejected_at_construction() {
        for bad in ["/abs/path", "../escape", "a/../b", "", "a\\b", "./here"] {
            assert!(
                matches!(
                    LogicalAddress::parse(bad),
                    Err(AdmissionError::TraversalRejected(_))
                ),
                "{bad:?} must be rejected"
            );
        }
        assert!(LogicalAddress::parse("templates/slice.toml").is_ok());
    }

    // VT-3: source-level admission gate (staleness-proof) — admit over the
    // on-disk publication/manifest.toml passes; authoritative regardless of embed
    // staleness (the rust-embed footgun). The repo root is resolved at RUNTIME via
    // test_support::repo_root(), never compile-time env! (CHR-014 / SL-162).
    #[test]
    fn shipped_manifest_admits_from_disk_source() {
        let path = crate::test_support::repo_root().join("publication/manifest.toml");
        let bytes = std::fs::read(&path).expect("shipped publication/manifest.toml on disk");
        let manifest = PublicationManifest::admit(&bytes).expect("shipped manifest admits");
        assert!(
            !manifest.entries().is_empty(),
            "shipped manifest declares at least one entry"
        );
        // Every shipped entry is MIT / customizable / declared (templates-only, D-D).
        for entry in manifest.entries() {
            assert!(entry.report_line().contains("licence=MIT"));
        }
    }

    // VT-4: embedded admission — load() over the shipped embed admits (proves
    // publication_manifest_bytes is wired).
    #[test]
    fn load_admits_the_embedded_manifest() {
        let manifest = PublicationManifest::load().expect("embedded manifest admits");
        assert!(!manifest.entries().is_empty());
    }
}
