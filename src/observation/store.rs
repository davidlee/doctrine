// SPDX-License-Identifier: GPL-3.0-only
//! Imperative filesystem seam: UUID-sharded loading, atomic no-clobber
//! publication, replay and collision checks (SL-231 PHASE-02, design §3.2, §5).
//!
//! This is the only observation module that touches disk.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::Serialize;

use crate::fsutil;
use crate::observation::wire::{self, Envelope, ObservationKind, Payload};

// ── Path constants ────────────────────────────────────────────────────────

/// The root directory for observation records relative to the repository root.
/// Its parent is the observation corpus root, under which `records/` and the
/// reserved chronological view are siblings (design § 2.1) — distribution rules
/// that must scope themselves to the corpus derive that root from here rather
/// than re-typing it.
pub(crate) const RECORDS_DIR: &str = ".doctrine/observations/records";

// ── Shard function ────────────────────────────────────────────────────────

/// Compute the shard directory name from a canonical UUID string.
/// Returns the last two characters (not bytes), safe for any UTF-8 input.
fn shard_dir(uid: &str) -> &str {
    let stripped = uid.strip_prefix("urn:uuid:").unwrap_or(uid);
    let char_count = stripped.chars().count();
    if char_count < 2 {
        stripped
    } else {
        // Find the byte offset of the (char_count - 2)-th character.
        let start_byte = stripped
            .char_indices()
            .nth(char_count - 2)
            .map_or(0, |(i, _)| i);
        &stripped[start_byte..]
    }
}

/// Compute the relative record path for a given UUID.
/// Always `records/<tail-2>/<uuid>.toml` — kind-blind and time-blind.
pub(crate) fn record_path(uid: &str) -> PathBuf {
    PathBuf::from(RECORDS_DIR)
        .join(shard_dir(uid))
        .join(format!("{uid}.toml"))
}

// ── Store ─────────────────────────────────────────────────────────────────

/// The observation store — a thin imperative wrapper over a repository root.
/// Owns atomic no-clobber publication, replay/collision checks, and tolerant
/// corpus loading.
#[derive(Debug)]
pub(crate) struct Store {
    /// The absolute repository root.
    pub(crate) root: PathBuf,
}

/// The outcome of a create operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CreateOutcome {
    /// A new record was created.
    Created,
    /// An identical intent was replayed — the existing record is returned.
    Replayed,
}

/// The receipt returned by a successful create.
#[derive(Debug, Clone)]
pub(crate) struct CreateReceipt {
    /// The UUID of the observation.
    pub(crate) uid: String,
    /// The observation kind.
    pub(crate) kind: ObservationKind,
    /// The recorded-at timestamp.
    pub(crate) recorded_at: String,
    /// The relative path from the repository root.
    pub(crate) rel_path: PathBuf,
    /// Whether this was created or replayed.
    pub(crate) outcome: CreateOutcome,
}

/// The machine-readable receipt an adapter renders for a create operation — the
/// ONE JSON contract shared by the CLI (`doctrine observation record`) and the
/// MCP capture tool (`observation_record`), design §3.1/§3.3.
///
/// A projection of [`CreateReceipt`] with the enums flattened to their wire
/// strings, so it belongs beside the value it projects rather than in either
/// adapter: the two adapters sit in the SAME tier and cannot import one another
/// (ADR-001 — the `mcp_server → commands` back edge is severed), so a shared
/// home below both is the only way one definition can serve both.
#[derive(Debug, Serialize)]
pub(crate) struct Receipt {
    pub(crate) uid: String,
    pub(crate) kind: String,
    pub(crate) recorded_at: String,
    pub(crate) rel_path: String,
    pub(crate) outcome: String,
}

impl From<CreateReceipt> for Receipt {
    fn from(r: CreateReceipt) -> Self {
        Receipt {
            uid: r.uid,
            kind: format!("{:?}", r.kind).to_lowercase(),
            recorded_at: r.recorded_at,
            rel_path: r.rel_path.to_string_lossy().to_string(),
            outcome: match r.outcome {
                CreateOutcome::Created => "created".to_string(),
                CreateOutcome::Replayed => "replayed".to_string(),
            },
        }
    }
}

/// Caller intent for replay detection — what the caller explicitly supplied.
/// Automatic time and enrichment are NOT part of replay comparison.
#[derive(Debug, Clone, PartialEq)]
struct CallerIntent {
    kind: ObservationKind,
    payload_intent: PayloadIntent,
    facets_toml: Option<String>,
}

/// The kind-specific payload fields the caller explicitly chose.
#[derive(Debug, Clone, PartialEq)]
enum PayloadIntent {
    Friction {
        summary: String,
        detail: Option<String>,
    },
    Measurement {
        source: String,
        counters: BTreeMap<String, u64>,
        gauges: BTreeMap<String, f64>,
        scope: Option<String>,
        units: Option<String>,
        completeness: Option<String>,
    },
    Supersession {
        old_uid: String,
        replacement_uid: String,
        reason: Option<String>,
    },
    Retraction {
        target_uid: String,
        reason: Option<String>,
    },
}

impl CallerIntent {
    fn from_envelope(envelope: &Envelope) -> Self {
        let payload_intent = match &envelope.payload {
            Payload::Friction { summary, detail } => PayloadIntent::Friction {
                summary: summary.clone(),
                detail: detail.clone(),
            },
            Payload::Measurement {
                source,
                counters,
                gauges,
                scope,
                units,
                completeness,
            } => PayloadIntent::Measurement {
                source: source.clone(),
                counters: counters.clone(),
                gauges: gauges.clone(),
                scope: scope.clone(),
                units: units.clone(),
                completeness: completeness.clone(),
            },
            Payload::Supersession {
                old_uid,
                replacement_uid,
                reason,
            } => PayloadIntent::Supersession {
                old_uid: old_uid.clone(),
                replacement_uid: replacement_uid.clone(),
                reason: reason.clone(),
            },
            Payload::Retraction { target_uid, reason } => PayloadIntent::Retraction {
                target_uid: target_uid.clone(),
                reason: reason.clone(),
            },
        };
        // Explicit facets only — serialize them for comparison (deterministic).
        let facets_toml = envelope.facets.as_ref().map(|f| {
            // Facets are only explicit if the caller set them; we capture
            // the serialized form for byte-level comparison.
            toml::to_string_pretty(f).unwrap_or_default()
        });
        CallerIntent {
            kind: envelope.kind(),
            payload_intent,
            facets_toml,
        }
    }
}

impl Store {
    /// Create a new store rooted at the given repository root (absolute).
    pub(crate) fn new(root: PathBuf) -> Self {
        Store { root }
    }

    /// Publish an envelope to the store.
    ///
    /// Returns `CreateOutcome::Created` for a new record, or
    /// `CreateOutcome::Replayed` when the same caller intent already exists.
    /// Different intent at the same UUID is an identity collision error.
    pub(crate) fn create(&self, envelope: &Envelope) -> anyhow::Result<CreateReceipt> {
        // Validate the envelope before touching disk.
        let diags = wire::validate(envelope);
        if !diags.is_empty() {
            let messages: Vec<String> = diags
                .iter()
                .map(|d| format!("{}: {}", d.path, d.reason))
                .collect();
            anyhow::bail!("validation failed:\n  {}", messages.join("\n  "));
        }

        let rel = record_path(&envelope.uid);
        let abs = self.root.join(&rel);

        // Ensure parent dirs exist. Don't need to track created dirs for
        // rollback — the publication is the only write.
        let mut created: Vec<PathBuf> = Vec::new();
        fsutil::ensure_parent_dirs(&self.root, &rel, &mut created).with_context(|| {
            format!(
                "Failed to create parent dirs for {rel}",
                rel = rel.display()
            )
        })?;
        drop(created); // created dirs not tracked for rollback

        let canonical = wire::canonical_toml(envelope)
            .with_context(|| format!("Failed to serialize envelope for {}", envelope.uid))?;

        let outcome = fsutil::publish_complete(&abs, canonical.as_bytes())
            .with_context(|| format!("Failed to publish record for {}", envelope.uid))?;

        match outcome {
            fsutil::PublishOutcome::Created => Ok(CreateReceipt {
                uid: envelope.uid.clone(),
                kind: envelope.kind(),
                recorded_at: envelope.recorded_at.clone(),
                rel_path: rel,
                outcome: CreateOutcome::Created,
            }),
            fsutil::PublishOutcome::AlreadyExists => {
                // Read the existing record to check intent.
                let existing_bytes = fs::read_to_string(&abs).with_context(|| {
                    format!("Failed to read existing record at {}", abs.display())
                })?;
                let existing: Envelope = toml::from_str(&existing_bytes).with_context(|| {
                    format!("Failed to parse existing record at {}", abs.display())
                })?;

                let existing_intent = CallerIntent::from_envelope(&existing);
                let caller_intent = CallerIntent::from_envelope(envelope);

                if existing_intent == caller_intent {
                    // Replay — return the first write's data. Compute kind
                    // first so we can destructure and move the Strings without
                    // triggering redundant_clone.
                    let kind = existing.kind();
                    let Envelope {
                        uid, recorded_at, ..
                    } = existing;
                    Ok(CreateReceipt {
                        uid,
                        kind,
                        recorded_at,
                        rel_path: rel,
                        outcome: CreateOutcome::Replayed,
                    })
                } else {
                    anyhow::bail!(
                        "UUID {} already exists with different intent — identity collision",
                        envelope.uid
                    );
                }
            }
        }
    }

    /// Load a single record by its absolute path. Returns the parsed envelope
    /// or an error. The path's filename UUID must agree with the envelope's uid.
    pub(crate) fn load_one(path: &Path) -> anyhow::Result<Envelope> {
        let toml_str = fs::read_to_string(path)
            .with_context(|| format!("Failed to read observation at {}", path.display()))?;
        let envelope: Envelope = toml::from_str(&toml_str)
            .with_context(|| format!("Failed to parse observation at {}", path.display()))?;

        // Path/envelope disagreement check.
        let expected_uid = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if envelope.uid != expected_uid {
            anyhow::bail!(
                "UUID/path disagreement: envelope uid '{}' does not match filename '{}'",
                envelope.uid,
                expected_uid
            );
        }

        Ok(envelope)
    }

    /// Load all observation records from the corpus, ignoring reserved temp
    /// names. Returns a list of loaded envelopes and a list of diagnostics
    /// for records that could not be parsed.
    pub(crate) fn load_all(&self) -> (Vec<Envelope>, Vec<wire::Diagnostic>) {
        let records_root = self.root.join(RECORDS_DIR);
        let mut envelopes: Vec<Envelope> = Vec::new();
        let mut diagnostics: Vec<wire::Diagnostic> = Vec::new();

        // If the records directory doesn't exist, empty corpus.
        if !records_root.is_dir() {
            return (envelopes, diagnostics);
        }

        // Walk shard directories.
        let shard_entries = match fs::read_dir(&records_root) {
            Ok(entries) => entries,
            Err(e) => {
                diagnostics.push(wire::Diagnostic::new(
                    records_root.display().to_string(),
                    format!("Failed to read records directory: {e}"),
                ));
                return (envelopes, diagnostics);
            }
        };

        for shard_entry in shard_entries {
            let shard_entry = match shard_entry {
                Ok(e) => e,
                Err(e) => {
                    diagnostics.push(wire::Diagnostic::new(
                        records_root.display().to_string(),
                        format!("Failed to read shard entry: {e}"),
                    ));
                    continue;
                }
            };
            let shard_path = shard_entry.path();
            if !shard_path.is_dir() {
                continue;
            }

            let file_entries = match fs::read_dir(&shard_path) {
                Ok(entries) => entries,
                Err(e) => {
                    diagnostics.push(wire::Diagnostic::new(
                        shard_path.display().to_string(),
                        format!("Failed to read shard directory: {e}"),
                    ));
                    continue;
                }
            };

            for file_entry in file_entries {
                let file_entry = match file_entry {
                    Ok(e) => e,
                    Err(e) => {
                        diagnostics.push(wire::Diagnostic::new(
                            shard_path.display().to_string(),
                            format!("Failed to read record entry: {e}"),
                        ));
                        continue;
                    }
                };
                let file_path = file_entry.path();

                // Ignore reserved temp names.
                let file_name = file_entry.file_name();
                let file_name_str = file_name.to_string_lossy();
                if file_name_str.starts_with(fsutil::PUBLICATION_TEMP_PREFIX) {
                    continue;
                }

                // Only process .toml files.
                if file_path.extension().and_then(|e| e.to_str()) != Some("toml") {
                    continue;
                }

                match Self::load_one(&file_path) {
                    Ok(envelope) => envelopes.push(envelope),
                    Err(e) => {
                        diagnostics.push(wire::Diagnostic::new(
                            file_path.display().to_string(),
                            format!("{e:#}"),
                        ));
                    }
                }
            }
        }

        (envelopes, diagnostics)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::wire::{Facets, Origin, ProvenanceFacet, SCHEMA, SCHEMA_VERSION};
    use std::sync::{Arc, Barrier};
    use std::thread;

    fn temp_store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().to_path_buf());
        (dir, store)
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

    #[expect(dead_code, reason = "available test helper")]
    fn friction_envelope_with_detail(uid: &str, summary: &str, detail: &str) -> Envelope {
        Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: uid.to_string(),
            recorded_at: "2026-01-01T00:00:00Z".to_string(),
            facets: None,
            payload: Payload::Friction {
                summary: summary.to_string(),
                detail: Some(detail.to_string()),
            },
        }
    }

    fn friction_envelope_with_facets(
        uid: &str,
        summary: &str,
        facets: Facets,
        recorded_at: &str,
    ) -> Envelope {
        Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: uid.to_string(),
            recorded_at: recorded_at.to_string(),
            facets: Some(facets),
            payload: Payload::Friction {
                summary: summary.to_string(),
                detail: None,
            },
        }
    }

    // ── UUID/shard/path tests ──────────────────────────────────────────────

    #[test]
    fn different_kind_same_uuid_is_identity_collision() {
        // Design §6: the same UUID under a different kind resolves to the
        // same authoritative path and must be rejected as an identity
        // collision — one path, one record.
        let (_dir, store) = temp_store();
        let uid = "019f1234-5678-7abc-8def-0123456789ab";

        // Create a friction at uid X.
        let friction = friction_envelope(uid, "friction at uid");
        let receipt = store.create(&friction).unwrap();
        assert_eq!(receipt.outcome, CreateOutcome::Created);

        // Attempt a measurement at the same uid — must collide.
        let measurement = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: uid.to_string(),
            recorded_at: "2026-01-01T00:00:00Z".to_string(),
            facets: None,
            payload: Payload::Measurement {
                source: "test".to_string(),
                counters: BTreeMap::new(),
                gauges: BTreeMap::new(),
                scope: None,
                units: None,
                completeness: None,
            },
        };
        let err = store.create(&measurement).unwrap_err();
        assert!(
            err.to_string().contains("identity collision"),
            "different kind at same uid must be identity collision, got: {err}"
        );
    }

    #[test]
    fn shard_dir_uses_last_two_hex_chars() {
        let uid = "019f1234-5678-7abc-8def-0123456789ab";
        assert_eq!(shard_dir(uid), "ab");
    }

    #[test]
    fn shard_dir_handles_multi_byte_input_without_panic() {
        // D2 (RV-317): shard_dir must not panic when a multi-byte
        // character straddles the byte-slice boundary. It operates on
        // characters, not bytes.
        assert_eq!(shard_dir("éa"), "éa"); // 2 chars, < 2 → whole string
        assert_eq!(shard_dir("abé"), "bé"); // 3 chars, last 2 = "bé"
        assert_eq!(shard_dir("a🎉"), "a🎉"); // 2 chars (4 bytes), < 2 → whole
        assert_eq!(shard_dir("abc🎉"), "c🎉"); // 4 chars, last 2 = "c🎉"
        // Pure ASCII still works.
        assert_eq!(shard_dir("019f1234-5678-7abc-8def-0123456789ab"), "ab");
    }

    #[test]
    fn uuid_path_contains_records_and_shard() {
        let uid = "019f1234-5678-7abc-8def-0123456789ab";
        let path = record_path(uid);
        let s = path.to_string_lossy();
        assert!(s.contains("records/"), "path must contain records/");
        assert!(s.contains("ab/"), "path must contain shard ab/");
        assert!(s.ends_with(".toml"), "path must end with .toml");
    }

    #[test]
    fn uuid_path_and_shard_disagreement_fails() {
        let dir = tempfile::tempdir().unwrap();
        // Write a record where the filename doesn't match the envelope uid.
        let bad_path = dir
            .path()
            .join(RECORDS_DIR)
            .join("ab")
            .join("wrong-uuid.toml");
        fs::create_dir_all(bad_path.parent().unwrap()).unwrap();
        #[expect(
            clippy::disallowed_methods,
            reason = "test fixture: write a deliberately malformed record"
        )]
        fs::write(
            &bad_path,
            "schema = \"doctrine.observation\"\nschema_version = 1\nuid = \"019f1234-5678-7abc-8def-0123456789ab\"\nrecorded_at = \"2026-01-01T00:00:00Z\"\nkind = \"friction\"\nsummary = \"x\"\n",
        )
        .unwrap();

        let err = Store::load_one(&bad_path).unwrap_err();
        assert!(
            err.to_string().contains("UUID/path disagreement"),
            "should fail on UUID/path disagreement, got: {}",
            err
        );
    }

    // ── Replay and collision tests ─────────────────────────────────────────

    #[test]
    fn same_intent_replays_without_rewrite() {
        let (_dir, store) = temp_store();
        let envelope = friction_envelope("019f1234-5678-7abc-8def-0123456789ab", "friction a");

        let receipt1 = store.create(&envelope).unwrap();
        assert_eq!(receipt1.outcome, CreateOutcome::Created);

        // Same intent again → replay.
        let receipt2 = store.create(&envelope).unwrap();
        assert_eq!(receipt2.outcome, CreateOutcome::Replayed);
        // Retains first write's recorded_at.
        assert_eq!(receipt2.recorded_at, receipt1.recorded_at);
        // File not rewritten — same path.
        assert_eq!(receipt2.rel_path, receipt1.rel_path);
    }

    #[test]
    fn different_intent_collides_without_overwrite() {
        let (_dir, store) = temp_store();
        let uid = "019f1234-5678-7abc-8def-0123456789ab";
        let e1 = friction_envelope(uid, "summary one");
        let receipt1 = store.create(&e1).unwrap();
        assert_eq!(receipt1.outcome, CreateOutcome::Created);

        // Different summary → collision.
        let e2 = friction_envelope(uid, "summary two");
        let err = store.create(&e2).unwrap_err();
        assert!(
            err.to_string().contains("identity collision"),
            "should fail on identity collision, got: {}",
            err
        );

        // Verify original is intact.
        let abs = store.root.join(record_path(uid));
        let loaded = Store::load_one(&abs).unwrap();
        // Payload doesn't derive PartialEq, compare kind.
        assert_eq!(loaded.kind(), e1.kind());
        match &loaded.payload {
            Payload::Friction { summary, .. } => {
                assert_eq!(summary, "summary one");
            }
            _ => panic!("expected friction payload"),
        }
    }

    #[test]
    fn replay_ignores_automatic_time_and_enrichment() {
        let (_dir, store) = temp_store();
        let uid = "019f1234-5678-7abc-8def-0123456789ab";

        // First write with time T1.
        let e1 = friction_envelope_with_facets(
            uid,
            "same summary",
            Facets {
                provenance: Some(ProvenanceFacet {
                    schema_version: 1,
                    author: Some("alice".to_string()),
                    author_origin: Some(Origin::Explicit),
                    ..Default::default()
                }),
                ..Default::default()
            },
            "2026-01-01T00:00:00Z",
        );
        let receipt1 = store.create(&e1).unwrap();
        assert_eq!(receipt1.outcome, CreateOutcome::Created);

        // Replay with different recorded_at (automatic) and same caller intent.
        let e2 = friction_envelope_with_facets(
            uid,
            "same summary",
            Facets {
                provenance: Some(ProvenanceFacet {
                    schema_version: 1,
                    author: Some("alice".to_string()),
                    author_origin: Some(Origin::Explicit),
                    ..Default::default()
                }),
                ..Default::default()
            },
            "2026-07-01T00:00:00Z", // different time — ignored for replay
        );
        let receipt2 = store.create(&e2).unwrap();
        assert_eq!(receipt2.outcome, CreateOutcome::Replayed);
        // First write's time is retained.
        assert_eq!(receipt2.recorded_at, "2026-01-01T00:00:00Z");
    }

    #[test]
    fn replay_detects_different_explicit_facets() {
        let (_dir, store) = temp_store();
        let uid = "019f1234-5678-7abc-8def-0123456789ab";

        // First write with author=alice.
        let e1 = friction_envelope_with_facets(
            uid,
            "same summary",
            Facets {
                provenance: Some(ProvenanceFacet {
                    schema_version: 1,
                    author: Some("alice".to_string()),
                    author_origin: Some(Origin::Explicit),
                    ..Default::default()
                }),
                ..Default::default()
            },
            "2026-01-01T00:00:00Z",
        );
        store.create(&e1).unwrap();

        // Different explicit facet → collision, not replay.
        let e2 = friction_envelope_with_facets(
            uid,
            "same summary",
            Facets {
                provenance: Some(ProvenanceFacet {
                    schema_version: 1,
                    author: Some("bob".to_string()),
                    author_origin: Some(Origin::Explicit),
                    ..Default::default()
                }),
                ..Default::default()
            },
            "2026-01-01T00:00:00Z",
        );
        let err = store.create(&e2).unwrap_err();
        assert!(
            err.to_string().contains("identity collision"),
            "different explicit facets should collide, got: {}",
            err
        );
    }

    // ── Concurrency tests ──────────────────────────────────────────────────

    #[test]
    fn concurrent_distinct_uuids_survive() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::new(dir.path().to_path_buf()));
        let barrier = Arc::new(Barrier::new(2));

        let uid_a = "019f1234-5678-7abc-8def-0123456789ab";
        let uid_b = "019f5678-1234-7abc-8def-0123456789cd";

        let store_a = Arc::clone(&store);
        let barrier_a = Arc::clone(&barrier);
        let t_a = thread::spawn(move || {
            let e = friction_envelope(uid_a, "thread a");
            barrier_a.wait();
            store_a.create(&e).unwrap()
        });

        let store_b = Arc::clone(&store);
        let barrier_b = Arc::clone(&barrier);
        let t_b = thread::spawn(move || {
            let e = friction_envelope(uid_b, "thread b");
            barrier_b.wait();
            store_b.create(&e).unwrap()
        });

        let ra = t_a.join().unwrap();
        let rb = t_b.join().unwrap();

        assert_eq!(ra.outcome, CreateOutcome::Created);
        assert_eq!(rb.outcome, CreateOutcome::Created);
        assert_ne!(ra.uid, rb.uid);

        // Both records exist on disk.
        let path_a = store.root.join(record_path(uid_a));
        let path_b = store.root.join(record_path(uid_b));
        assert!(path_a.is_file(), "record a must exist");
        assert!(path_b.is_file(), "record b must exist");
    }

    #[test]
    fn concurrent_same_uuid_exactly_one_wins() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::new(dir.path().to_path_buf()));
        let barrier = Arc::new(Barrier::new(2));

        let uid = "019f1234-5678-7abc-8def-0123456789ab";

        // Both threads use the SAME intent — one creates, the other replays.
        let store_a = Arc::clone(&store);
        let barrier_a = Arc::clone(&barrier);
        let t_a = thread::spawn(move || {
            let e = friction_envelope(uid, "same summary");
            barrier_a.wait();
            store_a.create(&e)
        });

        let store_b = Arc::clone(&store);
        let barrier_b = Arc::clone(&barrier);
        let t_b = thread::spawn(move || {
            let e = friction_envelope(uid, "same summary");
            barrier_b.wait();
            store_b.create(&e)
        });

        let ra = t_a.join().unwrap().unwrap();
        let rb = t_b.join().unwrap().unwrap();

        // One created, the other replayed (or both replayed — same outcome).
        let outcomes = [ra.outcome, rb.outcome];
        let created_count = outcomes
            .iter()
            .filter(|o| **o == CreateOutcome::Created)
            .count();
        let replayed_count = outcomes
            .iter()
            .filter(|o| **o == CreateOutcome::Replayed)
            .count();
        assert!(
            created_count == 1 && replayed_count == 1,
            "exactly one must create, one must replay; got created={created_count} replayed={replayed_count}"
        );

        // Record exists on disk.
        let path = store.root.join(record_path(uid));
        assert!(path.is_file(), "record must exist on disk");
    }

    // ── Load tests ─────────────────────────────────────────────────────────

    #[test]
    fn load_all_ignores_temp_names() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().to_path_buf());

        // Create a real record.
        let e1 = friction_envelope("019f1234-5678-7abc-8def-0123456789ab", "real record");
        store.create(&e1).unwrap();

        // Plant a temp file matching the publisher's reserved prefix —
        // proves the contract between fsutil::PUBLICATION_TEMP_PREFIX and
        // the loader's skip predicate (STD-001).
        let shard_path = dir.path().join(RECORDS_DIR).join("ab");
        let temp_path = shard_path.join(format!(
            "{}test.12345.0.pub",
            fsutil::PUBLICATION_TEMP_PREFIX
        ));
        #[expect(
            clippy::disallowed_methods,
            reason = "test fixture: plant a reserved temp file"
        )]
        fs::write(&temp_path, "garbage").unwrap();

        let (envelopes, _diags) = store.load_all();
        assert_eq!(envelopes.len(), 1, "only the real record, not the temp");
    }

    #[test]
    fn load_all_collects_diagnostics_for_malformed_records() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().to_path_buf());

        // Create a valid record.
        let e1 = friction_envelope("019f1234-5678-7abc-8def-0123456789ab", "good");
        store.create(&e1).unwrap();

        // Plant a malformed file.
        let bad_path = dir
            .path()
            .join(RECORDS_DIR)
            .join("ab")
            .join("019f1234-5678-7abc-8def-0123456789cd.toml");
        #[expect(
            clippy::disallowed_methods,
            reason = "test fixture: write a deliberately malformed record"
        )]
        fs::write(&bad_path, "this is not valid toml").unwrap();

        let (envelopes, diags) = store.load_all();
        assert_eq!(envelopes.len(), 1, "good record survives bad neighbor");
        assert!(!diags.is_empty(), "bad record produces diagnostics");
    }

    #[test]
    fn load_all_empty_corpus_is_ok() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().to_path_buf());
        let (envelopes, diags) = store.load_all();
        assert!(envelopes.is_empty());
        assert!(diags.is_empty());
    }

    #[test]
    fn store_validation_fails_before_publication() {
        let (_dir, store) = temp_store();
        // Empty summary → validation failure.
        let bad = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: "019f1234-5678-7abc-8def-0123456789ab".to_string(),
            recorded_at: "2026-01-01T00:00:00Z".to_string(),
            facets: None,
            payload: Payload::Friction {
                summary: String::new(),
                detail: None,
            },
        };
        let err = store.create(&bad).unwrap_err();
        assert!(
            err.to_string().contains("validation failed"),
            "must fail validation before touching disk, got: {}",
            err
        );
        // No file created
        let abs = store.root.join(record_path(&bad.uid));
        assert!(
            !abs.exists(),
            "no file should be created for invalid envelope"
        );
    }
}
