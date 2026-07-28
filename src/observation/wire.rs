// SPDX-License-Identifier: GPL-3.0-only
//! Pure typed observation envelopes, payloads, facets, origins, controls,
//! schema dispatch, strict write validation, and canonical serialization
//! (SL-231 PHASE-01, design §2, §4, §5).
//!
//! No clock, RNG, disk, environment, terminal, or MCP imports.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ── Schema constants ──────────────────────────────────────────────────────

/// The fixed schema discriminator written into every observation.
pub(crate) const SCHEMA: &str = "doctrine.observation";
/// Current schema version. Only V1 exists at this time.
pub(crate) const SCHEMA_VERSION: u32 = 1;

/// The product surface identifier every doctrine-authored observation carries.
/// Surface-independent (the CLI and MCP adapters both enrich with it), so it is
/// single-sourced here rather than re-typed per adapter (STD-001).
pub(crate) const PRODUCT_SURFACE: &str = "doctrine";

// ── Byte limits (design §2.4) ─────────────────────────────────────────────

/// Maximum byte length of a friction `summary`.
pub(crate) const SUMMARY_LIMIT: usize = 1024; // 1 KiB
/// Maximum byte length of a friction `detail`.
pub(crate) const DETAIL_LIMIT: usize = 32768; // 32 KiB
/// Maximum byte length of any single facet string value.
pub(crate) const FACET_STRING_LIMIT: usize = 512;
/// Maximum byte length of the complete serialized record.
pub(crate) const RECORD_LIMIT: usize = 65536; // 64 KiB

// ── Observation kind ──────────────────────────────────────────────────────

/// The primary classification of an observation record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ObservationKind {
    /// Human/agent-reported friction.
    Friction,
    /// Machine-produced measurement from a registered source.
    Measurement,
    /// An append-only correction replacing one primary with another.
    Supersession,
    /// An append-only correction retracting a primary.
    Retraction,
}

impl ObservationKind {
    /// Returns `true` when this is a primary (non-control) kind.
    pub(crate) fn is_primary(self) -> bool {
        matches!(
            self,
            ObservationKind::Friction | ObservationKind::Measurement
        )
    }

    /// Returns `true` when this is a control kind.
    #[expect(dead_code, reason = "not yet used in tests")]
    pub(crate) fn is_control(self) -> bool {
        matches!(
            self,
            ObservationKind::Supersession | ObservationKind::Retraction
        )
    }
}

// ── Payloads ──────────────────────────────────────────────────────────────

/// The kind-specific body of an observation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind")]
pub(crate) enum Payload {
    #[serde(rename = "friction")]
    Friction {
        summary: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
    },
    #[serde(rename = "measurement")]
    Measurement {
        /// Registered machine source name (e.g. "claude-p").
        source: String,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        counters: BTreeMap<String, u64>,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        gauges: BTreeMap<String, f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        scope: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        units: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        completeness: Option<String>,
    },
    #[serde(rename = "supersession")]
    Supersession {
        old_uid: String,
        replacement_uid: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
    #[serde(rename = "retraction")]
    Retraction {
        target_uid: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
}

impl Payload {
    /// The [`ObservationKind`] this payload variant represents.
    pub(crate) fn kind(&self) -> ObservationKind {
        match self {
            Payload::Friction { .. } => ObservationKind::Friction,
            Payload::Measurement { .. } => ObservationKind::Measurement,
            Payload::Supersession { .. } => ObservationKind::Supersession,
            Payload::Retraction { .. } => ObservationKind::Retraction,
        }
    }
}

// ── Field origin metadata ─────────────────────────────────────────────────

/// Whether a facet value was explicitly supplied or automatically enriched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Origin {
    Explicit,
    Automatic,
}

// ── Facets ────────────────────────────────────────────────────────────────

/// Exceptional attribution such as a human author, witness, or ratifier.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProvenanceFacet {
    pub(crate) schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) author_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) witness: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) witness_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) ratifier: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) ratifier_origin: Option<Origin>,
}

/// Execution context: interface, product surface, command, repository role,
/// harness, model, role, mode, lifecycle stage, and skill.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExecutionFacet {
    pub(crate) schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) interface: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) interface_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) product_surface: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) product_surface_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) command_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) repository_context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) repository_context_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) harness: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) harness_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) model_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) role_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) execution_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) execution_mode_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) lifecycle_stage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) lifecycle_stage_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) skill: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) skill_origin: Option<Origin>,
}

/// Canonical work references: slice, phase, backlog item, change, etc.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkContextFacet {
    pub(crate) schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) slice: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) slice_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) phase: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) phase_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) backlog: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) backlog_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) change: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) change_origin: Option<Origin>,
}

/// Correlation identifiers: agent, session, run, request, observations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CorrelationFacet {
    pub(crate) schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) agent_id_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) session: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) session_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) run: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) run_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) request: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) request_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) parent_observation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) parent_observation_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) related_observations: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) related_observations_origin: Option<Origin>,
}

/// Trustworthy machine-measured usage with source, scope, units, completeness.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct UsageFacet {
    pub(crate) schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) source_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) scope_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) units: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) units_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) total_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) total_tokens_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) prompt_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) prompt_tokens_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) completion_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) completion_tokens_origin: Option<Origin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) completeness: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) completeness_origin: Option<Origin>,
}

/// The five optional typed facets of an observation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Facets {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) provenance: Option<ProvenanceFacet>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) execution: Option<ExecutionFacet>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) work_context: Option<WorkContextFacet>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) correlation: Option<CorrelationFacet>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) usage: Option<UsageFacet>,
}

impl Facets {
    /// Gather all string values from every populated facet for search indexing.
    pub(crate) fn string_values(&self) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        if let Some(p) = self.provenance.as_ref() {
            if let Some(v) = p.author.as_deref() {
                out.push(v);
            }
            if let Some(v) = p.witness.as_deref() {
                out.push(v);
            }
            if let Some(v) = p.ratifier.as_deref() {
                out.push(v);
            }
        }
        if let Some(e) = self.execution.as_ref() {
            if let Some(v) = e.interface.as_deref() {
                out.push(v);
            }
            if let Some(v) = e.product_surface.as_deref() {
                out.push(v);
            }
            if let Some(v) = e.command.as_deref() {
                out.push(v);
            }
            if let Some(v) = e.repository_context.as_deref() {
                out.push(v);
            }
            if let Some(v) = e.harness.as_deref() {
                out.push(v);
            }
            if let Some(v) = e.model.as_deref() {
                out.push(v);
            }
            if let Some(v) = e.role.as_deref() {
                out.push(v);
            }
            if let Some(v) = e.execution_mode.as_deref() {
                out.push(v);
            }
            if let Some(v) = e.lifecycle_stage.as_deref() {
                out.push(v);
            }
            if let Some(v) = e.skill.as_deref() {
                out.push(v);
            }
        }
        if let Some(w) = self.work_context.as_ref() {
            if let Some(v) = w.slice.as_deref() {
                out.push(v);
            }
            if let Some(v) = w.phase.as_deref() {
                out.push(v);
            }
            if let Some(v) = w.backlog.as_deref() {
                out.push(v);
            }
            if let Some(v) = w.change.as_deref() {
                out.push(v);
            }
        }
        if let Some(c) = self.correlation.as_ref() {
            if let Some(v) = c.agent_id.as_deref() {
                out.push(v);
            }
            if let Some(v) = c.session.as_deref() {
                out.push(v);
            }
            if let Some(v) = c.run.as_deref() {
                out.push(v);
            }
            if let Some(v) = c.request.as_deref() {
                out.push(v);
            }
            if let Some(v) = c.parent_observation.as_deref() {
                out.push(v);
            }
            if let Some(vals) = &c.related_observations {
                for v in vals {
                    out.push(v.as_str());
                }
            }
        }
        if let Some(u) = self.usage.as_ref() {
            if let Some(v) = u.source.as_deref() {
                out.push(v);
            }
            if let Some(v) = u.scope.as_deref() {
                out.push(v);
            }
            if let Some(v) = u.units.as_deref() {
                out.push(v);
            }
            if let Some(v) = u.completeness.as_deref() {
                out.push(v);
            }
        }
        out
    }
}

/// Merge explicit (caller-supplied) facets over automatic enrichment, field by
/// field. **Explicit values win**, and each winning field carries the caller's
/// own origin marker.
///
/// Pure `Facets → Facets` policy with no adapter in it, so it lives in the leaf
/// beside the facet types themselves rather than in either adapter: the CLI
/// (`commands::observation`) and the MCP capture tool (`mcp_server::tools`) sit
/// in the SAME tier and cannot import one another (ADR-001 — the
/// `mcp_server → commands` back edge is severed), so a shared home below both is
/// the only place one implementation can serve both.
#[expect(
    clippy::assigning_clones,
    reason = "field-by-field Option merge: clone_from on Option is not clearer"
)]
pub(crate) fn merge_explicit_facets(auto: Facets, explicit: Option<Facets>) -> Facets {
    let Some(explicit) = explicit else {
        return auto;
    };

    let mut merged = auto;

    // For each facet group, if explicit has the group, merge field by field
    if let Some(ref e) = explicit.provenance {
        let m = merged.provenance.get_or_insert_with(|| ProvenanceFacet {
            schema_version: 1,
            ..Default::default()
        });
        if e.author.is_some() {
            m.author = e.author.clone();
            m.author_origin = e.author_origin;
        }
        if e.witness.is_some() {
            m.witness = e.witness.clone();
            m.witness_origin = e.witness_origin;
        }
        if e.ratifier.is_some() {
            m.ratifier = e.ratifier.clone();
            m.ratifier_origin = e.ratifier_origin;
        }
    }
    if let Some(ref e) = explicit.execution {
        let m = merged.execution.get_or_insert_with(|| ExecutionFacet {
            schema_version: 1,
            ..Default::default()
        });
        if e.interface.is_some() {
            m.interface = e.interface.clone();
            m.interface_origin = e.interface_origin;
        }
        if e.product_surface.is_some() {
            m.product_surface = e.product_surface.clone();
            m.product_surface_origin = e.product_surface_origin;
        }
        if e.command.is_some() {
            m.command = e.command.clone();
            m.command_origin = e.command_origin;
        }
        if e.repository_context.is_some() {
            m.repository_context = e.repository_context.clone();
            m.repository_context_origin = e.repository_context_origin;
        }
        if e.harness.is_some() {
            m.harness = e.harness.clone();
            m.harness_origin = e.harness_origin;
        }
        if e.model.is_some() {
            m.model = e.model.clone();
            m.model_origin = e.model_origin;
        }
        if e.role.is_some() {
            m.role = e.role.clone();
            m.role_origin = e.role_origin;
        }
        if e.execution_mode.is_some() {
            m.execution_mode = e.execution_mode.clone();
            m.execution_mode_origin = e.execution_mode_origin;
        }
        if e.lifecycle_stage.is_some() {
            m.lifecycle_stage = e.lifecycle_stage.clone();
            m.lifecycle_stage_origin = e.lifecycle_stage_origin;
        }
        if e.skill.is_some() {
            m.skill = e.skill.clone();
            m.skill_origin = e.skill_origin;
        }
    }
    if let Some(ref e) = explicit.work_context {
        let m = merged.work_context.get_or_insert_with(|| WorkContextFacet {
            schema_version: 1,
            ..Default::default()
        });
        if e.slice.is_some() {
            m.slice = e.slice.clone();
            m.slice_origin = e.slice_origin;
        }
        if e.phase.is_some() {
            m.phase = e.phase.clone();
            m.phase_origin = e.phase_origin;
        }
        if e.backlog.is_some() {
            m.backlog = e.backlog.clone();
            m.backlog_origin = e.backlog_origin;
        }
        if e.change.is_some() {
            m.change = e.change.clone();
            m.change_origin = e.change_origin;
        }
    }
    if let Some(ref e) = explicit.correlation {
        let m = merged.correlation.get_or_insert_with(|| CorrelationFacet {
            schema_version: 1,
            ..Default::default()
        });
        if e.agent_id.is_some() {
            m.agent_id = e.agent_id.clone();
            m.agent_id_origin = e.agent_id_origin;
        }
        if e.session.is_some() {
            m.session = e.session.clone();
            m.session_origin = e.session_origin;
        }
        if e.run.is_some() {
            m.run = e.run.clone();
            m.run_origin = e.run_origin;
        }
        if e.request.is_some() {
            m.request = e.request.clone();
            m.request_origin = e.request_origin;
        }
        if e.parent_observation.is_some() {
            m.parent_observation = e.parent_observation.clone();
            m.parent_observation_origin = e.parent_observation_origin;
        }
        if e.related_observations.is_some() {
            m.related_observations = e.related_observations.clone();
            m.related_observations_origin = e.related_observations_origin;
        }
    }
    if let Some(ref e) = explicit.usage {
        let m = merged.usage.get_or_insert_with(|| UsageFacet {
            schema_version: 1,
            ..Default::default()
        });
        if e.source.is_some() {
            m.source = e.source.clone();
            m.source_origin = e.source_origin;
        }
        if e.scope.is_some() {
            m.scope = e.scope.clone();
            m.scope_origin = e.scope_origin;
        }
        if e.units.is_some() {
            m.units = e.units.clone();
            m.units_origin = e.units_origin;
        }
        if e.total_tokens.is_some() {
            m.total_tokens = e.total_tokens;
            m.total_tokens_origin = e.total_tokens_origin;
        }
        if e.prompt_tokens.is_some() {
            m.prompt_tokens = e.prompt_tokens;
            m.prompt_tokens_origin = e.prompt_tokens_origin;
        }
        if e.completion_tokens.is_some() {
            m.completion_tokens = e.completion_tokens;
            m.completion_tokens_origin = e.completion_tokens_origin;
        }
        if e.completeness.is_some() {
            m.completeness = e.completeness.clone();
            m.completeness_origin = e.completeness_origin;
        }
    }

    merged
}

// ── Envelope ──────────────────────────────────────────────────────────────

/// The canonical typed observation record.
///
/// Every observation is one self-contained TOML document. The `kind`
/// field is derived from the payload tag — when serialized to TOML,
/// the payload's `kind` tag serves as the top-level discriminator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Envelope {
    pub(crate) schema: String,
    pub(crate) schema_version: u32,
    pub(crate) uid: String,
    pub(crate) recorded_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) facets: Option<Facets>,
    #[serde(flatten)]
    pub(crate) payload: Payload,
}

impl Envelope {
    /// The [`ObservationKind`] of this envelope (delegates to payload tag).
    pub(crate) fn kind(&self) -> ObservationKind {
        self.payload.kind()
    }

    /// Returns `true` when this is a primary observation.
    pub(crate) fn is_primary(&self) -> bool {
        self.kind().is_primary()
    }

    /// Returns `true` when this is a control record.
    #[expect(dead_code, reason = "not yet used in tests")]
    pub(crate) fn is_control(&self) -> bool {
        self.kind().is_control()
    }
}

// ── Diagnostics ───────────────────────────────────────────────────────────

/// A validation or resolution diagnostic tied to a specific path and reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Diagnostic {
    /// The path (filesystem location or record uid) this diagnostic concerns.
    pub(crate) path: String,
    /// Human-readable reason.
    pub(crate) reason: String,
}

impl Diagnostic {
    pub(crate) fn new(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Diagnostic {
            path: path.into(),
            reason: reason.into(),
        }
    }
}

// ── Strict write validation ───────────────────────────────────────────────

/// Reject a string that contains NUL bytes.
fn reject_nul(s: &str, field: &str) -> Option<Diagnostic> {
    if s.contains('\0') {
        Some(Diagnostic::new(
            field.to_string(),
            format!("{field} contains NUL byte"),
        ))
    } else {
        None
    }
}

/// Reject a string that exceeds `limit` bytes.
fn reject_over_limit(s: &str, field: &str, limit: usize) -> Option<Diagnostic> {
    if s.len() > limit {
        Some(Diagnostic::new(
            field.to_string(),
            format!("{field} exceeds {limit} byte limit (len={})", s.len()),
        ))
    } else {
        None
    }
}

/// Validate a string field: no NUL, within byte limit.
fn validate_string_field(s: &str, field: &str, limit: usize) -> Option<Diagnostic> {
    if let Some(d) = reject_nul(s, field) {
        return Some(d);
    }
    reject_over_limit(s, field, limit)
}

/// Validate a UUID string — must be parseable as a UUID.
pub(crate) fn validate_uid(uid: &str) -> Option<Diagnostic> {
    if uid.is_empty() {
        return Some(Diagnostic::new(uid.to_string(), "uid is empty".to_string()));
    }
    if let Some(d) = reject_nul(uid, "uid") {
        return Some(d);
    }
    if uuid::Uuid::parse_str(uid).is_err() {
        return Some(Diagnostic::new(
            uid.to_string(),
            format!("uid is not a valid UUID: {uid}"),
        ));
    }
    None
}

/// Basic ISO 8601 timestamp check — non-empty, no NUL, reasonable structure.
fn validate_recorded_at(ts: &str) -> Option<Diagnostic> {
    if ts.is_empty() {
        return Some(Diagnostic::new(
            ts.to_string(),
            "recorded_at is empty".to_string(),
        ));
    }
    if let Some(d) = reject_nul(ts, "recorded_at") {
        return Some(d);
    }
    // Acceptable: contains a 'T' separator and ends with 'Z' or a numeric offset.
    // This is deliberately lenient for V1 (the shell supplies the value).
    if !ts.contains('T') {
        return Some(Diagnostic::new(
            ts.to_string(),
            format!("recorded_at is not a valid ISO 8601 timestamp: {ts}"),
        ));
    }
    None
}

/// Validate a facet string value: no NUL, within `FACET_STRING_LIMIT`.
fn validate_facet_string(field: &str, value: &str) -> Option<Diagnostic> {
    validate_string_field(value, field, FACET_STRING_LIMIT)
}

/// Gather all facet string fields for validation.
fn validate_facet_strings(facets: &Facets) -> Vec<Diagnostic> {
    let mut diags: Vec<Diagnostic> = Vec::new();
    if let Some(p) = facets.provenance.as_ref() {
        if let Some(ref v) = p.author
            && let Some(d) = validate_facet_string("facets.provenance.author", v.as_str())
        {
            diags.push(d);
        }
        if let Some(ref v) = p.witness
            && let Some(d) = validate_facet_string("facets.provenance.witness", v.as_str())
        {
            diags.push(d);
        }
        if let Some(ref v) = p.ratifier
            && let Some(d) = validate_facet_string("facets.provenance.ratifier", v.as_str())
        {
            diags.push(d);
        }
    }
    if let Some(e) = facets.execution.as_ref() {
        if let Some(ref v) = e.interface
            && let Some(d) = validate_facet_string("facets.execution.interface", v.as_str())
        {
            diags.push(d);
        }
        if let Some(ref v) = e.product_surface
            && let Some(d) = validate_facet_string("facets.execution.product_surface", v.as_str())
        {
            diags.push(d);
        }
        if let Some(ref v) = e.command
            && let Some(d) = validate_facet_string("facets.execution.command", v.as_str())
        {
            diags.push(d);
        }
        if let Some(ref v) = e.repository_context
            && let Some(d) =
                validate_facet_string("facets.execution.repository_context", v.as_str())
        {
            diags.push(d);
        }
        if let Some(ref v) = e.harness
            && let Some(d) = validate_facet_string("facets.execution.harness", v.as_str())
        {
            diags.push(d);
        }
        if let Some(ref v) = e.model
            && let Some(d) = validate_facet_string("facets.execution.model", v.as_str())
        {
            diags.push(d);
        }
        if let Some(ref v) = e.role
            && let Some(d) = validate_facet_string("facets.execution.role", v.as_str())
        {
            diags.push(d);
        }
        if let Some(ref v) = e.execution_mode
            && let Some(d) = validate_facet_string("facets.execution.execution_mode", v.as_str())
        {
            diags.push(d);
        }
        if let Some(ref v) = e.lifecycle_stage
            && let Some(d) = validate_facet_string("facets.execution.lifecycle_stage", v.as_str())
        {
            diags.push(d);
        }
        if let Some(ref v) = e.skill
            && let Some(d) = validate_facet_string("facets.execution.skill", v.as_str())
        {
            diags.push(d);
        }
    }
    if let Some(w) = facets.work_context.as_ref() {
        if let Some(ref v) = w.slice
            && let Some(d) = validate_facet_string("facets.work_context.slice", v.as_str())
        {
            diags.push(d);
        }
        if let Some(ref v) = w.phase
            && let Some(d) = validate_facet_string("facets.work_context.phase", v.as_str())
        {
            diags.push(d);
        }
        if let Some(ref v) = w.backlog
            && let Some(d) = validate_facet_string("facets.work_context.backlog", v.as_str())
        {
            diags.push(d);
        }
        if let Some(ref v) = w.change
            && let Some(d) = validate_facet_string("facets.work_context.change", v.as_str())
        {
            diags.push(d);
        }
    }
    if let Some(c) = facets.correlation.as_ref() {
        if let Some(ref v) = c.agent_id
            && let Some(d) = validate_facet_string("facets.correlation.agent_id", v.as_str())
        {
            diags.push(d);
        }
        if let Some(ref v) = c.session
            && let Some(d) = validate_facet_string("facets.correlation.session", v.as_str())
        {
            diags.push(d);
        }
        if let Some(ref v) = c.run
            && let Some(d) = validate_facet_string("facets.correlation.run", v.as_str())
        {
            diags.push(d);
        }
        if let Some(ref v) = c.request
            && let Some(d) = validate_facet_string("facets.correlation.request", v.as_str())
        {
            diags.push(d);
        }
        if let Some(ref v) = c.parent_observation
            && let Some(d) =
                validate_facet_string("facets.correlation.parent_observation", v.as_str())
        {
            diags.push(d);
        }
        if let Some(ref vals) = c.related_observations {
            for (i, v) in vals.iter().enumerate() {
                let field = format!("facets.correlation.related_observations[{i}]");
                if let Some(d) = validate_facet_string(&field, v.as_str()) {
                    diags.push(d);
                }
            }
        }
    }
    if let Some(u) = facets.usage.as_ref() {
        if let Some(ref v) = u.source
            && let Some(d) = validate_facet_string("facets.usage.source", v.as_str())
        {
            diags.push(d);
        }
        if let Some(ref v) = u.scope
            && let Some(d) = validate_facet_string("facets.usage.scope", v.as_str())
        {
            diags.push(d);
        }
        if let Some(ref v) = u.units
            && let Some(d) = validate_facet_string("facets.usage.units", v.as_str())
        {
            diags.push(d);
        }
        if let Some(ref v) = u.completeness
            && let Some(d) = validate_facet_string("facets.usage.completeness", v.as_str())
        {
            diags.push(d);
        }
    }
    diags
}

/// Validate an envelope for strict write.
///
/// Returns a (possibly empty) list of diagnostics. An empty list means the
/// envelope passes all validation. Does NOT modify the envelope — callers
/// that receive a non-empty list MUST NOT publish the record.
pub(crate) fn validate(envelope: &Envelope) -> Vec<Diagnostic> {
    let mut diags: Vec<Diagnostic> = Vec::new();

    // Schema discriminator
    if envelope.schema != SCHEMA {
        diags.push(Diagnostic::new(
            envelope.uid.clone(),
            format!("schema must be \"{SCHEMA}\", got \"{}\"", envelope.schema),
        ));
    }
    if envelope.schema_version != SCHEMA_VERSION {
        diags.push(Diagnostic::new(
            envelope.uid.clone(),
            format!(
                "unsupported schema_version {}, expected {SCHEMA_VERSION}",
                envelope.schema_version
            ),
        ));
    }

    // uid
    if let Some(d) = validate_uid(&envelope.uid) {
        diags.push(d);
    }

    // recorded_at
    if let Some(d) = validate_recorded_at(&envelope.recorded_at) {
        diags.push(d);
    }

    // Payload validation
    match &envelope.payload {
        Payload::Friction { summary, detail } => {
            // Summary is required and non-empty
            if summary.is_empty() {
                diags.push(Diagnostic::new(
                    envelope.uid.clone(),
                    "friction summary is empty".to_string(),
                ));
            }
            if let Some(d) = validate_string_field(summary, "payload.summary", SUMMARY_LIMIT) {
                diags.push(d);
            }
            if let Some(detail_str) = detail {
                if detail_str.is_empty() {
                    diags.push(Diagnostic::new(
                        envelope.uid.clone(),
                        "friction detail is present but empty".to_string(),
                    ));
                }
                if let Some(d) = validate_string_field(detail_str, "payload.detail", DETAIL_LIMIT) {
                    diags.push(d);
                }
            }
        }
        Payload::Measurement {
            source,
            counters,
            gauges,
            scope,
            units,
            completeness,
        } => {
            if source.is_empty() {
                diags.push(Diagnostic::new(
                    envelope.uid.clone(),
                    "measurement source is empty".to_string(),
                ));
            }
            if let Some(d) = validate_string_field(source, "payload.source", FACET_STRING_LIMIT) {
                diags.push(d);
            }
            for key in counters.keys() {
                if let Some(d) = validate_facet_string("payload.counters key", key) {
                    diags.push(d);
                }
            }
            for key in gauges.keys() {
                if let Some(d) = validate_facet_string("payload.gauges key", key) {
                    diags.push(d);
                }
            }
            if let Some(s) = scope
                && let Some(d) = validate_facet_string("payload.scope", s)
            {
                diags.push(d);
            }
            if let Some(u) = units
                && let Some(d) = validate_facet_string("payload.units", u)
            {
                diags.push(d);
            }
            if let Some(c) = completeness
                && let Some(d) = validate_facet_string("payload.completeness", c)
            {
                diags.push(d);
            }
        }
        Payload::Supersession {
            old_uid,
            replacement_uid,
            reason,
        } => {
            if let Some(d) = validate_uid(old_uid) {
                diags.push(Diagnostic::new(
                    format!("payload.old_uid: {old_uid}"),
                    d.reason,
                ));
            }
            if let Some(d) = validate_uid(replacement_uid) {
                diags.push(Diagnostic::new(
                    format!("payload.replacement_uid: {replacement_uid}"),
                    d.reason,
                ));
            }
            if old_uid == replacement_uid {
                diags.push(Diagnostic::new(
                    old_uid.clone(),
                    "supersession old_uid equals replacement_uid".to_string(),
                ));
            }
            if let Some(r) = reason {
                if r.is_empty() {
                    diags.push(Diagnostic::new(
                        envelope.uid.clone(),
                        "supersession reason is present but empty".to_string(),
                    ));
                }
                if let Some(d) = validate_facet_string("payload.reason", r) {
                    diags.push(d);
                }
            }
        }
        Payload::Retraction { target_uid, reason } => {
            if let Some(d) = validate_uid(target_uid) {
                diags.push(Diagnostic::new(
                    format!("payload.target_uid: {target_uid}"),
                    d.reason,
                ));
            }
            if let Some(r) = reason {
                if r.is_empty() {
                    diags.push(Diagnostic::new(
                        envelope.uid.clone(),
                        "retraction reason is present but empty".to_string(),
                    ));
                }
                if let Some(d) = validate_facet_string("payload.reason", r) {
                    diags.push(d);
                }
            }
        }
    }

    // Facets validation
    if let Some(facets) = envelope.facets.as_ref() {
        diags.extend(validate_facet_strings(facets));
    }

    // Complete record size check (serialize to measure)
    let serialized = canonical_toml(envelope);
    match serialized {
        Ok(ref s) => {
            if s.len() > RECORD_LIMIT {
                diags.push(Diagnostic::new(
                    envelope.uid.clone(),
                    format!(
                        "complete record exceeds {RECORD_LIMIT} byte limit (len={})",
                        s.len()
                    ),
                ));
            }
        }
        Err(e) => {
            diags.push(Diagnostic::new(
                envelope.uid.clone(),
                format!("unable to measure record size: {e}"),
            ));
        }
    }

    diags
}

// ── Safe rendering of untrusted content ───────────────────────────────────
//
// The other half of the validation policy above. Write validation decides what
// may be STORED (design §6 keeps ANSI/CR/LF verbatim in the record and rejects
// only NUL); this decides how stored — or refused — content may be RENDERED.
// Both adapters need it: the CLI renderer (`commands::observation`) and the MCP
// capture tool (`mcp_server::tools`) are in the SAME tier and cannot import one
// another (ADR-001 — the `mcp_server → commands` back edge is severed), so the
// escaper lives in the leaf below them. Pure: no terminal, env, or I/O.

/// The rendering context for [`escape_hostile`], controlling whether `\n`
/// and `\t` are passed through or escaped.
#[derive(Clone, Copy, Debug)]
pub(crate) enum EscapeContext {
    /// Used for block rendering (one field per line). `\n` and `\t` pass
    /// through because embedded newlines cannot spoof a field key.
    Block,
    /// Used for inline rendering (table cells, single-line diagnostic slots).
    /// `\n` and `\t` are escaped as literal `\\n` and `\\t` so one observation
    /// always renders as exactly one logical row.
    Inline,
}

/// Escape control characters and ANSI escape sequences for safe terminal
/// rendering of untrusted observation content (EX-5).
///
/// Iterates over CHARACTERS (not bytes), so multi-byte UTF-8 sequences
/// pass through intact. ESC (U+001B) and any following CSI sequence are
/// replaced with the literal `\\x1b`. Other C0 controls (below U+0020)
/// and C1 controls (U+0080..U+009F) are rendered as `\\xNN` hex escapes.
///
/// In `Inline` context, `\n` is additionally escaped as `\\n` and `\t`
/// as `\\t` (literal two-character sequences) so content cannot inject
/// extra rows into table layouts. In `Block` context they pass through.
pub(crate) fn escape_hostile(s: &str, context: EscapeContext) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\x1b' => {
                out.push_str("\\x1b");
                // Consume CSI sequence if next char is '['
                if chars.peek() == Some(&'[') {
                    chars.next(); // consume '['
                    // Parameter bytes 0x30..0x3F and intermediate bytes 0x20..0x2F
                    while let Some(&ch) = chars.peek() {
                        if ('0'..='?').contains(&ch) || (' '..='/').contains(&ch) {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    // Final byte 0x40..0x7E
                    if let Some(&ch) = chars.peek()
                        && ('@'..='~').contains(&ch)
                    {
                        chars.next();
                    }
                }
            }
            '\n' if matches!(context, EscapeContext::Inline) => {
                out.push_str("\\n");
            }
            '\t' if matches!(context, EscapeContext::Inline) => {
                out.push_str("\\t");
            }
            c if (c < ' ' || c == '\x7f') && c != '\n' && c != '\t' => {
                write!(out, "\\x{:02x}", u32::from(c)).ok();
            }
            c if ('\u{80}'..='\u{9f}').contains(&c) => {
                write!(out, "\\x{:02x}", u32::from(c)).ok();
            }
            _ => {
                out.push(c);
            }
        }
    }
    out
}

// ── Canonical serialization ───────────────────────────────────────────────

/// Serialize an [`Envelope`] to its canonical TOML string.
///
/// # Errors
///
/// Returns a serde error if the envelope cannot be serialized.
pub(crate) fn canonical_toml(envelope: &Envelope) -> Result<String, toml::ser::Error> {
    toml::to_string_pretty(envelope)
}

/// Deserialize an [`Envelope`] from a canonical TOML string.
///
/// This is the tolerant read path: unknown fields trigger a descriptive
/// error but do not prevent other records from being loaded. The caller
/// is expected to convert parse errors into [`Diagnostic`]s.
///
/// # Errors
///
/// Returns a serde error if the string is not valid TOML or violates the
/// schema (unknown fields, type mismatches, etc.).
#[cfg_attr(not(test), expect(dead_code, reason = "PHASE-04 MCP + tests"))]
pub(crate) fn parse_canonical(toml_str: &str) -> Result<Envelope, toml::de::Error> {
    toml::from_str(toml_str)
}

// ── Tolerant read ─────────────────────────────────────────────────────────

/// Attempt to parse a TOML string into an [`Envelope`], producing
/// diagnostics for any parse failure. The `path` identifies the source
/// (usually the filesystem path) for diagnostic attribution.
///
/// This is the tolerant read entry point: malformed or unsupported records
/// yield a [`Diagnostic`] rather than panicking. A successful parse still
/// requires a subsequent [`validate`] call for write-path enforcement;
/// tolerant reads accept records that parse but may have lint-level issues.
#[cfg_attr(not(test), expect(dead_code, reason = "PHASE-04 MCP"))]
pub(crate) fn tolerant_read(toml_str: &str, path: &str) -> Result<Envelope, Vec<Diagnostic>> {
    match parse_canonical(toml_str) {
        Ok(envelope) => {
            // Schema version check for tolerant read: unsupported versions
            // are a diagnostic, not a hard parse error.
            if envelope.schema_version > SCHEMA_VERSION {
                return Err(vec![Diagnostic::new(
                    path.to_string(),
                    format!(
                        "unsupported schema_version {} (current is {SCHEMA_VERSION})",
                        envelope.schema_version
                    ),
                )]);
            }
            Ok(envelope)
        }
        Err(e) => Err(vec![Diagnostic::new(
            path.to_string(),
            format!("parse error: {e}"),
        )]),
    }
}

// ── Helpers for the public constructor surface ────────────────────────────

/// Build a friction envelope, validating all inputs.
///
/// Returns `Ok(envelope)` if validation passes or `Err(diagnostics)`.
pub(crate) fn build_friction(
    uid: String,
    recorded_at: String,
    summary: String,
    detail: Option<String>,
    facets: Option<Facets>,
) -> Result<Envelope, Vec<Diagnostic>> {
    let envelope = Envelope {
        schema: SCHEMA.to_string(),
        schema_version: SCHEMA_VERSION,
        uid,
        recorded_at,
        facets,
        payload: Payload::Friction { summary, detail },
    };
    let diags = validate(&envelope);
    if diags.is_empty() {
        Ok(envelope)
    } else {
        Err(diags)
    }
}

/// Build a measurement envelope, validating all inputs.
#[expect(
    clippy::too_many_arguments,
    reason = "measurement wire admits counters/gauges + optional metadata"
)]
#[cfg_attr(not(test), expect(dead_code, reason = "PHASE-04 MCP"))]
pub(crate) fn build_measurement(
    uid: String,
    recorded_at: String,
    source: String,
    counters: BTreeMap<String, u64>,
    gauges: BTreeMap<String, f64>,
    scope: Option<String>,
    units: Option<String>,
    completeness: Option<String>,
    facets: Option<Facets>,
) -> Result<Envelope, Vec<Diagnostic>> {
    let envelope = Envelope {
        schema: SCHEMA.to_string(),
        schema_version: SCHEMA_VERSION,
        uid,
        recorded_at,
        facets,
        payload: Payload::Measurement {
            source,
            counters,
            gauges,
            scope,
            units,
            completeness,
        },
    };
    let diags = validate(&envelope);
    if diags.is_empty() {
        Ok(envelope)
    } else {
        Err(diags)
    }
}

/// Build a supersession control envelope.
#[cfg_attr(not(test), expect(dead_code, reason = "PHASE-04 MCP"))]
pub(crate) fn build_supersession(
    uid: String,
    recorded_at: String,
    old_uid: String,
    replacement_uid: String,
    reason: Option<String>,
) -> Result<Envelope, Vec<Diagnostic>> {
    let envelope = Envelope {
        schema: SCHEMA.to_string(),
        schema_version: SCHEMA_VERSION,
        uid,
        recorded_at,
        facets: None,
        payload: Payload::Supersession {
            old_uid,
            replacement_uid,
            reason,
        },
    };
    let diags = validate(&envelope);
    if diags.is_empty() {
        Ok(envelope)
    } else {
        Err(diags)
    }
}

/// Build a retraction control envelope.
#[expect(dead_code, reason = "PHASE-04 MCP")]
pub(crate) fn build_retraction(
    uid: String,
    recorded_at: String,
    target_uid: String,
    reason: Option<String>,
) -> Result<Envelope, Vec<Diagnostic>> {
    let envelope = Envelope {
        schema: SCHEMA.to_string(),
        schema_version: SCHEMA_VERSION,
        uid,
        recorded_at,
        facets: None,
        payload: Payload::Retraction { target_uid, reason },
    };
    let diags = validate(&envelope);
    if diags.is_empty() {
        Ok(envelope)
    } else {
        Err(diags)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "test code is exempt from panic-family lints"
)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Build a minimal valid friction envelope for test reuse.
    fn friction(uid: &str, summary: &str) -> Envelope {
        let uid = uid.to_string();
        let recorded_at = "2026-07-26T10:11:12Z".to_string();
        let summary = summary.to_string();
        let envelope = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid,
            recorded_at,
            facets: None,
            payload: Payload::Friction {
                summary,
                detail: None,
            },
        };
        assert!(validate(&envelope).is_empty());
        envelope
    }

    fn supersession(uid: &str, old: &str, replacement: &str) -> Envelope {
        let uid = uid.to_string();
        let old_uid = old.to_string();
        let replacement_uid = replacement.to_string();
        let recorded_at = "2026-07-27T10:11:12Z".to_string();
        let envelope = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid,
            recorded_at,
            facets: None,
            payload: Payload::Supersession {
                old_uid,
                replacement_uid,
                reason: None,
            },
        };
        assert!(validate(&envelope).is_empty());
        envelope
    }

    fn retraction(uid: &str, target_uid: &str) -> Envelope {
        let uid = uid.to_string();
        let target_uid = target_uid.to_string();
        let recorded_at = "2026-07-28T10:11:12Z".to_string();
        let envelope = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid,
            recorded_at,
            facets: None,
            payload: Payload::Retraction {
                target_uid,
                reason: None,
            },
        };
        assert!(validate(&envelope).is_empty());
        envelope
    }

    fn full_facets() -> Facets {
        Facets {
            provenance: Some(ProvenanceFacet {
                schema_version: 1,
                author: Some("alice".to_string()),
                author_origin: Some(Origin::Explicit),
                ..Default::default()
            }),
            execution: Some(ExecutionFacet {
                schema_version: 1,
                interface: Some("cli".to_string()),
                interface_origin: Some(Origin::Automatic),
                product_surface: Some("doctrine".to_string()),
                product_surface_origin: Some(Origin::Automatic),
                command: Some("observation record".to_string()),
                command_origin: Some(Origin::Automatic),
                ..Default::default()
            }),
            work_context: Some(WorkContextFacet {
                schema_version: 1,
                slice: Some("SL-231".to_string()),
                slice_origin: Some(Origin::Explicit),
                ..Default::default()
            }),
            correlation: Some(CorrelationFacet {
                schema_version: 1,
                agent_id: Some("agent-1".to_string()),
                agent_id_origin: Some(Origin::Automatic),
                ..Default::default()
            }),
            usage: None,
        }
    }

    // ── Wire round-trips ──────────────────────────────────────────────

    #[test]
    fn wire_round_trips_all_kinds_and_facets() {
        // Friction with all five facets populated
        let facets = full_facets();
        let envelope = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: "01909a0a-0000-7000-8000-000000000001".to_string(),
            recorded_at: "2026-07-26T10:11:12Z".to_string(),
            facets: Some(facets),
            payload: Payload::Friction {
                summary: "test friction".to_string(),
                detail: Some("detailed description of friction".to_string()),
            },
        };
        assert!(validate(&envelope).is_empty());

        let toml_str = canonical_toml(&envelope).unwrap();
        let round_tripped = parse_canonical(&toml_str).unwrap();
        let re_ser = canonical_toml(&round_tripped).unwrap();
        assert_eq!(toml_str, re_ser, "round-trip must be stable");

        // Verify kind and payload
        assert_eq!(round_tripped.kind(), ObservationKind::Friction);
        assert!(matches!(round_tripped.payload, Payload::Friction { .. }));

        // Verify facets survived
        let rt_facets = round_tripped.facets.as_ref().unwrap();
        let p = rt_facets.provenance.as_ref().unwrap();
        assert_eq!(p.author.as_deref(), Some("alice"));
        assert_eq!(p.author_origin, Some(Origin::Explicit));
        let e = rt_facets.execution.as_ref().unwrap();
        assert_eq!(e.interface.as_deref(), Some("cli"));
        assert_eq!(e.interface_origin, Some(Origin::Automatic));

        // Measurement
        let mut counters: BTreeMap<String, u64> = BTreeMap::new();
        counters.insert("iterations".to_string(), 42);
        let measurement = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: "01909a0a-0000-7000-8000-000000000002".to_string(),
            recorded_at: "2026-07-26T11:00:00Z".to_string(),
            facets: None,
            payload: Payload::Measurement {
                source: "claude-p".to_string(),
                counters,
                gauges: BTreeMap::new(),
                scope: Some("invocation".to_string()),
                units: None,
                completeness: Some("complete".to_string()),
            },
        };
        assert!(validate(&measurement).is_empty());
        let toml_str = canonical_toml(&measurement).unwrap();
        let rt = parse_canonical(&toml_str).unwrap();
        assert_eq!(rt.kind(), ObservationKind::Measurement);

        // Supersession
        let ss = supersession(
            "01909a0a-0000-7000-8000-000000000003",
            "01909a0a-0000-7000-8000-000000000001",
            "01909a0a-0000-7000-8000-000000000004",
        );
        let toml_str = canonical_toml(&ss).unwrap();
        let rt = parse_canonical(&toml_str).unwrap();
        assert!(matches!(rt.payload, Payload::Supersession { .. }));

        // Retraction
        let ret = retraction(
            "01909a0a-0000-7000-8000-000000000005",
            "01909a0a-0000-7000-8000-000000000001",
        );
        let toml_str = canonical_toml(&ret).unwrap();
        let rt = parse_canonical(&toml_str).unwrap();
        assert!(matches!(rt.payload, Payload::Retraction { .. }));
    }

    // ── Write limits and NUL are strict ───────────────────────────────

    #[test]
    fn write_limits_and_nul_are_strict() {
        // Empty summary rejected
        let r = build_friction(
            "01909a0a-0000-7000-8000-000000000001".to_string(),
            "2026-07-26T10:11:12Z".to_string(),
            String::new(),
            None,
            None,
        );
        assert!(r.is_err(), "empty summary must be rejected");
        let diags = r.unwrap_err();
        assert!(
            diags.iter().any(|d| d.reason.contains("empty")),
            "must report empty summary: {diags:?}"
        );

        // NUL in summary rejected
        let r = build_friction(
            "01909a0a-0000-7000-8000-000000000001".to_string(),
            "2026-07-26T10:11:12Z".to_string(),
            "has\0nul".to_string(),
            None,
            None,
        );
        assert!(r.is_err(), "NUL in summary must be rejected");
        let diags = r.unwrap_err();
        assert!(
            diags.iter().any(|d| d.reason.contains("NUL")),
            "must report NUL: {diags:?}"
        );

        // Over-limit summary rejected
        let long_summary = "x".repeat(SUMMARY_LIMIT + 1);
        let r = build_friction(
            "01909a0a-0000-7000-8000-000000000001".to_string(),
            "2026-07-26T10:11:12Z".to_string(),
            long_summary,
            None,
            None,
        );
        assert!(r.is_err(), "over-limit summary must be rejected");
        let diags = r.unwrap_err();
        assert!(
            diags.iter().any(|d| d.reason.contains("exceeds")),
            "must report over-limit: {diags:?}"
        );

        // Over-limit detail
        let long_detail = "y".repeat(DETAIL_LIMIT + 1);
        let r = build_friction(
            "01909a0a-0000-7000-8000-000000000001".to_string(),
            "2026-07-26T10:11:12Z".to_string(),
            "valid summary".to_string(),
            Some(long_detail),
            None,
        );
        assert!(r.is_err(), "over-limit detail must be rejected");

        // NUL in uid
        let envelope = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: "bad\0uid".to_string(),
            recorded_at: "2026-07-26T10:11:12Z".to_string(),
            facets: None,
            payload: Payload::Friction {
                summary: "ok".to_string(),
                detail: None,
            },
        };
        let diags = validate(&envelope);
        assert!(!diags.is_empty(), "NUL in uid must be rejected");

        // Invalid UUID
        let envelope = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: "not-a-uuid".to_string(),
            recorded_at: "2026-07-26T10:11:12Z".to_string(),
            facets: None,
            payload: Payload::Friction {
                summary: "ok".to_string(),
                detail: None,
            },
        };
        let diags = validate(&envelope);
        assert!(
            diags.iter().any(|d| d.reason.contains("valid UUID")),
            "invalid UUID must be rejected: {diags:?}"
        );

        // Empty recorded_at
        let envelope = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: "01909a0a-0000-7000-8000-000000000001".to_string(),
            recorded_at: String::new(),
            facets: None,
            payload: Payload::Friction {
                summary: "ok".to_string(),
                detail: None,
            },
        };
        let diags = validate(&envelope);
        assert!(
            diags.iter().any(|d| d.reason.contains("empty")),
            "empty recorded_at must be rejected: {diags:?}"
        );

        // Supersession old==replacement
        let r = build_supersession(
            "01909a0a-0000-7000-8000-000000000010".to_string(),
            "2026-07-26T10:11:12Z".to_string(),
            "01909a0a-0000-7000-8000-000000000001".to_string(),
            "01909a0a-0000-7000-8000-000000000001".to_string(),
            None,
        );
        assert!(r.is_err(), "self-supersession must be rejected");

        // Facet string over limit
        let mut facets = Facets::default();
        let long = "z".repeat(FACET_STRING_LIMIT + 1);
        facets.provenance = Some(ProvenanceFacet {
            schema_version: 1,
            author: Some(long),
            ..Default::default()
        });
        let envelope = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: "01909a0a-0000-7000-8000-000000000001".to_string(),
            recorded_at: "2026-07-26T10:11:12Z".to_string(),
            facets: Some(facets),
            payload: Payload::Friction {
                summary: "ok".to_string(),
                detail: None,
            },
        };
        let diags = validate(&envelope);
        assert!(
            diags
                .iter()
                .any(|d| d.path.contains("provenance.author") && d.reason.contains("exceeds")),
            "over-limit facet string must be rejected: {diags:?}"
        );

        // Complete record over limit
        let very_long_summary = "w".repeat(RECORD_LIMIT);
        let envelope = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: "01909a0a-0000-7000-8000-000000000001".to_string(),
            recorded_at: "2026-07-26T10:11:12Z".to_string(),
            facets: None,
            payload: Payload::Friction {
                summary: very_long_summary,
                detail: None,
            },
        };
        let diags = validate(&envelope);
        assert!(
            diags.iter().any(|d| d.reason.contains("complete record")),
            "record over 64 KiB must be rejected: {diags:?}"
        );

        // Unknown field in deserialization
        let unknown_field_toml = r#"
schema = "doctrine.observation"
schema_version = 1
uid = "01909a0a-0000-7000-8000-000000000001"
kind = "friction"
recorded_at = "2026-07-26T10:11:12Z"
summary = "ok"
bogus_field = "should not be here"
"#;
        let result = parse_canonical(unknown_field_toml);
        assert!(result.is_err(), "unknown field must be rejected");
    }

    // ── Limit edge cases (D4, RV-317) ────────────────────────────────────

    #[test]
    fn summary_and_detail_limits_accept_exact_and_one_under() {
        // D4.2 (RV-317): the inclusive side of the boundary. Over-limit
        // rejection is pinned by `write_limits_and_nul_are_strict`.

        // Exactly SUMMARY_LIMIT bytes — accepted.
        let exact_summary = "x".repeat(SUMMARY_LIMIT);
        let r = build_friction(
            "01909a0a-0000-7000-8000-000000000001".to_string(),
            "2026-07-26T10:11:12Z".to_string(),
            exact_summary,
            None,
            None,
        );
        assert!(r.is_ok(), "exactly-at-limit summary must be accepted");

        // One under the limit — accepted.
        let under_summary = "x".repeat(SUMMARY_LIMIT - 1);
        let r = build_friction(
            "01909a0a-0000-7000-8000-000000000001".to_string(),
            "2026-07-26T10:11:12Z".to_string(),
            under_summary,
            None,
            None,
        );
        assert!(r.is_ok(), "one-under-limit summary must be accepted");

        // Exactly DETAIL_LIMIT bytes — accepted.
        let exact_detail = "y".repeat(DETAIL_LIMIT);
        let r = build_friction(
            "01909a0a-0000-7000-8000-000000000001".to_string(),
            "2026-07-26T10:11:12Z".to_string(),
            "valid".to_string(),
            Some(exact_detail),
            None,
        );
        assert!(r.is_ok(), "exactly-at-limit detail must be accepted");

        // DETAIL_LIMIT - 1 — accepted.
        let under_detail = "y".repeat(DETAIL_LIMIT - 1);
        let r = build_friction(
            "01909a0a-0000-7000-8000-000000000001".to_string(),
            "2026-07-26T10:11:12Z".to_string(),
            "valid".to_string(),
            Some(under_detail),
            None,
        );
        assert!(r.is_ok(), "one-under-limit detail must be accepted");
    }

    #[test]
    fn facet_string_limit_accepts_exact_and_one_under() {
        // Exactly FACET_STRING_LIMIT — accepted.
        let exact = "z".repeat(FACET_STRING_LIMIT);
        let mut facets = Facets::default();
        facets.provenance = Some(ProvenanceFacet {
            schema_version: 1,
            author: Some(exact),
            ..Default::default()
        });
        let envelope = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: "01909a0a-0000-7000-8000-000000000001".to_string(),
            recorded_at: "2026-07-26T10:11:12Z".to_string(),
            facets: Some(facets),
            payload: Payload::Friction {
                summary: "ok".to_string(),
                detail: None,
            },
        };
        let diags = validate(&envelope);
        assert!(diags.is_empty(), "exact facet string must be accepted");

        // One under — accepted.
        let under = "z".repeat(FACET_STRING_LIMIT - 1);
        let mut facets = Facets::default();
        facets.provenance = Some(ProvenanceFacet {
            schema_version: 1,
            author: Some(under),
            ..Default::default()
        });
        let envelope = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: "01909a0a-0000-7000-8000-000000000001".to_string(),
            recorded_at: "2026-07-26T10:11:12Z".to_string(),
            facets: Some(facets),
            payload: Payload::Friction {
                summary: "ok".to_string(),
                detail: None,
            },
        };
        let diags = validate(&envelope);
        assert!(diags.is_empty(), "one-under facet string must be accepted");
    }

    #[test]
    fn multi_byte_straddling_a_limit_is_rejected_not_truncated() {
        // D4.1 (RV-317): A multi-byte character straddling a byte limit
        // boundary must not panic and must produce a field-specific
        // over-limit diagnostic (never truncate).

        // 2-byte char: build SUMMARY_LIMIT-1 of ASCII, then append 'é' (2 bytes).
        // Total bytes = SUMMARY_LIMIT + 1 (exceeds limit).
        let prefix = "x".repeat(SUMMARY_LIMIT - 1);
        let multi_byte_summary = format!("{prefix}é");
        assert_eq!(
            multi_byte_summary.len(),
            SUMMARY_LIMIT + 1,
            "sanity: total bytes must exceed limit by 1"
        );
        let r = build_friction(
            "01909a0a-0000-7000-8000-000000000001".to_string(),
            "2026-07-26T10:11:12Z".to_string(),
            multi_byte_summary,
            None,
            None,
        );
        assert!(r.is_err(), "multi-byte summary over limit must be rejected");
        let diags = r.unwrap_err();
        assert!(
            diags
                .iter()
                .any(|d| d.path.contains("summary") && d.reason.contains("exceeds")),
            "must produce field-specific over-limit diagnostic for multi-byte summary: {diags:?}"
        );

        // Same for detail with 3-byte char (€, U+20AC).
        let detail_prefix = "y".repeat(DETAIL_LIMIT - 2);
        let multi_byte_detail = format!("{detail_prefix}\u{20AC}"); // '€' is 3 bytes
        assert_eq!(
            multi_byte_detail.len(),
            DETAIL_LIMIT + 1,
            "sanity: detail bytes must exceed limit by 1"
        );
        let r = build_friction(
            "01909a0a-0000-7000-8000-000000000001".to_string(),
            "2026-07-26T10:11:12Z".to_string(),
            "valid".to_string(),
            Some(multi_byte_detail),
            None,
        );
        assert!(r.is_err(), "multi-byte detail over limit must be rejected");

        // 4-byte char (🎉, U+1F389) at facet string boundary.
        let facet_prefix = "z".repeat(FACET_STRING_LIMIT - 3);
        let multi_byte_facet = format!("{facet_prefix}\u{1F389}"); // '🎉' is 4 bytes
        assert_eq!(
            multi_byte_facet.len(),
            FACET_STRING_LIMIT + 1,
            "sanity: facet bytes must exceed limit by 1"
        );
        let mut facets = Facets::default();
        facets.provenance = Some(ProvenanceFacet {
            schema_version: 1,
            author: Some(multi_byte_facet),
            ..Default::default()
        });
        let envelope = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: "01909a0a-0000-7000-8000-000000000001".to_string(),
            recorded_at: "2026-07-26T10:11:12Z".to_string(),
            facets: Some(facets),
            payload: Payload::Friction {
                summary: "ok".to_string(),
                detail: None,
            },
        };
        let diags = validate(&envelope);
        assert!(
            diags
                .iter()
                .any(|d| d.path.contains("provenance.author") && d.reason.contains("exceeds")),
            "multi-byte facet string over limit must be rejected: {diags:?}"
        );
    }

    #[test]
    fn decomposed_form_at_exact_byte_limit_is_accepted() {
        // D4.3 (RV-317): decomposed sequence (e + U+0301) where char count,
        // grapheme count and byte count all differ. Must be validated by
        // byte length, not char or grapheme count.
        // 'e' + combining acute = 2 chars, 3 bytes, 1 grapheme.
        let decomposed = "e\u{0301}";
        assert_eq!(decomposed.len(), 3, "decomposed form is 3 bytes");
        assert_eq!(decomposed.chars().count(), 2, "but 2 chars");

        // Build a summary at exactly SUMMARY_LIMIT bytes using decomposed chars.
        // Each decomposed unit is 3 bytes, so 341 units = 1023 bytes, + 'x' = 1024.
        let unit = "e\u{0301}";
        let units = SUMMARY_LIMIT / 3; // 341
        let mut exact = String::new();
        for _ in 0..units {
            exact.push_str(unit);
        }
        exact.push('x'); // 1023 + 1 = 1024 = SUMMARY_LIMIT
        assert_eq!(
            exact.len(),
            SUMMARY_LIMIT,
            "decomposed summary built to exact limit"
        );
        let r = build_friction(
            "01909a0a-0000-7000-8000-000000000001".to_string(),
            "2026-07-26T10:11:12Z".to_string(),
            exact,
            None,
            None,
        );
        assert!(r.is_ok(), "exact-limit decomposed summary must be accepted");
    }

    // ── Tolerant read isolates bad records ────────────────────────────

    #[test]
    fn tolerant_read_isolates_bad_records() {
        // Malformed TOML
        let r = tolerant_read("this is not toml {{{", "path/to/record.toml");
        assert!(r.is_err());
        let diags = r.unwrap_err();
        assert_eq!(diags.len(), 1);
        assert!(diags[0].reason.contains("parse error"));
        assert_eq!(diags[0].path, "path/to/record.toml");

        // Unsupported schema version
        let future_toml = format!(
            r#"
schema = "{SCHEMA}"
schema_version = 99
uid = "01909a0a-0000-7000-8000-000000000001"
kind = "friction"
recorded_at = "2026-07-26T10:11:12Z"
summary = "ok"
"#
        );
        let r = tolerant_read(&future_toml, "path/to/future.toml");
        assert!(r.is_err());
        let diags = r.unwrap_err();
        assert!(
            diags[0].reason.contains("unsupported schema_version"),
            "must reject unsupported schema_version: {diags:?}"
        );

        // Valid record passes tolerant read
        let valid_toml = format!(
            r#"
schema = "{SCHEMA}"
schema_version = 1
uid = "01909a0a-0000-7000-8000-000000000001"
kind = "friction"
recorded_at = "2026-07-26T10:11:12Z"
summary = "valid friction"
"#
        );
        let r = tolerant_read(&valid_toml, "path/to/valid.toml");
        assert!(r.is_ok(), "valid record must pass tolerant read: {r:?}");

        // Current version should parse as supported
        let current_toml = format!(
            r#"
schema = "{SCHEMA}"
schema_version = {SCHEMA_VERSION}
uid = "01909a0a-0000-7000-8000-000000000001"
kind = "friction"
recorded_at = "2026-07-26T10:11:12Z"
summary = "ok"
"#
        );
        let r = tolerant_read(&current_toml, "path/to/current.toml");
        assert!(r.is_ok(), "current version must pass tolerant read");
    }

    // ── Canonical serialization stability ─────────────────────────────

    #[test]
    fn canonical_serialization_is_stable() {
        let e = friction("01909a0a-0000-7000-8000-000000000001", "test summary");
        let first = canonical_toml(&e).unwrap();
        let second = canonical_toml(&e).unwrap();
        assert_eq!(
            first, second,
            "canonical serialization must be deterministic"
        );
    }

    // ── Facet string collections ──────────────────────────────────────

    #[test]
    fn facets_string_values_gathers_all_populated_strings() {
        let facets = full_facets();
        let values = facets.string_values();
        // Check a few key values
        let set: BTreeSet<&str> = values.into_iter().collect();
        assert!(set.contains("alice"));
        assert!(set.contains("cli"));
        assert!(set.contains("doctrine"));
        assert!(set.contains("observation record"));
        assert!(set.contains("SL-231"));
        assert!(set.contains("agent-1"));
    }

    /// Cardinality guard: `string_values()` and `validate_facet_strings`
    /// must walk the same number of string fields. A one-sided addition
    /// (e.g. adding a field to one function but not the other) fails this
    /// test before the real refactor lands.
    #[test]
    fn facet_string_field_sets_have_equal_cardinality() {
        // Interim guard for the triplicated facet enumeration (IMP-329): until
        // all three walks derive from one field source, this is what makes a
        // one-sided addition fail loudly.
        //
        // The exhaustive struct literals are LOAD-BEARING. With
        // `..Default::default()` a newly added field defaults to `None`, both
        // walks skip it, and the counts still match — the guard would pass the
        // exact mistake it exists to catch (verified: adding a field to
        // `string_values` alone left this test green). Listing every field means
        // a new one stops compiling HERE first, which forces it into the
        // fixture, which is what lets the comparison below see it at all.
        let long = "!".repeat(FACET_STRING_LIMIT + 1);
        let facets = Facets {
            provenance: Some(ProvenanceFacet {
                schema_version: 1,
                author: Some(long.clone()),
                witness: Some(long.clone()),
                ratifier: Some(long.clone()),
                author_origin: None,
                witness_origin: None,
                ratifier_origin: None,
            }),
            execution: Some(ExecutionFacet {
                schema_version: 1,
                interface: Some(long.clone()),
                product_surface: Some(long.clone()),
                command: Some(long.clone()),
                repository_context: Some(long.clone()),
                harness: Some(long.clone()),
                model: Some(long.clone()),
                role: Some(long.clone()),
                execution_mode: Some(long.clone()),
                lifecycle_stage: Some(long.clone()),
                skill: Some(long.clone()),
                interface_origin: None,
                product_surface_origin: None,
                command_origin: None,
                repository_context_origin: None,
                harness_origin: None,
                model_origin: None,
                role_origin: None,
                execution_mode_origin: None,
                lifecycle_stage_origin: None,
                skill_origin: None,
            }),
            work_context: Some(WorkContextFacet {
                schema_version: 1,
                slice: Some(long.clone()),
                phase: Some(long.clone()),
                backlog: Some(long.clone()),
                change: Some(long.clone()),
                slice_origin: None,
                phase_origin: None,
                backlog_origin: None,
                change_origin: None,
            }),
            correlation: Some(CorrelationFacet {
                schema_version: 1,
                agent_id: Some(long.clone()),
                session: Some(long.clone()),
                run: Some(long.clone()),
                request: Some(long.clone()),
                parent_observation: Some(long.clone()),
                related_observations: Some(vec![long.clone()]),
                agent_id_origin: None,
                session_origin: None,
                run_origin: None,
                request_origin: None,
                parent_observation_origin: None,
                related_observations_origin: None,
            }),
            usage: Some(UsageFacet {
                schema_version: 1,
                source: Some(long.clone()),
                scope: Some(long.clone()),
                units: Some(long.clone()),
                completeness: Some(long.clone()),
                total_tokens: None,
                prompt_tokens: None,
                completion_tokens: None,
                source_origin: None,
                scope_origin: None,
                units_origin: None,
                completeness_origin: None,
                total_tokens_origin: None,
                prompt_tokens_origin: None,
                completion_tokens_origin: None,
            }),
        };

        // Every string field is over-limit, so each walk yields exactly one
        // entry per field it knows about. Divergence means the enumerations
        // have drifted apart.
        let walked = facets.string_values().len();
        let validated = validate_facet_strings(&facets).len();
        assert_eq!(
            walked, validated,
            "string_values() walks {walked} facet string fields but \
             validate_facet_strings() validates {validated} — a field was added \
             to one enumeration and not the other (see IMP-329)"
        );
    }
    #[test]
    fn empty_summary_direct_rejection() {
        let diags = {
            let e = Envelope {
                schema: SCHEMA.to_string(),
                schema_version: SCHEMA_VERSION,
                uid: "01909a0a-0000-7000-8000-000000000001".to_string(),
                recorded_at: "2026-07-26T10:11:12Z".to_string(),
                facets: None,
                payload: Payload::Friction {
                    summary: String::new(),
                    detail: None,
                },
            };
            validate(&e)
        };
        assert!(!diags.is_empty());
    }

    #[test]
    fn empty_detail_present_is_rejected() {
        let e = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: "01909a0a-0000-7000-8000-000000000001".to_string(),
            recorded_at: "2026-07-26T10:11:12Z".to_string(),
            facets: None,
            payload: Payload::Friction {
                summary: "ok".to_string(),
                detail: Some(String::new()),
            },
        };
        let diags = validate(&e);
        assert!(
            diags.iter().any(|d| d.reason.contains("empty")),
            "empty detail must be rejected: {diags:?}"
        );
    }

    #[test]
    fn measurement_empty_source_rejected() {
        let r = build_measurement(
            "01909a0a-0000-7000-8000-000000000001".to_string(),
            "2026-07-26T10:11:12Z".to_string(),
            String::new(),
            BTreeMap::new(),
            BTreeMap::new(),
            None,
            None,
            None,
            None,
        );
        assert!(r.is_err(), "empty measurement source must be rejected");
    }

    #[test]
    fn recorded_at_missing_t_separator_rejected() {
        let e = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: "01909a0a-0000-7000-8000-000000000001".to_string(),
            recorded_at: "2026-07-26".to_string(),
            facets: None,
            payload: Payload::Friction {
                summary: "ok".to_string(),
                detail: None,
            },
        };
        let diags = validate(&e);
        assert!(
            diags.iter().any(|d| d.reason.contains("ISO 8601")),
            "missing T must be rejected: {diags:?}"
        );
    }

    #[test]
    fn supersession_empty_reason_rejected() {
        let envelope = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: "01909a0a-0000-7000-8000-000000000010".to_string(),
            recorded_at: "2026-07-26T10:11:12Z".to_string(),
            facets: None,
            payload: Payload::Supersession {
                old_uid: "01909a0a-0000-7000-8000-000000000001".to_string(),
                replacement_uid: "01909a0a-0000-7000-8000-000000000002".to_string(),
                reason: Some(String::new()),
            },
        };
        let diags = validate(&envelope);
        assert!(
            diags.iter().any(|d| d.reason.contains("empty")),
            "empty supersession reason must be rejected: {diags:?}"
        );
    }

    #[test]
    fn retraction_empty_reason_rejected() {
        let envelope = Envelope {
            schema: SCHEMA.to_string(),
            schema_version: SCHEMA_VERSION,
            uid: "01909a0a-0000-7000-8000-000000000010".to_string(),
            recorded_at: "2026-07-26T10:11:12Z".to_string(),
            facets: None,
            payload: Payload::Retraction {
                target_uid: "01909a0a-0000-7000-8000-000000000001".to_string(),
                reason: Some(String::new()),
            },
        };
        let diags = validate(&envelope);
        assert!(
            diags.iter().any(|d| d.reason.contains("empty")),
            "empty retraction reason must be rejected: {diags:?}"
        );
    }

    #[test]
    fn schema_discriminator_must_be_exact() {
        let e = Envelope {
            schema: "wrong.schema".to_string(),
            schema_version: SCHEMA_VERSION,
            uid: "01909a0a-0000-7000-8000-000000000001".to_string(),
            recorded_at: "2026-07-26T10:11:12Z".to_string(),
            facets: None,
            payload: Payload::Friction {
                summary: "ok".to_string(),
                detail: None,
            },
        };
        let diags = validate(&e);
        assert!(
            diags.iter().any(|d| d.reason.contains("schema must be")),
            "wrong schema must be rejected: {diags:?}"
        );
    }

    // ── Hostile strings survive canonical round-trip byte-exact ─────

    #[test]
    fn hostile_strings_round_trip_through_canonical_toml() {
        // Build one string covering every hostile-but-legal category
        // required by design §6: ANSI escapes, bare CR / bare LF,
        // TOML string delimiters, TOML structural characters, Unicode
        // bidi/zero-width, backslash runs, and \u-looking literals.
        let hostile = concat!(
            "\x1b[31mRED\x1b[0m ",
            "\x1b]0;title\x07 ",
            "\rCR ",
            "\nbarenl ",
            "\"basic\" ",
            "\"\"\"multi\"\"\" ",
            "= [ ] # ",
            "\u{200b}zwsp\u{202e}rtl ",
            "\\\\ ",
            "\\u0041"
        );

        // Sanity: no NUL — this content must pass validation.
        assert!(!hostile.contains('\0'));
        assert!(hostile.len() <= SUMMARY_LIMIT);
        assert!(hostile.len() <= DETAIL_LIMIT);
        assert!(hostile.len() <= FACET_STRING_LIMIT);

        // ── summary ───────────────────────────────────────────────
        {
            let e = Envelope {
                schema: SCHEMA.to_string(),
                schema_version: SCHEMA_VERSION,
                uid: "01909a0a-0000-7000-8000-000000000001".to_string(),
                recorded_at: "2026-07-26T10:11:12Z".to_string(),
                facets: None,
                payload: Payload::Friction {
                    summary: hostile.to_string(),
                    detail: None,
                },
            };
            assert!(validate(&e).is_empty());
            let toml_str = canonical_toml(&e).unwrap();
            let rt = parse_canonical(&toml_str).unwrap();
            let Payload::Friction { summary, .. } = &rt.payload else {
                panic!("expected Friction payload");
            };
            assert_eq!(
                summary, hostile,
                "hostile summary must survive canonical round-trip byte-exact"
            );
        }

        // ── detail ────────────────────────────────────────────────
        {
            let e = Envelope {
                schema: SCHEMA.to_string(),
                schema_version: SCHEMA_VERSION,
                uid: "01909a0a-0000-7000-8000-000000000001".to_string(),
                recorded_at: "2026-07-26T10:11:12Z".to_string(),
                facets: None,
                payload: Payload::Friction {
                    summary: "ok".to_string(),
                    detail: Some(hostile.to_string()),
                },
            };
            assert!(validate(&e).is_empty());
            let toml_str = canonical_toml(&e).unwrap();
            let rt = parse_canonical(&toml_str).unwrap();
            let Payload::Friction { detail, .. } = &rt.payload else {
                panic!("expected Friction payload");
            };
            assert_eq!(
                detail.as_deref(),
                Some(hostile),
                "hostile detail must survive canonical round-trip byte-exact"
            );
        }

        // ── facet string field (provenance.author) ────────────────
        {
            let mut facets = Facets::default();
            facets.provenance = Some(ProvenanceFacet {
                schema_version: 1,
                author: Some(hostile.to_string()),
                ..Default::default()
            });
            let e = Envelope {
                schema: SCHEMA.to_string(),
                schema_version: SCHEMA_VERSION,
                uid: "01909a0a-0000-7000-8000-000000000001".to_string(),
                recorded_at: "2026-07-26T10:11:12Z".to_string(),
                facets: Some(facets),
                payload: Payload::Friction {
                    summary: "ok".to_string(),
                    detail: None,
                },
            };
            assert!(validate(&e).is_empty());
            let toml_str = canonical_toml(&e).unwrap();
            let rt = parse_canonical(&toml_str).unwrap();
            let author = rt
                .facets
                .as_ref()
                .and_then(|f| f.provenance.as_ref())
                .and_then(|p| p.author.as_deref());
            assert_eq!(
                author,
                Some(hostile),
                "hostile facet string must survive canonical round-trip byte-exact"
            );
        }

        // ── NUL in the same field is REJECTED ──────────────────────
        {
            let nul_string = format!("{}before\0after", hostile);
            let e = Envelope {
                schema: SCHEMA.to_string(),
                schema_version: SCHEMA_VERSION,
                uid: "01909a0a-0000-7000-8000-000000000001".to_string(),
                recorded_at: "2026-07-26T10:11:12Z".to_string(),
                facets: None,
                payload: Payload::Friction {
                    summary: nul_string,
                    detail: None,
                },
            };
            let diags = validate(&e);
            assert!(
                diags.iter().any(|d| d.reason.contains("NUL")),
                "NUL in summary must be rejected, not escaped: {diags:?}"
            );
        }
    }

    // ── Omission means unknown ────────────────────────────────────────

    #[test]
    fn omission_means_unknown() {
        // When facets are None on a friction record, they should remain
        // None after round-trip — not default-initialized.
        let e = friction("01909a0a-0000-7000-8000-000000000001", "test");
        assert!(e.facets.is_none());
        let toml_str = canonical_toml(&e).unwrap();
        let rt = parse_canonical(&toml_str).unwrap();
        assert!(
            rt.facets.is_none(),
            "absent facets must remain None after round-trip"
        );

        // Detail omitted should stay None
        let e = friction("01909a0a-0000-7000-8000-000000000001", "test");
        assert!(matches!(&e.payload, Payload::Friction { detail: None, .. }));
        let toml_str = canonical_toml(&e).unwrap();
        let rt = parse_canonical(&toml_str).unwrap();
        assert!(
            matches!(&rt.payload, Payload::Friction { detail: None, .. }),
            "omitted detail must remain None after round-trip"
        );
    }
}
