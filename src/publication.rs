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
//! [`PublicationManifest::load`] and the [`SourceAdapter`] read boundary. It also
//! owns the storage-independent [`Resolver`] over the [`SourceAdapter`] interface
//! and the framing-free [`Resolver::emit`] (REQ-374 / REQ-363).
//!
//! **Fail-closed** (D6): an entry whose licence is outside the allowed set, or
//! any missing/unknown field, or a duplicate logical address, fails admission —
//! nothing is silently defaulted.

use serde::Deserialize;
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::io::Write;

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

/// The kind of published content — the closed vocabulary (STD-001), widened
/// additively (SL-227) to the full complement a library veneer renders. Existing
/// entries declare `template`; the new variants deserialize their lowercase
/// spellings (`reference`/`guidance`/`integration`) via [`ContentKind::parse`],
/// preserving the prior rename semantics. Adding variants is additive: no
/// existing entry's `kind` changes meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContentKind {
    Template,
    Reference,
    Guidance,
    Integration,
}

impl ContentKind {
    const TEMPLATE: &'static str = "template";
    const REFERENCE: &'static str = "reference";
    const GUIDANCE: &'static str = "guidance";
    const INTEGRATION: &'static str = "integration";

    fn parse(s: &str) -> Option<Self> {
        match s {
            Self::TEMPLATE => Some(Self::Template),
            Self::REFERENCE => Some(Self::Reference),
            Self::GUIDANCE => Some(Self::Guidance),
            Self::INTEGRATION => Some(Self::Integration),
            _ => None,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Template => Self::TEMPLATE,
            Self::Reference => Self::REFERENCE,
            Self::Guidance => Self::GUIDANCE,
            Self::Integration => Self::INTEGRATION,
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
    /// The stable logical address — the resolver's lookup key (read on the
    /// production path by `run_publication_validate`, which emits each entry).
    pub(crate) fn address(&self) -> &LogicalAddress {
        &self.address
    }

    /// The declared human-readable title (SL-227) — the structured accessor a
    /// library veneer renders in a list/tree, distinct from `report_line`'s
    /// single-string form. `backing` stays private; only the fields a veneer
    /// legitimately shows are surfaced.
    pub(crate) fn title(&self) -> &str {
        &self.title
    }

    /// The declared content kind (SL-227) — returned by value (`Copy`).
    pub(crate) fn kind(&self) -> ContentKind {
        self.kind
    }

    /// The declared licence (SL-227) — returned by value (`Copy`).
    pub(crate) fn licence(&self) -> Licence {
        self.licence
    }

    /// The declared customization status (SL-227) — returned by value (`Copy`).
    pub(crate) fn customization(&self) -> CustomizationStatus {
        self.customization
    }

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

/// The admitted manifest — the validated public set. Held immutable; a
/// [`Resolver`] binds it to one adapter.
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

    /// Whether any admitted entry declares `backing` as its physical embed key
    /// (SL-227) — a membership predicate the PHASE-03 derived reachability gate
    /// checks so it can ask "is this embed key published?" without `backing`
    /// leaving the module (the field stays private).
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "consumed by the PHASE-03 reachability gate (SL-227)"
        )
    )]
    pub(crate) fn declares_backing(&self, backing: &str) -> bool {
        self.entries.iter().any(|e| e.backing == backing)
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

/// Why resolution failed — one variant per reason so callers assert the REASON,
/// never a silent empty result. Constructed only on production paths (each
/// variant satisfies `deny(unused)` by its construction site, not by runtime
/// reachability).
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum ResolveError {
    /// No admitted entry declares this logical address.
    #[error("no published entry for logical address '{0}'")]
    UnknownAddress(String),
    /// The entry's backing key has no bytes behind it (a hollow embed, or a
    /// relocated key the bound adapter cannot serve).
    #[error("backing source '{0}' is missing")]
    BackingSourceMissing(String),
}

/// Why emit failed — a resolution failure or a writer IO failure. Wraps
/// [`ResolveError`] (emit resolves before it writes) and [`std::io::Error`];
/// the latter's lack of `PartialEq` is why this type is `matches!`-asserted.
#[derive(Debug, thiserror::Error)]
pub(crate) enum EmitError {
    #[error(transparent)]
    Resolve(#[from] ResolveError),
    #[error("emit write failed: {0}")]
    Io(#[from] std::io::Error),
}

/// Read-only by construction: the ONLY method is `read`; no write capability is
/// reachable from the resolver (REQ-381 seam foundation, §5.3). A source
/// reorganisation moves bytes behind `read`, never the logical addresses above.
pub(crate) trait SourceAdapter {
    fn read(&self, backing: &str) -> Result<Cow<'static, [u8]>, ResolveError>;

    /// Whether the adapter can serve bytes for `backing`, WITHOUT reading them
    /// (SL-227) — the availability probe a library veneer needs to report an
    /// entry's status without materializing its payload.
    fn exists(&self, backing: &str) -> bool;
}

/// The one production adapter: reads immutable compiled bytes by embed key via
/// the leaf `asset_source` seam. Holds no filesystem path and no write handle,
/// so no mutation is reachable through it (structural read-only).
pub(crate) struct EmbeddedAdapter;

impl SourceAdapter for EmbeddedAdapter {
    fn read(&self, backing: &str) -> Result<Cow<'static, [u8]>, ResolveError> {
        crate::asset_source::read_bytes(backing)
            .ok_or_else(|| ResolveError::BackingSourceMissing(backing.to_string()))
    }

    fn exists(&self, backing: &str) -> bool {
        crate::asset_source::exists(backing)
    }
}

/// An admitted manifest bound to exactly one SUBSTITUTABLE adapter — generic over
/// `A`, not a fixed concrete field (RV-287 F-1), so the storage-independence test
/// wires an in-memory adapter through the SAME `new`/`resolve`/`emit` API that
/// production wires `EmbeddedAdapter` into. NOT a multi-source registry: exactly
/// one adapter, chosen at construction (source-identity dispatch is deferred, OQ-4).
pub(crate) struct Resolver<A: SourceAdapter> {
    manifest: PublicationManifest,
    source: A,
}

impl<A: SourceAdapter> Resolver<A> {
    /// Bind an admitted manifest to one adapter.
    pub(crate) fn new(manifest: PublicationManifest, source: A) -> Self {
        Self { manifest, source }
    }

    /// The admitted public set this resolver is bound to — the command iterates
    /// it to emit every declared entry.
    pub(crate) fn manifest(&self) -> &PublicationManifest {
        &self.manifest
    }

    /// Whether the entry's backing is currently servable by the bound adapter
    /// (SL-227) — an availability probe that reads no bytes, so a library veneer
    /// can flag an admitted-but-unbacked entry. Reaches `backing` within-module
    /// (the field stays private to the engine).
    pub(crate) fn available(&self, e: &PublicationEntry) -> bool {
        self.source.exists(&e.backing)
    }

    /// Resolve a logical address to its backing bytes through the bound adapter.
    /// A lookup miss → [`ResolveError::UnknownAddress`] (never a silent empty
    /// result); absent backing → [`ResolveError::BackingSourceMissing`] from the
    /// adapter. Lookup is a linear scan: admission rejects duplicate addresses
    /// (D9), so the match is unique — no stored index is warranted (D-P03-a).
    pub(crate) fn resolve(
        &self,
        addr: &LogicalAddress,
    ) -> Result<Cow<'static, [u8]>, ResolveError> {
        let entry = self
            .manifest
            .entries()
            .iter()
            .find(|e| e.address == *addr)
            .ok_or_else(|| ResolveError::UnknownAddress(addr.as_str().to_string()))?;
        self.source.read(&entry.backing)
    }

    /// Stream the resolved asset's bytes **framing-free** to any writer — no
    /// length prefix, no separator — propagating a resolution failure or a writer
    /// IO error (D-A / RV-286 F6). The real consumer shape a later `library show`
    /// wraps; `publication validate` calls it per entry into `io::sink()`.
    pub(crate) fn emit(&self, addr: &LogicalAddress, out: &mut dyn Write) -> Result<(), EmitError> {
        let bytes = self.resolve(addr)?;
        out.write_all(&bytes)?;
        Ok(())
    }
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

    // ── PHASE-03: resolver + emit ────────────────────────────────────────────

    use std::collections::BTreeMap;

    /// An in-memory adapter keyed by backing — the SUBSTITUTABLE second adapter
    /// that proves `Resolver<A>` is storage-independent (address decoupled from
    /// physical layout). Owned bytes → `Cow::Owned` (`'static`), so it satisfies
    /// the same `Cow<'static,[u8]>` contract as the embedded adapter with no
    /// `from_utf8` in the path (binary-clean).
    struct MapAdapter {
        bytes: BTreeMap<String, Vec<u8>>,
    }

    impl SourceAdapter for MapAdapter {
        fn read(&self, backing: &str) -> Result<Cow<'static, [u8]>, ResolveError> {
            self.bytes
                .get(backing)
                .map(|v| Cow::Owned(v.clone()))
                .ok_or_else(|| ResolveError::BackingSourceMissing(backing.to_string()))
        }

        fn exists(&self, backing: &str) -> bool {
            self.bytes.contains_key(backing)
        }
    }

    /// A writer that always fails — drives emit's output-failure path (VT-2).
    struct FailingWriter;

    impl std::io::Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("writer boom"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// Admit a one-entry manifest with the given logical address + backing key
    /// (MIT/template/declared/customizable), for adapter-substitution tests.
    fn one_entry(address: &str, backing: &str) -> PublicationManifest {
        let body = format!(
            "[[entry]]\n\
             address = \"{address}\"\n\
             backing = \"{backing}\"\n\
             kind = \"template\"\n\
             title = \"Fixture\"\n\
             licence = \"MIT\"\n\
             provenance = \"declared\"\n\
             customization = \"customizable\"\n"
        );
        PublicationManifest::admit(body.as_bytes()).expect("fixture admits")
    }

    fn shipped_resolver() -> Resolver<EmbeddedAdapter> {
        Resolver::new(
            PublicationManifest::load().expect("shipped manifest loads"),
            EmbeddedAdapter,
        )
    }

    // VT-1: declared address resolves to EXACT bytes through EmbeddedAdapter;
    // undeclared -> UnknownAddress (not a silent empty).
    #[test]
    fn declared_address_resolves_to_exact_bytes() {
        let resolver = shipped_resolver();
        let addr = LogicalAddress::parse("templates/slice.toml").expect("addr");
        let got = resolver.resolve(&addr).expect("declared address resolves");
        let expected =
            crate::asset_source::read_bytes("templates/slice.toml").expect("backing embed present");
        assert_eq!(got, expected, "resolve returns the backing bytes verbatim");
    }

    #[test]
    fn undeclared_address_is_unknown_not_empty() {
        let resolver = shipped_resolver();
        let addr = LogicalAddress::parse("templates/not-published.toml").expect("addr");
        assert_eq!(
            resolver.resolve(&addr).expect_err("undeclared address"),
            ResolveError::UnknownAddress("templates/not-published.toml".to_string())
        );
    }

    // VT-1 (adapter miss): a declared entry whose backing key is absent from the
    // embed -> BackingSourceMissing (the production EmbeddedAdapter miss path).
    #[test]
    fn absent_backing_is_backing_source_missing() {
        let manifest = one_entry("templates/ghost.toml", "templates/does-not-exist.toml");
        let resolver = Resolver::new(manifest, EmbeddedAdapter);
        let addr = LogicalAddress::parse("templates/ghost.toml").expect("addr");
        assert_eq!(
            resolver.resolve(&addr).expect_err("absent backing"),
            ResolveError::BackingSourceMissing("templates/does-not-exist.toml".to_string())
        );
    }

    // VT-2: emit streams framing-free bytes byte-for-byte; a failing writer
    // surfaces EmitError::Io.
    #[test]
    fn emit_streams_framing_free_bytes() {
        let resolver = shipped_resolver();
        let addr = LogicalAddress::parse("templates/slice.toml").expect("addr");
        let mut buf: Vec<u8> = Vec::new();
        resolver.emit(&addr, &mut buf).expect("emits");
        let expected =
            crate::asset_source::read_bytes("templates/slice.toml").expect("backing embed present");
        // Byte-for-byte, no framing: emitted == resolved == backing bytes.
        assert_eq!(buf.as_slice(), expected.as_ref());
    }

    #[test]
    fn emit_surfaces_writer_io_failure() {
        let resolver = shipped_resolver();
        let addr = LogicalAddress::parse("templates/slice.toml").expect("addr");
        let err = resolver
            .emit(&addr, &mut FailingWriter)
            .expect_err("writer failure surfaces");
        assert!(matches!(err, EmitError::Io(_)));
    }

    // VT-3: storage independence — same LogicalAddress, backing RELOCATED to a key
    // only a second in-memory adapter owns, resolves identically through the SAME
    // Resolver<A> API (proves the generic is substitutable, D2/REQ-374).
    #[test]
    fn storage_independent_resolve_through_relocated_backing() {
        let manifest = one_entry("templates/slice.toml", "relocated/elsewhere.bin");
        let mut bytes = BTreeMap::new();
        bytes.insert("relocated/elsewhere.bin".to_string(), b"payload".to_vec());
        let resolver = Resolver::new(manifest, MapAdapter { bytes });
        let addr = LogicalAddress::parse("templates/slice.toml").expect("addr");
        assert_eq!(
            resolver
                .resolve(&addr)
                .expect("resolves via relocated backing"),
            Cow::Borrowed(b"payload".as_slice())
        );
    }

    // VT-4: binary round-trip — non-UTF-8 bytes emit byte-for-byte (no from_utf8
    // in resolve/emit), without shipping a binary asset into any embed (REQ-376).
    #[test]
    fn emit_binary_bytes_round_trip() {
        let raw = vec![0xff_u8, 0x00, 0xfe, 0x80, 0x01];
        assert!(
            std::str::from_utf8(&raw).is_err(),
            "fixture must be genuinely non-UTF-8"
        );
        let manifest = one_entry("blobs/raw.bin", "blobs/raw.bin");
        let mut bytes = BTreeMap::new();
        bytes.insert("blobs/raw.bin".to_string(), raw.clone());
        let resolver = Resolver::new(manifest, MapAdapter { bytes });
        let addr = LogicalAddress::parse("blobs/raw.bin").expect("addr");
        let mut buf: Vec<u8> = Vec::new();
        resolver.emit(&addr, &mut buf).expect("emits binary");
        assert_eq!(buf, raw, "binary bytes stream byte-for-byte");
    }

    // VT-5: a traversal-like address is rejected at LogicalAddress construction —
    // the type boundary, ResolveError-free — so the resolver can never be handed
    // one.
    #[test]
    fn traversal_address_rejected_before_resolve() {
        for bad in ["../escape", "/abs/path", "a/../b", "a\\b", "", "./here"] {
            assert!(
                matches!(
                    LogicalAddress::parse(bad),
                    Err(AdmissionError::TraversalRejected(_))
                ),
                "{bad:?} must be rejected at construction, never reach resolve"
            );
        }
    }

    // VT-6: no-write — load() + resolve + emit over a clean temp repo leaves every
    // path byte-for-byte unchanged (REQ-381 seam foundation; the path holds no
    // write capability by construction).
    #[test]
    fn resolve_emit_writes_nothing_to_disk() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let resolver = shipped_resolver();
        let mut sink = std::io::sink();
        for entry in resolver.manifest().entries() {
            resolver.emit(entry.address(), &mut sink).expect("emits");
        }
        assert!(
            std::fs::read_dir(tmp.path())
                .expect("read temp repo")
                .next()
                .is_none(),
            "resolve/emit must not create any path in the temp repo"
        );
    }

    // ── PHASE-01 (SL-227): accessors, availability, membership, widened kind ───

    /// Build a one-entry manifest with an explicit content kind (else MIT /
    /// declared / customizable), for accessor + widened-kind assertions.
    fn one_entry_kind(kind: &str) -> PublicationManifest {
        let body = format!(
            "[[entry]]\n\
             address = \"templates/thing.txt\"\n\
             backing = \"backing/thing.txt\"\n\
             kind = \"{kind}\"\n\
             title = \"Human Title\"\n\
             licence = \"MIT\"\n\
             provenance = \"declared\"\n\
             customization = \"customizable\"\n"
        );
        PublicationManifest::admit(body.as_bytes()).expect("fixture admits")
    }

    // Structured accessors read the declared values (backing stays private).
    #[test]
    fn entry_accessors_read_declared_values() {
        let manifest = one_entry_kind("template");
        let entry = &manifest.entries()[0];
        assert_eq!(entry.title(), "Human Title");
        assert_eq!(entry.kind(), ContentKind::Template);
        assert_eq!(entry.licence(), Licence::Mit);
        assert_eq!(entry.customization(), CustomizationStatus::Customizable);
    }

    // ContentKind widened additively: each new lowercase spelling admits and
    // round-trips through `kind()`; the existing `template` is unchanged.
    #[test]
    fn content_kind_admits_widened_variants() {
        for (raw, expected) in [
            ("template", ContentKind::Template),
            ("reference", ContentKind::Reference),
            ("guidance", ContentKind::Guidance),
            ("integration", ContentKind::Integration),
        ] {
            let manifest = one_entry_kind(raw);
            assert_eq!(
                manifest.entries()[0].kind(),
                expected,
                "kind {raw:?} admits and round-trips"
            );
            assert_eq!(expected.as_str(), raw, "as_str is the declared spelling");
        }
    }

    // available(): true for a backing the bound EmbeddedAdapter can serve,
    // false for a declared-but-absent backing (reads no bytes).
    #[test]
    fn available_true_for_shipped_backing_false_for_absent() {
        let resolver = shipped_resolver();
        let entry = resolver
            .manifest()
            .entries()
            .iter()
            .find(|e| e.address().as_str() == "templates/slice.toml")
            .expect("shipped entry present");
        assert!(resolver.available(entry), "shipped backing is available");

        let absent = one_entry("templates/ghost.toml", "backing/does-not-exist.bin");
        let resolver2 = Resolver::new(absent, EmbeddedAdapter);
        let ghost = &resolver2.manifest().entries()[0];
        assert!(!resolver2.available(ghost), "absent backing is unavailable");
    }

    // available() honours the bound adapter: the in-memory MapAdapter reports
    // its own membership.
    #[test]
    fn available_reflects_bound_adapter_membership() {
        let manifest = one_entry("templates/slice.toml", "relocated/elsewhere.bin");
        let mut bytes = BTreeMap::new();
        bytes.insert("relocated/elsewhere.bin".to_string(), b"payload".to_vec());
        let resolver = Resolver::new(manifest, MapAdapter { bytes });
        assert!(resolver.available(&resolver.manifest().entries()[0]));

        let missing = one_entry("templates/slice.toml", "relocated/elsewhere.bin");
        let empty = Resolver::new(
            missing,
            MapAdapter {
                bytes: BTreeMap::new(),
            },
        );
        assert!(!empty.available(&empty.manifest().entries()[0]));
    }

    // declares_backing(): membership over declared entries' backings.
    #[test]
    fn declares_backing_membership() {
        let manifest = one_entry("templates/slice.toml", "backing/present.bin");
        assert!(manifest.declares_backing("backing/present.bin"));
        assert!(!manifest.declares_backing("backing/not-declared.bin"));
    }

    // VT-4 (SL-227): every shipped entry resolves — the grown manifest declares
    // only real (non-hollow) backings — and every entry is licence=MIT (Option A
    // uniform MIT, D5, keeping the SL-223 VT-3 licence assertion green).
    #[test]
    fn every_shipped_entry_resolves_and_is_mit() {
        let resolver = shipped_resolver();
        let entries = resolver.manifest().entries();
        assert!(!entries.is_empty(), "shipped manifest is non-empty");
        for entry in entries {
            assert!(
                resolver.available(entry),
                "shipped entry {} has a real (non-hollow) backing",
                entry.address().as_str()
            );
            assert_eq!(entry.licence(), Licence::Mit, "Option A: uniform MIT");
            let mut sink = std::io::sink();
            resolver
                .emit(entry.address(), &mut sink)
                .unwrap_or_else(|e| panic!("{} resolves: {e}", entry.address().as_str()));
        }
    }

    // COMPLETENESS (SL-227, de-risks the PHASE-03 crux gate): the published set
    // covers the WHOLE projection complement. Enumerate every embedded install
    // asset via the leaf enumerator, exclude the install manifest and the three
    // projection base backings, and assert every remaining key is declared by some
    // publication entry (delta ⊆ published backings) — the pre-image of PHASE-03's
    // derived reachability gate, proven here through `declares_backing` so
    // `backing` stays private.
    #[test]
    fn published_set_covers_the_full_projection_complement() {
        const BASE_BACKINGS: [&str; 3] = [".gitignore", "doctrine.toml", "project-orientation.md"];
        let manifest = PublicationManifest::load().expect("shipped manifest loads");
        let mut checked = 0usize;
        for key in crate::asset_source::iter() {
            let key = key.as_ref();
            if key == "manifest.toml" || BASE_BACKINGS.contains(&key) {
                continue;
            }
            assert!(
                manifest.declares_backing(key),
                "embedded install asset {key:?} is neither base nor published — \
                 a silent-unreachable file the flip would strand"
            );
            checked += 1;
        }
        assert!(
            checked > 0,
            "the enumerator yielded install assets to check"
        );
    }
}
