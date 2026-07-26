# SPEC-028: Observation ledger

<!-- Reference forms: entity ids padded (SPEC-007, ADR-004); doc-local refs bare
     (D1 decision, OQ-1 open question). See .doctrine/glossary.md § reference forms. -->

## Overview

The observation ledger is Doctrine's forward-intent container for durable raw
occurrence signals. It realises PRD-018 under the Doctrine system context
(SPEC-003) and is deliberately not a component of the authored-entity engine
(SPEC-004): observations use UUID identity, one-file records, tolerant collection
reads, and consumer-owned processing state rather than the numbered
TOML-plus-Markdown entity lifecycle.

The container includes the observation wire, immutable store, correction resolver,
query engine, shared service façade, trusted CLI adapters, and bounded MCP capture
adapter. It reuses shared repository-root and safe-filesystem primitives without
inheriting an unrelated entity lifecycle.

This is a forward-intent specification. Its contracts are approved design intent;
implementation anchors and observed requirement coverage are added only when
SL-231 ships.

## Responsibilities

Mirrors the structured responsibilities list: own the wire model; immutable
partitioned store; resolved/history derivation; identity, facet, time, and text
queries; shared service contract; CLI/MCP capability boundary; enrichment
precedence; and pure/imperative separation.

### Wire model

An observation has a fixed schema discriminator and four-field core:
`schema_version`, UUID `uid`, `kind`, and `recorded_at`. The core selects a typed,
versioned payload. Optional registered facets—provenance, execution, work context,
correlation, and usage—carry their own schema version and field-level origin.
Unknown facts are omitted. There is no generic metadata bag and no Markdown
companion.

Primary V1 kinds are `friction` and `measurement`. Friction requires only a
non-empty summary and may carry detail. Measurement carries trustworthy
machine-reported values with source, scope, units, and completeness; it never
stores an estimated counter or derived efficiency score. `supersession` and
`retraction` are reserved control kinds.

### Store and creation

Each record is one TOML file:

```text
.doctrine/observations/<kind>/<year>/<month>/<uuid>.toml
```

The path kind, UTC partition, filename UUID, and envelope must agree. Creation is
atomic and create-only. A caller-stable UUID replays successfully only when kind,
typed payload, and explicit facets express the same caller intent; first-write time
and automatic enrichment remain frozen. Different intent at the same UUID is an
identity collision. Different UUIDs are never content-deduplicated.

The create operation returns a receipt and performs no stage, commit, push, index,
triage, or promotion action.

### Resolution and query

Supersession targets an existing public, kind-compatible replacement; retraction
targets exact identity. Each correction writes one control record and never edits
its target. Resolution is a pure fold into exact, active, and historical views.

The correction graph is partitioned into weakly connected components. A component
with a dangling target, cycle, multiple successors, incompatible replacement,
conflicting terminal controls, or recoverably malformed control is diagnostic and
inert as a whole; its parseable public observations remain independently active.
Valid components elsewhere resolve normally.

Exact lookup addresses any UUID regardless of current state. Collection queries
default to the active projection and can request history. Filters operate on kind,
time, and registered facet fields. Search is lexical over defined text fields,
with no clustering or relevance-ranking claim. Results order by `recorded_at`
newest-first and UUID, with stable pagination.

### Interface and enrichment

The trusted CLI owns capture, exact/resolved reads, list, search, supersession, and
retraction. Its shell supplies the repository root, clock, default UUID, rendering,
and allowlisted enrichment inputs to the shared engine service.

The MCP server exposes structured `observation_record` capture through the same
service. For confined workers it accepts only primary signal kinds, resolves the
registered primary repository root server-side, accepts no arbitrary path, and
refuses correction controls. It is a bounded broker, not a general filesystem
capability.

Automatic enrichment considers only named, bounded, non-secret sources. An explicit
facet replaces automatic assembly for that facet. Missing or failed automatic
enrichment warns and proceeds; invalid explicit data fails before creation.

## Concerns

- **Capture cost.** Required data must remain minimal. Enrichment failure cannot
  turn incidental recording into a second task.
- **Hostile content.** Every caller-authored string is untrusted at TOML, JSON,
  terminal, and MCP boundaries. Rendering must not permit instruction confusion,
  control-sequence injection, or structural breakout.
- **Confinement.** The MCP adapter crosses a worker filesystem wall. Server-side
  root resolution, closed kinds, strict schemas, and containment checks are the
  security boundary.
- **Partial corruption.** One malformed or future-schema file must not deny reads
  of the remaining corpus. Diagnostics must remain attached to exact paths and
  identities where recoverable.
- **Corpus growth.** Date partitioning enables bounded operational handling, but
  automated retention and aggregate indexes remain outside this container until
  observed growth justifies them.

## Hypotheses

- The friction caller is sufficient to justify a general observation envelope,
  while reporting consumers can wait until the collected corpus demonstrates their
  real needs.
- A bounded lexical and structured scan is sufficient for the first collection
  slice; derived indexes can be added without changing authoritative records.
- Harness usage schemas will differ. The registered usage facet can retain typed
  source measurements without claiming that all sources are comparable.
- A second conforming ledger consumer may eventually justify a more general ledger
  substrate. Creating that abstraction before then would be speculative.

## Decisions

- **D1 — observations are raw signals, not authored conclusions.** They use their
  own UUID-native lifecycle and do not ride the numbered entity engine.
- **D2 — thin core, typed extensions.** The envelope core remains fixed; payloads,
  facets, origins, and controls are registered versioned schemas.
- **D3 — one observation, one immutable file.** Independent create-new files remove
  the shared append target and preserve occurrence frequency.
- **D4 — strict writes, tolerant reads.** New invalid state is refused; existing
  bad or unknown state is diagnosed without suppressing valid records.
- **D5 — correction is append-only and fail-open.** Valid controls derive current
  state; invalid components are visible and inert.
- **D6 — one service, capability-narrow adapters.** CLI and MCP cannot diverge on
  creation semantics, while the MCP adapter exposes less authority.
- **D7 — pure core, one disk seam.** Wire validation, resolution, and query take
  injected facts; store is the only observation filesystem seam.
