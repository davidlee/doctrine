// SPDX-License-Identifier: GPL-3.0-only
//! Observation ledger — typed wire, resolution, query, store, and shared
//! façade (SL-231).
//!
//! ADR-001 classification: leaf. Imports only existing leaves
//! (`fsutil`, `root`, `lexical`). No clock, RNG, disk, environment,
//! terminal, or MCP types in the pure modules.
//!
//! ## Module layout
//!
//! | Module | Responsibility |
//! |---|---|
//! | `wire` | Pure typed envelopes, payloads, facets, origins, controls, schema dispatch, strict validation, explicit-over-automatic facet merge, safe rendering of untrusted content, canonical serialization |
//! | `resolve` | Pure active/history projection and deterministic per-control diagnostics |
//! | `query` | Pure filtering, shared lexical matching, total ordering, and keyset cursors |
//! | `store` | Imperative filesystem seam: UUID-sharded loading, atomic no-clobber publication, replay/collision |
//! | Façade (this module) | Shared service over injected root, identity, time, enrichment, and registered measurement-source admission |
//!
//! PHASE-03 adds the CLI adapter. PHASE-04 adds the MCP tool
//! (`mcp_server::tools::observation_record`). Both adapters sit in the SAME
//! tier and cannot import one another (ADR-001 — the `mcp_server → commands`
//! back edge is severed), so everything they genuinely share — the receipt
//! projection, the facet merge, the terminal escaper, the product-surface
//! constant — is homed here in the leaf rather than duplicated per surface.

pub(crate) mod query;
pub(crate) mod resolve;
pub(crate) mod store;
pub(crate) mod wire;

use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::Context;

use crate::observation::store::Store;
use crate::observation::wire::{
    Diagnostic, Envelope, ObservationKind, Payload, SCHEMA, SCHEMA_VERSION,
};

// ── Machine source registry ───────────────────────────────────────────────

/// The set of registered machine source names that are authorised to create
/// measurement observations. The production registry is empty until a
/// concrete source is settled under QUE-176; tests inject a fake.
#[derive(Debug, Clone)]
pub(crate) struct SourceRegistry {
    #[cfg_attr(not(test), expect(dead_code, reason = "test-only"))]
    sources: BTreeSet<String>,
}

impl SourceRegistry {
    /// Create a new empty registry (production default).
    pub(crate) fn empty() -> Self {
        SourceRegistry {
            sources: BTreeSet::new(),
        }
    }

    /// Register a source name (for tests).
    #[cfg_attr(not(test), expect(dead_code, reason = "test-only"))]
    pub(crate) fn register(&mut self, name: &str) {
        self.sources.insert(name.to_string());
    }

    /// Check whether a source is registered.
    #[cfg_attr(not(test), expect(dead_code, reason = "test-only"))]
    pub(crate) fn is_registered(&self, name: &str) -> bool {
        self.sources.contains(name)
    }
}

// ── Shared service ────────────────────────────────────────────────────────

/// The observation service — the shared façade over the store, identity
/// generation, time supply, enrichment, and measurement-source admission.
///
/// The repository root, `UUIDv7` generation, current time, enrichment context,
/// and source registry are all injected — no clock, RNG, or disk access in
/// this layer beyond the store delegation.
#[derive(Debug)]
pub(crate) struct Service {
    store: Store,
    #[cfg_attr(not(test), expect(dead_code, reason = "test-only until QUE-176"))]
    registry: SourceRegistry,
}

impl Service {
    /// Create a new service over the given repository root with the provided
    /// source registry.
    pub(crate) fn new(root: PathBuf, registry: SourceRegistry) -> Self {
        Service {
            store: Store::new(root),
            registry,
        }
    }

    /// Record a friction observation. The caller supplies the fully-built
    /// envelope (including `uid`, `recorded_at`, and optional enrichment).
    ///
    /// This is the public admission path — friction is always admitted.
    pub(crate) fn record_friction(
        &self,
        envelope: &Envelope,
    ) -> anyhow::Result<store::CreateReceipt> {
        if envelope.kind() != ObservationKind::Friction {
            anyhow::bail!(
                "record_friction called with non-friction kind: {:?}",
                envelope.kind()
            );
        }
        self.store.create(envelope)
    }

    /// Record a measurement observation. Only registered machine sources are
    /// admitted — caller-asserted source metadata cannot open the gate.
    ///
    /// Returns an error if the source is not in the registry.
    #[cfg_attr(not(test), expect(dead_code, reason = "test-only until QUE-176"))]
    pub(crate) fn record_measurement(
        &self,
        envelope: &Envelope,
    ) -> anyhow::Result<store::CreateReceipt> {
        if envelope.kind() != ObservationKind::Measurement {
            anyhow::bail!(
                "record_measurement called with non-measurement kind: {:?}",
                envelope.kind()
            );
        }
        // Extract source from payload.
        let source = match &envelope.payload {
            crate::observation::wire::Payload::Measurement { source, .. } => source.clone(),
            _ => anyhow::bail!("record_measurement requires a measurement payload"),
        };
        if !self.registry.is_registered(&source) {
            anyhow::bail!("measurement source '{source}' is not a registered machine source");
        }
        self.store.create(envelope)
    }

    /// Record a supersession control. The caller supplies `uid` and
    /// `recorded_at` (generated in the command shell — design §2.1).
    /// Validates that `old_uid` and `replacement_uid` both exist as
    /// primary observations and are kind-compatible, then creates the
    /// control atomically.
    pub(crate) fn record_supersession(
        &self,
        uid: &str,
        recorded_at: &str,
        old_uid: &str,
        replacement_uid: &str,
        reason: Option<&str>,
    ) -> anyhow::Result<store::CreateReceipt> {
        // Pre-creation validation: both targets must exist as primaries.
        let old_env = self
            .load_one(old_uid)
            .with_context(|| format!("supersession target {old_uid} does not exist"))?;
        if !old_env.is_primary() {
            anyhow::bail!("supersession target {old_uid} is a control, not a primary observation");
        }

        let replacement_env = self.load_one(replacement_uid).with_context(|| {
            format!("supersession replacement {replacement_uid} does not exist")
        })?;
        if !replacement_env.is_primary() {
            anyhow::bail!(
                "supersession replacement {replacement_uid} is a control, not a primary observation"
            );
        }

        // Kind compatibility
        if old_env.kind() != replacement_env.kind() {
            anyhow::bail!(
                "supersession replacement kind {:?} is incompatible with target kind {:?}",
                replacement_env.kind(),
                old_env.kind()
            );
        }

        let envelope = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: uid.to_string(),
            recorded_at: recorded_at.to_string(),
            facets: None,
            payload: Payload::Supersession {
                old_uid: old_uid.to_string(),
                replacement_uid: replacement_uid.to_string(),
                reason: reason.map(std::string::ToString::to_string),
            },
        };

        self.store.create(&envelope)
    }

    /// Record a retraction control. The caller supplies `uid` and
    /// `recorded_at` (generated in the command shell — design §2.1).
    /// Validates that `target_uid` exists as a primary observation,
    /// then creates the control atomically.
    pub(crate) fn record_retraction(
        &self,
        uid: &str,
        recorded_at: &str,
        target_uid: &str,
        reason: Option<&str>,
    ) -> anyhow::Result<store::CreateReceipt> {
        // Pre-creation validation: target must exist as a primary.
        let target_env = self
            .load_one(target_uid)
            .with_context(|| format!("retraction target {target_uid} does not exist"))?;
        if !target_env.is_primary() {
            anyhow::bail!("retraction target {target_uid} is a control, not a primary observation");
        }

        let envelope = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: uid.to_string(),
            recorded_at: recorded_at.to_string(),
            facets: None,
            payload: Payload::Retraction {
                target_uid: target_uid.to_string(),
                reason: reason.map(std::string::ToString::to_string),
            },
        };

        self.store.create(&envelope)
    }

    /// Load a single observation by UUID. Computes the path from the UUID
    /// and delegates to the store.
    pub(crate) fn load_one(&self, uid: &str) -> anyhow::Result<Envelope> {
        let rel = store::record_path(uid);
        let abs = self.store.root.join(&rel);
        Store::load_one(&abs)
    }

    /// Load all observation records from the corpus, returning envelopes
    /// and diagnostics for malformed records.
    pub(crate) fn load_all(&self) -> (Vec<Envelope>, Vec<Diagnostic>) {
        self.store.load_all()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::wire::{Payload, SCHEMA, SCHEMA_VERSION};
    use std::collections::BTreeMap;

    fn test_service() -> (tempfile::TempDir, Service) {
        let dir = tempfile::tempdir().unwrap();
        let registry = SourceRegistry::empty();
        let svc = Service::new(dir.path().to_path_buf(), registry);
        (dir, svc)
    }

    fn test_service_with_source(source_name: &str) -> (tempfile::TempDir, Service) {
        let dir = tempfile::tempdir().unwrap();
        let mut registry = SourceRegistry::empty();
        registry.register(source_name);
        let svc = Service::new(dir.path().to_path_buf(), registry);
        (dir, svc)
    }

    fn friction_envelope(uid: &str, summary: &str) -> Envelope {
        Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: uid.to_string(),
            recorded_at: "2026-01-01T00:00:00Z".to_string(),
            facets: None,
            payload: Payload::Friction {
                summary: summary.to_string(),
                detail: None,
            },
        }
    }

    fn measurement_envelope(uid: &str, source: &str) -> Envelope {
        Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: uid.to_string(),
            recorded_at: "2026-01-01T00:00:00Z".to_string(),
            facets: None,
            payload: Payload::Measurement {
                source: source.to_string(),
                counters: BTreeMap::new(),
                gauges: BTreeMap::new(),
                scope: None,
                units: None,
                completeness: None,
            },
        }
    }

    #[test]
    fn public_capture_refuses_measurement() {
        let (_dir, svc) = test_service();
        // Even with a well-formed measurement, the empty registry refuses it.
        let env = measurement_envelope("019f1234-5678-7abc-8def-0123456789ab", "claude-p");
        let err = svc.record_measurement(&env).unwrap_err();
        assert!(
            err.to_string().contains("not a registered machine source"),
            "must refuse unregistered source, got: {err}"
        );
    }

    #[test]
    fn registered_machine_source_admits_measurement() {
        let (_dir, svc) = test_service_with_source("claude-p");
        let env = measurement_envelope("019f1234-5678-7abc-8def-0123456789ab", "claude-p");
        let receipt = svc.record_measurement(&env).unwrap();
        assert_eq!(receipt.uid, "019f1234-5678-7abc-8def-0123456789ab");
    }

    #[test]
    fn asserted_source_cannot_open_measurement_gate() {
        // A caller claiming to be a registered source but not actually
        // registered is refused.
        let (_dir, svc) = test_service();
        let env = measurement_envelope("019f1234-5678-7abc-8def-0123456789ab", "claude-p");
        let err = svc.record_measurement(&env).unwrap_err();
        assert!(
            err.to_string().contains("not a registered machine source"),
            "caller-asserted source metadata must not open the gate, got: {err}"
        );
    }

    #[test]
    fn friction_is_always_admitted() {
        let (_dir, svc) = test_service();
        let env = friction_envelope("019f1234-5678-7abc-8def-0123456789ab", "test friction");
        let receipt = svc.record_friction(&env).unwrap();
        assert_eq!(receipt.uid, "019f1234-5678-7abc-8def-0123456789ab");
    }
}
