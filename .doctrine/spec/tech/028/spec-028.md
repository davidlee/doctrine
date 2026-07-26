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
UUID-sharded store; per-control resolved/history derivation; identity, facet,
time, and text queries; shared service and registered-source contracts;
CLI/MCP capability boundary; enrichment precedence; and pure/imperative
separation under ADR-001.

### Wire model

An observation has a fixed schema discriminator and four-field core:
`schema_version`, UUID `uid`, `kind`, and `recorded_at`. The core selects a typed,
versioned payload. Optional registered facets—provenance, execution, work context,
correlation, and usage—carry their own schema version and field-level origin.
Unknown facts are omitted. There is no generic metadata bag and no Markdown
companion.

The execution facet names `interface`, `product_surface`, `command`,
`repository_context` (`primary` or `worker`), harness, model, role, execution
mode or arm, lifecycle stage, and skill. The correlation facet names
`agent_id`, session, run, request, parent-observation, and related-observation
identifiers.

Primary V1 kinds are `friction` and `measurement`; **primary observation**
means either non-control kind. Friction requires only a non-empty summary and
may carry detail. Measurement carries trustworthy machine-reported values with
source, scope, units, completeness, and supported counters; it never stores an
estimated counter or derived efficiency score. `supersession` and `retraction`
are reserved control kinds.

Writes enforce deterministic UTF-8 byte bounds: 1 KiB summary, 32 KiB detail,
512 bytes per facet string, and 64 KiB per serialized record. NUL and
over-limit input are refused without silent truncation. Structured serializers
own TOML/JSON boundaries, terminal controls are escaped on display, and any
agent-facing observation text is framed as untrusted data.

### Store and creation

Each record is one TOML file:

```text
.doctrine/observations/records/<tail-2>/<uuid>.toml
```

`<tail-2>` is the last two lowercase hexadecimal characters of the canonical
UUID excluding hyphens. The authoritative path is therefore a function of UUID
alone; kind and time are envelope data, never identity routing inputs. The
filename UUID and envelope UUID must agree. Exact lookup and create do not scan
the corpus or consult a mutable identity index.

Creation uses a shared complete-content atomic no-clobber primitive: validate or
create parent components without following squatters, write and close a reserved
sibling temporary file, publish the complete inode through a hard link to the
create-only destination, then remove the temporary name after publication or
collision. A caller-stable UUID replays successfully only when kind, typed
payload, and explicit facets express the same caller intent; first-write time
and automatic enrichment remain frozen.
Different intent at the same UUID is an identity collision. Different UUIDs are
never content-deduplicated. Reserved temporary names are ignored by loading and
may be cleaned after interruption.

The create operation returns a receipt and performs no stage, commit, push, index,
triage, or promotion action. Authoritative records are authored collection data
by default. A project may ignore them to reduce review noise, but then accepts
local-only durability and loss of shared correlation unless another transport
exists. A possible `by-month/<year>/<month>/<uuid>.toml` relative-symlink view
is reserved follow-up direction: V1 creates and installs no such view or ignore
pattern, and capture and query never trust it.

### Resolution and query

Supersession targets an existing primary, kind-compatible replacement;
retraction targets an exact primary identity. Controls cannot target controls.
Each correction writes one control record and never edits its target. Resolution
is a pure fold into exact, active, and historical views.

Controls are validated and considered independently in canonical
`(recorded_at, uid)` order. Malformed, dangling, kind-incompatible,
cycle-introducing, and losing conflicting controls are individually diagnostic
and inert. Repeated retractions and repeated supersessions to the same
replacement are idempotent. Retraction dominates supersession for one target;
among distinct successors, the earliest valid supersession is effective. An
invalid later control cannot resurrect an observation or cancel an earlier
valid correction.

Exact lookup addresses any UUID regardless of current state. Collection queries
default to the active projection and can request history. Filters operate on kind,
time, and registered facet fields. Search reuses the shared tokenizer and
case-folding rules; every query token must occur somewhere in summary, detail,
or string facet values. Matching is Boolean and unranked. Results order by
`recorded_at` descending and UUID, with an opaque keyset cursor over that pair.
Head inserts do not duplicate or shift traversed rows; no frozen corpus snapshot
is promised.

Resolved exact lookup follows effective supersession edges transitively to the
first record without an effective successor. A retracted terminus remains the
resolved terminus and is rendered as retracted with its correction chain.
Corrections are intentionally irreversible through the V1 product surface:
history and exact lookup remain complete, while active-view recovery from a
mistaken control requires exceptional manual removal outside the capability.

Observations use bare UUID as their canonical identity and are not entity kinds.
Their CLI follows the shared `<kind> <verb>` and table/JSON conventions but does
not flatten SPEC-013's entity `CommonListArgs` or join its entity
list-conformance matrix.

### Interface and enrichment

The trusted CLI owns friction capture, exact/resolved reads, list, search,
supersession, and retraction. Its shell supplies the repository root, clock,
default UUID, rendering, and allowlisted enrichment inputs to the shared
observation service.

The MCP server exposes structured `observation_record` capture through the same
service. For confined workers it accepts friction only, resolves the registered
primary repository root server-side, accepts no arbitrary path, and refuses
measurement and correction controls. It is a bounded broker, not a general
filesystem capability.

Measurement creation is a closed service operation available only to a
registered machine source that supplies source, scope, units, completeness,
and supported counters. The V1 registry is only the trusted-source admission
boundary, not a harness extraction API. With an empty production registry, no
measurement can be created. Agent or operator assertion of source metadata does
not constitute registration. QUE-176 and the first instrumentation slice own
the concrete harness adapter interface.

Automatic enrichment considers only named adapter-known sources: CLI/MCP
`interface`, `product_surface`, and `command` constants map to those execution
fields; established primary-versus-worker context maps to
`execution.repository_context`; and an opaque agent identifier already supplied
through capture context maps to `correlation.agent_id`. Explicit values replace
automatic values field by field. Missing or failed automatic enrichment warns
and proceeds; invalid explicit data fails before creation. Harness, model, role,
arm, stage, skill, and run correlation are absent unless explicitly supplied or
known by a trusted adapter. General environment inspection is excluded.

Dogfood guidance is capability-aware: primary-tree agents use CLI, confined
Claude workers use MCP, and workers without a broker do not write
`.doctrine/**` in their fork; an orchestrator may proxy their reported friction.
The existing worker-mode guard classifies record, supersede, and retract as
Write, and show, list, and search as Read. A solo agent in a marked worktree
defers the signal through its runtime phase sheet or handoff and records it
after returning to the coordination tree.

## Concerns

- **Capture cost.** Required data must remain minimal. Enrichment failure cannot
  turn incidental recording into a second task.
- **Hostile content.** Every caller-authored string is untrusted at TOML, JSON,
  terminal, agent-context, and MCP boundaries. Bounds, structured serialization,
  escaped display, and untrusted-data framing prevent structural breakout,
  control-sequence injection, and instruction confusion.
- **Confinement.** The MCP adapter crosses a worker filesystem wall. Server-side
  root resolution, closed kinds, strict schemas, and containment checks are the
  security boundary.
- **Partial corruption.** One malformed or future-schema file must not deny reads
  of the remaining corpus. Diagnostics must remain attached to exact paths and
  identities where recoverable.
- **Corpus growth and review noise.** Random-tail sharding bounds directory size,
  while automated retention and aggregate indexes remain outside this container.
  Committed raw records preserve shared evidence but may crowd reviews; ignoring
  them is an explicit project/local tradeoff, not the default.

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
  state; each invalid or losing control is visible and inert without cancelling
  valid controls.
- **D6 — one service, capability-narrow adapters.** CLI and MCP cannot diverge on
  friction creation semantics, while measurement creation requires a registered
  machine source and MCP exposes less authority.
- **D7 — pure core, one disk seam.** Wire validation, resolution, and query take
  injected facts; store is the only observation filesystem seam.
- **D8 — identity routes storage.** The authoritative path derives only from UUID;
  time and kind never require an index or scan to enforce global identity.
- **D9 — observation is an ADR-001 leaf.** The umbrella imports only leaves;
  command and MCP adapters remain in the command tier.
- **D10 — corrections are irreversible in V1.** Controls cannot target controls;
  exact/history views preserve mistakes, and active-view repair is an exceptional
  manual operation rather than another ledger control.
- **D11 — source admission precedes harness adapters.** The versioned
  measurement wire and closed registered-source check prevent generic writers
  from becoming a trust API; QUE-176 and the first instrumentation slice own
  the concrete harness adapter interface.
